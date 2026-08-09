use tauri::utils::config::WindowConfig;
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;

pub(crate) const WEEKLY_REVIEW_LABEL: &str = "weekly-review";

/// Hides the Weekly Review window when the user closes it so the webview stays alive.
pub fn attach_weekly_review_lifecycle(app: &AppHandle) -> Result<(), String> {
    let review = app
        .get_webview_window(WEEKLY_REVIEW_LABEL)
        .ok_or("the weekly-review window is not configured")?;
    review.clone().on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = review.hide();
        }
    });
    Ok(())
}

/// Shows the Weekly Review window, recreating it from `tauri.conf.json` when it was closed.
#[tauri::command]
pub async fn open_weekly_review_window(app: AppHandle) -> Result<(), String> {
    let window = match app.get_webview_window(WEEKLY_REVIEW_LABEL) {
        Some(window) => window,
        None => create_weekly_review_window(&app)?,
    };
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

fn weekly_review_window_config(app: &AppHandle) -> Result<WindowConfig, String> {
    weekly_review_config_from_windows(&app.config().app.windows)
}

pub(crate) fn weekly_review_config_from_windows(
    windows: &[WindowConfig],
) -> Result<WindowConfig, String> {
    windows
        .iter()
        .find(|window| window.label == WEEKLY_REVIEW_LABEL)
        .cloned()
        .ok_or_else(|| "the weekly-review window is not configured".to_string())
}

fn create_weekly_review_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    let config = weekly_review_window_config(app)?;
    WebviewWindowBuilder::from_config(app, &config)
        .map_err(|error| error.to_string())?
        .build()
        .map_err(|error| error.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_windows() -> Vec<WindowConfig> {
        vec![
            WindowConfig {
                label: "main".into(),
                ..Default::default()
            },
            WindowConfig {
                label: WEEKLY_REVIEW_LABEL.into(),
                url: tauri::utils::config::WebviewUrl::App(
                    "index.html?surface=weekly-review".into(),
                ),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn weekly_review_config_is_found_among_app_windows() {
        let config = weekly_review_config_from_windows(&sample_windows()).expect("config");
        assert_eq!(config.label, WEEKLY_REVIEW_LABEL);
    }

    #[test]
    fn weekly_review_config_errors_when_window_is_missing() {
        let windows = vec![WindowConfig {
            label: "main".into(),
            ..Default::default()
        }];
        let error = weekly_review_config_from_windows(&windows).expect_err("missing config");
        assert!(error.contains("weekly-review"));
    }
}
