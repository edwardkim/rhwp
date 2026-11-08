//! Inject a Unicode→GID cmap subtable into an existing OTF/TTF font.
//!
//! PDFs produced by Word/LibreOffice/etc. typically embed CFF font
//! subsets that ship without a Unicode `cmap` table — text rendering
//! relies on per-byte CID encoding plus the document's own
//! `/ToUnicode` CMap. When we re-embed such a subset into a DOCX or
//! PPTX (via `PdfDocument::to_*_bytes`), the office consumer expects
//! the font to carry its own Unicode cmap. Without one,
//! `glyph_lookup` is empty and the font is unusable as far as the
//! renderer is concerned (see `EmbeddedFont::has_usable_unicode_cmap`).
//!
//! This module patches the OTF: it inserts (or replaces) the `cmap`
//! table with a format-4 subtable encoding the supplied Unicode→GID
//! mapping. The patched font then registers cleanly and renders with
//! the source typeface program instead of falling back to a base-14
//! family. Output bytes remain a valid SFNT — table directory
//! offsets, table checksums, and `head.checkSumAdjustment` are
//! recomputed.
//!
//! Limitations:
//! - Only emits a single format-4 subtable (BMP only — codepoints
//!   above U+FFFF are dropped silently). Adequate for Latin /
//!   Cyrillic / Greek source PDFs which is the vast majority of
//!   real-world office round-trips.
//! - The platform/encoding record is the conventional `(0, 3)`
//!   Unicode BMP entry (per OpenType spec), which all modern
//!   parsers honour.

use std::collections::HashMap;

const SFNT_TRUETYPE: u32 = 0x00010000;
const SFNT_OTTO: u32 = 0x4F54_544F; // 'OTTO'
const TAG_CMAP: u32 = 0x636D_6170; // 'cmap'
const TAG_HEAD: u32 = 0x6865_6164; // 'head'
const TAG_HHEA: u32 = 0x6868_6561; // 'hhea'
const TAG_HMTX: u32 = 0x686D_7478; // 'hmtx'
const TAG_MAXP: u32 = 0x6D61_7870; // 'maxp'

