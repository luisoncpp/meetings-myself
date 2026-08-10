use super::error::StoreError;
use super::ui_language::UiLanguage;
use chrono::{NaiveDate, NaiveTime};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Per-device configuration. Deliberately stored OUTSIDE the Synchronization
/// Folder: launch time, retry window, and folder path are device facts and must
/// never travel between machines (ADR 0001).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceSettings {
    pub device_id: String,
    pub device_name: String,
    pub sync_folder: Option<PathBuf>,
    pub launch_time: NaiveTime,
    pub retry_window_minutes: u32,
    pub last_missed_prompt: Option<NaiveDate>,
    #[serde(default)]
    pub ui_language: UiLanguage,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            device_id: uuid::Uuid::now_v7().to_string(),
            device_name: hostname(),
            sync_folder: None,
            launch_time: NaiveTime::from_hms_opt(7, 0, 0).expect("07:00 is a valid time"),
            retry_window_minutes: 240,
            last_missed_prompt: None,
            ui_language: UiLanguage::detect_default(),
        }
    }
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-device".to_string())
}

pub struct DeviceSettingsFile {
    path: PathBuf,
}

impl DeviceSettingsFile {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The OS config directory — never the Synchronization Folder.
    pub fn default_path() -> Result<PathBuf, StoreError> {
        let dirs = ProjectDirs::from("com", "gamecoderstudios", "self-planning")
            .ok_or(StoreError::NoConfigDirectory)?;
        Ok(dirs.config_dir().join("device-settings.json"))
    }

    /// Reads settings, creating defaults on first run so callers never handle "missing".
    pub fn load(&self) -> Result<DeviceSettings, StoreError> {
        if !self.path.exists() {
            let settings = DeviceSettings::default();
            self.save(&settings)?;
            return Ok(settings);
        }
        let text = std::fs::read_to_string(&self.path)?;
        serde_json::from_str(&text).map_err(|error| StoreError::Corrupt {
            path: self.path.clone(),
            detail: error.to_string(),
        })
    }

    pub fn save(&self, settings: &DeviceSettings) -> Result<(), StoreError> {
        create_parent(&self.path)?;
        let text = serde_json::to_string_pretty(settings).map_err(|error| StoreError::Corrupt {
            path: self.path.clone(),
            detail: error.to_string(),
        })?;
        std::fs::write(&self.path, text)?;
        Ok(())
    }
}

fn create_parent(path: &Path) -> Result<(), StoreError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn first_load_creates_defaults_and_persists_a_stable_device_id() {
        let dir = TempDir::new().unwrap();
        let file = DeviceSettingsFile::at(dir.path().join("device-settings.json"));

        let first = file.load().unwrap();
        assert_eq!(first.launch_time, NaiveTime::from_hms_opt(7, 0, 0).unwrap());
        assert_eq!(first.retry_window_minutes, 240);
        assert_eq!(first.sync_folder, None);

        let second = file.load().unwrap();
        assert_eq!(
            second.device_id, first.device_id,
            "device id must survive reload"
        );
    }

    #[test]
    fn saved_settings_round_trip() {
        let dir = TempDir::new().unwrap();
        let file = DeviceSettingsFile::at(dir.path().join("device-settings.json"));

        let mut settings = file.load().unwrap();
        settings.sync_folder = Some(PathBuf::from("/drive/self-planning"));
        settings.launch_time = NaiveTime::from_hms_opt(6, 30, 0).unwrap();
        file.save(&settings).unwrap();

        let reloaded = file.load().unwrap();
        assert_eq!(reloaded.sync_folder, settings.sync_folder);
        assert_eq!(reloaded.launch_time, settings.launch_time);
    }

    #[test]
    fn a_corrupt_file_reports_its_path_instead_of_silently_resetting() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("device-settings.json");
        std::fs::write(&path, "{ not json").unwrap();

        let error = DeviceSettingsFile::at(path.clone()).load().unwrap_err();
        assert!(matches!(error, StoreError::Corrupt { path: p, .. } if p == path));
    }
}
