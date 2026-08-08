use super::conflicts;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SetupGap {
    NoSyncFolder,
    NoHomeZone,
}

/// Whether the synchronized data can be trusted right now. Only `Ready` permits
/// a write; every other value is a state the UI must show and the launcher must
/// treat as "do not open the app" (ADR 0001).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum StoreHealth {
    Ready,
    SetupIncomplete {
        reason: SetupGap,
    },
    FolderMissing {
        path: PathBuf,
    },
    LockedByAnotherDevice {
        device_name: String,
        since: DateTime<Utc>,
    },
    SyncConflict {
        artifacts: Vec<PathBuf>,
    },
    Unreadable {
        detail: String,
    },
}

pub struct Assessment {
    pub sync_folder: Option<PathBuf>,
    pub home_zone_is_set: bool,
}

impl StoreHealth {
    /// Ordered most-blocking first so the UI always shows the fault the user can
    /// actually act on.
    pub fn assess(assessment: Assessment) -> Self {
        let Some(folder) = assessment.sync_folder else {
            return Self::SetupIncomplete {
                reason: SetupGap::NoSyncFolder,
            };
        };
        if !folder.is_dir() {
            return Self::FolderMissing { path: folder };
        }
        let artifacts = conflicts::scan(&folder);
        if !artifacts.is_empty() {
            return Self::SyncConflict { artifacts };
        }
        if !assessment.home_zone_is_set {
            return Self::SetupIncomplete {
                reason: SetupGap::NoHomeZone,
            };
        }
        Self::Ready
    }

    pub fn permits_writes(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn setup_is_incomplete_without_a_sync_folder() {
        let health = StoreHealth::assess(Assessment {
            sync_folder: None,
            home_zone_is_set: false,
        });
        assert!(matches!(
            health,
            StoreHealth::SetupIncomplete {
                reason: SetupGap::NoSyncFolder
            }
        ));
    }

    #[test]
    fn a_missing_folder_is_reported_before_anything_else() {
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(PathBuf::from("/definitely/not/here")),
            home_zone_is_set: true,
        });
        assert!(matches!(health, StoreHealth::FolderMissing { .. }));
    }

    #[test]
    fn a_present_folder_without_a_home_zone_is_incomplete() {
        let folder = TempDir::new().unwrap();
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(folder.path().to_path_buf()),
            home_zone_is_set: false,
        });
        assert!(matches!(
            health,
            StoreHealth::SetupIncomplete {
                reason: SetupGap::NoHomeZone
            }
        ));
    }

    #[test]
    fn conflict_artifacts_block_readiness() {
        let folder = TempDir::new().unwrap();
        std::fs::write(folder.path().join("CURRENT (1)"), "").unwrap();
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(folder.path().to_path_buf()),
            home_zone_is_set: true,
        });
        assert!(matches!(health, StoreHealth::SyncConflict { .. }));
    }

    #[test]
    fn a_clean_configured_folder_is_ready() {
        let folder = TempDir::new().unwrap();
        let health = StoreHealth::assess(Assessment {
            sync_folder: Some(folder.path().to_path_buf()),
            home_zone_is_set: true,
        });
        assert_eq!(health, StoreHealth::Ready);
        assert!(health.permits_writes());
    }

    #[test]
    fn store_health_serializes_as_the_frontend_expects() {
        let json = serde_json::to_string(&StoreHealth::SetupIncomplete {
            reason: SetupGap::NoHomeZone,
        })
        .unwrap();
        assert_eq!(
            json,
            r#"{"status":"setupIncomplete","reason":{"kind":"NoHomeZone"}}"#
        );
    }
}
