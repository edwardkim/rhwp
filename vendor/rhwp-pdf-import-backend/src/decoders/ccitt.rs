//! CCITTFaxDecode implementation.
//!
//! In-house ITU-T T.6 (Group 4, 2D) CCITT fax decoder for monochrome scanned
//! images, plus the pass-through filter used in the stream-filter chain.
//!
//! The image-decode path (`decode`) replaces a third-party crate that could not
//! honor `/EncodedByteAlign` (the filter has no such hook and its bit reader is
//! private), which made byte-aligned fax scanners decode to garbage/blank pages.
//! It also recovers partial content from truncated/damaged streams instead of
//! discarding the whole page.
//!
//! PDF Spec: ISO 32000-1:2008 §7.4.6 (CCITTFaxDecode); algorithm: ITU-T T.6/T.4.

use crate::decoders::{CcittParams, StreamDecoder};
use crate::error::{Error, Result};
use crate::extractors::ccitt_bilevel::{append_transition_row, transitions_to_bytes};
use crate::import_limits::ImportBudget;

/// CCITTFaxDecode stream filter (pass-through).
///
/// The raw CCITT codestream is kept compressed in the filter chain; actual
/// image decompression happens in `decode` at image-extraction time.
pub struct CcittFaxDecoder;

impl StreamDecoder for CcittFaxDecoder {
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        log::debug!("CCITTFaxDecode: Pass-through {} bytes", input.len());
        Ok(input.to_vec())
    }

    fn name(&self) -> &str {
        "CCITTFaxDecode"
    }
}

/// Outcome of an image decode: packed bilevel rows plus how the stream ended,
/// so the caller can distinguish a clean decode from recovered-partial content.
pub struct CcittDecoded {
    /// Packed bilevel bytes, `ceil(columns/8)` per row, MSB = leftmost pixel,
    /// `0 = white, 1 = black` (BEFORE any `BlackIs1` inversion). Padded with
    /// white rows to `Rows` when `Rows` is known.
    pub data: Vec<u8>,
    /// Rows actually decoded from the codestream (excludes white padding).
    pub rows_decoded: usize,
    /// True when the stream was truncated/damaged and the tail is recovered
    /// (white-padded) rather than a clean full-height / EOFB decode.
    pub recovered_partial: bool,
}

/// Decode a finite image without unmetered fax fallback or white substitution.
/// The caller supplies positive Rows from the validated image geometry.
pub fn decode_with_budget(
    data: &[u8],
    params: &CcittParams,
    budget: &ImportBudget,
) -> Result<CcittDecoded> {
    budget.check()?;
    let width = u16::try_from(params.columns)
        .ok()
        .filter(|&width| width != 0)
        .ok_or_else(|| Error::Unsupported("limited CCITT requires 1..65535 columns".into()))?;
    let rows = params
        .rows
        .filter(|&rows| rows != 0)
        .ok_or_else(|| Error::Unsupported("limited CCITT requires finite image rows".into()))?;
    let bytes_per_row = (width as usize).div_ceil(8);
    bytes_per_row
        .checked_mul(rows as usize)
        .filter(|&bytes| bytes <= budget.limits.stream_bytes)
        .ok_or_else(|| budget.reject("CCITT packed output bytes"))?;
    if data.len() > budget.limits.input_bytes || data.len().checked_mul(8).is_none() {
        return Err(budget.reject("CCITT input bytes"));
    }
    let mut reader = BitReader::new(data);
    let mut reference = transition_buffer(width, Some(budget))?;
    let mut current = transition_buffer(width, Some(budget))?;
    let mut output = budget.output();
    let mut byte_align = params.encoded_byte_align;
    for row in 0..rows {
        if byte_align && row != 0 {
            byte_align_skip(&mut reader, &mut byte_align);
        }
        let one_d = if params.is_group_4() {
            false
        } else {
            if skip_eols(&mut reader) >= 6 {
                return Err(Error::Decode("CCITT ended before image height".into()));
            }
            if params.k > 0 {
                let tag = reader
                    .peek(1)
                    .ok_or_else(|| Error::Decode("truncated CCITT row tag".into()))?;
                reader.consume(1);
                tag == 1
            } else {
                true
            }
        };
        if reader.eod() {
            return Err(Error::Decode("truncated CCITT image".into()));
        }
        current.clear();
        let status = if one_d {
            decode_row_g3_1d(&mut reader, width, &mut current, Some(budget))
        } else {
            decode_row_g4(&mut reader, &reference, width, &mut current, Some(budget))
        };
        budget.check()?;
        if !matches!(status, RowStatus::Ok) {
            return Err(Error::Decode("CCITT row requires unsupported recovery".into()));
        }
        budget.charge_codec_allocation(bytes_per_row)?;
        let packed = transitions_to_bytes(&current, width as usize)
            .map_err(|_| budget.reject("CCITT row allocation"))?;
        output.append(&packed)?;
        std::mem::swap(&mut reference, &mut current);
    }
    budget.check()?;
    Ok(CcittDecoded {
        data: output.finish(),
        rows_decoded: rows as usize,
        recovered_partial: false,
    })
}

