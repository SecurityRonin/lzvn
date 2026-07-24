//! Each fixture is a **real Apple LZVN stream** produced by Apple's own
//! `COMPRESSION_LZVN` encoder (`libcompression`) from the bytes in
//! `<name>.expected`, then **padded with trailing bytes after the end-of-stream
//! opcode** — the exact shape of a `decmpfs` resource-fork block, which strict
//! whole-stream decoders reject. The inputs are synthetic (so the fixtures are
//! freely redistributable); the decoder was *additionally* validated against 25
//! genuine macOS 26.5 system-file blocks against the same Apple oracle, and that
//! real-artifact corpus lives with the filesystem reader (hfsplus-forensic).

use std::fs;
use std::path::Path;

fn check(name: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let block = fs::read(dir.join(format!("{name}.lzvn"))).expect("block fixture");
    let expected = fs::read(dir.join(format!("{name}.expected"))).expect("expected fixture");
    let mut out = vec![0u8; expected.len()];
    let n = lzvn::decode_into(&block, &mut out).expect("decode_into");
    assert_eq!(n, expected.len(), "{name}: decoded length");
    assert_eq!(
        out, expected,
        "{name}: decoded content matches Apple-encoded input"
    );
}

#[test]
fn decodes_small_text() {
    check("text_small");
}

#[test]
fn decodes_heavy_match_overlap() {
    check("text_repeats");
}

#[test]
fn decodes_mixed_literals_and_matches() {
    check("mixed");
}

#[test]
fn decodes_low_compressibility() {
    check("near_random");
}

#[cfg(feature = "alloc")]
#[test]
fn decode_alloc_convenience() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let block = fs::read(dir.join("mixed.lzvn")).expect("block");
    let expected = fs::read(dir.join("mixed.expected")).expect("expected");
    assert_eq!(
        lzvn::decode(&block, expected.len()).expect("decode"),
        expected
    );
}
