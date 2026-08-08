# SurrealDB + RocksDB on Windows: build prerequisites

**Date:** 2026-08-08

## What to know before starting

Adding `surrealdb` with the `kv-rocksdb` feature on Windows MSVC triggers two unrelated native-toolchain failures long before any application code compiles. Both look like environment problems, not dependency bugs — but they block every `cargo check` that pulls `surrealdb-core`.

## Blocker 1: libclang (bindgen)

`surrealdb-librocksdb-sys` generates its RocksDB bindings with `bindgen`, which needs `libclang.dll`. Without LLVM installed the build panics:

```
Unable to find libclang: "couldn't find any valid shared libraries matching:
['clang.dll', 'libclang.dll'], set the `LIBCLANG_PATH` environment variable"
```

### Fix

Install LLVM and point bindgen at it:

```powershell
winget install --id LLVM.LLVM -e --accept-package-agreements --accept-source-agreements
```

If the DLL exists at `C:\Program Files\LLVM\bin\libclang.dll` but builds still fail, set the variable persistently:

```powershell
[Environment]::SetEnvironmentVariable('LIBCLANG_PATH', 'C:\Program Files\LLVM\bin', 'User')
```

## Blocker 2: NASM (aws-lc-sys)

`surrealdb-core` → `jsonwebtoken` 10 → `aws-lc-rs` → `aws-lc-sys`. On Windows MSVC that crate's build script requires NASM and panics with:

```
NASM command not found! Build cannot continue.
```

This happens even with `--no-default-features --features kv-rocksdb`, so it cannot be avoided by feature selection on `surrealdb` itself.

### Fix

Add a workspace-level dependency on `aws-lc-sys` with the `prebuilt-nasm` feature. Cargo's feature unification applies it to the transitive copy, removing the NASM build step:

```toml
# In root Cargo.toml [workspace.dependencies]
aws-lc-sys = { version = "0.44", features = ["prebuilt-nasm"] }
```

Reference it from any crate that depends on `surrealdb` (e.g. `planning-store`):

```toml
aws-lc-sys = { workspace = true }
```

The alternative — `winget install NASM.NASM` and putting it on `PATH` — also works.

## Verification

Probe project (surrealdb 3.2.4, `kv-rocksdb`, `aws-lc-sys` with `prebuilt-nasm`):

```powershell
$probe = Join-Path $env:TEMP "surreal-probe-$(Get-Random)"
cargo new --lib $probe
Set-Location $probe
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
cargo add surrealdb --no-default-features --features kv-rocksdb
cargo add aws-lc-sys --features prebuilt-nasm
cargo check
```

**First-build duration observed:** ~2 minutes 17 seconds (137 s) on Windows 11, MSVC 14.50, Rust 1.95, LLVM 22.1.8. Plan budgeted 10–20 minutes; actual time may vary with cache state and machine speed.

## Version note

The plan originally pinned `aws-lc-sys` 0.43; surrealdb 3.2.4 pulls 0.44 transitively. The workspace pin was updated to 0.44 to match.