/// Decode a CCITT codestream to packed bilevel bytes, dispatching on `/K`:
/// Group 4 (T.6, `K < 0`) here, Group 3 (T.4, `K >= 0`) in [`decode_g3`].
///
/// Returns `Err` only when zero usable rows could be produced (genuinely
/// undecodable) — the caller may then fall back.
/// `BlackIs1` is NOT applied here; the caller owns that inversion.
pub fn decode(data: &[u8], params: &CcittParams) -> Result<CcittDecoded> {
    let width = params.columns as u16;
    if width == 0 {
        return Err(Error::Decode("CCITT decode requires /Columns".to_string()));
    }
    if !params.is_group_4() {
        // Group 3: K == 0 is pure 1-D (Modified Huffman); K > 0 is mixed 1-D/2-D.
        return decode_g3(data, params);
    }

    let bytes_per_row = (width as usize).div_ceil(8);
    let mut reader = BitReader::new(data);
    let mut reference = transition_buffer(width, None)?; // imaginary all-white line above row 0
    let mut current = transition_buffer(width, None)?;
    let mut out = packed_output_buffer(bytes_per_row, params.rows)?;
    let mut decoded_rows = 0usize;
    let mut byte_align = params.encoded_byte_align;
    let mut recovered = false;

    loop {
        if let Some(h) = params.rows {
            if decoded_rows >= h as usize {
                break;
            }
        }
        // /EncodedByteAlign: each coded row begins on a byte boundary, so skip
        // the zero fill bits left over from the previous row before reading the
        // next row's first mode code (pdfium-guarded: if a skipped bit is 1 the
        // declared alignment is wrong for this stream — disable it rather than
        // corrupt every subsequent row).
        if byte_align && decoded_rows > 0 {
            byte_align_skip(&mut reader, &mut byte_align);
        }
        if reader.eod() {
            if let Some(h) = params.rows {
                if decoded_rows < h as usize {
                    recovered = true; // ran out of data before all rows
                }
            }
            break;
        }

        current.clear();
        match decode_row_g4(&mut reader, &reference, width, &mut current, None) {
            RowStatus::EndOfBlock => break, // clean EOFB
            RowStatus::AllocationFailed => {
                return Err(Error::Decode(
                    "Unable to grow CCITT row transition buffer".to_string(),
                ));
            },
            RowStatus::Error => {
                if decoded_rows >= 1 {
                    recovered = true; // keep the rows we have; do NOT blank the page
                    break;
                }
                return Err(Error::Decode("CCITT: stream undecodable from first row".to_string()));
            },
            RowStatus::Ok => {
                append_transition_row(&mut out, &current, width as usize)?;
                std::mem::swap(&mut reference, &mut current);
                decoded_rows += 1;
            },
        }
    }

    // Pad missing trailing rows with white when the height is known.
    if let Some(h) = params.rows {
        let white_row = transitions_to_bytes::<u16>(&[], width as usize)?;
        while out.len() / bytes_per_row.max(1) < h as usize {
            out.try_reserve(white_row.len()).map_err(|_| {
                Error::Decode(format!(
                    "Unable to grow CCITT output by {} padding bytes",
                    white_row.len()
                ))
            })?;
            out.extend_from_slice(&white_row);
        }
    }

    if out.is_empty() {
        return Err(Error::Decode("CCITT: no output produced".to_string()));
    }

    Ok(CcittDecoded {
        data: out,
        rows_decoded: decoded_rows,
        recovered_partial: recovered,
    })
}

/// Decode a CCITT **Group 3** (T.4) codestream. `params.k == 0` is pure 1-D
/// (Modified Huffman); `params.k > 0` is mixed 1-D/2-D where a tag bit after the
/// line's EOL selects that line's coding (`1` = 1-D, `0` = 2-D-relative). EOL
/// codes (`000000000001`, absorbing any leading fill bits) delimit lines and are
/// tolerated whether or not `/EndOfLine` is declared; six consecutive EOLs is the
/// return-to-control terminator. Shares the run tables, reference-line walk and
/// recovery contract with the Group 4 path. `BlackIs1` is the caller's to apply.
fn decode_g3(data: &[u8], params: &CcittParams) -> Result<CcittDecoded> {
    let width = params.columns as u16;
    let two_dimensional = params.k > 0;
    let bytes_per_row = (width as usize).div_ceil(8);
    let mut reader = BitReader::new(data);
    let mut reference = transition_buffer(width, None)?; // imaginary all-white line above row 0
    let mut current = transition_buffer(width, None)?;
    let mut out = packed_output_buffer(bytes_per_row, params.rows)?;
    let mut decoded_rows = 0usize;
    let mut byte_align = params.encoded_byte_align;
    let mut recovered = false;

    loop {
        if let Some(h) = params.rows {
            if decoded_rows >= h as usize {
                break;
            }
        }
        if byte_align && decoded_rows > 0 {
            byte_align_skip(&mut reader, &mut byte_align);
        }
        // Swallow the line's EOL (plus fill); 6+ in a row is RTC = end of page.
        let eols = skip_eols(&mut reader);
        if reader.eod() {
            if let Some(h) = params.rows {
                if decoded_rows < h as usize {
                    recovered = true;
                }
            }
            break;
        }
        if eols >= 6 {
            break;
        }

        // Mixed 2-D streams prefix every line with a 1-D/2-D tag bit.
        let one_d = if two_dimensional {
            match reader.peek(1) {
                Some(b) => {
                    reader.consume(1);
                    b == 1
                },
                None => break,
            }
        } else {
            true
        };

        current.clear();
        let status = if one_d {
            decode_row_g3_1d(&mut reader, width, &mut current, None)
        } else {
            decode_row_g4(&mut reader, &reference, width, &mut current, None)
        };
        match status {
            RowStatus::EndOfBlock => break,
            RowStatus::AllocationFailed => {
                return Err(Error::Decode(
                    "Unable to grow CCITT row transition buffer".to_string(),
                ));
            },
            RowStatus::Error => {
                if decoded_rows >= 1 {
                    recovered = true; // keep decoded rows; do NOT blank the page
                    break;
                }
                return Err(Error::Decode(
                    "CCITT G3: stream undecodable from first row".to_string(),
                ));
            },
            RowStatus::Ok => {
                append_transition_row(&mut out, &current, width as usize)?;
                std::mem::swap(&mut reference, &mut current);
                decoded_rows += 1;
            },
        }
    }

    if let Some(h) = params.rows {
        let white_row = transitions_to_bytes::<u16>(&[], width as usize)?;
        while out.len() / bytes_per_row.max(1) < h as usize {
            out.try_reserve(white_row.len()).map_err(|_| {
                Error::Decode(format!(
                    "Unable to grow CCITT output by {} padding bytes",
                    white_row.len()
                ))
            })?;
            out.extend_from_slice(&white_row);
        }
    }
    if out.is_empty() {
        return Err(Error::Decode("CCITT G3: no output produced".to_string()));
    }

    Ok(CcittDecoded {
        data: out,
        rows_decoded: decoded_rows,
        recovered_partial: recovered,
    })
}

