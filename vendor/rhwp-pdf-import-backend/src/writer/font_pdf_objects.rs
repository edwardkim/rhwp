//! Build the five PDF indirect objects required to embed a TrueType font.
//!
//! For Unicode-capable PDF text, ISO 32000-1 §9.6.4 / §9.7 / §9.8 / §9.10
//! requires a graph of five objects per font:
//!
//! ```text
//!   Type 0 dict          (the "outer" Font referenced by Resources/Font/Fxx)
//!     ├── DescendantFonts → CIDFontType2 dict
//!     │                       └── FontDescriptor → FontFile2 stream
//!     └── ToUnicode      → CMap stream (glyph id → source codepoint)
//! ```
//!
//! All glyph indexing in the content stream is done through the Type 0 dict
//! using Identity-H encoding, which is just "two bytes per glyph, big endian,
//! no remapping — the CID *is* the GID".
//!
//! v0.3.38 ships **real font subsetting** (FONT-3b): `FontFile2` carries the
//! output of [`crate::fonts::subset_font_bytes`] (only the glyphs actually
//! used by this document, typically ~1% of a full CJK face), `/W` widths
//! and the `ToUnicode` CMap are keyed by the subset's GIDs (compact 0..N),
//! and the content stream's embedded-text ops are rewritten with the same
//! subset GIDs at serialisation time by
//! [`crate::writer::content_stream::ContentStreamBuilder::build_with_remappers`].
//! The [`GlyphRemapper`] returned here is the single source of truth for
//! that renumbering, threaded from the font-object emission phase of
//! [`crate::writer::PdfWriter::finish`] into every page's content-stream
//! serialisation.

use crate::error::Result;
use crate::fonts::GlyphRemapper;
use crate::object::Object;
use crate::writer::font_manager::EmbeddedFont;
use crate::writer::object_serializer::ObjectSerializer;
use std::collections::HashMap;

/// Object IDs for one embedded-font dict graph. Returned by
/// [`build_embedded_font_objects`] so the caller can link these into the
/// PdfWriter object table and reference the Type 0 dict from the page
/// `/Resources /Font` entry.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedFontIds {
    /// Top-level Type 0 (`/Font /Subtype /Type0`) dict ID. This is what
    /// the page resource dictionary references.
    pub type0: u32,
    /// Descendant CIDFontType2 dict ID.
    pub cidfont: u32,
    /// FontDescriptor dict ID.
    pub descriptor: u32,
    /// FontFile2 stream ID (the actual TrueType bytes).
    pub font_file: u32,
    /// ToUnicode CMap stream ID (round-trips text extraction).
    pub tounicode: u32,
}

