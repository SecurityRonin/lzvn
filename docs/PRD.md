# lzvn — Purpose & Scope

*A library-tier intent document (this crate ships no user-facing binary — it is a
codec leaf that other fleet readers link). Every current-state claim below is
grounded in a same-session read of `src/lib.rs`, `Cargo.toml`, and
`docs/validation.md` (2026-07-24). Load-bearing decisions live as ADRs under
[`docs/decisions/`](decisions/).*

## What it is

`lzvn-core` (imported as `use lzvn::…`) is a safe, dependency-free, `no_std`
pure-Rust **decompressor for Apple's LZVN codec**. It decodes one raw LZVN block
to bytes and does nothing else.

LZVN is the codec macOS uses for HFS+/APFS **transparent compression**
(`decmpfs` type 7 inline and type 8 resource-fork) and the `bvxn` block type
inside an LZFSE stream. The crate's one differentiator is **length-tolerance**:
it stops at the LZVN end-of-stream opcode (`0x06`) and ignores the 80–300
trailing bytes real `decmpfs` resource-fork blocks carry — the property that
lets it read genuine macOS system files where strict whole-stream Rust decoders
(`lzfse_rust`, the third-party `lzvn` crate) reject them
([ADR 0001](decisions/0001-length-tolerant-decode.md)).

## Who links it

Filesystem and container readers in the fleet that must decompress
Apple-compressed file content:

- **`hfsplus-forensic`** — the primary consumer; owns the `decmpfs` framing
  (resource-fork block table, inline markers, uncompressed-length header) and
  calls `lzvn-core` per block. The real-artifact regression corpus (25 macOS
  26.5 type-8 blocks) lives there, because it is only meaningful with that
  framing ([ADR 0003](decisions/0003-codec-only-scope.md)).
- Any future APFS `decmpfs` path or LZFSE `bvxn` consumer that already knows the
  decompressed length.

The dependency arrow points **down**: readers depend on `lzvn-core`; this crate
depends on nothing.

## Surface

- `decode_into(src: &[u8], dst: &mut [u8]) -> Result<usize>` — the `no_std`
  primitive; decodes into a caller-owned buffer, returns bytes written.
- `decode(src: &[u8], decoded_len: usize) -> Result<Vec<u8>>` — an allocating
  convenience behind the default `alloc` feature
  ([ADR 0006](decisions/0006-caller-owned-output-buffer.md)).
- `Error` — a typed error (`TruncatedInput` / `OutputTooSmall` / `InvalidOpcode`
  / `InvalidMatchDistance`), each carrying the offending offset or value.

## Scope

**In scope:** decode a single raw LZVN block; length-tolerance for real-world
`decmpfs` blocks; a bounds-checked, panic-free, `forbid(unsafe)` implementation
for untrusted input.

**Non-goals:**

- **No encoder.** Decode only — Apple's `COMPRESSION_LZVN` is the validation
  oracle, and no forensic path needs to write LZVN
  ([ADR 0002](decisions/0002-purpose-built-decoder.md)).
- **No `decmpfs`/LZFSE framing.** The block table, inline-vs-resource marker
  conventions, and headers belong to the calling filesystem reader
  ([ADR 0003](decisions/0003-codec-only-scope.md)).
- **No file I/O, no paths, no filesystem types** — that is what keeps the crate
  `no_std` and dependency-free.
- **Not a `-core`/`-forensic` reader/analyzer pair.** A raw codec has no
  anomaly-auditing layer; the `-core` suffix here is driven only by the
  crates.io name collision ([ADR 0005](decisions/0005-crate-naming-collision.md)).

## How correctness is established

Two tiers, documented in [`docs/validation.md`](validation.md):

1. **Apple's encoder as oracle (committed, redistributable fixtures).** Each
   `tests/data/<name>.lzvn` is a real LZVN stream produced by Apple's
   `COMPRESSION_LZVN` (`libcompression`) from synthetic inputs, padded with
   trailing bytes to mirror a `decmpfs` block. Cases span heavy-overlap matches,
   mixed literals/matches, and low-compressibility input (`tests/decode.rs`).
2. **Real macOS 26.5 ("Tahoe", build 25F71) system files.** 25 genuine type-8
   resource-fork blocks decoded byte-for-byte against the kernel's transparent
   read: **25/25** here versus **0/25** on the strict decoders. That corpus lives
   with `hfsplus-forensic`.

**Robustness:** `#![forbid(unsafe_code)]`, bounds-checked reads, typed `Error` on
malformed input, and a `decode` fuzz target that ran clean over 1.37M executions
(invariant: never panic) ([ADR 0004](decisions/0004-safety-posture.md)).

---

[Privacy Policy](https://securityronin.github.io/lzvn/privacy/) · [Terms of Service](https://securityronin.github.io/lzvn/terms/) · © 2026 Security Ronin Ltd
