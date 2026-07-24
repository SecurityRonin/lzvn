# 4. Safety posture: no_std, forbid(unsafe), zero deps, bounds-checked, fuzzed

Date: 2026-07-24
Status: Accepted

## Context

This decoder parses **untrusted, attacker-controllable input** — LZVN blocks
lifted from an evidence disk image. The fleet *Paranoid Gatekeeper* standard
(`~/src/ronin-issen/CLAUDE.md`) requires such parsers to never panic, never read
out of bounds, and never trust a length field. The global *unsafe* law makes
`forbid(unsafe)` the default and the goal for a crate that has no genuine
performance reason to surrender the compiler's memory-safety proof — and a pure
LZVN codec (no `mmap`, no FFI, no SIMD intrinsics) has none.

## Decision

1. **`#![no_std]` + `#![forbid(unsafe_code)]`** at the crate root (`src/lib.rs`,
   lines 29–30; enforced again by `unsafe_code = "forbid"` in `Cargo.toml`
   `[lints.rust]`). This crate can wear the `unsafe forbidden` badge honestly —
   it is a true `forbid`, not a `deny` + bounded-allow.
2. **Zero dependencies.** No `safe-read` either: every read goes through the
   crate's own bounds-checked helpers — `byte()` returns
   `Error::TruncatedInput` via `slice::get(...).ok_or(...)` (lines 212–217), and
   `need_input`/`need_output` use `saturating_sub` before any copy
   (lines 219–234). Length and match-distance fields from the stream are
   range-checked before use: `copy_match` rejects a zero or out-of-range distance
   with `Error::InvalidMatchDistance` (lines 272–278).
3. **Typed `Error`, never a panic.** Malformed input yields one of
   `TruncatedInput` / `OutputTooSmall` / `InvalidOpcode` / `InvalidMatchDistance`
   (lines 37–67), each carrying the offending offset/value so a caller can
   diagnose it.
4. **Fuzz-hardened.** `fuzz/fuzz_targets/decode.rs` drives arbitrary bytes
   through `decode_into` into a fixed 64 KiB buffer; the invariant is "never
   panic". It ran clean over 1.37M executions
   ([`docs/validation.md`](../validation.md)).

## Consequences

- A crafted or corrupt block returns a typed error instead of reading out of
  bounds or panicking — the memory-corruption class safe Rust deletes stays
  deleted, provably (`forbid`, not `deny`).
- Being `no_std` and dependency-free, the crate links into any consumer,
  allocator or not, and adds nothing to a downstream `cargo deny` / `cargo vet`
  audit surface. The `deny.toml` license allow-list and `supply-chain/`
  cargo-vet config exist for the (currently empty) transitive graph and CI
  hygiene, not because the codec itself pulls anything.
- The `unwrap_used`/`expect_used = deny` panic lints the fleet applies to
  bytes-in parsers are **not** wired here; the panic-free guarantee rests on
  `forbid(unsafe)` + explicit `?`-returning helpers + the fuzz target instead.
  Adding those denies would be defense-in-depth and is a cheap future
  hardening step.
