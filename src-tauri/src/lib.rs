mod private;

use planning_app::{DeviceSettingsFile, PlanningApp, StartRequest, SystemClock};
use private::commands::{
    app_version, choose_sync_folder, reconnect, reconnect_store, set_home_zone, set_ui_language,
    store_health, ui_language, AppState,
};
use private::library_commands::{
    associations_for, create_goal, create_habit, create_task, create_value, library,
};
use private::lifecycle_commands::{
    achieve_goal, archive_entity, classify_task, complete_task, link, reopen_task, restore_entity,
    set_habit_cadence, set_habit_pinned, set_habit_strength, set_task_deadline, set_task_one_off,
    unachieve_goal, unlink,
};
use private::plan_commands::{
    add_habit_to_plan, add_to_focus, archive_recurring_task, create_recurring_task, quick_add_task,
    record_check_in, recurring_tasks, remove_from_focus, remove_from_plan, rename_recurring_task,
    reorder_plan, restore_recurring_task, select_into_plan, task_pool, today_view, weekly_focus,
    yesterday_view,
};
use private::review_commands::{
    open_current_review, open_weekly_review, report_path, save_reflection, weekly_summary,
};
use private::window_commands::{
    attach_window_lifecycle, available_time_zones, open_weekly_review_window, pick_sync_folder,
};
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
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Arc::new(tokio::sync::Mutex::new(app))))
        .setup(|app| {
            attach_window_lifecycle(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            store_health,
            reconnect,
            reconnect_store,
            ui_language,
            set_ui_language,
            choose_sync_folder,
            set_home_zone,
            library,
            create_value,
            create_task,
            create_goal,
            create_habit,
            archive_entity,
            restore_entity,
            complete_task,
            reopen_task,
            achieve_goal,
            unachieve_goal,
            classify_task,
            set_task_deadline,
            set_task_one_off,
            set_habit_cadence,
            set_habit_pinned,
            set_habit_strength,
            link,
            unlink,
            associations_for,
            today_view,
            yesterday_view,
            task_pool,
            select_into_plan,
            remove_from_plan,
            reorder_plan,
            add_habit_to_plan,
            quick_add_task,
            record_check_in,
            weekly_focus,
            add_to_focus,
            remove_from_focus,
            create_recurring_task,
            recurring_tasks,
            archive_recurring_task,
            restore_recurring_task,
            rename_recurring_task,
            open_weekly_review,
            open_current_review,
            save_reflection,
            weekly_summary,
            report_path,
            open_weekly_review_window,
            pick_sync_folder,
            available_time_zones,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Self-Planning application");
}