/// Patch the OTF/TTF byte stream to carry a format-4 `cmap` subtable
/// encoding the supplied Unicode→GID mapping. Returns `None` if the
/// input is not a recognisable SFNT or the patch fails. On success,
/// returns a fresh byte stream identical in every other respect (all
/// other tables byte-preserved with new offsets).
pub fn inject_unicode_cmap(
    font_bytes: &[u8],
    unicode_to_gid: &HashMap<u32, u16>,
) -> Option<Vec<u8>> {
    if font_bytes.len() < 12 || unicode_to_gid.is_empty() {
        return None;
    }
    let sfnt_version = read_u32(font_bytes, 0)?;
    if sfnt_version != SFNT_TRUETYPE && sfnt_version != SFNT_OTTO {
        return None;
    }

    let num_tables = read_u16(font_bytes, 4)? as usize;
    if font_bytes.len() < 12 + num_tables * 16 {
        return None;
    }

    // Read existing table directory entries.
    #[derive(Clone)]
    struct TableEntry {
        tag: u32,
        checksum: u32,
        offset: u32,
        length: u32,
        data: Vec<u8>,
    }

    let mut tables: Vec<TableEntry> = Vec::with_capacity(num_tables + 1);
    for i in 0..num_tables {
        let rec_off = 12 + i * 16;
        let tag = read_u32(font_bytes, rec_off)?;
        let checksum = read_u32(font_bytes, rec_off + 4)?;
        let offset = read_u32(font_bytes, rec_off + 8)?;
        let length = read_u32(font_bytes, rec_off + 12)?;
        let end = (offset as usize).checked_add(length as usize)?;
        if end > font_bytes.len() {
            return None;
        }
        let data = font_bytes[offset as usize..end].to_vec();
        tables.push(TableEntry {
            tag,
            checksum,
            offset,
            length,
            data,
        });
    }

    // Build the new cmap table bytes.
    let new_cmap_bytes = build_format4_cmap(unicode_to_gid)?;

    // Replace existing cmap, or insert a new entry preserving sort order.
    let new_cmap_checksum = checksum_table(&new_cmap_bytes);
    let cmap_idx = tables.iter().position(|t| t.tag == TAG_CMAP);
    if let Some(idx) = cmap_idx {
        tables[idx].length = new_cmap_bytes.len() as u32;
        tables[idx].checksum = new_cmap_checksum;
        tables[idx].data = new_cmap_bytes;
    } else {
        tables.push(TableEntry {
            tag: TAG_CMAP,
            checksum: new_cmap_checksum,
            offset: 0,
            length: new_cmap_bytes.len() as u32,
            data: new_cmap_bytes,
        });
    }

    // Tables in the directory are sorted by tag (per OpenType spec).
    // Sort by tag so the writer emits them in canonical order.
    tables.sort_by_key(|t| t.tag);

    // Compute new offsets. Each table is padded to 4-byte alignment.
    let new_num_tables = tables.len();
    let header_size = 12 + new_num_tables * 16;
    let mut cur_offset = header_size as u32;
    for t in tables.iter_mut() {
        t.offset = cur_offset;
        let pad = (4 - (t.length as usize & 3)) & 3;
        cur_offset += t.length + pad as u32;
    }

    // Recompute search range / entry selector / range shift per spec.
    let entry_selector = (new_num_tables as f64).log2().floor() as u16;
    let search_range = (1u16 << entry_selector) * 16;
    let range_shift = (new_num_tables as u16) * 16 - search_range;

    let total_size = cur_offset as usize;
    let mut out = vec![0u8; total_size];
    write_u32(&mut out, 0, sfnt_version);
    write_u16(&mut out, 4, new_num_tables as u16);
    write_u16(&mut out, 6, search_range);
    write_u16(&mut out, 8, entry_selector);
    write_u16(&mut out, 10, range_shift);
    for (i, t) in tables.iter().enumerate() {
        let rec_off = 12 + i * 16;
        write_u32(&mut out, rec_off, t.tag);
        write_u32(&mut out, rec_off + 4, t.checksum);
        write_u32(&mut out, rec_off + 8, t.offset);
        write_u32(&mut out, rec_off + 12, t.length);
        let off = t.offset as usize;
        out[off..off + t.data.len()].copy_from_slice(&t.data);
    }

    // Recompute head.checkSumAdjustment per OpenType spec:
    //   1. Set checkSumAdjustment field (head.offset+8 .. +12) to 0
    //   2. Sum entire file as u32 stream → sum_total
    //   3. checkSumAdjustment = 0xB1B0AFBA - sum_total
    if let Some(head) = tables.iter().find(|t| t.tag == TAG_HEAD) {
        let head_off = head.offset as usize;
        if head.length >= 12 && head_off + 12 <= out.len() {
            // Zero checkSumAdjustment first.
            write_u32(&mut out, head_off + 8, 0);
            let sum_total = sum_u32_padded(&out);
            let adjustment = 0xB1B0_AFBA_u32.wrapping_sub(sum_total);
            write_u32(&mut out, head_off + 8, adjustment);
        }
    }

    Some(out)
}