fn packed_output_buffer(bytes_per_row: usize, rows: Option<u32>) -> Result<Vec<u8>> {
    let Some(rows) = rows else {
        return Ok(Vec::new());
    };
    let rows = usize::try_from(rows)
        .map_err(|_| Error::Decode("CCITT row count exceeds platform limits".to_string()))?;
    let capacity = bytes_per_row
        .checked_mul(rows)
        .ok_or_else(|| Error::Decode("CCITT packed output size overflow".to_string()))?;
    let mut output = Vec::new();
    output.try_reserve_exact(capacity).map_err(|_| {
        Error::Decode(format!("Unable to allocate {capacity} bytes for CCITT packed output"))
    })?;
    Ok(output)
}

fn transition_buffer(width: u16, budget: Option<&ImportBudget>) -> Result<Vec<u16>> {
    let capacity = width as usize;
    if let Some(budget) = budget {
        budget.charge_codec_allocation(capacity * std::mem::size_of::<u16>())?;
    }
    let mut transitions = Vec::new();
    transitions.try_reserve_exact(capacity).map_err(|_| {
        budget.map_or_else(
            || Error::Decode(format!("Unable to allocate {capacity} CCITT row transitions")),
            |budget| budget.reject("CCITT transition allocation"),
        )
    })?;
    Ok(transitions)
}

/// Decode one Group 3 1-D (Modified Huffman) row: runs alternate white→black
/// from the left edge, each a make-up+terminating MH code. Pushes color-flip
/// columns (first run white) into `current`, matching the G4 row contract so the
/// shared `transitions_to_bytes` packs it identically.
fn decode_row_g3_1d(
    reader: &mut BitReader,
    width: u16,
    current: &mut Vec<u16>,
    budget: Option<&ImportBudget>,
) -> RowStatus {
    let mut a0: u16 = 0;
    let mut white = true;
    let mut any = false;
    while a0 < width {
        let run = match read_run(reader, white) {
            Some(v) => v,
            // The run tables can't read an EOL/EOFB or fill: a clean line ends
            // here if we already placed a run, otherwise the row is undecodable.
            None => {
                return if any && budget.is_none() {
                    RowStatus::Ok
                } else {
                    RowStatus::Error
                }
            },
        };
        let a1 = match a0.checked_add(run) {
            Some(v) => v,
            None => return RowStatus::Error,
        };
        any = true;
        if budget.is_some() && a1 > width {
            return RowStatus::Error;
        }
        if a1 >= width {
            break; // final run reaches the right edge; no trailing flip to push
        }
        if !try_push_transition(current, a1, budget) {
            return RowStatus::AllocationFailed;
        }
        a0 = a1;
        white = !white;
    }
    RowStatus::Ok
}

/// Consume a maximal run of EOL codes at the cursor (`>= 11` zero bits — fill
/// included — followed by a `1`), returning how many were eaten. Leaves the
/// reader untouched when the next bits are not an EOL; when a trailing fill of
/// `>= 11` zeros runs into the end of data it is treated as a terminator and
/// the reader is left at EOD.
fn skip_eols(reader: &mut BitReader) -> usize {
    let mut count = 0usize;
    loop {
        let save = reader.bit;
        let mut zeros = 0u32;
        loop {
            match reader.peek(1) {
                Some(0) => {
                    reader.consume(1);
                    zeros += 1;
                },
                Some(_) => break, // the terminating 1 bit of an EOL
                None => {
                    // Ran out of data inside the zero run.
                    if zeros < 11 {
                        reader.bit = save; // not an EOL — leave the cursor put
                    }
                    return count;
                },
            }
        }
        if zeros >= 11 {
            reader.consume(1); // eat the EOL's terminating 1
            count += 1;
        } else {
            reader.bit = save; // a short zero run is real code, not an EOL
            return count;
        }
    }
}

// ---------------------------------------------------------------------------
// Bit reader (MSB-first over a byte slice)
// ---------------------------------------------------------------------------

