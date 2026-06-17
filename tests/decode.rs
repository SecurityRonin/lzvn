//! Validation against **real macOS Tahoe 26.5 LZVN blocks**, extracted from
//! `decmpfs` type-8 resource forks on a read-only-mounted system volume. Each
//! `.expected` was produced by Apple's own `COMPRESSION_LZVN` (`libcompression`)
//! — an independent oracle. These blocks carry trailing bytes after the LZVN
//! end-of-stream opcode, which is exactly the case a strict decoder rejects.

use std::fs;
use std::path::Path;

fn fixture(name: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let block = fs::read(dir.join(format!("{name}.lzvn"))).expect("block fixture");
    let expected = fs::read(dir.join(format!("{name}.expected"))).expect("expected fixture");
    (block, expected)
}

fn check(name: &str) {
    let (block, expected) = fixture(name);
    let mut out = vec![0u8; expected.len()];
    let n = lzvn::decode_into(&block, &mut out).expect("decode_into");
    assert_eq!(n, expected.len(), "{name}: decoded length");
    assert_eq!(
        out, expected,
        "{name}: decoded content matches Apple oracle"
    );
}

#[test]
fn decodes_real_tahoe_single_block_with_trailing_bytes() {
    // 1180-byte block (LZVN stream + trailing padding) -> 1936 bytes.
    check("single_block_b0");
}

#[test]
fn decodes_real_tahoe_multi_block_file() {
    // Five 64 KiB chunks of a real binary, including a highly-compressible
    // 139-byte tail block that expands to 12 336 bytes.
    for i in 0..5 {
        check(&format!("multi_block_b{i}"));
    }
}

#[test]
fn decode_alloc_convenience_matches() {
    let (block, expected) = fixture("single_block_b0");
    let out = lzvn::decode(&block, expected.len()).expect("decode");
    assert_eq!(out, expected);
}
