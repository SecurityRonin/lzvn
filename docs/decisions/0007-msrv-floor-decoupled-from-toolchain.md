# 7. Declared MSRV floor decoupled from the pinned dev toolchain

Date: 2026-07-24
Status: Accepted

## Context

The fleet *Rust MSRV & Toolchain Policy* (`~/src/ronin-issen/CLAUDE.md`,
`~/.claude/CLAUDE.core.md`) separates two versions that are easy to conflate: the
**dev toolchain** (what the fleet builds/fmt/clippy with, pinned uniformly) and
the **declared MSRV** (`rust-version`, a downstream-facing compatibility promise).
For a **published library**, MSRV is kept low and set by need — raising it
narrows the crates.io audience and is a near-breaking change. `lzvn-core` is a
published library other fleet readers link.

## Decision

- **Dev toolchain: `rust-toolchain.toml` pins `channel = "1.96.0"`** with
  `clippy` + `rustfmt` components — the fleet-wide current stable, set in commit
  `5992150` ("chore: pin toolchain to 1.96.0 (fleet toolchain policy)").
- **Declared MSRV: `rust-version = "1.85"`** in `Cargo.toml` — a floor well below
  the dev toolchain, so third-party consumers on an older stable can still link
  the codec. The crate's language/library needs are modest: it uses
  `core::error::Error` (`src/lib.rs`, line 92), stabilized in Rust 1.81, and
  otherwise plain `no_std` core.

## Consequences

- A consumer is not forced onto the newest stable to use the decoder; the low
  floor is a deliberate compatibility feature and trust signal, per the library
  MSRV policy.
- Raising `rust-version` later must be a deliberate, justified bump (a new
  language feature genuinely needed), not a drift to match the dev toolchain.
- **Rationale for the *exact* `1.85` value (rather than, say, the `1.81` that
  `core::error::Error` alone would require, or the `1.75`/`1.80` floors used by
  other fleet libraries) is not recovered in available history** — no commit or
  comment records why 1.85 specifically was chosen. The decision *to keep a low
  declared floor decoupled from the 1.96 dev toolchain* is grounded in the fleet
  policy and the code's actual feature use; the precise cutoff is reconstructed
  from structure, original intent not recovered. A future change may lower it to
  the true minimum after a CI MSRV job confirms the floor.
