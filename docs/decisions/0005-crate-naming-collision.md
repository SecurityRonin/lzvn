# 5. Crate naming: publish `lzvn-core`, keep the `use lzvn::…` import path

Date: 2026-07-24
Status: Accepted

## Context

The natural crate name for this codec is the bare `lzvn`. That name is **already
taken on crates.io** by an unrelated third-party LZVN decoder — the same strict
whole-stream decoder this crate exists to replace
([ADR 0002](0002-purpose-built-decoder.md)). The fleet *Crate naming grammar*
(`~/src/ronin-issen/CLAUDE.md`) covers exactly this case: when the bare name is
taken, publish under a `-core` package but preserve the ergonomic import path via
`[lib] name`.

## Decision

Publish the crate as **`lzvn-core`** while setting `[lib] name = "lzvn"` so
consumers still write `use lzvn::…`:

```toml
[package]
name = "lzvn-core"
...
[lib]
name = "lzvn"
path = "src/lib.rs"
```

(`Cargo.toml`, with the inline comment: "The bare name `lzvn` is taken on
crates.io; we publish as `lzvn-core` but keep the import path `use lzvn::…` via
the lib name (the fleet collision rule)".) Dependents pin it as
`lzvn = { package = "lzvn-core", version = "0.1" }` (README, `fuzz/Cargo.toml`).

## Consequences

- Consumers get the clean `lzvn::decode_into(...)` call site; the published
  package name disambiguates from the third-party crate on crates.io.
- The badge and docs URLs track the two names: the Crates.io badge points at
  `lzvn-core`, the Docs.rs badge at `lzvn` (README badge block).
- This is a single-crate codec, not a container/filesystem repo, so it does
  **not** take the Pattern-A `<x>-core` reader + `<x>-forensic` analyzer split —
  there is no anomaly-auditing layer for a raw codec. The `-core` suffix here is
  driven purely by the crates.io name collision, not by a reader/analyzer split.
