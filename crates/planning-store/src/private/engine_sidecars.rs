use std::path::Path;

const ENGINE_SUBDIRS: &[&str] = &["wal", "sstables", "vlog", "manifest"];

/// SurrealKV lists every file in `wal/` and panics-as-error if a name is not a
/// segment id. Explorer and cloud clients drop sidecar files into those folders.
pub fn strip(db_dir: &Path) {
    for subdir in ENGINE_SUBDIRS {
        strip_dir(&db_dir.join(subdir));
    }
}

fn strip_dir(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !is_sidecar(name) {
            continue;
        }
        let _ = std::fs::remove_file(entry.path());
    }
}

fn is_sidecar(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "desktop.ini" | "thumbs.db" | "ehthumbs.db" | ".ds_store"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn desktop_ini_in_wal_is_removed_and_segments_are_kept() {
        let root = TempDir::new().unwrap();
        let wal = root.path().join("wal");
        std::fs::create_dir(&wal).unwrap();
        std::fs::write(wal.join("desktop.ini"), "[.ShellClassInfo]\n").unwrap();
        std::fs::write(wal.join("00000000000000000000.wal"), "segment").unwrap();

        strip(root.path());

        assert!(!wal.join("desktop.ini").exists());
        assert!(wal.join("00000000000000000000.wal").exists());
    }
}
