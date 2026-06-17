//! Safe, dependency-free, pure-Rust **LZVN decompressor**.
//!
//! Decodes raw Apple **LZVN** streams — the codec macOS uses for HFS+/APFS
//! transparent compression (`decmpfs` types 7 inline and 8 resource-fork) and
//! as the `bvxn` block type inside an LZFSE stream. The output length must be
//! known (or upper-bounded) by the caller, exactly like Apple's
//! `lzvn_decode_buffer`.
//!
//! **Length-tolerant by design.** A real-world `decmpfs` resource-fork block
//! ends with the LZVN end-of-stream opcode (`0x06`) and is then followed by
//! arbitrary trailing bytes (macOS leaves 80–300 bytes of them per block; the
//! kernel and Apple's `lzvn_decode_buffer` ignore them). [`decode_into`] stops
//! at the end-of-stream marker and returns — it does **not** reject the
//! trailing bytes, unlike a strict whole-stream decoder. This is the property
//! that lets it read genuine macOS system files; strict Rust LZVN/LZFSE
//! decoders reject those blocks.
//!
//! Built decoder-first for untrusted input: `#![forbid(unsafe_code)]`, zero
//! dependencies, fuzz-hardened — a malformed or crafted block returns a typed
//! [`Error`] rather than reading out of bounds or panicking.
//!
//! ```
//! // A minimal LZVN stream: large-literal opcode `0xe5` (5 literals) then EOS.
//! let block = [0xe5, b'h', b'e', b'l', b'l', b'o', 0x06, 0, 0, 0, 0, 0, 0, 0];
//! let mut out = [0u8; 5];
//! let n = lzvn::decode_into(&block, &mut out).unwrap();
//! assert_eq!(&out[..n], b"hello");
//! ```
#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt;

/// An error returned while decoding an LZVN stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The compressed input ended before an opcode could be fully read.
    TruncatedInput {
        /// Input offset where the truncation was detected.
        position: usize,
    },
    /// Decoding would write past the end of the output buffer.
    OutputTooSmall {
        /// Bytes written before the overflow.
        written: usize,
        /// Total output capacity.
        capacity: usize,
    },
    /// An opcode byte is not a defined LZVN instruction.
    InvalidOpcode {
        /// Input offset of the bad opcode.
        position: usize,
        /// The offending opcode byte.
        opcode: u8,
    },
    /// A match back-reference distance is zero or points before the output
    /// start (a corrupt block).
    InvalidMatchDistance {
        /// The requested back-reference distance.
        distance: usize,
        /// Output bytes available to reference.
        available: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TruncatedInput { position } => {
                write!(f, "lzvn: input truncated at offset {position}")
            }
            Error::OutputTooSmall { written, capacity } => {
                write!(f, "lzvn: output buffer too small ({written}/{capacity})")
            }
            Error::InvalidOpcode { position, opcode } => {
                write!(f, "lzvn: invalid opcode {opcode:#04x} at offset {position}")
            }
            Error::InvalidMatchDistance {
                distance,
                available,
            } => write!(
                f,
                "lzvn: invalid match distance {distance} (available {available})"
            ),
        }
    }
}

impl core::error::Error for Error {}

/// Result alias for this crate.
pub type Result<T> = core::result::Result<T, Error>;

/// Decode a raw LZVN stream into `dst`, returning the number of bytes written.
///
/// `dst` must be large enough to hold the entire decoded output. Decoding stops
/// at the LZVN end-of-stream opcode (`0x06`); **any bytes after it are
/// ignored** (see the crate docs on length-tolerance), so a `decmpfs`
/// resource-fork block can be passed verbatim, trailing padding and all.
pub fn decode_into(src: &[u8], dst: &mut [u8]) -> Result<usize> {
    Decoder {
        src,
        dst,
        ip: 0,
        op: 0,
        prev_distance: 0,
    }
    .run()
}

/// Decode a raw LZVN stream, allocating an output buffer of exactly
/// `decoded_len` bytes (the uncompressed size, known from the `decmpfs` header
/// or the LZFSE block header). Returns the decoded data.
#[cfg(feature = "alloc")]
pub fn decode(src: &[u8], decoded_len: usize) -> Result<alloc::vec::Vec<u8>> {
    let mut dst = alloc::vec![0u8; decoded_len];
    let n = decode_into(src, &mut dst)?;
    dst.truncate(n);
    Ok(dst)
}

struct Decoder<'a> {
    src: &'a [u8],
    dst: &'a mut [u8],
    ip: usize,
    op: usize,
    prev_distance: usize,
}

