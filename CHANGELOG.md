# Changelog

## 0.1.0 — unreleased

- Initial release: length-tolerant pure-Rust Apple LZVN decoder.
- `decode_into` / `decode` for raw LZVN streams; stops at the end-of-stream
  opcode and ignores trailing bytes (reads real `decmpfs` resource-fork blocks).
- `no_std`, `#![forbid(unsafe_code)]`, zero dependencies, typed `Error`.
- Validated against real macOS 26.5 LZVN blocks with Apple's `COMPRESSION_LZVN`
  as the oracle; fuzz target `decode`.
