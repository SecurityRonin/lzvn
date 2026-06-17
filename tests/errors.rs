//! Spec-derived behaviour + every error path. A forensic decoder must return a
//! typed error on malformed input, never panic or read out of bounds.

use lzvn::{decode, decode_into, Error};

#[test]
fn small_literal_then_eos() {
    // 0xe5 = small-literal opcode, 5 literals; then EOS.
    let block = [0xe5, b'h', b'e', b'l', b'l', b'o', 0x06];
    let mut out = [0u8; 5];
    assert_eq!(decode_into(&block, &mut out).unwrap(), 5);
    assert_eq!(&out, b"hello");
}

#[test]
fn trailing_bytes_after_eos_are_ignored() {
    // The length-tolerance contract: bytes after the EOS opcode are not read.
    let block = [0xe3, b'a', b'b', b'c', 0x06, 0xde, 0xad, 0xbe, 0xef];
    let mut out = [0u8; 3];
    assert_eq!(decode_into(&block, &mut out).unwrap(), 3);
    assert_eq!(&out, b"abc");
}

#[test]
fn nop_opcodes_are_skipped() {
    let block = [0x0e, 0x16, 0xe2, b'h', b'i', 0x06];
    let mut out = [0u8; 2];
    assert_eq!(decode_into(&block, &mut out).unwrap(), 2);
    assert_eq!(&out, b"hi");
}

#[test]
fn match_repeats_with_overlap() {
    // 3 literals "abc", then a small-distance match (len 3, distance 3) that
    // copies "abc" again -> "abcabc". 0x86 = lit 2? build via large-distance.
    // Use: small-literal 0xe1 'a', then large-distance opcode 0x07 (lit 0, len 3)
    // with distance 1 to repeat the last byte.
    let block = [0xe1, b'a', 0x07, 0x01, 0x00, 0x06];
    let mut out = [0u8; 4];
    let n = decode_into(&block, &mut out).unwrap();
    assert_eq!(&out[..n], b"aaaa");
}

#[test]
fn empty_input_is_truncated() {
    let mut out = [0u8; 0];
    assert!(matches!(
        decode_into(&[], &mut out),
        Err(Error::TruncatedInput { position: 0 })
    ));
}

#[test]
fn truncated_literal_payload() {
    let block = [0xe5, b'h', b'i']; // claims 5 literals, only 2 present
    let mut out = [0u8; 5];
    assert!(matches!(
        decode_into(&block, &mut out),
        Err(Error::TruncatedInput { .. })
    ));
}

#[test]
fn invalid_opcode_rejected() {
    let block = [0x70u8]; // 0x70..=0x7f are reserved/undefined
    let mut out = [0u8; 8];
    assert!(matches!(
        decode_into(&block, &mut out),
        Err(Error::InvalidOpcode {
            opcode: 0x70,
            position: 0
        })
    ));
}

#[test]
fn output_too_small() {
    let block = [0xe5, b'h', b'e', b'l', b'l', b'o', 0x06];
    let mut out = [0u8; 3]; // needs 5
    assert!(matches!(
        decode_into(&block, &mut out),
        Err(Error::OutputTooSmall { capacity: 3, .. })
    ));
}

#[test]
fn match_distance_zero_rejected() {
    // Small-match opcode with no prior output -> prev_distance 0 -> invalid.
    let block = [0xf3u8];
    let mut out = [0u8; 8];
    assert!(matches!(
        decode_into(&block, &mut out),
        Err(Error::InvalidMatchDistance { distance: 0, .. })
    ));
}

#[test]
fn error_display_is_nonempty() {
    for e in [
        Error::TruncatedInput { position: 1 },
        Error::OutputTooSmall {
            written: 1,
            capacity: 2,
        },
        Error::InvalidOpcode {
            position: 0,
            opcode: 0x70,
        },
        Error::InvalidMatchDistance {
            distance: 0,
            available: 0,
        },
    ] {
        assert!(!format!("{e}").is_empty());
    }
}

#[test]
fn decode_alloc_helper() {
    let block = [0xe5, b'h', b'e', b'l', b'l', b'o', 0x06];
    assert_eq!(decode(&block, 5).unwrap(), b"hello");
}
