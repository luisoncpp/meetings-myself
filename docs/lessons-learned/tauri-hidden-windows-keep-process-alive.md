# Tauri hidden windows keep the process alive

**Date:** 2026-08-09 — **Historical (2026-10-17):** plan 0009 moved Weekly Review into the main window, so the app has one window and no lifecycle code. Keep this in mind if a second window ever returns.

## What to know

Tauri exits only when every window is destroyed. `prevent_close` plus `hide()` keeps a window alive in memory, so closing the last *visible* window is not enough — the process keeps running until something calls `app.exit()` or the hidden window is destroyed.

If one window uses hide-on-close for fast reopen, the primary window's close handler must explicitly quit the app.

## Where this applies

- `src-tauri/src/private/window_commands.rs` — `attach_window_lifecycle`
