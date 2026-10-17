//! Type 1 font encoding parser.
//!
//! Parses the built-in encoding table from Type 1 font programs (FontFile)
//! to extract character code → glyph name mappings.
//!
//! Type 1 fonts store their encoding in the clear text (ASCII) section as:
//! ```text
//! /Encoding 256 array
//! 0 1 255 {1 index exch /.notdef put} for
//! dup 11 /ff put
//! dup 12 /fi put
//! ...
//! readonly def
//! ```
//!
//! Per PDF spec §9.6.6.2: when no /BaseEncoding is specified, the implicit
//! base encoding for an embedded font is the font program's built-in encoding.

use std::collections::HashMap;

/// Parse the encoding from a Type 1 font program.
///
/// Scans the clear text section of the font program for `/Encoding` array
/// definitions and extracts `dup CODE /GLYPHNAME put` entries.
///
/// Returns a HashMap mapping character codes (u8) to Unicode characters,
/// or None if no custom encoding is found.
pub fn parse_type1_encoding(font_data: &[u8]) -> Option<HashMap<u8, char>> {
    // Find the /Encoding keyword in the clear text section.
    // The clear text section ends at "currentfile eexec" or at binary data.
    // We search within the first portion of the file (clear text is typically <10KB).
    let search_limit = font_data.len().min(65536);
    let search_data = &font_data[..search_limit];

    let encoding_pos = find_bytes(search_data, b"/Encoding")?;

    // Check if it's "/Encoding StandardEncoding def" — if so, return None
    // to let the default StandardEncoding handling work.
    let after_encoding = &search_data[encoding_pos + 9..];
    let trimmed = skip_whitespace(after_encoding);
    if trimmed.starts_with(b"StandardEncoding") {
        log::debug!("Type 1 font uses StandardEncoding (predefined)");
        return None;
    }

    // Scan for "dup CODE /GLYPHNAME put" patterns starting from /Encoding
    let mut encoding_map = HashMap::new();
    let mut pos = encoding_pos;

    while pos < search_limit {
        // Find next "dup" keyword
        let remaining = &search_data[pos..];
        let dup_offset = match find_bytes(remaining, b"dup") {
            Some(off) => off,
            None => break,
        };
        pos += dup_offset + 3;

        // Check if we've hit "readonly" or "def" which ends the encoding block
        let before_dup = &search_data[pos - 3..pos.min(search_limit)];
        if before_dup == b"def" {
            break;
        }

        // Parse: whitespace CODE whitespace /GLYPHNAME whitespace put
        let remaining = &search_data[pos..search_limit.min(search_data.len())];
        if let Some((code, glyph_name, consumed)) = parse_dup_entry(remaining) {
            if let Some(unicode_char) = super::font_dict::glyph_name_to_unicode(&glyph_name) {
                encoding_map.insert(code, unicode_char);
            }
            pos += consumed;
        }

        // Stop if we hit "readonly" or "def" (end of encoding array)
        let remaining = &search_data[pos..search_limit.min(search_data.len())];
        let trimmed = skip_whitespace(remaining);
        if trimmed.starts_with(b"readonly") || trimmed.starts_with(b"def") {
            break;
        }
    }

    if encoding_map.is_empty() {
        None
    } else {
        log::debug!("Type 1 built-in encoding parsed: {} character mappings", encoding_map.len());
        Some(encoding_map)
    }
}

/// Parse a single "CODE /GLYPHNAME put" entry after "dup".
/// Returns (code, glyph_name, bytes_consumed) or None.
fn parse_dup_entry(data: &[u8]) -> Option<(u8, String, usize)> {
    let mut pos = 0;

    // Skip whitespace
    while pos < data.len() && is_whitespace(data[pos]) {
        pos += 1;
    }

    // Parse integer code
    let code_start = pos;
    while pos < data.len() && data[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos == code_start {
        return None;
    }
    let code_str = std::str::from_utf8(&data[code_start..pos]).ok()?;
    let code: u16 = code_str.parse().ok()?;
    if code > 255 {
        return None;
    }

    // Skip whitespace
    while pos < data.len() && is_whitespace(data[pos]) {
        pos += 1;
    }

    // Expect '/' followed by glyph name
    if pos >= data.len() || data[pos] != b'/' {
        return None;
    }
    pos += 1;

    // Parse glyph name (alphanumeric + dots + underscores)
    let name_start = pos;
    while pos < data.len() && is_glyph_name_char(data[pos]) {
        pos += 1;
    }
    if pos == name_start {
        return None;
    }
    let glyph_name = std::str::from_utf8(&data[name_start..pos]).ok()?;

    // Skip whitespace and expect "put"
    while pos < data.len() && is_whitespace(data[pos]) {
        pos += 1;
    }
    if !data[pos..].starts_with(b"put") {
        return None;
    }
    pos += 3;

    Some((code as u8, glyph_name.to_string(), pos))
}

/// Find a byte sequence in data, returning its starting offset.
fn find_bytes(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).position(|w| w == needle)
}

