# Svelte 5 Runes Need the `.svelte.ts` Extension

**Date:** 2026-08-08

## What looks fine

A store class in a plain `.ts` file compiles and runs. TypeScript accepts `$state` fields; Vitest can instantiate the class; no build error, no runtime throw.

## What breaks

`$state` is a **compile-time transform**, not a runtime API. Outside `.svelte` and `.svelte.ts` modules, the compiler does not rewrite `$state` into reactive proxies. Assignments update a plain field; Svelte never schedules a re-render. The screen simply does not change.

This is silent — the worst kind of failure.

## What we did

Surface stores live in `*.svelte.ts` files (`DailyPlanStore.svelte.ts`, `LibraryStore.svelte.ts`, …). Components import them normally; only the extension signals the compiler to apply rune transforms.

## Second trap: `vi.mock` and dynamic `import()`

Store tests use `await import('./DailyPlanStore.svelte')` so `vi.mock('../../../api')` is registered first. If the store is imported statically at the top of the test file, the real `api` module is captured before the mock runs — assertions hit the real bridge or stale stubs.

Pattern that works (`DailyPlanStore.test.ts`):

1. `vi.hoisted(() => vi.fn())` for each stubbed export.
2. `vi.mock('../../../api', () => ({ … }))` at module scope.
3. `const { DailyPlanStore } = await import('./DailyPlanStore.svelte')` inside each test.

## Generalizable lesson

Svelte 5 reactivity is not "use `$state` anywhere in TypeScript." It is "use `$state` only where the Svelte compiler processes the file." When tests dynamically import the module under test, hoist mocks above that import.