/// Build the five PDF objects that embed `font` as a **subset** (only the
/// glyphs actually recorded via [`crate::fonts::FontSubsetter`] during
/// content-stream emission).
///
/// The caller is responsible for inserting the returned `(id, Object)`
/// pairs into the writer's object table and for emitting the Type 0 ref
/// into the page `/Resources /Font` dictionary under `resource_name`.
///
/// `id_alloc` is called once per object in dependency order
/// (font_file → descriptor → cidfont → tounicode → type0) so callers using
/// a monotonic ID counter end up with the natural traversal order.
///
/// Returns the [`GlyphRemapper`] produced by
/// [`crate::fonts::subset_font_bytes`] alongside the object graph — the
/// caller *must* keep this remapper and pass it to
/// [`crate::writer::content_stream::ContentStreamBuilder::build_with_remappers`]
/// for every page that references the font, or the hex GIDs emitted in the
/// content stream will not match the subset face's glyph indices.
pub fn build_embedded_font_objects(
    font: &mut EmbeddedFont,
    mut id_alloc: impl FnMut() -> u32,
) -> Result<(EmbeddedFontIds, Vec<(u32, Object)>, GlyphRemapper)> {
    let font_file_id = id_alloc();
    let descriptor_id = id_alloc();
    let cidfont_id = id_alloc();
    let tounicode_id = id_alloc();
    let type0_id = id_alloc();

    let ids = EmbeddedFontIds {
        type0: type0_id,
        cidfont: cidfont_id,
        descriptor: descriptor_id,
        font_file: font_file_id,
        tounicode: tounicode_id,
    };

    let mut out: Vec<(u32, Object)> = Vec::with_capacity(5);

    // Subset name like "ABCDEF+DejaVuSans". The 6-letter tag is generated
    // deterministically from the used-glyph set so the same content always
    // produces the same subset name (helps reproducible builds).
    let base_font = font.subset_name().to_string();

    // ── 1. Font file stream — subset TrueType or CFF bytes ───────────────
    // Run the Typst `subsetter` over the original face, keeping only the
    // glyphs this document actually references (plus GID 0 / .notdef,
    // which `subset_font_bytes` always adds). The returned `GlyphRemapper`
    // renumbers the kept glyphs to a dense 0..N range — we hand it back
    // to the caller so the content stream, `/W`, and `ToUnicode` all see
    // the same subset GIDs.
    //
    // CFF/OTF fonts (magic `OTTO`) must be embedded as FontFile3 with
    // subtype CIDFontType0C; TrueType fonts use FontFile2 with Length1.
    // Using the wrong key causes PDF readers to misparse the font data
    // (bug #449).
    //
    // Subsetting can legitimately fail on synthesised fonts — notably
    // CFF subsets patched with a Unicode cmap by fonts::cmap_injector.
    // The patched SFNT parses fine for glyph lookup, but the external
    // `subsetter` crate's CFF processor sometimes can't produce a valid
    // subset because the CFF charset wasn't refreshed to match the new
    // cmap. Fall back to emitting the unsubset font bytes — slightly
    // larger PDFs but the source typeface program reaches the viewer.
    let (font_bytes, remapper) =
        match crate::fonts::subset_font_bytes(font.font_data(), 0, font.used_glyphs()) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!(
                    "  [font] subsetting failed for {} ({} bytes); embedding full font: {e}",
                    font.name,
                    font.font_data().len()
                );
                let bytes = font.font_data().to_vec();
                // Empty GlyphRemapper: `remapper.get(orig)` returns None
                // for every GID, so the content-stream emitter's
                // `.unwrap_or(orig)` path emits the original GID
                // unchanged — exactly what we need when the font bytes
                // are the full unsubset face. NEW_FROM_GLYPHS sequence
                // would compact to 0,1,2... which would mismatch the
                // CFF charset positions.
                let remapper = subsetter::GlyphRemapper::new();
                (bytes, remapper)
            },
        };
    let is_cff = font_bytes.starts_with(b"OTTO");
    let byte_len = font_bytes.len() as i64;
    let mut ff_dict: HashMap<String, Object> = HashMap::new();
    ff_dict.insert("Length".to_string(), ObjectSerializer::integer(byte_len));
    if is_cff {
        // PDF spec §9.9 Table 126: FontFile3 streams carry /Subtype.
        // CIDFontType0C signals CFF-based CID font data.
        ff_dict.insert("Subtype".to_string(), ObjectSerializer::name("CIDFontType0C"));
    } else {
        // Length1 is required for FontFile2 (TrueType) per PDF spec §9.9.
        ff_dict.insert("Length1".to_string(), ObjectSerializer::integer(byte_len));
    }
    out.push((
        font_file_id,
        Object::Stream {
            dict: ff_dict,
            data: crate::import_limits::StreamData::from(font_bytes),
        },
    ));

    // ── 2. FontDescriptor (§9.8.1, Table 122) ────────────────────────────
    // CFF fonts reference the stream via FontFile3; TrueType via FontFile2.
    let font_file_key = if is_cff { "FontFile3" } else { "FontFile2" };
    let (llx, lly, urx, ury) = font.bbox;
    let descriptor = ObjectSerializer::dict(vec![
        ("Type", ObjectSerializer::name("FontDescriptor")),
        ("FontName", ObjectSerializer::name(&base_font)),
        ("Flags", ObjectSerializer::integer(font.flags as i64)),
        (
            "FontBBox",
            ObjectSerializer::rect(llx as f64, lly as f64, urx as f64, ury as f64),
        ),
        ("ItalicAngle", Object::Real(font.italic_angle as f64)),
        ("Ascent", ObjectSerializer::integer(font.ascender as i64)),
        ("Descent", ObjectSerializer::integer(font.descender as i64)),
        ("CapHeight", ObjectSerializer::integer(font.cap_height as i64)),
        ("XHeight", ObjectSerializer::integer(font.x_height as i64)),
        ("StemV", ObjectSerializer::integer(font.stem_v as i64)),
        (font_file_key, ObjectSerializer::reference(font_file_id, 0)),
    ]);
    out.push((descriptor_id, descriptor));

    // ── 3. CIDFont dict (§9.7.4, Table 117) ──────────────────────────────
    // CFF fonts → CIDFontType0; TrueType → CIDFontType2 + CIDToGIDMap.
    // The subsetter always creates an identity GID→CID mapping so the
    // content stream GIDs are also the CIDs for both font kinds.
    // CIDToGIDMap is specific to CIDFontType2 (ISO 32000-1 §9.7.4.2) and
    // must not appear for CIDFontType0 (bug #449).
    let widths_str = font.generate_widths_array(&remapper);
    let cid_system_info = ObjectSerializer::dict(vec![
        ("Registry", ObjectSerializer::string("Adobe")),
        ("Ordering", ObjectSerializer::string("Identity")),
        ("Supplement", ObjectSerializer::integer(0)),
    ]);
    let cidfont_subtype = if is_cff {
        "CIDFontType0"
    } else {
        "CIDFontType2"
    };
    let mut cidfont_entries: Vec<(&str, Object)> = vec![
        ("Type", ObjectSerializer::name("Font")),
        ("Subtype", ObjectSerializer::name(cidfont_subtype)),
        ("BaseFont", ObjectSerializer::name(&base_font)),
        ("CIDSystemInfo", cid_system_info),
        ("FontDescriptor", ObjectSerializer::reference(descriptor_id, 0)),
        // /W carries glyph widths indexed by CID/GID.
        ("W", parse_widths_string_to_array(&widths_str)),
    ];
    if !is_cff {
        // CIDToGIDMap /Identity: source GIDs are the CIDs (TrueType only).
        cidfont_entries.insert(5, ("CIDToGIDMap", ObjectSerializer::name("Identity")));
    }
    let cidfont = ObjectSerializer::dict(cidfont_entries);
    out.push((cidfont_id, cidfont));

    // ── 4. ToUnicode CMap stream (§9.10.2) ───────────────────────────────
    // Generated by EmbeddedFont from the tracked-glyph set. This is the
    // round-trip path: PDF readers parse this CMap to recover source text
    // from glyph IDs, which is what every conformance check (and our own
    // extract_text) walks.
    let cmap_bytes = font.generate_tounicode_cmap(&remapper).into_bytes();
    let mut cmap_dict: HashMap<String, Object> = HashMap::new();
    cmap_dict.insert("Length".to_string(), ObjectSerializer::integer(cmap_bytes.len() as i64));
    out.push((
        tounicode_id,
        Object::Stream {
            dict: cmap_dict,
            data: crate::import_limits::StreamData::from(cmap_bytes),
        },
    ));

    // ── 5. Type 0 wrapper (§9.6.4, Table 110) ────────────────────────────
    let type0 = ObjectSerializer::dict(vec![
        ("Type", ObjectSerializer::name("Font")),
        ("Subtype", ObjectSerializer::name("Type0")),
        ("BaseFont", ObjectSerializer::name(&base_font)),
        ("Encoding", ObjectSerializer::name("Identity-H")),
        (
            "DescendantFonts",
            Object::Array(vec![ObjectSerializer::reference(cidfont_id, 0)]),
        ),
        ("ToUnicode", ObjectSerializer::reference(tounicode_id, 0)),
    ]);
    out.push((type0_id, type0));

    Ok((ids, out, remapper))
}

