use crate::private::service::{PlanningApp, StartRequest};
use chrono::{DateTime, TimeZone, Utc};
use planning_core::FixedClock;
use std::sync::Arc;
use tempfile::TempDir;

/// A fully set-up app at a caller-supplied instant.
pub async fn ready_app_at(
    instant: DateTime<Utc>,
) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
    let home = TempDir::new().unwrap();
    let drive = TempDir::new().unwrap();
    let clock = Arc::new(FixedClock::at(instant));
    let mut app = PlanningApp::start(StartRequest {
        settings_path: home.path().join("device-settings.json"),
        clock: clock.clone(),
    })
    .await
    .unwrap();
    app.choose_sync_folder(drive.path().to_path_buf())
        .await
        .unwrap();
    app.set_home_zone(chrono_tz::Tz::Europe__Madrid)
        .await
        .unwrap();
    (home, drive, app, clock)
}

/// A fully set-up app: device settings in one temp dir, sync folder in another.
pub async fn ready_app() -> (TempDir, TempDir, PlanningApp) {
    let (home, drive, app, _clock) = ready_app_at(
        Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap(),
    )
    .await;
    (home, drive, app)
}
