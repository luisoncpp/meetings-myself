use planning_app::{
    app_error_payload, AppError, AppErrorPayload, PlanningApp, StoreHealth, UiLanguage,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Managed state. A Mutex, not a RwLock: setup mutates, and the app is
/// single-user by construction.
pub struct AppState(pub Arc<Mutex<PlanningApp>>);

pub(super) fn app_error_message(error: AppError) -> String {
    app_error_payload(error).to_ipc_string()
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
pub async fn ui_language(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let language = state.0.lock().await.ui_language();
    Ok(match language {
        UiLanguage::En => "en".to_string(),
        UiLanguage::Es => "es".to_string(),
    })
}

#[tauri::command]
pub async fn set_ui_language(
    state: tauri::State<'_, AppState>,
    language: String,
) -> Result<(), String> {
    let parsed = match language.as_str() {
        "en" => UiLanguage::En,
        "es" => UiLanguage::Es,
        _ => {
            return Err(AppErrorPayload {
                code: "invalidUiLanguage".into(),
                params: Some(std::collections::HashMap::from([(
                    "language".into(),
                    language,
                )])),
            }
            .to_ipc_string())
        }
    };
    state
        .0
        .lock()
        .await
        .set_ui_language(parsed)
        .map_err(app_error_message)
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

/// Re-opens the sync folder after Drive mounts or a lock clears.
#[tauri::command]
pub async fn reconnect_store(state: tauri::State<'_, AppState>) -> Result<StoreHealth, String> {
    state
        .0
        .lock()
        .await
        .reconnect()
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
