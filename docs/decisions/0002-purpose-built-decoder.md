# 2. Build a purpose-built decoder rather than reuse the strict LZVN/LZFSE crates

Date: 2026-07-24
Status: Accepted

## Context

Rust already has LZVN decoders: `lzfse_rust` (LZVN is the `bvxn` block type
inside LZFSE) and the standalone `lzvn` crate. The fleet's build-vs-reuse
discipline (`~/src/ronin-issen/CLAUDE.md` → *Research-First*, *Dependency
Preference*) says to reuse a correct, maintained crate before writing our own.

Both existing crates fail the one requirement that matters here: they are
**whole-stream** decoders that reject the 80–300 trailing bytes real `decmpfs`
resource-fork blocks carry after end-of-stream (see
[ADR 0001](0001-length-tolerant-decode.md)). They decoded **0 of 25** genuine
macOS 26.5 blocks. The gap is not a bug we could patch around at the call site —
it is a structural property of a whole-stream decoder, so the artifact we need
does not exist in the ecosystem.

## Decision

Implement a small, dedicated LZVN decoder (crate `lzvn-core`, commit `ee8acae`
"feat: length-tolerant pure-Rust Apple LZVN decoder") whose sole
differentiators are:

1. **Length-tolerance** (stop at end-of-stream) — the property no existing crate
   offers.
2. **A pure-Rust, `no_std`, zero-dependency, `forbid(unsafe)` posture** suited to
   an untrusted-input forensic parser ([ADR 0004](0004-safety-posture.md)) — the
   full opcode dispatch lives in one file, `src/lib.rs` (~290 lines).

This also satisfies the fleet's *prefer-our-own-crates* rule: a
SecurityRonin-owned codec that other fleet readers link, rather than a
third-party dependency that cannot read the target files.

## Consequences

- The decoder is the codec leaf every LZVN-consuming fleet reader links
  (`hfsplus-forensic`, and any future APFS `decmpfs` path), instead of a
  third-party crate that would reject real evidence.
- Reimplementing an established algorithm carries a correctness burden. It is
  discharged against an **independent oracle** rather than self-authored
  round-trips: fixtures are produced by Apple's own `COMPRESSION_LZVN` encoder
  (`libcompression`), and the decoder was additionally validated against 25
  genuine macOS 26.5 blocks with the kernel's transparent read as ground truth
  ([`docs/validation.md`](../validation.md)).
- The existing crates remain useful as cross-check oracles for the *body* of a
  stream (the pre-end-of-stream bytes must decode identically); only the tail
  handling differs.
- Scope is intentionally narrow — decode only, no encoder (Apple's encoder is the
  oracle, and no forensic path needs to *write* LZVN).
