use tauri_plugin_dialog::DialogExt;

/// Shows the Weekly Review window, creating nothing — the window is declared in
/// tauri.conf.json so all Tauri APIs stay on the Rust side of the boundary.
#[tauri::command]
pub fn open_weekly_review_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let window = app
        .get_webview_window("weekly-review")
        .ok_or("the weekly-review window is not configured")?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

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