struct BitReader<'a> {
    data: &'a [u8],
    bit: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader { data, bit: 0 }
    }

    /// Peek the next `n` bits (≤16) MSB-first, or `None` if fewer remain.
    fn peek(&self, n: u8) -> Option<u16> {
        if n == 0 {
            return Some(0);
        }
        if n > 16 || self.bit + n as usize > self.data.len() * 8 {
            return None;
        }
        let mut v: u16 = 0;
        for i in 0..n as usize {
            let bp = self.bit + i;
            let bit = (self.data[bp / 8] >> (7 - (bp % 8))) & 1;
            v = (v << 1) | bit as u16;
        }
        Some(v)
    }

    fn consume(&mut self, n: u8) {
        self.bit += n as usize;
    }

    fn bits_to_byte_boundary(&self) -> u8 {
        ((8 - (self.bit % 8)) % 8) as u8
    }

    fn eod(&self) -> bool {
        self.bit >= self.data.len() * 8
    }
}

/// Skip zero fill to the next byte boundary; disable byte-align if a non-zero
/// fill bit shows the flag is mis-declared for this stream (pdfium behavior).
fn byte_align_skip(reader: &mut BitReader, active: &mut bool) {
    if !*active {
        return;
    }
    let pad = reader.bits_to_byte_boundary();
    for _ in 0..pad {
        match reader.peek(1) {
            Some(0) => reader.consume(1),
            Some(_) => {
                // A 1 in the fill region: the /EncodedByteAlign flag is wrong
                // for this stream — disable it rather than corrupt later rows.
                *active = false;
                return;
            },
            None => return,
        }
    }
}

// ---------------------------------------------------------------------------
// Mode + run-length code decoding
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Mode {
    Pass,
    Horizontal,
    Vertical(i8),
    Extension,
    Eof,
}

/// T.6 2D mode prefix codes (prefix-free, shortest first). Verified against
/// ITU-T T.6 and the `fax` crate tables.
static MODE_CODES: &[(u8, u16, Mode)] = &[
    (1, 0b1, Mode::Vertical(0)),
    (3, 0b001, Mode::Horizontal),
    (3, 0b010, Mode::Vertical(-1)),
    (3, 0b011, Mode::Vertical(1)),
    (4, 0b0001, Mode::Pass),
    (6, 0b000010, Mode::Vertical(-2)),
    (6, 0b000011, Mode::Vertical(2)),
    (7, 0b0000001, Mode::Extension),
    (7, 0b0000010, Mode::Vertical(-3)),
    (7, 0b0000011, Mode::Vertical(3)),
    (12, 0b0000_0000_0001, Mode::Eof),
];

fn read_mode(reader: &mut BitReader) -> Option<Mode> {
    for &(len, code, mode) in MODE_CODES {
        if reader.peek(len) == Some(code) {
            reader.consume(len);
            return Some(mode);
        }
    }
    None
}

/// Read one Modified-Huffman run length for the given color (accumulating
/// make-up codes until a terminating code `< 64`).
fn read_run(reader: &mut BitReader, white: bool) -> Option<u16> {
    let table = if white { WHITE_CODES } else { BLACK_CODES };
    let mut sum: u16 = 0;
    loop {
        let mut n = None;
        for &(len, code, val) in table {
            if reader.peek(len) == Some(code) {
                reader.consume(len);
                n = Some(val);
                break;
            }
        }
        let n = n?;
        sum = sum.checked_add(n)?;
        if n < 64 {
            return Some(sum);
        }
    }
}

// ---------------------------------------------------------------------------
// Reference-line changing-element walk (ported from the verified `fax` crate)
// ---------------------------------------------------------------------------

/// `edges[k]` is a color flip on the reference line; the run before `edges[0]`
/// is white, so `edges[k]` starts black when `k` is even, white when odd.
/// Find the first reference edge right of `start` whose run color is `want_black`
/// (the color opposite the current coding color). Advances `pos` past it.
fn next_color(
    edges: &[u16],
    pos: &mut usize,
    start: u16,
    want_black: bool,
    start_of_row: bool,
) -> Option<u16> {
    if start_of_row {
        if want_black {
            *pos = 1;
            return edges.first().copied();
        }
        *pos = 2;
        return edges.get(1).copied();
    }
    while *pos < edges.len() {
        if edges[*pos] <= start {
            *pos += 1;
            continue;
        }
        if (*pos).is_multiple_of(2) != want_black {
            *pos += 1;
        }
        break;
    }
    if *pos < edges.len() {
        let v = edges[*pos];
        *pos += 1;
        Some(v)
    } else {
        None
    }
}

fn ref_next(edges: &[u16], pos: &mut usize) -> Option<u16> {
    if *pos < edges.len() {
        let v = edges[*pos];
        *pos += 1;
        Some(v)
    } else {
        None
    }
}

fn seek_back(edges: &[u16], pos: &mut usize, start: u16) {
    *pos = (*pos).min(edges.len().saturating_sub(1));
    while *pos > 0 && start < edges[*pos - 1] {
        *pos -= 1;
    }
}

enum RowStatus {
    Ok,
    EndOfBlock,
    Error,
    AllocationFailed,
}

fn try_push_transition(
    transitions: &mut Vec<u16>,
    position: u16,
    budget: Option<&ImportBudget>,
) -> bool {
    if let Some(budget) = budget {
        if transitions.len() == transitions.capacity() {
            let capacity = transitions.capacity().max(1).checked_mul(2);
            let Some((capacity, bytes)) = capacity.and_then(|capacity| {
                capacity
                    .checked_mul(std::mem::size_of::<u16>())
                    .map(|bytes| (capacity, bytes))
            }) else {
                budget.reject("CCITT transition capacity");
                return false;
            };
            if budget.charge_codec_allocation(bytes).is_err() {
                return false;
            }
            if transitions
                .try_reserve_exact(capacity - transitions.len())
                .is_err()
            {
                budget.reject("CCITT transition allocation");
                return false;
            }
        }
        transitions.push(position);
        return true;
    }
    if transitions.try_reserve(1).is_err() {
        return false;
    }
    transitions.push(position);
    true
}

