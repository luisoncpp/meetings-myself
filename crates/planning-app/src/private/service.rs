use super::error::AppError;
use chrono_tz::Tz;
use planning_core::{Clock, HomeCalendar};
use planning_store::{
    AcquireLock, Assessment, Database, DeviceSettings, DeviceSettingsFile, StoreHealth, WriterLock,
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

    #[allow(dead_code)] // used by domain use cases in plan 0004
    pub(crate) fn require_database(&self) -> Result<&Database, AppError> {
        if !self.health.permits_writes() {
            return Err(AppError::NotReady(self.health.clone()));
        }
        self.database.as_ref().ok_or(AppError::NoDatabase)
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
