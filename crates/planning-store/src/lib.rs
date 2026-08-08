//! Persistence and device settings. The Synchronization Folder lives here.

mod private;

pub use private::database::Database;
pub use private::device_settings::{DeviceSettings, DeviceSettingsFile};
pub use private::error::StoreError;
pub use private::home_settings::{HomeSettings, HomeSettingsRepository, SetZone};
