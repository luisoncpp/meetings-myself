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
