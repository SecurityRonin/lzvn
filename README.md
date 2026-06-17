# lzvn

[![Crates.io](https://img.shields.io/crates/v/lzvn-core.svg)](https://crates.io/crates/lzvn-core)
[![Docs.rs](https://docs.rs/lzvn/badge.svg)](https://docs.rs/lzvn)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-blue.svg)](https://www.rust-lang.org)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](src/lib.rs)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa.svg)](https://github.com/sponsors/h4x0r)

**A safe, dependency-free, `no_std` pure-Rust LZVN decompressor that actually reads real macOS files** — length-tolerant where every other Rust decoder rejects them.

```rust
// A raw Apple LZVN block (small-literal "hello" + end-of-stream).
let block = [0xe5, b'h', b'e', b'l', b'l', b'o', 0x06, 0, 0, 0, 0, 0, 0, 0];
let mut out = [0u8; 5];
let n = lzvn::decode_into(&block, &mut out)?;
assert_eq!(&out[..n], b"hello");
# Ok::<(), lzvn::Error>(())
```

```toml
[dependencies]
lzvn = { package = "lzvn-core", version = "0.1" }
```

## Why this exists

LZVN is Apple's compression codec for HFS+/APFS **transparent compression** (`decmpfs` types 7 and 8) and the `bvxn` block type inside an LZFSE stream. Real macOS `decmpfs` resource-fork blocks end with the LZVN end-of-stream opcode (`0x06`) and are then followed by **80–300 bytes of arbitrary trailing data** per block. Apple's kernel and `lzvn_decode_buffer` ignore those bytes; **strict whole-stream Rust decoders (`lzfse_rust`, the `lzvn` crate) reject them** — so they fail on genuine macOS system files (verified against macOS 26.5 "Tahoe": 0/25 real type-8 files decoded by the strict path).

`lzvn` stops at the end-of-stream marker and returns. That single property is the difference between decoding a real evidence disk and erroring out.

## Trust, but verify

- **`#![forbid(unsafe_code)]`**, zero dependencies, `no_std` — bounds-checked, returns a typed [`Error`] on malformed input rather than panicking or reading out of bounds.
- **Validated against an independent oracle.** Committed fixtures are real LZVN streams produced by Apple's own `COMPRESSION_LZVN` encoder (`libcompression`), padded with trailing bytes to mirror a `decmpfs` block — and the decoder was additionally validated against **25 genuine macOS 26.5 system-file blocks** against the same Apple oracle (0/25 decoded by strict decoders, 25/25 here).
- **Fuzz-hardened.** `cargo fuzz run decode` drives arbitrary input through the decoder; the invariant is "never panic."

## Scope

This crate is the **codec** — it decodes a raw LZVN block to bytes. The `decmpfs` framing (the resource-fork block table, the inline-marker conventions) belongs to the filesystem reader that calls it (e.g. `hfsplus-forensic`). Decode-only; no encoder.

---

[Privacy Policy](https://securityronin.github.io/lzvn/privacy/) · [Terms of Service](https://securityronin.github.io/lzvn/terms/) · © 2026 Security Ronin Ltd
