//! .sav container reader translated from agevak/CM0102/CM0102Core/Save/
//! `SaveReader.cs`.
//!
//! **File layout** per agevak:
//! - `+0` u32 — compression flag (1 = uncompressed, 4 = compressed)
//! - `+4` u32 — unknown header bytes
//! - `+8` u32 — block count
//! - then `block_count` × 268-byte block-directory entries (see [`SavBlockHeader`])
//! - then the block data, laid out at each block's declared `position`
//!
//! **Block-data compression** — a simple RLE scheme applied per-block
//! when the compression flag is 4:
//! - byte `b <= 128` → one literal byte value `b`
//! - byte `b > 128`  → next byte, repeated `b - 128` times
//!
//! ## Cross-check vs [[sav-file-format]]
//!
//! Our memory ledger describes the section directory as
//! `{field_a, offset, size, name}` at 0x10c-byte stride. Agevak's Block
//! header has `{position, size, name}` at 268 bytes = 0x10c stride — the
//! stride matches exactly, but agevak has NO `field_a` and places the
//! offset at +0. Either the ledger's `field_a` is agevak's `position`
//! renamed, or one of the two decodes is off by 4 bytes. Flagged for
//! future verification against a real .sav.

use std::io::{self, Read, Seek, SeekFrom};

/// Size of a single block directory entry.
pub const BLOCK_HEADER_SIZE: usize = 268;    // 0x10c

/// The 268-byte directory entry preceding each block's data.
#[derive(Debug, Clone)]
pub struct SavBlockHeader {
    /// File offset where this block's raw data starts.
    pub position: u32,
    /// Uncompressed size (bytes) of this block's data.
    pub size: u32,
    /// Null-terminated ASCII name (up to 260 bytes).
    pub name: String,
    /// The raw 268-byte header exactly as read from disk.
    pub raw: [u8; BLOCK_HEADER_SIZE],
}

impl SavBlockHeader {
    /// Parse a 268-byte header slice into a typed [`SavBlockHeader`].
    pub fn from_bytes(bytes: &[u8; BLOCK_HEADER_SIZE]) -> Self {
        let position = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let size     = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        // Name is ASCII, null-terminated, starting at +8.
        let name_end = bytes[8..].iter().position(|&b| b == 0).unwrap_or(260) + 8;
        let name = String::from_utf8_lossy(&bytes[8..name_end]).into_owned();
        Self { position, size, name, raw: *bytes }
    }
}

/// A parsed .sav container — header + block directory. Block DATA is
/// loaded lazily on demand via [`SavReader::read_block_data`].
pub struct SavReader<R: Read + Seek> {
    /// True if the compression flag in the header was 4.
    pub was_compressed: bool,
    /// The 4 bytes of unknown header that follow the compression flag.
    pub unknown_header: u32,
    /// The block-directory entries in file order.
    pub blocks: Vec<SavBlockHeader>,
    /// The stream we were opened on — kept for lazy data reads.
    stream: R,
}

impl<R: Read + Seek> SavReader<R> {
    /// Parse the header and block directory from `stream`. Does not
    /// touch block data — call [`Self::read_block_data`] for that.
    pub fn open(mut stream: R) -> io::Result<Self> {
        let mut u32_buf = [0u8; 4];

        stream.read_exact(&mut u32_buf)?;
        let flag = u32::from_le_bytes(u32_buf);
        let was_compressed = flag == 4;

        stream.read_exact(&mut u32_buf)?;
        let unknown_header = u32::from_le_bytes(u32_buf);

        stream.read_exact(&mut u32_buf)?;
        let block_count = u32::from_le_bytes(u32_buf) as usize;

        let mut blocks = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            let mut hdr = [0u8; BLOCK_HEADER_SIZE];
            stream.read_exact(&mut hdr)?;
            blocks.push(SavBlockHeader::from_bytes(&hdr));
        }

        Ok(Self { was_compressed, unknown_header, blocks, stream })
    }

    /// Read one block's data by index. Applies RLE decompression if the
    /// container's compression flag was 4.
    pub fn read_block_data(&mut self, index: usize) -> io::Result<Vec<u8>> {
        let block = &self.blocks[index];
        self.stream.seek(SeekFrom::Start(block.position as u64))?;

        if !self.was_compressed {
            // Fast path — raw copy.
            let mut buf = vec![0u8; block.size as usize];
            self.stream.read_exact(&mut buf)?;
            return Ok(buf);
        }

        // RLE decompression — read bytes until we've filled `size` output.
        let mut out = Vec::with_capacity(block.size as usize);
        while out.len() < block.size as usize {
            let mut b = [0u8; 1];
            self.stream.read_exact(&mut b)?;
            let byte = b[0];
            if byte <= 128 {
                out.push(byte);
            } else {
                let repeat = (byte - 128) as usize;
                self.stream.read_exact(&mut b)?;
                let value = b[0];
                for _ in 0..repeat {
                    if out.len() == block.size as usize { break; }
                    out.push(value);
                }
            }
        }

        Ok(out)
    }

    /// Find a block by ASCII name and read its data. Returns `None`
    /// if no block has that name.
    pub fn read_named_block(&mut self, name: &str) -> io::Result<Option<Vec<u8>>> {
        let idx = self.blocks.iter().position(|b| b.name == name);
        match idx {
            Some(i) => Ok(Some(self.read_block_data(i)?)),
            None => Ok(None),
        }
    }
}