/// Convert the pre-formatted widths string produced by
/// [`EmbeddedFont::generate_widths_array`] (a textual PDF array literal
/// like `"[ 36 [ 600 600 ] 65 [ 720 ] ]"`) into a structural
/// [`Object::Array`] so the existing `ObjectSerializer` can serialise it
/// without a raw-string escape hatch.
///
/// This is a tiny one-pass parser — accepts integers and `[`/`]` delimiters,
/// rejects anything else. It exists only so `EmbeddedFont`'s existing
/// helpers can stay as `String`-returning APIs (their original v0.3.0
/// callers were inspector code, not the writer).
fn parse_widths_string_to_array(s: &str) -> Object {
    let mut stack: Vec<Vec<Object>> = vec![Vec::new()];
    let mut number = String::new();
    let flush_number = |stack: &mut Vec<Vec<Object>>, number: &mut String| {
        if !number.is_empty() {
            if let Ok(n) = number.parse::<i64>() {
                stack
                    .last_mut()
                    .expect("widths-array stack must never empty")
                    .push(ObjectSerializer::integer(n));
            }
            number.clear();
        }
    };
    for ch in s.chars() {
        match ch {
            '[' => {
                flush_number(&mut stack, &mut number);
                stack.push(Vec::new());
            },
            ']' => {
                flush_number(&mut stack, &mut number);
                let popped = stack.pop().unwrap_or_default();
                stack
                    .last_mut()
                    .expect("widths-array stack must keep at least the root level")
                    .push(Object::Array(popped));
            },
            c if c.is_ascii_digit() || c == '-' => number.push(c),
            _ => flush_number(&mut stack, &mut number),
        }
    }
    flush_number(&mut stack, &mut number);
    // The outer stack holds [[ <root array> ]]; the root array is what we want.
    let mut root = stack.pop().unwrap_or_default();
    if root.len() == 1 {
        root.pop().unwrap_or(Object::Array(Vec::new()))
    } else {
        Object::Array(root)
    }
}

/// Public alias so callers can reference an embedded font's PDF identity
/// without depending on the private dict-builder details.
pub type FontResourceName = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_widths_simple() {
        let parsed = parse_widths_string_to_array("[ 36 [ 600 600 ] 65 [ 720 ] ]");
        // Outer is an array of: integer 36, inner-array [600,600], integer 65, inner [720].
        match parsed {
            Object::Array(items) => assert_eq!(items.len(), 4),
            other => panic!("expected array, got {other:?}"),
        }
    }

    #[test]
    fn parse_widths_empty() {
        let parsed = parse_widths_string_to_array("[]");
        match parsed {
            Object::Array(items) => assert!(items.is_empty()),
            other => panic!("expected empty array, got {other:?}"),
        }
    }
}
