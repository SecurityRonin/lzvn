# 6. API shape: caller-owned output buffer, with an optional `alloc` convenience

Date: 2026-07-24
Status: Accepted

## Context

An LZVN block does not carry its own decompressed length — the length comes from
the `decmpfs` header or the LZFSE block header, which the caller already holds
([ADR 0003](0003-codec-only-scope.md)). Apple's own `lzvn_decode_buffer` takes a
caller-provided output buffer for the same reason. The crate is `no_std`
([ADR 0004](0004-safety-posture.md)), so the primitive entry point must not
require an allocator, yet most fleet consumers do have one and want a
`Vec`-returning convenience.

## Decision

1. **The core primitive is `decode_into(src: &[u8], dst: &mut [u8]) ->
   Result<usize>`** — it decodes into a caller-owned buffer and returns the byte
   count written (`src/lib.rs`, lines 103–112). It needs no allocator and is the
   only function on the `no_std` path.
2. **An allocating convenience `decode(src, decoded_len) -> Result<Vec<u8>>`**
   sits behind the **`alloc` feature**, which is **on by default** (`Cargo.toml`
   `[features] default = ["alloc"]`; `src/lib.rs`, lines 117–123 under
   `#[cfg(feature = "alloc")]`). It allocates exactly `decoded_len` bytes, calls
   `decode_into`, and truncates to the actual count.
3. A pure `no_std` consumer takes `default-features = false` to drop `alloc`
   (as `fuzz/Cargo.toml` does).

## Consequences

- The zero-config default (`alloc` on) gives ordinary consumers the ergonomic
  `decode(...)` call, consistent with the fleet *batteries-included* default;
  the lean `no_std` path is a deliberate opt-out for embedded/allocator-free
  reuse, not the default. `alloc` is the *only* feature — there is no capability
  hidden behind a flag an examiner must know to enable.
- The caller must know the decompressed length up front. That is correct for the
  target use: `decmpfs`/LZFSE both record it. `decode_into` returning
  `Error::OutputTooSmall` (with bytes-written and capacity) makes an undersized
  buffer a loud, typed failure rather than a truncated silent result.
- Returning the written count (not relying on `dst.len()`) is what lets
  end-of-stream — not buffer-fill — terminate decoding, which is the mechanism
  behind length-tolerance ([ADR 0001](0001-length-tolerant-decode.md)).
