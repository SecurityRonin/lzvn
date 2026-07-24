# Changelog

## [0.1.1](https://github.com/SecurityRonin/lzvn/compare/lzvn-core-v0.1.0...lzvn-core-v0.1.1) - 2026-07-24

### Documentation

- reverse-write PRD + ADRs; mkdocs excludes governance docs (fleet standard)
- use verbatim Apache-2.0 license text
- MkDocs site + Pages deploy (fleet standard)

### Fixed

- *(ci)* unbreak main — gate alloc-only tests, cover all opcode arms, run fuzz on nightly

## 0.1.0 — unreleased

- Initial release: length-tolerant pure-Rust Apple LZVN decoder.
- `decode_into` / `decode` for raw LZVN streams; stops at the end-of-stream
  opcode and ignores trailing bytes (reads real `decmpfs` resource-fork blocks).
- `no_std`, `#![forbid(unsafe_code)]`, zero dependencies, typed `Error`.
- Validated against real macOS 26.5 LZVN blocks with Apple's `COMPRESSION_LZVN`
  as the oracle; fuzz target `decode`.