impl Decoder<'_> {
    fn run(&mut self) -> Result<usize> {
        loop {
            let opcode = *self
                .src
                .get(self.ip)
                .ok_or(Error::TruncatedInput { position: self.ip })?;
            match opcode {
                // End of stream: stop here. Length-tolerant — we intentionally
                // do NOT inspect or reject the bytes that follow.
                0x06 => return Ok(self.op),
                // No-op padding bytes.
                0x0e | 0x16 => self.ip += 1,
                // Large literal: count in the following byte, base 16.
                0xe0 => {
                    let len = self.byte(self.ip + 1)? as usize + 16;
                    self.literal(2, len)?;
                }
                // Small literal: count in the low nibble.
                0xe1..=0xef => {
                    let len = (opcode & 0x0f) as usize;
                    self.literal(1, len)?;
                }
                // Large match: count in the following byte, base 16, reusing the
                // previous distance.
                0xf0 => {
                    let len = self.byte(self.ip + 1)? as usize + 16;
                    self.match_only(2, len, self.prev_distance)?;
                }
                // Small match: count in the low nibble, previous distance.
                0xf1..=0xff => {
                    let len = (opcode & 0x0f) as usize;
                    self.match_only(1, len, self.prev_distance)?;
                }
                // Medium distance: 3-byte opcode.
                0xa0..=0xbf => {
                    let b1 = self.byte(self.ip + 1)? as usize;
                    let b2 = self.byte(self.ip + 2)? as usize;
                    let lit = ((opcode >> 3) & 0x03) as usize;
                    let mlen = (((opcode & 0x07) as usize) << 2 | (b1 & 0x03)) + 3;
                    let dist = (b1 >> 2) | (b2 << 6);
                    self.literal_and_match(3, lit, mlen, dist)?;
                }
                // Previous-distance opcode set: 1 byte, reuse last distance.
                0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x86 | 0x8e | 0x96 | 0x9e | 0xc6
                | 0xce => {
                    let lit = (opcode >> 6) as usize;
                    let mlen = ((opcode >> 3) & 0x07) as usize + 3;
                    self.literal_and_match(1, lit, mlen, self.prev_distance)?;
                }
                // Large distance: 3-byte opcode, little-endian u16 distance.
                0x07 | 0x0f | 0x17 | 0x1f | 0x27 | 0x2f | 0x37 | 0x3f | 0x47 | 0x4f | 0x57
                | 0x5f | 0x67 | 0x6f | 0x87 | 0x8f | 0x97 | 0x9f | 0xc7 | 0xcf => {
                    let lit = (opcode >> 6) as usize;
                    let mlen = ((opcode >> 3) & 0x07) as usize + 3;
                    let dist =
                        u16::from_le_bytes([self.byte(self.ip + 1)?, self.byte(self.ip + 2)?])
                            as usize;
                    self.literal_and_match(3, lit, mlen, dist)?;
                }
                // Reserved / undefined opcodes.
                0x1e | 0x26 | 0x2e | 0x36 | 0x3e | 0x70..=0x7f | 0xd0..=0xdf => {
                    return Err(Error::InvalidOpcode {
                        position: self.ip,
                        opcode,
                    })
                }
                // Everything else: small distance, 2-byte opcode.
                _ => {
                    let b1 = self.byte(self.ip + 1)? as usize;
                    let lit = (opcode >> 6) as usize;
                    let mlen = ((opcode >> 3) & 0x07) as usize + 3;
                    let dist = (((opcode & 0x07) as usize) << 8) | b1;
                    self.literal_and_match(2, lit, mlen, dist)?;
                }
            }
        }
    }

    fn byte(&self, position: usize) -> Result<u8> {
        self.src
            .get(position)
            .copied()
            .ok_or(Error::TruncatedInput { position })
    }

    fn need_input(&self, needed: usize) -> Result<()> {
        if self.src.len().saturating_sub(self.ip) < needed {
            return Err(Error::TruncatedInput { position: self.ip });
        }
        Ok(())
    }

    fn need_output(&self, needed: usize) -> Result<()> {
        if self.dst.len().saturating_sub(self.op) < needed {
            return Err(Error::OutputTooSmall {
                written: self.op,
                capacity: self.dst.len(),
            });
        }
        Ok(())
    }

    fn literal(&mut self, opcode_len: usize, len: usize) -> Result<()> {
        self.need_input(opcode_len + len)?;
        self.need_output(len)?;
        let start = self.ip + opcode_len;
        self.dst[self.op..self.op + len].copy_from_slice(&self.src[start..start + len]);
        self.ip = start + len;
        self.op += len;
        Ok(())
    }

    fn match_only(&mut self, opcode_len: usize, len: usize, distance: usize) -> Result<()> {
        self.need_input(opcode_len)?;
        self.need_output(len)?;
        self.copy_match(len, distance)?;
        self.ip += opcode_len;
        Ok(())
    }

    fn literal_and_match(
        &mut self,
        opcode_len: usize,
        lit: usize,
        mlen: usize,
        distance: usize,
    ) -> Result<()> {
        self.need_input(opcode_len + lit)?;
        self.need_output(lit + mlen)?;
        let start = self.ip + opcode_len;
        self.dst[self.op..self.op + lit].copy_from_slice(&self.src[start..start + lit]);
        self.ip = start + lit;
        self.op += lit;
        self.copy_match(mlen, distance)
    }

    /// Copy a `len`-byte match `distance` bytes behind the output cursor,
    /// byte-by-byte so an overlapping copy repeats correctly (LZ77).
    fn copy_match(&mut self, len: usize, distance: usize) -> Result<()> {
        if distance == 0 || distance > self.op {
            return Err(Error::InvalidMatchDistance {
                distance,
                available: self.op,
            });
        }
        self.need_output(len)?;
        let start = self.op - distance;
        for i in 0..len {
            self.dst[self.op + i] = self.dst[start + i];
        }
        self.op += len;
        self.prev_distance = distance;
        Ok(())
    }
}