/// Patch the OTF/TTF byte stream to carry an `hmtx` table whose
/// `advanceWidth` records reflect the supplied `widths_by_gid`
/// mapping (in 1/1000 em units, i.e. PDF `/W` units; the function
/// converts to font design units using `head.unitsPerEm`). Returns
/// `None` if the input is not a recognisable SFNT or the patch
/// fails. On success, returns a fresh byte stream with `hmtx`
/// inserted/replaced and `hhea.numberOfHMetrics` rewritten to
/// `maxp.numGlyphs` so every glyph carries its own advance.
///
/// PDFs ship raw CFF font subsets (FontFile3) without an `hmtx`
/// table — widths live in the document's `/W` array. When pdf_oxide
/// wraps the raw CFF in a synthetic OpenType container for
/// downstream consumers (`font_dict::wrap_cff_in_opentype`) it
/// emits no `hmtx`. ttf-parser then returns 0 from
/// `glyph_hor_advance`, the office writer records zeros into its
/// width table, and the round-trip PDF emits a `/W` array of zeros
/// → glyphs render with no advance. This function rebuilds the
/// `hmtx` from the source PDF's `/W` so the round-trip writer's
/// own width-extraction sees correct values.
pub fn inject_hmtx(font_bytes: &[u8], widths_by_gid: &HashMap<u16, u16>) -> Option<Vec<u8>> {
    if font_bytes.len() < 12 {
        return None;
    }
    let sfnt_version = read_u32(font_bytes, 0)?;
    if sfnt_version != SFNT_TRUETYPE && sfnt_version != SFNT_OTTO {
        return None;
    }

    let num_tables = read_u16(font_bytes, 4)? as usize;
    if font_bytes.len() < 12 + num_tables * 16 {
        return None;
    }

    #[derive(Clone)]
    struct TableEntry {
        tag: u32,
        checksum: u32,
        offset: u32,
        length: u32,
        data: Vec<u8>,
    }

    let mut tables: Vec<TableEntry> = Vec::with_capacity(num_tables + 1);
    for i in 0..num_tables {
        let rec_off = 12 + i * 16;
        let tag = read_u32(font_bytes, rec_off)?;
        let checksum = read_u32(font_bytes, rec_off + 4)?;
        let offset = read_u32(font_bytes, rec_off + 8)?;
        let length = read_u32(font_bytes, rec_off + 12)?;
        let end = (offset as usize).checked_add(length as usize)?;
        if end > font_bytes.len() {
            return None;
        }
        let data = font_bytes[offset as usize..end].to_vec();
        tables.push(TableEntry {
            tag,
            checksum,
            offset,
            length,
            data,
        });
    }

    // Required tables for an hmtx rebuild.
    let maxp = tables.iter().find(|t| t.tag == TAG_MAXP)?.data.clone();
    if maxp.len() < 6 {
        return None;
    }
    let num_glyphs = read_u16(&maxp, 4)? as usize;
    if num_glyphs == 0 {
        return None;
    }

    let head_data = tables.iter().find(|t| t.tag == TAG_HEAD)?.data.clone();
    if head_data.len() < 20 {
        return None;
    }
    let units_per_em = read_u16(&head_data, 18)? as u32;
    if units_per_em == 0 {
        return None;
    }

    // Build new hmtx: numGlyphs * (advanceWidth u16 + lsb i16) = 4 bytes each.
    // widths_by_gid is in 1/1000 em (PDF /W units); convert to font design units.
    let mut new_hmtx = Vec::with_capacity(num_glyphs * 4);
    for gid in 0u16..num_glyphs as u16 {
        let advance_1000 = widths_by_gid.get(&gid).copied().unwrap_or(500);
        let advance_design = ((advance_1000 as u32 * units_per_em) / 1000).min(0xFFFF) as u16;
        new_hmtx.extend_from_slice(&advance_design.to_be_bytes());
        new_hmtx.extend_from_slice(&[0u8, 0u8]); // lsb = 0
    }
    let new_hmtx_checksum = checksum_table(&new_hmtx);

    let hmtx_idx = tables.iter().position(|t| t.tag == TAG_HMTX);
    if let Some(idx) = hmtx_idx {
        tables[idx].length = new_hmtx.len() as u32;
        tables[idx].checksum = new_hmtx_checksum;
        tables[idx].data = new_hmtx;
    } else {
        tables.push(TableEntry {
            tag: TAG_HMTX,
            checksum: new_hmtx_checksum,
            offset: 0,
            length: new_hmtx.len() as u32,
            data: new_hmtx,
        });
    }

    // Patch hhea.numberOfHMetrics (u16 at offset 34) to numGlyphs.
    let hhea_idx = tables.iter().position(|t| t.tag == TAG_HHEA)?;
    if tables[hhea_idx].data.len() < 36 {
        return None;
    }
    let mut new_hhea = tables[hhea_idx].data.clone();
    write_u16(&mut new_hhea, 34, num_glyphs as u16);
    tables[hhea_idx].checksum = checksum_table(&new_hhea);
    tables[hhea_idx].length = new_hhea.len() as u32;
    tables[hhea_idx].data = new_hhea;

    tables.sort_by_key(|t| t.tag);

    let new_num_tables = tables.len();
    let header_size = 12 + new_num_tables * 16;
    let mut cur_offset = header_size as u32;
    for t in tables.iter_mut() {
        t.offset = cur_offset;
        let pad = (4 - (t.length as usize & 3)) & 3;
        cur_offset += t.length + pad as u32;
    }

    let entry_selector = (new_num_tables as f64).log2().floor() as u16;
    let search_range = (1u16 << entry_selector) * 16;
    let range_shift = (new_num_tables as u16) * 16 - search_range;

    let total_size = cur_offset as usize;
    let mut out = vec![0u8; total_size];
    write_u32(&mut out, 0, sfnt_version);
    write_u16(&mut out, 4, new_num_tables as u16);
    write_u16(&mut out, 6, search_range);
    write_u16(&mut out, 8, entry_selector);
    write_u16(&mut out, 10, range_shift);
    for (i, t) in tables.iter().enumerate() {
        let rec_off = 12 + i * 16;
        write_u32(&mut out, rec_off, t.tag);
        write_u32(&mut out, rec_off + 4, t.checksum);
        write_u32(&mut out, rec_off + 8, t.offset);
        write_u32(&mut out, rec_off + 12, t.length);
        let off = t.offset as usize;
        out[off..off + t.data.len()].copy_from_slice(&t.data);
    }

    if let Some(head) = tables.iter().find(|t| t.tag == TAG_HEAD) {
        let head_off = head.offset as usize;
        if head.length >= 12 && head_off + 12 <= out.len() {
            write_u32(&mut out, head_off + 8, 0);
            let sum_total = sum_u32_padded(&out);
            let adjustment = 0xB1B0_AFBA_u32.wrapping_sub(sum_total);
            write_u32(&mut out, head_off + 8, adjustment);
        }
    }

    Some(out)
}

