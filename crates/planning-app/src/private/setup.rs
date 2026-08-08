use super::error::AppError;
use super::service::{PlanningApp, StartRequest};
use chrono_tz::Tz;
use planning_store::{Database, DeviceSettingsFile, HomeSettingsRepository, SetZone, StoreHealth};
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

    /// Re-runs the whole open sequence. Called at start, after choosing a folder,
    /// and by plan 0008 when synchronization recovers.
    pub async fn reconnect(&mut self) -> Result<StoreHealth, AppError> {
        self.lock = None;
        self.database = None;
        self.home_zone = None;

        let Some(folder) = self.settings.sync_folder.clone() else {
            self.health = self.assess();
            return Ok(self.health());
        };
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
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Utc;
    use chrono_tz::Tz;
    use planning_core::FixedClock;
    use planning_store::{SetupGap, StoreHealth};
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app(home: &TempDir) -> PlanningApp {
        let settings_path = home.path().join("device-settings.json");
        let clock = Arc::new(FixedClock::at(
            Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap(),
        ));
        PlanningApp::start(StartRequest {
            settings_path,
            clock,
        })
        .await
        .unwrap()
    }

    async fn app_after_restart(home: &TempDir) -> PlanningApp {
        let settings_path = home.path().join("device-settings.json");
        let clock: Arc<dyn planning_core::Clock> = Arc::new(FixedClock::at(
            Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap(),
        ));
        for _ in 0..20 {
            match PlanningApp::start(StartRequest {
                settings_path: settings_path.clone(),
                clock: Arc::clone(&clock),
            })
            .await
            {
                Ok(app) => return app,
                Err(AppError::Store(planning_store::StoreError::Database(_))) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(error) => panic!("{error:?}"),
            }
        }
        panic!("RocksDB lock did not clear after restart");
    }

    #[tokio::test]
    async fn a_fresh_install_reports_no_sync_folder() {
        let home = TempDir::new().unwrap();
        let app = app(&home).await;
        assert!(matches!(
            app.health(),
            StoreHealth::SetupIncomplete {
                reason: SetupGap::NoSyncFolder
            }
        ));
        assert!(
            app.calendar().is_err(),
            "no calendar before setup completes"
        );
    }

    #[tokio::test]
    async fn setup_completes_once_a_folder_and_a_zone_are_chosen() {
        let home = TempDir::new().unwrap();
        let drive = TempDir::new().unwrap();
        let mut app = app(&home).await;

        let after_folder = app
            .choose_sync_folder(drive.path().to_path_buf())
            .await
            .unwrap();
        assert!(matches!(
            after_folder,
            StoreHealth::SetupIncomplete {
                reason: SetupGap::NoHomeZone
            }
        ));

        let after_zone = app.set_home_zone(Tz::Europe__Madrid).await.unwrap();
        assert_eq!(after_zone, StoreHealth::Ready);
        assert_eq!(app.calendar().unwrap().zone(), Tz::Europe__Madrid);
    }

    #[tokio::test]
    async fn the_chosen_folder_survives_a_restart() {
        let home = TempDir::new().unwrap();
        let drive = TempDir::new().unwrap();
        {
            let mut app = app(&home).await;
            app.choose_sync_folder(drive.path().to_path_buf())
                .await
                .unwrap();
            app.set_home_zone(Tz::Europe__Madrid).await.unwrap();
        }
        let restarted = app_after_restart(&home).await;
        assert_eq!(restarted.health(), StoreHealth::Ready);
        assert_eq!(restarted.calendar().unwrap().zone(), Tz::Europe__Madrid);
    }
}
