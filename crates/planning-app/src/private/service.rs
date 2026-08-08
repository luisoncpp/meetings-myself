use super::error::AppError;
use chrono_tz::Tz;
use planning_core::{
    Clock, Goal, GoalId, Habit, HabitId, HomeCalendar, Task, TaskId, Value, ValueId,
};
use planning_store::{
    AcquireLock, Assessment, Database, DeviceSettings, DeviceSettingsFile, RecordKey, Records,
    StoreHealth, WriterLock,
};
use std::path::PathBuf;
use std::sync::Arc;

pub struct StartRequest {
    pub settings_path: PathBuf,
    pub clock: Arc<dyn Clock>,
}

/// The application API. `src-tauri` and `launcher` depend on this and nothing else.
pub struct PlanningApp {
    pub(crate) settings_file: DeviceSettingsFile,
    pub(crate) settings: DeviceSettings,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) database: Option<Database>,
    pub(crate) home_zone: Option<Tz>,
    pub(crate) health: StoreHealth,
    pub(crate) lock: Option<WriterLock>,
}

impl PlanningApp {
    pub fn health(&self) -> StoreHealth {
        self.health.clone()
    }

    /// Available only once setup is complete — this is what makes it impossible
    /// to compute a date before the home zone is known.
    pub fn calendar(&self) -> Result<HomeCalendar, AppError> {
        let zone = self
            .home_zone
            .ok_or(AppError::NotReady(self.health.clone()))?;
        if !self.health.permits_writes() {
            return Err(AppError::NotReady(self.health.clone()));
        }
        Ok(HomeCalendar::new(zone))
    }

    pub(crate) fn require_database(&self) -> Result<&Database, AppError> {
        if !self.health.permits_writes() {
            return Err(AppError::NotReady(self.health.clone()));
        }
        self.database.as_ref().ok_or(AppError::NoDatabase)
    }

    /// Saves one record, refusing unless the store is Ready.
    pub(crate) async fn store<T>(&self, table: &str, id: &str, record: &T) -> Result<(), AppError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let database = self.require_database()?;
        Records::save(database, RecordKey { table, id }, record).await?;
        Ok(())
    }

    pub(crate) async fn load_all<T>(&self, table: &str) -> Result<Vec<T>, AppError>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let database = self.require_database()?;
        Ok(Records::all(database, table).await?)
    }

    pub(crate) async fn load_one<T>(&self, table: &str, id: &str) -> Result<Option<T>, AppError>
    where
        T: serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        let database = self.require_database()?;
        Ok(Records::find(database, RecordKey { table, id }).await?)
    }

    /// Loads a record, applies `change`, and saves it back. The single
    /// read-modify-write path, so "not found" is handled in exactly one place.
    pub(crate) async fn mutate<T>(
        &self,
        key: (&'static str, String),
        change: impl FnOnce(&mut T),
    ) -> Result<T, AppError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
    {
        let (table, id) = key;
        let mut record: T = self.load_one(table, &id).await?.ok_or(AppError::NotFound {
            table,
            id: id.clone(),
        })?;
        change(&mut record);
        self.store(table, &id, &record).await?;
        Ok(record)
    }

    pub async fn values(&self) -> Result<Vec<Value>, AppError> {
        self.load_all(ValueId::TABLE).await
    }

    pub async fn goals(&self) -> Result<Vec<Goal>, AppError> {
        self.load_all(GoalId::TABLE).await
    }

    pub async fn tasks(&self) -> Result<Vec<Task>, AppError> {
        self.load_all(TaskId::TABLE).await
    }

    pub async fn habits(&self) -> Result<Vec<Habit>, AppError> {
        self.load_all(HabitId::TABLE).await
    }

    pub(crate) fn assess(&self) -> StoreHealth {
        StoreHealth::assess(Assessment {
            sync_folder: self.settings.sync_folder.clone(),
            home_zone_is_set: self.home_zone.is_some(),
        })
    }

    pub(crate) fn take_lock(&mut self) {
        let Some(folder) = self.settings.sync_folder.clone() else {
            return;
        };
        if !self.health.permits_writes() {
            return;
        }
        match WriterLock::acquire(
            &folder,
            AcquireLock {
                settings: &self.settings,
                clock: self.clock.as_ref(),
            },
        ) {
            Ok(lock) => self.lock = Some(lock),
            Err(blocked) => self.health = blocked,
        }
    }
}
