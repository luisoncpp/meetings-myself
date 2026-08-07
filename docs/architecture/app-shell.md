# App Shell

Tauri 2 desktop host + Svelte 5 frontend. The shell wires UI to Rust; domain logic lives in workspace crates (arriving in plan 0003+).

## Crate graph

One-way dependency flow — binaries never reach into crate internals:

```
launcher (plan 0008) ──┐
                       ├──► planning-app ──► planning-core
src-tauri ─────────────┘         │                  │
                                 ▼                  ▼
                          planning-store      planning-reports
```

`src-tauri` and `launcher` may depend on **planning-app only**. `lib.rs` is each crate's public interface; `src/private/` holds implementation details.

## Frontend IPC boundary

`src/lib/api` is the sole crossing point between Svelte and Tauri:

- `index.ts` — public API (`appVersion`, future commands)
- `Private/bridge.ts` — only file that imports `@tauri-apps/api`

Nothing else under `src/` may import `@tauri-apps/api`.

## Quality gate

`npm run check` is the definition of done: `cargo fmt` + `clippy` + `cargo test`, then `svelte-check --tsgo`, `eslint`, and `vitest run`. `fallow audit` enforces module boundaries (`.fallowrc.json`).

`tests/architecture.test.ts` enforces crate privacy, binary dependency rules, the IPC boundary, and token-only colours.

## Shell file map

| Path | Role |
|------|------|
| `Cargo.toml` | Workspace root; shared deps |
| `src-tauri/Cargo.toml` | Tauri binary crate manifest |
| `src-tauri/src/lib.rs` | Crate public interface; registers commands |
| `src-tauri/src/private/` | Command handlers and Tauri wiring |
| `src-tauri/tauri.conf.json` | Tauri app config |
| `src/main.ts` | Vite entry; mounts `App.svelte` |
| `src/App.svelte` | Root surface (placeholder until plan 0007) |
| `src/lib/api/` | Deep module — sole IPC bridge |
| `src/styles/tokens.css` | Design tokens (only place for raw hex) |
| `tests/architecture.test.ts` | Structural invariants |
| `tests/tokens.test.ts` | Token contract tests |
| `eslint.config.js` | Lint rules |
| `.fallowrc.json` | Boundary zones and fallow audit config |
| `package.json` | `check`, `typecheck`, `test`, `lint` scripts |