/// Decode one G4 (T.6) row relative to `reference`, pushing color-flip column
/// positions (first run white) into `current`. Ported from the verified `fax`
/// crate `Group4Decoder::advance`.
fn decode_row_g4(
    reader: &mut BitReader,
    reference: &[u16],
    width: u16,
    current: &mut Vec<u16>,
    budget: Option<&ImportBudget>,
) -> RowStatus {
    let mut pos = 0usize; // cursor over reference edges
    let mut a0: u16 = 0;
    let mut black = false; // current coding color (start white)
    let mut start_of_row = true;

    loop {
        let mode = match read_mode(reader) {
            Some(m) => m,
            None => return RowStatus::Error,
        };
        match mode {
            Mode::Pass => {
                if start_of_row && !black {
                    pos += 1;
                } else if next_color(reference, &mut pos, a0, !black, false).is_none() {
                    return RowStatus::Error;
                }
                if let Some(b2) = ref_next(reference, &mut pos) {
                    a0 = b2;
                }
            },
            Mode::Vertical(delta) => {
                let b1 = next_color(reference, &mut pos, a0, !black, start_of_row).unwrap_or(width);
                let a1i = b1 as i32 + delta as i32;
                if a1i < 0 || a1i > width as i32 {
                    if budget.is_some() {
                        return RowStatus::Error;
                    }
                    break; // malformed → end the line, keep what we have
                }
                let a1 = a1i as u16;
                if a1 < width && !try_push_transition(current, a1, budget) {
                    return RowStatus::AllocationFailed;
                }
                black = !black;
                a0 = a1;
                if delta < 0 {
                    seek_back(reference, &mut pos, a0);
                }
            },
            Mode::Horizontal => {
                let r1 = match read_run(reader, !black) {
                    Some(v) => v,
                    None => return RowStatus::Error,
                };
                let r2 = match read_run(reader, black) {
                    Some(v) => v,
                    None => return RowStatus::Error,
                };
                let a1 = match a0.checked_add(r1) {
                    Some(v) => v,
                    None => return RowStatus::Error,
                };
                let a2 = match a1.checked_add(r2) {
                    Some(v) => v,
                    None => return RowStatus::Error,
                };
                if budget.is_some() && a2 > width {
                    return RowStatus::Error;
                }
                if a1 < width && !try_push_transition(current, a1, budget) {
                    return RowStatus::AllocationFailed;
                }
                if a2 >= width {
                    break;
                }
                if !try_push_transition(current, a2, budget) {
                    return RowStatus::AllocationFailed;
                }
                a0 = a2;
            },
            Mode::Extension => return RowStatus::Error,
            Mode::Eof => return RowStatus::EndOfBlock,
        }
        start_of_row = false;
        if a0 >= width {
            break;
        }
    }
    RowStatus::Ok
}

// ---------------------------------------------------------------------------
// Modified-Huffman run tables (generated from ITU-T T.4, verified prefix-free).
// (len_bits, code, run_length); make-up codes have run >= 64.
// ---------------------------------------------------------------------------

#[rustfmt::skip]
static WHITE_CODES: &[(u8, u16, u16)] = &[
    (4,0b0111,2),(4,0b1000,3),(4,0b1011,4),(4,0b1100,5),(4,0b1110,6),(4,0b1111,7),
    (5,0b00111,10),(5,0b01000,11),(5,0b10010,128),(5,0b10011,8),(5,0b10100,9),(5,0b11011,64),
    (6,0b000011,13),(6,0b000111,1),(6,0b001000,12),(6,0b010111,192),(6,0b011000,1664),(6,0b101010,16),
    (6,0b101011,17),(6,0b110100,14),(6,0b110101,15),
    (7,0b0000011,22),(7,0b0000100,23),(7,0b0001000,20),(7,0b0001100,19),(7,0b0010011,26),(7,0b0010111,21),
    (7,0b0011000,28),(7,0b0100100,27),(7,0b0100111,18),(7,0b0101000,24),(7,0b0101011,25),(7,0b0110111,256),
    (8,0b00000010,29),(8,0b00000011,30),(8,0b00000100,45),(8,0b00000101,46),(8,0b00001010,47),(8,0b00001011,48),
    (8,0b00010010,33),(8,0b00010011,34),(8,0b00010100,35),(8,0b00010101,36),(8,0b00010110,37),(8,0b00010111,38),
    (8,0b00011010,31),(8,0b00011011,32),(8,0b00100100,53),(8,0b00100101,54),(8,0b00101000,39),(8,0b00101001,40),
    (8,0b00101010,41),(8,0b00101011,42),(8,0b00101100,43),(8,0b00101101,44),(8,0b00110010,61),(8,0b00110011,62),
    (8,0b00110100,63),(8,0b00110101,0),(8,0b00110110,320),(8,0b00110111,384),(8,0b01001010,59),(8,0b01001011,60),
    (8,0b01010010,49),(8,0b01010011,50),(8,0b01010100,51),(8,0b01010101,52),(8,0b01011000,55),(8,0b01011001,56),
    (8,0b01011010,57),(8,0b01011011,58),(8,0b01100100,448),(8,0b01100101,512),(8,0b01100111,640),(8,0b01101000,576),
    (9,0b010011000,1472),(9,0b010011001,1536),(9,0b010011010,1600),(9,0b010011011,1728),(9,0b011001100,704),
    (9,0b011001101,768),(9,0b011010010,832),(9,0b011010011,896),(9,0b011010100,960),(9,0b011010101,1024),
    (9,0b011010110,1088),(9,0b011010111,1152),(9,0b011011000,1216),(9,0b011011001,1280),(9,0b011011010,1344),
    (9,0b011011011,1408),
    (11,0b00000001000,1792),(11,0b00000001100,1856),(11,0b00000001101,1920),
    (12,0b000000010010,1984),(12,0b000000010011,2048),(12,0b000000010100,2112),(12,0b000000010101,2176),
    (12,0b000000010110,2240),(12,0b000000010111,2304),(12,0b000000011100,2368),(12,0b000000011101,2432),
    (12,0b000000011110,2496),(12,0b000000011111,2560),
];

