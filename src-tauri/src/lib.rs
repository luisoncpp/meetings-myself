mod private;

use planning_app::{DeviceSettingsFile, PlanningApp, StartRequest, SystemClock};
use private::commands::{app_version, choose_sync_folder, set_home_zone, store_health, AppState};
use std::sync::Arc;

/// Builds and runs the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_path = DeviceSettingsFile::default_path().expect("device settings path");
    let app = tauri::async_runtime::block_on(PlanningApp::start(StartRequest {
        settings_path,
        clock: Arc::new(SystemClock),
    }))
    .expect("failed to start planning app");

    tauri::Builder::default()
        .manage(AppState(Arc::new(tokio::sync::Mutex::new(app))))
        .invoke_handler(tauri::generate_handler![
            app_version,
            store_health,
            choose_sync_folder,
            set_home_zone
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Self-Planning application");
}
