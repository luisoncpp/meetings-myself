use super::engine_sidecars;
use super::error::StoreError;
use std::path::Path;
use std::time::Duration;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

pub struct Database {
    inner: Surreal<Db>,
}

impl Database {
    pub const DIRECTORY: &'static str = "planning-db";
    const NAMESPACE: &'static str = "planning";
    const DATABASE: &'static str = "planning";

    pub async fn open(sync_folder: &Path) -> Result<Self, StoreError> {
        let path = sync_folder.join(Self::DIRECTORY);
        std::fs::create_dir_all(&path)?;
        engine_sidecars::strip(&path);
        let inner = connect(&path).await?;
        inner.use_ns(Self::NAMESPACE).use_db(Self::DATABASE).await?;
        Ok(Self { inner })
    }

    pub fn inner(&self) -> &Surreal<Db> {
        &self.inner
    }
}

async fn connect(path: &Path) -> Result<Surreal<Db>, StoreError> {
    let mut delay = Duration::from_millis(100);
    let mut last = None;
    for _ in 0..5 {
        match Surreal::new::<SurrealKv>(path.to_string_lossy().as_ref()).await {
            Ok(inner) => return Ok(inner),
            Err(error) if is_transient_lock(&error) => {
                last = Some(error);
                std::thread::sleep(delay);
                delay *= 2;
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err(last
        .expect("transient lock retries always store the last error")
        .into())
}

fn is_transient_lock(error: &surrealdb::Error) -> bool {
    let text = error.to_string();
    text.contains("being used by another process")
        || text.contains("os error 32")
        || text.contains("os error 33")
        || text.contains("bloqueada una parte del archivo")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use surrealdb::types::SurrealValue;
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq, SurrealValue)]
    struct Probe {
        note: String,
    }

    #[tokio::test]
    async fn a_record_written_by_one_instance_is_read_by_the_next() {
        let folder = TempDir::new().unwrap();

        let first = Database::open(folder.path()).await.unwrap();
        let _: Option<Probe> = first
            .inner()
            .create(("probe", "one"))
            .content(Probe {
                note: "hello".into(),
            })
            .await
            .unwrap();
        drop(first);
        // Embedded engines can release the on-disk lock asynchronously after drop.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let second = Database::open(folder.path()).await.unwrap();
        let found: Option<Probe> = second.inner().select(("probe", "one")).await.unwrap();
        assert_eq!(
            found,
            Some(Probe {
                note: "hello".into(),
            })
        );
    }

    #[tokio::test]
    async fn opening_creates_the_database_directory_under_the_sync_folder() {
        let folder = TempDir::new().unwrap();
        let _database = Database::open(folder.path()).await.unwrap();
        assert!(folder.path().join(Database::DIRECTORY).is_dir());
    }

    #[tokio::test]
    async fn opening_survives_windows_sidecar_files_in_the_wal() {
        let folder = TempDir::new().unwrap();
        let first = Database::open(folder.path()).await.unwrap();
        drop(first);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let wal = folder.path().join(Database::DIRECTORY).join("wal");
        std::fs::write(wal.join("desktop.ini"), "[.ShellClassInfo]\n").unwrap();

        Database::open(folder.path())
            .await
            .expect("SurrealKV must ignore Explorer sidecar files in wal/");
    }
}
