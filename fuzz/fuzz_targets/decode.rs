#![no_main]

use libfuzzer_sys::fuzz_target;

// The decoder must never panic on arbitrary input — only return a typed error.
// The output buffer is fixed; OutputTooSmall is a normal, expected result.
fuzz_target!(|data: &[u8]| {
    let mut out = [0u8; 65536];
    let _ = lzvn::decode_into(data, &mut out);
});