/// Build the bytes of an OpenType `cmap` table containing exactly one
/// format-4 subtable (Unicode BMP). The `unicode_to_gid` keys are
/// filtered to the BMP (≤ 0xFFFF).
fn build_format4_cmap(unicode_to_gid: &HashMap<u32, u16>) -> Option<Vec<u8>> {
    // Collect (cp, gid) pairs in ascending cp order. Filter out
    // codepoints outside BMP and duplicates.
    let mut pairs: Vec<(u16, u16)> = unicode_to_gid
        .iter()
        .filter_map(|(&cp, &gid)| {
            if cp <= 0xFFFF {
                Some((cp as u16, gid))
            } else {
                None
            }
        })
        .collect();
    if pairs.is_empty() {
        return None;
    }
    pairs.sort_by_key(|(cp, _)| *cp);
    pairs.dedup_by_key(|(cp, _)| *cp);

    // Coalesce consecutive (cp, gid) pairs into segments where
    // gid - cp is constant (the format-4 idDelta encoding).
    struct Segment {
        start: u16,
        end: u16,
        delta: i32, // i32 to handle wraparound; will encode as i16
    }
    let mut segs: Vec<Segment> = Vec::new();
    for &(cp, gid) in pairs.iter() {
        let want_delta = gid as i32 - cp as i32;
        let extend = segs
            .last_mut()
            .map(|s| s.end as u32 + 1 == cp as u32 && s.delta == want_delta)
            .unwrap_or(false);
        if extend {
            let last = segs.last_mut().unwrap();
            last.end = cp;
        } else {
            segs.push(Segment {
                start: cp,
                end: cp,
                delta: want_delta,
            });
        }
    }

    // Final sentinel segment per format-4 spec: 0xFFFF→0xFFFF mapping
    // to glyph 0 (delta = 1 makes glyph index = 0xFFFF + 1 = 0).
    if segs.last().is_none_or(|s| s.end != 0xFFFF) {
        segs.push(Segment {
            start: 0xFFFF,
            end: 0xFFFF,
            delta: 1,
        });
    }

    let seg_count = segs.len();
    let seg_count_x2 = (seg_count as u16) * 2;
    let entry_selector = (seg_count as f64).log2().floor() as u16;
    let search_range = (1u16 << entry_selector) * 2;
    let range_shift = seg_count_x2 - search_range;

    // Subtable size: 14-byte format-4 header (format, length, language,
    // segCountX2, searchRange, entrySelector, rangeShift) +
    // endCode[segCount] (2*segCount) + reservedPad (2) +
    // startCode[segCount] + idDelta[segCount] + idRangeOffset[segCount]
    // (each 2*segCount). idRangeOffset is all zero so glyphIdArray is
    // empty. Total = 16 + 8 * segCount. (The earlier "+ 2" was a
    // double-count of reservedPad — it inflated the format-4 length
    // field by 2 bytes, leaving the last idRangeOffset slot reported
    // as off-the-end. ttf-parser/CoreText silently rejected the cmap;
    // some Win/macOS renderers then mapped the affected codepoints to
    // the wrong glyph, producing the broken-lowercase glyphs in
    // PDF→DOCX→PDF round-trips of MicrosoftSansSerif-subset PDFs.)
    let subtable_size = 16 + 8 * seg_count;
    let total_size = 4 /* cmap header */ + 8 /* one encoding record */ + subtable_size;
    let mut buf = Vec::with_capacity(total_size);

    // cmap header
    write_be_u16(&mut buf, 0); // version
    write_be_u16(&mut buf, 1); // numTables

    // encoding record (Unicode BMP, platformID=0, encodingID=3)
    write_be_u16(&mut buf, 0); // platformID
    write_be_u16(&mut buf, 3); // encodingID
    write_be_u32(&mut buf, 12); // offset to subtable

    // format-4 subtable
    write_be_u16(&mut buf, 4); // format
    write_be_u16(&mut buf, subtable_size as u16); // length
    write_be_u16(&mut buf, 0); // language (0 = "any")
    write_be_u16(&mut buf, seg_count_x2); // segCountX2
    write_be_u16(&mut buf, search_range);
    write_be_u16(&mut buf, entry_selector);
    write_be_u16(&mut buf, range_shift);

    // endCode array
    for s in &segs {
        write_be_u16(&mut buf, s.end);
    }
    write_be_u16(&mut buf, 0); // reservedPad

    // startCode array
    for s in &segs {
        write_be_u16(&mut buf, s.start);
    }

    // idDelta array (i16, encoded as u16 two's complement)
    for s in &segs {
        let d16 = s.delta.rem_euclid(0x1_0000) as u16;
        write_be_u16(&mut buf, d16);
    }

    // idRangeOffset array (all zero → use idDelta directly)
    for _ in &segs {
        write_be_u16(&mut buf, 0);
    }

    Some(buf)
}

