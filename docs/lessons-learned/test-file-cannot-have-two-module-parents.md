# Same test file cannot have two parents

**Date:** 2026-09-01

## What to know before starting

`setup_tests.rs` was included twice under `#[cfg(test)]`:

- `private/mod.rs` as `mod setup_tests` (`super` = `private`)
- `setup.rs` as `#[path = "setup_tests.rs"] mod tests` (`super` = `setup`)

Rust compiles that file in both module trees. Imports like `super::error` are valid for one parent and an unresolved-import error for the other, so `cargo test --lib` fails even though `cargo check --lib` (no tests) is clean.

Keep one parent. Sibling tests of `private` belong in `mod.rs`; tests that live next to one use-case file should use only that file's `#[path]` / `mod tests` and `super::super`.
