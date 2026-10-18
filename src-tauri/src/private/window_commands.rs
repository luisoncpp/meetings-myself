use tauri_plugin_dialog::DialogExt;

/// Opens a native folder picker for the first-run sync folder step.
#[tauri::command]
pub async fn pick_sync_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string()))
}

/// Lists every IANA zone the app accepts for the home calendar.
#[tauri::command]
pub fn available_time_zones() -> Result<Vec<String>, String> {
    Ok(chrono_tz::TZ_VARIANTS
        .iter()
        .map(|zone| zone.to_string())
        .collect())
}
