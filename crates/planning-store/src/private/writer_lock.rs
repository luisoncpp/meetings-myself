use super::device_settings::DeviceSettings;
use super::health::StoreHealth;
use chrono::{DateTime, Duration, Utc};
use planning_core::Clock;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LockRecord {
    device_id: String,
    device_name: String,
    heartbeat_at: DateTime<Utc>,
}

pub struct AcquireLock<'a> {
    pub settings: &'a DeviceSettings,
    pub clock: &'a dyn Clock,
}

/// Advisory one-active-writer lock. Google Drive gives no real file locking, so
/// this is a cooperative marker with a heartbeat: a device that crashed leaves a
/// stale record that another device may take over after STALE_AFTER_MINUTES.
#[derive(Debug)]
pub struct WriterLock {
    path: PathBuf,
    device_id: String,
}

impl WriterLock {
    pub const FILE_NAME: &'static str = "writer.lock";
    pub const STALE_AFTER_MINUTES: i64 = 15;

    pub fn acquire(sync_folder: &Path, request: AcquireLock<'_>) -> Result<Self, StoreHealth> {
        let path = sync_folder.join(Self::FILE_NAME);
        if let Some(holder) = read_lock(&path) {
            let ours = holder.device_id == request.settings.device_id;
            let age = request.clock.now() - holder.heartbeat_at;
            if !ours && age < Duration::minutes(Self::STALE_AFTER_MINUTES) {
                return Err(StoreHealth::LockedByAnotherDevice {
                    device_name: holder.device_name,
                    since: holder.heartbeat_at,
                });
            }
        }
        let lock = Self {
            path,
            device_id: request.settings.device_id.clone(),
        };
        lock.write(request.settings, request.clock)?;
        Ok(lock)
    }

    /// Call periodically while the app is open so another device can tell the
    /// difference between "in use" and "crashed".
    pub fn heartbeat(&self, settings: &DeviceSettings, clock: &dyn Clock) {
        let _ = self.write(settings, clock);
    }

    pub fn release(self) {
        drop(self);
    }

    fn write(&self, settings: &DeviceSettings, clock: &dyn Clock) -> Result<(), StoreHealth> {
        let record = LockRecord {
            device_id: self.device_id.clone(),
            device_name: settings.device_name.clone(),
            heartbeat_at: clock.now(),
        };
        let text = serde_json::to_string(&record)
            .map_err(|error| StoreHealth::Unreadable { detail: error.to_string() })?;
        std::fs::write(&self.path, text)
            .map_err(|error| StoreHealth::Unreadable { detail: error.to_string() })
    }
}

impl Drop for WriterLock {
    fn drop(&mut self) {
        // Best effort: a lost lock file only means the next device waits out the
        // staleness window. Never panic on shutdown.
        let _ = std::fs::remove_file(&self.path);
    }
}

fn read_lock(path: &Path) -> Option<LockRecord> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use planning_core::FixedClock;
    use tempfile::TempDir;

    fn settings(name: &str) -> DeviceSettings {
        DeviceSettings {
            device_name: name.to_string(),
            ..DeviceSettings::default()
        }
    }

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    #[test]
    fn a_second_device_is_refused_while_the_lock_is_fresh() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let desktop = settings("desktop");
        let clock = clock();

        let _held = WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &laptop,
                clock: &clock,
            },
        )
        .expect("first acquire succeeds");

        let refused = WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &desktop,
                clock: &clock,
            },
        )
        .unwrap_err();

        assert!(matches!(
            refused,
            StoreHealth::LockedByAnotherDevice { ref device_name, .. } if device_name == "laptop"
        ));
    }

    #[test]
    fn a_stale_lock_can_be_taken_over() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let desktop = settings("desktop");
        let clock = clock();

        let held = WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &laptop,
                clock: &clock,
            },
        )
        .unwrap();
        std::mem::forget(held); // simulate a crash: the lock file is left behind

        clock.advance(Duration::minutes(WriterLock::STALE_AFTER_MINUTES + 1));
        assert!(WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &desktop,
                clock: &clock
            }
        )
        .is_ok());
    }

    #[test]
    fn the_same_device_reacquires_its_own_lock() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let clock = clock();

        let held = WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &laptop,
                clock: &clock,
            },
        )
        .unwrap();
        std::mem::forget(held);

        assert!(WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &laptop,
                clock: &clock
            }
        )
        .is_ok());
    }

    #[test]
    fn releasing_removes_the_lock_file() {
        let folder = TempDir::new().unwrap();
        let laptop = settings("laptop");
        let clock = clock();

        let held = WriterLock::acquire(
            folder.path(),
            AcquireLock {
                settings: &laptop,
                clock: &clock,
            },
        )
        .unwrap();
        assert!(folder.path().join(WriterLock::FILE_NAME).exists());
        held.release();
        assert!(!folder.path().join(WriterLock::FILE_NAME).exists());
    }
}