/// Skip whitespace bytes, returning the remaining slice.
fn skip_whitespace(data: &[u8]) -> &[u8] {
    let mut pos = 0;
    while pos < data.len() && is_whitespace(data[pos]) {
        pos += 1;
    }
    &data[pos..]
}

fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

fn is_glyph_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'.' || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type1_encoding_basic() {
        let font_data = b"/Encoding 256 array\n\
            0 1 255 {1 index exch /.notdef put} for\n\
            dup 65 /A put\n\
            dup 66 /B put\n\
            dup 97 /a put\n\
            readonly def\n";

        let map = parse_type1_encoding(font_data).unwrap();
        assert_eq!(map.get(&65), Some(&'A'));
        assert_eq!(map.get(&66), Some(&'B'));
        assert_eq!(map.get(&97), Some(&'a'));
    }

    #[test]
    fn test_parse_type1_encoding_ligatures() {
        let font_data = b"/Encoding 256 array\n\
            0 1 255 {1 index exch /.notdef put} for\n\
            dup 11 /ff put\n\
            dup 12 /fi put\n\
            dup 14 /ffi put\n\
            readonly def\n";

        let map = parse_type1_encoding(font_data).unwrap();
        assert_eq!(map.get(&11), Some(&'\u{FB00}')); // ff ligature
        assert_eq!(map.get(&12), Some(&'\u{FB01}')); // fi ligature
        assert_eq!(map.get(&14), Some(&'\u{FB03}')); // ffi ligature
    }

    #[test]
    fn test_parse_type1_encoding_standard() {
        let font_data = b"/Encoding StandardEncoding def\n";
        assert!(parse_type1_encoding(font_data).is_none());
    }

    #[test]
    fn test_parse_type1_encoding_empty() {
        let font_data = b"no encoding here";
        assert!(parse_type1_encoding(font_data).is_none());
    }

    #[test]
    fn test_parse_type1_cmr_style_encoding() {
        // Simulate CMR-style Type1 font with ligatures and standard characters
        let font_data = b"%!PS-AdobeFont-1.0: CMR9 003.002\n\
            /Encoding 256 array\n\
            0 1 255 {1 index exch /.notdef put} for\n\
            dup 11 /ff put\n\
            dup 12 /fi put\n\
            dup 13 /fl put\n\
            dup 14 /ffi put\n\
            dup 15 /ffl put\n\
            dup 65 /A put\n\
            dup 97 /a put\n\
            dup 48 /zero put\n\
            dup 58 /colon put\n\
            dup 123 /endash put\n\
            readonly def\n\
            currentdict end\n\
            currentfile eexec\n";

        let map = parse_type1_encoding(font_data).unwrap();
        assert_eq!(map.len(), 10);
        assert_eq!(map.get(&11), Some(&'\u{FB00}')); // ff ligature
        assert_eq!(map.get(&12), Some(&'\u{FB01}')); // fi ligature
        assert_eq!(map.get(&13), Some(&'\u{FB02}')); // fl ligature
        assert_eq!(map.get(&14), Some(&'\u{FB03}')); // ffi ligature
        assert_eq!(map.get(&15), Some(&'\u{FB04}')); // ffl ligature
        assert_eq!(map.get(&65), Some(&'A'));
        assert_eq!(map.get(&97), Some(&'a'));
        assert_eq!(map.get(&48), Some(&'0'));
        assert_eq!(map.get(&58), Some(&':'));
        assert_eq!(map.get(&123), Some(&'\u{2013}')); // endash
    }
}
