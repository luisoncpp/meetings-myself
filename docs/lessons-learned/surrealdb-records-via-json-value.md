# SurrealDB Records via JSON Value

**Date:** 2026-08-08

## What we wanted

A generic `Records` gateway in `planning-store` that accepts any domain type with `Serialize`/`Deserialize` — no SurrealDB types leaking into `planning-core`.

## What SurrealDB 3 requires

`.upsert().content()` and `.select()` are typed with `SurrealValue`. Plain entity structs do not implement it unless you derive `SurrealValue` on every entity (coupling the domain to SurrealDB).

## What works

Round-trip through `serde_json::Value`:

1. **Save** — serialize entity to JSON, strip the top-level `id` field, upsert the value (record key `(table, id)` is authoritative).
2. **Find** — select as `Option<Value>`, inject `id` from the key, deserialize to `T`.
3. **All** — select table as `Vec<Value>`, normalize SurrealDB record ids (`task:\`uuid\``) back to bare UUID strings before deserializing.

Missing table/record errors containing `"does not exist"` map to `None` / `[]`.

## Generalizable lesson

When an embedded DB's Rust SDK wants its own value trait, keep domain types pure and adapt at the persistence boundary with JSON as the interchange format — especially when the gateway is generic across many entity types.
