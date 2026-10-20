use crate::import_limits::ImportBudget;
use byteorder::{BigEndian, ReadBytesExt};
/// TrueType cmap table extraction for font character mapping
///
/// This module extracts Unicode mappings from TrueType font cmap tables,
/// providing a fallback for Type0 fonts missing ToUnicode CMaps.
///
/// The cmap table maps glyph IDs (GIDs) to Unicode code points.
/// We support formats 4 (BMP), 6 (trimmed), and 12 (Unicode full).
use std::collections::HashMap;
use std::io::Cursor;

/// Mac Roman (platform 1, encoding 0) high-half → Unicode mapping.
///
/// Bytes 0x00..0x7F are identical to ASCII and are handled by the caller.
/// This table covers 0x80..0xFF per Apple's Mac Roman → Unicode reference.
#[rustfmt::skip]
const MAC_ROMAN_HIGH: [char; 128] = [
    'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á',
    'à', 'â', 'ä', 'ã', 'å', 'ç', 'é', 'è',
    'ê', 'ë', 'í', 'ì', 'î', 'ï', 'ñ', 'ó',
    'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü',
    '†', '°', '¢', '£', '§', '•', '¶', 'ß',
    '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø',
    '∞', '±', '≤', '≥', '¥', 'µ', '∂', '∑',
    '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø',
    '¿', '¡', '¬', '√', 'ƒ', '≈', '∆', '«',
    '»', '…', '\u{00A0}', 'À', 'Ã', 'Õ', 'Œ', 'œ',
    '–', '—', '"', '"', '\'', '\'', '÷', '◊',
    'ÿ', 'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ',
    '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á',
    'Ë', 'È', 'Í', 'Î', 'Ï', 'Ì', 'Ó', 'Ô',
    '\u{F8FF}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜',
    '¯', '˘', '˙', '˚', '¸', '˝', '˛', 'ˇ',
];

#[inline]
fn mac_roman_to_unicode(byte: u8) -> char {
    debug_assert!(byte >= 0x80);
    MAC_ROMAN_HIGH[(byte - 0x80) as usize]
}

/// Represents a TrueType cmap table extracted from an embedded font
#[derive(Debug, Clone)]
pub struct TrueTypeCMap {
    /// Mapping from Glyph ID to Unicode character
    gid_to_unicode: std::sync::Arc<HashMap<u16, char>>,
    /// Content-byte → GID, from a `(3,0)`/`(1,0)` subtable. Empty unless the
    /// font is symbolic-style; lets decode do byte→GID before GID→Unicode.
    symbol_code_to_gid: std::sync::Arc<HashMap<u32, u16>>,
}

impl TrueTypeCMap {
    /// Parse TrueType cmap table from font data
    ///
    /// The TrueType sfnt structure contains a directory of tables.
    /// We locate the 'cmap' table and parse the best available subtable.
    ///
    /// Priority for cmap subtables:
    /// 1. Platform 3 (Windows), Encoding 10 (Unicode full repertoire) - supports all Unicode
    /// 2. Platform 3 (Windows), Encoding 1 (Unicode BMP) - supports basic multilingual plane
    /// 3. Platform 0 (Unicode), Encoding 3 - fallback to old Unicode platform
    pub fn from_font_data(data: &[u8]) -> Result<Self, String> {
        Self::from_font_data_with_budget(data, None)
    }

    pub(crate) fn from_font_data_with_budget(
        data: &[u8],
        budget: Option<&ImportBudget>,
    ) -> Result<Self, String> {
        if let Some(budget) = budget {
            budget.check().map_err(|e| e.to_string())?;
        }
        let mut cursor = Cursor::new(data);

        // Parse sfnt header to locate table directory
        let (num_tables, search_range, entry_selector, range_shift) =
            Self::parse_sfnt_header(&mut cursor)?;

        // Find cmap table entry in the directory
        let (cmap_offset, cmap_length) = Self::find_cmap_table(
            &mut cursor,
            num_tables,
            search_range,
            entry_selector,
            range_shift,
        )?;

        // Parse cmap table and find the best subtable
        let start = cmap_offset as usize;
        let end = start
            .checked_add(cmap_length as usize)
            .ok_or("cmap table offset overflow")?;
        let cmap_data = data.get(start..end).ok_or("cmap table outside font data")?;
        if let Some(budget) = budget {
            if cmap_data.len() > budget.limits.cmap_bytes {
                return Err(budget.reject("TrueType cmap source bytes").to_string());
            }
            budget
                .charge_cmap_scan(cmap_data.len())
                .map_err(|e| e.to_string())?;
        }
        let mut cursor = Cursor::new(cmap_data);
        let cmap_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read cmap version: {}", e))?;

        if cmap_version != 0 {
            return Err(format!("Unsupported cmap table version: {}", cmap_version));
        }

        let num_subtables = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read cmap subtable count: {}", e))?;

        // Read all subtable records and pick the best Unicode subtable for the
        // GID→Unicode map.
        let mut best_subtable: Option<(u32, u32, u32)> = None; // (platform_id, encoding_id, offset)
        let mut best_priority = -1i32;

        for _ in 0..num_subtables {
            let platform_id = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| format!("Failed to read platform ID: {}", e))?;
            let encoding_id = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| format!("Failed to read encoding ID: {}", e))?;
            let offset = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read subtable offset: {}", e))?;

            let priority = match (platform_id, encoding_id) {
                (3, 10) => 30, // Windows, Unicode full repertoire
                (3, 1) => 20,  // Windows, Unicode BMP
                (0, 3) => 10,  // Unicode platform, Unicode 2.0
                _ => 0,
            };

