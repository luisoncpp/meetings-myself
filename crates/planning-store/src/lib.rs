//! Persistence for the planning domain: the embedded database, device settings,
//! and the sync-safety gate. Nothing here knows what a Task is.

mod private;

pub use private::database::Database;
pub use private::device_settings::{DeviceSettings, DeviceSettingsFile};
pub use private::error::StoreError;
pub use private::health::{Assessment, SetupGap, StoreHealth};
pub use private::home_settings::{HomeSettings, HomeSettingsRepository, SetZone};
pub use private::records::{RecordKey, Records};
pub use private::ui_language::UiLanguage;
pub use private::writer_lock::{AcquireLock, WriterLock};