/// Decode a null-terminated ISO-8859-1 (Latin-1) byte slice into a
/// `String`. Every byte 0x00..=0xFF maps to the equivalent U+00XX
/// codepoint, so this is a lossless direct translation.
pub fn latin1_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| b as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_header(compressed: bool, blocks: &[(u32, u32, &str)]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(if compressed { 4u32 } else { 1u32 }).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // unknown
        out.extend_from_slice(&(blocks.len() as u32).to_le_bytes());
        for (pos, size, name) in blocks {
            let mut hdr = [0u8; BLOCK_HEADER_SIZE];
            hdr[..4].copy_from_slice(&pos.to_le_bytes());
            hdr[4..8].copy_from_slice(&size.to_le_bytes());
            let name_bytes = name.as_bytes();
            hdr[8..8 + name_bytes.len()].copy_from_slice(name_bytes);
            out.extend_from_slice(&hdr);
        }
        out
    }

    #[test]
    fn parse_empty_directory() {
        let bytes = make_header(false, &[]);
        let reader = SavReader::open(Cursor::new(bytes)).unwrap();
        assert!(!reader.was_compressed);
        assert_eq!(reader.blocks.len(), 0);
    }

    #[test]
    fn parse_uncompressed_two_blocks_and_read_them() {
        // 12-byte header + 2*268-byte directory + 2 data payloads
        let dir_end = 12 + 2 * BLOCK_HEADER_SIZE;
        let pos_a = dir_end as u32;
        let pos_b = (dir_end + 5) as u32;
        let mut bytes = make_header(false, &[(pos_a, 5, "alpha"), (pos_b, 3, "beta")]);
        bytes.extend_from_slice(b"HELLO");
        bytes.extend_from_slice(b"BYE");

        let mut reader = SavReader::open(Cursor::new(bytes)).unwrap();
        assert_eq!(reader.blocks[0].name, "alpha");
        assert_eq!(reader.blocks[1].name, "beta");

        let alpha = reader.read_block_data(0).unwrap();
        assert_eq!(&alpha, b"HELLO");

        let beta = reader.read_named_block("beta").unwrap().unwrap();
        assert_eq!(&beta, b"BYE");

        let missing = reader.read_named_block("gamma").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn rle_decompresses_repeat_marker() {
        // Compressed block whose data is: literal 'A', run of 5 'X', literal 'B'
        // Encoding: 0x41 (literal 'A'), 0x85 (128+5), 0x58 ('X'), 0x42 ('B') → 7 out bytes
        let dir_end = 12 + BLOCK_HEADER_SIZE;
        let pos = dir_end as u32;
        let mut bytes = make_header(true, &[(pos, 7, "rle")]);
        bytes.push(0x41);          // literal A
        bytes.push(128 + 5);       // run marker: repeat 5 times
        bytes.push(0x58);          // the byte to repeat: X
        bytes.push(0x42);          // literal B

        let mut reader = SavReader::open(Cursor::new(bytes)).unwrap();
        assert!(reader.was_compressed);
        let data = reader.read_block_data(0).unwrap();
        assert_eq!(data, b"AXXXXXB");
    }

    #[test]
    fn rle_handles_boundary_between_literal_128_and_repeat_129() {
        // 128 is the last literal value; 129 is the first repeat count.
        // Sanity-check both.
        let dir_end = 12 + BLOCK_HEADER_SIZE;
        let pos = dir_end as u32;
        let mut bytes = make_header(true, &[(pos, 3, "b")]);
        bytes.push(128);           // literal 128
        bytes.push(129);           // run marker: repeat 1 time
        bytes.push(0x7F);          // the byte to repeat once
        let mut reader = SavReader::open(Cursor::new(bytes)).unwrap();
        // Output: [128, 0x7F, ???] — but we only asked for 3 bytes; last
        // one falls through with no more input. Actually we asked for 3
        // and produced 2 so the loop would read one more marker; let's
        // instead ask for 2.
        // Rewrite: expect size 2, output = [128, 0x7F].
        let block = &mut reader.blocks[0];
        block.size = 2;
        let data = reader.read_block_data(0).unwrap();
        assert_eq!(data, [128, 0x7F]);
    }

    #[test]
    fn latin1_decode() {
        // Includes bytes >= 0x80 (Latin-1) and a null terminator.
        let s = latin1_to_string(&[b'C', b'a', b'f', 0xE9, 0, 0xFF, 0xFF]);
        assert_eq!(s, "Café");
    }

    #[test]
    fn block_header_parses_position_size_name() {
        let mut hdr = [0u8; BLOCK_HEADER_SIZE];
        hdr[..4].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        hdr[4..8].copy_from_slice(&0x0000_1000u32.to_le_bytes());
        hdr[8..14].copy_from_slice(b"hello\0");
        let h = SavBlockHeader::from_bytes(&hdr);
        assert_eq!(h.position, 0x1234_5678);
        assert_eq!(h.size, 0x1000);
        assert_eq!(h.name, "hello");
    }
}
