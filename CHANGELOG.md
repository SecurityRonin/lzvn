# Changelog

## [0.1.3](https://github.com/SecurityRonin/lzvn/compare/lzvn-core-v0.1.2...lzvn-core-v0.1.3) - 2026-08-20

### Fixed

- *(gitignore)* unanchor the target rule so nested cargo projects are ignored

## [0.1.2](https://github.com/SecurityRonin/lzvn/compare/lzvn-core-v0.1.1...lzvn-core-v0.1.2) - 2026-08-05

### Documentation

- *(terms)* state Apache-2.0, the licence this repo actually ships

### Other

- complete the canonical lints block

## [0.1.1](https://github.com/SecurityRonin/lzvn/compare/lzvn-core-v0.1.0...lzvn-core-v0.1.1) - 2026-07-25

### Documentation

- reverse-write PRD + ADRs; mkdocs excludes governance docs (fleet standard)
- use verbatim Apache-2.0 license text
- MkDocs site + Pages deploy (fleet standard)

### Fixed

- *(vet)* declare own crates first-party so version bumps don't break supply-chain audit
- *(ci)* unbreak main — gate alloc-only tests, cover all opcode arms, run fuzz on nightly

## 0.1.0 — unreleased

- Initial release: length-tolerant pure-Rust Apple LZVN decoder.
- `decode_into` / `decode` for raw LZVN streams; stops at the end-of-stream
  opcode and ignores trailing bytes (reads real `decmpfs` resource-fork blocks).
- `no_std`, `#![forbid(unsafe_code)]`, zero dependencies, typed `Error`.
- Validated against real macOS 26.5 LZVN blocks with Apple's `COMPRESSION_LZVN`
  as the oracle; fuzz target `decode`.
