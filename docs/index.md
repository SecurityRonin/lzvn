# lzvn

A safe, dependency-free, `no_std` pure-Rust **LZVN decompressor** that reads real
macOS files — **length-tolerant** where strict decoders reject them.

```rust
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

## Why length-tolerance matters

LZVN is Apple's codec for HFS+/APFS transparent compression (`decmpfs` types 7
and 8) and the `bvxn` block inside an LZFSE stream. Real macOS `decmpfs`
resource-fork blocks end with the LZVN end-of-stream opcode (`0x06`) and are then
followed by 80–300 bytes of trailing data. Apple's kernel ignores those bytes;
strict whole-stream Rust decoders reject them — and so fail on genuine macOS
system files. `lzvn` stops at end-of-stream and returns.

See [Validation](validation.md) for the oracle methodology and the real macOS
26.5 (Tahoe) results.

## Trust, but verify

- `#![forbid(unsafe_code)]`, zero dependencies, `no_std`, typed errors.
- Fixtures encoded by Apple's own `COMPRESSION_LZVN`; decoder additionally
  validated against 25 genuine macOS 26.5 system-file blocks (0/25 strict, 25/25
  here).
- Fuzz target `decode` clean over 1.37M executions.

---

[Privacy Policy](privacy.md) · [Terms of Service](terms.md) · © 2026 Security Ronin Ltd
