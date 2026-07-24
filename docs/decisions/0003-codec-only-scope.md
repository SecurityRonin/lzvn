# 3. Codec-only scope: decmpfs and LZFSE framing belong to the caller

Date: 2026-07-24
Status: Accepted

## Context

LZVN appears in two framings a forensic reader cares about: the `decmpfs`
resource-fork block table (with its inline-marker conventions and per-block
offsets) on HFS+/APFS, and the `bvxn` block inside an LZFSE stream. It is
tempting to fold that framing into the codec so a caller can "just decode a
file". But the fleet layer architecture (`~/src/ronin-issen/CLAUDE.md` →
*Multi-Repo Architecture*, *Practical Decision Rule* #1) places *facts about a
format and pure codecs* at the KNOWLEDGE leaf, and *navigation of a filesystem's
structures* at the FILESYSTEM layer above it. Mixing the two inverts the
dependency direction.

## Decision

`lzvn-core` decodes exactly **one raw LZVN block to bytes** and nothing more.
The `decmpfs` framing — the resource-fork block table, the inline-vs-resource
marker conventions, the uncompressed-length header — is the responsibility of
the **filesystem reader that calls it** (e.g. `hfsplus-forensic`). This is stated
in the README *Scope* section and the crate-level docs (`src/lib.rs`, lines
3–7): "The output length must be known (or upper-bounded) by the caller, exactly
like Apple's `lzvn_decode_buffer`."

The dependency arrow points **down**: filesystem readers depend on `lzvn-core`;
`lzvn-core` depends on nothing (see [ADR 0004](0004-safety-posture.md)).

## Consequences

- The crate stays a reusable KNOWLEDGE/codec leaf with a tiny surface
  (`decode_into`, `decode`, `Error`) — usable by any consumer that already knows
  the decompressed length, regardless of framing (HFS+ `decmpfs`, APFS, or an
  LZFSE `bvxn` block).
- The real-artifact regression corpus (25 macOS 26.5 type-8 blocks) lives with
  the filesystem reader, not here, because it is only meaningful *with* the
  `decmpfs` framing that produced it ([`docs/validation.md`](../validation.md)).
  This repo commits only framing-independent codec fixtures.
- No file I/O, no path handling, and no filesystem types enter this crate; that
  keeps `no_std` and zero-dependency achievable ([ADR 0004](0004-safety-posture.md)).
