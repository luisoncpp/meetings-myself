/// Proves the IPC bridge works end to end. Plan 0004 replaces this module's
/// contents with the real application commands.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_the_cargo_manifest() {
        assert_eq!(app_version(), env!("CARGO_PKG_VERSION"));
    }
}
