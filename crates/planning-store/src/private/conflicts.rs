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

/// Walks the Synchronization Folder one level, then the whole `planning-db/` tree.
/// Drive conflict copies of WAL segments live under `planning-db/wal/`, not next
/// to `writer.lock`.
pub fn scan(sync_folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_from(sync_folder, &mut found);
    collect_tree(&sync_folder.join("planning-db"), &mut found);
    found
}

fn collect_tree(directory: &Path, found: &mut Vec<PathBuf>) {
    collect_from(directory, found);
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tree(&path, found);
        }
    }
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
        assert_eq!(
            found.len(),
            2,
            "clean files must not be reported: {found:?}"
        );
    }

    #[test]
    fn conflict_copies_inside_the_database_tree_are_detected() {
        let folder = TempDir::new().unwrap();
        let wal = folder.path().join("planning-db").join("wal");
        std::fs::create_dir_all(&wal).unwrap();
        std::fs::write(wal.join("00000000000000000000.wal (1)"), "").unwrap();

        let found: Vec<String> = scan(folder.path())
            .iter()
            .filter_map(|path| path.file_name()?.to_str().map(str::to_string))
            .collect();
        assert_eq!(found, vec!["00000000000000000000.wal (1)".to_string()]);
    }

    #[test]
    fn a_clean_folder_reports_nothing() {
        let folder = TempDir::new().unwrap();
        std::fs::create_dir(folder.path().join("planning-db")).unwrap();
        assert!(scan(folder.path()).is_empty());
    }
}
