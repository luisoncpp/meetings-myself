use super::error::StoreError;
use std::path::Path;
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
        let inner = Surreal::new::<SurrealKv>(path.to_string_lossy().as_ref()).await?;
        inner.use_ns(Self::NAMESPACE).use_db(Self::DATABASE).await?;
        Ok(Self { inner })
    }

    pub fn inner(&self) -> &Surreal<Db> {
        &self.inner
    }
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
}