#[rustfmt::skip]
static BLACK_CODES: &[(u8, u16, u16)] = &[
    (2,0b10,3),(2,0b11,2),(3,0b010,1),(3,0b011,4),(4,0b0010,6),(4,0b0011,5),(5,0b00011,7),
    (6,0b000100,9),(6,0b000101,8),(7,0b0000100,10),(7,0b0000101,11),(7,0b0000111,12),
    (8,0b00000100,13),(8,0b00000111,14),(9,0b000011000,15),
    (10,0b0000001000,18),(10,0b0000001111,64),(10,0b0000010111,16),(10,0b0000011000,17),(10,0b0000110111,0),
    (11,0b00000001000,1792),(11,0b00000001100,1856),(11,0b00000001101,1920),(11,0b00000010111,24),
    (11,0b00000011000,25),(11,0b00000101000,23),(11,0b00000110111,22),(11,0b00001100111,19),
    (11,0b00001101000,20),(11,0b00001101100,21),
    (12,0b000000010010,1984),(12,0b000000010011,2048),(12,0b000000010100,2112),(12,0b000000010101,2176),
    (12,0b000000010110,2240),(12,0b000000010111,2304),(12,0b000000011100,2368),(12,0b000000011101,2432),
    (12,0b000000011110,2496),(12,0b000000011111,2560),(12,0b000000100100,52),(12,0b000000100111,55),
    (12,0b000000101000,56),(12,0b000000101011,59),(12,0b000000101100,60),(12,0b000000110011,320),
    (12,0b000000110100,384),(12,0b000000110101,448),(12,0b000000110111,53),(12,0b000000111000,54),
    (12,0b000001010010,50),(12,0b000001010011,51),(12,0b000001010100,44),(12,0b000001010101,45),
    (12,0b000001010110,46),(12,0b000001010111,47),(12,0b000001011000,57),(12,0b000001011001,58),
    (12,0b000001011010,61),(12,0b000001011011,256),(12,0b000001100100,48),(12,0b000001100101,49),
    (12,0b000001100110,62),(12,0b000001100111,63),(12,0b000001101000,30),(12,0b000001101001,31),
    (12,0b000001101010,32),(12,0b000001101011,33),(12,0b000001101100,40),(12,0b000001101101,41),
    (12,0b000011001000,128),(12,0b000011001001,192),(12,0b000011001010,26),(12,0b000011001011,27),
    (12,0b000011001100,28),(12,0b000011001101,29),(12,0b000011010010,34),(12,0b000011010011,35),
    (12,0b000011010100,36),(12,0b000011010101,37),(12,0b000011010110,38),(12,0b000011010111,39),
    (12,0b000011011010,42),(12,0b000011011011,43),
    (13,0b0000001001010,640),(13,0b0000001001011,704),(13,0b0000001001100,768),(13,0b0000001001101,832),
    (13,0b0000001010010,1280),(13,0b0000001010011,1344),(13,0b0000001010100,1408),(13,0b0000001010101,1472),
    (13,0b0000001011010,1536),(13,0b0000001011011,1600),(13,0b0000001100100,1664),(13,0b0000001100101,1728),
    (13,0b0000001101100,512),(13,0b0000001101101,576),(13,0b0000001110010,896),(13,0b0000001110011,960),
    (13,0b0000001110100,1024),(13,0b0000001110101,1088),(13,0b0000001110110,1152),(13,0b0000001110111,1216),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccitt_decode_passthrough() {
        let decoder = CcittFaxDecoder;
        let ccitt_data = b"\x00\x01\x02\x03";
        assert_eq!(decoder.decode(ccitt_data).unwrap(), ccitt_data);
    }

    #[test]
    fn test_ccitt_decoder_name() {
        assert_eq!(CcittFaxDecoder.name(), "CCITTFaxDecode");
    }

    #[test]
    fn tables_are_prefix_free() {
        for table in [WHITE_CODES, BLACK_CODES] {
            for &(la, ca, _) in table {
                for &(lb, cb, _) in table {
                    if lb > la && (cb >> (lb - la)) == ca {
                        panic!("non-prefix-free pair: ({la},{ca:b}) vs ({lb},{cb:b})");
                    }
                }
            }
        }
    }

    fn p(params: CcittParams, bits: &str) -> Result<CcittDecoded> {
        // Pack an MSB-first bit string into bytes (zero-padded to a byte).
        let mut bytes = Vec::new();
        let mut acc = 0u8;
        let mut n = 0u8;
        for ch in bits.chars().filter(|c| *c == '0' || *c == '1') {
            acc = (acc << 1) | (ch == '1') as u8;
            n += 1;
            if n == 8 {
                bytes.push(acc);
                acc = 0;
                n = 0;
            }
        }
        if n > 0 {
            bytes.push(acc << (8 - n));
        }
        decode(&bytes, &params)
    }

    #[test]
    fn g4_all_white_row_v0() {
        // 8-wide, 1 row. V0 against the imaginary white ref → all-white row.
        let params = CcittParams {
            k: -1,
            columns: 8,
            rows: Some(1),
            ..Default::default()
        };
        let d = p(params, "1").unwrap();
        assert_eq!(d.rows_decoded, 1);
        assert_eq!(d.data, vec![0u8]); // all white
        assert!(!d.recovered_partial);
    }

    #[test]
    fn g4_horizontal_white3_black2() {
        // 8-wide row: Horizontal (001) + white run 3 (1000) + black run 2 (11)
        // ⇒ transitions [3,5], leaving a0=5; a trailing V0 (1) extends the final
        // white run to the right edge, completing the row.
        let params = CcittParams {
            k: -1,
            columns: 8,
            rows: Some(1),
            ..Default::default()
        };
        let d = p(params, "001 1000 11 1").unwrap();
        // pixels: white[0..3) black[3..5) white[5..8) ⇒ 0b00011000 = 0x18
        assert_eq!(d.data, vec![0b0001_1000]);
    }

    #[test]
    fn g4_negative_k_with_unknown_rows_grows_output_fallibly() {
        let params = CcittParams {
            k: -2,
            columns: 8,
            rows: None,
            ..Default::default()
        };
        let d = p(params, "001 1000 11 1 000000000001").unwrap();
        assert_eq!(d.rows_decoded, 1);
        assert_eq!(d.data, vec![0b0001_1000]);
        assert!(!d.recovered_partial);
    }

    #[test]
    fn g4_encoded_byte_align() {
        // Two all-white rows, each coded as a single V0 (`1`). Row 0's 1-bit
        // code is byte-padded, so the stream is [0x80, 0x80]. WITH
        // /EncodedByteAlign both rows decode; WITHOUT it the fill zeros after
        // row 0 mis-read as an invalid mode and the 2nd row is unrecoverable —
        // exactly the real-world fax-scanner failure of issue #738.
        let aligned = CcittParams {
            k: -1,
            columns: 8,
            rows: Some(2),
            encoded_byte_align: true,
            ..Default::default()
        };
        let d = decode(&[0x80, 0x80], &aligned).unwrap();
        assert_eq!(d.rows_decoded, 2);
        assert_eq!(d.data, vec![0u8, 0u8]); // both rows white
        assert!(!d.recovered_partial);

        let unaligned = CcittParams {
            encoded_byte_align: false,
            ..aligned
        };
        let d2 = decode(&[0x80, 0x80], &unaligned).unwrap();
        // Without alignment the 2nd row can't be found → 1 row decoded, the rest
        // recovered (white-padded) — NOT a silently-blank full page.
        assert_eq!(d2.rows_decoded, 1);
        assert!(d2.recovered_partial);
    }

    #[test]
    fn g4_zero_rows_errs_not_white() {
        // Garbage that cannot start a row → Err, NOT an all-white Ok buffer.
        let params = CcittParams {
            k: -1,
            columns: 8,
            rows: Some(4),
            ..Default::default()
        };
        // "000000000000000" is the EOL/EOFB region but with no valid first mode
        // before data ends → no rows.
        let d = p(params, "000000000001"); // single EOL = immediate EOFB
                                           // EOFB on the very first read → EndOfBlock with 0 rows → padded white.
                                           // That is a legitimately-blank scan, so it returns Ok(all white). A
                                           // truly undecodable stream errors instead:
        assert!(d.is_ok());
        let bad = CcittParams {
            k: -1,
            columns: 8,
            rows: Some(4),
            ..Default::default()
        };
        // 0b0000001 = Extension as the very first mode → Error, 0 rows → Err.
        assert!(p(bad, "0000001").is_err());
    }

    // -- Group 3 (T.4) --------------------------------------------------------

    #[test]
    fn g3_1d_all_white_row() {
        // 8-wide K=0 row: a single white run of 8 (MH code 10011).
        let params = CcittParams {
            k: 0,
            columns: 8,
            rows: Some(1),
            ..Default::default()
        };
        let d = p(params, "10011").unwrap();
        assert_eq!(d.rows_decoded, 1);
        assert_eq!(d.data, vec![0u8]); // all white
        assert!(!d.recovered_partial);
    }

    #[test]
    fn g3_1d_white3_black2() {
        // 8-wide K=0 row: white run 3 (1000) + black run 2 (11) + white run 3
        // (1000) ⇒ transitions [3,5] ⇒ 0b0001_1000, identical to the G4 case.
        let params = CcittParams {
            k: 0,
            columns: 8,
            rows: Some(1),
            ..Default::default()
        };
        let d = p(params, "1000 11 1000").unwrap();
        assert_eq!(d.data, vec![0b0001_1000]);
    }

    #[test]
    fn g3_1d_eol_delimited_rows() {
        // Two all-white rows, each preceded by an EOL (000000000001) as a real
        // T.4 stream emits. The EOLs must be swallowed, not mis-read as runs.
        let params = CcittParams {
            k: 0,
            columns: 8,
            rows: Some(2),
            end_of_line: true,
            ..Default::default()
        };
        let d = p(params, "000000000001 10011 000000000001 10011").unwrap();
        assert_eq!(d.rows_decoded, 2);
        assert_eq!(d.data, vec![0u8, 0u8]);
        assert!(!d.recovered_partial);
    }

    #[test]
    fn g3_1d_rtc_terminates() {
        // One white row then a return-to-control (6 EOLs) ⇒ stop cleanly; the
        // declared 3rd/4th rows are white-padded, not reported as decoded.
        let params = CcittParams {
            k: 0,
            columns: 8,
            rows: Some(4),
            ..Default::default()
        };
        let rtc = "000000000001".repeat(6);
        let d = p(params, &format!("10011 {rtc}")).unwrap();
        assert_eq!(d.rows_decoded, 1);
        assert_eq!(d.data, vec![0u8; 4]); // padded to declared height
    }

    // Real libtiff output for one 128×64 bilevel image, encoded both ways. The
    // G4 path is already trusted, so decoding the G3 (K=0) stream to the *same*
    // bitmap proves the new Modified-Huffman path — this is the codestream shape
    // behind the blank-page reports (K=0 / Group 3 1-D), which the in-house
    // decoder previously rejected outright.
    #[rustfmt::skip]
    const G3_1D_STREAM: &[u8] = &[
        0,17,192,240,103,0,24,3,193,104,0,77,120,3,192,172,0,77,92,1,224,176,0,38,165,0,120,19,128,9,168,176,7,
        129,184,0,154,132,128,60,20,192,4,212,60,1,224,202,0,38,162,137,1,224,160,0,77,69,18,3,193,64,0,154,138,
        36,7,130,128,1,53,20,72,15,5,0,2,106,40,144,30,10,0,4,212,81,32,60,20,0,9,168,162,64,120,40,0,19,81,68,
        128,240,80,0,38,162,137,1,224,160,0,77,69,18,3,193,64,0,154,138,36,7,130,128,1,53,20,72,15,5,0,2,106,40,
        144,30,10,0,4,212,81,64,60,54,0,9,168,162,74,5,138,0,252,0,77,69,18,160,199,221,15,14,7,160,1,53,20,72,
        227,29,142,99,129,232,0,77,69,18,29,138,224,126,0,38,162,137,9,10,12,112,61,0,9,168,162,65,45,14,135,135,
        3,208,0,154,138,36,20,117,8,120,112,61,0,9,168,162,65,97,112,31,128,9,168,162,64,196,1,240,0,154,138,36,
        25,224,15,64,2,106,40,144,108,176,102,0,19,81,68,128,188,1,96,0,154,138,36,25,80,11,32,2,106,40,144,102,
        64,20,0,9,168,162,64,209,0,112,0,38,162,137,3,84,1,32,0,154,138,36,26,80,10,64,2,106,40,144,106,64,50,0,
        9,168,162,65,173,0,172,0,38,160,120,49,0,168,0,38,160,120,103,128,218,0,19,80,60,54,64,54,0,9,168,30,10,
        32,53,128,4,212,15,3,16,26,128,2,106,7,134,92,6,144,0,154,129,225,155,0,212,0,38,160,120,52,192,52,0,9,
        168,30,13,112,25,128,2,106,7,134,156,6,80,0,154,129,225,171,0,92,0,77,64,240,215,128,110,0,38,160,120,54,
        192,104,0,19,80,60,54,224,8,0,19,80,60,21,96,23,0,19,80,60,21,224,28,0,77,64,240,101,128,224,2,106,7,130,
        156,4,0,19,80,60,13,224,80,1,53,3,192,158,8,0,77,64,240,88,134,0,38,160,120,21,198,0,38,160,120,45,64,
    ];
    #[rustfmt::skip]
    const G4_STREAM: &[u8] = &[
        35,129,224,206,28,17,227,12,60,48,240,195,195,15,12,60,138,37,255,255,255,255,255,255,195,225,21,4,88,52,
        120,97,160,71,116,8,195,248,97,164,227,248,99,16,151,134,202,128,200,224,122,236,66,40,114,135,253,132,16,
        255,226,16,66,15,196,53,225,135,225,131,240,97,248,97,248,97,248,97,248,97,248,97,248,97,248,97,226,24,120,
        97,225,135,134,30,24,120,97,225,135,134,30,24,120,97,225,135,134,30,24,120,97,225,135,134,30,24,120,97,225,
        135,134,30,24,120,97,225,134,0,32,2,
    ];

    #[test]
    fn g3_1d_matches_g4_for_same_image() {
        let g3 = decode(
            G3_1D_STREAM,
            &CcittParams {
                k: 0,
                columns: 128,
                rows: Some(64),
                end_of_line: true,
                ..Default::default()
            },
        )
        .unwrap();
        let g4 = decode(
            G4_STREAM,
            &CcittParams {
                k: -1,
                columns: 128,
                rows: Some(64),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(g3.rows_decoded, 64);
        assert!(!g3.recovered_partial);
        assert_eq!(g3.data.len(), 16 * 64);
        // Both encode the identical bitmap; the trusted G4 path is the oracle.
        assert_eq!(g3.data, g4.data);
        // And it is not a degenerate all-white decode.
        assert!(g3.data.iter().any(|&b| b != 0));
    }

    #[test]
    fn packed_output_buffer_rejects_size_overflow() {
        let result = packed_output_buffer(usize::MAX, Some(2));
        assert!(matches!(result, Err(Error::Decode(message)) if message.contains("size overflow")));
    }
}
