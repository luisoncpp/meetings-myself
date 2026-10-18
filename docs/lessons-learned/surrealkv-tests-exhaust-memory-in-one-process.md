# SurrealKV tests exhaust memory in one test process

**Date:** 2026-10-17

## What to know

`cargo test -p planning-app --lib` opens one embedded SurrealKV database per test, and memory is not given back between tests in the same process. On a Windows machine with ~4 GB free, the run aborts partway with `memory allocation of 1073741824 bytes failed` / `STATUS_STACK_BUFFER_OVERRUN` — even with `--test-threads=1`. The test that "crashes" is just whichever one asks for the next 1 GB; it passes when run alone.

## What to do

- Don't debug the named test. Re-run by module filter, e.g. `cargo test -p planning-app --lib task_completion`, one process per module.
- A failure that only appears in the full run and passes in its module batch is this, not a regression.

## Related

- `setup_tests::reconnect_recovers_when_the_folder_reappears` can fail on Windows with `Acceso denegado` when renaming the sync folder: the closed app's database files may still be held for a moment. Also environmental.
