use super::error::AppError;
use super::service::{PlanningApp, StartRequest};
use chrono_tz::Tz;
use planning_reports::WeeklyReportFile;
use planning_store::{
    Database, DeviceSettingsFile, HomeSettingsRepository, SetZone, StoreHealth, UiLanguage,
};
use std::path::PathBuf;

impl PlanningApp {
    pub async fn start(request: StartRequest) -> Result<Self, AppError> {
        let settings_file = DeviceSettingsFile::at(request.settings_path);
        let settings = settings_file.load()?;
        let mut app = Self {
            settings_file,
            settings,
            clock: request.clock,
            database: None,
            reports: None,
            home_zone: None,
            health: StoreHealth::Unreadable {
                detail: "not opened".into(),
            },
            lock: None,
        };
        app.reconnect().await?;
        Ok(app)
    }

    pub async fn choose_sync_folder(&mut self, folder: PathBuf) -> Result<StoreHealth, AppError> {
        self.settings.sync_folder = Some(folder);
        self.settings_file.save(&self.settings)?;
        self.reconnect().await?;
        Ok(self.health())
    }

    pub async fn set_home_zone(&mut self, zone: Tz) -> Result<StoreHealth, AppError> {
        let database = self.database.as_ref().ok_or(AppError::NoDatabase)?;
        HomeSettingsRepository::set_zone(
            database,
            SetZone {
                zone,
                clock: self.clock.as_ref(),
            },
        )
        .await?;
        self.home_zone = Some(zone);
        self.health = self.assess();
        self.take_lock();
        Ok(self.health())
    }

    pub async fn set_home_zone_name(&mut self, zone: &str) -> Result<StoreHealth, AppError> {
        let parsed: Tz = zone
            .parse()
            .map_err(|_| AppError::InvalidZone(zone.to_string()))?;
        self.set_home_zone(parsed).await
    }

    pub fn ui_language(&self) -> UiLanguage {
        self.settings.ui_language
    }

    pub fn set_ui_language(&mut self, language: UiLanguage) -> Result<(), AppError> {
        self.settings.ui_language = language;
        self.settings_file.save(&self.settings)?;
        Ok(())
    }

    /// Re-runs the whole open sequence.
    pub async fn reconnect(&mut self) -> Result<StoreHealth, AppError> {
        self.lock = None;
        self.database = None;
        self.reports = None;
        self.home_zone = None;

        let Some(folder) = self.settings.sync_folder.clone() else {
            self.health = self.assess();
            return Ok(self.health());
        };
        self.reports = Some(WeeklyReportFile::at(folder.clone()));
        if folder.is_dir() {
            let database = Database::open(&folder).await?;
            self.home_zone = HomeSettingsRepository::load(&database).await?.home_zone;
            self.database = Some(database);
        }
        self.health = self.assess();
        self.take_lock();
        Ok(self.health())
    }
}

#[cfg(test)]
#[path = "setup_tests.rs"]
mod tests;
