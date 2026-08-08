use std::path::{Path, PathBuf};

/// Google Drive for desktop renames a conflicting file rather than merging it.
/// Either pattern inside the Synchronization Folder means the database may be
/// torn, so the app must refuse to write (ADR 0001).
fn is_conflict_artifact(name: &str) -> bool {
    if name.contains("(conflicted copy") {
        return true;
    }
    // " (1)", " (2)" ... appended before or instead of an extension.
    let Some(open) = name.rfind(" (") else {
        return false;
    };
    let rest = &name[open + 2..];
    let Some(close) = rest.find(')') else {
        return false;
    };
    !rest[..close].is_empty() && rest[..close].chars().all(|c| c.is_ascii_digit())
}

/// Walks the Synchronization Folder one level deep plus the database directory.
pub fn scan(sync_folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in [sync_folder.to_path_buf(), sync_folder.join("planning-db")] {
        collect_from(&root, &mut found);
    }
    found
}

fn collect_from(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_conflict_artifact(name) {
            continue;
        }
        found.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn google_drive_conflict_artifacts_are_detected() {
        let folder = TempDir::new().unwrap();
        for name in [
            "planning-db",
            "CURRENT (1)",
            "MANIFEST-000004 (conflicted copy 2026-08-06)",
            "writer.lock",
        ] {
            std::fs::write(folder.path().join(name), "").unwrap();
        }

        let found: Vec<String> = scan(folder.path())
            .iter()
            .filter_map(|path| path.file_name()?.to_str().map(str::to_string))
            .collect();

        assert!(found.contains(&"CURRENT (1)".to_string()));
        assert!(found.contains(&"MANIFEST-000004 (conflicted copy 2026-08-06)".to_string()));
        assert_eq!(found.len(), 2, "clean files must not be reported: {found:?}");
    }

    #[test]
    fn a_clean_folder_reports_nothing() {
        let folder = TempDir::new().unwrap();
        std::fs::create_dir(folder.path().join("planning-db")).unwrap();
        assert!(scan(folder.path()).is_empty());
    }
}
