use planning_app::{AppError, PlanningApp, StoreHealth};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Managed state. A Mutex, not a RwLock: setup mutates, and the app is
/// single-user by construction.
pub struct AppState(pub Arc<Mutex<PlanningApp>>);

pub(super) fn app_error_message(error: AppError) -> String {
    error.to_string()
}

/// Proves the IPC bridge works end to end. Plan 0004 replaces this module's
/// contents with the real application commands.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn store_health(state: tauri::State<'_, AppState>) -> Result<StoreHealth, String> {
    Ok(state.0.lock().await.health())
}

#[tauri::command]
pub async fn choose_sync_folder(
    state: tauri::State<'_, AppState>,
    folder: PathBuf,
) -> Result<StoreHealth, String> {
    state
        .0
        .lock()
        .await
        .choose_sync_folder(folder)
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn set_home_zone(
    state: tauri::State<'_, AppState>,
    zone: String,
) -> Result<StoreHealth, String> {
    state
        .0
        .lock()
        .await
        .set_home_zone_name(&zone)
        .await
        .map_err(app_error_message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_the_cargo_manifest() {
        assert_eq!(app_version(), env!("CARGO_PKG_VERSION"));
    }
}
