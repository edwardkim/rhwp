//! Request-bounded PNG output for 8-bit grayscale, RGB and RGBA samples.
//!
//! Borrowed rows feed a streaming zlib compressor. Its private allocation goes
//! through a request-scoped, fallible allocator; compressed chunks go directly
//! to the bounded output writer. No complete filtered/compressed image is held.

use crate::error::{Error, Result};
use crate::import_limits::ImportBudget;
use crate::zlib_limited::{Memory, Stream};
use std::io::Write;
use zlib_rs::ReturnCode;

const BLOCK_BYTES: usize = 16 * 1024;

pub(super) fn encode(
    writer: &mut impl Write,
    pixels: &[u8],
    width: u32,
    height: u32,
    channels: usize,
    budget: &ImportBudget,
    interpolate: Option<bool>,
) -> Result<()> {
    budget.check()?;
    if width == 0 || height == 0 || ![1, 3, 4].contains(&channels) {
        return Err(Error::Image("invalid bounded PNG geometry or channels".into()));
    }
    let row_bytes = (width as usize)
        .checked_mul(channels)
        .ok_or_else(|| budget.reject("PNG row size overflow"))?;
    if row_bytes.checked_mul(height as usize) != Some(pixels.len()) {
        return Err(Error::Image("invalid bounded PNG sample length".into()));
    }
    let mut memory = Memory::new(budget);
    let mut compressor = Stream::deflater(&mut memory)?;
    writer.write_all(b"\x89PNG\r\n\x1a\n")?;
    let mut ihdr = [0u8; 13];
    ihdr[..4].copy_from_slice(&width.to_be_bytes());
    ihdr[4..8].copy_from_slice(&height.to_be_bytes());
    ihdr[8] = 8;
    ihdr[9] = match channels {
        1 => 0,
        3 => 2,
        4 => 6,
        _ => unreachable!("validated above"),
    };
    chunk(writer, b"IHDR", &ihdr, &[])?;
    // Private, ancillary, unsafe-to-copy PNG chunk. Version 6 preserves PDF
    // output colours separately from optional source interpolation: 0/1 are
    // explicit nearest/linear, 2 leaves the transformed bitmap's sampling to
    // the renderer. Hancom's bitmap print conversion must not recolour this
    // already converted PDF image. Foreign pixel editors must discard it.
    chunk(writer, b"rhWP", &[6, interpolate.map_or(2, u8::from)], &[])?;
    for row in pixels.chunks_exact(row_bytes) {
        // PNG filter None; the source samples remain borrowed.
        feed(&mut compressor, &[0], false, writer)?;
        // z_stream lengths are u32 even on a 64-bit host.
        for part in row.chunks(u32::MAX as usize) {
            feed(&mut compressor, part, false, writer)?;
        }
    }
    feed(&mut compressor, &[], true, writer)?;
    chunk(writer, b"IEND", &[], &[])?;
    Ok(())
}

fn feed(
    compressor: &mut Stream<'_>,
    mut input: &[u8],
    finish: bool,
    writer: &mut impl Write,
) -> Result<()> {
    let mut output = [0u8; BLOCK_BYTES];
    loop {
        let (status, consumed, written) = compressor.step(input, &mut output, finish)?;
        input = &input[consumed..];
        if written != 0 {
            chunk(writer, b"IDAT", &[], &output[..written])?;
        }
        if status == ReturnCode::StreamEnd && finish && input.is_empty() {
            return Ok(());
        }
        if status != ReturnCode::Ok {
            return Err(Error::Image("bounded PNG compression failed".into()));
        }
        if !finish && input.is_empty() {
            return Ok(());
        }
        if consumed == 0 && written == 0 {
            return Err(Error::Image("bounded PNG compressor made no progress".into()));
        }
    }
}

fn chunk(writer: &mut impl Write, kind: &[u8; 4], prefix: &[u8], data: &[u8]) -> Result<()> {
    // All callers pass a fixed header or at most BLOCK_BYTES compressed bytes.
    writer.write_all(&((prefix.len() + data.len()) as u32).to_be_bytes())?;
    writer.write_all(kind)?;
    writer.write_all(prefix)?;
    writer.write_all(data)?;
    let mut crc = u32::MAX;
    for &byte in kind.iter().chain(prefix).chain(data) {
        crc = CRC_TABLE[((crc ^ u32::from(byte)) & 255) as usize] ^ (crc >> 8);
    }
    writer.write_all(&(!crc).to_be_bytes())?;
    Ok(())
}

// PNG section 5.5, reflected CRC-32. Compile-time table, no heap/lazy init.
const CRC_TABLE: [u32; 256] = {
    let mut table = [0; 256];
    let mut i = 0;
    while i < table.len() {
        let mut value = i as u32;
        let mut bit = 0;
        while bit < 8 {
            value = (value >> 1) ^ if value & 1 != 0 { 0xedb88320 } else { 0 };
            bit += 1;
        }
        table[i] = value;
        i += 1;
    }
    table
};