// ── Endian-safe byte-stream helpers ─────────────────────────────────

fn read_u16(buf: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*buf.get(off)?, *buf.get(off + 1)?]))
}

fn read_u32(buf: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *buf.get(off)?,
        *buf.get(off + 1)?,
        *buf.get(off + 2)?,
        *buf.get(off + 3)?,
    ]))
}

fn write_u16(buf: &mut [u8], off: usize, v: u16) {
    buf[off..off + 2].copy_from_slice(&v.to_be_bytes());
}

fn write_u32(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_be_bytes());
}

fn write_be_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn write_be_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

/// OpenType table checksum: sum of u32-aligned big-endian words. The
/// table is implicitly zero-padded to a multiple of 4 bytes for the
/// checksum (per spec §2.2, "Calculating Checksums").
fn checksum_table(data: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut i = 0;
    while i < data.len() {
        let mut word = [0u8; 4];
        let take = (data.len() - i).min(4);
        word[..take].copy_from_slice(&data[i..i + take]);
        sum = sum.wrapping_add(u32::from_be_bytes(word));
        i += 4;
    }
    sum
}

/// Same as `checksum_table` but expects `data` to already be padded.
fn sum_u32_padded(data: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 4 <= data.len() {
        sum =
            sum.wrapping_add(u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]));
        i += 4;
    }
    if i < data.len() {
        let mut word = [0u8; 4];
        let take = data.len() - i;
        word[..take].copy_from_slice(&data[i..i + take]);
        sum = sum.wrapping_add(u32::from_be_bytes(word));
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_otf() -> Vec<u8> {
        // 12-byte SFNT header + zero tables. Not a valid font on its
        // own (no required tables) but a usable test scaffold for the
        // table-directory rewriting code paths.
        let mut buf = Vec::new();
        write_be_u32(&mut buf, SFNT_OTTO);
        write_be_u16(&mut buf, 0); // numTables
        write_be_u16(&mut buf, 0); // searchRange
        write_be_u16(&mut buf, 0); // entrySelector
        write_be_u16(&mut buf, 0); // rangeShift
        buf
    }

    #[test]
    fn empty_map_returns_none() {
        let font = minimal_otf();
        let map = HashMap::new();
        assert!(inject_unicode_cmap(&font, &map).is_none());
    }

    #[test]
    fn invalid_sfnt_returns_none() {
        let font = vec![0u8; 32];
        let mut map = HashMap::new();
        map.insert(0x41u32, 65u16);
        assert!(inject_unicode_cmap(&font, &map).is_none());
    }

    #[test]
    fn injects_into_empty_otf() {
        let font = minimal_otf();
        let mut map = HashMap::new();
        map.insert(b'A' as u32, 65u16);
        map.insert(b'B' as u32, 66u16);
        let patched = inject_unicode_cmap(&font, &map).expect("patch ok");
        // Should have one table now (the cmap).
        assert_eq!(read_u16(&patched, 4).unwrap(), 1);
        // First table tag should be 'cmap'.
        assert_eq!(read_u32(&patched, 12).unwrap(), TAG_CMAP);
    }

    #[test]
    fn segment_coalescing_consecutive_cps_consecutive_gids() {
        let mut map = HashMap::new();
        for i in 0..10u32 {
            map.insert(b'A' as u32 + i, 100 + i as u16);
        }
        let cmap = build_format4_cmap(&map).expect("build cmap");
        // segCountX2 lives at offset 6 of the format-4 header (cmap
        // header is 4 + 8 = 12 bytes before subtable; segCountX2 is
        // at subtable+6 → byte 18).
        let seg_count_x2 = read_u16(&cmap, 18).unwrap();
        // Should have 1 real segment + 1 sentinel = 2 segments → 4.
        assert_eq!(seg_count_x2, 4);
    }
}