            if priority > best_priority {
                best_priority = priority;
                best_subtable = Some((platform_id as u32, encoding_id as u32, offset));
            }
        }

        let (platform_id, encoding_id, subtable_offset) =
            best_subtable.ok_or_else(|| "No suitable cmap subtable found".to_string())?;

        log::debug!(
            "TrueType cmap: selected platform={} encoding={} offset={}",
            platform_id,
            encoding_id,
            subtable_offset
        );

        // Offsets are relative to the cmap table, lengths to the subtable.
        // A format-4 glyph array must not consume adjacent font tables.
        let sub = cmap_data
            .get(subtable_offset as usize..)
            .ok_or("cmap subtable outside table")?;
        let mut header = Cursor::new(sub);
        let format = header.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
        if !matches!(format, 0 | 4 | 6 | 12) {
            return Err(format!("Unsupported cmap format: {}", format));
        }
        let length = if format == 12 {
            header.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
            header.read_u32::<BigEndian>().map_err(|e| e.to_string())? as usize
        } else {
            header.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize
        };
        let sub = sub
            .get(..length)
            .ok_or("cmap subtable length outside table")?;
        let gid_to_unicode = Self::parse_cmap_subtable(&mut Cursor::new(sub), budget)?;

        Ok(TrueTypeCMap {
            gid_to_unicode: std::sync::Arc::new(gid_to_unicode),
            // Symbolic fonts index glyphs by content byte, not GID — build
            // byte→GID from the (3,0)/(1,0) subtable so decode can do that hop.
            symbol_code_to_gid: std::sync::Arc::new(Self::build_symbol_code_to_gid(data, budget)?),
        })
    }

    /// Build a content-byte→GID map from the font's `(3,0)` symbol or `(1,0)`
    /// Macintosh cmap subtable, resolving the `0xF000` symbol-PUA offset at build
    /// time. Reuses `ttf-parser` (already a dependency). Empty when the font has
    /// no such subtable or fails to parse — decode then treats the byte as a GID.
    fn build_symbol_code_to_gid(
        data: &[u8],
        budget: Option<&ImportBudget>,
    ) -> Result<HashMap<u32, u16>, String> {
        use ttf_parser::PlatformId;
        let mut map = HashMap::new();
        let Ok(face) = ttf_parser::Face::parse(data, 0) else {
            return Ok(map);
        };
        let Some(cmap) = face.tables().cmap else {
            return Ok(map);
        };
        let mut chosen = None;
        let mut best = 0;
        for sub in cmap.subtables {
            let pri = match (sub.platform_id, sub.encoding_id) {
                (PlatformId::Windows, 0) => 2,   // symbol
                (PlatformId::Macintosh, 0) => 1, // Roman
                _ => 0,
            };
            if pri > best {
                best = pri;
                chosen = Some(sub);
            }
        }
        if let Some(sub) = chosen {
            Self::reserve_mappings(&mut map, 256, budget)?;
            for code in 0u32..256 {
                if let Some(gid) = sub
                    .glyph_index(code)
                    .or_else(|| sub.glyph_index(0xF000 | code))
                {
                    map.insert(code, gid.0);
                }
            }
        }
        Ok(map)
    }

    /// Content-byte → GID via the font's symbol/Mac cmap. `None` when the font
    /// has no such subtable (decode then falls back to treating the byte as a GID).
    pub fn code_to_gid(&self, code: u16) -> Option<u16> {
        self.symbol_code_to_gid.get(&(code as u32)).copied()
    }

    /// Get Unicode character for a glyph ID
    pub fn get_unicode(&self, gid: u16) -> Option<char> {
        self.gid_to_unicode.get(&gid).copied()
    }

    /// Get the number of glyph mappings
    pub fn len(&self) -> usize {
        self.gid_to_unicode.len()
    }

    /// Check if cmap is empty
    pub fn is_empty(&self) -> bool {
        self.gid_to_unicode.is_empty()
    }

    // ==================================================================================
    // Private Helper Methods
    // ==================================================================================

    // Count expanded work (including overwrites), then reserve requested map
    // slots. This is cumulative allocation work, not allocator/RSS accounting.
    fn reserve_mappings<K: std::hash::Hash + Eq, V>(
        map: &mut HashMap<K, V>,
        count: usize,
        budget: Option<&ImportBudget>,
    ) -> Result<(), String> {
        if let Some(budget) = budget {
            budget
                .charge_font_entries(count)
                .map_err(|e| e.to_string())?;
        }
        map.try_reserve(count).map_err(|_| match budget {
            Some(budget) => budget.reject("TrueType map allocation").to_string(),
            None => "TrueType map allocation failed".into(),
        })
    }

    fn zeroed_array<T: Default + Clone>(
        count: usize,
        budget: Option<&ImportBudget>,
    ) -> Result<Vec<T>, String> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or("TrueType array size overflow")?;
        if let Some(budget) = budget {
            budget.charge_decoded(bytes).map_err(|e| e.to_string())?;
        }
        let mut values = Vec::new();
        values.try_reserve_exact(count).map_err(|_| match budget {
            Some(budget) => budget.reject("TrueType array allocation").to_string(),
            None => "TrueType array allocation failed".into(),
        })?;
        values.resize(count, T::default());
        Ok(values)
    }

    fn parse_sfnt_header(cursor: &mut Cursor<&[u8]>) -> Result<(u16, u16, u16, u16), String> {
        // Read sfnt version (4 bytes - can be 0x00010000 for TrueType or "OTTO" for OpenType)
        let version = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read sfnt version: {}", e))?;

        // 0x00010000 = TrueType, 0x4F54544F = OpenType (OTTO), 0x74727565 = Apple TrueType ("true")
        if version != 0x00010000 && version != 0x4F54544F && version != 0x74727565 {
            // 0x4F54544F = "OTTO" (OpenType)
            return Err(format!("Invalid sfnt version: 0x{:08X}", version));
        }

        let num_tables = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read table count: {}", e))?;
        let search_range = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read search range: {}", e))?;
        let entry_selector = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read entry selector: {}", e))?;
        let range_shift = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read range shift: {}", e))?;

        Ok((num_tables, search_range, entry_selector, range_shift))
    }

    fn find_cmap_table(
        cursor: &mut Cursor<&[u8]>,
        num_tables: u16,
        _search_range: u16,
        _entry_selector: u16,
        _range_shift: u16,
    ) -> Result<(u32, u32), String> {
        // Linear search through table directory for 'cmap' tag (0x636D6170)
        const CMAP_TAG: u32 = 0x636D6170;

        for _ in 0..num_tables {
            let tag = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read table tag: {}", e))?;
            let _checksum = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read table checksum: {}", e))?;
            let offset = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read table offset: {}", e))?;
            let length = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read table length: {}", e))?;

            if tag == CMAP_TAG {
                return Ok((offset, length));
            }
        }

        Err("cmap table not found in font".to_string())
    }

    fn parse_cmap_subtable(
        cursor: &mut Cursor<&[u8]>,
        budget: Option<&ImportBudget>,
    ) -> Result<HashMap<u16, char>, String> {
        let format = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read cmap format: {}", e))?;

        match format {
            0 => Self::parse_cmap_format0(cursor, budget),
            4 => Self::parse_cmap_format4(cursor, budget),
            6 => Self::parse_cmap_format6(cursor, budget),
            12 => Self::parse_cmap_format12(cursor, budget),
            _ => Err(format!("Unsupported cmap format: {}", format)),
        }
    }

    /// Parse cmap format 0 (legacy 1-byte indexed, Mac Roman era).
    ///
    /// Structure per Apple TrueType reference (subtable length 262):
    ///   u16 format       (already read by caller — 2 bytes)
    ///   u16 length       (262: 2+2+2+256)
    ///   u16 language
    ///   u8  glyphIdArray[256]   — glyphId for each byte code 0..255
    ///
    /// Microsoft Office subset fonts (Calibri, Times New Roman subsets in
    /// Word/Excel exports) still ship with a format-0 cmap for the `(1,0)`
    /// Macintosh encoding alongside their `(3,1)` Unicode cmap. When the
    /// Unicode cmap is missing or malformed we fall back to this one; the
    /// byte code acts as the Mac Roman character code.
    ///
    /// Benchmark canary: ~8 MS Office corpus fixtures previously logged
    /// "Unsupported cmap format: 0" warnings and lost font glyph mapping
    /// as a consequence (B9).
    fn parse_cmap_format0(
        cursor: &mut Cursor<&[u8]>,
        budget: Option<&ImportBudget>,
    ) -> Result<HashMap<u16, char>, String> {
        let _length = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read format 0 length: {}", e))?;
        let _language = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read format 0 language: {}", e))?;

        let mut glyph_ids = [0u8; 256];
        std::io::Read::read_exact(cursor, &mut glyph_ids)
            .map_err(|e| format!("Failed to read format 0 glyphIdArray: {}", e))?;

        let mut gid_to_unicode = HashMap::new();
        Self::reserve_mappings(&mut gid_to_unicode, 256, budget)?;
        for (byte, &gid) in glyph_ids.iter().enumerate() {
            if gid == 0 {
                continue;
            }
            // ASCII pass-through for 0x00..0x7F (Mac Roman lower half is
            // identical to ASCII). Above 0x7F we route through the Mac
            // Roman → Unicode table so byte 0x8A (a-umlaut in Mac Roman)
            // maps to U+00E4 instead of U+008A.
            let ch = if byte < 0x80 {
                char::from_u32(byte as u32)
            } else {
                Some(mac_roman_to_unicode(byte as u8))
            };
            if let Some(ch) = ch {
                gid_to_unicode.insert(gid as u16, ch);
            }
        }
        Ok(gid_to_unicode)
    }

    /// Parse cmap format 4 (BMP - supports characters U+0000 to U+FFFF)
    fn parse_cmap_format4(
        cursor: &mut Cursor<&[u8]>,
        budget: Option<&ImportBudget>,
    ) -> Result<HashMap<u16, char>, String> {
        let _length = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read format 4 length: {}", e))?
            as u32;
        let _language = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read format 4 language: {}", e))?;

        let seg_count_x2 = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read segCountX2: {}", e))?
            as usize;
        if !seg_count_x2.is_multiple_of(2) {
            return Err("odd TrueType segCountX2".into());
        }
        let seg_count = seg_count_x2 / 2;

        // Skip binary search parameters
        let _search_range = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read searchRange: {}", e))?;
        let _entry_selector = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read entrySelector: {}", e))?;
        let _range_shift = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read rangeShift: {}", e))?;

        // Validate the complete array payload before reserving any of it.
        if cursor
            .get_ref()
            .len()
            .saturating_sub(cursor.position() as usize)
            < seg_count * 8 + 2
        {
            return Err("truncated TrueType segment arrays".into());
        }
        // Read segment arrays
        let mut end_codes = Self::zeroed_array::<u16>(seg_count, budget)?;
        for i in 0..seg_count {
            end_codes[i] = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| format!("Failed to read endCode[{}]: {}", i, e))?;
        }

        // Reserved pad
        let _reserved = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read reserved pad: {}", e))?;

        let mut start_codes = Self::zeroed_array::<u16>(seg_count, budget)?;
        for i in 0..seg_count {
            start_codes[i] = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| format!("Failed to read startCode[{}]: {}", i, e))?;
        }

        let mut id_deltas = Self::zeroed_array::<i16>(seg_count, budget)?;
        for i in 0..seg_count {
            id_deltas[i] = cursor
                .read_i16::<BigEndian>()
                .map_err(|e| format!("Failed to read idDelta[{}]: {}", i, e))?;
        }

        // id_range_offsets require special parsing - just read as array
        let mut id_range_offsets = Self::zeroed_array::<u16>(seg_count, budget)?;
        for i in 0..seg_count {
            id_range_offsets[i] = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| format!("Failed to read idRangeOffset[{}]: {}", i, e))?;
        }

        // Read remaining bytes as glyphIdArray (used when idRangeOffset != 0)
        let glyph_count = cursor
            .get_ref()
            .len()
            .saturating_sub(cursor.position() as usize)
            / 2;
        let mut glyph_id_array = Self::zeroed_array::<u16>(glyph_count, budget)?;
        for value in &mut glyph_id_array {
            *value = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
        }

        // Build character to GID mappings
        let mut gid_to_unicode = HashMap::new();

        for seg in 0..seg_count {
            let start = start_codes[seg] as u32;
            let end = end_codes[seg] as u32;
            let id_delta = id_deltas[seg] as i32;

            let count = end
                .min(0xfffe)
                .checked_sub(start)
                .map_or(0, |n| n as usize + 1);
            Self::reserve_mappings(&mut gid_to_unicode, count, budget)?;
            for char_code in start..=end {
                if char_code == 0xFFFF {
                    break; // End segment marker
                }

                let gid = if id_range_offsets[seg] == 0 {
                    // Simple formula: GID = charCode + idDelta
                    (char_code as i32 + id_delta) as u16
                } else {
                    // Per TrueType spec: index into glyphIdArray
                    // offset = idRangeOffset[i]/2 + (charCode - startCode[i]) + i - segCount
                    let offset = (id_range_offsets[seg] as usize) / 2
                        + (char_code as usize - start as usize)
                        + seg;
                    let offset = offset
                        .checked_sub(seg_count)
                        .ok_or("invalid TrueType glyph offset")?;
                    if offset < glyph_id_array.len() {
                        let raw = glyph_id_array[offset];
                        if raw != 0 {
                            (raw as i32 + id_delta) as u16
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };

                if gid != 0 {
                    if let Some(ch) = char::from_u32(char_code) {
                        gid_to_unicode.insert(gid, ch);
                    }
                }
            }
        }

        Ok(gid_to_unicode)
    }

    /// Parse cmap format 6 (trimmed table)
    fn parse_cmap_format6(
        cursor: &mut Cursor<&[u8]>,
        budget: Option<&ImportBudget>,
    ) -> Result<HashMap<u16, char>, String> {
        let _length = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read format 6 length: {}", e))?;
        let _language = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read format 6 language: {}", e))?;

        let first_code = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read firstCode: {}", e))?;
        let count = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read entryCount: {}", e))? as usize;

        let mut gid_to_unicode = HashMap::new();

        if cursor
            .get_ref()
            .len()
            .saturating_sub(cursor.position() as usize)
            < count * 2
        {
            return Err("truncated TrueType glyph array".into());
        }
        Self::reserve_mappings(&mut gid_to_unicode, count, budget)?;
        for i in 0..count {
            let gid = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| format!("Failed to read glyphId[{}]: {}", i, e))?;

            let char_code = first_code as u32 + i as u32;
            if let Some(ch) = char::from_u32(char_code) {
                gid_to_unicode.insert(gid, ch);
            }
        }

        Ok(gid_to_unicode)
    }

    /// Parse cmap format 12 (segmented coverage - supports full Unicode)
    fn parse_cmap_format12(
        cursor: &mut Cursor<&[u8]>,
        budget: Option<&ImportBudget>,
    ) -> Result<HashMap<u16, char>, String> {
        // Skip reserved bytes
        let _reserved = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read reserved: {}", e))?;

        let _length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read format 12 length: {}", e))?;
        let _language = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read format 12 language: {}", e))?;

        let num_groups = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read numGroups: {}", e))?
            as usize;

        let mut gid_to_unicode = HashMap::new();

        let group_bytes = num_groups
            .checked_mul(12)
            .ok_or("TrueType group size overflow")?;
        if cursor
            .get_ref()
            .len()
            .saturating_sub(cursor.position() as usize)
            < group_bytes
        {
            return Err("truncated TrueType groups".into());
        }
        for _ in 0..num_groups {
            let start_char_code = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read startCharCode: {}", e))?;
            let end_char_code = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read endCharCode: {}", e))?;
            let start_gid = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| format!("Failed to read startGlyphId: {}", e))?;

            let delta = end_char_code
                .checked_sub(start_char_code)
                .ok_or("reversed TrueType group")?;
            if end_char_code > 0x10ffff
                || start_gid
                    .checked_add(delta)
                    .is_none_or(|gid| gid > u16::MAX as u32)
            {
                return Err("TrueType group exceeds Unicode or glyph range".into());
            }
            Self::reserve_mappings(&mut gid_to_unicode, delta as usize + 1, budget)?;
            for (offset, char_code) in (start_char_code..=end_char_code).enumerate() {
                let gid = (start_gid + offset as u32) as u16;
                if let Some(ch) = char::from_u32(char_code) {
                    gid_to_unicode.insert(gid, ch);
                }
            }
        }

        Ok(gid_to_unicode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BigEndian, WriteBytesExt};

    // The directory length is a real OpenType boundary, not an unused field.
    // These synthetic fonts put their only cmap table at offset 28.
    fn finish_cmap_length(data: &mut [u8]) {
        let length = (data.len() - 28) as u32;
        data[24..28].copy_from_slice(&length.to_be_bytes());
    }

    /// Build a minimal TrueType font with a cmap format 4 table.
    fn build_truetype_with_cmap_format4(mappings: &[(u16, u16)]) -> Vec<u8> {
        // We need: sfnt header + table directory (1 table: cmap) + cmap table
        let mut data = Vec::new();

        // ---- sfnt header ----
        data.write_u32::<BigEndian>(0x00010000).unwrap(); // TrueType version
        data.write_u16::<BigEndian>(1).unwrap(); // numTables = 1
        data.write_u16::<BigEndian>(16).unwrap(); // searchRange
        data.write_u16::<BigEndian>(0).unwrap(); // entrySelector
        data.write_u16::<BigEndian>(0).unwrap(); // rangeShift

        // ---- table directory (1 entry) ----
        let cmap_offset: u32 = 12 + 16; // sfnt header (12) + 1 table record (16)
        data.write_u32::<BigEndian>(0x636D6170).unwrap(); // 'cmap' tag
        data.write_u32::<BigEndian>(0).unwrap(); // checksum (unused)
        data.write_u32::<BigEndian>(cmap_offset).unwrap(); // offset
        data.write_u32::<BigEndian>(0).unwrap(); // length placeholder

        // ---- cmap table header ----
        data.write_u16::<BigEndian>(0).unwrap(); // version
        data.write_u16::<BigEndian>(1).unwrap(); // numSubtables = 1

        // subtable record: platform=3 (Windows), encoding=1 (Unicode BMP)
        let subtable_offset: u32 = 4 + 8; // cmap header (4) + 1 record (8)
        data.write_u16::<BigEndian>(3).unwrap(); // platformID
        data.write_u16::<BigEndian>(1).unwrap(); // encodingID
        data.write_u32::<BigEndian>(subtable_offset).unwrap();

        // ---- cmap format 4 subtable ----
        // Build segments from mappings. Each mapping is (charCode, gid).
        // For simplicity, create one segment per mapping + the sentinel 0xFFFF segment.
        let mut segments: Vec<(u16, u16, i16)> = Vec::new(); // (start, end, delta)
        for &(char_code, gid) in mappings {
            let delta = gid as i16 - char_code as i16;
            segments.push((char_code, char_code, delta));
        }
        segments.push((0xFFFF, 0xFFFF, 1)); // sentinel

        let seg_count = segments.len();
        let seg_count_x2 = (seg_count * 2) as u16;

        data.write_u16::<BigEndian>(4).unwrap(); // format
                                                 // length placeholder (we'll fill in later)
        let length_pos = data.len();
        data.write_u16::<BigEndian>(0).unwrap(); // length
        data.write_u16::<BigEndian>(0).unwrap(); // language

        data.write_u16::<BigEndian>(seg_count_x2).unwrap();
        data.write_u16::<BigEndian>(0).unwrap(); // searchRange
        data.write_u16::<BigEndian>(0).unwrap(); // entrySelector
        data.write_u16::<BigEndian>(0).unwrap(); // rangeShift

        // endCode array
        for seg in &segments {
            data.write_u16::<BigEndian>(seg.1).unwrap();
        }
        // reserved pad
        data.write_u16::<BigEndian>(0).unwrap();
        // startCode array
        for seg in &segments {
            data.write_u16::<BigEndian>(seg.0).unwrap();
        }
        // idDelta array
        for seg in &segments {
            data.write_i16::<BigEndian>(seg.2).unwrap();
        }
        // idRangeOffset array (all zeros = use delta formula)
        for _ in &segments {
            data.write_u16::<BigEndian>(0).unwrap();
        }

        // Fill in format 4 length
        let fmt4_start = length_pos - 2; // format field
        let fmt4_len = data.len() - fmt4_start;
        let len_bytes = (fmt4_len as u16).to_be_bytes();
        data[length_pos] = len_bytes[0];
        data[length_pos + 1] = len_bytes[1];

        finish_cmap_length(&mut data);
        data
    }

    /// Build a minimal TrueType font with a cmap format 0 table.
    /// `glyph_ids` is the 256-entry byte → glyph id array.
    fn build_truetype_with_cmap_format0(glyph_ids: [u8; 256]) -> Vec<u8> {
        let mut data = Vec::new();

        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap(); // numTables
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();

        // table directory
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap(); // 'cmap'
        data.write_u32::<BigEndian>(0).unwrap(); // checksum
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap(); // length

        // cmap table header
        data.write_u16::<BigEndian>(0).unwrap(); // version
        data.write_u16::<BigEndian>(1).unwrap(); // numSubtables = 1

        // Use platform (1, 0) — Macintosh / Roman — so the parser routes
        // directly through the format-0 path.
        let subtable_offset: u32 = 4 + 8;
        data.write_u16::<BigEndian>(1).unwrap(); // platformID = Macintosh
        data.write_u16::<BigEndian>(0).unwrap(); // encodingID = Roman
        data.write_u32::<BigEndian>(subtable_offset).unwrap();

        // ---- cmap format 0 subtable ----
        data.write_u16::<BigEndian>(0).unwrap(); // format
        data.write_u16::<BigEndian>(262).unwrap(); // length (fixed for format 0)
        data.write_u16::<BigEndian>(0).unwrap(); // language
        data.extend_from_slice(&glyph_ids);

        finish_cmap_length(&mut data);
        data
    }

    /// Build a minimal TrueType font with a cmap format 6 table.
    fn build_truetype_with_cmap_format6(first_code: u16, gids: &[u16]) -> Vec<u8> {
        let mut data = Vec::new();

        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();

        // table directory
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        // cmap header
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(3).unwrap(); // platform 3
        data.write_u16::<BigEndian>(1).unwrap(); // encoding 1
        data.write_u32::<BigEndian>(4 + 8).unwrap();

        // format 6
        data.write_u16::<BigEndian>(6).unwrap(); // format
        data.write_u16::<BigEndian>((10 + gids.len() * 2) as u16)
            .unwrap(); // length
        data.write_u16::<BigEndian>(0).unwrap(); // language
        data.write_u16::<BigEndian>(first_code).unwrap();
        data.write_u16::<BigEndian>(gids.len() as u16).unwrap();
        for &gid in gids {
            data.write_u16::<BigEndian>(gid).unwrap();
        }

        finish_cmap_length(&mut data);
        data
    }

    /// Build a minimal TrueType font with a cmap format 12 table.
    fn build_truetype_with_cmap_format12(groups: &[(u32, u32, u32)]) -> Vec<u8> {
        let mut data = Vec::new();

        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();

        // table directory
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        // cmap header
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(3).unwrap(); // platform 3
        data.write_u16::<BigEndian>(10).unwrap(); // encoding 10 (full repertoire)
        data.write_u32::<BigEndian>(4 + 8).unwrap();

        // format 12
        data.write_u16::<BigEndian>(12).unwrap(); // format
        data.write_u16::<BigEndian>(0).unwrap(); // reserved
        data.write_u32::<BigEndian>((16 + groups.len() * 12) as u32)
            .unwrap(); // length
        data.write_u32::<BigEndian>(0).unwrap(); // language
        data.write_u32::<BigEndian>(groups.len() as u32).unwrap();
        for &(start, end, start_gid) in groups {
            data.write_u32::<BigEndian>(start).unwrap();
            data.write_u32::<BigEndian>(end).unwrap();
            data.write_u32::<BigEndian>(start_gid).unwrap();
        }

        finish_cmap_length(&mut data);
        data
    }

    /// A standalone cmap format-4 subtable body (format … idRangeOffset), one
    /// segment per `(code, gid)` mapping plus the `0xFFFF` sentinel.
    fn fmt4_subtable(mappings: &[(u16, u16)]) -> Vec<u8> {
        let mut segs: Vec<(u16, u16, i16)> = mappings
            .iter()
            .map(|&(c, g)| (c, c, (g as i32 - c as i32) as i16))
            .collect();
        segs.push((0xFFFF, 0xFFFF, 1));
        let mut s = Vec::new();
        s.write_u16::<BigEndian>(4).unwrap(); // format
        let len_pos = s.len();
        s.write_u16::<BigEndian>(0).unwrap(); // length placeholder
        s.write_u16::<BigEndian>(0).unwrap(); // language
        s.write_u16::<BigEndian>((segs.len() * 2) as u16).unwrap(); // segCountX2
        for _ in 0..3 {
            s.write_u16::<BigEndian>(0).unwrap(); // searchRange/entrySelector/rangeShift
        }
        for seg in &segs {
            s.write_u16::<BigEndian>(seg.1).unwrap(); // endCode
        }
        s.write_u16::<BigEndian>(0).unwrap(); // reserved pad
        for seg in &segs {
            s.write_u16::<BigEndian>(seg.0).unwrap(); // startCode
        }
        for seg in &segs {
            s.write_i16::<BigEndian>(seg.2).unwrap(); // idDelta
        }
        for _ in &segs {
            s.write_u16::<BigEndian>(0).unwrap(); // idRangeOffset
        }
        let len = (s.len() as u16).to_be_bytes();
        s[len_pos] = len[0];
        s[len_pos + 1] = len[1];
        s
    }

    /// Two-subtable cmap: a `(3,1)` Unicode subtable and a `(3,0)` symbol one.
    fn cmap_two_subtables(unicode: &[(u16, u16)], symbol: &[(u16, u16)]) -> Vec<u8> {
        let body_uni = fmt4_subtable(unicode);
        let body_sym = fmt4_subtable(symbol);
        let off_uni = 4 + 2 * 8; // version+numTables + 2 encoding records
        let off_sym = off_uni + body_uni.len();
        let mut t = Vec::new();
        t.write_u16::<BigEndian>(0).unwrap(); // version
        t.write_u16::<BigEndian>(2).unwrap(); // numTables
        for (plat, enc, off) in [(3u16, 1u16, off_uni), (3, 0, off_sym)] {
            t.write_u16::<BigEndian>(plat).unwrap();
            t.write_u16::<BigEndian>(enc).unwrap();
            t.write_u32::<BigEndian>(off as u32).unwrap();
        }
        t.extend(body_uni);
        t.extend(body_sym);
        t
    }

    /// Assemble an sfnt from `(tag, data)` tables — sorted by tag (the directory
    /// is binary-searched), 4-byte aligned. Just enough for `ttf-parser`.
    fn assemble(mut tables: Vec<(u32, Vec<u8>)>) -> Vec<u8> {
        tables.sort_by_key(|(tag, _)| *tag);
        let mut out = Vec::new();
        out.write_u32::<BigEndian>(0x0001_0000).unwrap(); // sfnt version
        out.write_u16::<BigEndian>(tables.len() as u16).unwrap();
        for _ in 0..3 {
            out.write_u16::<BigEndian>(0).unwrap(); // searchRange/entrySelector/rangeShift
        }
        let mut offset = 12 + tables.len() * 16;
        for (tag, data) in &tables {
            out.write_u32::<BigEndian>(*tag).unwrap();
            out.write_u32::<BigEndian>(0).unwrap(); // checksum (unchecked)
            out.write_u32::<BigEndian>(offset as u32).unwrap();
            out.write_u32::<BigEndian>(data.len() as u32).unwrap();
            offset += (data.len() + 3) & !3;
        }
        for (_, data) in &tables {
            out.extend_from_slice(data);
            while out.len() % 4 != 0 {
                out.push(0);
            }
        }
        out
    }

    /// Regression for the symbolic-font Ç→Ê bug, exercising the real
    /// `from_font_data` → `ttf-parser` extraction path. The content byte is NOT
    /// the GID: the `(3,0)` subtable maps byte 3 → GID 2 (the `Ç` glyph), while
    /// the byte used directly as a GID hits GID 3 → `Ê` (the old behaviour).
    #[test]
    fn symbolic_byte_resolves_via_symbol_cmap() {
        // head/hhea/maxp are the minimum ttf-parser's Face::parse requires.
        let mut head = Vec::new();
        head.write_u16::<BigEndian>(1).unwrap(); // majorVersion
        head.write_u16::<BigEndian>(0).unwrap(); // minorVersion
        head.write_u32::<BigEndian>(0).unwrap(); // fontRevision
        head.write_u32::<BigEndian>(0).unwrap(); // checkSumAdjustment
        head.write_u32::<BigEndian>(0x5F0F_3CF5).unwrap(); // magicNumber
        head.write_u16::<BigEndian>(0).unwrap(); // flags
        head.write_u16::<BigEndian>(1000).unwrap(); // unitsPerEm (offset 18)
        head.write_u64::<BigEndian>(0).unwrap(); // created
        head.write_u64::<BigEndian>(0).unwrap(); // modified
        for v in [0i16, 0, 1000, 1000, 0, 0, 0, 0, 0] {
            head.write_i16::<BigEndian>(v).unwrap(); // xMin..glyphDataFormat (incl. indexToLocFormat=0)
        }
        assert_eq!(head.len(), 54);

        let mut hhea = Vec::new();
        hhea.write_u16::<BigEndian>(1).unwrap(); // majorVersion
        hhea.write_u16::<BigEndian>(0).unwrap(); // minorVersion
        for v in [800i16, -200, 0, 1000, 0, 0, 1000, 1, 0, 0, 0, 0, 0, 0, 0] {
            hhea.write_i16::<BigEndian>(v).unwrap();
        }
        hhea.write_u16::<BigEndian>(1).unwrap(); // numberOfHMetrics (offset 34)
        assert_eq!(hhea.len(), 36);

        let mut maxp = Vec::new();
        maxp.write_u32::<BigEndian>(0x0000_5000).unwrap(); // version 0.5
        maxp.write_u16::<BigEndian>(4).unwrap(); // numGlyphs

        let cmap = cmap_two_subtables(&[(0x00C7, 2), (0x00CA, 3)], &[(3, 2)]);
        let font = assemble(vec![
            (0x6865_6164, head), // 'head'
            (0x6868_6561, hhea), // 'hhea'
            (0x6D61_7870, maxp), // 'maxp'
            (0x636D_6170, cmap), // 'cmap'
        ]);

        let parsed = TrueTypeCMap::from_font_data(&font).expect("parse synthetic font");
        assert_eq!(parsed.code_to_gid(3), Some(2)); // byte → GID via (3,0)
        assert_eq!(parsed.get_unicode(2), Some('Ç')); // fixed: byte → GID → Unicode
        assert_eq!(parsed.get_unicode(3), Some('Ê')); // old byte-as-GID path was wrong
        assert_eq!(parsed.code_to_gid(99), None); // no symbol entry → fallback to byte-as-GID
    }

    #[test]
    fn test_sfnt_header_parsing() {
        // Valid TrueType with empty cmap format 4
        let data = build_truetype_with_cmap_format4(&[]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert!(cmap.is_empty());
    }

    #[test]
    fn test_invalid_sfnt_version() {
        let mut data = vec![0u8; 100];
        // Invalid version bytes
        data[0] = 0xFF;
        data[1] = 0xFF;
        data[2] = 0xFF;
        data[3] = 0xFF;
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid sfnt version"));
    }

    #[test]
    fn test_opentype_version_accepted() {
        // Build data with OTTO version
        let mut data = build_truetype_with_cmap_format4(&[(65, 1)]); // 'A' -> gid 1
                                                                     // Replace version with OTTO (0x4F54544F)
        data[0] = 0x4F;
        data[1] = 0x54;
        data[2] = 0x54;
        data[3] = 0x4F;
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apple_truetype_version_accepted() {
        let mut data = build_truetype_with_cmap_format4(&[(65, 1)]);
        // Replace version with "true" (0x74727565)
        data[0] = 0x74;
        data[1] = 0x72;
        data[2] = 0x75;
        data[3] = 0x65;
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_cmap_table() {
        let mut data = Vec::new();
        // sfnt header with 1 table but NOT cmap
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        // table record for 'head' (not 'cmap')
        data.write_u32::<BigEndian>(0x68656164).unwrap(); // 'head'
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(28).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cmap table not found"));
    }

    #[test]
    fn test_format4_basic_ascii() {
        // Map A(65)->1, B(66)->2, C(67)->3
        let data = build_truetype_with_cmap_format4(&[(65, 1), (66, 2), (67, 3)]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.len(), 3);
        assert_eq!(cmap.get_unicode(1), Some('A'));
        assert_eq!(cmap.get_unicode(2), Some('B'));
        assert_eq!(cmap.get_unicode(3), Some('C'));
        assert_eq!(cmap.get_unicode(4), None);
    }

    #[test]
    fn test_format4_extended_unicode() {
        // Map some non-ASCII: é(233)->10, ñ(241)->11
        let data = build_truetype_with_cmap_format4(&[(233, 10), (241, 11)]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.get_unicode(10), Some('é'));
        assert_eq!(cmap.get_unicode(11), Some('ñ'));
    }

    #[test]
    fn test_format6_basic() {
        // Format 6: first_code=65, gids=[1, 2, 3] -> maps A->gid1, B->gid2, C->gid3
        let data = build_truetype_with_cmap_format6(65, &[1, 2, 3]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.len(), 3);
        assert_eq!(cmap.get_unicode(1), Some('A'));
        assert_eq!(cmap.get_unicode(2), Some('B'));
        assert_eq!(cmap.get_unicode(3), Some('C'));
    }

    #[test]
    fn test_format6_non_zero_first_code() {
        // Start at code 48 ('0') for digits
        let data = build_truetype_with_cmap_format6(48, &[10, 11, 12]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.get_unicode(10), Some('0'));
        assert_eq!(cmap.get_unicode(11), Some('1'));
        assert_eq!(cmap.get_unicode(12), Some('2'));
    }

    #[test]
    fn test_format12_basic() {
        // One group: chars 65-67 -> gids 1-3
        let data = build_truetype_with_cmap_format12(&[(65, 67, 1)]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.len(), 3);
        assert_eq!(cmap.get_unicode(1), Some('A'));
        assert_eq!(cmap.get_unicode(2), Some('B'));
        assert_eq!(cmap.get_unicode(3), Some('C'));
    }

    #[test]
    fn test_format12_multiple_groups() {
        let data = build_truetype_with_cmap_format12(&[
            (65, 67, 1),  // A-C -> gids 1-3
            (48, 50, 10), // 0-2 -> gids 10-12
        ]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.len(), 6);
        assert_eq!(cmap.get_unicode(1), Some('A'));
        assert_eq!(cmap.get_unicode(10), Some('0'));
        assert_eq!(cmap.get_unicode(12), Some('2'));
    }

    #[test]
    fn test_get_unicode_missing() {
        let data = build_truetype_with_cmap_format4(&[(65, 1)]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.get_unicode(999), None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let data_empty = build_truetype_with_cmap_format4(&[]);
        let cmap_empty = TrueTypeCMap::from_font_data(&data_empty).unwrap();
        assert_eq!(cmap_empty.len(), 0);
        assert!(cmap_empty.is_empty());

        let data_one = build_truetype_with_cmap_format4(&[(65, 1)]);
        let cmap_one = TrueTypeCMap::from_font_data(&data_one).unwrap();
        assert_eq!(cmap_one.len(), 1);
        assert!(!cmap_one.is_empty());
    }

    #[test]
    fn test_cmap_format0_byte_indexed() {
        // Build a format-0 cmap where byte code 0x41 ('A') maps to gid 10,
        // 0x42 ('B') to gid 11, and everything else is 0 (.notdef).
        let mut gids = [0u8; 256];
        gids[0x41] = 10;
        gids[0x42] = 11;
        gids[0x7A] = 50;
        let data = build_truetype_with_cmap_format0(gids);
        let cmap = TrueTypeCMap::from_font_data(&data).expect("format 0 parse");
        assert_eq!(cmap.get_unicode(10), Some('A'));
        assert_eq!(cmap.get_unicode(11), Some('B'));
        assert_eq!(cmap.get_unicode(50), Some('z'));
        // Absent glyphs map to nothing.
        assert_eq!(cmap.get_unicode(99), None);
    }

    #[test]
    fn test_cmap_format0_mac_roman_high_half() {
        // Byte 0x8A in Mac Roman is 'ä' (U+00E4), not raw 0x8A.
        let mut gids = [0u8; 256];
        gids[0x41] = 10; // 'A' — ASCII pass-through
        gids[0x8A] = 20; // 'ä' via Mac Roman table
        gids[0xA5] = 30; // '•' bullet via Mac Roman
        let data = build_truetype_with_cmap_format0(gids);
        let cmap = TrueTypeCMap::from_font_data(&data).expect("format 0 parse");
        assert_eq!(cmap.get_unicode(10), Some('A'));
        assert_eq!(
            cmap.get_unicode(20),
            Some('ä'),
            "Mac Roman 0x8A must map to U+00E4 (a-umlaut), not raw 0x8A"
        );
        assert_eq!(cmap.get_unicode(30), Some('•'));
    }

    #[test]
    fn test_cmap_format0_rejects_truncated() {
        // 256-byte array but declared length wrong → truncated.
        let mut data = Vec::new();
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(4 + 8).unwrap();
        data.write_u16::<BigEndian>(0).unwrap(); // format
                                                 // Declare the correct length (262) but only append 8 bytes of
                                                 // glyphIdArray instead of 256 — parser should detect the
                                                 // truncation via read_exact.
        data.write_u16::<BigEndian>(262).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.extend_from_slice(&[0u8; 8]);
        finish_cmap_length(&mut data);
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_cmap_format() {
        let mut data = Vec::new();
        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        // cmap table directory entry
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        // cmap header
        data.write_u16::<BigEndian>(0).unwrap(); // version
        data.write_u16::<BigEndian>(1).unwrap(); // 1 subtable
        data.write_u16::<BigEndian>(3).unwrap(); // platform 3
        data.write_u16::<BigEndian>(1).unwrap(); // encoding 1
        data.write_u32::<BigEndian>(4 + 8).unwrap(); // subtable offset
                                                     // format 2 (unsupported)
        data.write_u16::<BigEndian>(2).unwrap();

        finish_cmap_length(&mut data);
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported cmap format"));
    }

    #[test]
    fn test_unsupported_cmap_version() {
        let mut data = Vec::new();
        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        // cmap table directory entry
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        // cmap header with invalid version
        data.write_u16::<BigEndian>(99).unwrap(); // version != 0

        finish_cmap_length(&mut data);
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unsupported cmap table version"));
    }

    #[test]
    fn test_no_suitable_subtable() {
        let mut data = Vec::new();
        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        // cmap table directory entry
        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        // cmap header with 0 subtables
        data.write_u16::<BigEndian>(0).unwrap(); // version
        data.write_u16::<BigEndian>(0).unwrap(); // 0 subtables

        finish_cmap_length(&mut data);
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No suitable cmap subtable"));
    }

    #[test]
    fn test_truncated_data() {
        // Just a few bytes - not even a valid header
        let data = vec![0u8; 4];
        let result = TrueTypeCMap::from_font_data(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_and_debug() {
        let data = build_truetype_with_cmap_format4(&[(65, 1)]);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        let cloned = cmap.clone();
        assert_eq!(cloned.get_unicode(1), Some('A'));
        let debug = format!("{:?}", cmap);
        assert!(debug.contains("TrueTypeCMap"));
    }

    #[test]
    fn test_platform_priority_windows_full_over_bmp() {
        // Build a font with 2 subtables: platform 3/encoding 1 and 3/10
        // The 3/10 (full) should be preferred
        let mut data = Vec::new();

        // sfnt header
        data.write_u32::<BigEndian>(0x00010000).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u16::<BigEndian>(16).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();
        data.write_u16::<BigEndian>(0).unwrap();

        let cmap_offset: u32 = 12 + 16;
        data.write_u32::<BigEndian>(0x636D6170).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(cmap_offset).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        // cmap header with 2 subtables
        data.write_u16::<BigEndian>(0).unwrap(); // version
        data.write_u16::<BigEndian>(2).unwrap(); // 2 subtables

        // Both point to same subtable (format 12) for simplicity
        let subtable_off: u32 = 4 + 8 * 2; // cmap header + 2 records
                                           // Record 1: platform 3, encoding 1
        data.write_u16::<BigEndian>(3).unwrap();
        data.write_u16::<BigEndian>(1).unwrap();
        data.write_u32::<BigEndian>(subtable_off).unwrap();
        // Record 2: platform 3, encoding 10 (higher priority)
        data.write_u16::<BigEndian>(3).unwrap();
        data.write_u16::<BigEndian>(10).unwrap();
        data.write_u32::<BigEndian>(subtable_off).unwrap();

        // format 12 subtable: one group: A(65)->gid1
        data.write_u16::<BigEndian>(12).unwrap();
        data.write_u16::<BigEndian>(0).unwrap(); // reserved
        data.write_u32::<BigEndian>(28).unwrap(); // length
        data.write_u32::<BigEndian>(0).unwrap(); // language
        data.write_u32::<BigEndian>(1).unwrap(); // 1 group
        data.write_u32::<BigEndian>(65).unwrap();
        data.write_u32::<BigEndian>(65).unwrap();
        data.write_u32::<BigEndian>(1).unwrap();

        finish_cmap_length(&mut data);
        let cmap = TrueTypeCMap::from_font_data(&data).unwrap();
        assert_eq!(cmap.get_unicode(1), Some('A'));
    }
}
