use super::database::Database;
use super::error::StoreError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct RecordKey<'a> {
    pub table: &'a str,
    pub id: &'a str,
}

/// The single persistence gateway. Every entity stores the same way, so there is
/// one implementation rather than six.
///
/// There is no `delete` and there never will be: nothing is hard-deleted (ADR 0002).
pub struct Records;

impl Records {
    pub async fn save<T>(
        database: &Database,
        key: RecordKey<'_>,
        record: &T,
    ) -> Result<(), StoreError>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        let mut data = to_json(record)?;
        strip_id(&mut data);
        let _saved: Option<Value> = database
            .inner()
            .upsert((key.table, key.id))
            .content(data)
            .await?;
        Ok(())
    }

    pub async fn find<T>(database: &Database, key: RecordKey<'_>) -> Result<Option<T>, StoreError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let result: Result<Option<Value>, surrealdb::Error> =
            database.inner().select((key.table, key.id)).await;
        match result {
            Ok(Some(mut value)) => {
                inject_id(&mut value, key.id);
                from_json_value(value).map(Some)
            }
            Ok(None) => Ok(None),
            Err(error) if record_missing(&error) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn all<T>(database: &Database, table: &str) -> Result<Vec<T>, StoreError>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let result: Result<Vec<Value>, surrealdb::Error> = database.inner().select(table).await;
        match result {
            Ok(rows) => rows.into_iter().map(decode_row).collect(),
            Err(error) if record_missing(&error) => Ok(vec![]),
            Err(error) => Err(error.into()),
        }
    }
}

fn to_json<T: Serialize>(record: &T) -> Result<Value, StoreError> {
    serde_json::to_value(record).map_err(|error| StoreError::Database(error.to_string()))
}

fn decode_row<T: DeserializeOwned>(mut value: Value) -> Result<T, StoreError> {
    normalize_id(&mut value);
    from_json_value(value)
}

fn from_json_value<T: DeserializeOwned>(value: Value) -> Result<T, StoreError> {
    serde_json::from_value(value).map_err(|error| StoreError::Database(error.to_string()))
}

fn strip_id(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    object.remove("id");
}

fn inject_id(value: &mut Value, id: &str) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    object.insert("id".to_string(), Value::String(id.to_string()));
}

fn normalize_id(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let Some(raw) = object.get("id").and_then(Value::as_str) else {
        return;
    };
    object.insert("id".to_string(), Value::String(parse_record_id(raw)));
}

fn parse_record_id(raw: &str) -> String {
    let segment = raw.split(':').nth(1).unwrap_or(raw);
    segment.trim_matches('`').to_string()
}

fn record_missing(error: &surrealdb::Error) -> bool {
    error.to_string().contains("does not exist")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Note {
        title: String,
    }

    async fn database() -> (TempDir, Database) {
        let folder = TempDir::new().unwrap();
        let database = Database::open(folder.path()).await.unwrap();
        (folder, database)
    }

    #[tokio::test]
    async fn records_round_trip_and_saving_twice_updates_rather_than_duplicates() {
        let (_folder, database) = database().await;
        let key = RecordKey {
            table: "note",
            id: "n1",
        };

        Records::save(
            &database,
            key,
            &Note {
                title: "first".into(),
            },
        )
        .await
        .unwrap();
        Records::save(
            &database,
            key,
            &Note {
                title: "second".into(),
            },
        )
        .await
        .unwrap();

        let found: Option<Note> = Records::find(&database, key).await.unwrap();
        assert_eq!(
            found,
            Some(Note {
                title: "second".into()
            })
        );

        let all: Vec<Note> = Records::all(&database, "note").await.unwrap();
        assert_eq!(all.len(), 1, "saving twice must not create a second record");
    }

    #[tokio::test]
    async fn a_missing_record_is_none_rather_than_an_error() {
        let (_folder, database) = database().await;
        let found: Option<Note> = Records::find(
            &database,
            RecordKey {
                table: "note",
                id: "absent",
            },
        )
        .await
        .unwrap();
        assert_eq!(found, None);
    }
}
