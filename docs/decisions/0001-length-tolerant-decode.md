# 1. Length-tolerant decode: stop at end-of-stream, ignore trailing bytes

Date: 2026-07-24
Status: Accepted

## Context

LZVN is Apple's compression codec for HFS+/APFS **transparent compression**
(`decmpfs` types 7 inline and 8 resource-fork) and the `bvxn` block type inside
an LZFSE stream. A real-world `decmpfs` **resource-fork** block ends with the
LZVN end-of-stream opcode (`0x06`) and is then followed by **80–300 bytes of
arbitrary trailing data** per block. Apple's kernel and `lzvn_decode_buffer`
decode up to the end-of-stream marker and ignore everything after it.

Strict *whole-stream* Rust decoders — `lzfse_rust` and the third-party `lzvn`
crate — treat those trailing bytes as a malformed tail and **reject the block**.
Measured against a macOS 26.5 ("Tahoe", build 25F71) system volume, the strict
path decoded **0 of 25** genuine type-8 blocks (see
[`docs/validation.md`](../validation.md)). A decoder that cannot read real
system files is useless to a forensic reader.

## Decision

Decode terminates at the end-of-stream opcode and returns the bytes produced so
far; the decoder **does not inspect or reject** any input after `0x06`.

The `run()` dispatch loop makes this explicit:

```
// End of stream: stop here. Length-tolerant — we intentionally
// do NOT inspect or reject the bytes that follow.
0x06 => return Ok(self.op),
```

(`src/lib.rs`, lines 141–143.) A `decmpfs` resource-fork block can therefore be
passed **verbatim**, trailing padding and all, to `decode_into` — the caller
does not pre-trim it.

## Consequences

- This single property is the difference between reading a real evidence disk
  and erroring out: **25/25** genuine macOS 26.5 blocks decode here versus
  **0/25** on the strict decoders ([`docs/validation.md`](../validation.md)).
- The caller must supply the expected output length (from the `decmpfs` header
  or the LZFSE block header); the decoder stops on end-of-stream, not on input
  exhaustion, so trailing bytes never inflate the result (see
  [ADR 0006](0006-caller-owned-output-buffer.md)).
- The decoder is deliberately *lenient on the tail* but stays *strict on the
  body*: an undefined opcode before end-of-stream is still an
  `Error::InvalidOpcode`, and every read is bounds-checked
  ([ADR 0004](0004-safety-posture.md)). Length-tolerance is scoped to the
  post-end-of-stream tail only.
