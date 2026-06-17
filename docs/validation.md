# Validation

How `lzo`'s correctness and robustness are established. The guiding principle is
**differential testing against an independent reference**: every compressed test
input is produced by the canonical C **liblzo2** library, and `lzo` must decode
it back to the exact original. Because `lzo` does not encode, the encoder and the
decoder share no code — a round-trip mismatch can only mean `lzo` is wrong, which
is what makes the test meaningful (tests whose fixtures were produced by the code
under test would not).

## Summary

| Layer | What it checks | Where |
|---|---|---|
| Reference round-trip vectors | Decodes blocks from liblzo2 `lzo1x_1` / `lzo1x_999` | `tests/roundtrip.rs` (CI, every push) |
| Real-world corpus | 32.4 MB of real files across all three liblzo2 variants | one-time run (below); reproducible via `validation/` |
| Differential vs `rust-lzo` | Output matches an independent (Linux-derived) decoder, incl. mutation fuzz | one-time run (below) |
| Robustness / fuzzing | Never panics on arbitrary, truncated, or crafted input | `tests/errors.rs` + `fuzz/` |
| Coverage | 100% of lines, reached through the public API alone | CI coverage gate |

What is **observed**: every block listed below decompressed to a byte-exact copy
of its original. What is **out of scope** (and therefore *not* claimed) is listed
under [Scope and limits](#scope-and-limits).

## 1. Reference round-trip vectors (CI)

`tests/data/*.{raw,lzo}` holds `<name>.raw` (the original) and `<name>.lzo` (the
liblzo2-compressed block). `tests/roundtrip.rs` decodes each and asserts equality
on every push.

Two groups:

- **Opcode probes** (`lzo1x_1`) — `empty`, `hello`, `run_a`, `pattern`,
  `incompressible`, `farmatch` — small inputs hand-chosen to exercise specific
  instructions: literal runs, the zero-byte length extension, M1–M4 matches,
  overlapping copies (distance < length), and the end-of-stream marker.
- **Real-content vectors** (`lzo1x_999`) — `readme` (this project's README, real
  prose) and `libsrc` (`src/lib.rs`, real Rust source). `lzo1x_999` is the
  max-compression variant; it emits long-distance M3/M4 matches and length
  extensions far more densely than `lzo1x_1`, exercising those paths on real
  byte distributions rather than crafted probes.

## 2. Real-world corpus (one-time differential run)

The reference encoder was run over a corpus of **real files** — not synthetic
fixtures — and every resulting block was decoded by `lzo` and compared byte-for-
byte. This run is documented here rather than committed to CI because the source
files (system binaries, a system photo) are not redistributable; it is fully
reproducible with the harness in [`validation/`](../validation) against any local
files.

**Environment:** liblzo2 2.10 (Homebrew), macOS (arm64), `lzo` 0.1.0.

**Result (observed):** all **27** blocks — 9 files × 3 variants (`lzo1x_1`,
`lzo1x_1_15`, `lzo1x_999`), **32,439,408 bytes** of original data in total —
decoded byte-exact.

| File | Kind | Size | Why it matters |
|---|---|---|---|
| `/usr/share/dict/words` | English text | 2.49 MB | large, highly compressible text |
| `/bin/bash` | Mach-O binary | 1.31 MB | real executable, mixed entropy |
| `/bin/ls` | Mach-O binary | 155 KB | smaller real executable |
| system `.heic` photo | image | 6.09 MB | real image; near-incompressible, long literal runs |
| `words.gz` | gzip stream | 754 KB | already-compressed; stresses literal-run paths |
| `src/lib.rs` | Rust source | 8.3 KB | real code |
| `README.md` | prose | 2.8 KB | real prose |
| empty / 1-byte | edge | 0 / 1 B | boundary inputs |

Across the three variants this spans the full opcode surface a `lzo1x` stream can
use, on real data of every entropy profile from highly compressible text to
incompressible image bytes.

## 3. Differential validation against an independent decoder

The liblzo2 round-trips above prove `lzo` agrees with the reference *encoder*. As
a second, independent check, `lzo`'s output was compared block-for-block against
[`rust-lzo`](https://crates.io/crates/rust-lzo) — a separate, GPL-2.0 pure-Rust
decoder converted from Linux's `lzo1x_decompress_safe`. (`rust-lzo` is used here
only as a local validation oracle; it is **not** a dependency of this crate — its
GPL licence never touches the shipped code.)

**Result (observed):**

- **Real corpus** — all **27** liblzo2 blocks (32,439,408 bytes): `lzo` and
  `rust-lzo` produced identical output, equal to the original, byte-for-byte.
- **Mutation fuzz** — **3,000,000** inputs, each a real lzo block with 1–4 bytes
  randomly mutated (plus occasional truncation), so a large fraction stay
  near-valid. Of these, **903,069** were accepted by *both* decoders, and in
  **every** such case the two outputs were identical (**0** divergences). For the
  remaining inputs both decoders rejected (≈2.10 M); there were **0** cases where
  one accepted and the other rejected — i.e. the accept/reject boundary matched
  the Linux-derived implementation exactly. `lzo` panicked on none.

This is a stronger statement than encoder round-tripping alone: it confirms `lzo`
matches a second, lineage-independent decoder both on valid data and on the
boundary of malformed input. The harness is `validation/lzodiff` (see
[Reproducing](#reproducing)).

## 4. Robustness against malicious / corrupted input

A safe decoder must never panic, read out of bounds, or loop forever on hostile
input — it must return a typed [`Error`]. This is enforced two ways:

- **`tests/errors.rs`** drives the public API on truncated markers, undersized
  output buffers, back-references before the output start (lookbehind overrun),
  trailing bytes after end-of-stream, truncated literal runs, oversized length
  runs, output-overflowing matches, and non-canonical end markers — each
  asserting the specific `Error`. A loop feeds **20,000** pseudo-random inputs
  and asserts none panics.
- **`fuzz/fuzz_targets/decompress.rs`** is a libfuzzer target running the decoder
  on arbitrary bytes. CI builds it on nightly every push (`cargo fuzz build`);
  extended fuzzing runs locally with `cargo fuzz run decompress`.

`#![forbid(unsafe_code)]` means every index and slice is bounds-checked by the
compiler — an out-of-bounds access is a typed `Error`, never undefined behaviour.

## 5. Coverage

CI enforces **100% line coverage** (`cargo llvm-cov`; any uncovered line fails
the build). It additionally enforces that the **integration tests alone** reach
100% — i.e. every line is reachable through the public API, with no line covered
only by white-box unit tests. There are no `lzo`-internal unit tests; the whole
decoder is validated as a black box.

## Scope and limits

What this validation does **not** establish, stated plainly:

- **Containers are out of scope.** `lzo` decodes a *raw* `lzo1x` block. It does
  not parse the `lzop` file format, the dar `block_compressor` framing, btrfs
  extents, or kernel/initramfs headers — those wrap raw blocks and are the
  caller's responsibility. The intended forensic consumer,
  [`dar-forensic`](https://github.com/SecurityRonin/dar-forensic), performs that
  framing; end-to-end validation of dar `-zlzo` archives belongs there.
- **The kernel bitstream-version (RLE) extension is not supported** and not
  tested. Standard `lzo1x` streams only.
- **Decompression only.** There is no encoder to validate.
- **No big-endian host has been exercised.** The decoder reads little-endian
  distances explicitly (not via native byte order), so it is expected to be
  endian-independent, but this is an *inference* from the code, not an observed
  result — all runs above were on arm64.

## Reproducing

```sh
# Build the reference-encoder harness (needs liblzo2 dev headers).
cc -O2 -I"$(brew --prefix lzo)/include" validation/lzo_compress.c \
   -L"$(brew --prefix)/lib" -llzo2 -o /tmp/lzo_compress

# Compress any real file with each variant, then decode + compare.
/tmp/lzo_compress 999 /bin/ls /tmp/ls.lzo
# (decode /tmp/ls.lzo with lzo::decompress_into and compare to /bin/ls)

# The committed vectors run in CI:
cargo test --test roundtrip
cargo test --test errors

# Regenerate a committed vector:
/tmp/lzo_compress 999 tests/data/readme.raw tests/data/readme.lzo

# Differential check vs the independent rust-lzo decoder (real corpus + mutation
# fuzz). This pulls the GPL-2.0 rust-lzo crate as a *local* dev oracle only — it
# is deliberately NOT a dependency of the lzo crate. Point RW at a dir of
# <name>.raw / <name>.<algo>.lzo pairs (see validation/lzo_compress.c above).
cd validation/lzodiff && RW=/path/to/pairs cargo run --release
```

`validation/lzodiff/` is a standalone, non-published crate (it is not part of the
`lzo` workspace and is never built by `cargo test`/`cargo publish`), so the GPL
oracle stays entirely outside the shipped, MIT-licensed crate.

[`Error`]: https://docs.rs/lzo/latest/lzo/enum.Error.html
