use super::error::AppError;
use super::service::{PlanningApp, StartRequest};
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
            Ok(app) => {
                if still_waiting_for_engine_lock(&app) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                return app;
            }
            Err(AppError::Store(planning_store::StoreError::Database(_))) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => panic!("{error:?}"),
        }
    }
    panic!("database lock did not clear after restart");
}

fn still_waiting_for_engine_lock(app: &PlanningApp) -> bool {
    let StoreHealth::Unreadable { detail } = app.health() else {
        return false;
    };
    detail.contains("os error 32")
        || detail.contains("os error 33")
        || detail.contains("being used by another process")
        || detail.contains("bloqueada una parte del archivo")
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
async fn a_wal_the_engine_rejects_is_unreadable_not_a_failed_start() {
    let home = TempDir::new().unwrap();
    let drive = TempDir::new().unwrap();
    let wal = drive.path().join("planning-db").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join("not-a-segment"), "junk").unwrap();

    let mut app = app(&home).await;
    let health = app
        .choose_sync_folder(drive.path().to_path_buf())
        .await
        .expect("start and reconnect must succeed even when the engine cannot open");
    assert!(
        matches!(health, StoreHealth::Unreadable { .. }),
        "got {health:?}"
    );
}

#[tokio::test]
async fn a_conflict_copy_inside_wal_is_reported_without_opening() {
    let home = TempDir::new().unwrap();
    let drive = TempDir::new().unwrap();
    let wal = drive.path().join("planning-db").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join("notes (1).txt"), "drive copy").unwrap();

    let mut app = app(&home).await;
    let health = app
        .choose_sync_folder(drive.path().to_path_buf())
        .await
        .unwrap();
    assert!(
        matches!(health, StoreHealth::SyncConflict { .. }),
        "got {health:?}"
    );
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
