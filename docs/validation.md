# Validation

`lzvn` is a decode-only codec for Apple LZVN. Correctness is established two ways.

## 1. Apple's encoder as oracle (committed fixtures)

Each `tests/data/<name>.lzvn` is a real LZVN stream produced by Apple's own
`COMPRESSION_LZVN` encoder (`libcompression`, constant `0x900`) from the bytes in
`<name>.expected`, then padded with trailing bytes after the end-of-stream
opcode to mirror a `decmpfs` resource-fork block. The inputs are synthetic so the
fixtures are freely redistributable. Cases span heavy-overlap matches
(`text_repeats`), mixed literals/matches (`mixed`), and low-compressibility input
(`near_random`). See `tests/decode.rs`.

## 2. Real macOS 26.5 (Tahoe) system files

The decoder was validated against **25 genuine type-8 (LZVN resource-fork) blocks**
read from a read-only-mounted macOS 26.5 (build 25F71) system volume, each decoded
and compared byte-for-byte to the kernel's transparent read (and to
`COMPRESSION_LZVN`). Result: **25/25** here vs **0/25** for strict whole-stream
decoders (`lzfse_rust`, the `lzvn` crate), which reject the 80–300 trailing bytes
real blocks carry after end-of-stream. That real-artifact regression corpus lives
with the filesystem reader (`hfsplus-forensic`, `tests/data/decmpfs/tahoe_type8.*`).

## Robustness

`#![forbid(unsafe_code)]`, bounds-checked, typed `Error` on malformed input. The
`decode` fuzz target ran clean over 1.37M executions (invariant: never panic).
