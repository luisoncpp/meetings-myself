use super::*;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Tz;
use planning_core::FixedClock;
use planning_store::{SetupGap, StoreHealth};
use std::sync::Arc;
use tempfile::TempDir;

fn clock() -> Arc<dyn planning_core::Clock> {
    Arc::new(FixedClock::at(
        Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap(),
    ))
}

async fn app(home: &TempDir) -> PlanningApp {
    PlanningApp::start(StartRequest {
        settings_path: home.path().join("device-settings.json"),
        clock: clock(),
    })
    .await
    .unwrap()
}

async fn app_after_restart(home: &TempDir) -> PlanningApp {
    let settings_path = home.path().join("device-settings.json");
    let clock = clock();
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
    panic!("database lock did not clear after restart");
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
async fn reconnect_recovers_when_the_folder_reappears() {
    let home = TempDir::new().unwrap();
    let drive = TempDir::new().unwrap();
    let path = drive.path().to_path_buf();
    {
        let mut app = app(&home).await;
        app.choose_sync_folder(path.clone()).await.unwrap();
        app.set_home_zone(Tz::Europe__Madrid).await.unwrap();
    }
    let parked = home.path().join("parked-sync");
    std::fs::rename(&path, &parked).unwrap();
    let mut restarted = app_after_restart(&home).await;
    assert!(matches!(
        restarted.health(),
        StoreHealth::FolderMissing { .. }
    ));
    std::fs::rename(&parked, &path).unwrap();
    for _ in 0..20 {
        match restarted.reconnect().await {
            Ok(health) => {
                assert_eq!(health, StoreHealth::Ready);
                return;
            }
            Err(AppError::Store(planning_store::StoreError::Database(_))) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => panic!("{error:?}"),
        }
    }
    panic!("database lock did not clear after the folder reappeared");
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
