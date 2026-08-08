//! The application API. The desktop binary and the launcher depend on this crate
//! and on nothing beneath it.

mod private;

pub use planning_core::{CalendarWeek, Clock, FixedClock, HomeCalendar, SystemClock};
pub use planning_store::{DeviceSettings, DeviceSettingsFile, SetupGap, StoreHealth};
pub use private::error::AppError;
pub use private::service::{PlanningApp, StartRequest};
