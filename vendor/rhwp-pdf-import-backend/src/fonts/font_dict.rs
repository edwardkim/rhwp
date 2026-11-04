//! Font dictionary parsing.
//!
//! This module handles parsing of PDF font dictionaries and encoding information.
//! Fonts in PDF can have various encodings, and the ToUnicode CMap provides the
//! most accurate character-to-Unicode mapping.

use super::adobe_glyph_list::ADOBE_GLYPH_LIST;
use super::allocation::FontAllocation;
use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::fonts::cmap::LazyCMap;
use crate::fonts::TrueTypeCMap;
use crate::layout::text_block::FontWeight;
use crate::object::Object;
use std::collections::HashMap;
use std::sync::Arc;

/// Name-derived Standard-14 classification of a font, resolved once and
/// memoized (see [`FontInfo::std14_memo`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Std14Flags {
    /// Font is one of the Times family.
    pub is_times: bool,
    /// Font is one of the Courier (monospace) family.
    pub is_courier: bool,
    /// Font name carries a Bold marker.
    pub is_bold: bool,
    /// Font name carries a BoldItalic marker.
    pub is_bold_italic: bool,
    /// Font is one of the Helvetica family.
    pub is_helvetica: bool,
    /// Font name carries an Italic marker.
    pub is_italic: bool,
}

/// Font information extracted from a PDF font dictionary.
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Base font name (e.g., "Times-Roman", "Helvetica-Bold")
    pub base_font: String,
    /// Font subtype (e.g., "Type1", "TrueType", "Type0")
    pub subtype: String,
    /// Every CharProc is proven to contain metrics only; searchable text paints no glyph.
    pub type3_metrics_only: bool,
    /// Encoding information
    pub encoding: Encoding,
    /// ToUnicode CMap (character code to Unicode mapping)
    /// Lazily parsed on first character lookup for improved performance
    pub to_unicode: Option<LazyCMap>,
    /// Font weight from FontDescriptor (400 = normal, 700 = bold)
    pub font_weight: Option<i32>,
    /// Font descriptor flags (bit field)
    /// Bit 1: FixedPitch, Bit 2: Serif, Bit 3: Symbolic, Bit 4: Script,
    /// Bit 6: Nonsymbolic, Bit 7: Italic
    /// PDF Spec: ISO 32000-1:2008, Table 5.20
    pub flags: Option<i32>,
    /// Stem thickness (vertical) from FontDescriptor (used for weight inference)
    /// PDF Spec: ISO 32000-1:2008, Section 9.6.2
    /// Typical values: <80 = light, 80-110 = normal/medium, >110 = bold
    pub stem_v: Option<f32>,
    /// Ascent above the baseline (fraction of em, from FontDescriptor /Ascent).
    /// Converted from PDF's 1/1000-em units to a fraction of em (raw value ÷ 1000).
    /// Defaults to 0.95 when the font descriptor is absent (matching Poppler's fallback).
    pub ascent: f32,
    /// Descent below the baseline (fraction of em, from FontDescriptor /Descent).
    /// Converted from PDF's 1/1000-em units to a fraction of em; always ≤ 0.
    /// Defaults to -0.35 when the font descriptor is absent (matching Poppler's fallback).
    pub descent: f32,
    /// Embedded TrueType font data (from FontFile2 stream)
    /// Shared via Arc to avoid expensive cloning
    pub embedded_font_data: Option<Arc<Vec<u8>>>,
    /// Descriptor program declaration for a simple Type1 font. `None` means
    /// unknown/unsupported descriptor, not absent. A declared but unreadable
    /// FontFile is still `Some(true)`; callers must not mask a decoding failure.
    pub embedded_program_present: Option<bool>,
    /// Lazily-extracted TrueType cmap table (GID to Unicode mappings).
    /// Used as fallback when ToUnicode CMap is missing.
    /// Initialized on first access via `truetype_cmap()` accessor to avoid
    /// the 10-25ms per-font extraction cost when ToUnicode resolves all chars.
    pub truetype_cmap: std::sync::OnceLock<Option<TrueTypeCMap>>,
    /// Originating import request, retained for lazy font program parsing.
    /// None for legacy, unbounded documents; never shared across requests.
    pub import_budget: Option<Arc<crate::import_limits::ImportBudget>>,
    /// Lazily-extracted embedded TrueType/CFF `post`-table glyph names,
    /// indexed by GID. `None` element = no name for that GID (post format 3,
    /// or the glyph name table is absent). Used by §9.10.2 Priority 3c
    /// fallback in `decode_char_to_unicode`: when `truetype_cmap.get_unicode`
    /// misses, we try this glyph name via `glyph_name_to_unicode` (AGL +
    /// `uniXXXX`/`uXXXXX` synth) before falling through to the hardcoded
    /// `gid_to_standard_glyph_name` ASCII map and CID-as-Unicode last
    /// resort. Resolves `•` → `❍` substitution and `fi`/`fl` ligature
    /// corruption on Identity-H subset fonts without `CIDToGIDMap`.
    ///
    pub embedded_glyph_names: std::sync::OnceLock<Option<Vec<Option<String>>>>,
    /// Whether this font has an embedded TrueType font (FontFile2).
    /// Controls whether lazy truetype_cmap extraction is attempted.
    pub is_truetype_font: bool,
    /// CID to GID mapping (Type0 fonts only, Phase 3)
    /// Converts Character IDs in the PDF to Glyph IDs in the embedded font
    /// Used to look up Unicode values via the TrueType cmap table
    /// Phase 3: Enables CFF/OpenType support via CIDToGIDMap parsing
    pub cid_to_gid_map: Option<CIDToGIDMap>,
    /// CIDFont character collection info (Type0 fonts only)
    /// Identifies the character set (e.g., Adobe-Japan1, Adobe-GB1)
    pub cid_system_info: Option<CIDSystemInfo>,
    /// CIDFont subtype ("CIDFontType0" for CFF, "CIDFontType2" for TrueType)
    pub cid_font_type: Option<String>,
    /// `FontMatrix[a]` element — scales glyph-space widths to text-space units.
    /// Standard Type1/TrueType: 0.001 (widths in 1/1000 em).
    /// Type3 with `FontMatrix [1 0 0 1 0 0]`: 1.0 (widths already in text-space units).
    /// `advance_in_text_space = width × font_matrix_a × font_size`
    pub font_matrix_a: f32,
    /// Character widths in 1000ths of em (PDF units)
    /// For simple fonts (Type1, TrueType): array indexed by (char_code - first_char)
    /// PDF Spec: ISO 32000-1:2008, Section 9.7.4
    pub widths: Option<Vec<f32>>,
    /// First character code covered by widths array
    /// Used to map character codes to width array indices
    pub first_char: Option<u32>,
    /// Last character code covered by widths array
    pub last_char: Option<u32>,
    /// Default width for characters not in widths array (in 1000ths of em)
    /// Typical values: 500-600 for proportional fonts, 600 for monospace
    pub default_width: f32,
    /// CID to width mapping for Type0 (CIDFont) fonts
    /// Per PDF Spec ISO 32000-1:2008, Section 9.7.4.3
    /// Widths in 1000ths of em. Uses HashMap for sparse CID distributions.
    pub cid_widths: Option<HashMap<u16, f32>>,
    /// Default width for CIDs not in cid_widths (Type0 fonts only)
    /// Per PDF Spec: default is 1000 if /DW not specified
    pub cid_default_width: f32,
    /// Whether /DW was explicitly present in the CIDFont dictionary.
    /// Used by has_explicit_widths() and get_glyph_width() to distinguish
    /// a spec-default 1000 from an authored 1000 (F14/F15 fix).
    pub has_explicit_dw: bool,
    /// Multi-character encoding map for compound glyph names (e.g. f_f → "ff")
    /// Stores mappings from character code to multi-char strings
    pub multi_char_map: HashMap<u8, String>,
    /// CFF byte_code → glyph_id mapping for embedded CFF subset fonts.
    /// Allows direct glyph rendering without Unicode cmap.
    pub cff_gid_map: Option<HashMap<u8, u16>>,
    /// CID → GID mapping read from a **CID-keyed** CFF's charset.
    ///
    /// ISO 32000-1:2008 §9.7.4.2 (`docs/spec/pdf.md`:18641-18644): when the
    /// embedded CFF's Top DICT uses CIDFont operators, "the CIDs shall be used
    /// to determine the GID value for the glyph procedure using the charset
    /// table in the CFF program" — and the NOTE at :18646 warns the two "may
    /// differ". `None` means the font is not CID-keyed, in which case the same
    /// clause (:18649-18650) says the CIDs are the GIDs.
    pub cff_cid_to_gid: Option<HashMap<u16, u16>>,
    /// Pre-computed byte→char lookup for simple (non-Type0) fonts.
    /// Index by byte value (0-255). '\0' means "use full char_to_unicode fallback".
    /// Built lazily on first text decode. Avoids per-byte HashMap lookups.
    pub byte_to_char_table: std::sync::OnceLock<[char; 256]>,
    /// Per-font memo of `char_to_unicode`. Type0/CID fonts have no
    /// `byte_to_char_table`, so without this each glyph re-runs the decode
    /// cascade. `Arc<Mutex<…>>` keeps `FontInfo: Clone` (clones share the memo).
    pub type0_unicode_memo:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<u32, Option<String>>>>,
    /// Pre-computed byte→width lookup for simple (non-Type0) fonts.
    /// Index by byte value (0-255). Built lazily on first advance_position call.
    /// Eliminates per-byte bounds check and subtraction in get_glyph_width.
    pub byte_to_width_table: std::sync::OnceLock<[f32; 256]>,
    /// Memo of [`FontInfo::get_font_weight`]. The name-based fallback lowercases
    /// `base_font` and runs a dozen substring searches; text extraction asks for
    /// the weight once per glyph, where the answer is loop-invariant.
    pub weight_memo: std::sync::OnceLock<FontWeight>,
    /// Memo of [`FontInfo::is_italic`] — same per-glyph hot path as `weight_memo`.
    pub italic_memo: std::sync::OnceLock<bool>,
    /// Memo of the Standard-14 name classification. `get_standard_font_width`
    /// is called per glyph and otherwise re-strips the subset prefix and
    /// re-scans a 15-name table every time.
    pub std14_memo: std::sync::OnceLock<Option<Std14Flags>>,
    /// Raw `/Differences` glyph names retained by character code (simple fonts).
    /// Populated alongside the `Encoding::Custom` map during `parse_encoding`,
    /// but unlike the Custom map (which stores the *resolved* char) this keeps the
    /// authoritative glyph *name* the writer assigned via the encoding dictionary's
    /// `/Differences` array (ISO 32000-1 §9.6.6.1, Table 114). Used by
    /// `glyph_name_for_code` to recover punctuation (`period`/`comma`/`hyphen`/
    /// `minus`) when an upstream decode yields a non-sensible symbol — see the
    /// glyph-name-gated interceptions in `char_to_unicode`.
    pub diff_glyph_names: HashMap<u8, String>,
    /// Writing mode resolved from this font's encoding and (when available)
    /// from the embedded CMap stream's `/WMode` directive.
    ///
    /// - `0` (default): horizontal writing — glyph advance along x-axis.
    /// - `1`: vertical writing (tategaki) — glyph advance along y-axis with
    ///   per-CID vertical-origin offset applied per glyph.
    ///
    /// Resolution rules (highest precedence first):
    /// 1. The embedded CMap stream's `/WMode` directive when one is parsed
    ///    (via `LazyCMap::wmode()` on the encoding's CMap).
    /// 2. Predefined PDF CMap name ending in `-V` (Identity-V, UniJIS-UTF16-V,
    ///    UniGB-UTF16-V, UniCNS-UTF16-V, UniKS-UTF16-V) or the bare legacy
    ///    `V`. The original encoding name is retained even when the
    ///    `Encoding` enum collapses `Identity-H`/`Identity-V` into
    ///    `Encoding::Identity`.
    /// 3. Otherwise `0`.
    pub wmode: u8,
    /// Per-CID vertical-writing metrics parsed from the CIDFont's `/W2`
    /// array (ISO 32000-1 §9.7.4.3). `None` for horizontal-only fonts so
    /// they pay no allocation/hash-lookup cost.
    pub cid_vertical_metrics: Option<HashMap<u16, VerticalMetrics>>,
    /// Default vertical metrics for CIDs not covered by `cid_vertical_metrics`.
    /// Parsed from `/DW2` (defaults to [`VerticalMetrics::SPEC_DEFAULT`] when
    /// `/DW2` is absent). Held by value because the struct is `Copy`.
    pub cid_default_vertical_metrics: VerticalMetrics,
    /// `Some(collection)` when this is a Type0 CIDFont referencing one of
    /// Adobe's predefined CJK base names (`Ryumin-Light`, `GothicBBB-Medium`,
    /// `STSong-Light`, `MHei-Medium`, `HYSMyeongJo-Medium`, …), has no
    /// embedded font program (no `/FontFile{,2,3}` key on either the Type0
    /// wrapper's or the CIDFont descendant's descriptor), AND uses an
    /// Identity charcode→CID `/Encoding` (Identity-H/V or an
    /// Adobe-collection identity CMap stream). ISO 32000-2 §9.7.5.2 requires
    /// a conforming reader to supply glyphs for these character collections;
    /// the renderer consults this field to route the paint through a bundled
    /// covering font (see [`super::predefined_cidfont`]) and convert each CID
    /// through the appropriate [`super::cid_mappings`] table to a Unicode
    /// code point. The collection follows the descendant's `/CIDSystemInfo`
    /// Ordering when it names a known collection, falling back to the
    /// name-derived collection for Identity/unknown orderings.
    ///
    /// `None` for every other font, including:
    /// - Type0 fonts whose CIDFont declares an embedded program — even when
    ///   that program fails to load/decode, substitution would mask the
    ///   decode defect; the failure is logged instead;
    /// - Type0 fonts with a non-Identity predefined CMap (`90ms-RKSJ-H`,
    ///   `GBK-EUC-H`, …) whose charcodes are raw legacy multi-byte values,
    ///   not CIDs — unsupported until a charcode→CID CMap pass is wired;
    /// - Type0 fonts whose base name is not in the predefined registry (we
    ///   cannot safely guess a substitution);
    /// - Simple Type1 / TrueType fonts.
    pub cjk_substitution: Option<super::predefined_cidfont::CharacterCollection>,
}

/// Font encoding types.
#[derive(Debug, Clone)]
pub enum Encoding {
    /// Standard PDF encoding (WinAnsiEncoding, MacRomanEncoding, etc.)
    Standard(String),
    /// Custom encoding with explicit character mappings
    Custom(HashMap<u8, char>),
    /// Identity encoding (typically used for CID fonts)
    Identity,
}

/// CID to GID mapping for Type 2 CIDFonts (TrueType-based)
/// Per PDF Spec ISO 32000-1:2008, Section 9.7.4.2
///
/// This mapping converts Character IDs (CIDs) in the PDF document to Glyph IDs (GIDs)
/// in the embedded TrueType font, which can then be mapped to Unicode via the cmap table.
#[derive(Debug, Clone)]
pub enum CIDToGIDMap {
    /// Identity mapping: CID == GID (default, most common)
    /// Used when each character ID directly corresponds to a glyph ID
    Identity,

    /// Explicit mapping: CID → GID via uint16 stream
    /// Stream format: GID at bytes [2*CID, 2*CID+1], big-endian
    /// Used for non-standard glyph ID assignments
    Explicit(Vec<u16>),
}

impl CIDToGIDMap {
    /// Convert a Character ID (CID) to a Glyph ID (GID) using this mapping.
    ///
    /// Per PDF Spec ISO 32000-1:2008, Section 9.7.4.2:
    /// - Identity mapping: CID == GID (most common, default)
    /// - Explicit mapping: Use uint16 array lookup
    ///
    /// # Arguments
    ///
    /// * `cid` - The Character ID from the PDF document
    ///
    /// # Returns
    ///
    /// The corresponding Glyph ID in the embedded font
    pub fn get_gid(&self, cid: u16) -> u16 {
        match self {
            CIDToGIDMap::Identity => cid,
            CIDToGIDMap::Explicit(gid_array) => {
                if (cid as usize) < gid_array.len() {
                    gid_array[cid as usize]
                } else {
                    // Out of range - fall back to identity mapping
                    cid
                }
            },
        }
    }
}

/// CIDFont character collection identifier
/// Per PDF Spec ISO 32000-1:2008, Section 9.7.4.2
///
/// Identifies which character encoding the CIDFont uses, such as:
/// - Adobe-Japan1: Japanese text
/// - Adobe-GB1: Simplified Chinese
/// - Adobe-CNS1: Traditional Chinese
/// - Adobe-Korea1: Korean
#[derive(Debug, Clone)]
pub struct CIDSystemInfo {
    /// Registry name (typically "Adobe")
    pub registry: String,

    /// Ordering string (e.g., "Japan1", "GB1", "CNS1", "Korea1")
    pub ordering: String,

    /// Supplement number (version of the character collection)
    pub supplement: i32,
}

/// Per-CID vertical-writing metrics from a CIDFont's `/W2` array.
///
/// Per ISO 32000-1:2008 §9.7.4.3 and the Adobe CMap & CIDFont Files
/// Specification §9.7. In vertical writing mode the glyph advances along the
/// y-axis (not the x-axis) and is shifted from its default horizontal origin
/// to a vertical origin so that the glyph stacks correctly within a column.
///
/// All values are in 1000ths-of-em (glyph-space units), matching the
/// convention used throughout PDF font dictionaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerticalMetrics {
    /// `w1y`: vertical displacement (advance) of the glyph along the y-axis.
    ///
    /// Typically negative (around `-1000` for a full-em CJK glyph) because PDF
    /// user space has y increasing upward, while vertical text advances
    /// downward. The text matrix is translated by `w1y * font_size / 1000`
    /// after the glyph is painted.
    pub w1y: f32,

    /// `v_x`: x-component of the vector from the default (horizontal) origin
    /// to the vertical origin, in 1000ths-of-em.
    ///
    /// Spec default `500` (half-em) places the vertical origin at the glyph's
    /// horizontal center, which is correct for monospaced full-width CJK
    /// glyphs.
    pub v_x: f32,

    /// `v_y`: y-component of the vertical-origin offset, in 1000ths-of-em.
    ///
    /// Spec default `880` places the vertical origin near the top of the em.
    pub v_y: f32,
}

impl VerticalMetrics {
    /// Spec default per ISO 32000-1 §9.7.4.3: vertical origin at
    /// `(500, 880)` and glyph displacement `-1000` (one full em downward).
    pub const SPEC_DEFAULT: VerticalMetrics = VerticalMetrics {
        w1y: -1000.0,
        v_x: 500.0,
        v_y: 880.0,
    };
}

/// Decide writing mode from a predefined PDF CMap name.
///
/// Per ISO 32000-1 §9.7.5.2 (Table 118) and the Adobe CMap & CIDFont Files
/// Specification, predefined CMap names whose suffix is `-V` (e.g.
/// `Identity-V`, `UniJIS-UTF16-V`, `UniGB-UTF16-V`, `UniCNS-UTF16-V`,
/// `UniKS-UTF16-V`, `GBK-EUC-V`, `90ms-RKSJ-V`, …) and the bare legacy `V`
/// declare vertical writing (`/WMode 1`). Every other name implies
/// horizontal writing (`/WMode 0`).
///
/// This function is the canonical name-to-wmode decision used by both
/// `FontInfo::resolve_encoding_writing_mode` and the encoding-name fallback
/// inside `FontInfo::from_dict`.
pub(crate) fn wmode_from_predefined_cmap_name(name: &str) -> u8 {
    if name == "V" || name.ends_with("-V") {
        1
    } else {
        0
    }
}

impl FontInfo {
    /// Get the TrueType cmap, lazily extracting it on first access.
    /// Returns `None` if the font is not TrueType or has no embedded data.
    pub fn truetype_cmap(&self) -> Option<&TrueTypeCMap> {
        if let Some(budget) = &self.import_budget {
            budget.check().ok()?;
        }
        self.truetype_cmap
            .get_or_init(|| {
                if !self.is_truetype_font {
                    return None;
                }
                let font_data = self.embedded_font_data.as_ref()?;
                if font_data.is_empty() {
                    return None;
                }
                match TrueTypeCMap::from_font_data_with_budget(
                    font_data,
                    self.import_budget.as_deref(),
                ) {
                    Ok(cmap) if !cmap.is_empty() => {
                        log::info!(
                            "Lazy-extracted TrueType cmap for font '{}': {} mappings",
                            self.base_font,
                            cmap.len()
                        );
                        Some(cmap)
                    },
                    Ok(_) => None,
                    Err(e) => {
                        log::warn!(
                            "Font '{}': TrueType cmap extraction failed: {}",
                            self.base_font,
                            e
                        );
                        None
                    },
                }
            })
            .as_ref()
    }

    /// Set the TrueType cmap directly (used by share_truetype_cmaps and tests).
    pub fn set_truetype_cmap(&mut self, cmap: Option<TrueTypeCMap>) {
        self.truetype_cmap = std::sync::OnceLock::new();
        if let Some(c) = cmap {
            let _ = self.truetype_cmap.set(Some(c));
        } else {
            let _ = self.truetype_cmap.set(None);
        }
    }

    /// Check if a TrueType cmap is available (either already extracted or extractable).
    pub fn has_truetype_cmap(&self) -> bool {
        self.truetype_cmap().is_some()
    }

    /// The most authoritative Unicode-mapping resource this font offers, as a
    /// [`MappingProvenance`](crate::fonts::MappingProvenance).
    ///
    /// This is a **fact** derived from the font's structure — which mapping
    /// resources exist — not a decode of any particular character code. It
    /// mirrors the ISO 32000-1 §9.10.2 priority order and covers every font
    /// type, so it is complete where a font-type-specific structural check is
    /// not.
    ///
    /// [`Fallback`](crate::fonts::MappingProvenance::Fallback) is the important
    /// value: it means the font carries **no** mapping resource — no usable
    /// `/ToUnicode`, no predefined CID→Unicode collection, no embedded `cmap`,
    /// and no simple-font encoding — so any Unicode extracted for its glyphs is
    /// a fabricated echo, not read from the file (§9.10.2: "there is no way to
    /// determine what the character code represents"). Callers compose their own
    /// policy from this (route to OCR, flag the page, keep the raw echo).
    pub fn best_mapping_provenance(&self) -> crate::fonts::MappingProvenance {
        use crate::fonts::MappingProvenance as P;
        // 1. A present, non-empty /ToUnicode CMap is authoritative (§9.10.2).
        if self
            .to_unicode
            .as_ref()
            .and_then(|c| c.get())
            .is_some_and(|m| !m.is_empty())
        {
            return P::ToUnicode;
        }
        // 2. A predefined CID→Unicode collection: a Type0 font whose descendant
        //    uses a known, non-Identity ordering (Adobe-GB1/CNS1/Japan1/Korea1).
        if self.subtype == "Type0" {
            if let Some(info) = &self.cid_system_info {
                if info.ordering != "Identity" && !info.ordering.is_empty() {
                    return P::PredefinedCMap;
                }
            }
        }
        // 3. The embedded program's own cmap (recoverable byte-as-GID / Identity
        //    subsets that kept a usable cmap).
        if self.has_truetype_cmap() {
            return P::EmbeddedCmap;
        }
        // 4. A simple font resolves through its /Encoding → glyph name → AGL, and
        //    symbolic Symbol/ZapfDingbats through their built-in encodings.
        if self.subtype != "Type0" {
            return P::EncodingName;
        }
        // 5. A Type0 font with none of the above severs every path to Unicode.
        P::Fallback
    }

    /// Look up the embedded font program's `post`-table glyph name for the
    /// given GID.
    ///
    /// Lazily parses the embedded TrueType/OpenType font (via `ttf-parser`)
    /// on first access, then caches a `Vec<Option<String>>` indexed by GID
    /// for O(1) subsequent lookups. The parsed font's `Face::glyph_name`
    /// abstracts over TrueType `post` Format 2 names and CFF `charset` SIDs,
    /// so this works for both TrueType (FontFile2) and CFF / Type1C
    /// (FontFile3) subset fonts.
    ///
    /// Returns `None` when:
    /// - the font has no embedded program (`embedded_font_data == None`),
    /// - the font program is empty or fails to parse,
    /// - the `post` table is Format 3 (no names) or the GID is out of range,
    /// - the parsed name is `.notdef` (which AGL doesn't map and isn't
    ///   useful as text anyway).
    ///
    /// Used by §9.10.2 Priority 3c in `decode_char_to_unicode`.
    pub(crate) fn embedded_glyph_name(&self, gid: u16) -> Option<&str> {
        let allocation = FontAllocation(self.import_budget.as_deref());
        allocation.check()?;
        let names = self
            .embedded_glyph_names
            .get_or_init(|| {
                let font_data = self.embedded_font_data.as_ref()?;
                if font_data.is_empty() {
                    return None;
                }
                let face = match ttf_parser::Face::parse(font_data, 0) {
                    Ok(f) => f,
                    Err(e) => {
                        log::debug!(
                            "Font '{}': ttf-parser Face::parse failed for glyph-name extraction: {:?}",
                            self.base_font,
                            e
                        );
                        return None;
                    },
                };
                let n = face.number_of_glyphs();
                // `number_of_glyphs` returns u16; cap the vec at that size.
                let mut out: Vec<Option<String>> = allocation.vec(n as usize)?;
                let mut found_any = false;
                for g in 0..n {
                    let raw_name = face
                        .glyph_name(ttf_parser::GlyphId(g))
                        .filter(|s| !s.is_empty() && *s != ".notdef")
                        ;
                    let name = match raw_name { Some(name) => Some(allocation.string(name)?), None => None };
                    if name.is_some() {
                        found_any = true;
                    }
                    out.push(name);
                }
                if !found_any {
                    log::debug!(
                        "Font '{}': embedded program has no usable glyph names (post Format 3 or stripped)",
                        self.base_font
                    );
                    return None;
                }
                log::info!(
                    "Font '{}': cached {} embedded glyph names (post/charset) for §9.10.2 Priority 3c fallback",
                    self.base_font,
                    out.iter().filter(|n| n.is_some()).count(),
                );
                Some(out)
            })
            .as_ref()?;
        allocation.check()?;
        names.get(gid as usize).and_then(|n| n.as_deref())
    }

    /// Authoritative glyph name for a *simple* font character code, in priority
    /// order (ISO 32000-1 §9.6.6.1 / §9.10.2):
    /// (a) the `/Differences` glyph name retained in `diff_glyph_names`;
    /// (b) else the embedded post/charset glyph name for the code's GID
    ///     (`embedded_glyph_name`), when the embedded program carries names.
    ///
    /// Used by the Item 1 punctuation-recovery interceptions in `char_to_unicode`.
    fn glyph_name_for_code(&self, char_code: u32) -> Option<&str> {
        if let Some(name) = self.diff_glyph_names.get(&(char_code as u8)) {
            return Some(name.as_str());
        }
        // Fall back to the embedded program's glyph name for this code's GID.
        // For embedded CFF subsets the byte_code → GID map is authoritative;
        // otherwise treat the code as the GID (TrueType simple-font convention).
        let gid = self
            .cff_gid_map
            .as_ref()
            .and_then(|m| m.get(&(char_code as u8)).copied())
            .unwrap_or(char_code as u16);
        self.embedded_glyph_name(gid)
    }

    /// Parse font information from a font dictionary object.
    ///
    /// # Arguments
    ///
    /// * `dict` - The font dictionary object (should be a Dictionary or Stream)
    /// * `doc` - The PDF document (needed to load referenced objects)
    ///
    /// # Returns
    ///
    /// A FontInfo struct containing the parsed font information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The object is not a dictionary
    /// - Required font dictionary entries are missing or invalid
    /// - Referenced objects cannot be loaded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pdf_oxide::document::PdfDocument;
    /// use pdf_oxide::fonts::FontInfo;
    /// use pdf_oxide::object::ObjectRef;
    ///
    /// # fn example(doc: PdfDocument, font_ref: ObjectRef) -> Result<(), Box<dyn std::error::Error>> {
    /// let font_obj = doc.load_object(font_ref)?;
    /// let font_info = FontInfo::from_dict(&font_obj, &doc)?;
    /// println!("Font: {}", font_info.base_font);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_dict(dict: &Object, doc: &PdfDocument) -> Result<Self> {
        doc.check_import_budget()?;
        let result = Self::from_dict_impl(dict, doc);
        doc.check_import_budget()?;
        result
    }

    fn from_dict_impl(dict: &Object, doc: &PdfDocument) -> Result<Self> {
        let font_dict = dict.as_dict().ok_or_else(|| Error::ParseError {
            offset: 0,
            reason: "Font object is not a dictionary".to_string(),
        })?;

        let allocation = FontAllocation(doc.import_budget().map(AsRef::as_ref));
        let allocation_error = || Error::ImportResourceLimit("font metadata allocation");
        // Extract BaseFont (required)
        let base_font = allocation
            .string(
                font_dict
                    .get("BaseFont")
                    .and_then(|obj| obj.as_name())
                    .unwrap_or("Unknown"),
            )
            .ok_or_else(allocation_error)?;

        // Extract Subtype (required)
        let subtype = allocation
            .string(
                font_dict
                    .get("Subtype")
                    .and_then(|obj| obj.as_name())
                    .unwrap_or("Unknown"),
            )
            .ok_or_else(allocation_error)?;

        // Log Type 3 fonts - may require special glyph name mapping
        if subtype == "Type3" {
            let msg =
                format!("Font '{}' is Type 3 - may require special glyph name mapping", base_font);
            log::warn!("{}", msg);
            // push into the structured warning
            // sink. PDF Spec §9.6.4 "Type 3 Fonts" describes the
            // user-defined CharProcs glyph-program model; the
            // standard glyph name registry doesn't apply, so
            // extraction may fall back to glyph-name heuristics.
            // Straight to THIS document's sink. The free-function sink is
            // keyed by thread, not by document, so two documents parsed in
            // sequence on one thread take each other's warnings. `from_dict`
            // already holds `doc`, so no signature has to change — and it must
            // not, because this is a `pub fn` and the crate ships bindings for
            // fourteen languages.
            doc.push_structured_warning(crate::extractors::warnings::Warning {
                category: crate::extractors::warnings::WarningCategory::Type3Font,
                page: None,
                message: msg,
                spec_section: Some("9.6.4"),
            });
        }

        // Parse FontMatrix [a] for Type 3 fonts.
        // Standard Type 1 FontMatrix is [0.001 0 0 0.001 0 0], so widths are in 1/1000 em.
        // Type 3 fonts can use an identity FontMatrix [1 0 0 1 0 0], meaning widths are
        // in text-space units directly (no 1/1000 scaling needed).
        let font_matrix_a = if subtype == "Type3" {
            font_dict
                .get("FontMatrix")
                .and_then(|obj| obj.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| {
                    v.as_real()
                        .map(|r| r as f32)
                        .or_else(|| v.as_integer().map(|i| i as f32))
                })
                // A degenerate FontMatrix[0] — zero, near-zero, or non-finite —
                // is a malformed horizontal scale (ISO 32000-1 §9.2.4 / §9.6.5)
                // and would make the `default_width * 0.001 / font_matrix_a`
                // rescale below divide by ~0 → inf/NaN, and the
                // `font_size * font_matrix_a` advance collapse to 0. Reject it
                // and fall back to the standard 0.001 (Type 1) scale.
                .filter(|a| a.is_finite() && a.abs() > 1e-6)
                .unwrap_or(0.001)
        } else {
            0.001
        };

        // Parse FontDescriptor FIRST to get font flags (needed for encoding decision)
        // PDF Spec: ISO 32000-1:2008, Section 9.6.2 - Font Descriptor
        let (
            font_weight,
            flags,
            stem_v,
            mut embedded_font_data,
            is_truetype_font,
            raw_ascent,
            raw_descent,
            mut has_font_program,
        ) = if let Some(descriptor_ref) = font_dict
            .get("FontDescriptor")
            .and_then(|obj| obj.as_reference())
        {
            // Load the FontDescriptor object
            if let Ok(descriptor_obj) = doc.load_object(descriptor_ref) {
                if let Some(descriptor_dict) = descriptor_obj.as_dict() {
                    let weight = descriptor_dict
                        .get("FontWeight")
                        .and_then(|weight_obj| weight_obj.as_integer())
                        .map(|w| w as i32);

                    let descriptor_flags = descriptor_dict
                        .get("Flags")
                        .and_then(|flags_obj| flags_obj.as_integer())
                        .map(|f| f as i32);

                    let stem_v_value = descriptor_dict.get("StemV").and_then(|sv_obj| {
                        sv_obj
                            .as_real()
                            .map(|r| r as f32)
                            .or_else(|| sv_obj.as_integer().map(|i| i as f32))
                    });

                    let ascent_value = descriptor_dict.get("Ascent").and_then(|obj| {
                        obj.as_real()
                            .map(|r| r as f32)
                            .or_else(|| obj.as_integer().map(|i| i as f32))
                    });

                    let descent_value = descriptor_dict.get("Descent").and_then(|obj| {
                        obj.as_real()
                            .map(|r| r as f32)
                            .or_else(|| obj.as_integer().map(|i| i as f32))
                    });

                    // Load embedded font data from FontFile2 (TrueType), FontFile (Type 1), or FontFile3 (CFF/OpenType)
                    // IMPORTANT: Track whether font is TrueType or CFF - only TrueType fonts have cmaps!
                    //
                    // Key presence is recorded separately from extraction
                    // success: a present-but-undecodable font program means
                    // the document intended to be self-contained, which
                    // downstream gates (CJK predefined-CIDFont substitution)
                    // must distinguish from "no program at all".
                    let has_font_program = descriptor_dict.contains_key("FontFile2")
                        || descriptor_dict.contains_key("FontFile3")
                        || descriptor_dict.contains_key("FontFile");
                    let (embedded_font, is_truetype_font) = if let Some(ff2_obj) =
                        descriptor_dict.get("FontFile2")
                    {
                        log::info!("Font '{}' has FontFile2 entry (TrueType)", base_font);
                        let font_data = ff2_obj
                            .as_reference()
                            .and_then(|ff2_ref| {
                                doc.load_object(ff2_ref)
                                    .inspect_err(|e| {
                                        log::warn!(
                                            "Font '{}': failed to load FontFile2 object {}: {}",
                                            base_font,
                                            ff2_ref,
                                            e
                                        )
                                    })
                                    .ok()
                                    .map(|obj| (obj, ff2_ref))
                            })
                            .and_then(|(ff2_stream, ff2_ref)| {
                                doc.decode_stream_with_encryption(&ff2_stream, ff2_ref)
                                    .inspect_err(|e| {
                                        log::warn!(
                                            "Font '{}': failed to decode FontFile2 stream {}: {}",
                                            base_font,
                                            ff2_ref,
                                            e
                                        )
                                    })
                                    .ok()
                            })
                            .map(|data| {
                                log::info!(
                                    "Font '{}' loaded embedded TrueType font ({} bytes)",
                                    base_font,
                                    data.len()
                                );
                                Arc::new(data)
                            });
                        (font_data, true) // TrueType - can have cmaps
                    } else if let Some(ff3_obj) = descriptor_dict.get("FontFile3") {
                        log::info!(
                            "Font '{}' has FontFile3 entry (CFF/OpenType - no TrueType cmap)",
                            base_font
                        );
                        let font_data = ff3_obj
                            .as_reference()
                            .and_then(|ff3_ref| {
                                doc.load_object(ff3_ref)
                                    .inspect_err(|e| {
                                        log::warn!(
                                            "Font '{}': failed to load FontFile3 object {}: {}",
                                            base_font,
                                            ff3_ref,
                                            e
                                        )
                                    })
                                    .ok()
                                    .map(|obj| (obj, ff3_ref))
                            })
                            .and_then(|(ff3_stream, ff3_ref)| {
                                doc.decode_stream_with_encryption(&ff3_stream, ff3_ref)
                                    .inspect_err(|e| {
                                        log::warn!(
                                            "Font '{}': failed to decode FontFile3 stream {}: {}",
                                            base_font,
                                            ff3_ref,
                                            e
                                        )
                                    })
                                    .ok()
                            })
                            .map(|data| {
                                // Wrap raw CFF in OpenType container for ttf-parser
                                let data = if !data.is_empty() && data[0] == 1 && data.len() > 4 {
                                    log::info!(
                                        "Font '{}': Wrapping raw CFF in OpenType ({} bytes)",
                                        base_font,
                                        data.len()
                                    );
                                    wrap_cff_in_opentype_with_budget(
                                        &data,
                                        doc.import_budget().map(AsRef::as_ref),
                                    )
                                } else {
                                    log::info!(
                                        "Font '{}' loaded embedded CFF/OpenType font ({} bytes)",
                                        base_font,
                                        data.len()
                                    );
                                    data
                                };
                                Arc::new(data)
                            });
                        (font_data, false) // CFF - no TrueType cmap
                    } else if let Some(ff_obj) = descriptor_dict.get("FontFile") {
                        log::info!("Font '{}' has FontFile entry (Type 1)", base_font);
                        let font_data = ff_obj
                            .as_reference()
                            .and_then(|ff_ref| {
                                doc.load_object(ff_ref)
                                    .inspect_err(|e| {
                                        log::warn!(
                                            "Font '{}': failed to load FontFile object {}: {}",
                                            base_font,
                                            ff_ref,
                                            e
                                        )
                                    })
                                    .ok()
                                    .map(|obj| (obj, ff_ref))
                            })
                            .and_then(|(ff_stream, ff_ref)| {
                                doc.decode_stream_with_encryption(&ff_stream, ff_ref)
                                    .inspect_err(|e| {
                                        log::warn!(
                                            "Font '{}': failed to decode FontFile stream {}: {}",
                                            base_font,
                                            ff_ref,
                                            e
                                        )
                                    })
                                    .ok()
                            })
                            .map(|data| {
                                log::info!(
                                    "Font '{}' loaded embedded Type 1 font ({} bytes)",
                                    base_font,
                                    data.len()
                                );
                                Arc::new(data)
                            });
                        (font_data, false) // Type 1 - no TrueType cmap
                    } else {
                        log::debug!("Font '{}' has no embedded font data", base_font);
                        (None, false)
                    };

                    (
                        weight,
                        descriptor_flags,
                        stem_v_value,
                        embedded_font,
                        is_truetype_font,
                        ascent_value,
                        descent_value,
                        has_font_program,
                    )
                } else {
                    (None, None, None, None, false, None, None, false)
                }
            } else {
                (None, None, None, None, false, None, None, false)
            }
        } else {
            (None, None, None, None, false, None, None, false)
        };

        // TrueType cmap extraction is now LAZY — deferred until first access via
        // truetype_cmap() accessor. This saves 10-25ms per font when ToUnicode CMap
        // (Priority 1) resolves all characters, making the cmap unnecessary.
        // The is_truetype_font flag is recorded here for the lazy accessor to use.

        // Helper function to check if font is symbolic (bit 3 set)
        let is_symbolic_font = |flags_opt: Option<i32>| -> bool {
            if let Some(flags_value) = flags_opt {
                const SYMBOLIC_BIT: i32 = 1 << 2; // Bit 3
                (flags_value & SYMBOLIC_BIT) != 0
            } else {
                // Fallback: check font name
                let name_lower = base_font.to_lowercase();
                name_lower.contains("symbol")
                    || name_lower.contains("zapf")
                    || name_lower.contains("dingbat")
            }
        };

        // Parse encoding (now that we have flags)
        // PDF Spec: ISO 32000-1:2008, Section 9.6.6.1
        // "For symbolic fonts, the Encoding entry is ignored"
        //
        // However, many PDF generators (LaTeX, LibreOffice, etc.) incorrectly set the
        // Symbolic flag on non-symbolic fonts. When an explicit /Encoding entry exists,
        // we always parse it — real-world PDF viewers (MuPDF, poppler, pdf.js) do the same.
        // The Symbolic flag only controls behavior when NO /Encoding is present.
        // Pre-parse font program encoding (needed for /Differences base encoding per PDF spec)
        let font_program_enc_cache: Option<HashMap<u8, char>> =
            if let Some(font_data) = &embedded_font_data {
                if subtype == "Type1" || subtype == "MMType1" {
                    super::type1_encoding::parse_type1_encoding_with_budget(
                        font_data,
                        doc.import_budget().map(AsRef::as_ref),
                    )
                } else {
                    super::cff_encoding::parse_cff_encoding_with_budget(
                        font_data,
                        doc.import_budget().map(AsRef::as_ref),
                    )
                }
            } else {
                None
            };

        // Writing-mode signal sourced from the encoding object. Resolved
        // here because the `Encoding` enum collapses `Identity-H` and
        // `Identity-V` to the same `Encoding::Identity` variant — we need
        // the original name to recover wmode. Defaults to `0` (horizontal)
        // when no encoding object is present.
        let mut encoding_wmode: u8 = 0;
        let (encoding, diff_multi_char_map, diff_glyph_names) = if let Some(enc_obj) =
            font_dict.get("Encoding")
        {
            let resolved_enc_obj = if let Some(obj_ref) = enc_obj.as_reference() {
                std::borrow::Cow::Owned(doc.load_object(obj_ref)?)
            } else {
                std::borrow::Cow::Borrowed(enc_obj)
            };

            // Inspect for `-V` predefined name or embedded `/WMode 1 def`
            // before parse_encoding flattens the variant.
            let (_enc_name, wm) = Self::resolve_encoding_writing_mode(&resolved_enc_obj, doc);
            encoding_wmode = wm;

            if is_symbolic_font(flags) {
                log::debug!(
                    "Font '{}' is symbolic (Flags={:?}) but has /Encoding — parsing it anyway (common in LaTeX/LibreOffice PDFs)",
                    base_font,
                    flags
                );
            } else {
                log::debug!("Font '{}' using /Encoding entry", base_font);
            }
            let (mut parsed_enc, mut multi_map, glyph_names) =
                Self::parse_encoding(&resolved_enc_obj, doc, font_program_enc_cache.as_ref())?;

            // When /Encoding is a named encoding (e.g., /WinAnsiEncoding) AND the font
            // has an embedded program, merge the font program's encoding. This handles
            // fonts where the program maps glyphs to non-standard code positions
            // (e.g., space at 0xCA) that the named encoding maps differently.
            // The font program's mappings override the standard encoding.
            if matches!(parsed_enc, Encoding::Standard(_)) {
                if let Some(prog_enc) = &font_program_enc_cache {
                    let std_name = match &parsed_enc {
                        Encoding::Standard(n) => {
                            allocation.string(n).ok_or_else(allocation_error)?
                        },
                        _ => allocation
                            .string("StandardEncoding")
                            .ok_or_else(allocation_error)?,
                    };

                    // Decide whether the embedded program's built-in encoding is a
                    // meaningful text encoding (a few non-standard slots to overlay,
                    // e.g. space at 0xCA) or a re-indexed *cipher* — a subset font's
                    // own glyph ordering that bears no relation to the producer's
                    // declared named base encoding. Overlaying a cipher rewrites every
                    // mapped code into mojibake. Discriminate by agreement: count how
                    // many program codes resolve to the SAME character the named base
                    // already gives. A real encoding agrees on most; a cipher on
                    // almost none.
                    let looks_like_cipher = builtin_encoding_looks_like_cipher_with_budget(
                        prog_enc, &std_name, allocation,
                    );

                    if looks_like_cipher {
                        // Trust the producer-declared named encoding; the built-in
                        // cipher would corrupt it. Leave `parsed_enc` as the named
                        // Standard encoding.
                        log::debug!(
                            "Font '{base_font}': built-in encoding disagrees with {std_name} on most overlapping codes — treating as a subset cipher and keeping the named encoding"
                        );
                    } else {
                        log::info!(
                            "Font '{}': merging {} font program encoding entries with {}",
                            base_font,
                            prog_enc.len(),
                            std_name,
                        );
                        // Build Custom map: start with the named encoding, overlay the
                        // (consistent) font program for its few non-standard slots.
                        let mut custom_map: HashMap<u8, char> =
                            allocation.map(0).ok_or_else(allocation_error)?;
                        for code in 0u8..=255 {
                            if let Some(unicode_str) =
                                standard_encoding_lookup_with_budget(&std_name, code, allocation)
                            {
                                if let Some(ch) = unicode_str.chars().next() {
                                    allocation
                                        .insert(&mut custom_map, code, ch)
                                        .ok_or_else(allocation_error)?;
                                }
                            }
                        }
                        for (&code, &ch) in prog_enc {
                            allocation
                                .insert(&mut custom_map, code, ch)
                                .ok_or_else(allocation_error)?;
                            if is_ligature_char(ch) {
                                if let Some(expanded) = expand_ligature_char(ch) {
                                    let text =
                                        allocation.string(expanded).ok_or_else(allocation_error)?;
                                    allocation
                                        .insert(&mut multi_map, code, text)
                                        .ok_or_else(allocation_error)?;
                                }
                            }
                        }
                        parsed_enc = Encoding::Custom(custom_map);
                    }
                }
            }

            (parsed_enc, multi_map, glyph_names)
        } else {
            // No /Encoding entry — use font program's built-in encoding if available
            if let Some(prog_enc) = font_program_enc_cache {
                log::info!(
                    "Font '{}' using built-in font program encoding ({} mappings)",
                    base_font,
                    prog_enc.len()
                );
                let mut multi_map: HashMap<u8, String> = HashMap::new();
                for (&code, &ch) in &prog_enc {
                    if is_ligature_char(ch) {
                        if let Some(expanded) = expand_ligature_char(ch) {
                            let text = allocation.string(expanded).ok_or_else(allocation_error)?;
                            allocation
                                .insert(&mut multi_map, code, text)
                                .ok_or_else(allocation_error)?;
                        }
                    }
                }
                (Encoding::Custom(prog_enc), multi_map, HashMap::new())
            } else if is_symbolic_font(flags) {
                log::debug!(
                    "Font '{}' is symbolic with no /Encoding - will use built-in encoding (Symbol/ZapfDingbats)",
                    base_font
                );
                (
                    Encoding::Standard(
                        allocation
                            .string("SymbolicBuiltIn")
                            .ok_or_else(allocation_error)?,
                    ),
                    HashMap::new(),
                    HashMap::new(),
                )
            } else {
                log::debug!(
                    "Font '{}' has no /Encoding entry - defaulting to StandardEncoding",
                    base_font
                );
                (
                    Encoding::Standard(
                        allocation
                            .string("StandardEncoding")
                            .ok_or_else(allocation_error)?,
                    ),
                    HashMap::new(),
                    HashMap::new(),
                )
            }
        };

        // Parse ToUnicode CMap if present (Phase 5.1: Lazy Loading)
        // The CMap stream is stored raw and parsed only on first character lookup
        let to_unicode = if let Some(cmap_ref) = font_dict
            .get("ToUnicode")
            .and_then(|obj| obj.as_reference())
        {
            let stream_opt = match doc.load_object(cmap_ref) {
                Ok(cmap_obj) => {
                    match doc.decode_stream_with_encryption(&cmap_obj, cmap_ref) {
                        Ok(data) => Some(data),
                        Err(e) => {
                            log::warn!(
                                "Font '{}': Failed to decrypt/decode ToUnicode CMap stream {:?}: {}",
                                base_font, cmap_ref, e
                            );
                            None
                        },
                    }
                },
                Err(e) => {
                    log::warn!(
                        "Font '{}': Failed to load ToUnicode CMap object {:?}: {}",
                        base_font,
                        cmap_ref,
                        e
                    );
                    None
                },
            };

            if let Some(stream_bytes) = stream_opt {
                // Store raw bytes for lazy parsing — LazyCMap handles errors on first access.
                // Skipping eager validation avoids parsing every CMap twice.
                log::info!(
                    "ToUnicode CMap stream loaded for font '{}': {} bytes (lazy parsing enabled)",
                    base_font,
                    stream_bytes.len()
                );
                Some(match doc.import_budget() {
                    Some(budget) => LazyCMap::new_with_budget(stream_bytes, Arc::clone(budget))?,
                    None => LazyCMap::new(stream_bytes),
                })
            } else {
                // Specific error already logged above in the match arms
                None
            }
        } else {
            if subtype == "Type0" {
                let msg = format!("Type0 font '{}' has no ToUnicode entry!", base_font);
                log::warn!("{}", msg);
                // push to the structured sink. PDF
                // Spec §9.10.2 "ToUnicode CMaps" describes the
                // mapping; absent ToUnicode triggers the fallback
                // chain (Encoding → AGL → CID-as-Unicode) per §9.10.3.
                // Same reasoning as the Type 3 site above: the document is
                // already in scope, so the warning is attributed to it rather
                // than to whichever thread happened to parse it.
                doc.push_structured_warning(crate::extractors::warnings::Warning {
                    category: crate::extractors::warnings::WarningCategory::ToUnicodeMissing,
                    page: None,
                    message: msg,
                    spec_section: Some("9.10.2"),
                });
            }
            None
        };

        // Parse /Widths array for glyph width information
        // PDF Spec: ISO 32000-1:2008, Section 9.7.4 - Font Widths
        //
        // For simple fonts (Type1, TrueType), widths are specified as an array
        // of integers in 1000ths of em, indexed from FirstChar to LastChar.
        //
        // Note: Type0 (CID) fonts use a different /W array format, parsed via parse_descendant_fonts below
        let (widths, first_char, last_char) = if subtype != "Type0" {
            // Try to parse /Widths array
            let widths_opt = font_dict.get("Widths").and_then(|widths_obj| {
                // Handle both direct arrays and references
                let resolved = if let Some(ref_obj) = widths_obj.as_reference() {
                    doc.load_object(ref_obj).ok()?
                } else {
                    crate::object_copy::clone_object(
                        widths_obj,
                        doc.import_budget().map(AsRef::as_ref),
                    )
                    .ok()?
                };

                resolved.as_array().and_then(|arr| {
                    let mut widths = allocation.vec(arr.len())?;
                    widths.extend(arr.iter().filter_map(|obj| {
                        // Widths can be integers or reals
                        obj.as_integer()
                            .map(|i| i as f32)
                            .or_else(|| obj.as_real().map(|r| r as f32))
                    }));
                    Some(widths)
                })
            });

            let first = font_dict
                .get("FirstChar")
                .and_then(|obj| obj.as_integer())
                .map(|i| i as u32);

            let last = font_dict
                .get("LastChar")
                .and_then(|obj| obj.as_integer())
                .map(|i| i as u32);

            if widths_opt.is_some() {
                log::debug!(
                    "Font '{}': parsed {} widths (FirstChar={:?}, LastChar={:?})",
                    base_font,
                    widths_opt.as_ref().map(|w| w.len()).unwrap_or(0),
                    first,
                    last
                );
            } else {
                log::debug!("Font '{}': no /Widths array found, will use default width", base_font);
            }

            (widths_opt, first, last)
        } else {
            // Type0 fonts use /W and /DW arrays parsed via parse_descendant_fonts
            log::debug!("Font '{}': Type0 font, widths parsed from CIDFont /W array", base_font);
            (None, None, None)
        };

        // Set default width based on font characteristics
        // PDF Spec: Typical values are 500-600 for proportional fonts, ~600 for monospace
        let default_width = if let Some(flags_val) = flags {
            const FIXED_PITCH_BIT: i32 = 1 << 0; // Bit 1
            if (flags_val & FIXED_PITCH_BIT) != 0 {
                600.0 // Monospace font
            } else {
                500.0 // Proportional font
            }
        } else {
            // No flags, use middle-ground default
            550.0
        };

        // The heuristic above is calibrated for standard fonts where font_matrix_a = 0.001
        // (i.e. glyph-space units are 1/1000 em).  Type3 fonts can use an arbitrary
        // FontMatrix; if font_matrix_a differs from 0.001, rescale so that callers
        // multiplying by font_matrix_a still get the intended em-fraction result.
        let default_width = if subtype == "Type3" && font_matrix_a != 0.001 {
            default_width * 0.001 / font_matrix_a
        } else {
            default_width
        };

        // Phase 3: Parse DescendantFonts for Type0 fonts
        let (
            cid_to_gid_map,
            cid_system_info,
            cid_font_type,
            cid_widths,
            cid_default_width,
            has_explicit_dw,
            descendant_tt_cmap,
            desc_raw_ascent,
            desc_raw_descent,
            cid_vertical_metrics,
            cid_default_vertical_metrics,
        ) = if subtype == "Type0" {
            match Self::parse_descendant_fonts(font_dict, &base_font, doc) {
                Ok((
                    map,
                    info,
                    ftype,
                    widths,
                    dw,
                    explicit_dw,
                    tt_cmap,
                    (desc_has_font_program, desc_embedded),
                    d_ascent,
                    d_descent,
                    vmetrics,
                    dvmetrics,
                )) => {
                    log::info!(
                            "Font '{}': Parsed DescendantFonts - CIDFontType={}, CIDSystemInfo={}-{}, widths={}, embedded={}",
                            base_font,
                            ftype.as_ref().unwrap_or(&"Unknown".to_string()),
                            info.as_ref()
                                .map(|s| s.registry.as_str())
                                .unwrap_or("Unknown"),
                            info.as_ref()
                                .map(|s| s.ordering.as_str())
                                .unwrap_or("Unknown"),
                            widths.as_ref().map(|m| m.len()).unwrap_or(0),
                            desc_embedded.is_some()
                        );
                    // Use embedded font data from CIDFont descendant if top-level didn't have it
                    if desc_embedded.is_some() && embedded_font_data.is_none() {
                        embedded_font_data = desc_embedded;
                    }
                    has_font_program |= desc_has_font_program;
                    (
                        map,
                        info,
                        ftype,
                        widths,
                        dw,
                        explicit_dw,
                        tt_cmap,
                        d_ascent,
                        d_descent,
                        vmetrics,
                        dvmetrics,
                    )
                },
                Err(e) => {
                    log::warn!(
                        "Font '{}': Failed to parse DescendantFonts: {}. Using Identity fallback.",
                        base_font,
                        e
                    );
                    (
                        Some(CIDToGIDMap::Identity),
                        None,
                        None,
                        None,
                        1000.0,
                        false,
                        None,
                        None,
                        None,
                        None,
                        VerticalMetrics::SPEC_DEFAULT,
                    )
                },
            }
        } else {
            (
                None,
                None,
                None,
                None,
                1000.0,
                false,
                None,
                None,
                None,
                None,
                VerticalMetrics::SPEC_DEFAULT,
            )
        };

        // For Type0 fonts the /FontDescriptor lives on the CIDFont descendant (§9.7.4).
        // If the top-level font had no descriptor (the common case), fall back to the
        // descendant's values so CID/CJK glyphs get real metrics instead of the 0.95/-0.35
        // Poppler-compatible default.
        let raw_ascent = raw_ascent.or(desc_raw_ascent);
        let raw_descent = raw_descent.or(desc_raw_descent);

        // Pre-populate OnceLock with descendant's TrueType cmap if available.
        // Otherwise leave it for lazy extraction from embedded_font_data.
        let truetype_cmap_lock = std::sync::OnceLock::new();
        if let Some(desc_cmap) = descendant_tt_cmap {
            let _ = truetype_cmap_lock.set(Some(desc_cmap));
        }

        // Parse CFF GID mapping ONLY for simple (non-Type0) fonts with embedded CFF data.
        // Type0/CID fonts use Identity-H encoding and CIDToGIDMap, not CFF Standard Encoding.
        //
        // §9.6.6: the byte → GID resolution must use the PDF font dictionary's
        // /Encoding as the byte → glyph-name source and the CFF Charset as the
        // glyph-name → GID resolver. Subsetter-emitted custom CFF Encoding
        // tables are frequently sparse (some prepress subsetters emit only
        // `space` and `A`) and would silently drop most content bytes to
        // `.notdef` without this routing.
        // A CID-keyed CFF carries its own CID -> GID table in the charset, and
        // §9.7.4.2 requires it to be used. Only built for Type0, since the
        // clause is about CIDFonts; `None` for everything else means "the CID
        // is the GID", which is the other half of the same clause.
        let cff_cid_to_gid = if subtype == "Type0" {
            embedded_font_data.as_ref().and_then(|data| {
                super::cff_encoding::parse_cff_cid_to_gid_with_budget(
                    data,
                    doc.import_budget().map(AsRef::as_ref),
                )
                .inspect(|map| {
                    log::debug!(
                        "Font '{}': CID-keyed CFF, {} CID->GID entries from the charset",
                        base_font,
                        map.len()
                    );
                })
            })
        } else {
            None
        };

        let cff_gid_map = if subtype != "Type0" {
            embedded_font_data.as_ref().and_then(|data| {
                super::cff_encoding::parse_cff_gid_mapping_with_pdf_encoding_with_budget(
                    data,
                    &encoding,
                    &diff_glyph_names,
                    doc.import_budget().map(AsRef::as_ref),
                )
                .inspect(|map| {
                    log::debug!(
                        "Font '{}': parsed CFF GID mapping via PDF /Encoding ({} entries)",
                        base_font,
                        map.len()
                    );
                })
            })
        } else {
            None
        };

        // Normalize ascent/descent from 1000ths-of-em to fraction-of-em.
        // PDF spec says these are in 1/1000 of em (glyph space units).
        // Fall back to standard font metrics for the 14 standard PDF fonts,
        // then to Poppler-compatible defaults (0.95 / -0.35).
        let (default_ascent, default_descent) =
            standard_font_metrics(&base_font).unwrap_or((0.95, -0.35));
        let ascent = raw_ascent.map(|v| v / 1000.0).unwrap_or(default_ascent);
        // PDF Descent should be ≤ 0 (below baseline). Some PDFs store it as a positive
        // magnitude; Poppler normalizes by negating. Mirror that here.
        let descent = raw_descent
            .map(|v| {
                let d = v / 1000.0;
                if d > 0.0 {
                    -d
                } else {
                    d
                }
            })
            .unwrap_or(default_descent);

        // Final writing-mode resolution.
        //
        // Per ISO 32000-1:2008 §9.10.2 the ToUnicode CMap is for
        // extraction-time character → Unicode mapping ONLY. The active
        // writing mode is determined by the /Encoding CMap (§9.7.5):
        // either an embedded `/WMode 1 def` directive or a predefined
        // encoding name whose suffix is `-V`. Consulting the ToUnicode
        // CMap's `/WMode` here would silently flip a horizontal document
        // to vertical whenever a producer left a stale `/WMode 1 def`
        // in the ToUnicode prologue — a real-world tooling failure mode.
        //
        // We still emit a debug log when ToUnicode disagrees with the
        // /Encoding so producer bugs are diagnosable.
        let wmode = encoding_wmode;
        if let Some(tu) = to_unicode.as_ref() {
            let tu_wmode = tu.wmode();
            if tu_wmode != encoding_wmode {
                log::debug!(
                    "Font '{}': ToUnicode CMap declares /WMode {} but /Encoding wmode is {}. \
                     Honoring /Encoding per ISO 32000-1 §9.10.2.",
                    base_font,
                    tu_wmode,
                    encoding_wmode
                );
            }
        }

        // Detect Adobe predefined CIDFont substitution candidates.
        // Conditions (all must hold):
        //   1. Type0 font (the only place predefined CMaps are referenced).
        //   2. No embedded font program — neither the Type0 wrapper's nor the
        //      CIDFont descendant's FontDescriptor carries a `/FontFile{,2,3}`
        //      KEY. Key presence (not extraction success) is what gates here:
        //      a present-but-undecodable program means the document embeds its
        //      own outlines and the decode failure must surface as a warning,
        //      not be masked by a silent sans-serif substitution.
        //   3. The /Encoding resolves to an Identity charcode→CID mapping
        //      (Identity-H/V or an Adobe-collection identity CMap stream).
        //      Non-Identity predefined CMaps (90ms-RKSJ-H, GBK-EUC-H, …) carry
        //      raw legacy multi-byte codes, not CIDs — substituting would
        //      index the CID→Unicode tables with Shift-JIS / EUC values and
        //      paint wrong glyphs. Those CMaps stay unsubstituted until a
        //      charcode→CID CMap pass is wired.
        //   4. The base font name (after subset-prefix + CMap-suffix strip)
        //      matches one of the registered predefined names from
        //      Technical Notes #5078 / #5079 / #5080 / #5093.
        // The character collection comes from the descendant's /CIDSystemInfo
        // Ordering when it names a known collection (it is authoritative for
        // CID semantics per ISO 32000-1 §9.7.3); the name-derived collection
        // is the fallback for Identity/unknown orderings.
        // When all hold, the renderer routes the paint through the bundled
        // covering font; otherwise we leave `cjk_substitution` at `None` and
        // the existing render path runs unchanged.
        let cjk_substitution = if subtype == "Type0"
            && !has_font_program
            && embedded_font_data.is_none()
            && matches!(encoding, Encoding::Identity)
        {
            use super::predefined_cidfont::CharacterCollection;
            let name_collection = super::predefined_cidfont::is_predefined(&base_font);
            let ordering_collection =
                cid_system_info
                    .as_ref()
                    .and_then(|info| match info.ordering.as_str() {
                        "Japan1" => Some(CharacterCollection::AdobeJapan1),
                        "GB1" => Some(CharacterCollection::AdobeGB1),
                        "CNS1" => Some(CharacterCollection::AdobeCNS1),
                        "Korea1" => Some(CharacterCollection::AdobeKorea1),
                        _ => None,
                    });
            let collection = match (name_collection, ordering_collection) {
                (Some(n), Some(o)) if n != o => {
                    log::info!(
                        "Font '{}': base name implies collection {:?} but \
                         /CIDSystemInfo Ordering says {:?}; trusting CIDSystemInfo",
                        base_font,
                        n,
                        o
                    );
                    Some(o)
                },
                (Some(n), _) => Some(n),
                (None, _) => None,
            };
            if let Some(c) = collection {
                log::info!(
                    "Font '{}': flagged for CJK predefined-CIDFont substitution \
                     (collection {:?}); no embedded outlines, base name is an \
                     Adobe predefined CIDFont per ISO 32000-2 §9.7.5.2",
                    base_font,
                    c
                );
            }
            collection
        } else {
            if subtype == "Type0" && super::predefined_cidfont::is_predefined(&base_font).is_some()
            {
                if has_font_program && embedded_font_data.is_none() {
                    log::warn!(
                        "Font '{}': /FontFile{{,2,3}} present but the font program \
                         failed to load/decode (see warnings above); NOT substituting \
                         the bundled CJK fallback — glyphs for this font will not render",
                        base_font
                    );
                } else if !has_font_program && !matches!(encoding, Encoding::Identity) {
                    log::info!(
                        "Font '{}': Adobe predefined CIDFont without embedded outlines, \
                         but /Encoding is a non-Identity CMap — charcodes are not CIDs, \
                         so CJK substitution is skipped",
                        base_font
                    );
                }
            }
            None
        };

        let embedded_program_present = if has_font_program {
            Some(true)
        } else if subtype == "Type1" {
            Self::descriptor_program_presence(font_dict.get("FontDescriptor"), doc)
        } else {
            None
        };

        let type3_metrics_only = subtype == "Type3" && super::type3_empty::proven_empty(dict, doc)?;
        Ok(FontInfo {
            base_font,
            subtype,
            type3_metrics_only,
            encoding,
            to_unicode,
            font_weight,
            flags,
            stem_v,
            ascent,
            descent,
            embedded_font_data,
            embedded_program_present,
            truetype_cmap: truetype_cmap_lock,
            import_budget: doc.import_budget().cloned(),
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font,
            cid_to_gid_map,
            cid_system_info,
            cid_font_type,
            font_matrix_a,
            widths,
            first_char,
            last_char,
            default_width,
            cid_widths,
            cid_default_width,
            has_explicit_dw,
            cff_gid_map,
            cff_cid_to_gid,
            multi_char_map: diff_multi_char_map,
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names,
            wmode,
            cid_vertical_metrics,
            cid_default_vertical_metrics,
            cjk_substitution,
        })
    }

    /// Parse encoding from an encoding object.
    ///
    /// Phase 3: Parse CIDSystemInfo from CIDFont dictionary
    /// Extracts Registry, Ordering, and Supplement for character collection identification
    /// Per PDF Spec ISO 32000-1:2008, Section 9.7.3
    fn parse_cidsysteminfo(
        cidfont_dict: &HashMap<String, Object>,
        doc: &PdfDocument,
    ) -> Result<CIDSystemInfo> {
        let sysinfo_obj = cidfont_dict
            .get("CIDSystemInfo")
            .ok_or_else(|| Error::ParseError {
                offset: 0,
                reason: "CIDFont missing required /CIDSystemInfo entry".to_string(),
            })?;

        // Resolve reference if needed
        let resolved = if let Some(ref_obj) = sysinfo_obj.as_reference() {
            std::borrow::Cow::Owned(doc.load_object(ref_obj)?)
        } else {
            std::borrow::Cow::Borrowed(sysinfo_obj)
        };

        let sysinfo_dict = resolved.as_dict().ok_or_else(|| Error::ParseError {
            offset: 0,
            reason: "CIDSystemInfo is not a dictionary".to_string(),
        })?;

        let allocation = FontAllocation(doc.import_budget().map(AsRef::as_ref));
        let name = |key: &str| -> Result<String> {
            let bytes = sysinfo_dict
                .get(key)
                .and_then(Object::as_string)
                .unwrap_or(b"Unknown");
            // Lossy UTF-8 can allocate up to three bytes per source byte before
            // the final owned name copy; reserve that temporary first.
            allocation
                .bytes(bytes.len().saturating_mul(3))
                .ok_or(Error::ImportResourceLimit("CID name decoding"))?;
            allocation
                .string(&String::from_utf8_lossy(bytes))
                .ok_or(Error::ImportResourceLimit("CID name copy"))
        };
        let registry = name("Registry")?;
        let ordering = name("Ordering")?;

        let supplement = sysinfo_dict
            .get("Supplement")
            .and_then(|obj| obj.as_integer())
            .unwrap_or(0) as i32;

        log::debug!(
            "CIDSystemInfo parsed: Registry={}, Ordering={}, Supplement={}",
            registry,
            ordering,
            supplement
        );

        Ok(CIDSystemInfo {
            registry,
            ordering,
            supplement,
        })
    }

    /// Phase 3: Parse DescendantFonts array for Type0 fonts
    /// Extracts CIDFont dictionary and related information
    /// Per PDF Spec ISO 32000-1:2008, Section 9.7.1
    ///
    /// Returns: (CIDToGIDMap, CIDSystemInfo, CIDFontType, CIDWidths, DefaultWidth,
    ///          has_explicit_dw, TrueTypeCMap, (has_font_program, EmbeddedFontData),
    ///          raw_ascent, raw_descent, vertical_metrics, dw2)
    ///
    /// The embedded-font element pairs descriptor `/FontFile{,2,3}` key
    /// presence with the extracted bytes so callers can tell "no program"
    /// apart from "program present but failed to load/decode".
    #[allow(clippy::type_complexity)] // tuple grew incrementally; refactor deferred to a follow-up
    fn parse_descendant_fonts(
        font_dict: &HashMap<String, Object>,
        base_font: &str,
        doc: &PdfDocument,
    ) -> Result<(
        Option<CIDToGIDMap>,
        Option<CIDSystemInfo>,
        Option<String>,
        Option<HashMap<u16, f32>>,
        f32,                                   // cid_default_width
        bool,                                  // has_explicit_dw (F14/F15 fix)
        Option<TrueTypeCMap>,                  // TrueType cmap from descendant's embedded font
        (bool, Option<Arc<Vec<u8>>>),          // (/FontFile{,2,3} key present, extracted data)
        Option<f32>,                           // raw_ascent from descendant FontDescriptor
        Option<f32>,                           // raw_descent from descendant FontDescriptor
        Option<HashMap<u16, VerticalMetrics>>, // /W2 per-CID vertical metrics
        VerticalMetrics,                       // /DW2 default vertical metrics (or spec defaults)
    )> {
        let descendant_obj = font_dict
            .get("DescendantFonts")
            .ok_or_else(|| Error::ParseError {
                offset: 0,
                reason: format!(
                    "Type0 font '{}' missing required /DescendantFonts entry",
                    base_font
                ),
            })?;

        // Resolve reference if needed
        let resolved = if let Some(ref_obj) = descendant_obj.as_reference() {
            std::borrow::Cow::Owned(doc.load_object(ref_obj)?)
        } else {
            std::borrow::Cow::Borrowed(descendant_obj)
        };

        let array = resolved.as_array().ok_or_else(|| Error::ParseError {
            offset: 0,
            reason: format!("Type0 font '{}': DescendantFonts is not an array", base_font),
        })?;

        if array.is_empty() {
            return Err(Error::ParseError {
                offset: 0,
                reason: format!(
                    "Type0 font '{}': DescendantFonts array is empty - must have at least 1 element",
                    base_font
                ),
            });
        }

        // Use first element (PDF spec: "Usually contains a single element")
        if array.len() > 1 {
            log::warn!(
                "Font '{}': DescendantFonts array has {} elements, using first",
                base_font,
                array.len()
            );
        }

        // accept both indirect
        // references AND direct dictionary objects in DescendantFonts.
        // PDF spec §9.7.6 mandates indirect refs, but Persian / Farsi
        // PDFs from older XeTeX / pdfTeX writers (Nazanin, Yagut,
        // Mitra, Lotus fonts) commonly inline the CIDFont dict
        // directly. Older versions rejected the inline form with
        // "DescendantFonts[0] is not a reference" and fell back to
        // Identity-H, which emits CIDs as Latin-Extended-B garbage
        // instead of mapping through the CIDSystemInfo collection.
        // Accepting the inline form gets the parser past this gate;
        // bundling the official Adobe-Persian-1-UCS2 /
        // Adobe-Arabic-1-UCS2 CMap data is a separate follow-up.
        let cidfont_obj_owned;
        let cidfont_dict = match array[0].as_reference() {
            Some(cidfont_ref) => {
                cidfont_obj_owned = doc.load_object(cidfont_ref)?;
                cidfont_obj_owned
                    .as_dict()
                    .ok_or_else(|| Error::ParseError {
                        offset: 0,
                        reason: format!("Type0 font '{}': CIDFont is not a dictionary", base_font),
                    })?
            },
            None => {
                // Inline-dict path — accept it per §9.7.6 lenient
                // reader posture.
                log::info!(
                    "Type0 font '{}': DescendantFonts[0] is a direct dictionary \
                     (non-conformant per §9.7.6 but recoverable); parsing inline",
                    base_font,
                );
                array[0].as_dict().ok_or_else(|| Error::ParseError {
                    offset: 0,
                    reason: format!(
                        "Type0 font '{}': DescendantFonts[0] is neither a reference \
                         nor a dictionary",
                        base_font
                    ),
                })?
            },
        };

        // Get CIDFont subtype (required: CIDFontType0 or CIDFontType2)
        let cid_font_type = cidfont_dict
            .get("Subtype")
            .and_then(|obj| obj.as_name())
            .ok_or_else(|| Error::ParseError {
                offset: 0,
                reason: format!("Type0 font '{}': CIDFont missing required /Subtype", base_font),
            })?;

        // Validate subtype
        if cid_font_type != "CIDFontType0" && cid_font_type != "CIDFontType2" {
            return Err(Error::ParseError {
                offset: 0,
                reason: format!(
                    "Type0 font '{}': Invalid CIDFontType '{}' (must be CIDFontType0 or CIDFontType2)",
                    base_font, cid_font_type
                ),
            });
        }

        // Parse CIDSystemInfo (required for all CIDFonts)
        let cid_system_info = match Self::parse_cidsysteminfo(cidfont_dict, doc) {
            Ok(info) => Some(info),
            Err(e) => {
                log::warn!(
                    "Font '{}': Failed to parse CIDSystemInfo: {}. Continuing with None.",
                    base_font,
                    e
                );
                None
            },
        };

        // Parse CIDToGIDMap (only for CIDFontType2 - TrueType-based)
        let cid_to_gid_map = if cid_font_type == "CIDFontType2" {
            match cidfont_dict.get("CIDToGIDMap") {
                None => {
                    // Default to Identity if not specified
                    log::debug!(
                        "Font '{}': CIDToGIDMap not specified, defaulting to Identity",
                        base_font
                    );
                    Some(CIDToGIDMap::Identity)
                },
                Some(cidtogid_obj) => {
                    // Handle Name object "/Identity"
                    if let Some(name) = cidtogid_obj.as_name() {
                        if name == "Identity" {
                            log::debug!("Font '{}': CIDToGIDMap is Identity", base_font);
                            Some(CIDToGIDMap::Identity)
                        } else {
                            log::warn!(
                                "Font '{}': Invalid CIDToGIDMap name '{}' (only 'Identity' is valid as name)",
                                base_font,
                                name
                            );
                            Some(CIDToGIDMap::Identity) // Fallback
                        }
                    } else if let Some(stream_ref) = cidtogid_obj.as_reference() {
                        // Handle Stream object (binary uint16 array)
                        match doc.load_object(stream_ref) {
                            Ok(stream_obj) => {
                                match doc.decode_stream_with_encryption(&stream_obj, stream_ref) {
                                    Ok(stream_data) => {
                                        // Validate stream length (must be even)
                                        if stream_data.len() % 2 != 0 {
                                            log::warn!(
                                            "Font '{}': CIDToGIDMap stream has odd length {} (must be even). Using Identity fallback.",
                                            base_font,
                                            stream_data.len()
                                        );
                                            Some(CIDToGIDMap::Identity)
                                        } else if stream_data.is_empty() {
                                            log::warn!(
                                            "Font '{}': CIDToGIDMap stream is empty. Using Identity fallback.",
                                            base_font
                                        );
                                            Some(CIDToGIDMap::Identity)
                                        } else {
                                            // Parse big-endian uint16 array
                                            let num_entries = stream_data.len() / 2;
                                            let mut map = Vec::with_capacity(num_entries);
                                            for i in 0..num_entries {
                                                let gid = u16::from_be_bytes([
                                                    stream_data[i * 2],
                                                    stream_data[i * 2 + 1],
                                                ]);
                                                map.push(gid);
                                            }
                                            log::debug!(
                                            "Font '{}': Loaded explicit CIDToGIDMap with {} entries",
                                            base_font,
                                            num_entries
                                        );
                                            Some(CIDToGIDMap::Explicit(map))
                                        }
                                    },
                                    Err(e) => {
                                        log::warn!(
                                        "Font '{}': CIDToGIDMap stream decode failed: {}. Using Identity fallback.",
                                        base_font,
                                        e
                                    );
                                        Some(CIDToGIDMap::Identity)
                                    },
                                }
                            },
                            Err(e) => {
                                log::warn!(
                                    "Font '{}': CIDToGIDMap stream object load failed: {}. Using Identity fallback.",
                                    base_font,
                                    e
                                );
                                Some(CIDToGIDMap::Identity)
                            },
                        }
                    } else {
                        log::warn!(
                            "Font '{}': CIDToGIDMap is neither Name nor Stream reference. Using Identity fallback.",
                            base_font
                        );
                        Some(CIDToGIDMap::Identity)
                    }
                },
            }
        } else {
            // CIDFontType0 (CFF/OpenType) doesn't use CIDToGIDMap
            log::debug!(
                "Font '{}': CIDFontType0 (CFF/OpenType) - no CIDToGIDMap needed",
                base_font
            );
            None
        };

        // Parse /DW (default width for CIDs) - PDF Spec Section 9.7.4.3
        // Default is 1000 if not specified
        let dw_value = cidfont_dict.get("DW").and_then(|obj| {
            // Resolve indirect reference if needed
            let resolved = if let Some(r) = obj.as_reference() {
                doc.load_object(r).ok()
            } else {
                None
            };
            match resolved.as_ref().unwrap_or(obj) {
                Object::Integer(i) => Some(*i as f32),
                Object::Real(r) => Some(*r as f32),
                _ => None,
            }
        });
        // F14/F15 fix: track whether /DW was explicitly present in the PDF.
        let has_explicit_dw = dw_value.is_some();
        let cid_default_width = dw_value.unwrap_or(1000.0);

        // Parse /W array (CID widths) - PDF Spec Section 9.7.4.3
        // Resolve /W reference if needed before parsing (common for large arrays)
        let resolved_cidfont_dict = if let Some(w_obj) = cidfont_dict.get("W") {
            if let Some(r) = w_obj.as_reference() {
                match doc.load_object(r) {
                    Ok(resolved) => {
                        let mut dict_clone = crate::object_copy::clone_dictionary(
                            cidfont_dict,
                            doc.import_budget().map(AsRef::as_ref),
                        )?;
                        dict_clone.insert("W".to_string(), resolved);
                        std::borrow::Cow::Owned(dict_clone)
                    },
                    Err(e) => {
                        log::warn!("Font '{}': Failed to resolve /W reference: {}", base_font, e);
                        std::borrow::Cow::Borrowed(cidfont_dict)
                    },
                }
            } else {
                std::borrow::Cow::Borrowed(cidfont_dict)
            }
        } else {
            std::borrow::Cow::Borrowed(cidfont_dict)
        };
        let cid_widths = match doc.import_budget() {
            Some(budget) => {
                Self::parse_cid_widths_with_budget(&resolved_cidfont_dict, base_font, Some(budget))
            },
            None => Self::parse_cid_widths(&resolved_cidfont_dict, base_font),
        };
        doc.check_import_budget()?;

        if cid_widths.is_some() {
            log::debug!(
                "Font '{}': Parsed CID widths - {} entries, default width {}",
                base_font,
                cid_widths.as_ref().map(|m| m.len()).unwrap_or(0),
                cid_default_width
            );
        }

        // Parse /W2 (per-CID vertical metrics) and /DW2 (default vertical
        // metrics) — ISO 32000-1 §9.7.4.3. Most fonts ship horizontal-only;
        // when /W2 is absent the per-CID HashMap is never allocated.
        let resolved_for_w2 = if let Some(w2_obj) = cidfont_dict.get("W2") {
            if let Some(r) = w2_obj.as_reference() {
                match doc.load_object(r) {
                    Ok(resolved) => {
                        let mut dict_clone = crate::object_copy::clone_dictionary(
                            resolved_cidfont_dict.as_ref(),
                            doc.import_budget().map(AsRef::as_ref),
                        )?;
                        dict_clone.insert("W2".to_string(), resolved);
                        std::borrow::Cow::Owned(dict_clone)
                    },
                    Err(e) => {
                        log::warn!("Font '{}': Failed to resolve /W2 reference: {}", base_font, e);
                        std::borrow::Cow::Borrowed(resolved_cidfont_dict.as_ref())
                    },
                }
            } else {
                std::borrow::Cow::Borrowed(resolved_cidfont_dict.as_ref())
            }
        } else {
            std::borrow::Cow::Borrowed(resolved_cidfont_dict.as_ref())
        };
        let cid_vertical_metrics = match doc.import_budget() {
            Some(budget) => Self::parse_cid_vertical_metrics_with_budget(
                &resolved_for_w2,
                base_font,
                Some(budget),
            ),
            None => Self::parse_cid_vertical_metrics(&resolved_for_w2, base_font),
        };
        doc.check_import_budget()?;
        let cid_default_vertical_metrics = Self::parse_dw2(&resolved_for_w2);
        if cid_vertical_metrics.is_some() {
            log::debug!(
                "Font '{}': Parsed /W2 vertical metrics - {} entries, /DW2 defaults w1y={} v_x={} v_y={}",
                base_font,
                cid_vertical_metrics.as_ref().map(|m| m.len()).unwrap_or(0),
                cid_default_vertical_metrics.w1y,
                cid_default_vertical_metrics.v_x,
                cid_default_vertical_metrics.v_y,
            );
        }

        // Extract TrueType cmap from descendant's FontDescriptor if available.
        // Type0 parent fonts often have no embedded data — it's on the CIDFont.
        let descendant_tt_cmap = if cid_font_type == "CIDFontType2" {
            Self::extract_truetype_cmap_from_descriptor(cidfont_dict, base_font, doc)
        } else {
            None
        };

        // Extract embedded font data from CIDFont's FontDescriptor.
        // Per PDF spec, embedded font programs for Type0 fonts live on the
        // CIDFont descendant's FontDescriptor, not on the Type0 wrapper.
        let descendant_embedded =
            Self::extract_embedded_font_from_descriptor(cidfont_dict, base_font, doc);

        // Extract ascent/descent from the CIDFont's FontDescriptor (§9.7.4 / Table 117).
        // The Type0 wrapper has no top-level /FontDescriptor, so these values must be
        // read from the descendant.
        let (desc_raw_ascent, desc_raw_descent) =
            Self::read_raw_ascent_descent_from_descriptor(cidfont_dict, doc);

        Ok((
            cid_to_gid_map,
            cid_system_info,
            Some(
                FontAllocation(doc.import_budget().map(AsRef::as_ref))
                    .string(cid_font_type)
                    .ok_or(Error::ImportResourceLimit("CID subtype copy"))?,
            ),
            cid_widths,
            cid_default_width,
            has_explicit_dw,
            descendant_tt_cmap,
            descendant_embedded,
            desc_raw_ascent,
            desc_raw_descent,
            cid_vertical_metrics,
            cid_default_vertical_metrics,
        ))
    }

    fn descriptor_program_presence(descriptor: Option<&Object>, doc: &PdfDocument) -> Option<bool> {
        let Some(descriptor) = descriptor else {
            return Some(false);
        };
        let resolved = match descriptor.as_reference() {
            Some(reference) => std::borrow::Cow::Owned(doc.load_object(reference).ok()?),
            None => std::borrow::Cow::Borrowed(descriptor),
        };
        let dictionary = resolved.as_dict()?;
        Some(
            ["FontFile", "FontFile2", "FontFile3"]
                .iter()
                .any(|key| dictionary.contains_key(*key)),
        )
    }

    /// Extract TrueType cmap from a font dictionary's /FontDescriptor /FontFile2.
    fn extract_truetype_cmap_from_descriptor(
        font_dict: &HashMap<String, Object>,
        base_font: &str,
        doc: &PdfDocument,
    ) -> Option<TrueTypeCMap> {
        let desc_obj = font_dict.get("FontDescriptor")?;
        let desc = if let Some(r) = desc_obj.as_reference() {
            std::borrow::Cow::Owned(doc.load_object(r).ok()?)
        } else {
            std::borrow::Cow::Borrowed(desc_obj)
        };
        let desc_dict = desc.as_dict()?;
        let ff2_obj = desc_dict.get("FontFile2")?;
        let ff2_ref = ff2_obj.as_reference()?;
        let ff2_stream = match doc.load_object(ff2_ref) {
            Ok(obj) => obj,
            Err(e) => {
                log::warn!(
                    "Font '{}': Failed to load FontFile2 object {:?}: {}",
                    base_font,
                    ff2_ref,
                    e
                );
                return None;
            },
        };
        let font_data = match doc.decode_stream_with_encryption(&ff2_stream, ff2_ref) {
            Ok(data) => data,
            Err(e) => {
                log::warn!(
                    "Font '{}': Failed to decrypt/decode FontFile2 stream {:?}: {}",
                    base_font,
                    ff2_ref,
                    e
                );
                return None;
            },
        };
        if font_data.is_empty() {
            return None;
        }
        match TrueTypeCMap::from_font_data_with_budget(
            &font_data,
            doc.import_budget().map(AsRef::as_ref),
        ) {
            Ok(cmap) if !cmap.is_empty() => {
                log::info!(
                    "Font '{}': Extracted TrueType cmap from descendant CIDFont ({} mappings)",
                    base_font,
                    cmap.len()
                );
                Some(cmap)
            },
            _ => None,
        }
    }

    /// Read raw /Ascent and /Descent from a font dictionary's /FontDescriptor.
    /// Returns (raw_ascent, raw_descent) in PDF 1/1000-em units, or None if absent.
    /// Used to pull ascent/descent off a CIDFont descendant (§9.7.4 / Table 117).
    fn read_raw_ascent_descent_from_descriptor(
        font_dict: &HashMap<String, Object>,
        doc: &PdfDocument,
    ) -> (Option<f32>, Option<f32>) {
        let desc_obj = match font_dict.get("FontDescriptor") {
            Some(obj) => obj,
            None => return (None, None),
        };
        let desc = if let Some(r) = desc_obj.as_reference() {
            match doc.load_object(r) {
                Ok(obj) => std::borrow::Cow::Owned(obj),
                Err(_) => return (None, None),
            }
        } else {
            std::borrow::Cow::Borrowed(desc_obj)
        };
        let desc_dict = match desc.as_dict() {
            Some(d) => d,
            None => return (None, None),
        };
        let read_f32 = |key: &str| -> Option<f32> {
            desc_dict.get(key).and_then(|o| {
                o.as_real()
                    .map(|r| r as f32)
                    .or_else(|| o.as_integer().map(|i| i as f32))
            })
        };
        (read_f32("Ascent"), read_f32("Descent"))
    }

    /// Extract embedded font data from a font dictionary's /FontDescriptor.
    /// Checks FontFile2 (TrueType), FontFile3 (CFF/OpenType), and FontFile (Type 1).
    fn extract_embedded_font_from_descriptor(
        font_dict: &HashMap<String, Object>,
        base_font: &str,
        doc: &PdfDocument,
    ) -> (bool, Option<Arc<Vec<u8>>>) {
        let Some(desc_obj) = font_dict.get("FontDescriptor") else {
            return (false, None);
        };
        let desc = if let Some(r) = desc_obj.as_reference() {
            match doc.load_object(r) {
                Ok(obj) => std::borrow::Cow::Owned(obj),
                Err(_) => return (false, None),
            }
        } else {
            std::borrow::Cow::Borrowed(desc_obj)
        };
        let Some(desc_dict) = desc.as_dict() else {
            return (false, None);
        };

        // Try FontFile2 (TrueType), FontFile3 (CFF/OpenType), FontFile (Type 1)
        let font_file_keys = ["FontFile2", "FontFile3", "FontFile"];
        // Key presence ≠ extraction success: callers gating on "the document
        // embeds its own outlines" must see `true` even when every present
        // stream fails to load/decode below.
        let has_font_program = font_file_keys
            .iter()
            .any(|key| desc_dict.contains_key(*key));
        for key in &font_file_keys {
            if let Some(ff_obj) = desc_dict.get(*key) {
                let ff_ref = match ff_obj.as_reference() {
                    Some(r) => r,
                    None => continue,
                };
                let ff_stream = match doc.load_object(ff_ref) {
                    Ok(obj) => std::borrow::Cow::Owned(obj),
                    Err(e) => {
                        log::warn!(
                            "Font '{}': Failed to load {} {:?}: {}",
                            base_font,
                            key,
                            ff_ref,
                            e
                        );
                        continue;
                    },
                };
                let font_data = match doc.decode_stream_with_encryption(&ff_stream, ff_ref) {
                    Ok(data) => data,
                    Err(e) => {
                        log::warn!("Font '{}': Failed to decode {} stream: {}", base_font, key, e);
                        continue;
                    },
                };
                if !font_data.is_empty() {
                    // If this is raw CFF data (FontFile3), wrap it in an OpenType
                    // container so ttf-parser can parse it.
                    let font_data = if *key == "FontFile3" && !font_data.is_empty()
                        && font_data[0] == 1 // CFF version 1
                        && font_data.len() > 4
                    {
                        log::info!(
                            "Font '{}': Wrapping raw CFF in OpenType container ({} bytes)",
                            base_font,
                            font_data.len()
                        );
                        wrap_cff_in_opentype_with_budget(
                            &font_data,
                            doc.import_budget().map(AsRef::as_ref),
                        )
                    } else {
                        font_data
                    };
                    log::info!(
                        "Font '{}': Extracted embedded font from {} ({} bytes)",
                        base_font,
                        key,
                        font_data.len()
                    );
                    return (has_font_program, Some(Arc::new(font_data)));
                }
            }
        }
        (has_font_program, None)
    }
}

/// Wrap raw CFF font data in a minimal OpenType container so ttf-parser can parse it.
/// Creates an OpenType font with `head` and `CFF ` tables (both required by ttf-parser).
#[cfg(test)]
fn wrap_cff_in_opentype(cff_data: &[u8]) -> Vec<u8> {
    wrap_cff_in_opentype_with_budget(cff_data, None)
}

fn wrap_cff_in_opentype_with_budget(
    cff_data: &[u8],
    budget: Option<&crate::import_limits::ImportBudget>,
) -> Vec<u8> {
    let allocation = FontAllocation(budget);
    if allocation.check().is_none() {
        return Vec::new();
    }
    let num_tables: u16 = 4; // CFF + head + hhea + maxp
    let search_range: u16 = 32; // largest power of 2 <= numTables*16 = 64 → 32 (2 tables)
    let entry_selector: u16 = 2;
    let range_shift: u16 = (num_tables * 16) - search_range;

    // The em size the CFF's charstrings are actually drawn in.
    //
    // ISO 32000-1:2008 §9.2.2 (`docs/spec/pdf.md`:8459-8461): "The
    // transformation from glyph space to text space shall be defined by the
    // font matrix. For most types of fonts, this matrix shall be predefined to
    // map 1000 units of glyph space to 1 unit of text space." *Most* — a CFF
    // may declare its own `/FontMatrix` (Adobe TN #5176, operator 12 7), and
    // subsetters from a 2048-unit-em workflow routinely emit 1/2048.
    //
    // This head table is ours, fabricated to wrap a bare CFF, and the
    // rasteriser scales every outline by `font_size / units_per_em` read from
    // it. Writing 1000 unconditionally therefore painted a 2048-unit font at
    // 2.048x its intended size. Advances were unaffected — they come from
    // `/Widths` — so the glyphs simply overlapped and smeared, which on one
    // form rendered the whole page about 21 grey levels too dark.
    //
    // Only a plain uniform scale is folded in; anything skewed, rotated or
    // non-uniform keeps the 1000 default, which is also the CFF default.
    let units_per_em: u16 =
        crate::fonts::cff_encoding::parse_cff_font_matrix_scale_with_budget(cff_data, budget)
            .map(|sx| (1.0 / sx).round())
            .filter(|u| (16.0..=16384.0).contains(u))
            .map(|u| u as u16)
            .unwrap_or(1000);
    let [upem_hi, upem_lo] = units_per_em.to_be_bytes();

    // Minimal head table (54 bytes) — OpenType spec required fields
    let head_table: [u8; 54] = [
        0x00, 0x01, 0x00, 0x00, // majorVersion=1, minorVersion=0
        0x00, 0x01, 0x00, 0x00, // fontRevision=1.0
        0x00, 0x00, 0x00, 0x00, // checksumAdjustment (0, will be ignored)
        0x5F, 0x0F, 0x3C, 0xF5, // magicNumber
        0x00, 0x0B, // flags (baseline at y=0, lsb at x=0, etc)
        upem_hi, upem_lo, // unitsPerEm, from the CFF's own /FontMatrix
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // created (0)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // modified (0)
        0xFF, 0x38, // xMin = -200
        0xFF, 0x38, // yMin = -200
        0x03, 0xE8, // xMax = 1000
        0x03, 0xE8, // yMax = 1000
        0x00, 0x00, // macStyle
        0x00, 0x08, // lowestRecPPEM = 8
        0x00, 0x02, // fontDirectionHint
        0x00, 0x01, // indexToLocFormat = 1 (long)
        0x00, 0x00, // glyphDataFormat
    ];

    // Minimal hhea table (36 bytes)
    let hhea_table: [u8; 36] = [
        0x00, 0x01, 0x00, 0x00, // majorVersion=1, minorVersion=0
        0x03, 0x20, // ascender = 800
        0xFF, 0x38, // descender = -200
        0x00, 0x00, // lineGap = 0
        0x04, 0x00, // advanceWidthMax = 1024
        0x00, 0x00, // minLeftSideBearing = 0
        0x00, 0x00, // minRightSideBearing = 0
        0x04, 0x00, // xMaxExtent = 1024
        0x00, 0x01, // caretSlopeRise = 1
        0x00, 0x00, // caretSlopeRun = 0
        0x00, 0x00, // caretOffset = 0
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, // reserved
        0x00, 0x00, // metricDataFormat = 0
        0x01, 0x00, // numberOfHMetrics = 256
    ];

    // Minimal maxp table (6 bytes for CFF fonts — version 0.5)
    let maxp_table: [u8; 6] = [
        0x00, 0x00, 0x50, 0x00, // version = 0.5 (CFF)
        0x01, 0x00, // numGlyphs = 256
    ];

    // Layout: offset table (12) + 4 table records (64) = 76 bytes header
    let header_size: u32 = 12 + (num_tables as u32) * 16;
    // Place tables: head, hhea, maxp, CFF (alphabetical by tag within each group)
    let head_offset = (header_size + 3) & !3;
    let head_len = head_table.len() as u32;
    let hhea_offset = ((head_offset + head_len) + 3) & !3;
    let hhea_len = hhea_table.len() as u32;
    let maxp_offset = ((hhea_offset + hhea_len) + 3) & !3;
    let maxp_len = maxp_table.len() as u32;
    let cff_offset = ((maxp_offset + maxp_len) + 3) & !3;
    let cff_len = cff_data.len() as u32;

    fn table_checksum(data: &[u8]) -> u32 {
        let mut sum: u32 = 0;
        for chunk in data.chunks(4) {
            let mut bytes = [0u8; 4];
            bytes[..chunk.len()].copy_from_slice(chunk);
            sum = sum.wrapping_add(u32::from_be_bytes(bytes));
        }
        sum
    }

    let capacity = (cff_offset as usize).checked_add(cff_len as usize);
    let Some(capacity) = capacity else {
        allocation.reject();
        return Vec::new();
    };
    if allocation.bytes(capacity).is_none() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if out.try_reserve_exact(capacity).is_err() {
        allocation.reject();
        return Vec::new();
    }

    // Offset table (12 bytes)
    out.extend_from_slice(b"OTTO");
    out.extend_from_slice(&num_tables.to_be_bytes());
    out.extend_from_slice(&search_range.to_be_bytes());
    out.extend_from_slice(&entry_selector.to_be_bytes());
    out.extend_from_slice(&range_shift.to_be_bytes());

    // Table record: CFF (alphabetical order: CFF before head)
    out.extend_from_slice(b"CFF ");
    out.extend_from_slice(&table_checksum(cff_data).to_be_bytes());
    out.extend_from_slice(&cff_offset.to_be_bytes());
    out.extend_from_slice(&cff_len.to_be_bytes());

    // Table record: head
    out.extend_from_slice(b"head");
    out.extend_from_slice(&table_checksum(&head_table).to_be_bytes());
    out.extend_from_slice(&head_offset.to_be_bytes());
    out.extend_from_slice(&head_len.to_be_bytes());

    // Table record: hhea
    out.extend_from_slice(b"hhea");
    out.extend_from_slice(&table_checksum(&hhea_table).to_be_bytes());
    out.extend_from_slice(&hhea_offset.to_be_bytes());
    out.extend_from_slice(&hhea_len.to_be_bytes());

    // Table record: maxp
    out.extend_from_slice(b"maxp");
    out.extend_from_slice(&table_checksum(&maxp_table).to_be_bytes());
    out.extend_from_slice(&maxp_offset.to_be_bytes());
    out.extend_from_slice(&maxp_len.to_be_bytes());

    // head table data
    while out.len() < head_offset as usize {
        out.push(0);
    }
    out.extend_from_slice(&head_table);

    // hhea table data
    while out.len() < hhea_offset as usize {
        out.push(0);
    }
    out.extend_from_slice(&hhea_table);

    // maxp table data
    while out.len() < maxp_offset as usize {
        out.push(0);
    }
    out.extend_from_slice(&maxp_table);

    // Pad to CFF offset
    while out.len() < cff_offset as usize {
        out.push(0);
    }

    // CFF data
    out.extend_from_slice(cff_data);

    out
}

impl FontInfo {
    /// Parse CIDFont /W array for glyph widths.
    ///
    /// Per PDF Spec ISO 32000-1:2008, Section 9.7.4.3, the /W array has two formats:
    /// - `c [w1 w2 ... wn]` - CID c has width w1, c+1 has width w2, etc.
    /// - `cfirst clast w` - CIDs from cfirst to clast all have width w
    ///
    /// These formats can be mixed in a single array.
    ///
    /// # Example /W array
    /// ```pdf
    /// /W [
    ///   1 [500 600 700] % CID 1=500, CID 2=600, CID 3=700
    ///   100 200 300 % CIDs 100-200 all have width 300
    /// ]
    /// ```
    /// Inspect a Type0 font's `/Encoding` object and resolve the writing
    /// mode it implies, plus the encoding name preserved for diagnostics.
    ///
    /// Returns a pair `(name, wmode)` where:
    /// - `name` is the predefined-CMap name when `/Encoding` is a `/Name`
    ///   atom (`Identity-H`, `Identity-V`, `UniJIS-UTF16-V`, …) or the
    ///   embedded CMap stream's `/CMapName` value when `/Encoding` is a
    ///   stream/dict reference.
    /// - `wmode` is `1` when the resolved name ends in `-V` or equals the
    ///   bare legacy `V`, or when the embedded CMap stream contains a
    ///   `/WMode 1 def` directive. `0` otherwise (including unknown).
    ///
    /// The two signals are surfaced separately so callers can apply the
    /// precedence rules from ISO 32000-1 §9.7.5.4: an embedded CMap stream's
    /// explicit `/WMode` overrides what the name might suggest.
    fn resolve_encoding_writing_mode(enc_obj: &Object, doc: &PdfDocument) -> (Option<String>, u8) {
        let allocation = FontAllocation(doc.import_budget().map(AsRef::as_ref));
        // Case 1: /Encoding is a /Name atom — predefined CMap name.
        if let Some(name) = enc_obj.as_name() {
            let wmode = wmode_from_predefined_cmap_name(name);
            return (allocation.string(name), wmode);
        }

        // Case 2: /Encoding is a stream/dict — embedded CMap. The dict may
        // expose a /CMapName and the stream body may carry /WMode N def.
        let dict = enc_obj.as_dict();
        let name = dict
            .and_then(|d| d.get("CMapName"))
            .and_then(|n| n.as_name())
            .and_then(|s| allocation.string(s));

        // Try to decode the CMap stream and scan for /WMode. We swallow
        // decode errors here — if the stream cannot be decoded, the existing
        // `parse_encoding` path will eventually log it; for wmode detection
        // we silently fall back to the name-based signal.
        let stream_wmode = match enc_obj.decode_stream_data() {
            Ok(bytes) => {
                let content = String::from_utf8_lossy(&bytes);
                crate::fonts::cmap::parse_wmode_directive_public(&content)
            },
            Err(_) => None,
        };
        let _ = doc; // doc reserved for future use (e.g. resolving /UseCMap refs).

        let name_wmode = name
            .as_deref()
            .map(wmode_from_predefined_cmap_name)
            .unwrap_or(0);
        let wmode = stream_wmode.unwrap_or(name_wmode);
        (name, wmode)
    }

    /// Parse `/DW2` from a CIDFont dictionary.
    ///
    /// Per ISO 32000-1 §9.7.4.3 the value is an array of two numbers:
    /// `[v_y_default w1y_default]`. Spec default when `/DW2` is absent is
    /// `[880 -1000]`. The default `v_x` is always `500` (half-em) — the spec
    /// does not provide a way to override it via `/DW2`.
    ///
    /// Returns the parsed defaults, or [`VerticalMetrics::SPEC_DEFAULT`] when
    /// `/DW2` is missing or malformed.
    fn parse_dw2(cidfont_dict: &HashMap<String, Object>) -> VerticalMetrics {
        let Some(dw2_obj) = cidfont_dict.get("DW2") else {
            return VerticalMetrics::SPEC_DEFAULT;
        };
        let Some(arr) = dw2_obj.as_array() else {
            return VerticalMetrics::SPEC_DEFAULT;
        };
        if arr.len() < 2 {
            return VerticalMetrics::SPEC_DEFAULT;
        }
        let v_y = match &arr[0] {
            Object::Integer(i) => *i as f32,
            Object::Real(r) => *r as f32,
            _ => return VerticalMetrics::SPEC_DEFAULT,
        };
        let w1y = match &arr[1] {
            Object::Integer(i) => *i as f32,
            Object::Real(r) => *r as f32,
            _ => return VerticalMetrics::SPEC_DEFAULT,
        };
        VerticalMetrics {
            w1y,
            v_x: 500.0,
            v_y,
        }
    }

    /// Parse `/W2` (per-CID vertical metrics) from a CIDFont dictionary.
    ///
    /// Per ISO 32000-1 §9.7.4.3 the `/W2` array uses two forms, both of which
    /// may be intermixed within a single `/W2`:
    ///
    /// - Form A — explicit per-CID metrics:
    ///   `c [ w1y v_x v_y w1y v_x v_y … ]` — the inner array holds successive
    ///   `(w1y, v_x, v_y)` triples assigned to CIDs `c, c+1, c+2, …`.
    ///
    /// - Form B — range:
    ///   `c_first c_last w1y v_x v_y` — every CID in `c_first..=c_last`
    ///   shares the same `(w1y, v_x, v_y)`.
    ///
    /// Returns `None` when `/W2` is absent or empty, allowing callers to skip
    /// the HashMap allocation entirely on horizontal fonts.
    fn parse_cid_vertical_metrics(
        cidfont_dict: &HashMap<String, Object>,
        base_font: &str,
    ) -> Option<HashMap<u16, VerticalMetrics>> {
        Self::parse_cid_vertical_metrics_with_budget(cidfont_dict, base_font, None)
    }

    fn parse_cid_vertical_metrics_with_budget(
        cidfont_dict: &HashMap<String, Object>,
        base_font: &str,
        budget: Option<&crate::import_limits::ImportBudget>,
    ) -> Option<HashMap<u16, VerticalMetrics>> {
        let w2_obj = cidfont_dict.get("W2")?;
        let w2_array = w2_obj.as_array()?;

        if w2_array.is_empty() {
            return None;
        }

        let mut metrics: HashMap<u16, VerticalMetrics> = HashMap::new();
        let mut i = 0;

        while i < w2_array.len() {
            let cid_start = match &w2_array[i] {
                Object::Integer(c) => *c as u16,
                _ => {
                    log::warn!(
                        "Font '{}': /W2 array element {} is not an integer, skipping",
                        base_font,
                        i
                    );
                    i += 1;
                    continue;
                },
            };
            i += 1;

            if i >= w2_array.len() {
                break;
            }

            match &w2_array[i] {
                Object::Array(triples) => {
                    if let Some(budget) = budget {
                        let count = triples.len() / 3;
                        budget.charge_font_entries(count).ok()?;
                        metrics
                            .try_reserve(count)
                            .map_err(|_| budget.reject("font allocation"))
                            .ok()?;
                    }
                    // Form A: c [ w1y v_x v_y w1y v_x v_y … ]
                    // Walk the inner array in groups of three. A triple is
                    // atomic: if any of its three elements is non-numeric
                    // we drop the WHOLE triple (advance j+=3, emitted+=1)
                    // so the CID alignment of the rest of the inner array
                    // is preserved. The original implementation advanced
                    // j by 1 on a malformed element, which silently
                    // shifted every subsequent CID by one slot.
                    let mut j = 0;
                    let mut emitted: u32 = 0;
                    let read_num = |obj: &Object| -> Option<f32> {
                        match obj {
                            Object::Integer(v) => Some(*v as f32),
                            Object::Real(v) => Some(*v as f32),
                            _ => None,
                        }
                    };
                    while j + 2 < triples.len() {
                        let triple = (
                            read_num(&triples[j]),
                            read_num(&triples[j + 1]),
                            read_num(&triples[j + 2]),
                        );
                        // Compute CID with overflow detection BEFORE writing.
                        // saturating_add(emitted) would collapse every
                        // overflowing slot onto u16::MAX; instead we stop.
                        let Some(cid) = (cid_start as u32).checked_add(emitted) else {
                            log::warn!(
                                "Font '{}': /W2 Form A starting at CID {} overflowed u32 \
                                 at emitted offset {}; stopping",
                                base_font,
                                cid_start,
                                emitted
                            );
                            break;
                        };
                        if cid > u16::MAX as u32 {
                            log::warn!(
                                "Font '{}': /W2 Form A starting at CID {} would assign \
                                 beyond u16::MAX at emitted offset {}; stopping",
                                base_font,
                                cid_start,
                                emitted
                            );
                            break;
                        }
                        match triple {
                            (Some(w1y), Some(v_x), Some(v_y)) => {
                                metrics.insert(cid as u16, VerticalMetrics { w1y, v_x, v_y });
                            },
                            _ => {
                                log::warn!(
                                    "Font '{}': /W2 Form A triple starting at CID {} (offset \
                                     {}) is malformed; dropping it (keeping CID alignment)",
                                    base_font,
                                    cid_start,
                                    emitted
                                );
                            },
                        }
                        emitted += 1;
                        j += 3;
                    }
                    i += 1;
                },
                Object::Integer(cid_end_int) => {
                    // Form B: c_first c_last w1y v_x v_y
                    let cid_end = *cid_end_int as u16;
                    i += 1;
                    if i + 2 >= w2_array.len() {
                        log::warn!(
                            "Font '{}': /W2 range starting at CID {} truncated",
                            base_font,
                            cid_start
                        );
                        break;
                    }
                    let read = |obj: &Object| -> Option<f32> {
                        match obj {
                            Object::Integer(v) => Some(*v as f32),
                            Object::Real(v) => Some(*v as f32),
                            _ => None,
                        }
                    };
                    let Some(w1y) = read(&w2_array[i]) else {
                        i += 3;
                        continue;
                    };
                    let Some(v_x) = read(&w2_array[i + 1]) else {
                        i += 3;
                        continue;
                    };
                    let Some(v_y) = read(&w2_array[i + 2]) else {
                        i += 3;
                        continue;
                    };
                    i += 3;
                    let metric = VerticalMetrics { w1y, v_x, v_y };
                    if let Some(budget) = budget {
                        let count = usize::from(cid_end)
                            .checked_sub(usize::from(cid_start))
                            .map_or(0, |n| n + 1);
                        budget.charge_font_entries(count).ok()?;
                        metrics
                            .try_reserve(count)
                            .map_err(|_| budget.reject("font allocation"))
                            .ok()?;
                    }
                    for cid in cid_start..=cid_end {
                        metrics.insert(cid, metric);
                    }
                },
                _ => {
                    log::warn!(
                        "Font '{}': /W2 array has unexpected element type after CID {}",
                        base_font,
                        cid_start
                    );
                    i += 1;
                },
            }
        }

        if metrics.is_empty() {
            None
        } else {
            Some(metrics)
        }
    }

    fn parse_cid_widths(
        cidfont_dict: &HashMap<String, Object>,
        base_font: &str,
    ) -> Option<HashMap<u16, f32>> {
        Self::parse_cid_widths_with_budget(cidfont_dict, base_font, None)
    }

    fn parse_cid_widths_with_budget(
        cidfont_dict: &HashMap<String, Object>,
        base_font: &str,
        budget: Option<&crate::import_limits::ImportBudget>,
    ) -> Option<HashMap<u16, f32>> {
        let w_obj = cidfont_dict.get("W")?;
        let w_array = w_obj.as_array()?;

        if w_array.is_empty() {
            return None;
        }

        let mut widths: HashMap<u16, f32> = HashMap::new();
        let mut i = 0;

        while i < w_array.len() {
            // First element must be a CID (integer)
            let cid_start = match &w_array[i] {
                Object::Integer(c) => *c as u16,
                _ => {
                    log::warn!(
                        "Font '{}': /W array element {} is not an integer, skipping",
                        base_font,
                        i
                    );
                    i += 1;
                    continue;
                },
            };
            i += 1;

            if i >= w_array.len() {
                break;
            }

            // Second element is either:
            // - An array of widths (format: c [w1 w2 ...])
            // - An integer CID end (format: cfirst clast w)
            match &w_array[i] {
                Object::Array(width_array) => {
                    if let Some(budget) = budget {
                        budget.charge_font_entries(width_array.len()).ok()?;
                        widths
                            .try_reserve(width_array.len())
                            .map_err(|_| budget.reject("font allocation"))
                            .ok()?;
                    }
                    // Format: c [w1 w2 ... wn]
                    for (j, width_obj) in width_array.iter().enumerate() {
                        let width = match width_obj {
                            Object::Integer(w) => *w as f32,
                            Object::Real(w) => *w as f32,
                            _ => continue,
                        };
                        let cid = cid_start.saturating_add(j as u16);
                        widths.insert(cid, width);
                    }
                    i += 1;
                },
                Object::Integer(cid_end) => {
                    // Format: cfirst clast w
                    let cid_end = *cid_end as u16;
                    i += 1;

                    if i >= w_array.len() {
                        log::warn!(
                            "Font '{}': /W array missing width for CID range {}-{}",
                            base_font,
                            cid_start,
                            cid_end
                        );
                        break;
                    }

                    let width = match &w_array[i] {
                        Object::Integer(w) => *w as f32,
                        Object::Real(w) => *w as f32,
                        _ => {
                            log::warn!(
                                "Font '{}': /W array has invalid width for CID range {}-{}",
                                base_font,
                                cid_start,
                                cid_end
                            );
                            i += 1;
                            continue;
                        },
                    };
                    i += 1;

                    // Apply width to all CIDs in range
                    if let Some(budget) = budget {
                        let count = usize::from(cid_end)
                            .checked_sub(usize::from(cid_start))
                            .map_or(0, |n| n + 1);
                        budget.charge_font_entries(count).ok()?;
                        widths
                            .try_reserve(count)
                            .map_err(|_| budget.reject("font allocation"))
                            .ok()?;
                    }
                    for cid in cid_start..=cid_end {
                        widths.insert(cid, width);
                    }
                },
                _ => {
                    log::warn!(
                        "Font '{}': /W array has unexpected element type after CID {}",
                        base_font,
                        cid_start
                    );
                    i += 1;
                },
            }
        }

        if widths.is_empty() {
            None
        } else {
            Some(widths)
        }
    }

    /// Vertical advance and origin offset for a CID, in 1000ths-of-em.
    ///
    /// Lookup order:
    /// 1. Per-CID entry from `/W2` (if `cid_vertical_metrics` is populated).
    /// 2. `/DW2` defaults (`cid_default_vertical_metrics`).
    /// 3. Spec defaults from [`VerticalMetrics::SPEC_DEFAULT`] when the font
    ///    is not a CIDFont (e.g. simple Type1/TrueType): callers that
    ///    reach this with a non-Type0 font are degenerate, but returning
    ///    spec defaults is safe.
    ///
    /// This is the vertical counterpart to [`FontInfo::get_glyph_width`] and
    /// is read on the hot path of the renderer / extractor whenever
    /// `self.wmode == 1`.
    #[inline]
    pub fn get_vertical_metrics(&self, cid: u16) -> VerticalMetrics {
        if let Some(map) = &self.cid_vertical_metrics {
            if let Some(&m) = map.get(&cid) {
                return m;
            }
        }
        self.cid_default_vertical_metrics
    }

    /// Handles both named encodings (e.g., /WinAnsiEncoding) and encoding dictionaries
    /// with /Differences arrays that override specific character codes.
    ///
    /// # PDF Spec Reference
    ///
    /// ISO 32000-1:2008, Section 9.6.6.2 - Character Encoding
    ///
    /// A /Differences array has the format:
    /// ```pdf
    /// /Encoding <<
    ///     /BaseEncoding /WinAnsiEncoding
    ///     /Differences [code1 /name1 /name2 ... codeN /nameN ...]
    /// >>
    /// ```
    ///
    /// Where integers specify starting codes, and names specify glyphs for consecutive codes.
    ///
    /// The third element of the returned tuple is the `diff_glyph_names` side
    /// map: `code → /Differences glyph name` for simple fonts (empty otherwise).
    /// It retains the authoritative glyph *name* (not the resolved char) so the
    /// punctuation-recovery interceptions in `char_to_unicode` can consult it.
    fn parse_encoding(
        enc_obj: &Object,
        doc: &PdfDocument,
        font_program_encoding: Option<&HashMap<u8, char>>,
    ) -> Result<(Encoding, HashMap<u8, String>, HashMap<u8, String>)> {
        let a = FontAllocation(doc.import_budget().map(AsRef::as_ref));
        let failure = || Error::ImportResourceLimit("font encoding allocation");
        let empty_map = HashMap::new();
        // Encoding can be either a name or a dictionary
        if let Some(name) = enc_obj.as_name() {
            // Standard encoding names (no /Differences ⇒ no glyph-name side map)
            match name {
                "WinAnsiEncoding" => Ok((
                    Encoding::Standard(a.string("WinAnsiEncoding").ok_or_else(failure)?),
                    empty_map,
                    HashMap::new(),
                )),
                "MacRomanEncoding" => Ok((
                    Encoding::Standard(a.string("MacRomanEncoding").ok_or_else(failure)?),
                    empty_map,
                    HashMap::new(),
                )),
                "MacExpertEncoding" => Ok((
                    Encoding::Standard(a.string("MacExpertEncoding").ok_or_else(failure)?),
                    empty_map,
                    HashMap::new(),
                )),
                "Identity-H" | "Identity-V" => Ok((Encoding::Identity, empty_map, HashMap::new())),
                _ => Ok((
                    Encoding::Standard(a.string(name).ok_or_else(failure)?),
                    empty_map,
                    HashMap::new(),
                )),
            }
        } else if let Some(dict) = enc_obj.as_dict() {
            // Check if this is a CMap stream (Type0 font encoding reference)
            // Per PDF Spec §9.7.5.2, Type0 fonts can reference a CMap stream
            // via /Encoding. For known Adobe character collections (Japan1, GB1,
            // CNS1, Korea1), these define charcode→CID identity mappings and we
            // can resolve CIDs via predefined CID-to-Unicode tables.
            // For custom CMaps (e.g., "Prince-ArialMT-H"), we preserve the default
            // behavior since we can't parse arbitrary CMap programs yet.
            if let Some(cmap_name) = dict.get("CMapName").and_then(|n| n.as_name()) {
                // Check for Adobe standard character collection CMaps.
                // These are named like "Adobe-Japan1-2", "Adobe-Korea1-0", etc.
                // For these collections, charcode→CID is identity, and we can
                // resolve CID→Unicode via predefined tables.
                let is_adobe_collection = cmap_name.starts_with("Adobe-")
                    && (cmap_name.contains("Japan")
                        || cmap_name.contains("GB")
                        || cmap_name.contains("CNS")
                        || cmap_name.contains("Korea"));
                if is_adobe_collection {
                    log::debug!(
                        "Encoding is Adobe CMap stream (CMapName={:?}), treating as Identity",
                        cmap_name
                    );
                    return Ok((Encoding::Identity, HashMap::new(), HashMap::new()));
                }
                // For predefined PDF CMaps like "Identity-H", "Identity-V"
                if cmap_name == "Identity-H" || cmap_name == "Identity-V" {
                    return Ok((Encoding::Identity, HashMap::new(), HashMap::new()));
                }
                // Custom CMap streams (e.g., "Prince-ArialMT-H", "OneByteIdentityH")
                log::debug!(
                    "Encoding is custom CMap stream (CMapName={:?}), treating as Standard",
                    cmap_name
                );
                return Ok((
                    Encoding::Standard(a.string(cmap_name).ok_or_else(failure)?),
                    HashMap::new(),
                    HashMap::new(),
                ));
            }

            // Custom encoding dictionary - parse /Differences array
            let mut multi_char_map: HashMap<u8, String> = HashMap::new();
            // Retain the raw /Differences glyph name per code (see field docs).
            let mut diff_glyph_names: HashMap<u8, String> = HashMap::new();

            // Step 1: Get base encoding (if specified)
            let mut encoding_map: HashMap<u8, char> =
                if let Some(base_enc_obj) = dict.get("BaseEncoding") {
                    // Resolve indirect reference for /BaseEncoding
                    let resolved_base = if let Some(obj_ref) = base_enc_obj.as_reference() {
                        doc.load_object(obj_ref).ok()
                    } else {
                        None
                    };
                    let base_obj = resolved_base.as_ref().unwrap_or(base_enc_obj);

                    if let Some(base_name) = base_obj.as_name() {
                        // Build initial encoding from base encoding
                        let mut map = a.map(0).ok_or_else(failure)?;
                        for code in 0u8..=255 {
                            if let Some(unicode_str) =
                                standard_encoding_lookup_with_budget(base_name, code, a)
                            {
                                // Convert the first character of the unicode string
                                if let Some(ch) = unicode_str.chars().next() {
                                    a.insert(&mut map, code, ch).ok_or_else(failure)?;
                                }
                            }
                        }
                        map
                    } else {
                        HashMap::new()
                    }
                } else if let Some(prog_enc) = font_program_encoding {
                    // PDF Spec ISO 32000-1:2008, Section 9.6.6.1:
                    // "If BaseEncoding is absent and the font has a built-in encoding,
                    // the built-in encoding shall be used as the base encoding."
                    let mut map = a.map(prog_enc.len()).ok_or_else(failure)?;
                    map.extend(prog_enc.iter().map(|(&code, &ch)| (code, ch)));
                    map
                } else {
                    // No base encoding specified and no font program - use StandardEncoding as default
                    let mut map = a.map(0).ok_or_else(failure)?;
                    for code in 0u8..=255 {
                        if let Some(unicode_str) =
                            standard_encoding_lookup_with_budget("StandardEncoding", code, a)
                        {
                            if let Some(ch) = unicode_str.chars().next() {
                                a.insert(&mut map, code, ch).ok_or_else(failure)?;
                            }
                        }
                    }
                    map
                };

            // Step 2: Apply /Differences array if present
            if let Some(differences_obj) = dict.get("Differences") {
                log::info!("Found /Differences array in encoding dictionary");

                // Resolve indirect reference for /Differences itself
                let resolved_diff = if let Some(obj_ref) = differences_obj.as_reference() {
                    doc.load_object(obj_ref).ok()
                } else {
                    None
                };
                let diff_obj = resolved_diff.as_ref().unwrap_or(differences_obj);

                if let Some(diff_array) = diff_obj.as_array() {
                    log::info!("/Differences array has {} items", diff_array.len());
                    let mut current_code: u32 = 0;

                    for item in diff_array {
                        // Resolve indirect references within the array
                        let resolved_item = if let Some(obj_ref) = item.as_reference() {
                            doc.load_object(obj_ref).ok()
                        } else {
                            None
                        };
                        let actual_item = resolved_item.as_ref().unwrap_or(item);

                        match actual_item {
                            Object::Integer(code) => {
                                // New starting code
                                current_code = *code as u32;
                            },
                            Object::Name(glyph_name) => {
                                // Retain the authoritative glyph name for this code
                                // (ISO 32000-1 §9.6.6.1, Table 114). Kept regardless
                                // of whether it resolves to a single/compound/unknown
                                // Unicode value, so the punctuation-recovery
                                // interceptions in `char_to_unicode` can consult it.
                                if current_code <= 255 {
                                    let name = a.string(glyph_name).ok_or_else(failure)?;
                                    a.insert(&mut diff_glyph_names, current_code as u8, name)
                                        .ok_or_else(failure)?;
                                }
                                // Map glyph name to Unicode character(s)
                                if let Some(unicode_char) = glyph_name_to_unicode(glyph_name) {
                                    if current_code <= 255 {
                                        a.insert(
                                            &mut encoding_map,
                                            current_code as u8,
                                            unicode_char,
                                        )
                                        .ok_or_else(failure)?;
                                        if is_ligature_char(unicode_char) {
                                            log::info!(
                                                "/Differences: code {} → /{} → '{}' (U+{:04X})",
                                                current_code,
                                                glyph_name,
                                                unicode_char,
                                                unicode_char as u32
                                            );
                                        }
                                    }
                                } else if let Some(unicode_string) =
                                    glyph_name_to_unicode_string_with_budget(glyph_name, a)
                                {
                                    // Compound glyph name (e.g. f_f → "ff", f_f_i → "ffi")
                                    if current_code <= 255 {
                                        let copied =
                                            a.string(&unicode_string).ok_or_else(failure)?;
                                        a.insert(&mut multi_char_map, current_code as u8, copied)
                                            .ok_or_else(failure)?;
                                        log::info!(
                                            "/Differences: code {} → /{} → {:?} (compound)",
                                            current_code,
                                            glyph_name,
                                            unicode_string
                                        );
                                    }
                                } else {
                                    log::debug!(
                                        "Unknown glyph name '{}' at code {} in /Differences array",
                                        glyph_name,
                                        current_code
                                    );
                                }
                                current_code += 1;
                            },
                            _ => {
                                // Invalid item in /Differences array - skip
                                log::warn!(
                                    "Unexpected item in /Differences array: {:?}",
                                    actual_item
                                );
                            },
                        }
                    }

                    log::debug!(
                        "Parsed /Differences array with {} custom mappings",
                        encoding_map.len()
                    );
                } else {
                    log::warn!("/Differences is not an array: {:?}", diff_obj);
                }
            }

            if !encoding_map.is_empty() || !multi_char_map.is_empty() {
                Ok((Encoding::Custom(encoding_map), multi_char_map, diff_glyph_names))
            } else {
                Ok((
                    Encoding::Standard(a.string("StandardEncoding").ok_or_else(failure)?),
                    HashMap::new(),
                    diff_glyph_names,
                ))
            }
        } else {
            Ok((
                Encoding::Standard(a.string("StandardEncoding").ok_or_else(failure)?),
                HashMap::new(),
                HashMap::new(),
            ))
        }
    }

    /// Map a character code to a Unicode string.
    ///
    /// Priority:
    /// 1. ToUnicode CMap (most accurate)
    /// 2. Built-in encoding
    /// 3. Symbol font encoding (for Symbol/ZapfDingbats fonts)
    /// 4. Ligature expansion (for ligature characters)
    /// 5. Identity mapping (as fallback)
    ///
    /// # Arguments
    ///
    /// * `char_code` - The character code from the PDF content stream
    ///
    /// # Returns
    ///
    /// The Unicode string for this character, or None if no mapping exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::fonts::FontInfo;
    /// # fn example(font: &FontInfo) {
    /// if let Some(unicode) = font.char_to_unicode(0x41) {
    ///     println!("Character: {}", unicode); // Should print "A"
    /// }
    /// # }
    /// ```
    /// Convert a character code to Unicode string.
    ///
    /// Per PDF Spec ISO 32000-1:2008, Section 9.10.2 "Mapping Character Codes to Unicode Values":
    ///
    /// Priority order (STRICTLY FOLLOWED):
    /// 1. ToUnicode CMap (if present) - highest priority, NO EXCEPTIONS
    /// 2. Predefined encodings for simple fonts with standard glyphs
    /// 3. Font descriptor's symbolic flag + built-in encoding (e.g., Symbol, ZapfDingbats)
    /// 4. Font's /Encoding + /Differences
    ///
    /// IMPORTANT: We do NOT apply heuristics to override ToUnicode. If the PDF has
    /// a buggy ToUnicode CMap, that is a PDF authoring error, not our responsibility
    /// to "fix" by guessing what the author meant.
    /// Get glyph width for a character code.
    ///
    /// Returns width in 1000ths of em (PDF units) per PDF Spec ISO 32000-1:2008, Section 9.7.4.
    /// Must be multiplied by (font_size / 1000) to get actual width in user space units.
    ///
    /// # Arguments
    ///
    /// * `char_code` - Character code from PDF content stream (e.g., byte value from Tj/TJ operator)
    ///
    /// # Returns
    ///
    /// Width in 1000ths of em. Returns `default_width` if the character code is not
    /// in the widths array or if widths are not available for this font.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pdf_oxide::fonts::FontInfo;
    ///
    /// # fn example(font: &FontInfo) {
    /// // Get width for character 'A' (code 65)
    /// let width = font.get_glyph_width(65);
    /// let font_size = 12.0;
    /// let actual_width = width * font_size / 1000.0;
    /// println!("Width of 'A' at 12pt: {:.2}pt", actual_width);
    /// # }
    /// ```
    pub fn get_glyph_width(&self, char_code: u16) -> f32 {
        // For Type0 (CID) fonts, use /W array then fall back to /DW (cid_default_width).
        // F15 fix: when /DW was NOT explicitly set (has_explicit_dw=false) and the char
        // code has no entry in /W, fall through to default_width instead of returning
        // the spec-default 1000.
        // NOTE: ISO 32000-1 §9.7.4 Table 117 specifies the default for a missing /DW
        // as 1000 units. This implementation intentionally deviates from that default
        // because many non-fullwidth CID fonts omit /DW; returning 1000 for their glyphs
        // over-estimates widths and disables the gap-correction heuristic. Purely
        // fullwidth CJK fonts that omit /DW may have glyph widths under-estimated as
        // a consequence — an acceptable trade-off for the common mixed-script case.
        if self.subtype == "Type0" {
            if let Some(cid_widths) = &self.cid_widths {
                if let Some(&width) = cid_widths.get(&char_code) {
                    return width;
                }
            }
            // Only use cid_default_width if /DW was explicitly present in the font dict.
            if self.has_explicit_dw {
                return self.cid_default_width;
            }
            // Fall through to default_width — same path as simple fonts without /Widths.
        }

        // For simple fonts, use the widths array
        if let Some(widths) = &self.widths {
            if let Some(first_char) = self.first_char {
                let index = char_code as i32 - first_char as i32;
                if index >= 0 && (index as usize) < widths.len() {
                    return widths[index as usize];
                }
            }
        }
        // For standard 14 fonts without /Widths, use built-in metrics
        if let Some(w) = self.get_standard_font_width(char_code) {
            return w;
        }
        self.default_width
    }

    /// Look up width from standard 14 font metrics when /Widths array is absent
    /// or the char code falls outside the [FirstChar, LastChar] range.
    fn get_standard_font_width(&self, char_code: u16) -> Option<f32> {
        // If a /Widths array covers this specific char code, trust it — don't override
        // with standard metrics. For chars OUTSIDE the range (including the common case
        // where space U+0020 = 32 is below a FirstChar like 66) we prefer named-font
        // metrics over the generic default_width (500), which is often too wide.
        if let Some(widths) = &self.widths {
            if let Some(first_char) = self.first_char {
                let index = char_code as i32 - first_char as i32;
                if index >= 0 && (index as usize) < widths.len() {
                    return None; // within explicit widths range – use actual width
                }
            }
        }
        // The name classification below is a pure function of `base_font`, but
        // this runs once per glyph — so it is resolved once and memoized.
        let std14 = (*self.std14_memo.get_or_init(|| self.classify_std14()))?;
        let is_bold = std14.is_bold;
        if std14.is_courier {
            return Some(600.0); // Monospace
        }
        let is_times = std14.is_times;
        let code = char_code as u8;
        if let Some(w) = self.winansi_punctuation_width(std14, is_times, is_bold, code) {
            return Some(w);
        }
        self.std14_width(std14, is_times, is_bold, code)
    }

    /// Standard-14 widths for the WinAnsi punctuation above ASCII.
    ///
    /// The per-family tables in [`Self::std14_width`] run from 32 to 126 and
    /// return `None` past that, so an en dash (150) or em dash (151) falls
    /// through to the generic `default_width` of 550/1000 em. Helvetica advances
    /// an em dash by a full em, so a caption reading
    /// `TABLE 66.01-11(5)-COORDINATES` measured 3.648 pt narrower than its ink
    /// — 3.600 from the em dash, 0.048 from the en dash — and the span ended
    /// short of the run beside it.
    ///
    /// ISO 32000-1:2008 §9.6.2.2 (docs/spec/pdf.md:17706) has the reader supply
    /// the metrics when a Standard-14 dictionary omits `/Widths`; §9.4.4
    /// (:17433) then spends them on the text matrix. Annex D.2 gives the code
    /// for each glyph. Values are the Adobe Core-14 AFM advances — the same
    /// argument `5d711c2e` made for completing these tables across ASCII.
    ///
    /// WinAnsiEncoding names these glyphs at these codes. StandardEncoding is
    /// admitted because Annex D.2 leaves 128..=159 unused there, so a byte in
    /// that range against a Standard-14 font is a producer writing CP1252 and
    /// the decoder already reads it that way. MacRomanEncoding *does* define
    /// that range and keeps the previous behaviour, as does any `/Differences`.
    fn winansi_punctuation_width(
        &self,
        std14: Std14Flags,
        is_times: bool,
        is_bold: bool,
        code: u8,
    ) -> Option<f32> {
        // The two encodings put these glyphs at different codes, so which
        // encoding is named decides which codes mean what. Annex D.2 lists
        // both: WinAnsi has the em dash at 151, StandardEncoding at 208.
        let is_italic = std14.is_italic || std14.is_bold_italic;
        let endash = if is_times { 500.0 } else { 556.0 };
        let emdash = if is_times && is_italic && !is_bold {
            889.0
        } else {
            1000.0
        };
        let ellipsis = emdash;
        let quotedbl = match (is_times, is_bold, is_italic) {
            (true, false, true) => 556.0,
            (true, true, true) => 500.0,
            (true, _, _) => 444.0,
            (false, true, _) => 500.0,
            (false, false, _) => 333.0,
        };
        let quotesingle = if is_times {
            333.0
        } else if is_bold {
            278.0
        } else {
            222.0
        };
        let Encoding::Standard(name) = &self.encoding else {
            return None;
        };
        Some(match (name.as_str(), code) {
            // WinAnsiEncoding: the CP1252 punctuation block.
            ("WinAnsiEncoding", 150) => endash,
            ("WinAnsiEncoding", 151) => emdash,
            ("WinAnsiEncoding", 133) => ellipsis,
            ("WinAnsiEncoding", 149) => 350.0,
            ("WinAnsiEncoding", 145 | 146) => quotesingle,
            ("WinAnsiEncoding", 147 | 148) => quotedbl,
            // StandardEncoding places the same glyphs elsewhere entirely.
            ("StandardEncoding", 177) => endash,
            ("StandardEncoding", 208) => emdash,
            ("StandardEncoding", 188) => ellipsis,
            ("StandardEncoding", 183) => 350.0,
            ("StandardEncoding", 170) => quotedbl,
            ("StandardEncoding", 186) => quotedbl,
            _ => return None,
        })
    }

    /// Classify `base_font` against the Standard-14 set (ISO 32000-1 Annex D).
    /// `None` when the font is not one of the width-bearing standard families.
    fn classify_std14(&self) -> Option<Std14Flags> {
        // F13 fix: use exact match against the canonical 14 standard PDF font names
        // after stripping any SUBSET+ prefix (e.g. "ABCDEF+Helvetica" → "Helvetica").
        // `contains` would incorrectly match "HelveticaCorp-Custom" as Helvetica.
        let raw_name = &self.base_font;
        let name: &str = if let Some(idx) = raw_name.find('+') {
            // Strip subset prefix: the part after '+' is the actual font name
            let suffix = &raw_name[idx + 1..];
            if suffix.is_empty() {
                raw_name
            } else {
                suffix
            }
        } else {
            raw_name
        };
        // Canonical Standard-14 font names per ISO 32000-1 Annex D.
        // "Helvetica-Oblique" is the name used by virtually all real-world PDFs;
        // the spec's canonical PostScript name is "HelveticaOblique" (no hyphen).
        // Both are accepted.
        const STANDARD_14: &[&str] = &[
            "Courier",
            "Courier-Bold",
            "Courier-BoldOblique",
            "Courier-Oblique",
            "Helvetica",
            "Helvetica-Bold",
            "Helvetica-BoldOblique",
            "Helvetica-Oblique",
            "HelveticaOblique",
            "Times-Roman",
            "Times-Bold",
            "Times-BoldItalic",
            "Times-Italic",
            "Symbol",
            "ZapfDingbats",
        ];
        if !STANDARD_14.contains(&name) {
            return None;
        }
        let is_times = name.starts_with("Times");
        let is_helvetica = name.starts_with("Helvetica");
        let is_courier = name.starts_with("Courier");

        if !is_times && !is_helvetica && !is_courier {
            return None;
        }

        Some(Std14Flags {
            is_times,
            is_courier,
            is_bold: name.contains("Bold"),
            is_bold_italic: name.contains("BoldItalic"),
            is_helvetica,
            is_italic: name.contains("Italic"),
        })
    }

    /// Standard-14 width tables, keyed off the memoized classification.
    fn std14_width(
        &self,
        std14: Std14Flags,
        is_times: bool,
        is_bold: bool,
        code: u8,
    ) -> Option<f32> {
        // Times-Roman / Times-Bold / Times-BoldItalic standard widths (Adobe AFM metrics)
        if is_times {
            if std14.is_bold_italic {
                // Times-BoldItalic widths (Adobe Core 14 Fonts AFM).
                return Some(match code {
                    32 => 250.0,
                    33 => 389.0,
                    34 => 555.0,
                    35 => 500.0,
                    36 => 500.0,
                    37 => 833.0,
                    38 => 778.0,
                    39 => 333.0,
                    40 => 333.0,
                    41 => 333.0,
                    42 => 500.0,
                    43 => 570.0,
                    44 => 250.0,
                    45 => 333.0,
                    46 => 250.0,
                    47 => 278.0,
                    48..=57 => 500.0,
                    58 => 333.0,
                    59 => 333.0,
                    60 => 570.0,
                    61 => 570.0,
                    62 => 570.0,
                    63 => 500.0,
                    64 => 832.0,
                    65 => 667.0,
                    66 => 667.0,
                    67 => 667.0,
                    68 => 722.0,
                    69 => 667.0,
                    70 => 667.0,
                    71 => 722.0,
                    72 => 778.0,
                    73 => 389.0,
                    74 => 500.0,
                    75 => 667.0,
                    76 => 611.0,
                    77 => 889.0,
                    78 => 722.0,
                    79 => 722.0,
                    80 => 611.0,
                    81 => 722.0,
                    82 => 667.0,
                    83 => 556.0,
                    84 => 611.0,
                    85 => 722.0,
                    86 => 667.0,
                    87 => 889.0,
                    88 => 667.0,
                    89 => 611.0,
                    90 => 611.0,
                    91 => 333.0,
                    92 => 278.0,
                    93 => 333.0,
                    94 => 570.0,
                    95 => 500.0,
                    96 => 333.0, // StandardEncoding: quoteleft, not grave
                    97 => 500.0,
                    98 => 500.0,
                    99 => 444.0,
                    100 => 500.0,
                    101 => 444.0,
                    102 => 333.0,
                    103 => 500.0,
                    104 => 556.0,
                    105 => 278.0,
                    106 => 278.0,
                    107 => 500.0,
                    108 => 278.0,
                    109 => 778.0,
                    110 => 556.0,
                    111 => 500.0,
                    112 => 500.0,
                    113 => 500.0,
                    114 => 389.0,
                    115 => 389.0,
                    116 => 278.0,
                    117 => 556.0,
                    118 => 444.0,
                    119 => 667.0,
                    120 => 500.0,
                    121 => 444.0,
                    122 => 389.0,
                    123 => 348.0,
                    124 => 220.0,
                    125 => 348.0,
                    126 => 570.0,
                    _ => return None,
                });
            }
            if is_bold {
                // Times-Bold widths (Adobe Core 14 Fonts AFM).
                return Some(match code {
                    32 => 250.0,
                    33 => 333.0,
                    34 => 555.0,
                    35 => 500.0,
                    36 => 500.0,
                    37 => 1000.0,
                    38 => 833.0,
                    39 => 333.0,
                    40 => 333.0,
                    41 => 333.0,
                    42 => 500.0,
                    43 => 570.0,
                    44 => 250.0,
                    45 => 333.0,
                    46 => 250.0,
                    47 => 278.0,
                    48..=57 => 500.0,
                    58 => 333.0,
                    59 => 333.0,
                    60 => 570.0,
                    61 => 570.0,
                    62 => 570.0,
                    63 => 500.0,
                    64 => 930.0,
                    65 => 722.0,
                    66 => 667.0,
                    67 => 722.0,
                    68 => 722.0,
                    69 => 667.0,
                    70 => 611.0,
                    71 => 778.0,
                    72 => 778.0,
                    73 => 389.0,
                    74 => 500.0,
                    75 => 778.0,
                    76 => 667.0,
                    77 => 944.0,
                    78 => 722.0,
                    79 => 778.0,
                    80 => 611.0,
                    81 => 778.0,
                    82 => 722.0,
                    83 => 556.0,
                    84 => 667.0,
                    85 => 722.0,
                    86 => 722.0,
                    87 => 1000.0,
                    88 => 722.0,
                    89 => 722.0,
                    90 => 667.0,
                    91 => 333.0,
                    92 => 278.0,
                    93 => 333.0,
                    94 => 581.0,
                    95 => 500.0,
                    96 => 333.0, // StandardEncoding: quoteleft, not grave
                    97 => 500.0,
                    98 => 556.0,
                    99 => 444.0,
                    100 => 556.0,
                    101 => 444.0,
                    102 => 333.0,
                    103 => 500.0,
                    104 => 556.0,
                    105 => 278.0,
                    106 => 333.0,
                    107 => 556.0,
                    108 => 278.0,
                    109 => 833.0,
                    110 => 556.0,
                    111 => 500.0,
                    112 => 556.0,
                    113 => 556.0,
                    114 => 444.0,
                    115 => 389.0,
                    116 => 333.0,
                    117 => 556.0,
                    118 => 500.0,
                    119 => 722.0,
                    120 => 500.0,
                    121 => 500.0,
                    122 => 444.0,
                    123 => 394.0,
                    124 => 220.0,
                    125 => 394.0,
                    126 => 520.0,
                    _ => return None,
                });
            }
            if std14.is_italic {
                // Times-Italic widths (Adobe Core 14 Fonts AFM).
                return Some(match code {
                    32 => 250.0,
                    33 => 333.0,
                    34 => 420.0,
                    35 => 500.0,
                    36 => 500.0,
                    37 => 833.0,
                    38 => 778.0,
                    39 => 333.0,
                    40 => 333.0,
                    41 => 333.0,
                    42 => 500.0,
                    43 => 675.0,
                    44 => 250.0,
                    45 => 333.0,
                    46 => 250.0,
                    47 => 278.0,
                    48..=57 => 500.0,
                    58 => 333.0,
                    59 => 333.0,
                    60 => 675.0,
                    61 => 675.0,
                    62 => 675.0,
                    63 => 500.0,
                    64 => 920.0,
                    65 => 611.0,
                    66 => 611.0,
                    67 => 667.0,
                    68 => 722.0,
                    69 => 611.0,
                    70 => 611.0,
                    71 => 722.0,
                    72 => 722.0,
                    73 => 333.0,
                    74 => 444.0,
                    75 => 667.0,
                    76 => 556.0,
                    77 => 833.0,
                    78 => 667.0,
                    79 => 722.0,
                    80 => 611.0,
                    81 => 722.0,
                    82 => 611.0,
                    83 => 500.0,
                    84 => 556.0,
                    85 => 722.0,
                    86 => 611.0,
                    87 => 833.0,
                    88 => 611.0,
                    89 => 556.0,
                    90 => 556.0,
                    91 => 389.0,
                    92 => 278.0,
                    93 => 389.0,
                    94 => 422.0,
                    95 => 500.0,
                    96 => 333.0, // StandardEncoding: quoteleft, not grave
                    97 => 500.0,
                    98 => 500.0,
                    99 => 444.0,
                    100 => 500.0,
                    101 => 444.0,
                    102 => 278.0,
                    103 => 500.0,
                    104 => 500.0,
                    105 => 278.0,
                    106 => 278.0,
                    107 => 444.0,
                    108 => 278.0,
                    109 => 722.0,
                    110 => 500.0,
                    111 => 500.0,
                    112 => 500.0,
                    113 => 500.0,
                    114 => 389.0,
                    115 => 389.0,
                    116 => 278.0,
                    117 => 500.0,
                    118 => 444.0,
                    119 => 667.0,
                    120 => 444.0,
                    121 => 444.0,
                    122 => 389.0,
                    123 => 400.0,
                    124 => 275.0,
                    125 => 400.0,
                    126 => 541.0,
                    _ => return None,
                });
            }
            return Some(match code {
                32 => 250.0,
                33 => 333.0,
                34 => 408.0,
                35 => 500.0,
                36 => 500.0,
                37 => 833.0,
                38 => 778.0,
                39 => 333.0,
                40 => 333.0,
                41 => 333.0,
                42 => 500.0,
                43 => 564.0,
                44 => 250.0,
                45 => 333.0,
                46 => 250.0,
                47 => 278.0,
                48 => 500.0,
                49 => 500.0,
                50 => 500.0,
                51 => 500.0,
                52 => 500.0,
                53 => 500.0,
                54 => 500.0,
                55 => 500.0,
                56 => 500.0,
                57 => 500.0,
                58 => 278.0,
                59 => 278.0,
                60 => 564.0,
                61 => 564.0,
                62 => 564.0,
                63 => 444.0,
                64 => 921.0,
                65 => 722.0,
                66 => 667.0,
                67 => 667.0,
                68 => 722.0,
                69 => 611.0,
                70 => 556.0,
                71 => 722.0,
                72 => 722.0,
                73 => 333.0,
                74 => 389.0,
                75 => 722.0,
                76 => 611.0,
                77 => 889.0,
                78 => 722.0,
                79 => 722.0,
                80 => 556.0,
                81 => 722.0,
                82 => 667.0,
                83 => 556.0,
                84 => 611.0,
                85 => 722.0,
                86 => 722.0,
                87 => 944.0,
                88 => 722.0,
                89 => 722.0,
                90 => 611.0,
                91 => 333.0,
                92 => 278.0,
                93 => 333.0,
                94 => 469.0,
                95 => 500.0,
                96 => 333.0, // StandardEncoding: quoteleft, not grave
                97 => 444.0,
                98 => 500.0,
                99 => 444.0,
                100 => 500.0,
                101 => 444.0,
                102 => 333.0,
                103 => 500.0,
                104 => 500.0,
                105 => 278.0,
                106 => 278.0,
                107 => 500.0,
                108 => 278.0,
                109 => 778.0,
                110 => 500.0,
                111 => 500.0,
                112 => 500.0,
                113 => 500.0,
                114 => 333.0,
                115 => 389.0,
                116 => 278.0,
                117 => 500.0,
                118 => 500.0,
                119 => 722.0,
                120 => 500.0,
                121 => 500.0,
                122 => 444.0,
                123 => 480.0,
                124 => 200.0,
                125 => 480.0,
                126 => 541.0,
                _ => return None,
            });
        }

        // Helvetica / Helvetica-Bold standard widths (Adobe AFM metrics)
        if std14.is_helvetica {
            if is_bold {
                // Helvetica-Bold / Helvetica-BoldOblique widths (Adobe Core 14 Fonts AFM).
                return Some(match code {
                    32 => 278.0,
                    33 => 333.0,
                    34 => 474.0,
                    35 => 556.0,
                    36 => 556.0,
                    37 => 889.0,
                    38 => 722.0,
                    39 => 278.0, // StandardEncoding: quoteright, not quotesingle
                    40 => 333.0,
                    41 => 333.0,
                    42 => 389.0,
                    43 => 584.0,
                    44 => 278.0,
                    45 => 333.0,
                    46 => 278.0,
                    47 => 278.0,
                    48..=57 => 556.0,
                    58 => 333.0,
                    59 => 333.0,
                    60 => 584.0,
                    61 => 584.0,
                    62 => 584.0,
                    63 => 611.0,
                    64 => 975.0,
                    65 => 722.0,
                    66 => 722.0,
                    67 => 722.0,
                    68 => 722.0,
                    69 => 667.0,
                    70 => 611.0,
                    71 => 778.0,
                    72 => 722.0,
                    73 => 278.0,
                    74 => 556.0,
                    75 => 722.0,
                    76 => 611.0,
                    77 => 833.0,
                    78 => 722.0,
                    79 => 778.0,
                    80 => 667.0,
                    81 => 778.0,
                    82 => 722.0,
                    83 => 667.0,
                    84 => 611.0,
                    85 => 722.0,
                    86 => 667.0,
                    87 => 944.0,
                    88 => 667.0,
                    89 => 667.0,
                    90 => 611.0,
                    91 => 333.0,
                    92 => 278.0,
                    93 => 333.0,
                    94 => 584.0,
                    95 => 556.0,
                    96 => 278.0, // StandardEncoding: quoteleft, not grave
                    97 => 556.0,
                    98 => 611.0,
                    99 => 556.0,
                    100 => 611.0,
                    101 => 556.0,
                    102 => 333.0,
                    103 => 611.0,
                    104 => 611.0,
                    105 => 278.0,
                    106 => 278.0,
                    107 => 556.0,
                    108 => 278.0,
                    109 => 889.0,
                    110 => 611.0,
                    111 => 611.0,
                    112 => 611.0,
                    113 => 611.0,
                    114 => 389.0,
                    115 => 556.0,
                    116 => 333.0,
                    117 => 611.0,
                    118 => 556.0,
                    119 => 778.0,
                    120 => 556.0,
                    121 => 556.0,
                    122 => 500.0,
                    123 => 389.0,
                    124 => 280.0,
                    125 => 389.0,
                    126 => 584.0,
                    _ => return None,
                });
            }
            return Some(match code {
                32 => 278.0,
                33 => 278.0,
                34 => 355.0,
                35 => 556.0,
                36 => 556.0,
                37 => 889.0,
                38 => 667.0,
                39 => 222.0, // StandardEncoding: quoteright, not quotesingle
                40 => 333.0,
                41 => 333.0,
                42 => 389.0,
                43 => 584.0,
                44 => 278.0,
                45 => 333.0,
                46 => 278.0,
                47 => 278.0,
                48..=57 => 556.0, // digits
                58 => 278.0,
                59 => 278.0,
                60 => 584.0,
                61 => 584.0,
                62 => 584.0,
                63 => 556.0,
                64 => 1015.0,
                65 => 667.0,
                66 => 667.0,
                67 => 722.0,
                68 => 722.0,
                69 => 667.0,
                70 => 611.0,
                71 => 778.0,
                72 => 722.0,
                73 => 278.0,
                74 => 500.0,
                75 => 667.0,
                76 => 556.0,
                77 => 833.0,
                78 => 722.0,
                79 => 778.0,
                80 => 667.0,
                81 => 778.0,
                82 => 722.0,
                83 => 667.0,
                84 => 611.0,
                85 => 722.0,
                86 => 667.0,
                87 => 944.0,
                88 => 667.0,
                89 => 667.0,
                90 => 611.0,
                91 => 278.0,
                92 => 278.0,
                93 => 278.0,
                94 => 469.0,
                95 => 556.0,
                96 => 222.0, // StandardEncoding: quoteleft, not grave
                97 => 556.0,
                98 => 556.0,
                99 => 500.0,
                100 => 556.0,
                101 => 556.0,
                102 => 278.0,
                103 => 556.0,
                104 => 556.0,
                105 => 222.0,
                106 => 222.0,
                107 => 500.0,
                108 => 222.0,
                109 => 833.0,
                110 => 556.0,
                111 => 556.0,
                112 => 556.0,
                113 => 556.0,
                114 => 333.0,
                115 => 500.0,
                116 => 278.0,
                117 => 556.0,
                118 => 500.0,
                119 => 722.0,
                120 => 500.0,
                121 => 500.0,
                122 => 500.0, // Adobe AFM: Helvetica z is 500; 444 is the Times value
                123 => 334.0,
                124 => 260.0,
                125 => 334.0,
                126 => 584.0,
                _ => return None,
            });
        }
        None
    }

    /// Get the width of the space glyph (U+0020) in font units.
    ///
    /// Returns the width in 1000ths of em per PDF spec Section 9.7.4.
    /// Used for font-aware spacing threshold calculations.
    ///
    /// Per PDF Spec Section 9.4.4, word spacing should be based on actual font metrics
    /// rather than fixed ratios. This method returns the actual space glyph width,
    /// which is used to compute adaptive TJ offset thresholds that account for
    /// different font sizes and families.
    ///
    /// # Returns
    ///
    /// The width of the space character (code 0x20) in 1000ths of em. When no
    /// real space glyph is defined — a simple font with a near-zero 0x20, or a
    /// CID font with no explicit /W entry for 0x20 — returns the 0.25 em (250)
    /// typographic default rather than the font's (often much wider) /DW.
    pub fn get_space_glyph_width(&self) -> f32 {
        // The space advance feeds the caller's geometric word-gap threshold
        // (threshold = space_width × ratio); a value that is not actually the
        // space glyph's advance skews that threshold and mis-detects word
        // boundaries.
        //
        // Type0 (CID-keyed) fonts under Identity-H/V — the encoding of nearly
        // every embedded subset — map character code 0x20 to CID 32, an
        // arbitrary font-internal glyph, NOT the space. The space glyph, if
        // present, lives at a CID reached through the font's CMap / ToUnicode,
        // never at code 0x20 (ISO 32000-2 §9.7.5.2, §9.10.2). So `cid_widths`
        // keyed by 0x20 is the advance of whatever glyph sits at CID 32 —
        // frequently ~0.5 em+ (TimesNewRomanPSMT reports 563) — and feeding it
        // into the threshold makes it so wide that real justified word gaps
        // fall below it and adjacent words glue together ("All rights reserved"
        // -> "Allrightsreserved", #803). For Identity-encoded Type0 fonts,
        // ignore code 0x20 entirely and use the 0.25 em typographic default.
        if self.subtype == "Type0" {
            if matches!(self.encoding, Encoding::Identity) {
                return 250.0;
            }
            // Non-Identity predefined CMap (e.g. 90ms-RKSJ-H): code 0x20 can map
            // to a real space CID, so an explicit /W entry is meaningful.
            return match self.cid_widths.as_ref().and_then(|w| w.get(&0x20)) {
                Some(&w) if w >= 50.0 => w,
                _ => 250.0,
            };
        }
        // Space character is always code 0x20 (32) in a simple font.
        let w = self.get_glyph_width(0x20);
        // Many simple subset fonts (notably shaped Arabic from Chrome /
        // browser print) omit a glyph for code 0x20 entirely, so this returns
        // ~0. A zero width collapses the threshold to 0, so *every* inter-glyph
        // kerning gap is read as a word boundary and cursive Arabic words
        // shatter into single letters. Fall back to a typographic
        // default of 0.25 em (250 font units) — the same value
        // `should_insert_space` uses when the font is absent.
        if w < 50.0 {
            250.0
        } else {
            w
        }
    }

    /// Map a Glyph ID (GID) to a standard PostScript glyph name.
    ///
    /// This is used as a fallback for Type0 fonts without ToUnicode CMaps.
    /// For ASCII range GIDs (32-126), maps to standard PostScript glyph names
    /// that can be looked up in the Adobe Glyph List.
    ///
    /// Phase 1.2: Adobe Glyph List Fallback
    ///
    /// # Arguments
    ///
    /// * `gid` - The Glyph ID to map (typically 0x20-0x7E for ASCII)
    ///
    /// # Returns
    ///
    /// The standard glyph name if GID is in the ASCII range, None otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(FontInfo::gid_to_standard_glyph_name(0x41), Some("A"));
    /// assert_eq!(FontInfo::gid_to_standard_glyph_name(0x20), Some("space"));
    /// assert_eq!(FontInfo::gid_to_standard_glyph_name(0xFFFF), None);
    /// ```
    pub fn gid_to_standard_glyph_name(gid: u16) -> Option<&'static str> {
        // Map GIDs to standard PostScript glyph names across multiple ranges:
        // - ASCII printable range (0x20-0x7E)
        // - Extended Latin / Windows-1252 range (0x80-0xFF)
        // - Latin-1 Supplement range (0xA0-0xFF)
        match gid {
            // Control characters and whitespace (32-33)
            0x20 => Some("space"),
            0x21 => Some("exclam"),
            0x22 => Some("quotedbl"),
            0x23 => Some("numbersign"),
            0x24 => Some("dollar"),
            0x25 => Some("percent"),
            0x26 => Some("ampersand"),
            0x27 => Some("quoteright"),
            0x28 => Some("parenleft"),
            0x29 => Some("parenright"),
            0x2A => Some("asterisk"),
            0x2B => Some("plus"),
            0x2C => Some("comma"),
            0x2D => Some("hyphen"),
            0x2E => Some("period"),
            0x2F => Some("slash"),
            // Digits (48-57)
            0x30 => Some("zero"),
            0x31 => Some("one"),
            0x32 => Some("two"),
            0x33 => Some("three"),
            0x34 => Some("four"),
            0x35 => Some("five"),
            0x36 => Some("six"),
            0x37 => Some("seven"),
            0x38 => Some("eight"),
            0x39 => Some("nine"),
            // Punctuation (58-64)
            0x3A => Some("colon"),
            0x3B => Some("semicolon"),
            0x3C => Some("less"),
            0x3D => Some("equal"),
            0x3E => Some("greater"),
            0x3F => Some("question"),
            0x40 => Some("at"),
            // Uppercase letters (65-90)
            0x41 => Some("A"),
            0x42 => Some("B"),
            0x43 => Some("C"),
            0x44 => Some("D"),
            0x45 => Some("E"),
            0x46 => Some("F"),
            0x47 => Some("G"),
            0x48 => Some("H"),
            0x49 => Some("I"),
            0x4A => Some("J"),
            0x4B => Some("K"),
            0x4C => Some("L"),
            0x4D => Some("M"),
            0x4E => Some("N"),
            0x4F => Some("O"),
            0x50 => Some("P"),
            0x51 => Some("Q"),
            0x52 => Some("R"),
            0x53 => Some("S"),
            0x54 => Some("T"),
            0x55 => Some("U"),
            0x56 => Some("V"),
            0x57 => Some("W"),
            0x58 => Some("X"),
            0x59 => Some("Y"),
            0x5A => Some("Z"),
            // Brackets (91-96)
            0x5B => Some("bracketleft"),
            0x5C => Some("backslash"),
            0x5D => Some("bracketright"),
            0x5E => Some("asciicircum"),
            0x5F => Some("underscore"),
            0x60 => Some("quoteleft"),
            // Lowercase letters (97-122)
            0x61 => Some("a"),
            0x62 => Some("b"),
            0x63 => Some("c"),
            0x64 => Some("d"),
            0x65 => Some("e"),
            0x66 => Some("f"),
            0x67 => Some("g"),
            0x68 => Some("h"),
            0x69 => Some("i"),
            0x6A => Some("j"),
            0x6B => Some("k"),
            0x6C => Some("l"),
            0x6D => Some("m"),
            0x6E => Some("n"),
            0x6F => Some("o"),
            0x70 => Some("p"),
            0x71 => Some("q"),
            0x72 => Some("r"),
            0x73 => Some("s"),
            0x74 => Some("t"),
            0x75 => Some("u"),
            0x76 => Some("v"),
            0x77 => Some("w"),
            0x78 => Some("x"),
            0x79 => Some("y"),
            0x7A => Some("z"),
            // Braces (123-126)
            0x7B => Some("braceleft"),
            0x7C => Some("bar"),
            0x7D => Some("braceright"),
            0x7E => Some("asciitilde"),

            // ==================================================================================
            // Extended Latin / Windows-1252 range (0x80-0xFF)
            // ==================================================================================
            // These mappings cover the extended ASCII characters commonly found in Western
            // European PDFs. When a Type0 font with Identity CMap encounters these GIDs,
            // we map them to their standard PostScript glyph names for AGL lookup.
            //
            // Per PDF Spec ISO 32000-1:2008 Section 9.10.2, when ToUnicode CMap is unavailable,
            // readers may use glyph name lookup as a fallback mechanism.

            // 0x80-0x8F: Windows-1252 extended control characters and symbols
            0x80 => Some("euro"), // U+20AC (Euro sign)
            // 0x81: undefined in Windows-1252
            0x82 => Some("quotesinglbase"), // U+201A (Single low quotation mark)
            0x83 => Some("florin"),         // U+0192 (Latin small letter f with hook)
            0x84 => Some("quotedblbase"),   // U+201E (Double low quotation mark)
            0x85 => Some("ellipsis"),       // U+2026 (Horizontal ellipsis)
            0x86 => Some("dagger"),         // U+2020 (Dagger)
            0x87 => Some("daggerdbl"),      // U+2021 (Double dagger)
            0x88 => Some("circumflex"),     // U+02C6 (Modifier letter circumflex accent)
            0x89 => Some("perthousand"),    // U+2030 (Per mille sign)
            0x8A => Some("Scaron"),         // U+0160 (Latin capital letter S with caron)
            0x8B => Some("guilsinglleft"),  // U+2039 (Single left-pointing angle quotation mark)
            0x8C => Some("OE"),             // U+0152 (Latin capital ligature OE)
            // 0x8D: undefined in Windows-1252
            0x8E => Some("Zcaron"), // U+017D (Latin capital letter Z with caron)
            // 0x8F: undefined in Windows-1252

            // 0x90-0x9F: Windows-1252 smart quotes, dashes, and accents
            // 0x90: undefined in Windows-1252
            0x91 => Some("quoteleft"), // U+2018 (Left single quotation mark)
            0x92 => Some("quoteright"), // U+2019 (Right single quotation mark)
            0x93 => Some("quotedblleft"), // U+201C (Left double quotation mark)
            0x94 => Some("quotedblright"), // U+201D (Right double quotation mark)
            0x95 => Some("bullet"),    // U+2022 (Bullet)
            0x96 => Some("endash"),    // U+2013 (En dash)
            0x97 => Some("emdash"),    // U+2014 (Em dash)
            0x98 => Some("tilde"),     // U+02DC (Small tilde)
            0x99 => Some("trademark"), // U+2122 (Trade mark sign)
            0x9A => Some("scaron"),    // U+0161 (Latin small letter s with caron)
            0x9B => Some("guilsinglright"), // U+203A (Single right-pointing angle quotation mark)
            0x9C => Some("oe"),        // U+0153 (Latin small ligature oe)
            // 0x9D: undefined in Windows-1252
            0x9E => Some("zcaron"), // U+017E (Latin small letter z with caron)
            0x9F => Some("Ydieresis"), // U+0178 (Latin capital letter Y with diaeresis)

            // 0xA0-0xFF: Latin-1 Supplement (ISO 8859-1)
            // Most of these are direct character mappings (À-ÿ)
            // Implement programmatically for common characters and fallback to glyph name generation
            0xA0 => Some("space"),          // U+00A0 (No-break space)
            0xA1 => Some("exclamdown"),     // U+00A1 (Inverted exclamation mark)
            0xA2 => Some("cent"),           // U+00A2 (Cent sign)
            0xA3 => Some("sterling"),       // U+00A3 (Pound sign)
            0xA4 => Some("currency"),       // U+00A4 (Currency sign)
            0xA5 => Some("yen"),            // U+00A5 (Yen sign)
            0xA6 => Some("brokenbar"),      // U+00A6 (Broken bar)
            0xA7 => Some("section"),        // U+00A7 (Section sign)
            0xA8 => Some("dieresis"),       // U+00A8 (Diaeresis)
            0xA9 => Some("copyright"),      // U+00A9 (Copyright sign)
            0xAA => Some("ordfeminine"),    // U+00AA (Feminine ordinal indicator)
            0xAB => Some("guillemotleft"),  // U+00AB (Left-pointing double angle quotation mark)
            0xAC => Some("logicalnot"),     // U+00AC (Not sign)
            0xAD => Some("uni00AD"),        // U+00AD (Soft hyphen)
            0xAE => Some("registered"),     // U+00AE (Registered sign)
            0xAF => Some("macron"),         // U+00AF (Macron)
            0xB0 => Some("degree"),         // U+00B0 (Degree sign)
            0xB1 => Some("plusminus"),      // U+00B1 (Plus-minus sign)
            0xB2 => Some("twosuperior"),    // U+00B2 (Superscript two)
            0xB3 => Some("threesuperior"),  // U+00B3 (Superscript three)
            0xB4 => Some("acute"),          // U+00B4 (Acute accent)
            0xB5 => Some("mu"),             // U+00B5 (Micro sign)
            0xB6 => Some("paragraph"),      // U+00B6 (Pilcrow)
            0xB7 => Some("middot"),         // U+00B7 (Middle dot)
            0xB8 => Some("cedilla"),        // U+00B8 (Cedilla)
            0xB9 => Some("onesuperior"),    // U+00B9 (Superscript one)
            0xBA => Some("ordmasculine"),   // U+00BA (Masculine ordinal indicator)
            0xBB => Some("guillemotright"), // U+00BB (Right-pointing double angle quotation mark)
            0xBC => Some("onequarter"),     // U+00BC (Vulgar fraction one quarter)
            0xBD => Some("onehalf"),        // U+00BD (Vulgar fraction one half)
            0xBE => Some("threequarters"),  // U+00BE (Vulgar fraction three quarters)
            0xBF => Some("questiondown"),   // U+00BF (Inverted question mark)

            // 0xC0-0xFE: Latin-1 Supplement letters (À-þ)
            // These map directly to their Unicode equivalents via standard PostScript names
            // Format: glyph name is the Unicode character itself (e.g., "Agrave" for U+00C0)
            0xC0 => Some("Agrave"), // U+00C0 (Latin capital letter A with grave)
            0xC1 => Some("Aacute"), // U+00C1 (Latin capital letter A with acute)
            0xC2 => Some("Acircumflex"), // U+00C2 (Latin capital letter A with circumflex)
            0xC3 => Some("Atilde"), // U+00C3 (Latin capital letter A with tilde)
            0xC4 => Some("Adieresis"), // U+00C4 (Latin capital letter A with diaeresis)
            0xC5 => Some("Aring"),  // U+00C5 (Latin capital letter A with ring above)
            0xC6 => Some("AE"),     // U+00C6 (Latin capital letter AE)
            0xC7 => Some("Ccedilla"), // U+00C7 (Latin capital letter C with cedilla)
            0xC8 => Some("Egrave"), // U+00C8 (Latin capital letter E with grave)
            0xC9 => Some("Eacute"), // U+00C9 (Latin capital letter E with acute)
            0xCA => Some("Ecircumflex"), // U+00CA (Latin capital letter E with circumflex)
            0xCB => Some("Edieresis"), // U+00CB (Latin capital letter E with diaeresis)
            0xCC => Some("Igrave"), // U+00CC (Latin capital letter I with grave)
            0xCD => Some("Iacute"), // U+00CD (Latin capital letter I with acute)
            0xCE => Some("Icircumflex"), // U+00CE (Latin capital letter I with circumflex)
            0xCF => Some("Idieresis"), // U+00CF (Latin capital letter I with diaeresis)
            0xD0 => Some("Eth"),    // U+00D0 (Latin capital letter Eth)
            0xD1 => Some("Ntilde"), // U+00D1 (Latin capital letter N with tilde)
            0xD2 => Some("Ograve"), // U+00D2 (Latin capital letter O with grave)
            0xD3 => Some("Oacute"), // U+00D3 (Latin capital letter O with acute)
            0xD4 => Some("Ocircumflex"), // U+00D4 (Latin capital letter O with circumflex)
            0xD5 => Some("Otilde"), // U+00D5 (Latin capital letter O with tilde)
            0xD6 => Some("Odieresis"), // U+00D6 (Latin capital letter O with diaeresis)
            0xD7 => Some("multiply"), // U+00D7 (Multiplication sign)
            0xD8 => Some("Oslash"), // U+00D8 (Latin capital letter O with stroke)
            0xD9 => Some("Ugrave"), // U+00D9 (Latin capital letter U with grave)
            0xDA => Some("Uacute"), // U+00DA (Latin capital letter U with acute)
            0xDB => Some("Ucircumflex"), // U+00DB (Latin capital letter U with circumflex)
            0xDC => Some("Udieresis"), // U+00DC (Latin capital letter U with diaeresis)
            0xDD => Some("Yacute"), // U+00DD (Latin capital letter Y with acute)
            0xDE => Some("Thorn"),  // U+00DE (Latin capital letter Thorn)
            0xDF => Some("germandbls"), // U+00DF (Latin small letter sharp s)
            0xE0 => Some("agrave"), // U+00E0 (Latin small letter a with grave)
            0xE1 => Some("aacute"), // U+00E1 (Latin small letter a with acute)
            0xE2 => Some("acircumflex"), // U+00E2 (Latin small letter a with circumflex)
            0xE3 => Some("atilde"), // U+00E3 (Latin small letter a with tilde)
            0xE4 => Some("adieresis"), // U+00E4 (Latin small letter a with diaeresis)
            0xE5 => Some("aring"),  // U+00E5 (Latin small letter a with ring above)
            0xE6 => Some("ae"),     // U+00E6 (Latin small letter ae)
            0xE7 => Some("ccedilla"), // U+00E7 (Latin small letter c with cedilla)
            0xE8 => Some("egrave"), // U+00E8 (Latin small letter e with grave)
            0xE9 => Some("eacute"), // U+00E9 (Latin small letter e with acute)
            0xEA => Some("ecircumflex"), // U+00EA (Latin small letter e with circumflex)
            0xEB => Some("edieresis"), // U+00EB (Latin small letter e with diaeresis)
            0xEC => Some("igrave"), // U+00EC (Latin small letter i with grave)
            0xED => Some("iacute"), // U+00ED (Latin small letter i with acute)
            0xEE => Some("icircumflex"), // U+00EE (Latin small letter i with circumflex)
            0xEF => Some("idieresis"), // U+00EF (Latin small letter i with diaeresis)
            0xF0 => Some("eth"),    // U+00F0 (Latin small letter eth)
            0xF1 => Some("ntilde"), // U+00F1 (Latin small letter n with tilde)
            0xF2 => Some("ograve"), // U+00F2 (Latin small letter o with grave)
            0xF3 => Some("oacute"), // U+00F3 (Latin small letter o with acute)
            0xF4 => Some("ocircumflex"), // U+00F4 (Latin small letter o with circumflex)
            0xF5 => Some("otilde"), // U+00F5 (Latin small letter o with tilde)
            0xF6 => Some("odieresis"), // U+00F6 (Latin small letter o with diaeresis)
            0xF7 => Some("divide"), // U+00F7 (Division sign)
            0xF8 => Some("oslash"), // U+00F8 (Latin small letter o with stroke)
            0xF9 => Some("ugrave"), // U+00F9 (Latin small letter u with grave)
            0xFA => Some("uacute"), // U+00FA (Latin small letter u with acute)
            0xFB => Some("ucircumflex"), // U+00FB (Latin small letter u with circumflex)
            0xFC => Some("udieresis"), // U+00FC (Latin small letter u with diaeresis)
            0xFD => Some("yacute"), // U+00FD (Latin small letter y with acute)
            0xFE => Some("thorn"),  // U+00FE (Latin small letter thorn)
            0xFF => Some("ydieresis"), // U+00FF (Latin small letter y with diaeresis)

            // All other GIDs not in the supported ranges
            _ => None,
        }
    }

    /// Get the pre-computed byte→char lookup table for OneByte (simple) fonts.
    /// Built lazily on first call by running `char_to_unicode` for all 256 byte values.
    /// Returns a 256-element array: non-'\0' = single printable char, '\0' = needs fallback.
    /// Control chars (except tab/newline/cr), multi-char, and \u{FFFD} are stored as '\0'.
    pub fn get_byte_to_char_table(&self) -> &[char; 256] {
        self.byte_to_char_table.get_or_init(|| {
            let mut tbl = ['\0'; 256];
            for i in 0..=255u8 {
                if let Some(s) = self.char_to_unicode(i as u32) {
                    let mut chars = s.chars();
                    if let Some(c) = chars.next() {
                        if chars.next().is_none()
                            && c != '\u{FFFD}'
                            && (c >= '\x20' || c == '\t' || c == '\n' || c == '\r')
                        {
                            tbl[i as usize] = c;
                        }
                        // Multi-char, replacement, or control char: leave as '\0'
                    }
                }
            }
            tbl
        })
    }

    /// Pre-computed byte→width lookup for simple (non-Type0) fonts.
    /// Returns a 256-entry array where index i = glyph width for byte i.
    /// Eliminates per-byte bounds check and subtraction in advance_position.
    #[inline]
    pub fn get_byte_to_width_table(&self) -> &[f32; 256] {
        self.byte_to_width_table.get_or_init(|| {
            let mut tbl = [self.default_width; 256];
            if let Some(widths) = &self.widths {
                if let Some(first_char) = self.first_char {
                    for (idx, &w) in widths.iter().enumerate() {
                        let code = first_char as usize + idx;
                        if code < 256 {
                            tbl[code] = w;
                        }
                    }
                }
            } else {
                // Standard-14 fonts ship without /Widths arrays; per PDF
                // spec (ISO 32000-1 §9.6.2.2) readers must use built-in
                // metrics. get_standard_font_width returns Some(w) for
                // Helvetica/Times/Courier variants and None otherwise,
                // so non-standard fonts retain the default_width fallback.
                for byte_code in 0..256u16 {
                    if let Some(w) = self.get_standard_font_width(byte_code) {
                        tbl[byte_code as usize] = w;
                    }
                }
            }
            tbl
        })
    }

    /// Convert a character code to Unicode string.
    ///
    /// Returns the faithful Unicode mapping per PDF Spec §9.10.2. Ligature
    /// characters (U+FB00–FB06) are preserved here; expansion into component
    /// letters is done by the text pipeline via `LigatureDecisionMaker`, which
    /// inspects surrounding context (neighboring chars, word boundaries) to
    /// decide whether to split — keeping font_dict a pure encoding layer.
    pub fn char_to_unicode(&self, char_code: u32) -> Option<String> {
        let allocation = FontAllocation(self.import_budget.as_deref());
        allocation.check()?;
        if let Ok(memo) = self.type0_unicode_memo.lock() {
            if let Some(cached) = memo.get(&char_code) {
                return match cached {
                    Some(text) => allocation.string(text),
                    None => None,
                };
            }
        }
        // Misses, including absent mappings, create memo work. Hit copies consume
        // decoded bytes but do not charge another memo entry.
        allocation.entries(1)?;
        let result = match self.char_to_unicode_uncached(char_code) {
            Some(text) => Some(normalize_cjk_radical_forms_owned(text, allocation)?),
            None => None,
        };
        allocation.check()?;
        let cached = match &result {
            Some(text) => Some(allocation.string(text)?),
            None => None,
        };
        if let Ok(mut memo) = self.type0_unicode_memo.lock() {
            memo.try_reserve(1).map_err(|_| allocation.reject()).ok()?;
            memo.insert(char_code, cached);
        } else if allocation.0.is_some() {
            allocation.reject();
            return None;
        }
        result
    }

    /// Uncached decode cascade behind [`Self::char_to_unicode`].
    fn char_to_unicode_uncached(&self, char_code: u32) -> Option<String> {
        let allocation = FontAllocation(self.import_budget.as_deref());
        allocation.check()?;
        // char_code is now u32 to support 4-byte character codes (0x00000000-0xFFFFFFFF)
        // This is backward compatible - u16 values are automatically promoted to u32

        // ==================================================================================
        // PRIORITY 1: ToUnicode CMap (PDF Spec Section 9.10.2, Method 1)
        // ==================================================================================
        //
        // Per §9.10.2: if a ToUnicode CMap is PRESENT it is the authoritative source.
        // For composite (Type0) fonts a present-but-incomplete ToUnicode means the
        // unmapped codes genuinely have no Unicode equivalent. Falling through to the
        // predefined-CMap path (Priority 3 §9.10.2) for Type0 would guess wrong CJK
        // characters and score near zero versus ground truth. Therefore:
        //
        //   • ToUnicode hit → return the mapped string (or U+FFFD if it maps to FFFD
        //     or a bare C0 control character).
        //   • ToUnicode miss AND font is Type0 → return U+FFFD, do NOT fall through.
        //   • ToUnicode miss AND font is NOT Type0 → fall through to lower priorities
        //     (simple fonts with standard encoding still benefit from further lookup).
        //
        // Fix A (§9.10.2 Priority-3 guard): implemented in the CMap-miss branch below.
        // Fix B (control-character filter): applied on CMap hits.
        if let Some(lazy_cmap) = &self.to_unicode {
            if let Some(cmap) = lazy_cmap.get() {
                let raw_unicode = cmap.get_with_budget(&char_code, allocation.0);
                let had_hit = raw_unicode.is_some();

                // For Identity-encoded fonts, a U+FFFD (notdefrange) or a BMP
                // noncharacter (U+FFFE / U+FFFF) result is NOT a definitive
                // mapping — some producers stuff these into ToUnicode as a
                // "no glyph" placeholder (arial_unicode_ab_cidfont maps every
                // CID to U+FFFF). The CID→GID→embedded-cmap / CID-as-Unicode
                // fallback below recovers the real character, so treat them as a
                // CMap miss. Noncharacters are permanently reserved and never
                // valid text, so this can only ever improve Identity-font output.
                let effective_hit = raw_unicode.filter(|u| {
                    let is_placeholder = !u.is_empty()
                        && u.chars()
                            .all(|c| matches!(c, '\u{FFFD}' | '\u{FFFE}' | '\u{FFFF}'));
                    !(is_placeholder && matches!(self.encoding, Encoding::Identity))
                });

                if let Some(unicode) = effective_hit {
                    // Fix B: filter bare C0 control characters (U+0000–U+001F except
                    // tab/LF/CR which are legitimate whitespace in extracted text).
                    // These are never valid visible text and typically indicate a
                    // broken ToUnicode entry. Return U+FFFD and do NOT fall through
                    // even for simple fonts — the CMap explicitly mapped this code.
                    let is_c0_control = unicode
                        .chars()
                        .all(|c| matches!(c as u32, 0x00..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F))
                        && !unicode.is_empty();

                    if unicode.as_ref() == "\u{FFFD}" {
                        log::debug!(
                            "ToUnicode CMap has U+FFFD for code 0x{:02X} in font '{}' - returning U+FFFD",
                            char_code, self.base_font
                        );
                        return allocation.string("\u{FFFD}");
                    } else if is_c0_control {
                        log::debug!(
                            "ToUnicode CMap maps code 0x{:04X} to C0 control char(s) in font '{}' - returning U+FFFD",
                            char_code, self.base_font
                        );
                        return allocation.string("\u{FFFD}");
                    } else {
                        // Interception A (Item 1): glyph-name-gated punctuation
                        // recovery. When a present ToUnicode CMap resolves a code to
                        // a non-sensible symbol (e.g. U+00AC `¬`) but the font's
                        // authoritative glyph name for that code is punctuation
                        // (`period`/`comma`/`hyphen`/`minus` via /Differences or the
                        // embedded post/charset table), prefer the §9.10.2(a)+(b) AGL
                        // result. Gated so a correctly-mapped period (whose hit is
                        // already `.`) never enters here.
                        if is_non_sensible_symbol(&unicode) {
                            if let Some(glyph_name) = self.glyph_name_for_code(char_code) {
                                if let Some(punct) = punctuation_unicode_for_glyph_name(glyph_name)
                                {
                                    log::debug!(
                                        "Interception A: code 0x{:04X} ToUnicode '{}' is a non-sensible symbol; glyph name '{}' → '{}' (font '{}')",
                                        char_code, unicode, glyph_name, punct, self.base_font
                                    );
                                    return allocation.string(punct);
                                }
                            }
                        }
                        return match unicode {
                            std::borrow::Cow::Borrowed(text) => allocation.string(text),
                            std::borrow::Cow::Owned(text) => Some(text),
                        };
                    }
                } else {
                    if had_hit {
                        log::debug!(
                            "Identity font '{}': notdefrange U+FFFD treated as miss for code 0x{:04X} — falling through to CID-as-Unicode",
                            self.base_font, char_code
                        );
                    } else {
                        log::debug!(
                            "ToUnicode CMap MISS: font='{}' subtype='{}' code=0x{:04X} (cmap has {} entries)",
                            self.base_font, self.subtype, char_code, cmap.len()
                        );
                    }

                    // Fix A (§9.10.2): for composite (Type0) fonts a present ToUnicode
                    // CMap is the authoritative mapping. A miss means the glyph has no
                    // Unicode equivalent — do NOT fall through to the predefined-CMap
                    // path which would produce plausible-looking but wrong CJK chars.
                    // Exception: Identity-encoded fonts map CID directly to Unicode, so
                    // a CMap miss still has a valid fallback (CID == Unicode codepoint).
                    // Blocking them here would suppress spaces and Latin characters.
                    if self.subtype == "Type0" && !matches!(self.encoding, Encoding::Identity) {
                        log::debug!(
                            "Type0 font '{}': ToUnicode present but code 0x{:04X} not covered → U+FFFD (no Priority-3 fallback per §9.10.2)",
                            self.base_font, char_code
                        );
                        return allocation.string("\u{FFFD}");
                    }
                }
            } else {
                log::warn!(
                    "Failed to parse lazy CMap for font '{}' - will fall back to Priority 2",
                    self.base_font
                );
            }
        } else if self.subtype == "Type0" {
            log::debug!(
                "Type0 font '{}' missing ToUnicode CMap - will fall back to Priority 2",
                self.base_font
            );
        }

        // ==================================================================================
        // PRIORITY 2: Predefined CMaps (PDF Spec Section 9.7.5.2)
        // ==================================================================================
        // Phase 3.1: Identity-H/Identity-V Predefined CMap Support
        //
        // For CID-keyed fonts (Type0 subtype), predefined CMaps provide character mapping
        // when no ToUnicode CMap is present. This is critical for CJK PDFs using standard
        // Adobe CID collections (Adobe-Identity, Adobe-GB1, Adobe-Japan1, etc.)
        //
        // Identity-H/Identity-V: The simplest predefined CMap
        // - Maps 2-byte CID directly to 2-byte Unicode code point: CID == Unicode
        // - Used with ANY font when encoding is "Identity-H" or "Identity-V"
        // - Per PDF Spec ISO 32000-1:2008 Section 9.7.5.2
        //
        // Examples:
        // - CID 0x4E00 → U+4E00 (CJK UNIFIED IDEOGRAPH "一" in Chinese/Japanese)
        // - CID 0x0041 → U+0041 (Latin Capital Letter A)
        //
        // NOTE: Identity-H/V is actually handled by checking the encoding field.
        // It is checked here for Type0 fonts to ensure it happens before other fallbacks.
        if self.subtype == "Type0" {
            if let Encoding::Standard(ref encoding_name) = self.encoding {
                if encoding_name == "Identity-H"
                    || encoding_name == "Identity-V"
                    || encoding_name.contains("UCS2")
                    || encoding_name.contains("UTF16")
                {
                    // For Identity-H/V: CID value IS the Unicode code point (2-byte)
                    // Valid Unicode range for 2-byte CID: 0x0000 to 0xFFFF
                    // (Standard Unicode BMP - Basic Multilingual Plane)
                    // Since char_code is u16, it's always in range [0x0000, 0xFFFF]
                    //
                    // IMPORTANT: Per PDF Spec 9.10.2, Type0 fonts require either:
                    // 1. A ToUnicode CMap, OR
                    // 2. A predefined CMap (which requires CIDSystemInfo)
                    //
                    // If neither exists, we should NOT treat Identity-H/V as valid for Type0.
                    // This prevents "identity" treatment when there's no proper CIDSystemInfo.
                    if self.cid_system_info.is_some() {
                        // For Adobe-Identity ordering, CIDs are glyph indices (GIDs),
                        // NOT Unicode code points. Try the embedded TrueType cmap first.
                        let is_identity_ordering = self
                            .cid_system_info
                            .as_ref()
                            .map(|info| info.ordering == "Identity")
                            .unwrap_or(false);

                        if is_identity_ordering {
                            // Try TrueType cmap: CID → GID → Unicode
                            if let Some(tt_cmap) = self.truetype_cmap() {
                                let gid = if let Some(ref cid_to_gid) = self.cid_to_gid_map {
                                    cid_to_gid.get_gid(char_code as u16)
                                } else {
                                    char_code as u16
                                };
                                if let Some(unicode_char) = tt_cmap.get_unicode(gid) {
                                    return allocation.character(unicode_char);
                                }
                            }
                        }

                        // For UCS2/UTF16 encodings, char codes ARE Unicode values directly.
                        // For Identity-H/V with non-Identity ordering (e.g., Adobe-GB1),
                        // char codes are CIDs that need CID-to-Unicode lookup.
                        let is_ucs2_or_utf16 =
                            encoding_name.contains("UCS2") || encoding_name.contains("UTF16");
                        let is_non_identity_ordering = self
                            .cid_system_info
                            .as_ref()
                            .map(|info| info.ordering != "Identity")
                            .unwrap_or(false);

                        if !is_ucs2_or_utf16 && is_non_identity_ordering {
                            // Identity-H/V with CJK collection: CIDs are NOT Unicode!
                            if let Some(unicode_codepoint) = lookup_predefined_cmap(
                                encoding_name,
                                &self.cid_system_info,
                                char_code as u16,
                            ) {
                                if let Some(unicode_char) = char::from_u32(unicode_codepoint) {
                                    return allocation.character(unicode_char);
                                }
                            }
                            // CID lookup failed — fall through to Priority 2b and beyond
                        } else {
                            // UCS2/UTF16 or Adobe-Identity: char code == Unicode
                            if let Some(unicode_char) = char::from_u32(char_code) {
                                if !unicode_char.is_control() || unicode_char == ' ' {
                                    return allocation.character(unicode_char);
                                }
                            }
                        }
                    } else {
                        // No CIDSystemInfo — use CID-as-Unicode as last resort.
                        // Many PDF generators assign CID values equal to Unicode code points
                        // even without proper CIDSystemInfo. MuPDF uses this fallback.
                        if let Some(unicode_char) = char::from_u32(char_code) {
                            if !unicode_char.is_control() || unicode_char == ' ' {
                                log::debug!(
                                    "Identity-H/V CID-as-Unicode fallback (no CIDSystemInfo): font='{}' CID=0x{:04X} → '{}' (U+{:04X})",
                                    self.base_font,
                                    char_code,
                                    unicode_char,
                                    unicode_char as u32
                                );
                                return allocation.character(unicode_char);
                            }
                        }
                        log::debug!(
                            "Type0 font '{}' with {} encoding: CID 0x{:04X} is not a valid Unicode code point",
                            self.base_font,
                            encoding_name,
                            char_code
                        );
                    }
                }
            }
        }

        // ==================================================================================
        // PRIORITY 2a: Shift-JIS (RKSJ) direct decoding
        // ==================================================================================
        // For fonts using 90ms-RKSJ-H/V encoding, the char_code is a Shift-JIS value
        // (after byte grouping in decode_text_to_unicode). Convert directly to Unicode.
        if self.subtype == "Type0" {
            if let Encoding::Standard(ref enc) = self.encoding {
                if enc.contains("RKSJ") {
                    if let Some(unicode_char) = shift_jis_to_unicode(char_code as u16) {
                        return allocation.character(unicode_char);
                    }
                }
            }
        }

        // ==================================================================================
        // PRIORITY 2b: Unicode-based Predefined CMaps (Phase 3.2)
        // ==================================================================================
        // For Type0 fonts without a ToUnicode CMap: follow PDF §9.10.2 priority order.
        //
        // The spec defines two distinct encoding CMap kinds:
        //
        //   (a) Byte-encoding CMaps (GBpc-EUC-H, GB-EUC-H, B5pc-H, EUC-H, KSC-EUC-H,
        //       etc.): the value in the content stream is a raw multi-byte code in a
        //       legacy encoding (GBK, EUC-CN, Big5, EUC-JP, EUC-KR). §9.10.2 says to
        //       map char code → CID first, but those encoding CMap tables are not
        //       embedded here. Decoding the raw bytes directly with encoding_rs is
        //       equivalent (same Unicode output) and is permitted by the spec's fallback
        //       clause: "there is no way to determine … a conforming reader may choose a
        //       character code of their choosing."
        //
        //   (b) Identity / UCS2 CMaps (Identity-H, UniGB-UCS2-H, etc.): the value in
        //       the content stream IS (or approximates) a CID. Use the Adobe-XX CID →
        //       Unicode table directly (§9.10.2 step b).
        //
        // `decode_cjk_raw_charcode` returns None for non-byte-encoding CMaps, so
        // trying it first is safe: it is a no-op for Identity/UCS2 fonts.
        if self.subtype == "Type0" {
            let enc_name = match &self.encoding {
                Encoding::Standard(name) => name.as_str(),
                Encoding::Identity => "Identity-H",
                Encoding::Custom(_) => "",
            };

            // Step (a): try direct byte decode for legacy CJK byte-encoding CMaps.
            // This is the correct primary path for GBpc-EUC-H, GB-EUC-H, B5pc-H,
            // EUC-H, KSC-EUC-H, etc. Returns None for Identity/UCS2 CMaps, in
            // which case we fall through to the CID lookup below.
            if let Some(result) = decode_cjk_raw_charcode_with_budget(
                char_code,
                enc_name,
                &self.cid_system_info,
                allocation,
            ) {
                return Some(result);
            }

            // Step (b): CID → Unicode lookup for identity / UCS2 CMaps where the
            // char code in the stream is already a CID (or very close to one).
            if let Some(unicode_codepoint) =
                lookup_predefined_cmap(enc_name, &self.cid_system_info, char_code as u16)
            {
                if let Some(unicode_char) = char::from_u32(unicode_codepoint) {
                    return allocation.character(unicode_char);
                }
            }
        }

        // ==================================================================================
        // PRIORITY 2: Predefined Encodings (PDF Spec Section 9.10.2, Method 2)
        // ==================================================================================
        // For symbolic fonts (Flags bit 3 set), the PDF spec requires us to IGNORE any
        // /Encoding entry and use the font's built-in encoding directly.
        //
        // PDF Spec ISO 32000-1:2008, Section 9.6.6.1:
        // "For symbolic fonts, the Encoding entry is ignored; characters are mapped directly
        // using their character codes to glyphs in the font."
        //
        // Common symbolic fonts: Symbol (Greek/math), ZapfDingbats (decorative)
        if self.is_symbolic() {
            let font_name_lower = &self.base_font;

            // Symbol font: Maps character codes to Greek letters and mathematical symbols
            // Standard encoding defined in PDF spec Annex D.4
            if contains_ignore_ascii_case(font_name_lower, "symbol") {
                if let Some(unicode_char) = symbol_encoding_lookup(char_code as u8) {
                    log::debug!(
                        "Symbolic font '{}': code 0x{:02X} → '{}' (U+{:04X}) [using Symbol encoding]",
                        self.base_font,
                        char_code,
                        unicode_char,
                        unicode_char as u32
                    );
                    return allocation.character(unicode_char);
                }
            }
            // ZapfDingbats font: Maps character codes to decorative symbols
            // Standard encoding defined in PDF spec Annex D.5
            else if contains_ignore_ascii_case(font_name_lower, "zapf")
                || contains_ignore_ascii_case(font_name_lower, "dingbat")
            {
                if let Some(unicode_char) = zapf_dingbats_encoding_lookup(char_code as u8) {
                    log::debug!(
                        "Symbolic font '{}': code 0x{:02X} → '{}' (U+{:04X}) [using ZapfDingbats encoding]",
                        self.base_font,
                        char_code,
                        unicode_char,
                        unicode_char as u32
                    );
                    return allocation.character(unicode_char);
                }
            }

            // For other symbolic fonts without specific encoding, fall through to /Encoding
            // (though spec says to ignore /Encoding, some PDFs may still work with it)
        }

        // ==================================================================================
        // PRIORITY 3: Font's /Encoding Entry (PDF Spec Section 9.10.2, Method 3)
        // ==================================================================================
        // For non-symbolic fonts, use the /Encoding entry which can be:
        // - A predefined encoding name (e.g., WinAnsiEncoding, MacRomanEncoding)
        // - A custom encoding dictionary with /BaseEncoding and /Differences array
        //
        // The /Differences array allows overriding specific character codes with custom
        // glyph names, which are then mapped to Unicode via the Adobe Glyph List (AGL).
        match &self.encoding {
            Encoding::Standard(name) => {
                // Check for Identity-H and Identity-V encodings (common for Type0 fonts)
                if name == "Identity-H" || name == "Identity-V" {
                    // NOTE: Type0 fonts with Identity-H/V are handled at Priority 2 (predefined CMaps)
                    // above, so this code path is only reached for simple fonts (Type1, TrueType).
                    // Type0 fonts will have already returned at Priority 2 if the CID is valid Unicode.
                    if self.subtype == "Type0" {
                        // Priority 2 didn't map this CID. Use CID-as-Unicode fallback.
                        if let Some(unicode_char) = char::from_u32(char_code) {
                            if !unicode_char.is_control() || unicode_char == ' ' {
                                log::debug!(
                                    "Type0 font '{}' {} encoding Priority 3 CID-as-Unicode: CID 0x{:04X} → '{}' (U+{:04X})",
                                    self.base_font,
                                    name,
                                    char_code,
                                    unicode_char,
                                    unicode_char as u32
                                );
                                return allocation.character(unicode_char);
                            }
                        }
                        return allocation.string("\u{FFFD}");
                    }
                    // For simple fonts, Identity encoding is valid
                    if let Some(ch) = char::from_u32(char_code) {
                        return allocation.character(ch);
                    }
                }

                // For TrueType subset fonts with no /Encoding, character codes are often
                // GIDs (glyph indices), not standard encoding values. Per PDF Spec 9.6.5.4,
                // when no /Encoding exists and the font has a (3,1) cmap, character codes
                // map through the cmap. Try TrueType cmap first for these fonts.
                if (self.subtype == "TrueType" || self.subtype == "Type1")
                    && name == "StandardEncoding"
                {
                    if let Some(tt_cmap) = self.truetype_cmap() {
                        let gid = tt_cmap
                            .code_to_gid(char_code as u16)
                            .unwrap_or(char_code as u16);
                        if let Some(unicode_char) = tt_cmap.get_unicode(gid) {
                            return allocation.character(unicode_char);
                        }
                    }
                }

                // Predefined encodings: StandardEncoding, WinAnsiEncoding, MacRomanEncoding, etc.
                if let Some(unicode) =
                    standard_encoding_lookup_with_budget(name, char_code as u8, allocation)
                {
                    log::debug!(
                        "Standard encoding '{}': code 0x{:02X} → '{}'",
                        name,
                        char_code,
                        unicode
                    );
                    return Some(unicode);
                }
            },
            Encoding::Custom(map) => {
                // Custom encoding with /Differences array
                // Maps character code → glyph name → Unicode (via AGL)
                if let Some(&custom_char) = map.get(&(char_code as u8)) {
                    log::debug!(
                        "Custom encoding: code 0x{:02X} → '{}' (U+{:04X})",
                        char_code,
                        custom_char,
                        custom_char as u32
                    );

                    // Interception B (Item 1): glyph-name-gated punctuation
                    // override. If the base/program encoding resolved this code to a
                    // non-sensible symbol but the /Differences glyph name is
                    // punctuation, the name is authoritative (ISO 32000-1 §9.6.6.1) —
                    // return the AGL punctuation so a `/period`-named code always wins
                    // as `.` regardless of how the resolved char came out.
                    if is_non_sensible_symbol(custom_char.encode_utf8(&mut [0; 4])) {
                        if let Some(glyph_name) = self.diff_glyph_names.get(&(char_code as u8)) {
                            if let Some(punct) = punctuation_unicode_for_glyph_name(glyph_name) {
                                log::debug!(
                                    "Interception B: code 0x{:02X} resolved to non-sensible symbol '{}'; /Differences name '{}' → '{}' (font '{}')",
                                    char_code, custom_char, glyph_name, punct, self.base_font
                                );
                                return allocation.string(punct);
                            }
                        }
                    }

                    // Handle ligatures (ff, fi, fl, ffi, ffl) by expanding to component characters
                    // This is NOT in the PDF spec but improves text extraction usability
                    if is_ligature_char(custom_char) {
                        if let Some(expanded) = expand_ligature_char(custom_char) {
                            return allocation.string(expanded);
                        }
                    }

                    return allocation.character(custom_char);
                }
                // Check multi_char_map for compound glyph names (e.g., f_f → "ff")
                if let Some(multi_str) = self.multi_char_map.get(&(char_code as u8)) {
                    return allocation.string(multi_str);
                }
            },
            Encoding::Identity => {
                // CRITICAL: Identity encoding assumes char_code == Unicode.
                // This is ONLY valid for simple fonts, NOT Type0/CID fonts.
                // Per PDF Spec ISO 32000-1:2008 Section 9.7.6.3:
                // "Type0 fonts REQUIRE ToUnicode CMaps for proper character mapping"

                if self.subtype == "Type0" {
                    // Type0 fonts: character codes are CID (glyph indices), NOT Unicode
                    // Per PDF Spec ISO 32000-1:2008 Section 9.7.4.2, when no ToUnicode CMap exists,
                    // conforming readers SHALL use the TrueType font's internal "cmap" table as fallback.
                    // This requires translating CID → GID via the CIDToGIDMap, then looking up Unicode.

                    if let Some(tt_cmap) = self.truetype_cmap() {
                        // Translate CID → GID using the CIDToGIDMap
                        // Note: CIDToGIDMap only works with u16 CIDs (2-byte codes)
                        // For CIDs > 0xFFFF, we skip CIDToGIDMap and use char_code as GID if it fits in u16
                        let gid = if char_code <= 0xFFFF {
                            if let Some(ref cid_to_gid) = self.cid_to_gid_map {
                                cid_to_gid.get_gid(char_code as u16)
                            } else {
                                // No explicit mapping - assume Identity (CID == GID)
                                char_code as u16
                            }
                        } else {
                            // Large CID (> 0xFFFF) - cannot use CIDToGIDMap
                            // GIDs are typically u16, so large CIDs won't map correctly
                            log::debug!(
                                "CID 0x{:X} in font '{}' is too large (> 0xFFFF) for CIDToGIDMap - skipping TrueType cmap",
                                char_code,
                                self.base_font
                            );
                            // Return early to skip TrueType cmap lookup for large CIDs
                            return None;
                        };

                        if let Some(unicode_char) = tt_cmap.get_unicode(gid) {
                            log::debug!(
                                "TrueType cmap fallback SUCCESS: font='{}' CID=0x{:04X} (GID={}) → '{}' (U+{:04X})",
                                self.base_font,
                                char_code,
                                gid,
                                unicode_char,
                                unicode_char as u32
                            );
                            return allocation.character(unicode_char);
                        } else {
                            log::debug!(
                                "TrueType cmap: GID {} not found in font '{}' (CID 0x{:04X} mapped via {})",
                                gid,
                                self.base_font,
                                char_code,
                                if self.cid_to_gid_map.is_some() {
                                    "explicit CIDToGIDMap"
                                } else {
                                    "Identity mapping"
                                }
                            );
                        }

                        // ==========================================================================
                        // PRIORITY 3c (#535,): embedded post/charset glyph name → AGL+synth
                        // ==========================================================================
                        // Per ISO 32000-1:2008 §9.10.2 fallback chain, consult the embedded font
                        // program's own glyph-name table when the TrueType `cmap` reverse lookup
                        // misses. Common on PowerPoint/Acrobat-exported Type0 Identity-H subset
                        // fonts that strip the Unicode `cmap` but keep `post` Format 2 names —
                        // bullets and `fi`/`fl` ligatures only recover via this path. Mirrors
                        // pdf.js / MuPDF / PDFBox 3.x behaviour. The earlier `gid_to_standard_
                        // glyph_name` (P5) only knows hardcoded ASCII-range GID → name; the post
                        // table is the font's own authoritative source.
                        if let Some(glyph_name) = self.embedded_glyph_name(gid) {
                            if let Some(unicode) =
                                super::character_mapper::glyph_name_to_unicode_char(glyph_name)
                            {
                                log::debug!(
                                    "Priority 3c (embedded post glyph name): font='{}' CID=0x{:04X} (GID={}) → '{}' → '{}'",
                                    self.base_font,
                                    char_code,
                                    gid,
                                    glyph_name,
                                    unicode,
                                );
                                return allocation.character(unicode);
                            } else {
                                log::debug!(
                                    "Priority 3c: font='{}' GID={} → name='{}' but AGL/synth lookup failed",
                                    self.base_font,
                                    gid,
                                    glyph_name,
                                );
                            }
                        }
                    }

                    // ==================================================================================
                    // PRIORITY 5: Adobe Glyph List Fallback (Phase 1.2)
                    // ==================================================================================
                    // When TrueType cmap fails (or is not available), try Adobe Glyph List fallback.
                    // This handles Type0 fonts with standard glyph names (e.g., Aptos, LMRoman)
                    // that don't have ToUnicode CMaps or embedded TrueType fonts.
                    //
                    // Process: CID → GID (via CIDToGIDMap) → Glyph Name → Unicode (via AGL)
                    //
                    // IMPORTANT: Only apply AGL fallback if a CIDToGIDMap is explicitly defined
                    // (even if it's Identity). This distinguishes between:
                    // - Type0 fonts with proper CIDToGIDMap (may have standard glyphs)
                    // - Malformed Type0 fonts without CIDToGIDMap (unlikely to work)
                    //
                    // Per PDF Spec ISO 32000-1:2008 Section 9.10.2:
                    // "If a ToUnicode CMap is not available, conforming readers may fall back
                    // to predefined encodings and glyph name lookup."

                    // A present-but-empty /ToUnicode (0 bfchar/bfrange) maps nothing, so it
                    // counts as absent — otherwise an Identity-ordered font with an empty CMap
                    // would suppress the fallbacks below and drop all its text.
                    let has_usable_tounicode = self
                        .to_unicode
                        .as_ref()
                        .and_then(|c| c.get())
                        .is_some_and(|cmap| !cmap.is_empty());
                    let is_identity_ordered = self
                        .cid_system_info
                        .as_ref()
                        .map(|info| info.ordering == "Identity")
                        .unwrap_or(false);

                    // The GID→AGL fallback below is a numeric *guess*: it reads the GID as a
                    // codepoint via the standard glyph-name table → AGL. It is meaningless for
                    // Identity-ordered subset fonts, whose GIDs are arbitrary — a remapped GID
                    // lands on an unrelated punctuation name (e.g. "Justin" → "J)'(i#") and would
                    // shadow the CID-as-Unicode mapping below — so it is skipped there. With a
                    // usable /ToUnicode present a code reaching here is genuinely unmapped, so the
                    // guess is suppressed entirely — prefer U+FFFD so the gap is detectable.
                    if !has_usable_tounicode && !is_identity_ordered {
                        if let Some(ref cid_to_gid) = self.cid_to_gid_map {
                            // CIDToGIDMap only works with u16 CIDs (2-byte codes)
                            if char_code > 0xFFFF {
                                log::debug!(
                                    "CID 0x{:X} in font '{}' is too large (> 0xFFFF) for CIDToGIDMap AGL fallback - skipping",
                                    char_code,
                                    self.base_font
                                );
                                // Fall through to continue fallback attempts
                            } else {
                                let gid = cid_to_gid.get_gid(char_code as u16);

                                if let Some(glyph_name) = Self::gid_to_standard_glyph_name(gid) {
                                    if let Some(&unicode_char) = ADOBE_GLYPH_LIST.get(glyph_name) {
                                        log::debug!(
                                            "Adobe Glyph List fallback SUCCESS: font='{}' CID=0x{:04X} (GID={}) → glyph '{}' → '{}' (U+{:04X})",
                                            self.base_font,
                                            char_code,
                                            gid,
                                            glyph_name,
                                            unicode_char,
                                            unicode_char as u32
                                        );
                                        return allocation.character(unicode_char);
                                    }
                                }
                            }
                        }
                    }

                    // CID-as-Unicode fallback: many producers assign CID == Unicode codepoint.
                    // Used when there is no usable /ToUnicode, and — for Identity-ordered fonts —
                    // also for uncovered whitespace (CID 0x20 → space, which producers routinely
                    // omit and is reliably U+0020; dropping it would wreck word boundaries). Any
                    // other uncovered CID in a font that *has* a /ToUnicode has no codepoint we can
                    // trust (e.g. a ligature subset slot), so it decodes to U+FFFD instead of a
                    // plausible-but-wrong, per-file-varying guess.
                    let identity_whitespace = is_identity_ordered && char_code == 0x20;
                    if !has_usable_tounicode || identity_whitespace {
                        if let Some(unicode_char) = char::from_u32(char_code) {
                            if !unicode_char.is_control() || unicode_char == ' ' {
                                log::debug!(
                                    "Type0 font '{}' Identity encoding CID-as-Unicode fallback: CID 0x{:04X} → '{}' (U+{:04X})",
                                    self.base_font,
                                    char_code,
                                    unicode_char,
                                    unicode_char as u32
                                );
                                return allocation.character(unicode_char);
                            }
                        }
                    }
                    log::warn!(
                        "Type0 font '{}' using Identity encoding: CID 0x{:04X} could not be mapped to Unicode. \
                         Embedded font: {} bytes.",
                        self.base_font,
                        char_code,
                        self.embedded_font_data
                            .as_ref()
                            .map(|d| d.len())
                            .unwrap_or(0)
                    );
                    return allocation.string("\u{FFFD}");
                }

                // For simple fonts (Type1, TrueType), Identity encoding MAY be valid
                if let Some(ch) = char::from_u32(char_code) {
                    log::debug!(
                        "Identity encoding (simple font '{}'): code 0x{:02X} → '{}' (U+{:04X})",
                        self.base_font,
                        char_code,
                        ch,
                        ch as u32
                    );
                    return allocation.character(ch);
                }
            },
        }

        // ==================================================================================
        // PRIORITY 4: TrueType cmap fallback for simple fonts
        // ==================================================================================
        // When all encoding-based lookups fail, try the embedded TrueType cmap as a last
        // resort. For subset fonts, character codes may be GIDs that the encoding table
        // doesn't cover. The cmap provides GID → Unicode mapping.
        if self.subtype != "Type0" {
            if let Some(tt_cmap) = self.truetype_cmap() {
                // Symbolic TrueType fonts index glyphs by content byte through a
                // (3,0)/(1,0) symbol cmap, so the byte is not the GID. Resolve
                // byte→GID first; fall back to byte-as-GID when no symbol cmap.
                let gid = tt_cmap
                    .code_to_gid(char_code as u16)
                    .unwrap_or(char_code as u16);
                if let Some(unicode_char) = tt_cmap.get_unicode(gid) {
                    return allocation.character(unicode_char);
                }
            }
        }

        // ==================================================================================
        // PRIORITY 5: Fallback - No Mapping Found
        // ==================================================================================
        // If we reach here, the character is either:
        // - A control character (0x00-0x1F, 0x7F-0x9F) - intentionally omitted
        // - A character code outside all known encodings
        // - From a malformed PDF missing encoding information
        //
        // Control characters don't have visible representations, so returning None
        // (which becomes empty string) is more appropriate than returning � (U+FFFD).
        log::debug!(
            "No Unicode mapping for font '{}' code=0x{:02X} (symbolic={}, encoding={:?}) - likely control char",
            self.base_font,
            char_code,
            self.is_symbolic(),
            self.encoding
        );

        // ==================================================================================
        // PRIORITY 6: Unicode Ligature Fallback
        // ==================================================================================
        // If no encoding mapping was found and the raw character code falls
        // in the Unicode ligature block (U+FB00-U+FB06), decompose into the
        // component letters. This is a pure-fallback codepath — when no
        // font data identifies the glyph, standard ligature decomposition
        // is the safest recovery. LaTeX and scientific PDF producers emit
        // these codes directly.
        let ligature_components = match char_code {
            0xFB00 => Some("ff"),
            0xFB01 => Some("fi"),
            0xFB02 => Some("fl"),
            0xFB03 => Some("ffi"),
            0xFB04 => Some("ffl"),
            0xFB05 | 0xFB06 => Some("st"),
            _ => None,
        };
        if let Some(s) = ligature_components {
            return allocation.string(s);
        }

        None
    }

    /// Determine the font weight using a comprehensive cascade of PDF spec methods.
    ///
    /// Priority order per PDF Spec ISO 32000-1:2008:
    /// 1. FontWeight field from FontDescriptor (Table 122) - MOST RELIABLE
    /// 2. ForceBold flag (bit 19) from Flags field (Table 123)
    /// 3. Font name heuristics (fallback for legacy PDFs)
    /// 4. StemV analysis (stem thickness correlates with weight)
    ///
    /// # Returns
    ///
    /// FontWeight enum value (Thin to Black scale)
    ///
    /// # PDF Spec References
    ///
    /// - Table 122 (page 456): FontWeight values 100-900
    /// - Table 123 (page 457): ForceBold flag at bit 19 (0x80000)
    /// - Section 9.6.2: StemV field interpretation
    pub fn get_font_weight(&self) -> FontWeight {
        *self.weight_memo.get_or_init(|| self.compute_font_weight())
    }

    /// Uncached [`Self::get_font_weight`] body. Everything it reads is fixed at
    /// font-load time, so `weight_memo` can hold the answer for the font's life.
    fn compute_font_weight(&self) -> FontWeight {
        // ==================================================================================
        // PRIORITY 1: FontWeight Field (PDF Spec Table 122)
        // ==================================================================================
        // Most reliable method. If present, use directly.
        if let Some(weight_value) = self.font_weight {
            return FontWeight::from_pdf_value(weight_value);
        }

        // ==================================================================================
        // PRIORITY 2: ForceBold Flag (PDF Spec Table 123, Bit 19)
        // ==================================================================================
        // If ForceBold flag is set, font is explicitly bold.
        // Bit 19 = 0x80000 (524288 decimal)
        if let Some(flags_value) = self.flags {
            const FORCE_BOLD_BIT: i32 = 0x80000; // Bit 19 = 524288
            if (flags_value & FORCE_BOLD_BIT) != 0 {
                log::debug!("Font '{}': ForceBold flag set (bit 19) → Bold", self.base_font);
                return FontWeight::Bold;
            }
        }

        // ==================================================================================
        // PRIORITY 3: Font Name Heuristics
        // ==================================================================================
        // Fallback for fonts without FontDescriptor or with missing fields.
        // Checks for bold-indicating keywords in the font name.
        let name_lower = self.base_font.to_lowercase();

        // Check for explicit weight keywords in order of strength
        if name_lower.contains("black") || name_lower.contains("heavy") {
            return FontWeight::Black; // 900
        }
        if name_lower.contains("extrabold") || name_lower.contains("ultrabold") {
            return FontWeight::ExtraBold; // 800
        }
        if name_lower.contains("bold") {
            // Distinguish between "SemiBold" and "Bold"
            if name_lower.contains("semibold") || name_lower.contains("demibold") {
                return FontWeight::SemiBold; // 600
            }
            return FontWeight::Bold; // 700
        }
        if name_lower.contains("medium") {
            return FontWeight::Medium; // 500
        }
        if name_lower.contains("light") {
            if name_lower.contains("extralight") || name_lower.contains("ultralight") {
                return FontWeight::ExtraLight; // 200
            }
            return FontWeight::Light; // 300
        }
        if name_lower.contains("thin") {
            return FontWeight::Thin; // 100
        }

        // ==================================================================================
        // PRIORITY 4: StemV Analysis (EXPERIMENTAL)
        // ==================================================================================
        // StemV measures vertical stem thickness. Empirically:
        // - StemV > 110: Usually bold (700+)
        // - StemV 80-110: Medium (500-600)
        // - StemV < 80: Normal or lighter (400-)
        //
        // NOTE: This is a heuristic and may not be reliable for all fonts.
        // PDF spec does not mandate this correlation.
        if let Some(stem_v) = self.stem_v {
            log::debug!("Font '{}': Using StemV analysis (StemV={})", self.base_font, stem_v);

            if stem_v > 110.0 {
                return FontWeight::Bold; // 700
            } else if stem_v >= 80.0 {
                return FontWeight::Medium; // 500
            }
            // If StemV < 80, continue to default (Normal)
        }

        // ==================================================================================
        // DEFAULT: Normal Weight (400)
        // ==================================================================================
        // If no other method yields a weight, assume normal.
        FontWeight::Normal
    }

    /// Check if this font is bold (convenience method).
    ///
    /// Returns true if font weight is SemiBold (600) or higher.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if font.is_bold() {
    ///     // Apply bold markdown formatting
    /// }
    /// ```
    pub fn is_bold(&self) -> bool {
        self.get_font_weight().is_bold()
    }

    /// Return true when this font's per-glyph widths come from the PDF's
    /// `/Widths` array (for simple fonts) or `/W` array (for Type0/CID
    /// fonts), rather than from the generic 500/550/600-thousandths-of-em
    /// fallback that `FontInfo::new` uses when neither is present.
    ///
    /// Callers use this to decide whether `byte_to_width_table` is
    /// trustworthy: when it returns false, every glyph reports the same
    /// fallback advance, so bounding-box widths computed from those
    /// advances systematically over- or under-estimate the visible text
    /// extent. On affected PDFs that collapses the real gap between
    /// adjacent `Tj`-positioned words, gluing words together in
    /// `extract_text` output even though the PDF itself places them on
    /// distinct positions. See issue #328.
    pub fn has_explicit_widths(&self) -> bool {
        // F14 fix: return true only when the font actually has explicit width data.
        // Previously returned true for ALL Type0 fonts, which disabled gap-correction
        // for Type0 fonts with no /W or /DW — exactly the fonts that need correction.
        // Now: true when /Widths is present (simple fonts), or when /W has entries
        // (CID fonts), or when /DW was explicitly set in the CIDFont dictionary.
        self.widths.is_some() || self.cid_widths.is_some() || self.has_explicit_dw
    }

    /// Check if this font is likely italic based on the font name.
    ///
    /// This is a heuristic check looking for "Italic" or "Oblique" in the font name.
    pub fn is_italic(&self) -> bool {
        *self.italic_memo.get_or_init(|| {
            let name_lower = self.base_font.to_lowercase();
            name_lower.contains("italic") || name_lower.contains("oblique")
        })
    }

    /// Check if this is a symbolic font based on FontDescriptor flags.
    ///
    /// Symbolic fonts (bit 3 set in /Flags) contain glyphs outside the Adobe standard
    /// Latin character set. For symbolic fonts, the PDF spec requires ignoring any
    /// Encoding entry and using direct character code mapping to the font's built-in encoding.
    ///
    /// Common symbolic fonts: Symbol, ZapfDingbats
    ///
    /// PDF Spec: ISO 32000-1:2008, Table 5.20 - Font descriptor flags
    /// Bit 3: Symbolic - Font contains glyphs outside Adobe standard Latin character set
    /// Bit 6: Nonsymbolic - Font uses Adobe standard Latin character set (mutually exclusive with bit 3)
    pub fn is_symbolic(&self) -> bool {
        // Priority 1: Check FontDescriptor /Flags bit 3
        if let Some(flags_value) = self.flags {
            // Bit 3 = 0x04 (1 << 2, since bits are numbered starting at 1 in PDF spec)
            const SYMBOLIC_BIT: i32 = 1 << 2; // Bit 3
            return (flags_value & SYMBOLIC_BIT) != 0;
        }

        // Priority 2: Fallback to font name heuristic
        let name_lower = &self.base_font;
        contains_ignore_ascii_case(name_lower, "symbol")
            || contains_ignore_ascii_case(name_lower, "zapf")
            || contains_ignore_ascii_case(name_lower, "dingbat")
    }

    /// Get character from encoding (custom or standard).
    ///
    /// Custom encoding support
    ///
    /// This method normalizes a raw character code through the font's encoding,
    /// converting it to the actual Unicode character. This ensures word boundary
    /// detection works on real characters, not raw byte codes.
    ///
    /// Per PDF Spec ISO 32000-1:2008, Section 9.6.6:
    /// - Custom encodings with /Differences override standard encodings
    /// - Standard encodings have well-defined mappings
    /// - Identity encoding passes codes through as-is
    ///
    /// # Arguments
    ///
    /// * `code` - The raw byte value from the PDF content stream
    ///
    /// # Returns
    ///
    /// The normalized Unicode character, or None if no mapping exists
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::fonts::FontInfo;
    ///
    /// let font_info = /* ... load font ... */;
    /// if let Some(ch) = font_info.get_encoded_char(0x64) {
    ///     println!("Code 0x64 maps to: {}", ch);
    /// }
    /// ```
    pub fn get_encoded_char(&self, code: u8) -> Option<char> {
        match &self.encoding {
            Encoding::Custom(mappings) => {
                // Custom encoding: use explicit character mappings
                mappings.get(&code).copied()
            },
            Encoding::Standard(_encoding_name) => {
                // Standard encoding: for now, assume ToUnicode CMap handles this
                // If we need explicit standard encoding tables, add them here
                // For basic ASCII range, we can pass through
                if code < 128 {
                    Some(code as char)
                } else {
                    None
                }
            },
            Encoding::Identity => {
                // Identity encoding: code == Unicode (for CID fonts)
                // For single-byte codes, treat as Unicode
                if code < 128 {
                    Some(code as char)
                } else {
                    None
                }
            },
        }
    }

    /// Check if font has custom encoding.
    ///
    /// Custom encoding support
    ///
    /// Returns true if the font uses a custom encoding with /Differences array,
    /// which overrides standard encoding for specific character codes.
    ///
    /// # Returns
    ///
    /// true if the font has a custom encoding, false otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use pdf_oxide::fonts::FontInfo;
    ///
    /// let font_info = /* ... load font ... */;
    /// if font_info.has_custom_encoding() {
    ///     println!("Font uses custom encoding");
    /// }
    /// ```
    pub fn has_custom_encoding(&self) -> bool {
        matches!(self.encoding, Encoding::Custom(_))
    }
}

/// Map a PDF glyph name to a Unicode character.
///
/// This function implements the Adobe Glyph List (AGL) specification,
/// which defines standard mappings from PostScript glyph names to Unicode.
/// This is essential for parsing /Differences arrays in custom encodings.
///
/// # Arguments
///
/// * `glyph_name` - The PostScript glyph name (e.g., "bullet", "emdash", "Aacute")
///
/// # Returns
///
/// The corresponding Unicode character, or None if the glyph name is not recognized.
///
/// # References
///
/// - Adobe Glyph List Specification: https://github.com/adobe-type-tools/agl-specification
/// - PDF 32000-1:2008, Section 9.6.6.2 (Differences Arrays)
///
/// # Examples
///
/// ```ignore
/// # use pdf_oxide::fonts::font_dict::glyph_name_to_unicode;
/// assert_eq!(glyph_name_to_unicode("bullet"), Some('•'));
/// assert_eq!(glyph_name_to_unicode("emdash"), Some('—'));
/// assert_eq!(glyph_name_to_unicode("A"), Some('A'));
/// assert_eq!(glyph_name_to_unicode("unknown"), None);
/// ```ignore
///
/// Extended glyph names from TeX/math fonts (MSAM, MSBM, Computer Modern, etc.)
/// not present in the standard Adobe Glyph List.
static TEX_MATH_GLYPH_NAMES: phf::Map<&'static str, char> = phf::phf_map! {
    // AMS math symbols (MSAM10, MSBM10)
    "square" => '\u{25A1}',           // WHITE SQUARE
    "squaredot" => '\u{22A1}',        // SQUARED DOT OPERATOR
    "blacksquare" => '\u{25A0}',      // BLACK SQUARE
    "dblarrowup" => '\u{21C8}',       // UPWARDS PAIRED ARROWS
    "dblarrowdwn" => '\u{21CA}',      // DOWNWARDS PAIRED ARROWS
    "dblarrowleft" => '\u{21C7}',     // LEFTWARDS PAIRED ARROWS
    "dblarrowright" => '\u{21C9}',    // RIGHTWARDS PAIRED ARROWS
    "triangle" => '\u{25B3}',         // WHITE UP-POINTING TRIANGLE
    "triangledown" => '\u{25BD}',     // WHITE DOWN-POINTING TRIANGLE
    "triangleleft" => '\u{25C1}',     // WHITE LEFT-POINTING TRIANGLE
    "triangleright" => '\u{25B7}',    // WHITE RIGHT-POINTING TRIANGLE
    "blacktriangle" => '\u{25B2}',    // BLACK UP-POINTING TRIANGLE
    "blacktriangledown" => '\u{25BC}',// BLACK DOWN-POINTING TRIANGLE
    "blacktriangleleft" => '\u{25C0}',// BLACK LEFT-POINTING TRIANGLE
    "blacktriangleright" => '\u{25B6}',// BLACK RIGHT-POINTING TRIANGLE
    "diamond" => '\u{25C7}',          // WHITE DIAMOND
    "blackdiamond" => '\u{25C6}',     // BLACK DIAMOND
    "circle" => '\u{25CB}',           // WHITE CIRCLE
    "bullet1" => '\u{2219}',          // BULLET OPERATOR
    "star" => '\u{22C6}',             // STAR OPERATOR
    "bigstar" => '\u{2605}',          // BLACK STAR
    "checkmark" => '\u{2713}',        // CHECK MARK
    "maltese" => '\u{2720}',          // MALTESE CROSS
    // TeX arrows
    "arrowleft" => '\u{2190}',        // LEFTWARDS ARROW
    "arrowright" => '\u{2192}',       // RIGHTWARDS ARROW
    "arrowup" => '\u{2191}',          // UPWARDS ARROW
    "arrowdown" => '\u{2193}',        // DOWNWARDS ARROW
    "arrowboth" => '\u{2194}',        // LEFT RIGHT ARROW
    "arrowdblup" => '\u{21D1}',       // UPWARDS DOUBLE ARROW
    "arrowdbldown" => '\u{21D3}',     // DOWNWARDS DOUBLE ARROW
    "arrowdblleft" => '\u{21D0}',     // LEFTWARDS DOUBLE ARROW
    "arrowdblright" => '\u{21D2}',    // RIGHTWARDS DOUBLE ARROW
    "arrowdblboth" => '\u{21D4}',     // LEFT RIGHT DOUBLE ARROW
    // TeX math operators
    "langle" => '\u{27E8}',           // MATHEMATICAL LEFT ANGLE BRACKET
    "rangle" => '\u{27E9}',           // MATHEMATICAL RIGHT ANGLE BRACKET
    "lfloor" => '\u{230A}',           // LEFT FLOOR
    "rfloor" => '\u{230B}',           // RIGHT FLOOR
    "lceil" => '\u{2308}',            // LEFT CEILING
    "rceil" => '\u{2309}',            // RIGHT CEILING
    "emptyset" => '\u{2205}',         // EMPTY SET
    "infty" => '\u{221E}',            // INFINITY (alias)
    "nabla" => '\u{2207}',            // NABLA
    "partial" => '\u{2202}',          // PARTIAL DIFFERENTIAL
    "forall" => '\u{2200}',           // FOR ALL
    "exists" => '\u{2203}',           // THERE EXISTS
    "neg" => '\u{00AC}',              // NOT SIGN
    "backslash" => '\u{005C}',        // REVERSE SOLIDUS
    "prime" => '\u{2032}',            // PRIME
    "natural" => '\u{266E}',          // MUSIC NATURAL SIGN
    "flat" => '\u{266D}',             // MUSIC FLAT SIGN
    "sharp" => '\u{266F}',            // MUSIC SHARP SIGN
};

/// Convert a Shift-JIS encoded byte sequence (1 or 2 bytes) to a Unicode character.
/// Uses the encoding_rs crate for correct, complete Shift-JIS decoding.
fn shift_jis_to_unicode(code: u16) -> Option<char> {
    let pair = [(code >> 8) as u8, code as u8];
    let bytes = if code <= 0xFF { &pair[1..] } else { &pair[..] };
    // Two source bytes produce at most two Unicode scalars. Keep temporary
    // decoding on the stack; the caller reserves the returned Unicode string.
    let mut output = [0u8; 8];
    let mut decoder = encoding_rs::SHIFT_JIS.new_decoder_without_bom_handling();
    let (status, read, written) =
        decoder.decode_to_utf8_without_replacement(bytes, &mut output, true);
    if !matches!(status, encoding_rs::DecoderResult::InputEmpty) || read != bytes.len() {
        return None;
    }
    let mut chars = std::str::from_utf8(&output[..written]).ok()?.chars();
    let c = chars.next()?;
    // Ensure only one character was produced
    if chars.next().is_some() {
        return None;
    }
    Some(c)
}

/// Normalize CJK radical "presentation" codepoints to their canonical unified
/// ideograph: CJK Radicals Supplement (U+2E80–2EFF) and Kangxi Radicals
/// (U+2F00–2FDF). These blocks hold the radical glyphs used in dictionaries and
/// are never part of running text — but a font cmap that maps a glyph shared
/// between a radical and its ideograph to the *radical* codepoint (and a
/// GID→Unicode reverse lookup that then prefers it) surfaces e.g. 欠→⽋, 立→⽴.
/// NFKC carries each radical to its ideograph; only chars inside the two radical
/// blocks are touched, so legitimate text (incl. fullwidth forms) is unchanged.
/// Fast-path returns the input untouched when it contains no radical-block char.
#[cfg(test)]
fn normalize_cjk_radical_forms(s: &str) -> String {
    normalize_cjk_radical_forms_owned(s.to_owned(), FontAllocation(None)).unwrap_or_default()
}

fn normalize_cjk_radical_forms_owned(
    text: String,
    allocation: FontAllocation<'_>,
) -> Option<String> {
    use unicode_normalization::UnicodeNormalization;
    let is_radical = |c: char| matches!(c as u32, 0x2E80..=0x2FDF);
    if !text.chars().any(is_radical) {
        return Some(text);
    }
    let characters = || {
        text.chars().flat_map(|c| {
            let radical = is_radical(c);
            c.nfkc()
                .take(if radical { usize::MAX } else { 0 })
                .chain(std::iter::once(c).take(if radical { 0 } else { 1 }))
        })
    };
    let length = characters().try_fold(0usize, |size, c| size.checked_add(c.len_utf8()))?;
    let mut output = allocation.string_capacity(length)?;
    output.extend(characters());
    Some(output)
}

fn contains_ignore_ascii_case(text: &str, needle: &str) -> bool {
    text.as_bytes()
        .windows(needle.len())
        .any(|part| part.eq_ignore_ascii_case(needle.as_bytes()))
}

pub(crate) fn glyph_name_to_unicode(glyph_name: &str) -> Option<char> {
    // Priority 1: Adobe Glyph List (AGL) lookup - O(1) with perfect hash
    // PDF Spec: ISO 32000-1:2008, Section 9.10.2
    if let Some(&unicode_char) = super::adobe_glyph_list::ADOBE_GLYPH_LIST.get(glyph_name) {
        return Some(unicode_char);
    }

    // Priority 1b: Extended glyph names from TeX/math fonts (MSAM, MSBM, etc.)
    // These are well-known glyph names not in the standard AGL but common in
    // academic/mathematical PDFs generated by TeX/LaTeX.
    if let Some(&unicode_char) = TEX_MATH_GLYPH_NAMES.get(glyph_name) {
        return Some(unicode_char);
    }

    // Priority 2: Parse "uniXXXX" format (e.g., uni0041 -> A)
    // Common in custom fonts and font subsets
    if glyph_name.starts_with("uni") && glyph_name.len() == 7 {
        if let Ok(code_point) = u32::from_str_radix(&glyph_name[3..], 16) {
            if let Some(c) = char::from_u32(code_point) {
                return Some(c);
            }
        }
    }

    // Priority 3: Parse "uXXXX" format (e.g., u0041 -> A)
    // Alternative format used by some PDF generators
    if glyph_name.starts_with('u') && glyph_name.len() >= 5 {
        if let Ok(code_point) = u32::from_str_radix(&glyph_name[1..], 16) {
            if let Some(c) = char::from_u32(code_point) {
                return Some(c);
            }
        }
    }

    // Priority 4: Underscore-delimited compound glyph names (AGL spec section 2)
    // e.g. "f_f" → 'f'+'f', "f_i" → 'f'+'i', "T_h" → 'T'+'h'
    // Return the first component character for single-char return type
    if glyph_name.contains('_') {
        if let Some(first) = glyph_name.split('_').next() {
            if let Some(&ch) = super::adobe_glyph_list::ADOBE_GLYPH_LIST.get(first) {
                return Some(ch);
            }
        }
    }

    // Priority 5 (#535 follow-up): delegate to the unified fallback chain
    // in `character_mapper::glyph_name_to_unicode`. The newer chain adds:
    //   - Variant-suffix stripping (`A.sc`, `bullet.alt`, `fi.001`) — common in
    //     subset fonts where producers append stylistic-variant tags.
    //   - Stricter `uniXXXX` (exactly 4 hex, no control chars) and `uXXXXX`
    //     (4..6 hex, no surrogates, no control chars) validation.
    // This brings simple-font / Type1 / CFF / Differences-array callers (which
    // route through this `font_dict::glyph_name_to_unicode` entry) onto the
    // same fallback chain as the #535 Type0 Identity-H path. Inline-
    // image font streams (PDF spec §8.9.7) that resolve glyph names by this
    // path inherit the same behaviour transparently — no separate inline-image
    // codepath exists in this crate; inline images per spec carry only image
    // data, but any future inline-image font-resolution callsite will use this
    // unified chain by construction.
    if let Some(character) = super::character_mapper::glyph_name_to_unicode_char(glyph_name) {
        return Some(character);
    }

    // Unknown glyph name - not in AGL and not a recognized format
    log::debug!("Unknown glyph name not in Adobe Glyph List: '{}'", glyph_name);
    None
}

/// Resolve a glyph name to a Unicode string, handling compound names.
///
/// Like `glyph_name_to_unicode` but returns a full String for compound glyph names
/// (underscore-delimited per AGL spec, e.g. "f_f" → "ff", "f_f_i" → "ffi").
#[cfg(test)]
pub(crate) fn glyph_name_to_unicode_string(glyph_name: &str) -> Option<String> {
    glyph_name_to_unicode_string_with_budget(glyph_name, FontAllocation(None))
}

fn glyph_name_to_unicode_string_with_budget(
    glyph_name: &str,
    allocation: FontAllocation<'_>,
) -> Option<String> {
    allocation.check()?;
    // Try single char lookup first
    if let Some(ch) = glyph_name_to_unicode(glyph_name) {
        return allocation.character(ch);
    }

    // Handle underscore-delimited compound names (AGL spec section 2)
    if glyph_name.contains('_') {
        let length = glyph_name.split('_').try_fold(0usize, |size, part| {
            size.checked_add(glyph_name_to_unicode(part)?.len_utf8())
        })?;
        let mut result = allocation.string_capacity(length)?;
        for part in glyph_name.split('_') {
            // If any component is unknown, fail entirely.
            let ch = glyph_name_to_unicode(part)?;
            result.push(ch);
        }
        if !result.is_empty() {
            return Some(result);
        }
    }

    // Final fallback (#535 follow-up): unified chain — variant-suffix
    // stripping + strict uniXXXX / uXXXXX synth. Returns the full `String` shape
    // (multi-codepoint AGL entries are forwarded unchanged).
    super::character_mapper::glyph_name_to_unicode_char(glyph_name)
        .and_then(|c| allocation.character(c))
}

/// AGL Unicode for the closed set of punctuation glyph names that the Item 1
/// fix recovers (ISO 32000-1 §9.10.2(a)+(b)). Restricted deliberately to these
/// four names — generalising to all AGL names would re-introduce regression risk
/// against fonts whose ToUnicode is genuinely authoritative.
///
/// `period`→`"."`, `comma`→`","`, `hyphen`→`"-"`, `minus`→`"\u{2212}"`;
/// anything else → `None`.
fn punctuation_unicode_for_glyph_name(name: &str) -> Option<&'static str> {
    match name {
        "period" => Some("."),
        "comma" => Some(","),
        "hyphen" => Some("-"),
        "minus" => Some("\u{2212}"),
        _ => None,
    }
}

/// True iff `s` is a single character that is a "non-sensible symbol" — i.e. a
/// symbol/arrow/math glyph that is clearly not the punctuation a `period`/
/// `comma`/`hyphen`/`minus` glyph name denotes. This gates the Item 1
/// interceptions so they fire only when an upstream decode produced a wrong
/// symbol (e.g. U+00AC `¬` or an arrow/math char) for a punctuation-named code.
///
/// Covers the Latin-1 supplement symbol range (U+00A1..=U+00BF, which includes
/// U+00AC `¬`) and the arrow/math/symbol blocks (U+2190..=U+2BFF). Returns
/// `false` for `.`, `,`, `-`, ASCII digits, and any alphabetic letter.
fn is_non_sensible_symbol(s: &str) -> bool {
    let mut chars = s.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        // Empty or multi-char strings are not single non-sensible symbols.
        return false;
    };
    // Sensible by construction: letters and ASCII digits/punctuation.
    if c.is_alphabetic() || c.is_ascii_digit() || c.is_ascii_punctuation() {
        return false;
    }
    let cp = c as u32;
    // Latin-1 supplement symbol range (¬ = U+00AC, etc.) and the
    // arrow / mathematical-operator / misc-symbol blocks.
    matches!(cp, 0x00A1..=0x00BF | 0x2190..=0x2BFF)
}

// Removed old implementation - replaced with compact AGL lookup above
// Old code: ~350 lines of match arms with ~200 hardcoded glyphs
// New code: 4281 glyphs from official Adobe Glyph List via perfect hash map
#[allow(dead_code)]
fn _old_glyph_name_to_unicode_removed() {
    // This function body intentionally left empty.
    // The old match-based implementation has been replaced with
    // a lookup in the complete Adobe Glyph List static map.
    // See super::adobe_glyph_list::ADOBE_GLYPH_LIST for the new implementation.
}

// Old implementation removed - was 350+ lines of hardcoded match arms
// Now using complete Adobe Glyph List with 4281 entries from adobe_glyph_list module

/// Check if a character is a ligature.
///
/// This function identifies Unicode ligature characters (U+FB00 to U+FB06)
/// that are commonly used in PDFs for typographic ligatures.
///
/// # Arguments
///
/// * `c` - The character to check
///
/// # Returns
///
/// `true` if the character is a ligature, `false` otherwise.
///
/// # Examples
///
/// ```ignore
/// # use pdf_oxide::fonts::font_dict::is_ligature_char;
/// assert_eq!(is_ligature_char('ﬁ'), true); // U+FB01
/// assert_eq!(is_ligature_char('ﬂ'), true); // U+FB02
/// assert_eq!(is_ligature_char('A'), false);
/// ```ignore
fn is_ligature_char(c: char) -> bool {
    matches!(
        c,
        'ﬀ' |  // ff  - U+FB00
        'ﬁ' |  // fi  - U+FB01
        'ﬂ' |  // fl  - U+FB02
        'ﬃ' |  // ffi - U+FB03
        'ﬄ' |  // ffl - U+FB04
        'ﬅ' |  // st (long s + t) - U+FB05
        'ﬆ' // st - U+FB06
    )
}

/// Expand a ligature character to its ASCII equivalent.
///
/// This function handles the Unicode ligature characters (U+FB00 to U+FB06)
/// and expands them to their multi-character ASCII equivalents.
///
/// # Arguments
///
/// * `c` - The character to potentially expand
///
/// # Returns
///
/// The expanded string if `c` is a ligature, None otherwise.
///
/// # Examples
///
/// ```ignore
/// # use pdf_oxide::fonts::font_dict::expand_ligature_char;
/// assert_eq!(expand_ligature_char('ﬁ'), Some("fi"));
/// assert_eq!(expand_ligature_char('ﬂ'), Some("fl"));
/// assert_eq!(expand_ligature_char('A'), None);
/// ```ignore
fn expand_ligature_char(c: char) -> Option<&'static str> {
    match c {
        'ﬀ' => Some("ff"),  // U+FB00
        'ﬁ' => Some("fi"),  // U+FB01
        'ﬂ' => Some("fl"),  // U+FB02
        'ﬃ' => Some("ffi"), // U+FB03
        'ﬄ' => Some("ffl"), // U+FB04
        'ﬅ' => Some("st"),  // U+FB05 (long s + t)
        'ﬆ' => Some("st"),  // U+FB06
        _ => None,
    }
}

/// Expand a Unicode ligature character code to its ASCII equivalent.
///
/// This function handles the Unicode ligature character codes (U+FB00 to U+FB04)
/// and expands them to their multi-character ASCII equivalents.
///
/// This is the u16 character code variant, used in the character mapping priority chain
/// where character codes come as u16 values directly from the PDF.
///
/// # Arguments
///
/// * `char_code` - The character code (as u16) to potentially expand
///
/// # Returns
///
/// The expanded string if `char_code` is a ligature, None otherwise.
///
/// # Examples
/// Look up a character in the Adobe Symbol font encoding.
///
/// This function implements the Symbol font encoding table as defined in
/// PDF Specification Appendix D.4 (ISO 32000-1:2008, pages 996-997).
///
/// Symbol font is used extensively in mathematical and scientific documents
/// for Greek letters, mathematical operators, and special symbols.
///
/// # Arguments
///
/// * `code` - The character code (0-255)
///
/// # Returns
///
/// The corresponding Unicode character, or None if not in the encoding.
///
/// # References
///
/// - PDF 32000-1:2008, Appendix D.4 - Symbol Encoding
///
/// # Examples
///
/// ```ignore
/// # use pdf_oxide::fonts::font_dict::symbol_encoding_lookup;
/// assert_eq!(symbol_encoding_lookup(0x72), Some('ρ')); // rho
/// assert_eq!(symbol_encoding_lookup(0x61), Some('α')); // alpha
/// assert_eq!(symbol_encoding_lookup(0xF2), Some('∫')); // integral
/// ```ignore
fn symbol_encoding_lookup(code: u8) -> Option<char> {
    match code {
        // Greek lowercase letters
        0x61 => Some('α'), // alpha
        0x62 => Some('β'), // beta
        0x63 => Some('χ'), // chi
        0x64 => Some('δ'), // delta
        0x65 => Some('ε'), // epsilon
        0x66 => Some('φ'), // phi
        0x67 => Some('γ'), // gamma
        0x68 => Some('η'), // eta
        0x69 => Some('ι'), // iota
        0x6A => Some('ϕ'), // phi1 (variant)
        0x6B => Some('κ'), // kappa
        0x6C => Some('λ'), // lambda
        0x6D => Some('μ'), // mu
        0x6E => Some('ν'), // nu
        0x6F => Some('ο'), // omicron
        0x70 => Some('π'), // pi
        0x71 => Some('θ'), // theta
        0x72 => Some('ρ'), // rho ← THE IMPORTANT ONE for Pearson's ρ!
        0x73 => Some('σ'), // sigma
        0x74 => Some('τ'), // tau
        0x75 => Some('υ'), // upsilon
        0x76 => Some('ϖ'), // omega1 (variant pi)
        0x77 => Some('ω'), // omega
        0x78 => Some('ξ'), // xi
        0x79 => Some('ψ'), // psi
        0x7A => Some('ζ'), // zeta

        // Greek uppercase letters
        0x41 => Some('Α'), // Alpha
        0x42 => Some('Β'), // Beta
        0x43 => Some('Χ'), // Chi
        0x44 => Some('Δ'), // Delta
        0x45 => Some('Ε'), // Epsilon
        0x46 => Some('Φ'), // Phi
        0x47 => Some('Γ'), // Gamma
        0x48 => Some('Η'), // Eta
        0x49 => Some('Ι'), // Iota
        0x4B => Some('Κ'), // Kappa
        0x4C => Some('Λ'), // Lambda
        0x4D => Some('Μ'), // Mu
        0x4E => Some('Ν'), // Nu
        0x4F => Some('Ο'), // Omicron
        0x50 => Some('Π'), // Pi
        0x51 => Some('Θ'), // Theta
        0x52 => Some('Ρ'), // Rho
        0x53 => Some('Σ'), // Sigma
        0x54 => Some('Τ'), // Tau
        0x55 => Some('Υ'), // Upsilon
        0x57 => Some('Ω'), // Omega
        0x58 => Some('Ξ'), // Xi
        0x59 => Some('Ψ'), // Psi
        0x5A => Some('Ζ'), // Zeta

        // Mathematical operators
        0xB1 => Some('±'), // plusminus
        0xB4 => Some('÷'), // divide
        0xB5 => Some('∞'), // infinity
        0xB6 => Some('∂'), // partialdiff
        0xB7 => Some('•'), // bullet
        0xB9 => Some('≠'), // notequal
        0xBA => Some('≡'), // equivalence
        0xBB => Some('≈'), // approxequal
        0xBC => Some('…'), // ellipsis
        0xBE => Some('⊥'), // perpendicular
        0xBF => Some('⊙'), // circleplus

        0xD0 => Some('°'), // degree
        0xD1 => Some('∇'), // gradient (nabla)
        0xD2 => Some('¬'), // logicalnot
        0xD3 => Some('∧'), // logicaland
        0xD4 => Some('∨'), // logicalor
        0xD5 => Some('∏'), // product ← Product symbol!
        0xD6 => Some('√'), // radical ← Square root!
        0xD7 => Some('⋅'), // dotmath
        0xD8 => Some('⊕'), // circleplus
        0xD9 => Some('⊗'), // circletimes

        0xDA => Some('∈'), // element
        0xDB => Some('∉'), // notelement
        0xDC => Some('∠'), // angle
        0xDD => Some('∇'), // gradient
        0xDE => Some('®'), // registered
        0xDF => Some('©'), // copyright
        0xE0 => Some('™'), // trademark

        0xE1 => Some('∑'), // summation ← Summation symbol!
        0xE2 => Some('⊂'), // propersubset
        0xE3 => Some('⊃'), // propersuperset
        0xE4 => Some('⊆'), // reflexsubset
        0xE5 => Some('⊇'), // reflexsuperset
        0xE6 => Some('∪'), // union
        0xE7 => Some('∩'), // intersection
        0xE8 => Some('∀'), // universal
        0xE9 => Some('∃'), // existential
        0xEA => Some('¬'), // logicalnot

        0xF1 => Some('〈'), // angleleft
        0xF2 => Some('∫'),  // integral ← Integral symbol!
        0xF3 => Some('⌠'),  // integraltp
        0xF4 => Some('⌡'),  // integralbt
        0xF5 => Some('⊓'),  // square intersection
        0xF6 => Some('⊔'),  // square union
        0xF7 => Some('〉'), // angleright

        // Basic punctuation and symbols (overlap with ASCII)
        0x20 => Some(' '), // space
        0x21 => Some('!'), // exclam
        0x22 => Some('∀'), // universal (sometimes mapped here)
        0x23 => Some('#'), // numbersign
        0x24 => Some('∃'), // existential (sometimes mapped here)
        0x25 => Some('%'), // percent
        0x26 => Some('&'), // ampersand
        0x27 => Some('∋'), // suchthat
        0x28 => Some('('), // parenleft
        0x29 => Some(')'), // parenright
        0x2A => Some('∗'), // asteriskmath
        0x2B => Some('+'), // plus
        0x2C => Some(','), // comma
        0x2D => Some('−'), // minus
        0x2E => Some('.'), // period
        0x2F => Some('/'), // slash

        // Digits 0-9 (0x30-0x39) map to themselves
        0x30..=0x39 => Some(code as char),

        0x3A => Some(':'), // colon
        0x3B => Some(';'), // semicolon
        0x3C => Some('<'), // less
        0x3D => Some('='), // equal
        0x3E => Some('>'), // greater
        0x3F => Some('?'), // question

        0x40 => Some('≅'), // congruent

        // Brackets and arrows
        0x5B => Some('['), // bracketleft
        0x5C => Some('∴'), // therefore
        0x5D => Some(']'), // bracketright
        0x5E => Some('⊥'), // perpendicular
        0x5F => Some('_'), // underscore

        0x7B => Some('{'), // braceleft
        0x7C => Some('|'), // bar
        0x7D => Some('}'), // braceright
        0x7E => Some('∼'), // similar

        // Math operators previously missing from the Adobe Symbol set (Annex D.5).
        0xA3 => Some('\u{2264}'), // ≤ lessequal    (octal 243)
        0xA5 => Some('\u{221E}'), // ∞ infinity     (octal 245)
        0xB3 => Some('\u{2265}'), // ≥ greaterequal (octal 263)

        _ => None,
    }
}

/// Look up a character in the Adobe ZapfDingbats font encoding.
///
/// This function implements a subset of the ZapfDingbats font encoding table
/// as defined in PDF Specification Appendix D.5 (ISO 32000-1:2008, page 998).
///
/// ZapfDingbats font is used for ornamental symbols, arrows, and decorative characters.
///
/// # Arguments
///
/// * `code` - The character code (0-255)
///
/// # Returns
///
/// The corresponding Unicode character, or None if not in the encoding.
///
/// # References
///
/// - PDF 32000-1:2008, Appendix D.5 - ZapfDingbats Encoding
fn zapf_dingbats_encoding_lookup(code: u8) -> Option<char> {
    match code {
        0x20 => Some(' '), // space
        0x21 => Some('✁'), // scissors
        0x22 => Some('✂'), // scissors (filled)
        0x23 => Some('✃'), // scissors (outline)
        0x24 => Some('✄'), // scissors (small)
        0x25 => Some('☎'), // telephone
        0x26 => Some('✆'), // telephone (filled)
        0x27 => Some('✇'), // tape drive
        0x28 => Some('✈'), // airplane
        0x29 => Some('✉'), // envelope
        0x2A => Some('☛'), // hand pointing right
        0x2B => Some('☞'), // hand pointing right (filled)
        0x2C => Some('✌'), // victory hand
        0x2D => Some('✍'), // writing hand
        0x2E => Some('✎'), // pencil
        0x2F => Some('✏'), // pencil (filled)

        0x30 => Some('✐'), // pen nib
        0x31 => Some('✑'), // pen nib (filled)
        0x32 => Some('✒'), // pen nib (outline)
        0x33 => Some('✓'), // checkmark
        0x34 => Some('✔'), // checkmark (bold)
        0x35 => Some('✕'), // multiplication X
        0x36 => Some('✖'), // multiplication X (heavy)
        0x37 => Some('✗'), // ballot X
        0x38 => Some('✘'), // ballot X (heavy)
        0x39 => Some('✙'), // outlined Greek cross
        0x3A => Some('✚'), // heavy Greek cross
        0x3B => Some('✛'), // open center cross
        0x3C => Some('✜'), // heavy open center cross
        0x3D => Some('✝'), // Latin cross
        0x3E => Some('✞'), // Latin cross (shadowed)
        0x3F => Some('✟'), // Latin cross (outline)

        // Common symbols
        0x40 => Some('✠'), // Maltese cross
        0x41 => Some('✡'), // Star of David
        0x42 => Some('✢'), // four teardrop-spoked asterisk
        0x43 => Some('✣'), // four balloon-spoked asterisk
        0x44 => Some('✤'), // heavy four balloon-spoked asterisk
        0x45 => Some('✥'), // four club-spoked asterisk
        0x46 => Some('✦'), // black four pointed star
        0x47 => Some('✧'), // white four pointed star
        0x48 => Some('★'), // black star
        0x49 => Some('✩'), // outlined black star
        0x4A => Some('✪'), // circled white star
        0x4B => Some('✫'), // circled black star
        0x4C => Some('✬'), // shadowed white star
        0x4D => Some('✭'), // heavy asterisk
        0x4E => Some('✮'), // eight spoke asterisk
        0x4F => Some('✯'), // eight pointed black star

        // More ornaments
        0x50 => Some('✰'), // eight pointed pinwheel star
        0x51 => Some('✱'), // heavy eight pointed pinwheel star
        0x52 => Some('✲'), // eight pointed star
        0x53 => Some('✳'), // eight pointed star (outlined)
        0x54 => Some('✴'), // eight pointed star (heavy)
        0x55 => Some('✵'), // six pointed black star
        0x56 => Some('✶'), // six pointed star
        0x57 => Some('✷'), // eight pointed star (black)
        0x58 => Some('✸'), // heavy eight pointed star
        0x59 => Some('✹'), // twelve pointed black star
        0x5A => Some('✺'), // sixteen pointed star
        0x5B => Some('✻'), // teardrop-spoked asterisk
        0x5C => Some('✼'), // open center teardrop-spoked asterisk
        0x5D => Some('✽'), // heavy teardrop-spoked asterisk
        0x5E => Some('✾'), // six petalled black and white florette
        0x5F => Some('✿'), // black florette

        // Geometric shapes
        0x60 => Some('❀'), // white florette
        0x61 => Some('❁'), // eight petalled outlined black florette
        0x62 => Some('❂'), // circled open center eight pointed star
        0x63 => Some('❃'), // heavy teardrop-spoked pinwheel asterisk
        0x64 => Some('❄'), // snowflake
        0x65 => Some('❅'), // tight trifoliate snowflake
        0x66 => Some('❆'), // heavy chevron snowflake
        0x67 => Some('❇'), // sparkle
        0x68 => Some('❈'), // heavy sparkle
        0x69 => Some('❉'), // balloon-spoked asterisk
        0x6A => Some('❊'), // eight teardrop-spoked propeller asterisk
        0x6B => Some('❋'), // heavy eight teardrop-spoked propeller asterisk

        // Arrows
        0x6C => Some('●'), // black circle
        0x6D => Some('○'), // white circle
        0x6E => Some('❍'), // shadowed white circle
        0x6F => Some('■'), // black square
        0x70 => Some('□'), // white square
        0x71 => Some('▢'), // white square with rounded corners
        0x72 => Some('▣'), // white square containing black small square
        0x73 => Some('▤'), // square with horizontal fill
        0x74 => Some('▥'), // square with vertical fill
        0x75 => Some('▦'), // square with orthogonal crosshatch fill
        0x76 => Some('▧'), // square with upper left to lower right fill
        0x77 => Some('▨'), // square with upper right to lower left fill
        0x78 => Some('▩'), // square with diagonal crosshatch fill
        0x79 => Some('▪'), // black small square
        0x7A => Some('▫'), // white small square

        // Circled digits (Annex D.6, octal 254–323), previously dropped. Codes
        // are the spec's octal CODE in hex; each range is contiguous in Unicode.
        0xAC..=0xB5 => char::from_u32(0x2460 + (code as u32 - 0xAC)), // ① ⑩  a120 a129
        0xB6..=0xBF => char::from_u32(0x2776 + (code as u32 - 0xB6)), // ❶ ❿  a130 a139
        0xC0..=0xC9 => char::from_u32(0x2780 + (code as u32 - 0xC0)), // ➀ ➉  a140 a149
        0xCA..=0xD3 => char::from_u32(0x278A + (code as u32 - 0xCA)), // ➊ ➓  a150 a159

        // Arrows (Annex D.6, octal 324–376): four singletons, then two runs.
        0xD4 => Some('\u{2794}'), // ➔ a160  heavy wide-headed rightwards arrow
        0xD5 => Some('\u{2192}'), // → a161  rightwards arrow
        0xD6 => Some('\u{2194}'), // ↔ a163  left right arrow
        0xD7 => Some('\u{2195}'), // ↕ a164  up down arrow
        0xD8..=0xEF => char::from_u32(0x2798 + (code as u32 - 0xD8)), // ➘ ➯  a196…a182
        0xF1..=0xFE => char::from_u32(0x27B1 + (code as u32 - 0xF1)), // ➱ ➾  a201…a191

        _ => None,
    }
}

/// Look up a character in PDFDocEncoding.
///
/// PDFDocEncoding is a superset of ISO Latin-1 used as the default encoding
/// for PDF text strings and metadata (bookmarks, annotations, document info).
///
/// Codes 0-127 are identical to ASCII.
/// Codes 128-159 have special mappings (different from ISO Latin-1).
/// Codes 160-255 are identical to ISO Latin-1.
///
/// # PDF Spec Reference
///
/// ISO 32000-1:2008, Appendix D.2, Table D.2, page 994
///
/// # Arguments
///
/// * `code` - The byte code to look up (0-255)
///
/// # Returns
///
/// The Unicode character for this code, or None for undefined codes
pub fn pdfdoc_encoding_lookup(code: u8) -> Option<char> {
    match code {
        // ASCII range (0-127)
        0x00..=0x7F => Some(code as char),

        // PDFDocEncoding special range (128-159)
        0x80 => Some('•'),        // bullet
        0x81 => Some('†'),        // dagger
        0x82 => Some('‡'),        // daggerdbl
        0x83 => Some('…'),        // ellipsis
        0x84 => Some('—'),        // emdash
        0x85 => Some('–'),        // endash
        0x86 => Some('ƒ'),        // florin
        0x87 => Some('⁄'),        // fraction
        0x88 => Some('‹'),        // guilsinglleft
        0x89 => Some('›'),        // guilsinglright
        0x8A => Some('−'),        // minus (different from hyphen!)
        0x8B => Some('‰'),        // perthousand
        0x8C => Some('„'),        // quotedblbase
        0x8D => Some('"'),        // quotedblleft
        0x8E => Some('"'),        // quotedblright
        0x8F => Some('\u{2018}'), // quoteleft (left single quotation mark)
        0x90 => Some('\u{2019}'), // quoteright (right single quotation mark)
        0x91 => Some('‚'),        // quotesinglbase
        0x92 => Some('™'),        // trademark
        0x93 => Some('ﬁ'),        // fi ligature
        0x94 => Some('ﬂ'),        // fl ligature
        0x95 => Some('Ł'),        // Lslash
        0x96 => Some('Œ'),        // OE
        0x97 => Some('Š'),        // Scaron
        0x98 => Some('Ÿ'),        // Ydieresis
        0x99 => Some('Ž'),        // Zcaron
        0x9A => Some('ı'),        // dotlessi
        0x9B => Some('ł'),        // lslash
        0x9C => Some('œ'),        // oe
        0x9D => Some('š'),        // scaron
        0x9E => Some('ž'),        // zcaron
        0x9F => None,             // undefined

        // ISO Latin-1 range (160-255) - direct mapping
        0xA0..=0xFF => Some(code as char),
    }
}

/// Look up a character in a standard PDF encoding.
///
/// This function provides support for standard PDF encodings including
/// PDFDocEncoding, WinAnsiEncoding, StandardEncoding, and MacRomanEncoding.
///
/// # Arguments
///
/// * `encoding` - The encoding name (e.g., "WinAnsiEncoding", "PDFDocEncoding")
/// * `code` - The character code (0-255)
///
/// # Returns
///
/// Whether an embedded font program's built-in `/Encoding` (`prog_enc`,
/// code→char) looks like a re-indexed subset **cipher** rather than a
/// meaningful text encoding to overlay on the producer-declared named base
/// `std_name`.
///
/// A real encoding (a few non-standard slots over a named base, e.g. space at
/// 0xCA) agrees with the named base on most of the codes they share; a subset
/// cipher — the font's own arbitrary glyph ordering — agrees on almost none,
/// and overlaying it would rewrite every mapped code into mojibake. Decide by
/// agreement: of the codes present in both, fewer than half resolving to the
/// same character means cipher. Empty overlap is treated as NOT a cipher (no
/// evidence either way; keep the prior overlay behaviour).
#[cfg(test)]
fn builtin_encoding_looks_like_cipher(prog_enc: &HashMap<u8, char>, std_name: &str) -> bool {
    builtin_encoding_looks_like_cipher_with_budget(prog_enc, std_name, FontAllocation(None))
}

fn builtin_encoding_looks_like_cipher_with_budget(
    prog_enc: &HashMap<u8, char>,
    std_name: &str,
    allocation: FontAllocation<'_>,
) -> bool {
    let (mut agree, mut overlap) = (0u32, 0u32);
    for (&code, &ch) in prog_enc {
        if let Some(us) = standard_encoding_lookup_with_budget(std_name, code, allocation) {
            if let Some(sc) = us.chars().next() {
                overlap += 1;
                if sc == ch {
                    agree += 1;
                }
            }
        }
    }
    overlap > 0 && (agree as f32 / overlap as f32) < 0.5
}

/// The Unicode string for this character, or None if not in the encoding.
#[cfg(test)]
fn standard_encoding_lookup(encoding: &str, code: u8) -> Option<String> {
    standard_encoding_lookup_with_budget(encoding, code, FontAllocation(None))
}

fn standard_encoding_lookup_with_budget(
    encoding: &str,
    code: u8,
    allocation: FontAllocation<'_>,
) -> Option<String> {
    allocation.check()?;
    match encoding {
        "PDFDocEncoding" => {
            // PDFDocEncoding: superset of ISO Latin-1 with special 128-159 range
            pdfdoc_encoding_lookup(code).and_then(|c| allocation.character(c))
        },
        "WinAnsiEncoding" => {
            // ASCII printable range (32-126)
            if (32..=126).contains(&code) {
                return allocation.character(code as char);
            }

            // WinAnsiEncoding extended range (128-255)
            // Based on Windows-1252 encoding
            let unicode = match code {
                0x80 => '\u{20AC}', // Euro sign
                0x82 => '\u{201A}', // Single low-9 quotation mark
                0x83 => '\u{0192}', // Latin small letter f with hook
                0x84 => '\u{201E}', // Double low-9 quotation mark
                0x85 => '\u{2026}', // Horizontal ellipsis
                0x86 => '\u{2020}', // Dagger
                0x87 => '\u{2021}', // Double dagger
                0x88 => '\u{02C6}', // Modifier letter circumflex accent
                0x89 => '\u{2030}', // Per mille sign
                0x8A => '\u{0160}', // Latin capital letter S with caron
                0x8B => '\u{2039}', // Single left-pointing angle quotation mark
                0x8C => '\u{0152}', // Latin capital ligature OE
                0x8E => '\u{017D}', // Latin capital letter Z with caron
                0x91 => '\u{2018}', // Left single quotation mark
                0x92 => '\u{2019}', // Right single quotation mark
                0x93 => '\u{201C}', // Left double quotation mark
                0x94 => '\u{201D}', // Right double quotation mark
                0x95 => '\u{2022}', // Bullet
                0x96 => '\u{2013}', // En dash
                0x97 => '\u{2014}', // Em dash
                0x98 => '\u{02DC}', // Small tilde
                0x99 => '\u{2122}', // Trade mark sign
                0x9A => '\u{0161}', // Latin small letter s with caron
                0x9B => '\u{203A}', // Single right-pointing angle quotation mark
                0x9C => '\u{0153}', // Latin small ligature oe
                0x9E => '\u{017E}', // Latin small letter z with caron
                0x9F => '\u{0178}', // Latin capital letter Y with diaeresis
                // 0xA0-0xFF: Direct mapping to Unicode (ISO-8859-1)
                _ if code >= 0xA0 => char::from_u32(code as u32)?,
                _ => return None,
            };
            allocation.character(unicode)
        },
        "StandardEncoding" => {
            // PostScript StandardEncoding per PDF Spec ISO 32000-1:2008, Annex D, Table D.1
            // NOTE: StandardEncoding differs significantly from ISO-8859-1 in the 0xA0-0xFF range.
            // Using ISO-8859-1 fallback here would produce wrong characters for ligatures,
            // smart quotes, accents, and other typographic characters.
            if (32..=126).contains(&code) {
                // Most codes in 32–126 match ASCII, with one notable exception:
                // 0x27 = "quoteright" → U+2019 (RIGHT SINGLE QUOTATION MARK)
                // All other printable ASCII codes are identity-mapped.
                let ch = match code {
                    0x27 => '\u{2019}', // quoteright
                    _ => code as char,
                };
                allocation.character(ch)
            } else {
                let unicode = match code {
                    // 0xA0-0xAF
                    0xA1 => '\u{00A1}', // exclamdown
                    0xA2 => '\u{00A2}', // cent
                    0xA3 => '\u{00A3}', // sterling
                    0xA4 => '\u{2044}', // fraction (NOT currency ¤)
                    0xA5 => '\u{00A5}', // yen
                    0xA6 => '\u{0192}', // florin (NOT broken bar)
                    0xA7 => '\u{00A7}', // section
                    0xA8 => '\u{00A4}', // currency (NOT dieresis)
                    0xA9 => '\u{0027}', // quotesingle (NOT copyright)
                    0xAA => '\u{201C}', // quotedblleft (NOT ordfeminine)
                    0xAB => '\u{00AB}', // guillemotleft
                    0xAC => '\u{2039}', // guilsinglleft (NOT not-sign)
                    0xAD => '\u{203A}', // guilsinglright (NOT soft-hyphen)
                    0xAE => '\u{FB01}', // fi ligature (NOT registered)
                    0xAF => '\u{FB02}', // fl ligature (NOT macron)
                    // 0xB0-0xBF
                    0xB1 => '\u{2013}', // endash (NOT plus-minus)
                    0xB2 => '\u{2020}', // dagger (NOT superscript 2)
                    0xB3 => '\u{2021}', // daggerdbl (NOT superscript 3)
                    0xB4 => '\u{00B7}', // periodcentered (NOT acute accent)
                    0xB6 => '\u{00B6}', // paragraph
                    0xB7 => '\u{2022}', // bullet (NOT middle dot)
                    0xB8 => '\u{201A}', // quotesinglbase (NOT cedilla)
                    0xB9 => '\u{201E}', // quotedblbase (NOT superscript 1)
                    0xBA => '\u{201D}', // quotedblright (NOT ordmasculine)
                    0xBB => '\u{00BB}', // guillemotright
                    0xBC => '\u{2026}', // ellipsis (NOT one quarter)
                    0xBD => '\u{2030}', // perthousand (NOT one half)
                    0xBF => '\u{00BF}', // questiondown
                    // 0xC0-0xCF — accent marks and modifiers
                    0xC1 => '\u{0060}', // grave (NOT A-grave)
                    0xC2 => '\u{00B4}', // acute (NOT A-circumflex)
                    0xC3 => '\u{02C6}', // circumflex (NOT A-tilde)
                    0xC4 => '\u{02DC}', // tilde (NOT A-dieresis)
                    0xC5 => '\u{00AF}', // macron (NOT A-ring)
                    0xC6 => '\u{02D8}', // breve (NOT AE)
                    0xC7 => '\u{02D9}', // dotaccent (NOT C-cedilla)
                    0xC8 => '\u{00A8}', // dieresis (NOT E-grave)
                    0xCA => '\u{02DA}', // ring (NOT E-circumflex)
                    0xCB => '\u{00B8}', // cedilla (NOT E-dieresis)
                    0xCD => '\u{02DD}', // hungarumlaut (NOT I-acute)
                    0xCE => '\u{02DB}', // ogonek (NOT I-circumflex)
                    0xCF => '\u{02C7}', // caron (NOT I-dieresis)
                    // 0xD0 — em dash
                    0xD0 => '\u{2014}', // emdash (NOT Eth)
                    // 0xE0-0xEF — uppercase special chars
                    0xE1 => '\u{00C6}', // AE (NOT a-acute)
                    0xE3 => '\u{00AA}', // ordfeminine (NOT a-tilde)
                    0xE8 => '\u{0141}', // Lslash (NOT e-grave)
                    0xE9 => '\u{00D8}', // Oslash (NOT e-acute)
                    0xEA => '\u{0152}', // OE (NOT e-circumflex)
                    0xEB => '\u{00BA}', // ordmasculine (NOT e-dieresis)
                    // 0xF0-0xFF — lowercase special chars
                    0xF1 => '\u{00E6}', // ae (NOT n-tilde)
                    0xF5 => '\u{0131}', // dotlessi (NOT o-tilde)
                    0xF8 => '\u{0142}', // lslash (NOT o-stroke)
                    0xF9 => '\u{00F8}', // oslash (NOT u-grave)
                    0xFA => '\u{0153}', // oe (NOT u-acute)
                    0xFB => '\u{00DF}', // germandbls (NOT u-circumflex)
                    _ => return None,
                };
                allocation.character(unicode)
            }
        },
        "MacRomanEncoding" => {
            // ASCII range is the same
            if (32..=126).contains(&code) {
                allocation.character(code as char)
            } else {
                // Complete Mac OS Roman encoding per PDF Spec ISO 32000-1:2008, Annex D, Table D.2
                let unicode = match code {
                    // 0x80-0x9F: Accented letters
                    0x80 => '\u{00C4}', // Adieresis
                    0x81 => '\u{00C5}', // Aring
                    0x82 => '\u{00C7}', // Ccedilla
                    0x83 => '\u{00C9}', // Eacute
                    0x84 => '\u{00D1}', // Ntilde
                    0x85 => '\u{00D6}', // Odieresis
                    0x86 => '\u{00DC}', // Udieresis
                    0x87 => '\u{00E1}', // aacute
                    0x88 => '\u{00E0}', // agrave
                    0x89 => '\u{00E2}', // acircumflex
                    0x8A => '\u{00E4}', // adieresis
                    0x8B => '\u{00E3}', // atilde
                    0x8C => '\u{00E5}', // aring
                    0x8D => '\u{00E7}', // ccedilla
                    0x8E => '\u{00E9}', // eacute
                    0x8F => '\u{00E8}', // egrave
                    0x90 => '\u{00EA}', // ecircumflex
                    0x91 => '\u{00EB}', // edieresis
                    0x92 => '\u{00ED}', // iacute
                    0x93 => '\u{00EC}', // igrave
                    0x94 => '\u{00EE}', // icircumflex
                    0x95 => '\u{00EF}', // idieresis
                    0x96 => '\u{00F1}', // ntilde
                    0x97 => '\u{00F3}', // oacute
                    0x98 => '\u{00F2}', // ograve
                    0x99 => '\u{00F4}', // ocircumflex
                    0x9A => '\u{00F6}', // odieresis
                    0x9B => '\u{00F5}', // otilde
                    0x9C => '\u{00FA}', // uacute
                    0x9D => '\u{00F9}', // ugrave
                    0x9E => '\u{00FB}', // ucircumflex
                    0x9F => '\u{00FC}', // udieresis
                    // 0xA0-0xBF: Symbols and punctuation (NOT Latin-1!)
                    0xA0 => '\u{2020}', // dagger (NOT NBSP)
                    0xA1 => '\u{00B0}', // degree (NOT inverted exclamation)
                    0xA2 => '\u{00A2}', // cent
                    0xA3 => '\u{00A3}', // sterling
                    0xA4 => '\u{00A7}', // section (NOT currency sign)
                    0xA5 => '\u{2022}', // bullet (NOT yen)
                    0xA6 => '\u{00B6}', // paragraph (NOT broken bar)
                    0xA7 => '\u{00DF}', // germandbls (NOT section)
                    0xA8 => '\u{00AE}', // registered (NOT dieresis)
                    0xA9 => '\u{00A9}', // copyright
                    0xAA => '\u{2122}', // trademark (NOT ordfeminine)
                    0xAB => '\u{00B4}', // acute (NOT guillemotleft)
                    0xAC => '\u{00A8}', // dieresis (NOT logical not)
                    0xAD => '\u{2260}', // notequal (NOT soft hyphen)
                    0xAE => '\u{00C6}', // AE (NOT registered)
                    0xAF => '\u{00D8}', // Oslash (NOT macron)
                    0xB0 => '\u{221E}', // infinity (NOT degree)
                    0xB1 => '\u{00B1}', // plusminus
                    0xB2 => '\u{2264}', // lessequal (NOT superscript 2)
                    0xB3 => '\u{2265}', // greaterequal (NOT superscript 3)
                    0xB4 => '\u{00A5}', // yen (NOT acute)
                    0xB5 => '\u{00B5}', // mu
                    0xB6 => '\u{2202}', // partialdiff (NOT paragraph)
                    0xB7 => '\u{2211}', // summation (NOT middle dot)
                    0xB8 => '\u{220F}', // product (NOT cedilla)
                    0xB9 => '\u{03C0}', // pi (NOT superscript 1)
                    0xBA => '\u{222B}', // integral (NOT ordmasculine)
                    0xBB => '\u{00AA}', // ordfeminine (NOT guillemotright)
                    0xBC => '\u{00BA}', // ordmasculine (NOT one quarter)
                    0xBD => '\u{2126}', // Omega (NOT one half)
                    0xBE => '\u{00E6}', // ae (NOT three quarters)
                    0xBF => '\u{00F8}', // oslash (NOT inverted question)
                    // 0xC0-0xCF: More symbols and accented capitals
                    0xC0 => '\u{00BF}', // questiondown
                    0xC1 => '\u{00A1}', // exclamdown
                    0xC2 => '\u{00AC}', // logicalnot
                    0xC3 => '\u{221A}', // radical
                    0xC4 => '\u{0192}', // florin
                    0xC5 => '\u{2248}', // approxequal
                    0xC6 => '\u{2206}', // Delta
                    0xC7 => '\u{00AB}', // guillemotleft
                    0xC8 => '\u{00BB}', // guillemotright
                    0xC9 => '\u{2026}', // ellipsis
                    0xCA => '\u{00A0}', // nonbreakingspace
                    0xCB => '\u{00C0}', // Agrave
                    0xCC => '\u{00C3}', // Atilde
                    0xCD => '\u{00D5}', // Otilde
                    0xCE => '\u{0152}', // OE
                    0xCF => '\u{0153}', // oe
                    // 0xD0-0xDF: Dashes, quotes, ligatures
                    0xD0 => '\u{2013}', // endash
                    0xD1 => '\u{2014}', // emdash
                    0xD2 => '\u{201C}', // quotedblleft
                    0xD3 => '\u{201D}', // quotedblright
                    0xD4 => '\u{2018}', // quoteleft
                    0xD5 => '\u{2019}', // quoteright
                    0xD6 => '\u{00F7}', // divide
                    0xD7 => '\u{25CA}', // lozenge
                    0xD8 => '\u{00FF}', // ydieresis
                    0xD9 => '\u{0178}', // Ydieresis
                    0xDA => '\u{2044}', // fraction
                    0xDB => '\u{20AC}', // Euro
                    0xDC => '\u{2039}', // guilsinglleft
                    0xDD => '\u{203A}', // guilsinglright
                    0xDE => '\u{FB01}', // fi ligature
                    0xDF => '\u{FB02}', // fl ligature
                    // 0xE0-0xEF: More symbols and accented capitals
                    0xE0 => '\u{2021}', // daggerdbl
                    0xE1 => '\u{00B7}', // periodcentered
                    0xE2 => '\u{201A}', // quotesinglbase
                    0xE3 => '\u{201E}', // quotedblbase
                    0xE4 => '\u{2030}', // perthousand
                    0xE5 => '\u{00C2}', // Acircumflex
                    0xE6 => '\u{00CA}', // Ecircumflex
                    0xE7 => '\u{00C1}', // Aacute
                    0xE8 => '\u{00CB}', // Edieresis
                    0xE9 => '\u{00C8}', // Egrave
                    0xEA => '\u{00CD}', // Iacute
                    0xEB => '\u{00CE}', // Icircumflex
                    0xEC => '\u{00CF}', // Idieresis
                    0xED => '\u{00CC}', // Igrave
                    0xEE => '\u{00D3}', // Oacute
                    0xEF => '\u{00D4}', // Ocircumflex
                    // 0xF0-0xFF: More accented and special chars
                    0xF0 => '\u{F8FF}', // Apple logo (private use area)
                    0xF1 => '\u{00D2}', // Ograve
                    0xF2 => '\u{00DA}', // Uacute
                    0xF3 => '\u{00DB}', // Ucircumflex
                    0xF4 => '\u{00D9}', // Ugrave
                    0xF5 => '\u{0131}', // dotlessi
                    0xF6 => '\u{02C6}', // circumflex
                    0xF7 => '\u{02DC}', // tilde
                    0xF8 => '\u{00AF}', // macron
                    0xF9 => '\u{02D8}', // breve
                    0xFA => '\u{02D9}', // dotaccent
                    0xFB => '\u{02DA}', // ring
                    0xFC => '\u{00B8}', // cedilla
                    0xFD => '\u{02DD}', // hungarumlaut
                    0xFE => '\u{02DB}', // ogonek
                    0xFF => '\u{02C7}', // caron
                    _ => return None,
                };
                allocation.character(unicode)
            }
        },
        _ => {
            // Unknown encoding, try identity mapping for ASCII
            if code.is_ascii() && code >= 32 {
                allocation.character(code as char)
            } else {
                None
            }
        },
    }
}

/// Decode a raw CJK multi-byte character code to Unicode using legacy encodings.
///
/// For Type0 fonts using named CJK CMaps (e.g., "GBK-EUC-H", "GB-EUC-H",
/// "ETen-B5-H", "EUC-H", "KSC-EUC-H"), the 2-byte value read from the content
/// stream is NOT an Adobe CID — it is a raw multi-byte encoding value (GBK,
/// EUC-CN, Big5, EUC-JP, or EUC-KR). Adobe-GB1 CIDs cap at ~30 553, so
/// `lookup_predefined_cmap` always returns None for GBK values ≥ 0xA1A1,
/// the caller falls through to a broken `char::from_u32` path that maps them
/// to Korean Hangul (same code-point range).
///
/// This function catches that case and decodes with encoding_rs so the correct
/// CJK characters come out.
#[cfg(test)]
fn decode_cjk_raw_charcode(
    char_code: u32,
    enc_name: &str,
    cid_system_info: &Option<CIDSystemInfo>,
) -> Option<String> {
    decode_cjk_raw_charcode_with_budget(char_code, enc_name, cid_system_info, FontAllocation(None))
}

fn decode_cjk_raw_charcode_with_budget(
    char_code: u32,
    enc_name: &str,
    cid_system_info: &Option<CIDSystemInfo>,
    allocation: FontAllocation<'_>,
) -> Option<String> {
    let ordering = cid_system_info
        .as_ref()
        .map(|i| i.ordering.as_str())
        .unwrap_or("");

    // CORPUS-3: the bare Adobe predefined CMaps "H"/"V" are (overwhelmingly)
    // Adobe-Japan1-H/V and carry JIS X 0208 codes in GL form (both bytes
    // 0x21–0x7E). encoding_rs decodes EUC-JP (high bit set), so lift GL→EUC by
    // OR-ing 0x8080, then decode. Recovers non-embedded Japanese (noembed-jis7:
    // "あいうえお" was emitted as garbage "CACCCECGCI").
    if (enc_name == "H" || enc_name == "V") && (ordering == "Japan1" || ordering.is_empty()) {
        let hi = (char_code >> 8) & 0xFF;
        let lo = char_code & 0xFF;
        if (0x21..=0x7E).contains(&hi) && (0x21..=0x7E).contains(&lo) {
            let euc = [(hi | 0x80) as u8, (lo | 0x80) as u8];
            if let Some(decoded) = decode_legacy_cjk(encoding_rs::EUC_JP, &euc, allocation) {
                return Some(decoded);
            }
            allocation.check()?;
        }
        // ASCII range (single-byte-ish codes 0x20–0x7E) pass through as-is.
        if char_code <= 0x7E {
            if let Some(c) = char::from_u32(char_code) {
                return allocation.character(c);
            }
        }
    }

    // Determine which legacy encoding applies based on the CMap name and ordering.
    // CMap names that imply raw legacy encoding (not CID-keyed identity):
    let enc: Option<&'static encoding_rs::Encoding> = if enc_name.contains("GBK")
        || enc_name.contains("GB-")
        || enc_name.contains("GBpc")
        || (enc_name.contains("EUC") && (ordering == "GB1" || enc_name.starts_with("GB")))
    {
        Some(encoding_rs::GBK)
    } else if enc_name.contains("B5")
        || enc_name.contains("CNS")
        || (enc_name.contains("EUC") && ordering == "CNS1")
    {
        Some(encoding_rs::BIG5)
    } else if enc_name.contains("EUC") && ordering == "Japan1" {
        Some(encoding_rs::EUC_JP)
    } else if (enc_name.contains("KSC") || enc_name.contains("KSCms")) && ordering == "Korea1" {
        Some(encoding_rs::EUC_KR)
    } else {
        None
    };

    let enc = enc?;

    // Reconstruct the raw bytes from the 2-byte char_code (big-endian)
    let bytes: [u8; 2] = [((char_code >> 8) & 0xFF) as u8, (char_code & 0xFF) as u8];

    decode_legacy_cjk(enc, &bytes, allocation)
}
fn decode_legacy_cjk(
    encoding: &'static encoding_rs::Encoding,
    bytes: &[u8],
    allocation: FontAllocation<'_>,
) -> Option<String> {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let capacity = decoder.max_utf8_buffer_length_without_replacement(bytes.len())?;
    let mut output = allocation.string_capacity(capacity)?;
    let (status, consumed) = decoder.decode_to_string_without_replacement(bytes, &mut output, true);
    if !matches!(status, encoding_rs::DecoderResult::InputEmpty) || consumed != bytes.len() {
        return None;
    }
    output.retain(|c| c != '\u{FFFD}');
    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

// Maximum valid CID for each Adobe character collection (Fix C – OOB guard).
// CIDs beyond these values have no defined Unicode mapping; return None early
// to avoid accidental wrap-around in future table expansions.
//
// Sources:
//   Adobe-GB1-5 (TN #5079): 30,283 CIDs (0–30,283)
//   Adobe-Japan1-7 (TN #5078): 23,059 CIDs (0–23,059)
//   Adobe-CNS1-7 (TN #5080): 20,316 CIDs (0–20,316)
//   Adobe-Korea1-2 (TN #5093): 18,351 CIDs (0–18,351)
const CID_MAX_GB1: u16 = 30_283;
const CID_MAX_JAPAN1: u16 = 23_059;
const CID_MAX_CNS1: u16 = 20_316;
const CID_MAX_KOREA1: u16 = 18_351;

/// Lookup Unicode code point for a CID in a predefined Unicode-based CMap.
///
/// Predefined CMaps for CJK fonts map CID values from Adobe character collections to Unicode.
/// Per PDF Spec ISO 32000-1:2008 Section 9.7.5.2.
///
/// # Arguments
///
/// * `cmap_name` - The predefined CMap name (e.g., "UniGB-UCS2-H")
/// * `cid_system_info` - The CIDSystemInfo identifying the character collection
/// * `cid` - The Character ID (CID) to look up
///
/// # Returns
///
/// The corresponding Unicode code point, or None if not found.
///
/// # Predefined CMaps Supported
///
/// - UniGB-UCS2-H: Adobe-GB1 (Simplified Chinese)
/// - UniJIS-UCS2-H: Adobe-Japan1 (Japanese)
/// - UniCNS-UCS2-H: Adobe-CNS1 (Traditional Chinese)
/// - UniKS-UCS2-H: Adobe-Korea1 (Korean)
fn lookup_predefined_cmap(
    cmap_name: &str,
    cid_system_info: &Option<CIDSystemInfo>,
    cid: u16,
) -> Option<u32> {
    // Verify that we have CIDSystemInfo to match against the CMap
    let system_info = cid_system_info.as_ref()?;

    // Fix C: guard out-of-bounds CIDs before hitting the lookup table.
    // CIDs beyond the collection maximum have no defined Unicode mapping.
    let max_cid = match system_info.ordering.as_str() {
        "GB1" => CID_MAX_GB1,
        "Japan1" => CID_MAX_JAPAN1,
        "CNS1" => CID_MAX_CNS1,
        "Korea1" => CID_MAX_KOREA1,
        // Adobe-Arabic-1 / Adobe-Persian-1: `lookup_adobe_arabic` rejects
        // unmapped CIDs itself, so the bound is just an early-out.
        "Arabic" | "Persian" => u16::MAX,
        _ => return None,
    };
    if cid > max_cid {
        log::debug!(
            "CID {} exceeds max {} for ordering '{}' → returning None (OOB)",
            cid,
            max_cid,
            system_info.ordering
        );
        return None;
    }

    // Route to the appropriate CMap lookup based on name and character collection
    match (cmap_name, system_info.ordering.as_str()) {
        ("UniGB-UCS2-H", "GB1") => lookup_adobe_gb1_to_unicode(cid),
        ("UniJIS-UCS2-H", "Japan1") => lookup_adobe_japan1_to_unicode(cid),
        ("UniCNS-UCS2-H", "CNS1") => lookup_adobe_cns1_to_unicode(cid),
        ("UniKS-UCS2-H", "Korea1") => lookup_adobe_korea1_to_unicode(cid),
        // Fallback: match by CIDSystemInfo ordering alone.
        // Some PDFs use encoding CMaps with custom names (e.g., "Adobe-Japan1-2")
        // that are identity mappings (charcode == CID). The CID→Unicode lookup
        // should still work based on the character collection ordering.
        (_, "GB1") => lookup_adobe_gb1_to_unicode(cid),
        (_, "Japan1") => lookup_adobe_japan1_to_unicode(cid),
        (_, "CNS1") => lookup_adobe_cns1_to_unicode(cid),
        (_, "Korea1") => lookup_adobe_korea1_to_unicode(cid),
        // Adobe-Arabic-1 / Adobe-Persian-1 CIDFonts without /ToUnicode (Nazanin,
        // Yagut, Mitra, Lotus). `lookup_adobe_arabic` is the §9.10.3 step-3
        // identity fallback; without it these decode as Latin-Extended-B garbage.
        (_, "Arabic") | (_, "Persian") => crate::fonts::cid_mappings::lookup_adobe_arabic(cid),
        _ => None,
    }
}

/// Map CID from Adobe-GB1 character collection to Unicode.
///
/// Adobe-GB1 contains Simplified Chinese characters from GB 2312 and extensions.
/// Reference: Adobe Technical Note #5079 (Adobe-GB1-4 Character Collection)
fn lookup_adobe_gb1_to_unicode(cid: u16) -> Option<u32> {
    crate::fonts::cid_mappings::lookup_adobe_gb1(cid)
}

/// Map CID from Adobe-Japan1 character collection to Unicode.
///
/// Adobe-Japan1 contains Japanese characters from JIS X 0208, JIS X 0212, etc.
/// Reference: Adobe Technical Note #5078 (Adobe-Japan1-4 Character Collection)
fn lookup_adobe_japan1_to_unicode(cid: u16) -> Option<u32> {
    crate::fonts::cid_mappings::lookup_adobe_japan1(cid)
}

/// Map CID from Adobe-CNS1 character collection to Unicode.
///
/// Adobe-CNS1 contains Traditional Chinese characters from CNS 11643 and extensions.
/// Reference: Adobe Technical Note #5080 (Adobe-CNS1-4 Character Collection)
fn lookup_adobe_cns1_to_unicode(cid: u16) -> Option<u32> {
    crate::fonts::cid_mappings::lookup_adobe_cns1(cid)
}

/// Map CID from Adobe-Korea1 character collection to Unicode.
///
/// Adobe-Korea1 contains Korean characters from KS X 1001 and KS X 1002.
/// Reference: Adobe Technical Note #5093 (Adobe-Korea1-2 Character Collection)
fn lookup_adobe_korea1_to_unicode(cid: u16) -> Option<u32> {
    crate::fonts::cid_mappings::lookup_adobe_korea1(cid)
}

/// Ascent/descent (as fractions of em) for the 14 standard PDF fonts.
/// Values from Adobe AFM files; used when no FontDescriptor is present.
fn standard_font_metrics(base_font: &str) -> Option<(f32, f32)> {
    // Strip subset prefix (e.g. "ABCDEF+Courier" -> "Courier")
    let name = if let Some(pos) = base_font.find('+') {
        &base_font[pos + 1..]
    } else {
        base_font
    };
    match name {
        "Courier" | "Courier-Bold" | "Courier-Oblique" | "Courier-BoldOblique" => {
            Some((0.629, -0.157))
        },
        "Helvetica" | "Helvetica-Bold" | "Helvetica-Oblique" | "Helvetica-BoldOblique" => {
            Some((0.718, -0.207))
        },
        "Times-Roman" => Some((0.683, -0.217)),
        "Times-Bold" => Some((0.676, -0.205)),
        "Times-Italic" => Some((0.683, -0.205)),
        "Times-BoldItalic" => Some((0.683, -0.205)),
        "Symbol" => Some((1.010, -0.293)),
        "ZapfDingbats" => Some((0.820, -0.143)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cipher_discriminator_flags_disagreeing_builtin_encoding() {
        // A subset cipher: ASCII-letter codes resolve to unrelated glyphs, so
        // they disagree with WinAnsi on every overlapping code (0/N agree).
        let cipher: HashMap<u8, char> = [(b'A', 'ñ'), (b'B', 'k'), (b'C', 'º'), (b'D', 'p')]
            .into_iter()
            .collect();
        assert!(builtin_encoding_looks_like_cipher(&cipher, "WinAnsiEncoding"));
    }

    #[test]
    fn cipher_discriminator_keeps_mostly_agreeing_builtin_encoding() {
        // A real text encoding: agrees with the named base on most codes (a
        // single non-standard slot is not enough to look like a cipher).
        let real: HashMap<u8, char> = [(b'A', 'A'), (b'B', 'B'), (b'C', 'C'), (0xCA, ' ')]
            .into_iter()
            .collect();
        assert!(!builtin_encoding_looks_like_cipher(&real, "WinAnsiEncoding"));
    }

    #[test]
    fn cipher_discriminator_no_overlap_is_not_a_cipher() {
        // No codes overlap the named base's mapped range → no evidence → not a
        // cipher (preserve the prior overlay behaviour).
        let empty: HashMap<u8, char> = HashMap::new();
        assert!(!builtin_encoding_looks_like_cipher(&empty, "WinAnsiEncoding"));
    }

    #[test]
    fn test_standard_encoding_ascii() {
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", b'A'), Some("A".to_string()));
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", b'Z'), Some("Z".to_string()));
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", b'0'), Some("0".to_string()));
    }

    #[test]
    fn test_standard_encoding_space() {
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", b' '), Some(" ".to_string()));
    }

    #[test]
    fn test_font_info_is_bold() {
        let font = FontInfo {
            base_font: "Times-Bold".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: Some(700),
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert!(font.is_bold());

        let font2 = FontInfo {
            base_font: "Helvetica".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: Some(400),
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert!(!font2.is_bold());
    }

    #[test]
    fn test_font_info_is_italic() {
        let font = FontInfo {
            base_font: "Times-Italic".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert!(font.is_italic());

        let font2 = FontInfo {
            base_font: "Courier-Oblique".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert!(font2.is_italic());
    }

    #[test]
    fn test_char_to_unicode_with_tounicode() {
        // Create a simple CMap with one custom mapping
        let cmap_data = b"beginbfchar\n<0041> <0058>\nendbfchar"; // Map 0x41 to 'X'

        let font = FontInfo {
            base_font: "CustomFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: Some(LazyCMap::new(cmap_data.to_vec())),
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Should use ToUnicode mapping (priority)
        assert_eq!(font.char_to_unicode(0x41), Some("X".to_string()));
        // Should fall back to standard encoding
        assert_eq!(font.char_to_unicode(0x42), Some("B".to_string()));
    }

    #[test]
    fn test_char_to_unicode_standard_encoding() {
        let font = FontInfo {
            base_font: "Times-Roman".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
        assert_eq!(font.char_to_unicode(0x20), Some(" ".to_string()));
    }

    #[test]
    fn test_char_to_unicode_identity() {
        // Test Type0 font WITHOUT ToUnicode - should return U+FFFD per PDF Spec 9.10.2
        let font_type0 = FontInfo {
            base_font: "CIDFont".to_string(),
            subtype: "Type0".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Identity,
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Type0 without ToUnicode should use CID-as-Unicode fallback
        assert_eq!(font_type0.char_to_unicode(0x41), Some("A".to_string()));
        assert_eq!(font_type0.char_to_unicode(0x263A), Some("\u{263A}".to_string()));

        // Test Type1 font WITH Identity encoding - should work correctly
        let font_type1 = FontInfo {
            base_font: "TimesRoman".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Identity,
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Simple fonts (Type1) CAN use Identity encoding for valid Unicode codes
        assert_eq!(font_type1.char_to_unicode(0x41), Some("A".to_string()));
        assert_eq!(font_type1.char_to_unicode(0x263A), Some("☺".to_string()));
    }

    #[test]
    fn test_lookup_predefined_cmap_adobe_gb1() {
        // Test Adobe-GB1 (Simplified Chinese) CMap lookup
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "GB1".to_string(),
            supplement: 2,
        });

        // Test ASCII from CID (CID 34 -> 'A')
        assert_eq!(lookup_predefined_cmap("UniGB-UCS2-H", &cid_system_info, 34), Some(0x41));

        // Test known CJK mapping (CID 4559 -> U+4E2D "中")
        assert_eq!(lookup_predefined_cmap("UniGB-UCS2-H", &cid_system_info, 4559), Some(0x4E2D));

        // Test unknown CID
        assert_eq!(lookup_predefined_cmap("UniGB-UCS2-H", &cid_system_info, 50000), None);

        // Test without CIDSystemInfo (should return None)
        assert_eq!(lookup_predefined_cmap("UniGB-UCS2-H", &None, 34), None);
    }

    #[test]
    fn test_lookup_predefined_cmap_adobe_japan1() {
        // Test Adobe-Japan1 (Japanese) CMap lookup
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Japan1".to_string(),
            supplement: 4,
        });

        // Test ASCII from CID (CID 34 -> 'A')
        assert_eq!(lookup_predefined_cmap("UniJIS-UCS2-H", &cid_system_info, 34), Some(0x41));

        // Test Hiragana from CID (CID 843 -> あ U+3042)
        assert_eq!(lookup_predefined_cmap("UniJIS-UCS2-H", &cid_system_info, 843), Some(0x3042));

        // Test unknown CID
        assert_eq!(lookup_predefined_cmap("UniJIS-UCS2-H", &cid_system_info, 50000), None);
    }

    #[test]
    fn test_lookup_predefined_cmap_adobe_cns1() {
        // Test Adobe-CNS1 (Traditional Chinese) CMap lookup
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "CNS1".to_string(),
            supplement: 3,
        });

        // Test ASCII from CID (CID 34 -> 'A')
        assert_eq!(lookup_predefined_cmap("UniCNS-UCS2-H", &cid_system_info, 34), Some(0x41));

        // Test CJK from CID (CID 595 -> 一 U+4E00)
        assert_eq!(lookup_predefined_cmap("UniCNS-UCS2-H", &cid_system_info, 595), Some(0x4E00));
    }

    #[test]
    fn test_lookup_predefined_cmap_adobe_korea1() {
        // Test Adobe-Korea1 (Korean) CMap lookup
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Korea1".to_string(),
            supplement: 1,
        });

        // Test ASCII from CID (CID 34 -> 'A')
        assert_eq!(lookup_predefined_cmap("UniKS-UCS2-H", &cid_system_info, 34), Some(0x41));

        // Test Hangul from CID (CID 1086 -> 가 U+AC00)
        assert_eq!(lookup_predefined_cmap("UniKS-UCS2-H", &cid_system_info, 1086), Some(0xAC00));
    }

    #[test]
    fn test_lookup_predefined_cmap_wrong_ordering() {
        // Test that lookup fails if CIDSystemInfo ordering doesn't match
        let cid_system_info_wrong = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "WrongOrdering".to_string(),
            supplement: 1,
        });

        // Should return None because ordering doesn't match
        assert_eq!(lookup_predefined_cmap("UniGB-UCS2-H", &cid_system_info_wrong, 34), None);
    }

    #[test]
    fn test_encoding_clone() {
        let enc = Encoding::Standard("WinAnsiEncoding".to_string());
        let enc2 = enc.clone();
        match enc2 {
            Encoding::Standard(name) => assert_eq!(name, "WinAnsiEncoding"),
            _ => panic!("Wrong encoding type"),
        }
    }

    #[test]
    fn test_font_info_clone() {
        let font = FontInfo {
            base_font: "Test".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        let font2 = font.clone();
        assert_eq!(font2.base_font, "Test");
    }

    #[test]
    fn test_glyph_name_to_unicode_basic() {
        assert_eq!(glyph_name_to_unicode("A"), Some('A'));
        assert_eq!(glyph_name_to_unicode("a"), Some('a'));
        assert_eq!(glyph_name_to_unicode("zero"), Some('0'));
        assert_eq!(glyph_name_to_unicode("nine"), Some('9'));
    }

    #[test]
    fn test_glyph_name_to_unicode_punctuation() {
        assert_eq!(glyph_name_to_unicode("space"), Some(' '));
        assert_eq!(glyph_name_to_unicode("quotesingle"), Some('\''));
        assert_eq!(glyph_name_to_unicode("grave"), Some('`'));
        assert_eq!(glyph_name_to_unicode("hyphen"), Some('-'));
        // Official AGL: "minus" maps to U+2212 (MINUS SIGN), not U+002D (HYPHEN-MINUS)
        assert_eq!(glyph_name_to_unicode("minus"), Some('−'));
    }

    #[test]
    fn test_glyph_name_to_unicode_special() {
        assert_eq!(glyph_name_to_unicode("bullet"), Some('•'));
        assert_eq!(glyph_name_to_unicode("dagger"), Some('†'));
        assert_eq!(glyph_name_to_unicode("daggerdbl"), Some('‡'));
        assert_eq!(glyph_name_to_unicode("ellipsis"), Some('…'));
        assert_eq!(glyph_name_to_unicode("emdash"), Some('—'));
        assert_eq!(glyph_name_to_unicode("endash"), Some('–'));
    }

    #[test]
    fn test_glyph_name_to_unicode_quotes() {
        assert_eq!(glyph_name_to_unicode("quotesinglbase"), Some('‚'));
        assert_eq!(glyph_name_to_unicode("quotedblbase"), Some('„'));
        // Official AGL uses proper curly quotes, not straight quotes
        assert_eq!(glyph_name_to_unicode("quotedblleft"), Some('\u{201C}')); // LEFT DOUBLE QUOTATION MARK
        assert_eq!(glyph_name_to_unicode("quotedblright"), Some('\u{201D}')); // RIGHT DOUBLE QUOTATION MARK
        assert_eq!(glyph_name_to_unicode("quoteleft"), Some('\u{2018}'));
        assert_eq!(glyph_name_to_unicode("quoteright"), Some('\u{2019}'));
    }

    #[test]
    fn test_glyph_name_to_unicode_accented() {
        assert_eq!(glyph_name_to_unicode("Aacute"), Some('Á'));
        assert_eq!(glyph_name_to_unicode("aacute"), Some('á'));
        assert_eq!(glyph_name_to_unicode("Ntilde"), Some('Ñ'));
        assert_eq!(glyph_name_to_unicode("ntilde"), Some('ñ'));
    }

    #[test]
    fn test_glyph_name_to_unicode_currency() {
        assert_eq!(glyph_name_to_unicode("Euro"), Some('€'));
        assert_eq!(glyph_name_to_unicode("sterling"), Some('£'));
        assert_eq!(glyph_name_to_unicode("yen"), Some('¥'));
        assert_eq!(glyph_name_to_unicode("cent"), Some('¢'));
    }

    #[test]
    fn test_glyph_name_to_unicode_ligatures() {
        assert_eq!(glyph_name_to_unicode("fi"), Some('ﬁ'));
        assert_eq!(glyph_name_to_unicode("fl"), Some('ﬂ'));
        assert_eq!(glyph_name_to_unicode("ffi"), Some('ﬃ'));
    }

    #[test]
    fn test_glyph_name_to_unicode_uni_xxxx() {
        // Test uni format (4 hex digits)
        assert_eq!(glyph_name_to_unicode("uni0041"), Some('A'));
        assert_eq!(glyph_name_to_unicode("uni2022"), Some('•'));
    }

    #[test]
    fn test_glyph_name_to_unicode_u_xxxx() {
        // Test u format (variable hex digits)
        assert_eq!(glyph_name_to_unicode("u0041"), Some('A'));
        assert_eq!(glyph_name_to_unicode("u2022"), Some('•'));
    }

    #[test]
    fn test_glyph_name_to_unicode_unknown() {
        assert_eq!(glyph_name_to_unicode("unknownglyph"), None);
        assert_eq!(glyph_name_to_unicode(""), None);
    }

    #[test]
    fn test_char_to_unicode_custom_encoding() {
        // Create a custom encoding map
        let mut custom_map = HashMap::new();
        custom_map.insert(0x41, 'X'); // A -> X
        custom_map.insert(0x42, '•'); // B -> bullet

        let font = FontInfo {
            base_font: "CustomFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Custom(custom_map),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Should use custom encoding
        assert_eq!(font.char_to_unicode(0x41), Some("X".to_string()));
        assert_eq!(font.char_to_unicode(0x42), Some("•".to_string()));
        // Unmapped character should return None
        assert_eq!(font.char_to_unicode(0x43), None);
    }

    /// Integration Test 1: ForceBold flag detection (PDF Spec Table 123, bit 19)
    #[test]
    fn test_get_font_weight_force_bold_flag() {
        // Test ForceBold flag set (bit 19 = 0x80000 = 524288)
        let font_with_force_bold = FontInfo {
            base_font: "Helvetica".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,    // No explicit weight
            flags: Some(0x80000), // ForceBold flag set
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_with_force_bold.get_font_weight(), FontWeight::Bold);
        assert!(font_with_force_bold.is_bold());

        // Test without ForceBold flag
        let font_without_force_bold = FontInfo {
            base_font: "Helvetica".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: Some(0x40000), // Different flag, NOT ForceBold
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_without_force_bold.get_font_weight(), FontWeight::Normal);
        assert!(!font_without_force_bold.is_bold());
    }

    /// Integration Test 2: StemV analysis for weight inference
    #[test]
    fn test_get_font_weight_stem_v_analysis() {
        // Test StemV > 110 → Bold
        let font_heavy_stem = FontInfo {
            base_font: "UnknownFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: Some(120.0), // Heavy stem
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_heavy_stem.get_font_weight(), FontWeight::Bold);
        assert!(font_heavy_stem.is_bold());

        // Test StemV 80-110 → Medium
        let font_medium_stem = FontInfo {
            base_font: "UnknownFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: Some(95.0), // Medium stem
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_medium_stem.get_font_weight(), FontWeight::Medium);
        assert!(!font_medium_stem.is_bold());

        // Test StemV < 80 → Normal
        let font_light_stem = FontInfo {
            base_font: "UnknownFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: Some(70.0), // Light stem
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_light_stem.get_font_weight(), FontWeight::Normal);
        assert!(!font_light_stem.is_bold());
    }

    /// Integration Test 3: Priority cascade (FontWeight > ForceBold > Name > StemV)
    #[test]
    fn test_get_font_weight_priority_cascade() {
        // Priority 1: Explicit FontWeight field overrides everything
        let font_explicit = FontInfo {
            base_font: "Helvetica-Bold".to_string(), // Name says Bold
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: Some(300), // But explicit weight is Light
            flags: Some(0x80000),   // ForceBold flag set
            stem_v: Some(120.0),    // Heavy stem
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_explicit.get_font_weight(), FontWeight::Light);
        assert!(!font_explicit.is_bold());

        // Priority 2: ForceBold overrides name and StemV
        let font_force_bold = FontInfo {
            base_font: "Helvetica".to_string(), // Name says Normal
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,    // No explicit weight
            flags: Some(0x80000), // ForceBold flag set
            stem_v: Some(70.0),   // Light stem
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_force_bold.get_font_weight(), FontWeight::Bold);
        assert!(font_force_bold.is_bold());

        // Priority 3: Name heuristics override StemV
        let font_name = FontInfo {
            base_font: "Helvetica-Bold".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: Some(70.0), // Light stem, but name says Bold
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        assert_eq!(font_name.get_font_weight(), FontWeight::Bold);
        assert!(font_name.is_bold());
    }

    /// Integration Test 4: Name heuristics for all weight categories
    #[test]
    fn test_get_font_weight_name_heuristics() {
        // Test Black/Heavy
        let font_black = FontInfo {
            base_font: "Helvetica-Black".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_black.get_font_weight(), FontWeight::Black);
        assert!(font_black.is_bold());

        // Test ExtraBold
        let font_extrabold = FontInfo {
            base_font: "Arial-ExtraBold".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_extrabold.get_font_weight(), FontWeight::ExtraBold);
        assert!(font_extrabold.is_bold());

        // Test Bold (but not SemiBold)
        let font_bold = FontInfo {
            base_font: "TimesNewRoman-Bold".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_bold.get_font_weight(), FontWeight::Bold);
        assert!(font_bold.is_bold());

        // Test SemiBold
        let font_semibold = FontInfo {
            base_font: "Arial-SemiBold".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_semibold.get_font_weight(), FontWeight::SemiBold);
        assert!(font_semibold.is_bold());

        // Test Medium
        let font_medium = FontInfo {
            base_font: "Roboto-Medium".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_medium.get_font_weight(), FontWeight::Medium);
        assert!(!font_medium.is_bold());

        // Test Light (but not ExtraLight)
        let font_light = FontInfo {
            base_font: "Helvetica-Light".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_light.get_font_weight(), FontWeight::Light);
        assert!(!font_light.is_bold());

        // Test ExtraLight
        let font_extralight = FontInfo {
            base_font: "Roboto-ExtraLight".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_extralight.get_font_weight(), FontWeight::ExtraLight);
        assert!(!font_extralight.is_bold());

        // Test Thin
        let font_thin = FontInfo {
            base_font: "HelveticaNeue-Thin".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_thin.get_font_weight(), FontWeight::Thin);
        assert!(!font_thin.is_bold());

        // Test Normal/Regular (no weight keywords)
        let font_normal = FontInfo {
            base_font: "Helvetica".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        assert_eq!(font_normal.get_font_weight(), FontWeight::Normal);
        assert!(!font_normal.is_bold());
    }

    /// Test CIDToGIDMap Identity mapping
    /// Per PDF Spec ISO 32000-1:2008, Section 9.7.4.2
    #[test]
    fn test_cid_to_gid_identity() {
        let identity_map = CIDToGIDMap::Identity;

        // In identity mapping, CID == GID
        assert_eq!(identity_map.get_gid(0), 0);
        assert_eq!(identity_map.get_gid(100), 100);
        assert_eq!(identity_map.get_gid(0xFFFF), 0xFFFF);
    }

    /// Test CIDToGIDMap Explicit mapping
    /// Verifies that explicit GID arrays are looked up correctly
    #[test]
    fn test_cid_to_gid_explicit() {
        // Create explicit mapping: CID 0→10, CID 1→20, CID 2→30
        let gid_array = vec![10, 20, 30];
        let explicit_map = CIDToGIDMap::Explicit(gid_array);

        assert_eq!(explicit_map.get_gid(0), 10);
        assert_eq!(explicit_map.get_gid(1), 20);
        assert_eq!(explicit_map.get_gid(2), 30);

        // Out of range - falls back to identity
        assert_eq!(explicit_map.get_gid(3), 3);
        assert_eq!(explicit_map.get_gid(100), 100);
    }

    // ==================================================================================
    // Extended Latin AGL Fallback Tests
    // ==================================================================================
    // These tests verify that Type0 fonts with Identity CMap can recover unmapped
    // characters using the Adobe Glyph List fallback for extended Latin characters
    // (0x80-0xFF range).

    #[test]
    fn test_gid_to_glyph_name_ascii_range() {
        // Verify ASCII printable range (0x20-0x7E) is still working
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x20), Some("space"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x41), Some("A"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x61), Some("a"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x30), Some("zero"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x7E), Some("asciitilde"));
    }

    #[test]
    fn test_gid_to_glyph_name_windows1252_symbols() {
        // Test Windows-1252 extended symbols (0x80-0x9F range)
        // These are commonly found in Western European PDFs

        // Currency and special symbols
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x80), Some("euro"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x83), Some("florin"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x85), Some("ellipsis"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x8C), Some("OE"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x9C), Some("oe"));

        // Diacritical marks
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x8A), Some("Scaron"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x9A), Some("scaron"));

        // Smart quotes and dashes
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x91), Some("quoteleft"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x92), Some("quoteright"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x93), Some("quotedblleft"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x94), Some("quotedblright"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x96), Some("endash"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x97), Some("emdash"));
    }

    #[test]
    fn test_gid_to_glyph_name_latin1_supplement() {
        // Test Latin-1 Supplement range (0xA0-0xFF)
        // These cover Western European languages (French, Spanish, German, etc.)

        // Currency and symbols
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xA2), Some("cent"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xA3), Some("sterling"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xA4), Some("currency"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xA5), Some("yen"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xA9), Some("copyright"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xAE), Some("registered"));

        // Math symbols
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xB0), Some("degree"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xB1), Some("plusminus"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xD7), Some("multiply"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xF7), Some("divide"));
    }

    #[test]
    fn test_gid_to_glyph_name_uppercase_accented() {
        // Test uppercase Latin letters with diacritical marks
        // These are essential for French (accented A, E), Spanish (N with tilde), German (Umlaut)
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC0), Some("Agrave"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC1), Some("Aacute"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC2), Some("Acircumflex"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC3), Some("Atilde"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC4), Some("Adieresis"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC5), Some("Aring"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC6), Some("AE"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xC7), Some("Ccedilla"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xD1), Some("Ntilde"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xD6), Some("Odieresis"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xDC), Some("Udieresis"));
    }

    #[test]
    fn test_gid_to_glyph_name_lowercase_accented() {
        // Test lowercase Latin letters with diacritical marks
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE0), Some("agrave"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE1), Some("aacute"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE2), Some("acircumflex"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE3), Some("atilde"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE4), Some("adieresis"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE5), Some("aring"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE6), Some("ae"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xE7), Some("ccedilla"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xF1), Some("ntilde"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xF6), Some("odieresis"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xFC), Some("udieresis"));
    }

    #[test]
    fn test_gid_to_glyph_name_special_characters() {
        // Test ordinal indicators and special characters
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xAA), Some("ordfeminine"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xBA), Some("ordmasculine"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xB2), Some("twosuperior"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xB3), Some("threesuperior"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xB9), Some("onesuperior"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xBC), Some("onequarter"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xBD), Some("onehalf"));
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xBE), Some("threequarters"));
    }

    #[test]
    fn test_gid_to_glyph_name_undefined_codes() {
        // Test that undefined codes in Windows-1252 return None
        // (0x81, 0x8D, 0x8F, 0x90, 0x9D are undefined)
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x81), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x8D), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x8F), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x90), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x9D), None);
    }

    #[test]
    fn test_gid_to_glyph_name_out_of_range() {
        // Test that GIDs outside supported ranges return None
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x100), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x1000), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xFFFF), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x0000), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x0001), None);
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x001F), None);
    }

    #[test]
    fn test_agl_fallback_euro_sign() {
        // Test that CID 0x80 (Euro sign) maps through AGL correctly
        // This is a real-world case: Type0 fonts without ToUnicode often need Euro mapping
        let glyph_name =
            FontInfo::gid_to_standard_glyph_name(0x80).expect("0x80 should map to euro");
        assert_eq!(glyph_name, "euro");

        // Verify the glyph exists in AGL
        assert!(ADOBE_GLYPH_LIST.get(glyph_name).is_some());

        // Verify it maps to the correct Unicode
        if let Some(&unicode_char) = ADOBE_GLYPH_LIST.get(glyph_name) {
            assert_eq!(unicode_char as u32, 0x20AC); // Euro sign U+20AC
        }
    }

    #[test]
    fn test_agl_fallback_extended_latin_coverage() {
        // Test that all common extended Latin characters have AGL mappings
        // This ensures the implementation works end-to-end through AGL lookup
        let test_cases = vec![
            (0x80, "euro", 0x20AC),           // Euro sign
            (0x82, "quotesinglbase", 0x201A), // Single low quote
            (0x83, "florin", 0x0192),         // f with hook
            (0x84, "quotedblbase", 0x201E),   // Double low quote
            (0x85, "ellipsis", 0x2026),       // Ellipsis
            (0xA9, "copyright", 0x00A9),      // Copyright
            (0xAE, "registered", 0x00AE),     // Registered
            (0xB0, "degree", 0x00B0),         // Degree
            (0xC1, "Aacute", 0x00C1),         // A acute
            (0xE1, "aacute", 0x00E1),         // a acute
        ];

        for (gid, expected_glyph, expected_unicode) in test_cases {
            // Step 1: GID -> Glyph name
            let glyph_name = FontInfo::gid_to_standard_glyph_name(gid as u16)
                .unwrap_or_else(|| panic!("GID 0x{:02X} should map to a glyph name", gid));
            assert_eq!(glyph_name, expected_glyph);

            // Step 2: Glyph name -> Unicode (via AGL)
            if let Some(&unicode_char) = ADOBE_GLYPH_LIST.get(glyph_name) {
                assert_eq!(unicode_char as u32, expected_unicode);
            } else {
                panic!("Glyph '{}' should exist in Adobe Glyph List", glyph_name);
            }
        }
    }

    #[test]
    fn test_agl_fallback_priority_integration() {
        // Integration test: Verify AGL fallback would activate for unmapped Type0 CIDs
        // This simulates the Priority 5 fallback in char_to_unicode()
        //
        // Scenario:
        // 1. Type0 font with Identity-H CMap
        // 2. No ToUnicode CMap
        // 3. No TrueType cmap
        // 4. CID 0xC1 (Á - A with acute accent) - common in Spanish/French documents
        //
        // Expected: CID 0xC1 -> GID 0xC1 -> "Aacute" -> U+00C1

        let glyph_name =
            FontInfo::gid_to_standard_glyph_name(0xC1).expect("GID 0xC1 should map to Aacute");
        assert_eq!(glyph_name, "Aacute");

        // Verify AGL has the mapping
        assert!(ADOBE_GLYPH_LIST.get("Aacute").is_some());

        // Verify correct Unicode
        if let Some(&unicode_char) = ADOBE_GLYPH_LIST.get("Aacute") {
            let result = unicode_char.to_string();
            assert_eq!(unicode_char as u32, 0x00C1);
            assert!(!result.is_empty());
        }
    }

    // =============================================================================
    // Type 0 /W Array (CID Width) Tests - PDF Spec 9.7.4.3
    // =============================================================================

    #[test]
    fn test_get_glyph_width_uses_cid_widths() {
        // Test that get_glyph_width properly uses cid_widths for Type0 fonts
        let mut cid_widths = HashMap::new();
        cid_widths.insert(1u16, 500.0f32);
        cid_widths.insert(2u16, 600.0f32);
        cid_widths.insert(3u16, 700.0f32);

        let font = FontInfo {
            base_font: "CIDFont".to_string(),
            subtype: "Type0".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Identity,
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: Some(cid_widths),
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Widths from cid_widths
        assert_eq!(font.get_glyph_width(1), 500.0);
        assert_eq!(font.get_glyph_width(2), 600.0);
        assert_eq!(font.get_glyph_width(3), 700.0);

        // CID not in cid_widths should return cid_default_width
        assert_eq!(font.get_glyph_width(100), 1000.0);
    }

    #[test]
    fn test_get_glyph_width_cid_default_width() {
        // Test that cid_default_width is used when CID is not in cid_widths
        let mut cid_widths = HashMap::new();
        cid_widths.insert(1u16, 500.0f32);

        let font = FontInfo {
            base_font: "CIDFont".to_string(),
            subtype: "Type0".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Identity,
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 500.0, // Simple font default
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: Some(cid_widths),
            cid_default_width: 800.0, // CID default width
            has_explicit_dw: true,    // F15: /DW was explicitly set
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // CID 1 has explicit width
        assert_eq!(font.get_glyph_width(1), 500.0);

        // Other CIDs use cid_default_width (not default_width) when has_explicit_dw=true
        assert_eq!(font.get_glyph_width(2), 800.0);
        assert_eq!(font.get_glyph_width(999), 800.0);
    }

    #[test]
    fn test_get_glyph_width_no_cid_widths_uses_default() {
        // Test that fonts without cid_widths fall back to default_width
        let font = FontInfo {
            base_font: "SimpleFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 600.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // All CIDs use default_width when no cid_widths and no widths array
        assert_eq!(font.get_glyph_width(1), 600.0);
        assert_eq!(font.get_glyph_width(65), 600.0);
    }

    #[test]
    fn test_cid_widths_large_range() {
        // Test CID widths with a large range of values (simulating real CJK fonts)
        let mut cid_widths = HashMap::new();
        // Simulate /W array: [1 100 1000] - CIDs 1-100 all have width 1000
        for cid in 1u16..=100 {
            cid_widths.insert(cid, 1000.0f32);
        }
        // Add some individual widths
        cid_widths.insert(200, 500.0);
        cid_widths.insert(201, 600.0);

        let font = FontInfo {
            base_font: "CJKFont".to_string(),
            subtype: "Type0".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Identity,
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 500.0,
            cid_to_gid_map: None,
            cid_system_info: Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Japan1".to_string(),
                supplement: 4,
            }),
            cid_font_type: Some("CIDFontType2".to_string()),
            cid_widths: Some(cid_widths),
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Range test
        assert_eq!(font.get_glyph_width(1), 1000.0);
        assert_eq!(font.get_glyph_width(50), 1000.0);
        assert_eq!(font.get_glyph_width(100), 1000.0);

        // Individual widths
        assert_eq!(font.get_glyph_width(200), 500.0);
        assert_eq!(font.get_glyph_width(201), 600.0);

        // F15 fix: has_explicit_dw=false → fall back to default_width (500.0), not cid_default_width.
        // When /DW is not explicit in the PDF, we cannot trust cid_default_width as authoritative.
        assert_eq!(font.get_glyph_width(300), 500.0);
    }

    // =========================================================================
    // Helper: create a minimal FontInfo for testing (reduces boilerplate)
    // =========================================================================
    fn make_font(overrides: impl FnOnce(&mut FontInfo)) -> FontInfo {
        let mut f = FontInfo {
            base_font: "TestFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 500.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };
        overrides(&mut f);
        f
    }

    // =========================================================================
    // best_mapping_provenance — font-level Unicode-mapping capability (fact)
    // =========================================================================

    // The critical case: a Type0 / Identity-ordered subset with no ToUnicode
    // and no embedded cmap severs every path to Unicode → Fallback.
    #[test]
    fn best_mapping_provenance_fallback_on_severed_identity_type0() {
        let f = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Identity-H".to_string());
            f.cid_system_info = Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Identity".to_string(),
                supplement: 0,
            });
            f.to_unicode = None;
        });
        assert_eq!(f.best_mapping_provenance(), crate::fonts::MappingProvenance::Fallback);
    }

    #[test]
    fn best_mapping_provenance_fallback_type0_without_cidsysteminfo() {
        let f = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Identity-H".to_string());
            f.cid_system_info = None;
            f.to_unicode = None;
        });
        assert_eq!(f.best_mapping_provenance(), crate::fonts::MappingProvenance::Fallback);
    }

    // A known character collection (non-Identity ordering) → predefined CMap.
    #[test]
    fn best_mapping_provenance_predefined_for_known_collection() {
        let f = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.cid_system_info = Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Japan1".to_string(),
                supplement: 6,
            });
        });
        assert_eq!(f.best_mapping_provenance(), crate::fonts::MappingProvenance::PredefinedCMap);
    }

    // A simple font resolves through its /Encoding → glyph name → AGL.
    #[test]
    fn best_mapping_provenance_encoding_for_simple_font() {
        let f = make_font(|_| {});
        assert_eq!(f.best_mapping_provenance(), crate::fonts::MappingProvenance::EncodingName);
    }

    // =========================================================================
    // get_space_glyph_width — the space advance drives the geometric word-gap
    // threshold, so it must be a REAL space advance, never an arbitrary glyph.
    // Regression guard for #803 (justified TJ words glued together).
    // =========================================================================

    #[test]
    fn space_width_identity_type0_ignores_cid32_glyph() {
        // #803: under Identity-H, character code 0x20 maps to CID 32 — an
        // arbitrary font glyph (real repro: TimesNewRomanPSMT reports 563 units
        // ≈ 0.56 em), NOT the space. Trusting it as the space advance inflated
        // the word-gap threshold (0.75 × 0.56 em) so far that real ~0.25 em
        // justified word gaps were suppressed and adjacent words glued together
        // ("All rights reserved" -> "Allrightsreserved"). The reference must
        // fall back to the 0.25 em (250-unit) typographic default instead.
        let mut cid_widths = HashMap::new();
        cid_widths.insert(0x20_u16, 563.0);
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity;
            f.cid_widths = Some(cid_widths);
        });
        assert_eq!(
            font.get_space_glyph_width(),
            250.0,
            "Identity Type0 must not treat the CID-32 glyph width as the space advance"
        );
    }

    #[test]
    fn space_width_identity_type0_without_cid32_defaults() {
        // Identity Type0 with no /W entry for CID 32 also defaults to 0.25 em.
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity;
            f.cid_widths = Some(HashMap::new());
        });
        assert_eq!(font.get_space_glyph_width(), 250.0);
    }

    #[test]
    fn space_width_non_identity_type0_trusts_explicit_space_cid() {
        // A non-Identity predefined CMap can genuinely place the space at code
        // 0x20, so an explicit /W entry there is a real space advance and is
        // kept — only Identity encoding remaps 0x20 to an arbitrary CID.
        let mut cid_widths = HashMap::new();
        cid_widths.insert(0x20_u16, 280.0);
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Predefined-CMap".to_string());
            f.cid_widths = Some(cid_widths);
        });
        assert_eq!(font.get_space_glyph_width(), 280.0);
    }

    #[test]
    fn space_width_simple_font_uses_explicit_widths_space() {
        // A simple font that declares 0x20 in /Widths (FirstChar covers code 32)
        // uses that real space advance unchanged.
        let font = make_font(|f| {
            f.subtype = "Type1".to_string();
            f.first_char = Some(32);
            // index 0 = code 32 (space) = 260 units.
            f.widths = Some(vec![260.0, 500.0, 500.0]);
        });
        assert_eq!(font.get_space_glyph_width(), 260.0);
    }

    // =========================================================================
    // parse_cid_widths — unit tests for the /W array parser
    // =========================================================================

    #[test]
    fn test_parse_cid_widths_array_format() {
        // Format: c [w1 w2 ... wn]
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                Object::Integer(10), // start CID
                Object::Array(vec![
                    Object::Integer(500),
                    Object::Integer(600),
                    Object::Integer(700),
                ]),
            ]),
        );
        let widths = FontInfo::parse_cid_widths(&dict, "Test").unwrap();
        assert_eq!(widths.get(&10), Some(&500.0));
        assert_eq!(widths.get(&11), Some(&600.0));
        assert_eq!(widths.get(&12), Some(&700.0));
        assert_eq!(widths.get(&13), None);
    }

    #[test]
    fn test_parse_cid_widths_range_format() {
        // Format: cfirst clast w
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                Object::Integer(100),
                Object::Integer(105),
                Object::Integer(300),
            ]),
        );
        let widths = FontInfo::parse_cid_widths(&dict, "Test").unwrap();
        for cid in 100..=105 {
            assert_eq!(widths.get(&cid), Some(&300.0), "CID {} should be 300", cid);
        }
        assert_eq!(widths.get(&106), None);
    }

    #[test]
    fn test_parse_cid_widths_mixed_formats() {
        // Mix array-format and range-format in one /W array
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                // Array format
                Object::Integer(1),
                Object::Array(vec![Object::Integer(200), Object::Integer(300)]),
                // Range format
                Object::Integer(50),
                Object::Integer(52),
                Object::Integer(400),
            ]),
        );
        let widths = FontInfo::parse_cid_widths(&dict, "Test").unwrap();
        assert_eq!(widths.get(&1), Some(&200.0));
        assert_eq!(widths.get(&2), Some(&300.0));
        assert_eq!(widths.get(&50), Some(&400.0));
        assert_eq!(widths.get(&51), Some(&400.0));
        assert_eq!(widths.get(&52), Some(&400.0));
    }

    #[test]
    fn test_parse_cid_widths_real_values() {
        // Widths specified as Real (float) values
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![Object::Integer(5), Object::Array(vec![Object::Real(123.5)])]),
        );
        let widths = FontInfo::parse_cid_widths(&dict, "Test").unwrap();
        assert_eq!(widths.get(&5), Some(&123.5));
    }

    #[test]
    fn test_parse_cid_widths_empty_array() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert("W".to_string(), Object::Array(vec![]));
        assert!(FontInfo::parse_cid_widths(&dict, "Test").is_none());
    }

    #[test]
    fn test_parse_cid_widths_missing_w() {
        let dict: HashMap<String, Object> = HashMap::new();
        assert!(FontInfo::parse_cid_widths(&dict, "Test").is_none());
    }

    #[test]
    fn test_parse_cid_widths_non_integer_start() {
        // First element is not an integer — should skip
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                Object::Name("bad".to_string()),
                Object::Integer(10),
                Object::Array(vec![Object::Integer(500)]),
            ]),
        );
        let widths = FontInfo::parse_cid_widths(&dict, "Test").unwrap();
        assert_eq!(widths.get(&10), Some(&500.0));
    }

    #[test]
    fn test_parse_cid_widths_truncated_range() {
        // Range format with missing width — should just stop
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                Object::Integer(10),
                Object::Integer(15),
                // missing width
            ]),
        );
        assert!(FontInfo::parse_cid_widths(&dict, "Test").is_none());
    }

    #[test]
    fn test_parse_cid_widths_unexpected_second_element() {
        // Second element after CID is neither Array nor Integer
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![Object::Integer(10), Object::Name("bad".to_string())]),
        );
        // Should produce empty widths
        assert!(FontInfo::parse_cid_widths(&dict, "Test").is_none());
    }

    #[test]
    fn test_parse_cid_widths_range_with_bad_width() {
        // Range format where the width value is not a number
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                Object::Integer(1),
                Object::Integer(3),
                Object::Name("notanumber".to_string()),
            ]),
        );
        // Bad width for range, should skip and produce no widths
        assert!(FontInfo::parse_cid_widths(&dict, "Test").is_none());
    }

    #[test]
    fn test_parse_cid_widths_range_with_real_width() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W".to_string(),
            Object::Array(vec![
                Object::Integer(10),
                Object::Integer(12),
                Object::Real(750.5),
            ]),
        );
        let widths = FontInfo::parse_cid_widths(&dict, "Test").unwrap();
        assert_eq!(widths.get(&10), Some(&750.5));
        assert_eq!(widths.get(&11), Some(&750.5));
        assert_eq!(widths.get(&12), Some(&750.5));
    }

    // =========================================================================
    // parse_cid_vertical_metrics + parse_dw2 — /W2 and /DW2 (vertical writing)
    // =========================================================================

    /// `/W2` Form A: `c [ w1y v_x v_y w1y v_x v_y … ]` assigns successive
    /// triples to CIDs `c`, `c+1`, `c+2`, … Drives per-CID lookups for
    /// vertical advance and vertical-origin offset on tategaki layouts.
    #[test]
    fn test_parse_w2_explicit_array_form() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                Object::Integer(10),
                Object::Array(vec![
                    // CID 10: w1y=-880 v_x=500 v_y=900
                    Object::Integer(-880),
                    Object::Integer(500),
                    Object::Integer(900),
                    // CID 11: w1y=-1000 v_x=520 v_y=850
                    Object::Integer(-1000),
                    Object::Integer(520),
                    Object::Integer(850),
                ]),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        assert_eq!(
            metrics.get(&10),
            Some(&VerticalMetrics {
                w1y: -880.0,
                v_x: 500.0,
                v_y: 900.0
            })
        );
        assert_eq!(
            metrics.get(&11),
            Some(&VerticalMetrics {
                w1y: -1000.0,
                v_x: 520.0,
                v_y: 850.0
            })
        );
        assert_eq!(metrics.get(&12), None);
    }

    /// `/W2` Form B: `c_first c_last w1y v_x v_y` assigns the same metrics
    /// to every CID in the inclusive range.
    #[test]
    fn test_parse_w2_range_form() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                Object::Integer(100),
                Object::Integer(102),
                Object::Integer(-1000),
                Object::Integer(500),
                Object::Integer(880),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        let expected = VerticalMetrics {
            w1y: -1000.0,
            v_x: 500.0,
            v_y: 880.0,
        };
        assert_eq!(metrics.get(&100), Some(&expected));
        assert_eq!(metrics.get(&101), Some(&expected));
        assert_eq!(metrics.get(&102), Some(&expected));
        assert_eq!(metrics.get(&103), None);
        assert_eq!(metrics.get(&99), None);
    }

    /// `/W2` Form A and Form B can be intermixed in a single array. Real
    /// CIDFonts use this routinely — explicit triples for outliers and
    /// ranges for runs of full-width CJK glyphs.
    #[test]
    fn test_parse_w2_mixed_forms() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                // Form A: CID 5 explicit triple
                Object::Integer(5),
                Object::Array(vec![
                    Object::Integer(-900),
                    Object::Integer(490),
                    Object::Integer(870),
                ]),
                // Form B: CIDs 200..=201
                Object::Integer(200),
                Object::Integer(201),
                Object::Integer(-1000),
                Object::Integer(500),
                Object::Integer(880),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        assert_eq!(
            metrics.get(&5),
            Some(&VerticalMetrics {
                w1y: -900.0,
                v_x: 490.0,
                v_y: 870.0
            })
        );
        let range_default = VerticalMetrics {
            w1y: -1000.0,
            v_x: 500.0,
            v_y: 880.0,
        };
        assert_eq!(metrics.get(&200), Some(&range_default));
        assert_eq!(metrics.get(&201), Some(&range_default));
        assert_eq!(metrics.get(&202), None);
    }

    /// Missing `/W2` ⇒ `None`. Horizontal-only fonts must skip the HashMap
    /// allocation so they pay no per-glyph lookup cost in the hot path.
    #[test]
    fn test_parse_w2_missing_returns_none() {
        let dict: HashMap<String, Object> = HashMap::new();
        assert!(FontInfo::parse_cid_vertical_metrics(&dict, "Test").is_none());
    }

    /// Empty `/W2` array ⇒ `None`.
    #[test]
    fn test_parse_w2_empty_returns_none() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert("W2".to_string(), Object::Array(vec![]));
        assert!(FontInfo::parse_cid_vertical_metrics(&dict, "Test").is_none());
    }

    /// `/W2` accepts real-valued metrics (some writers use floats for
    /// fine-tuned vertical adjustments).
    #[test]
    fn test_parse_w2_real_values() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                Object::Integer(1),
                Object::Integer(1),
                Object::Real(-987.5),
                Object::Real(501.25),
                Object::Real(879.75),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        assert_eq!(
            metrics.get(&1),
            Some(&VerticalMetrics {
                w1y: -987.5,
                v_x: 501.25,
                v_y: 879.75
            })
        );
    }

    /// `/W2` Form A with a malformed inner triple must not desynchronise
    /// the CID assignment of subsequent triples. The original
    /// implementation advanced `j` by 1 on a non-numeric element without
    /// touching `emitted`, so every following triple was shifted up by
    /// one CID. Spec stance: a triple is atomic — drop the whole triple
    /// (advance `j` by 3 and `emitted` by 1) so the CID alignment of the
    /// rest of the inner array is preserved.
    #[test]
    fn test_parse_w2_form_a_skips_malformed_triple_without_desync() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        // CID 10 is intentionally malformed (a name where w1y should be).
        // CID 11 must remain aligned to its proper triple, not slide into
        // CID 10's slot.
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                Object::Integer(10),
                Object::Array(vec![
                    // CID 10: malformed (name instead of number).
                    Object::Name("Bogus".to_string()),
                    Object::Integer(500),
                    Object::Integer(880),
                    // CID 11: well-formed (-1000, 500, 880).
                    Object::Integer(-1000),
                    Object::Integer(500),
                    Object::Integer(880),
                ]),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        // CID 10 was malformed: must NOT carry the metrics that belong to CID 11.
        assert!(
            !metrics.contains_key(&10),
            "malformed CID 10 must not appear in metrics; got {:?}",
            metrics.get(&10)
        );
        // CID 11 must carry its own metrics — not collapsed onto CID 10 or
        // shifted into a different CID slot.
        assert_eq!(
            metrics.get(&11),
            Some(&VerticalMetrics {
                w1y: -1000.0,
                v_x: 500.0,
                v_y: 880.0
            })
        );
    }

    /// `/W2` Form B near the top of the u16 range must not silently
    /// collapse every overflowing CID onto u16::MAX via saturating
    /// arithmetic. The loop must break (with a warning log) when the
    /// requested range would wrap past 0xFFFF.
    #[test]
    fn test_parse_w2_form_b_overflow_does_not_collapse() {
        // c_first = 0xFFFB, c_last = 0xFFFF — fits exactly within u16 so
        // every CID in 65531..=65535 must be inserted distinctly. A
        // saturating-add bug would collapse them all onto u16::MAX (and
        // an unchecked-add bug would wrap around to 0).
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                Object::Integer(0xFFFB),
                Object::Integer(0xFFFF),
                Object::Integer(-1000),
                Object::Integer(500),
                Object::Integer(880),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        let expected = VerticalMetrics {
            w1y: -1000.0,
            v_x: 500.0,
            v_y: 880.0,
        };
        for cid in 0xFFFBu16..=0xFFFFu16 {
            assert_eq!(
                metrics.get(&cid),
                Some(&expected),
                "CID 0x{:04X} must carry the range metrics",
                cid
            );
        }
        // Exactly five distinct CIDs were inserted; nothing else.
        assert_eq!(
            metrics.len(),
            5,
            "Form B near u16::MAX should insert 5 distinct entries; got {}",
            metrics.len()
        );
    }

    /// `/W2` Form A with a CID start near u16::MAX and an inner array
    /// long enough to overflow MUST stop emitting on overflow rather than
    /// silently collapsing every subsequent CID onto u16::MAX via
    /// saturating arithmetic.
    #[test]
    fn test_parse_w2_form_a_stops_on_overflow() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        // cid_start = 0xFFFE — only two slots remain (0xFFFE, 0xFFFF) so
        // the third triple would wrap. Confirm we emit exactly two
        // distinct CIDs, not three (which would imply two metrics
        // collapsed onto u16::MAX) and not zero (which would imply a
        // panic-on-overflow bug).
        dict.insert(
            "W2".to_string(),
            Object::Array(vec![
                Object::Integer(0xFFFE),
                Object::Array(vec![
                    // CID 0xFFFE
                    Object::Integer(-1000),
                    Object::Integer(500),
                    Object::Integer(880),
                    // CID 0xFFFF
                    Object::Integer(-900),
                    Object::Integer(510),
                    Object::Integer(870),
                    // CID 0x10000 — overflows; must be DROPPED.
                    Object::Integer(-800),
                    Object::Integer(520),
                    Object::Integer(860),
                ]),
            ]),
        );
        let metrics = FontInfo::parse_cid_vertical_metrics(&dict, "Test").unwrap();
        assert_eq!(
            metrics.get(&0xFFFE),
            Some(&VerticalMetrics {
                w1y: -1000.0,
                v_x: 500.0,
                v_y: 880.0
            })
        );
        assert_eq!(
            metrics.get(&0xFFFF),
            Some(&VerticalMetrics {
                w1y: -900.0,
                v_x: 510.0,
                v_y: 870.0
            })
        );
        assert_eq!(
            metrics.len(),
            2,
            "Form A overflow must drop overflowing triples; got {} entries",
            metrics.len()
        );
    }

    /// `/DW2` overrides only `v_y` and `w1y`; `v_x` always defaults to
    /// `500` per spec (§9.7.4.3 — only two numbers are settable via /DW2).
    #[test]
    fn test_parse_dw2_overrides_defaults() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "DW2".to_string(),
            Object::Array(vec![Object::Integer(850), Object::Integer(-1100)]),
        );
        let dw2 = FontInfo::parse_dw2(&dict);
        assert_eq!(dw2.v_y, 850.0);
        assert_eq!(dw2.w1y, -1100.0);
        assert_eq!(dw2.v_x, 500.0, "v_x is not settable via /DW2");
    }

    /// Missing `/DW2` ⇒ spec defaults `(w1y=-1000, v_x=500, v_y=880)`.
    #[test]
    fn test_parse_dw2_missing_uses_spec_default() {
        let dict: HashMap<String, Object> = HashMap::new();
        assert_eq!(FontInfo::parse_dw2(&dict), VerticalMetrics::SPEC_DEFAULT);
    }

    /// Malformed `/DW2` (single element instead of two) ⇒ spec defaults.
    /// Better to use safe defaults than expose half-parsed metrics that
    /// would shift glyph positions in unpredictable ways.
    #[test]
    fn test_parse_dw2_short_array_uses_spec_default() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert("DW2".to_string(), Object::Array(vec![Object::Integer(800)]));
        assert_eq!(FontInfo::parse_dw2(&dict), VerticalMetrics::SPEC_DEFAULT);
    }

    /// `/DW2` with real-valued numbers parses cleanly.
    #[test]
    fn test_parse_dw2_real_values() {
        let mut dict: HashMap<String, Object> = HashMap::new();
        dict.insert(
            "DW2".to_string(),
            Object::Array(vec![Object::Real(875.5), Object::Real(-990.25)]),
        );
        let dw2 = FontInfo::parse_dw2(&dict);
        assert_eq!(dw2.v_y, 875.5);
        assert_eq!(dw2.w1y, -990.25);
        assert_eq!(dw2.v_x, 500.0);
    }

    /// `wmode_from_predefined_cmap_name` returns 1 for any name with a `-V`
    /// suffix and for the bare legacy `V`. This is the cheap fast path that
    /// avoids parsing the encoding CMap stream when we already know the name
    /// declares vertical writing.
    #[test]
    fn test_wmode_from_predefined_cmap_name_vertical() {
        assert_eq!(wmode_from_predefined_cmap_name("Identity-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("UniJIS-UTF16-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("UniGB-UTF16-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("UniCNS-UTF16-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("UniKS-UTF16-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("GBK-EUC-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("90ms-RKSJ-V"), 1);
        assert_eq!(wmode_from_predefined_cmap_name("V"), 1);
    }

    /// Horizontal-mode names (the overwhelming majority) must return 0 so
    /// the wmode flag stays cold for normal documents.
    #[test]
    fn test_wmode_from_predefined_cmap_name_horizontal() {
        assert_eq!(wmode_from_predefined_cmap_name("Identity-H"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("UniJIS-UTF16-H"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("UniGB-UTF16-H"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("H"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("WinAnsiEncoding"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("MacRomanEncoding"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("Adobe-Japan1-6"), 0);
        // Edge case: the substring `-V` appears inside but not as a suffix.
        assert_eq!(wmode_from_predefined_cmap_name("V-foo"), 0);
        assert_eq!(wmode_from_predefined_cmap_name("Volt"), 0);
    }

    /// `FontInfo::get_vertical_metrics` returns per-CID metrics when
    /// available, falls back to `/DW2` defaults otherwise. This is the
    /// accessor the rasterizer and extractor call on the hot path of every
    /// vertical-mode glyph.
    #[test]
    fn test_get_vertical_metrics_lookup_precedence() {
        let mut per_cid: HashMap<u16, VerticalMetrics> = HashMap::new();
        per_cid.insert(
            7,
            VerticalMetrics {
                w1y: -900.0,
                v_x: 480.0,
                v_y: 870.0,
            },
        );

        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.cid_vertical_metrics = Some(per_cid);
            f.cid_default_vertical_metrics = VerticalMetrics {
                w1y: -1050.0,
                v_x: 500.0,
                v_y: 900.0,
            };
        });

        // Per-CID hit
        assert_eq!(
            font.get_vertical_metrics(7),
            VerticalMetrics {
                w1y: -900.0,
                v_x: 480.0,
                v_y: 870.0
            }
        );
        // Per-CID miss → /DW2 defaults
        assert_eq!(
            font.get_vertical_metrics(99),
            VerticalMetrics {
                w1y: -1050.0,
                v_x: 500.0,
                v_y: 900.0
            }
        );
    }

    /// When neither `/W2` nor `/DW2` is parsed, `get_vertical_metrics`
    /// returns the spec defaults — keeping rendering correct for the common
    /// case of a CIDFont that ships only horizontal metrics but is used in
    /// a vertical context (caller has already established wmode=1 by name).
    #[test]
    fn test_get_vertical_metrics_spec_default_fallback() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.cid_vertical_metrics = None;
            f.cid_default_vertical_metrics = VerticalMetrics::SPEC_DEFAULT;
        });
        assert_eq!(font.get_vertical_metrics(0x4E00), VerticalMetrics::SPEC_DEFAULT);
        assert_eq!(VerticalMetrics::SPEC_DEFAULT.w1y, -1000.0);
        assert_eq!(VerticalMetrics::SPEC_DEFAULT.v_x, 500.0);
        assert_eq!(VerticalMetrics::SPEC_DEFAULT.v_y, 880.0);
    }

    // =========================================================================
    // shift_jis_to_unicode
    // =========================================================================

    #[test]
    fn test_shift_jis_single_byte_ascii() {
        // Single-byte ASCII should decode normally
        assert_eq!(shift_jis_to_unicode(0x41), Some('A'));
        assert_eq!(shift_jis_to_unicode(0x20), Some(' '));
    }

    #[test]
    fn test_shift_jis_two_byte_katakana() {
        // 0x8341 is Shift-JIS for katakana "ア" (U+30A2)
        assert_eq!(shift_jis_to_unicode(0x8341), Some('ア'));
    }

    #[test]
    fn test_shift_jis_invalid() {
        // 0xFFFF is not a valid Shift-JIS sequence
        assert_eq!(shift_jis_to_unicode(0xFFFF), None);
    }

    // =========================================================================
    // standard_encoding_lookup — extended coverage
    // =========================================================================

    #[test]
    fn test_standard_encoding_lookup_standard_encoding_ascii() {
        assert_eq!(standard_encoding_lookup("StandardEncoding", b'A'), Some("A".to_string()));
        assert_eq!(standard_encoding_lookup("StandardEncoding", b' '), Some(" ".to_string()));
    }

    #[test]
    fn test_standard_encoding_lookup_standard_encoding_extended() {
        // StandardEncoding 0xAE → fi ligature (U+FB01)
        assert_eq!(
            standard_encoding_lookup("StandardEncoding", 0xAE),
            Some("\u{FB01}".to_string())
        );
        // 0xD0 → emdash (U+2014)
        assert_eq!(
            standard_encoding_lookup("StandardEncoding", 0xD0),
            Some("\u{2014}".to_string())
        );
        // 0xA1 → exclamdown
        assert_eq!(
            standard_encoding_lookup("StandardEncoding", 0xA1),
            Some("\u{00A1}".to_string())
        );
    }

    #[test]
    fn test_standard_encoding_lookup_standard_encoding_unmapped() {
        // 0x00 is in the control range, outside 32..=126
        assert_eq!(standard_encoding_lookup("StandardEncoding", 0x00), None);
        // 0xB0 is not mapped in StandardEncoding
        assert_eq!(standard_encoding_lookup("StandardEncoding", 0xB0), None);
    }

    #[test]
    fn test_standard_encoding_lookup_macroman_ascii() {
        assert_eq!(standard_encoding_lookup("MacRomanEncoding", b'Z'), Some("Z".to_string()));
    }

    #[test]
    fn test_standard_encoding_lookup_macroman_extended() {
        // 0x80 → Adieresis (U+00C4)
        assert_eq!(
            standard_encoding_lookup("MacRomanEncoding", 0x80),
            Some("\u{00C4}".to_string())
        );
        // 0xD0 → endash (U+2013)
        assert_eq!(
            standard_encoding_lookup("MacRomanEncoding", 0xD0),
            Some("\u{2013}".to_string())
        );
        // 0xCA → NBSP (U+00A0)
        assert_eq!(
            standard_encoding_lookup("MacRomanEncoding", 0xCA),
            Some("\u{00A0}".to_string())
        );
        // 0xF0 → Apple logo (private use U+F8FF)
        assert_eq!(
            standard_encoding_lookup("MacRomanEncoding", 0xF0),
            Some("\u{F8FF}".to_string())
        );
    }

    #[test]
    fn test_standard_encoding_lookup_macroman_unmapped() {
        // 0x00 is control range
        assert_eq!(standard_encoding_lookup("MacRomanEncoding", 0x00), None);
    }

    #[test]
    fn test_standard_encoding_lookup_winansi_extended() {
        // 0x80 → Euro sign (U+20AC)
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x80), Some("\u{20AC}".to_string()));
        // 0x96 → En dash (U+2013)
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x96), Some("\u{2013}".to_string()));
        // 0xA0 → NBSP direct ISO-8859-1 mapping
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0xA0), Some("\u{00A0}".to_string()));
    }

    #[test]
    fn test_standard_encoding_lookup_winansi_undefined_holes() {
        // 0x81 is undefined in WinAnsi/Windows-1252
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x81), None);
        // 0x8D is undefined
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x8D), None);
    }

    #[test]
    fn test_standard_encoding_lookup_pdfdoc() {
        // 0x80 → bullet (U+2022)
        assert_eq!(standard_encoding_lookup("PDFDocEncoding", 0x80), Some("\u{2022}".to_string()));
        // 0x84 → emdash (U+2014)
        assert_eq!(standard_encoding_lookup("PDFDocEncoding", 0x84), Some("\u{2014}".to_string()));
        // ASCII range
        assert_eq!(standard_encoding_lookup("PDFDocEncoding", b'B'), Some("B".to_string()));
    }

    #[test]
    fn test_standard_encoding_lookup_unknown_encoding() {
        // Unknown encoding: ASCII passthrough for printable chars
        assert_eq!(standard_encoding_lookup("SomeWeirdEncoding", b'X'), Some("X".to_string()));
        // Non-printable or < 32 → None
        assert_eq!(standard_encoding_lookup("SomeWeirdEncoding", 0x01), None);
        // High byte → None (not ASCII)
        assert_eq!(standard_encoding_lookup("SomeWeirdEncoding", 0x80), None);
    }

    // =========================================================================
    // pdfdoc_encoding_lookup
    // =========================================================================

    #[test]
    fn test_pdfdoc_encoding_ascii_range() {
        assert_eq!(pdfdoc_encoding_lookup(0x00), Some('\0'));
        assert_eq!(pdfdoc_encoding_lookup(0x41), Some('A'));
        assert_eq!(pdfdoc_encoding_lookup(0x7F), Some('\x7F'));
    }

    #[test]
    fn test_pdfdoc_encoding_special_range() {
        assert_eq!(pdfdoc_encoding_lookup(0x80), Some('\u{2022}')); // bullet
        assert_eq!(pdfdoc_encoding_lookup(0x85), Some('\u{2013}')); // endash
        assert_eq!(pdfdoc_encoding_lookup(0x93), Some('\u{FB01}')); // fi ligature
        assert_eq!(pdfdoc_encoding_lookup(0x94), Some('\u{FB02}')); // fl ligature
        assert_eq!(pdfdoc_encoding_lookup(0x92), Some('\u{2122}')); // trademark
    }

    #[test]
    fn test_pdfdoc_encoding_undefined() {
        assert_eq!(pdfdoc_encoding_lookup(0x9F), None);
    }

    #[test]
    fn test_pdfdoc_encoding_latin1_range() {
        assert_eq!(pdfdoc_encoding_lookup(0xA0), Some('\u{00A0}')); // NBSP
        assert_eq!(pdfdoc_encoding_lookup(0xFF), Some('\u{00FF}')); // ydieresis
    }

    // =========================================================================
    // symbol_encoding_lookup — extended coverage
    // =========================================================================

    #[test]
    fn test_symbol_encoding_greek_lowercase() {
        assert_eq!(symbol_encoding_lookup(0x61), Some('α'));
        assert_eq!(symbol_encoding_lookup(0x62), Some('β'));
        assert_eq!(symbol_encoding_lookup(0x67), Some('γ'));
        assert_eq!(symbol_encoding_lookup(0x72), Some('ρ'));
        assert_eq!(symbol_encoding_lookup(0x77), Some('ω'));
    }

    #[test]
    fn test_symbol_encoding_greek_uppercase() {
        assert_eq!(symbol_encoding_lookup(0x44), Some('Δ'));
        assert_eq!(symbol_encoding_lookup(0x53), Some('Σ'));
        assert_eq!(symbol_encoding_lookup(0x57), Some('Ω'));
    }

    #[test]
    fn test_symbol_encoding_math_operators() {
        assert_eq!(symbol_encoding_lookup(0xE1), Some('∑')); // summation
        assert_eq!(symbol_encoding_lookup(0xF2), Some('∫')); // integral
        assert_eq!(symbol_encoding_lookup(0xD6), Some('√')); // radical
        assert_eq!(symbol_encoding_lookup(0xB1), Some('±')); // plusminus
        assert_eq!(symbol_encoding_lookup(0xB9), Some('≠')); // notequal
    }

    #[test]
    fn test_symbol_encoding_digits() {
        // Digits 0x30-0x39 map to themselves
        assert_eq!(symbol_encoding_lookup(0x30), Some('0'));
        assert_eq!(symbol_encoding_lookup(0x39), Some('9'));
    }

    #[test]
    fn test_symbol_encoding_punctuation() {
        assert_eq!(symbol_encoding_lookup(0x20), Some(' '));
        assert_eq!(symbol_encoding_lookup(0x2B), Some('+'));
        assert_eq!(symbol_encoding_lookup(0x2D), Some('−')); // minus (not hyphen)
    }

    #[test]
    fn test_symbol_encoding_unmapped() {
        assert_eq!(symbol_encoding_lookup(0x00), None);
        assert_eq!(symbol_encoding_lookup(0x01), None);
    }

    // =========================================================================
    // zapf_dingbats_encoding_lookup — extended coverage
    // =========================================================================

    #[test]
    fn test_zapf_dingbats_common() {
        assert_eq!(zapf_dingbats_encoding_lookup(0x20), Some(' '));
        assert_eq!(zapf_dingbats_encoding_lookup(0x21), Some('✁'));
        assert_eq!(zapf_dingbats_encoding_lookup(0x33), Some('✓')); // checkmark
        assert_eq!(zapf_dingbats_encoding_lookup(0x34), Some('✔')); // bold checkmark
        assert_eq!(zapf_dingbats_encoding_lookup(0x48), Some('★')); // black star
    }

    #[test]
    fn test_zapf_dingbats_geometric() {
        assert_eq!(zapf_dingbats_encoding_lookup(0x6C), Some('●')); // black circle
        assert_eq!(zapf_dingbats_encoding_lookup(0x6F), Some('■')); // black square
    }

    /// ZapfDingbats circled-digit ranges (Annex D.6); codes in hex of the
    /// spec's octal CODE column.
    #[test]
    fn test_zapf_dingbats_circled_digits() {
        // ① ⑩  (a120–a129, octal 254–265) → U+2460–U+2469
        assert_eq!(zapf_dingbats_encoding_lookup(0xAC), Some('\u{2460}')); // ①
        assert_eq!(zapf_dingbats_encoding_lookup(0xB5), Some('\u{2469}')); // ⑩
                                                                           // ❶ ❿  (a130–a139, octal 266–277) → U+2776–U+277F
        assert_eq!(zapf_dingbats_encoding_lookup(0xB6), Some('\u{2776}')); // ❶
        assert_eq!(zapf_dingbats_encoding_lookup(0xBF), Some('\u{277F}')); // ❿
                                                                           // ➀ ➉  (a140–a149, octal 300–311) → U+2780–U+2789
        assert_eq!(zapf_dingbats_encoding_lookup(0xC0), Some('\u{2780}')); // ➀
        assert_eq!(zapf_dingbats_encoding_lookup(0xC9), Some('\u{2789}')); // ➉
                                                                           // ➊ ➓  (a150–a159, octal 312–323) → U+278A–U+2793
        assert_eq!(zapf_dingbats_encoding_lookup(0xCA), Some('\u{278A}')); // ➊
        assert_eq!(zapf_dingbats_encoding_lookup(0xD3), Some('\u{2793}')); // ➓
    }

    /// ZapfDingbats arrow ranges (Annex D.6, octal 324–376).
    #[test]
    fn test_zapf_dingbats_arrows() {
        assert_eq!(zapf_dingbats_encoding_lookup(0xD4), Some('\u{2794}')); // ➔
        assert_eq!(zapf_dingbats_encoding_lookup(0xD5), Some('\u{2192}')); // →
        assert_eq!(zapf_dingbats_encoding_lookup(0xD8), Some('\u{2798}')); // ➘
        assert_eq!(zapf_dingbats_encoding_lookup(0xEF), Some('\u{27AF}')); // ➯
        assert_eq!(zapf_dingbats_encoding_lookup(0xF0), None); // undefined in D.6
        assert_eq!(zapf_dingbats_encoding_lookup(0xF1), Some('\u{27B1}')); // ➱
        assert_eq!(zapf_dingbats_encoding_lookup(0xFE), Some('\u{27BE}')); // ➾
    }

    #[test]
    fn test_zapf_dingbats_unmapped() {
        assert_eq!(zapf_dingbats_encoding_lookup(0x00), None);
        assert_eq!(zapf_dingbats_encoding_lookup(0xFF), None);
    }

    // =========================================================================
    // glyph_name_to_unicode — extended edge cases
    // =========================================================================

    #[test]
    fn test_glyph_name_to_unicode_tex_math() {
        assert_eq!(glyph_name_to_unicode("square"), Some('\u{25A1}'));
        assert_eq!(glyph_name_to_unicode("emptyset"), Some('\u{2205}'));
        assert_eq!(glyph_name_to_unicode("infty"), Some('\u{221E}'));
        assert_eq!(glyph_name_to_unicode("nabla"), Some('\u{2207}'));
        assert_eq!(glyph_name_to_unicode("forall"), Some('\u{2200}'));
        assert_eq!(glyph_name_to_unicode("checkmark"), Some('\u{2713}'));
    }

    #[test]
    fn test_glyph_name_to_unicode_underscore_compound() {
        // "f_f" should return first component 'f' via AGL
        assert_eq!(glyph_name_to_unicode("f_f"), Some('f'));
        // "T_h" should return first component 'T' via AGL
        assert_eq!(glyph_name_to_unicode("T_h"), Some('T'));
    }

    #[test]
    fn test_glyph_name_to_unicode_uni_format_edge_cases() {
        // Too short (not 7 chars total)
        assert_eq!(glyph_name_to_unicode("uni004"), None);
        // Invalid hex
        assert_eq!(glyph_name_to_unicode("uniZZZZ"), None);
    }

    #[test]
    fn test_glyph_name_to_unicode_u_format_long() {
        // u1F600 = grinning face emoji
        assert_eq!(glyph_name_to_unicode("u1F600"), Some('\u{1F600}'));
    }

    // =========================================================================
    // glyph_name_to_unicode_string — compound names
    // =========================================================================

    #[test]
    fn test_glyph_name_to_unicode_string_simple() {
        // Single char should just return it as string
        assert_eq!(glyph_name_to_unicode_string("A"), Some("A".to_string()));
    }

    #[test]
    fn test_glyph_name_to_unicode_string_compound_ff() {
        // glyph_name_to_unicode("f_f") returns Some('f') — first component via AGL
        // So glyph_name_to_unicode_string wraps it as "f" (single-char short-circuit)
        assert_eq!(glyph_name_to_unicode_string("f_f"), Some("f".to_string()));
    }

    #[test]
    fn test_glyph_name_to_unicode_string_compound_all_known() {
        // Use a compound name where each component is known individually.
        // "T_h" → glyph_name_to_unicode finds 'T' (first component) → returns "T"
        assert_eq!(glyph_name_to_unicode_string("T_h"), Some("T".to_string()));
    }

    #[test]
    fn test_glyph_name_to_unicode_string_compound_unknown_part() {
        // "f_unknownglyph" — glyph_name_to_unicode finds 'f' (first component via underscore rule)
        // So it returns Some("f") not None
        assert_eq!(glyph_name_to_unicode_string("f_unknownglyph"), Some("f".to_string()));
    }

    #[test]
    fn test_glyph_name_to_unicode_string_totally_unknown_compound() {
        // Both parts unknown — should return None
        assert_eq!(glyph_name_to_unicode_string("xyzzy_plugh"), None);
    }

    #[test]
    fn test_glyph_name_to_unicode_string_unknown() {
        assert_eq!(glyph_name_to_unicode_string("totallyunknown"), None);
    }

    // =========================================================================
    // #535 follow-up — unified AGL fallback chain (v0.3.55)
    //
    // The #535 fix added a robust ToUnicode + embedded-cmap + AGL
    // fallback chain in `src/fonts/character_mapper.rs::glyph_name_to_unicode`,
    // but the original full-document Type0 / Identity-H call site at
    // `font_dict.rs::Font::char_code_to_unicode` was the only consumer. Simple
    // fonts, Type1 / CFF embedded encodings, and `/Differences` arrays still
    // routed through this `font_dict::glyph_name_to_unicode` entry, which
    // lacked the newer chain's variant-suffix stripping (`.alt`, `.sc`,
    // `.001`). delegates to the unified chain as a final fallback so
    // all callers — including any future inline-image font-resolution path
    // (PDF spec §8.9.7) — share the same behaviour.
    //
    // Refs #535.
    // =========================================================================

    #[test]
    fn glyph_name_with_variant_suffix_resolves_via_unified_chain() {
        // Subset fonts append stylistic-variant tags (`.sc`, `.alt`, `.001`)
        // to the canonical glyph name. The chain strips the suffix
        // returns the base codepoint; this entry now picks that up too.
        assert_eq!(glyph_name_to_unicode("A.sc"), Some('A'));
        assert_eq!(glyph_name_to_unicode("bullet.alt"), Some('\u{2022}'));
        assert_eq!(glyph_name_to_unicode("fi.001"), Some('\u{FB01}'));
        // Unknown base + suffix → still unknown.
        assert_eq!(glyph_name_to_unicode("xyzzy.sc"), None);
    }

    #[test]
    fn glyph_name_string_with_variant_suffix_resolves_via_unified_chain() {
        // Same as above through the multi-codepoint return surface used by
        // /Differences-array parsing.
        assert_eq!(glyph_name_to_unicode_string("A.sc"), Some("A".to_string()));
        assert_eq!(glyph_name_to_unicode_string("bullet.alt"), Some("\u{2022}".to_string()));
        assert_eq!(glyph_name_to_unicode_string("fi.001"), Some("\u{FB01}".to_string()));
    }

    #[test]
    fn unified_chain_does_not_regress_existing_lookups() {
        // Belt-and-suspenders: the canonical AGL names and uniXXXX / uXXXXX
        // synth patterns the old chain handled stay byte-identical.
        assert_eq!(glyph_name_to_unicode("A"), Some('A'));
        assert_eq!(glyph_name_to_unicode("space"), Some(' '));
        assert_eq!(glyph_name_to_unicode("bullet"), Some('\u{2022}'));
        assert_eq!(glyph_name_to_unicode("fi"), Some('\u{FB01}'));
        assert_eq!(glyph_name_to_unicode("uni2022"), Some('\u{2022}'));
        assert_eq!(glyph_name_to_unicode("u1F600"), Some('\u{1F600}'));
        // Unknown stays unknown.
        assert_eq!(glyph_name_to_unicode("totallyunknown"), None);
    }

    // =========================================================================
    // is_ligature_char and expand_ligature_char
    // =========================================================================

    #[test]
    fn test_is_ligature_char_all_variants() {
        assert!(is_ligature_char('\u{FB00}')); // ff
        assert!(is_ligature_char('\u{FB01}')); // fi
        assert!(is_ligature_char('\u{FB02}')); // fl
        assert!(is_ligature_char('\u{FB03}')); // ffi
        assert!(is_ligature_char('\u{FB04}')); // ffl
        assert!(is_ligature_char('\u{FB05}')); // long s + t
        assert!(is_ligature_char('\u{FB06}')); // st
        assert!(!is_ligature_char('A'));
        assert!(!is_ligature_char(' '));
    }

    #[test]
    fn test_expand_ligature_char_all_variants() {
        assert_eq!(expand_ligature_char('\u{FB00}'), Some("ff"));
        assert_eq!(expand_ligature_char('\u{FB01}'), Some("fi"));
        assert_eq!(expand_ligature_char('\u{FB02}'), Some("fl"));
        assert_eq!(expand_ligature_char('\u{FB03}'), Some("ffi"));
        assert_eq!(expand_ligature_char('\u{FB04}'), Some("ffl"));
        assert_eq!(expand_ligature_char('\u{FB05}'), Some("st"));
        assert_eq!(expand_ligature_char('\u{FB06}'), Some("st"));
        assert_eq!(expand_ligature_char('x'), None);
    }

    // =========================================================================
    // get_glyph_width — simple font widths array
    // =========================================================================

    #[test]
    fn test_get_glyph_width_simple_font_widths_array() {
        let font = make_font(|f| {
            f.widths = Some(vec![200.0, 300.0, 400.0, 500.0]);
            f.first_char = Some(65); // 'A'
            f.last_char = Some(68); // 'D'
            f.default_width = 600.0;
        });
        assert_eq!(font.get_glyph_width(65), 200.0); // 'A'
        assert_eq!(font.get_glyph_width(66), 300.0); // 'B'
        assert_eq!(font.get_glyph_width(67), 400.0); // 'C'
        assert_eq!(font.get_glyph_width(68), 500.0); // 'D'
                                                     // Out of range → default_width
        assert_eq!(font.get_glyph_width(64), 600.0);
        assert_eq!(font.get_glyph_width(69), 600.0);
    }

    #[test]
    fn test_get_glyph_width_below_first_char() {
        let font = make_font(|f| {
            f.widths = Some(vec![250.0]);
            f.first_char = Some(100);
            f.last_char = Some(100);
            f.default_width = 777.0;
        });
        // char_code < first_char → negative index → default
        assert_eq!(font.get_glyph_width(50), 777.0);
    }

    #[test]
    fn test_get_glyph_width_no_widths_no_cid() {
        let font = make_font(|f| {
            f.default_width = 550.0;
        });
        assert_eq!(font.get_glyph_width(65), 550.0);
    }

    // =========================================================================
    // get_space_glyph_width
    // =========================================================================

    #[test]
    fn test_get_space_glyph_width_from_array() {
        let font = make_font(|f| {
            f.widths = Some(vec![250.0]); // only one entry
            f.first_char = Some(32); // space = 0x20 = 32
            f.last_char = Some(32);
        });
        assert_eq!(font.get_space_glyph_width(), 250.0);
    }

    #[test]
    fn test_get_space_glyph_width_default() {
        let font = make_font(|f| {
            f.default_width = 333.0;
        });
        assert_eq!(font.get_space_glyph_width(), 333.0);
    }

    #[test]
    fn test_normalize_cjk_radical_forms() {
        // Kangxi Radicals (U+2F00–2FDF) → unified ideograph.
        assert_eq!(normalize_cjk_radical_forms("⽋点"), "欠点");
        assert_eq!(normalize_cjk_radical_forms("⽴⾮⾔⾦"), "立非言金");
        // Mixed radical + normal text: only the radical is rewritten.
        assert_eq!(normalize_cjk_radical_forms("実⽴確率"), "実立確率");
        // Fast path: no radical-block char → returned unchanged (incl. fullwidth).
        assert_eq!(normalize_cjk_radical_forms("欠点０１２"), "欠点０１２");
        assert_eq!(normalize_cjk_radical_forms("hello"), "hello");
    }

    // =========================================================================
    // is_symbolic — flags and name-based detection
    // =========================================================================

    #[test]
    fn test_is_symbolic_flag_set() {
        let font = make_font(|f| {
            f.flags = Some(0x04); // bit 3 set
        });
        assert!(font.is_symbolic());
    }

    #[test]
    fn test_is_symbolic_flag_not_set() {
        let font = make_font(|f| {
            f.flags = Some(0x20); // nonsymbolic bit only
        });
        assert!(!font.is_symbolic());
    }

    #[test]
    fn test_is_symbolic_no_flags_symbol_name() {
        let font = make_font(|f| {
            f.base_font = "Symbol".to_string();
        });
        assert!(font.is_symbolic());
    }

    #[test]
    fn test_is_symbolic_no_flags_zapf_name() {
        let font = make_font(|f| {
            f.base_font = "ZapfDingbats".to_string();
        });
        assert!(font.is_symbolic());
    }

    #[test]
    fn test_is_symbolic_no_flags_normal_name() {
        let font = make_font(|f| {
            f.base_font = "Helvetica".to_string();
        });
        assert!(!font.is_symbolic());
    }

    // =========================================================================
    // get_encoded_char
    // =========================================================================

    #[test]
    fn test_get_encoded_char_custom() {
        let mut map = HashMap::new();
        map.insert(0x41, 'X');
        map.insert(0x42, 'Y');
        let font = make_font(|f| {
            f.encoding = Encoding::Custom(map);
        });
        assert_eq!(font.get_encoded_char(0x41), Some('X'));
        assert_eq!(font.get_encoded_char(0x42), Some('Y'));
        assert_eq!(font.get_encoded_char(0x43), None);
    }

    #[test]
    fn test_get_encoded_char_standard_ascii() {
        let font = make_font(|f| {
            f.encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        });
        assert_eq!(font.get_encoded_char(0x41), Some('A'));
        assert_eq!(font.get_encoded_char(0x20), Some(' '));
        // High byte → None (>= 128)
        assert_eq!(font.get_encoded_char(0x80), None);
    }

    #[test]
    fn test_get_encoded_char_identity_ascii() {
        let font = make_font(|f| {
            f.encoding = Encoding::Identity;
        });
        assert_eq!(font.get_encoded_char(0x41), Some('A'));
        assert_eq!(font.get_encoded_char(0x80), None);
    }

    // =========================================================================
    // has_custom_encoding
    // =========================================================================

    #[test]
    fn test_has_custom_encoding_true() {
        let font = make_font(|f| {
            f.encoding = Encoding::Custom(HashMap::new());
        });
        assert!(font.has_custom_encoding());
    }

    #[test]
    fn test_has_custom_encoding_false_standard() {
        let font = make_font(|_| {});
        assert!(!font.has_custom_encoding());
    }

    #[test]
    fn test_has_custom_encoding_false_identity() {
        let font = make_font(|f| {
            f.encoding = Encoding::Identity;
        });
        assert!(!font.has_custom_encoding());
    }

    // =========================================================================
    // char_to_unicode — Symbol font path
    // =========================================================================

    #[test]
    fn test_char_to_unicode_symbol_font() {
        let font = make_font(|f| {
            f.base_font = "Symbol".to_string();
            f.flags = Some(0x04); // Symbolic
            f.encoding = Encoding::Standard("SymbolicBuiltIn".to_string());
        });
        // alpha
        assert_eq!(font.char_to_unicode(0x61), Some("α".to_string()));
        // Sigma
        assert_eq!(font.char_to_unicode(0x53), Some("Σ".to_string()));
        // integral
        assert_eq!(font.char_to_unicode(0xF2), Some("∫".to_string()));
    }

    #[test]
    fn test_char_to_unicode_zapfdingbats_font() {
        let font = make_font(|f| {
            f.base_font = "ZapfDingbats".to_string();
            f.flags = Some(0x04); // Symbolic
            f.encoding = Encoding::Standard("SymbolicBuiltIn".to_string());
        });
        // checkmark
        assert_eq!(font.char_to_unicode(0x33), Some("✓".to_string()));
        // star
        assert_eq!(font.char_to_unicode(0x48), Some("★".to_string()));
    }

    // =========================================================================
    // char_to_unicode — ligature expansion path
    // =========================================================================

    #[test]
    fn test_char_to_unicode_ligature_fallback_expansion() {
        // When no encoding/ToUnicode mapping exists, Priority 6 falls back
        // to standard Unicode ligature decomposition (U+FB00–FB06 → components).
        let font = make_font(|f| {
            f.encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        });
        assert_eq!(font.char_to_unicode(0xFB01), Some("fi".to_string()));
        assert_eq!(font.char_to_unicode(0xFB03), Some("ffi".to_string()));
    }

    // =========================================================================
    // char_to_unicode — custom encoding with ligature
    // =========================================================================

    #[test]
    fn test_char_to_unicode_custom_encoding_with_ligature() {
        let mut custom = HashMap::new();
        custom.insert(0x01, '\u{FB01}'); // fi ligature
        let font = make_font(|f| {
            f.encoding = Encoding::Custom(custom);
        });
        // Should expand ligature
        assert_eq!(font.char_to_unicode(0x01), Some("fi".to_string()));
    }

    #[test]
    fn test_char_to_unicode_custom_encoding_multi_char_map() {
        let font = make_font(|f| {
            f.encoding = Encoding::Custom(HashMap::new());
            f.multi_char_map.insert(0x01, "ff".to_string());
        });
        assert_eq!(font.char_to_unicode(0x01), Some("ff".to_string()));
    }

    // =========================================================================
    // char_to_unicode — ToUnicode with U+FFFD (replacement character skip)
    // =========================================================================

    #[test]
    fn test_char_to_unicode_tounicode_fffd_fallback() {
        // A ToUnicode mapping to U+FFFD means the font author explicitly declared
        // "no Unicode equivalent" for this code. Per Fix B (§9.10.2) the function
        // must return U+FFFD and NOT fall through to the encoding-based path.
        let cmap_data = b"beginbfchar\n<0041> <FFFD>\nendbfchar";
        let font = make_font(|f| {
            f.to_unicode = Some(LazyCMap::new(cmap_data.to_vec()));
            f.encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        });
        // ToUnicode says U+FFFD → return U+FFFD, do NOT fall through to WinAnsi 'A'
        assert_eq!(font.char_to_unicode(0x41), Some("\u{FFFD}".to_string()));
    }

    #[test]
    fn test_char_to_unicode_tounicode_control_char_fallback() {
        // A ToUnicode mapping to a C0 control character is filtered by Fix B.
        // The function must return U+FFFD and NOT fall through to the encoding.
        let cmap_data = b"beginbfchar\n<0041> <0001>\nendbfchar";
        let font = make_font(|f| {
            f.to_unicode = Some(LazyCMap::new(cmap_data.to_vec()));
            f.encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        });
        // C0 control char (U+0001) → U+FFFD, do NOT fall through to WinAnsi 'A'
        assert_eq!(font.char_to_unicode(0x41), Some("\u{FFFD}".to_string()));
    }

    // =========================================================================
    // char_to_unicode — Type0 with Identity-H and CIDSystemInfo
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_identity_h_with_sysinfo() {
        let font = make_font(|f| {
            f.base_font = "CIDFont+F1".to_string();
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Identity-H".to_string());
            f.cid_system_info = Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Identity".to_string(),
                supplement: 0,
            });
            f.cid_to_gid_map = Some(CIDToGIDMap::Identity);
        });
        // Adobe-Identity with no TrueType cmap → CID-as-Unicode fallback
        // For non-control Unicode code points, char_code == Unicode
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
        assert_eq!(font.char_to_unicode(0x4E2D), Some("\u{4E2D}".to_string())); // 中
    }

    #[test]
    fn test_char_to_unicode_type0_identity_h_no_sysinfo() {
        let font = make_font(|f| {
            f.base_font = "CIDFont+F2".to_string();
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Identity-H".to_string());
        });
        // No CIDSystemInfo → CID-as-Unicode last resort
        assert_eq!(font.char_to_unicode(0x42), Some("B".to_string()));
    }

    // =========================================================================
    // char_to_unicode — Type0 with Identity encoding (not Standard)
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_identity_encoding_cid_as_unicode() {
        let font = make_font(|f| {
            f.base_font = "MyCIDFont".to_string();
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity;
            // No TrueType cmap, no CIDToGIDMap → CID-as-Unicode fallback
        });
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
    }

    #[test]
    fn test_char_to_unicode_type0_identity_encoding_control_char() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity;
        });
        // Control char (0x01) should return FFFD because CID-as-Unicode skips controls
        // but the last resort returns FFFD
        let result = font.char_to_unicode(0x01);
        assert_eq!(result, Some("\u{FFFD}".to_string()));
    }

    // =========================================================================
    // char_to_unicode — Identity encoding for simple (non-Type0) fonts
    // =========================================================================

    #[test]
    fn test_char_to_unicode_simple_font_identity() {
        let font = make_font(|f| {
            f.subtype = "Type1".to_string();
            f.encoding = Encoding::Identity;
        });
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
        assert_eq!(font.char_to_unicode(0x263A), Some("☺".to_string()));
    }

    // =========================================================================
    // char_to_unicode — TrueType StandardEncoding fallback
    // =========================================================================

    #[test]
    fn test_char_to_unicode_truetype_standard_encoding_ascii() {
        let font = make_font(|f| {
            f.subtype = "TrueType".to_string();
            f.encoding = Encoding::Standard("StandardEncoding".to_string());
        });
        // Should use standard encoding lookup for ASCII
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
    }

    // =========================================================================
    // char_to_unicode — MacRomanEncoding
    // =========================================================================

    #[test]
    fn test_char_to_unicode_macroman_extended() {
        let font = make_font(|f| {
            f.encoding = Encoding::Standard("MacRomanEncoding".to_string());
        });
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
        // 0x80 → Adieresis (Ä)
        assert_eq!(font.char_to_unicode(0x80), Some("\u{00C4}".to_string()));
    }

    // =========================================================================
    // get_font_weight — DemiBold name heuristic
    // =========================================================================

    #[test]
    fn test_get_font_weight_demibold() {
        let font = make_font(|f| {
            f.base_font = "MyFont-DemiBold".to_string();
        });
        assert_eq!(font.get_font_weight(), FontWeight::SemiBold);
    }

    #[test]
    fn test_get_font_weight_heavy() {
        let font = make_font(|f| {
            f.base_font = "MyFont-Heavy".to_string();
        });
        assert_eq!(font.get_font_weight(), FontWeight::Black);
    }

    #[test]
    fn test_get_font_weight_ultrabold() {
        let font = make_font(|f| {
            f.base_font = "MyFont-UltraBold".to_string();
        });
        assert_eq!(font.get_font_weight(), FontWeight::ExtraBold);
    }

    #[test]
    fn test_get_font_weight_ultralight() {
        let font = make_font(|f| {
            f.base_font = "MyFont-UltraLight".to_string();
        });
        assert_eq!(font.get_font_weight(), FontWeight::ExtraLight);
    }

    // =========================================================================
    // get_byte_to_char_table
    // =========================================================================

    #[test]
    fn test_get_byte_to_char_table_basic() {
        let font = make_font(|f| {
            f.encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        });
        let table = font.get_byte_to_char_table();
        // ASCII 'A' (0x41 = 65) should be 'A'
        assert_eq!(table[0x41], 'A');
        // space (0x20 = 32)
        assert_eq!(table[0x20], ' ');
        // Control chars (except tab/newline/cr) should be '\0'
        assert_eq!(table[0x01], '\0');
    }

    #[test]
    fn test_get_byte_to_char_table_tab_newline_passthrough() {
        let font = make_font(|f| {
            let mut custom = HashMap::new();
            custom.insert(0x09u8, '\t');
            custom.insert(0x0Au8, '\n');
            custom.insert(0x0Du8, '\r');
            f.encoding = Encoding::Custom(custom);
        });
        let table = font.get_byte_to_char_table();
        assert_eq!(table[0x09], '\t');
        assert_eq!(table[0x0A], '\n');
        assert_eq!(table[0x0D], '\r');
    }

    // =========================================================================
    // get_byte_to_width_table
    // =========================================================================

    #[test]
    fn test_get_byte_to_width_table_basic() {
        let font = make_font(|f| {
            f.widths = Some(vec![200.0, 300.0, 400.0]);
            f.first_char = Some(65); // 'A'
            f.default_width = 500.0;
        });
        let table = font.get_byte_to_width_table();
        assert_eq!(table[65], 200.0);
        assert_eq!(table[66], 300.0);
        assert_eq!(table[67], 400.0);
        // Unmapped code uses default
        assert_eq!(table[0], 500.0);
        assert_eq!(table[100], 500.0);
    }

    #[test]
    fn test_get_byte_to_width_table_no_widths() {
        let font = make_font(|f| {
            f.default_width = 600.0;
        });
        let table = font.get_byte_to_width_table();
        // All entries should be default_width
        for &w in table.iter() {
            assert_eq!(w, 600.0);
        }
    }

    // =========================================================================
    // lookup_predefined_cmap — fallback by ordering alone
    // =========================================================================

    #[test]
    fn test_lookup_predefined_cmap_ordering_fallback_gb1() {
        // Even with non-standard CMap name, ordering "GB1" should work
        let sysinfo = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "GB1".to_string(),
            supplement: 2,
        });
        assert_eq!(lookup_predefined_cmap("SomeCustomCMap", &sysinfo, 34), Some(0x41));
    }

    #[test]
    fn test_lookup_predefined_cmap_ordering_fallback_japan1() {
        let sysinfo = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Japan1".to_string(),
            supplement: 4,
        });
        assert_eq!(lookup_predefined_cmap("CustomJapanCMap", &sysinfo, 34), Some(0x41));
    }

    #[test]
    fn test_lookup_predefined_cmap_ordering_fallback_cns1() {
        let sysinfo = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "CNS1".to_string(),
            supplement: 3,
        });
        assert_eq!(lookup_predefined_cmap("CustomCNSCMap", &sysinfo, 34), Some(0x41));
    }

    #[test]
    fn test_lookup_predefined_cmap_ordering_fallback_korea1() {
        let sysinfo = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Korea1".to_string(),
            supplement: 1,
        });
        assert_eq!(lookup_predefined_cmap("CustomKoreaCMap", &sysinfo, 34), Some(0x41));
    }

    #[test]
    fn test_lookup_predefined_cmap_unknown_ordering() {
        let sysinfo = Some(CIDSystemInfo {
            registry: "Custom".to_string(),
            ordering: "Unknown".to_string(),
            supplement: 0,
        });
        assert_eq!(lookup_predefined_cmap("AnyCMap", &sysinfo, 34), None);
    }

    // =========================================================================
    // truetype_cmap() accessor — non-TrueType font
    // =========================================================================

    #[test]
    fn test_truetype_cmap_not_truetype() {
        let font = make_font(|f| {
            f.is_truetype_font = false;
            f.embedded_font_data = None;
        });
        assert!(font.truetype_cmap().is_none());
    }

    #[test]
    fn test_truetype_cmap_truetype_no_data() {
        let font = make_font(|f| {
            f.is_truetype_font = true;
            f.embedded_font_data = None;
        });
        assert!(font.truetype_cmap().is_none());
    }

    #[test]
    fn test_truetype_cmap_truetype_empty_data() {
        let font = make_font(|f| {
            f.is_truetype_font = true;
            f.embedded_font_data = Some(Arc::new(vec![]));
        });
        assert!(font.truetype_cmap().is_none());
    }

    #[test]
    fn test_truetype_cmap_truetype_invalid_data() {
        let font = make_font(|f| {
            f.is_truetype_font = true;
            f.embedded_font_data = Some(Arc::new(vec![0xFF, 0xFF, 0xFF, 0xFF]));
        });
        // Invalid font data → extraction fails → None
        assert!(font.truetype_cmap().is_none());
    }

    #[test]
    fn test_has_truetype_cmap_no_data() {
        let font = make_font(|f| {
            f.is_truetype_font = false;
        });
        assert!(!font.has_truetype_cmap());
    }

    // =========================================================================
    // set_truetype_cmap
    // =========================================================================

    #[test]
    fn test_set_truetype_cmap_to_none() {
        let mut font = make_font(|_| {});
        font.set_truetype_cmap(None);
        assert!(font.truetype_cmap().is_none());
    }

    // =========================================================================
    // CIDToGIDMap edge cases
    // =========================================================================

    #[test]
    fn test_cid_to_gid_explicit_empty() {
        let map = CIDToGIDMap::Explicit(vec![]);
        // Empty array → all fall back to identity
        assert_eq!(map.get_gid(0), 0);
        assert_eq!(map.get_gid(100), 100);
    }

    #[test]
    fn test_cid_to_gid_explicit_boundary() {
        let map = CIDToGIDMap::Explicit(vec![99, 88]);
        assert_eq!(map.get_gid(0), 99);
        assert_eq!(map.get_gid(1), 88);
        // index 2 is out of bounds → identity
        assert_eq!(map.get_gid(2), 2);
    }

    #[test]
    fn test_cid_to_gid_identity_max() {
        let map = CIDToGIDMap::Identity;
        assert_eq!(map.get_gid(u16::MAX), u16::MAX);
    }

    // =========================================================================
    // char_to_unicode — Type0 Identity encoding with CIDToGIDMap + AGL fallback
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_identity_agl_fallback() {
        let font = make_font(|f| {
            f.base_font = "SubsetFont+F3".to_string();
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity;
            f.cid_to_gid_map = Some(CIDToGIDMap::Identity);
            // No TrueType cmap → AGL fallback path
        });
        // CID 0x41 → GID 0x41 → glyph name "A" → AGL → 'A'
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
    }

    // =========================================================================
    // char_to_unicode — Type0 RKSJ (Shift-JIS) path
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_rksj() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("90ms-RKSJ-H".to_string());
        });
        // ASCII char through Shift-JIS
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
    }

    // =========================================================================
    // char_to_unicode — Type0 Identity-H/V at Priority 3 fallback
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_identity_v() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Identity-V".to_string());
        });
        // No CIDSystemInfo → CID-as-Unicode last resort
        assert_eq!(font.char_to_unicode(0x42), Some("B".to_string()));
    }

    // =========================================================================
    // char_to_unicode — unknown encoding for simple font
    // =========================================================================

    #[test]
    fn test_char_to_unicode_unknown_standard_encoding() {
        let font = make_font(|f| {
            f.encoding = Encoding::Standard("SomeRandomEncoding".to_string());
        });
        // Unknown encoding falls back to ASCII passthrough for printable
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
        // Non-ASCII will return None from standard_encoding_lookup
        assert_eq!(font.char_to_unicode(0x80), None);
    }

    // =========================================================================
    // Encoding enum Debug/Clone
    // =========================================================================

    #[test]
    fn test_encoding_identity_clone() {
        let enc = Encoding::Identity;
        let enc2 = enc.clone();
        assert!(matches!(enc2, Encoding::Identity));
    }

    #[test]
    fn test_encoding_custom_clone() {
        let mut map = HashMap::new();
        map.insert(1u8, 'X');
        let enc = Encoding::Custom(map);
        let enc2 = enc.clone();
        match enc2 {
            Encoding::Custom(m) => assert_eq!(m.get(&1), Some(&'X')),
            _ => panic!("Wrong encoding type"),
        }
    }

    #[test]
    fn test_encoding_debug() {
        let enc = Encoding::Standard("WinAnsiEncoding".to_string());
        let debug = format!("{:?}", enc);
        assert!(debug.contains("WinAnsiEncoding"));
    }

    // =========================================================================
    // CIDSystemInfo clone/debug
    // =========================================================================

    #[test]
    fn test_cidsysteminfo_clone() {
        let info = CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Japan1".to_string(),
            supplement: 6,
        };
        let info2 = info.clone();
        assert_eq!(info2.registry, "Adobe");
        assert_eq!(info2.ordering, "Japan1");
        assert_eq!(info2.supplement, 6);
    }

    #[test]
    fn test_cidsysteminfo_debug() {
        let info = CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "GB1".to_string(),
            supplement: 2,
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("Adobe"));
        assert!(debug.contains("GB1"));
    }

    // =========================================================================
    // CIDToGIDMap clone/debug
    // =========================================================================

    #[test]
    fn test_cidtogidmap_clone() {
        let map = CIDToGIDMap::Explicit(vec![1, 2, 3]);
        let map2 = map.clone();
        assert_eq!(map2.get_gid(0), 1);
    }

    #[test]
    fn test_cidtogidmap_debug() {
        let map = CIDToGIDMap::Identity;
        let debug = format!("{:?}", map);
        assert!(debug.contains("Identity"));
    }

    // =========================================================================
    // char_to_unicode — Type0 Identity encoding with large CID (> 0xFFFF)
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_identity_large_cid() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity;
            f.cid_to_gid_map = Some(CIDToGIDMap::Identity);
        });
        // CID > 0xFFFF: TrueType cmap lookup returns early with None,
        // AGL fallback also returns early for large CIDs,
        // then CID-as-Unicode fallback kicks in: 0x10000 is valid Unicode (Linear B Syllable B008 A)
        assert_eq!(font.char_to_unicode(0x10000), Some("\u{10000}".to_string()));
        // But a CID that maps to a control character should return FFFD
        assert_eq!(font.char_to_unicode(0x01), Some("\u{FFFD}".to_string()));
    }

    // =========================================================================
    // char_to_unicode — Type0 predefined CMap fallback (Priority 2b)
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_predefined_cmap_japan1() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Identity; // Will be tried at priority 2b
            f.cid_system_info = Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Japan1".to_string(),
                supplement: 4,
            });
        });
        // CID 843 → Hiragana あ (U+3042) via predefined Japan1 CMap
        assert_eq!(font.char_to_unicode(843), Some("\u{3042}".to_string()));
    }

    // =========================================================================
    // gid_to_standard_glyph_name — boundary checks
    // =========================================================================

    #[test]
    fn test_gid_to_standard_glyph_name_boundary_values() {
        // First valid entry
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x20), Some("space"));
        // Last valid in basic ASCII
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x7E), Some("asciitilde"));
        // Just before first valid
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x1F), None);
        // 0x7F (DEL) is not mapped
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0x7F), None);
        // Last valid entry
        assert_eq!(FontInfo::gid_to_standard_glyph_name(0xFF), Some("ydieresis"));
    }

    // =========================================================================
    // glyph_name_to_unicode — AGL completeness spot checks
    // =========================================================================

    #[test]
    fn test_glyph_name_to_unicode_math_symbols() {
        assert_eq!(glyph_name_to_unicode("infinity"), Some('∞'));
        assert_eq!(glyph_name_to_unicode("notequal"), Some('≠'));
        assert_eq!(glyph_name_to_unicode("lessequal"), Some('≤'));
        assert_eq!(glyph_name_to_unicode("greaterequal"), Some('≥'));
    }

    #[test]
    fn test_glyph_name_to_unicode_german_sharp_s() {
        assert_eq!(glyph_name_to_unicode("germandbls"), Some('ß'));
    }

    #[test]
    fn test_glyph_name_to_unicode_copyright_registered() {
        assert_eq!(glyph_name_to_unicode("copyright"), Some('©'));
        assert_eq!(glyph_name_to_unicode("registered"), Some('®'));
        assert_eq!(glyph_name_to_unicode("trademark"), Some('™'));
    }

    // =========================================================================
    // char_to_unicode — Type0 Identity-H with non-Identity ordering (CJK)
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_identity_h_cjk_ordering() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("Identity-H".to_string());
            f.cid_system_info = Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Japan1".to_string(),
                supplement: 4,
            });
        });
        // Non-identity ordering with Identity-H: CIDs are NOT Unicode
        // Should use predefined CMap lookup
        // CID 843 → Hiragana あ (U+3042)
        assert_eq!(font.char_to_unicode(843), Some("\u{3042}".to_string()));
    }

    // =========================================================================
    // char_to_unicode — UCS2/UTF16 encoding variant
    // =========================================================================

    #[test]
    fn test_char_to_unicode_type0_ucs2_encoding() {
        let font = make_font(|f| {
            f.subtype = "Type0".to_string();
            f.encoding = Encoding::Standard("UniJIS-UCS2-H".to_string());
            f.cid_system_info = Some(CIDSystemInfo {
                registry: "Adobe".to_string(),
                ordering: "Identity".to_string(),
                supplement: 0,
            });
        });
        // UCS2 encoding: char_code IS the Unicode value
        assert_eq!(font.char_to_unicode(0x41), Some("A".to_string()));
    }

    // =========================================================================
    // Standard encoding control char handling
    // =========================================================================

    #[test]
    fn test_standard_encoding_winansi_control_range() {
        // Codes 0-31 are control range — WinAnsi doesn't map these
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x00), None);
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x01), None);
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x1F), None);
    }

    // =========================================================================
    // WinAnsi full extended range spot checks
    // =========================================================================

    #[test]
    fn test_standard_encoding_winansi_full_extended() {
        // 0x85 → Horizontal ellipsis (U+2026)
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x85), Some("\u{2026}".to_string()));
        // 0x99 → Trade mark sign (U+2122)
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0x99), Some("\u{2122}".to_string()));
        // 0xFF → Latin small letter y with diaeresis
        assert_eq!(standard_encoding_lookup("WinAnsiEncoding", 0xFF), Some("\u{00FF}".to_string()));
    }

    // ==========================================
    // wrap_cff_in_opentype tests
    // ==========================================

    #[test]
    fn test_wrap_cff_in_opentype_header() {
        // Minimal CFF data (version 1.0, hdrSize=4, offSize=1)
        let cff = vec![1, 0, 4, 1, 0, 0, 0, 0];
        let otf = super::wrap_cff_in_opentype(&cff);

        // Must start with 'OTTO' tag
        assert_eq!(&otf[0..4], b"OTTO");
        // numTables = 4 (CFF, head, hhea, maxp)
        assert_eq!(u16::from_be_bytes([otf[4], otf[5]]), 4);
        // Must contain the CFF data at some offset
        assert!(otf.windows(cff.len()).any(|w| w == &cff[..]));
    }

    #[test]
    fn test_wrap_cff_in_opentype_contains_required_tables() {
        let cff = vec![1, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let otf = super::wrap_cff_in_opentype(&cff);

        // Check all 4 required table tags exist in the table directory
        // Table directory starts at offset 12, each record is 16 bytes
        let mut found_tables = Vec::new();
        for i in 0..4 {
            let offset = 12 + i * 16;
            let tag = std::str::from_utf8(&otf[offset..offset + 4]).unwrap_or("????");
            found_tables.push(tag.to_string());
        }
        found_tables.sort();
        assert_eq!(found_tables, vec!["CFF ", "head", "hhea", "maxp"]);
    }

    #[test]
    fn test_wrap_cff_in_opentype_parseable() {
        // Create a minimal but valid CFF font stub
        let cff = vec![1, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let otf = super::wrap_cff_in_opentype(&cff);

        // ttf-parser should be able to parse the header (head + hhea + maxp)
        // without panicking, even if CFF data is minimal
        let result = ttf_parser::Face::parse(&otf, 0);
        // May fail on CFF content but should not panic on table parsing
        // The fact that it doesn't panic is the test
        let _ = result;
    }

    // ==========================================
    // get_standard_font_width tests
    // ==========================================

    #[test]
    fn test_standard_font_width_times_roman() {
        let font = FontInfo {
            base_font: "Times-Roman".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            widths: None, // No widths → should use standard metrics
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 500.0,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // 'A' = 722 in Times-Roman (not the default 500)
        assert_eq!(font.get_glyph_width(65), 722.0);
        // 'i' = 278 (narrow)
        assert_eq!(font.get_glyph_width(105), 278.0);
        // space = 250
        assert_eq!(font.get_glyph_width(32), 250.0);
        // 'm' = 778 (wide)
        assert_eq!(font.get_glyph_width(109), 778.0);
    }

    #[test]
    fn test_standard_font_width_courier_monospace() {
        let font = FontInfo {
            base_font: "Courier".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 500.0,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Courier is monospace — all chars 600
        assert_eq!(font.get_glyph_width(65), 600.0); // A
        assert_eq!(font.get_glyph_width(105), 600.0); // i
        assert_eq!(font.get_glyph_width(32), 600.0); // space
    }

    #[test]
    fn test_standard_font_width_not_applied_with_widths_array() {
        let font = FontInfo {
            base_font: "Times-Roman".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            widths: Some(vec![999.0]), // Has explicit widths
            first_char: Some(65),      // Starting at 'A'
            last_char: Some(65),
            font_matrix_a: 0.001,
            default_width: 500.0,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Should use explicit width (999), not standard Times width (722)
        assert_eq!(font.get_glyph_width(65), 999.0);
    }

    #[test]
    fn test_standard_font_width_not_applied_to_unknown_font() {
        let font = FontInfo {
            base_font: "MyCustomFont".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 500.0,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        // Unknown font → should fall back to default_width (500)
        assert_eq!(font.get_glyph_width(65), 500.0);
    }

    /// Pins the standard-14 fallback path in `get_byte_to_width_table`:
    /// when `widths` is `None`, the table must be populated from
    /// `get_standard_font_width` (PDF spec Appendix D metrics), not
    /// from `default_width`. Also pins the fallback-within-the-fallback
    /// for byte codes that don't appear in the standard-14 table —
    /// those still use `default_width`.
    #[test]
    fn fallback_uses_standard_14_metrics_when_widths_absent() {
        let font = FontInfo {
            base_font: "Helvetica".to_string(),
            subtype: "Type1".to_string(),
            type3_metrics_only: false,
            encoding: Encoding::Standard("WinAnsiEncoding".to_string()),
            to_unicode: None,
            font_weight: Some(400),
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info: None,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        };

        let table = font.get_byte_to_width_table();

        // Standard-14 Helvetica metrics (PDF spec Appendix D).
        assert_eq!(table[32], 278.0, "space");
        assert_eq!(table[48], 556.0, "digit '0'");
        assert_eq!(table[65], 667.0, "'A'");
        assert_eq!(table[87], 944.0, "'W'");

        // Byte codes not in the standard-14 table fall back to default_width.
        assert_eq!(table[0], 1000.0, "NUL -> default_width fallback");
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Fix A / B / C tests (§9.10.2 Priority-3 guard + control filter + OOB CID)
    // ────────────────────────────────────────────────────────────────────────────

    /// Build a minimal ToUnicode CMap stream that maps codes 0x0041–0x005A
    /// (hex 2-byte keys) to U+0041–U+005A (A–Z).
    fn make_tounicode_az() -> Vec<u8> {
        let stream = concat!(
            "/CIDInit /ProcSet findresource begin\n",
            "12 dict begin\n",
            "begincmap\n",
            "/CIDSystemInfo 3 dict dup begin\n",
            "  /Registry (Adobe) def\n",
            "  /Ordering (UCS) def\n",
            "  /Supplement 0 def\n",
            "end def\n",
            "/CMapName /Adobe-Identity-UCS def\n",
            "/CMapType 2 def\n",
            "1 begincodespacerange\n",
            "<0000> <FFFF>\n",
            "endcodespacerange\n",
            "26 beginbfchar\n",
            "<0041> <0041>\n", // A
            "<0042> <0042>\n",
            "<0043> <0043>\n",
            "<0044> <0044>\n",
            "<0045> <0045>\n",
            "<0046> <0046>\n",
            "<0047> <0047>\n",
            "<0048> <0048>\n",
            "<0049> <0049>\n",
            "<004A> <004A>\n",
            "<004B> <004B>\n",
            "<004C> <004C>\n",
            "<004D> <004D>\n",
            "<004E> <004E>\n",
            "<004F> <004F>\n",
            "<0050> <0050>\n",
            "<0051> <0051>\n",
            "<0052> <0052>\n",
            "<0053> <0053>\n",
            "<0054> <0054>\n",
            "<0055> <0055>\n",
            "<0056> <0056>\n",
            "<0057> <0057>\n",
            "<0058> <0058>\n",
            "<0059> <0059>\n",
            "<005A> <005A>\n", // Z
            "endbfchar\n",
            "endcmap\n",
            "CMapName currentdict /CMap defineresource pop\n",
            "end\n",
            "end\n",
        );
        stream.as_bytes().to_vec()
    }

    /// Build a minimal ToUnicode CMap that maps code 0x0001 to U+0007 (BEL).
    fn make_tounicode_bel() -> Vec<u8> {
        let stream = concat!(
            "/CIDInit /ProcSet findresource begin\n",
            "12 dict begin\n",
            "begincmap\n",
            "/CIDSystemInfo 3 dict dup begin\n",
            "  /Registry (Adobe) def\n",
            "  /Ordering (UCS) def\n",
            "  /Supplement 0 def\n",
            "end def\n",
            "/CMapName /Test-BEL def\n",
            "/CMapType 2 def\n",
            "1 begincodespacerange\n",
            "<0000> <FFFF>\n",
            "endcodespacerange\n",
            "1 beginbfchar\n",
            "<0001> <0007>\n", // BEL control character
            "endbfchar\n",
            "endcmap\n",
            "CMapName currentdict /CMap defineresource pop\n",
            "end\n",
            "end\n",
        );
        stream.as_bytes().to_vec()
    }

    /// Construct a minimal Type0 FontInfo with the given ToUnicode stream and CIDSystemInfo.
    fn make_type0_font(
        to_unicode_stream: Option<Vec<u8>>,
        encoding_name: &str,
        cid_system_info: Option<CIDSystemInfo>,
    ) -> FontInfo {
        FontInfo {
            base_font: "TestType0Font".to_string(),
            subtype: "Type0".to_string(),
            type3_metrics_only: false,
            // Mirror the real parser (`parse_encoding`): a `/Identity-H` or
            // `/Identity-V` encoding name resolves to `Encoding::Identity`, not
            // `Encoding::Standard("Identity-H")` — production never produces the
            // latter for an Identity name, so tests must not either (#504).
            encoding: match encoding_name {
                "Identity-H" | "Identity-V" => Encoding::Identity,
                name => Encoding::Standard(name.to_string()),
            },
            to_unicode: to_unicode_stream.map(LazyCMap::new),
            font_weight: None,
            flags: None,
            stem_v: None,
            ascent: 0.95,
            descent: -0.35,
            embedded_font_data: None,
            embedded_program_present: None,
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: None,
            cid_system_info,
            cid_font_type: None,
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: false,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: std::collections::HashMap::new(),
            wmode: 0,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        }
    }

    /// Fix A — ToUnicode present but code not covered → U+FFFD (no Priority-3 fallback).
    ///
    /// A Type0 font with Adobe-GB1 ordering, a *non-Identity* predefined-CMap
    /// encoding (`UniGB-UCS2-H` → `Encoding::Standard`), and a ToUnicode CMap
    /// covering only A–Z. The Fix-A guard is deliberately scoped to
    /// non-Identity Type0 fonts (Identity fonts map CID→Unicode directly
    /// have a valid CMap-miss fallback), so the encoding here must be a real
    /// predefined CMap — not Identity-H — for this guard to apply in
    /// production. Querying code 0x0061 (not in the ToUnicode CMap) must
    /// return U+FFFD, NOT the CJK character the Priority-3 predefined CMap
    /// lookup would otherwise produce.
    #[test]
    fn test_fix_a_tounicode_present_miss_returns_fffd_not_cjk() {
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "GB1".to_string(),
            supplement: 2,
        });
        let font = make_type0_font(Some(make_tounicode_az()), "UniGB-UCS2-H", cid_system_info);

        // Code 0x0061 ('a') is NOT in the ToUnicode CMap (which only covers A–Z).
        // The Priority-3 predefined CMap for Adobe-GB1 would map CID 97 to some
        // Latin character. With Fix A, the function must return U+FFFD instead.
        let result = font.char_to_unicode(0x0061);
        assert_eq!(
            result,
            Some("\u{FFFD}".to_string()),
            "Type0 font with ToUnicode present but missing code 0x61 must return U+FFFD, \
             not fall through to predefined CMap"
        );

        // Codes that ARE in the CMap (A–Z) must still work correctly.
        assert_eq!(font.char_to_unicode(0x0041), Some("A".to_string()));
        assert_eq!(font.char_to_unicode(0x005A), Some("Z".to_string()));
    }

    /// Fix A — ToUnicode absent, Priority-3 predefined CMap is triggered.
    ///
    /// A Type0 font with Adobe-Japan1 ordering and NO ToUnicode CMap.
    /// Querying CID 843 must return U+3042 (あ) via the predefined CMap.
    ///
    /// `Identity-H` resolves to `Encoding::Identity` (as in production);
    /// combined with a non-Identity CIDSystemInfo ordering (Japan1) and no
    /// ToUnicode CMap, the lookup routes through the predefined-CMap path
    /// (`lookup_predefined_cmap`) rather than treating the CID as a raw
    /// Unicode code point.
    #[test]
    fn test_fix_a_no_tounicode_priority3_triggered() {
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Japan1".to_string(),
            supplement: 4,
        });
        // Identity-H with a non-Identity (Japan1) ordering: CIDs route through
        // lookup_predefined_cmap, NOT treated as raw Unicode code points.
        let font = make_type0_font(None, "Identity-H", cid_system_info);

        // CID 843 maps to U+3042 (あ) per the Adobe-Japan1 collection.
        let result = font.char_to_unicode(843);
        assert_eq!(
            result,
            Some("\u{3042}".to_string()),
            "Type0 font without ToUnicode must use predefined CMap for CID 843 → U+3042"
        );
    }

    /// Fix C — OOB CID guard: CID well beyond the Adobe-GB1 maximum → None.
    ///
    /// lookup_predefined_cmap with an OOB CID must return None without panicking.
    #[test]
    fn test_fix_c_oob_cid_returns_none() {
        let cid_system_info = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "GB1".to_string(),
            supplement: 2,
        });
        // CID 99_999 is far beyond CID_MAX_GB1 (30_283).
        // The function takes u16, so we use the max u16 value (65535) which still
        // exceeds CID_MAX_GB1.
        let result = lookup_predefined_cmap("UniGB-UCS2-H", &cid_system_info, 65535);
        assert_eq!(result, None, "OOB CID (65535 > CID_MAX_GB1 30283) must return None");

        // Same for Japan1.
        let cid_japan = Some(CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Japan1".to_string(),
            supplement: 4,
        });
        let result_j = lookup_predefined_cmap("UniJIS-UCS2-H", &cid_japan, 65535);
        assert_eq!(result_j, None, "OOB CID (65535 > CID_MAX_JAPAN1 23059) must return None");
    }

    /// Fix B — C0 control character filter: ToUnicode mapping to U+0007 (BEL) → U+FFFD.
    ///
    /// A ToUnicode CMap that explicitly maps code 0x0001 to U+0007 (BEL).
    /// The function must return U+FFFD, not the BEL character.
    #[test]
    fn test_fix_b_control_char_filter_returns_fffd() {
        let font = make_type0_font(Some(make_tounicode_bel()), "Identity-H", None);

        // Code 0x0001 maps to U+0007 (BEL) in the ToUnicode CMap.
        // Fix B must intercept this and return U+FFFD.
        let result = font.char_to_unicode(0x0001);
        assert_eq!(
            result,
            Some("\u{FFFD}".to_string()),
            "Code mapping to U+0007 (BEL) must be filtered to U+FFFD by Fix B"
        );
    }

    /// A ToUnicode CMap that maps only code 0x0041 → U+005A ('Z'); every other
    /// code is absent.
    fn make_tounicode_single_z() -> Vec<u8> {
        concat!(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n",
            "/CIDSystemInfo 3 dict dup begin\n",
            "  /Registry (Adobe) def\n  /Ordering (UCS) def\n  /Supplement 0 def\nend def\n",
            "/CMapName /Test-Z def\n/CMapType 2 def\n",
            "1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
            "1 beginbfchar\n<0041> <005A>\nendbfchar\n",
            "endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n",
        )
        .as_bytes()
        .to_vec()
    }

    /// A structurally-valid `/ToUnicode` CMap with zero `bfchar`/`bfrange` entries:
    /// present but maps nothing. Must count as *absent*.
    fn make_tounicode_empty() -> Vec<u8> {
        concat!(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n",
            "/CIDSystemInfo 3 dict dup begin\n",
            "  /Registry (Adobe) def\n  /Ordering (UCS) def\n  /Supplement 0 def\nend def\n",
            "/CMapName /Test-Empty def\n/CMapType 2 def\n",
            "1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
            "endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n",
        )
        .as_bytes()
        .to_vec()
    }

    /// With a present-but-incomplete `/ToUnicode` on an Identity-H Type0 font, a
    /// drawn CID absent from it has no Unicode anywhere in the file, so it must
    /// decode to U+FFFD rather than a numeric *guess* — the CID read as a code
    /// point, or the GID via the standard glyph-name table → AGL. Both guess
    /// paths are exercised: 0x0100 (CID-as-char) and 0x003A (gid 0x3A = "colon");
    /// `cid_to_gid_map` is set so the gid→glyph-name path is actually reachable.
    #[test]
    fn test_type0_tounicode_gap_returns_fffd_not_guess() {
        let mut font = make_type0_font(Some(make_tounicode_single_z()), "Identity-H", None);
        font.cid_to_gid_map = Some(CIDToGIDMap::Identity);

        // Mapped code decodes via ToUnicode (proves the CMap is authoritative).
        assert_eq!(font.char_to_unicode(0x0041), Some("Z".to_string()));
        // Uncovered CIDs are unmapped, not guessed.
        assert_eq!(
            font.char_to_unicode(0x0100),
            Some("\u{FFFD}".to_string()),
            "uncovered CID must not be guessed as CID-as-Unicode"
        );
        assert_eq!(
            font.char_to_unicode(0x003A),
            Some("\u{FFFD}".to_string()),
            "uncovered CID must not be guessed via gid→glyph-name→AGL"
        );
    }

    /// Without a `/ToUnicode`, the CID-as-Unicode heuristic still applies — many
    /// generators assign CID == Unicode — so this path must not regress to U+FFFD.
    #[test]
    fn test_type0_no_tounicode_keeps_cid_as_unicode() {
        let mut font = make_type0_font(None, "Identity-H", None);
        font.cid_to_gid_map = Some(CIDToGIDMap::Identity);
        assert_eq!(font.char_to_unicode(0x0100), Some("\u{0100}".to_string()));
    }

    /// For an Identity-ordered font with a present-but-incomplete `/ToUnicode`, an
    /// uncovered CID decodes to U+FFFD (honest gap) rather than the CID-as-Unicode guess —
    /// except whitespace (0x20 → space), which is retained so word boundaries survive.
    #[test]
    fn test_type0_identity_uncovered_cid_is_fffd_keeps_space() {
        let csi = CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Identity".to_string(),
            supplement: 0,
        };
        let mut font = make_type0_font(Some(make_tounicode_single_z()), "Identity-H", Some(csi));
        font.cid_to_gid_map = Some(CIDToGIDMap::Identity);

        // Mapped code still resolves via ToUnicode.
        assert_eq!(font.char_to_unicode(0x0041), Some("Z".to_string()), "ToUnicode hit");
        // Whitespace is retained even when uncovered (word boundaries survive).
        assert_eq!(font.char_to_unicode(0x0020), Some(" ".to_string()), "space retained");
        // Any other uncovered CID → U+FFFD, not a CID-as-Unicode guess.
        assert_eq!(
            font.char_to_unicode(0x0043),
            Some("\u{FFFD}".to_string()),
            "uncovered non-space Identity CID must be U+FFFD"
        );
    }

    /// A present-but-*empty* `/ToUnicode` (0 bfchar/bfrange) maps nothing, so it must count
    /// as absent and an Identity-ordered font must recover its text via CID-as-Unicode. The
    /// `CIDToGIDMap` here remaps each letter to a low *punctuation* GID, so the GID→standard-
    /// glyph-name→AGL guess (if it ran) would yield `J)'(i#`; CID-as-Unicode must win instead.
    /// This is the faithful subset case the `CIDToGIDMap::Identity` variant can't reproduce.
    #[test]
    fn test_type0_identity_empty_tounicode_keeps_cid_as_unicode() {
        let csi = CIDSystemInfo {
            registry: "Adobe".to_string(),
            ordering: "Identity".to_string(),
            supplement: 1,
        };
        let mut font = make_type0_font(Some(make_tounicode_empty()), "Identity-H", Some(csi));

        // Each letter CID remaps to a punctuation GID (0x21..=0x26 → exclam/quotedbl/…),
        // which gid_to_standard_glyph_name → AGL would otherwise turn into a wrong char.
        let letters = [
            (0x004A, "J"),
            (0x0075, "u"),
            (0x0073, "s"),
            (0x0074, "t"),
            (0x0069, "i"),
            (0x006E, "n"),
        ];
        let mut gid_map = vec![0u16; 0x80];
        for (i, (cid, _)) in letters.iter().enumerate() {
            gid_map[*cid as usize] = 0x21 + i as u16;
        }
        font.cid_to_gid_map = Some(CIDToGIDMap::Explicit(gid_map));

        for (cid, ch) in letters {
            assert_eq!(
                font.char_to_unicode(cid),
                Some(ch.to_string()),
                "empty /ToUnicode + Identity ordering must use CID-as-Unicode for 0x{cid:04X}, \
                 not the GID→glyph-name guess"
            );
        }
    }

    /// #504: `make_type0_font` must mirror the real `parse_encoding`
    /// mapping. A direct guard so a future revert of the helper is caught
    /// tightly (the Fix-A/B tests above only assert it *indirectly* via
    /// `char_to_unicode` outcomes).
    #[test]
    fn test_make_type0_font_encoding_matches_parser() {
        assert!(
            matches!(make_type0_font(None, "Identity-H", None).encoding, Encoding::Identity),
            "Identity-H must map to Encoding::Identity (production never yields Standard(\"Identity-H\"))"
        );
        assert!(
            matches!(make_type0_font(None, "Identity-V", None).encoding, Encoding::Identity),
            "Identity-V must map to Encoding::Identity"
        );
        match make_type0_font(None, "WinAnsiEncoding", None).encoding {
            Encoding::Standard(ref n) => assert_eq!(n, "WinAnsiEncoding"),
            other => panic!("non-Identity name must stay Encoding::Standard, got {other:?}"),
        }
        match make_type0_font(None, "UniGB-UCS2-H", None).encoding {
            Encoding::Standard(ref n) => assert_eq!(n, "UniGB-UCS2-H"),
            other => panic!("predefined CMap name must be Encoding::Standard, got {other:?}"),
        }
    }

    /// Type0/CID fonts read ascent/descent from the CIDFont descendant's FontDescriptor
    /// (§9.7.4 / Table 117), not from the Type0 wrapper (which has no top-level
    /// /FontDescriptor). Verify that `FontInfo::from_dict` on a Type0 font with a
    /// descendant FontDescriptor that carries Ascent=800 / Descent=-200 yields
    /// ascent ≈ 0.8 and descent ≈ -0.2 (both normalised from 1/1000-em to fraction-of-em).
    #[test]
    fn test_type0_ascent_descent_from_descendant_descriptor() {
        // Build an inline CIDFont dictionary with a FontDescriptor containing Ascent/Descent.
        let mut desc: HashMap<String, Object> = HashMap::new();
        desc.insert("Type".to_string(), Object::Name("FontDescriptor".to_string()));
        desc.insert("Ascent".to_string(), Object::Integer(800));
        desc.insert("Descent".to_string(), Object::Integer(-200));

        // Build the CIDFont dictionary (inline, no object references needed).
        let mut cidfont: HashMap<String, Object> = HashMap::new();
        cidfont.insert("Type".to_string(), Object::Name("Font".to_string()));
        cidfont.insert("Subtype".to_string(), Object::Name("CIDFontType0".to_string()));
        cidfont.insert("BaseFont".to_string(), Object::Name("TestCIDFont".to_string()));
        cidfont.insert("DW".to_string(), Object::Integer(1000));
        cidfont.insert(
            "CIDSystemInfo".to_string(),
            Object::Dictionary({
                let mut si = HashMap::new();
                si.insert("Registry".to_string(), Object::String(b"Adobe".to_vec()));
                si.insert("Ordering".to_string(), Object::String(b"Identity".to_vec()));
                si.insert("Supplement".to_string(), Object::Integer(0));
                si
            }),
        );
        cidfont.insert("FontDescriptor".to_string(), Object::Dictionary(desc));

        // Wrap the CIDFont in the Type0 outer font dictionary.
        let mut type0: HashMap<String, Object> = HashMap::new();
        type0.insert("Type".to_string(), Object::Name("Font".to_string()));
        type0.insert("Subtype".to_string(), Object::Name("Type0".to_string()));
        type0.insert("BaseFont".to_string(), Object::Name("TestType0Font".to_string()));
        type0.insert("Encoding".to_string(), Object::Name("Identity-H".to_string()));
        type0.insert(
            "DescendantFonts".to_string(),
            Object::Array(vec![Object::Dictionary(cidfont)]),
        );

        let doc = minimal_pdf_doc();
        let font = FontInfo::from_dict(&Object::Dictionary(type0), &doc)
            .expect("Type0 font with inline descendant must parse");

        assert!(
            (font.ascent - 0.8).abs() < 1e-4,
            "Expected ascent ≈ 0.8 (800/1000), got {}",
            font.ascent
        );
        assert!(
            (font.descent - (-0.2)).abs() < 1e-4,
            "Expected descent ≈ -0.2 (-200/1000), got {}",
            font.descent
        );
    }

    // =========================================================================
    // Item 1 (v0.3.58) — decimal-point / punctuation glyph recovery.
    // =========================================================================

    /// A minimal in-memory PDF so `parse_encoding` (which takes `&PdfDocument`)
    /// can run in a unit test. The encoding dict and /Differences array below
    /// use only inline objects, so the document is never actually dereferenced.
    fn minimal_pdf_doc() -> crate::document::PdfDocument {
        let pdf = b"%PDF-1.4\n\
            1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
            2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
            3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
            xref\n\
            0 4\n\
            0000000000 65535 f \n\
            0000000009 00000 n \n\
            0000000058 00000 n \n\
            0000000115 00000 n \n\
            trailer\n<< /Size 4 /Root 1 0 R >>\n\
            startxref\n197\n%%EOF\n";
        crate::document::PdfDocument::from_bytes(pdf.to_vec()).expect("minimal PDF should parse")
    }

    /// A synthetic CMMI-like encoding dict that parks `/period` at code 58.
    fn cmmi_like_encoding_obj() -> Object {
        let mut enc: HashMap<String, Object> = HashMap::new();
        enc.insert(
            "Differences".to_string(),
            Object::Array(vec![
                Object::Integer(44),
                Object::Name("arrowhookleft".to_string()),
                Object::Integer(58),
                Object::Name("period".to_string()),
            ]),
        );
        Object::Dictionary(enc)
    }

    /// Task 1 verify: the /Differences glyph name survives parse time.
    #[test]
    fn test_diff_glyph_names_retains_period_for_code_58() {
        let doc = minimal_pdf_doc();
        let (_enc, _multi, diff_names) =
            FontInfo::parse_encoding(&cmmi_like_encoding_obj(), &doc, None).unwrap();
        assert_eq!(diff_names.get(&58).map(String::as_str), Some("period"));
        assert_eq!(diff_names.get(&44).map(String::as_str), Some("arrowhookleft"));
    }

    /// Task 2 verify: the closed-set AGL punctuation helper.
    #[test]
    fn test_punctuation_unicode_for_glyph_name_closed_set() {
        assert_eq!(punctuation_unicode_for_glyph_name("period"), Some("."));
        assert_eq!(punctuation_unicode_for_glyph_name("comma"), Some(","));
        assert_eq!(punctuation_unicode_for_glyph_name("hyphen"), Some("-"));
        assert_eq!(punctuation_unicode_for_glyph_name("minus"), Some("\u{2212}"));
        // Anything outside the closed set → None (no over-generalisation).
        assert_eq!(punctuation_unicode_for_glyph_name("colon"), None);
        assert_eq!(punctuation_unicode_for_glyph_name("logicalnot"), None);
        assert_eq!(punctuation_unicode_for_glyph_name("A"), None);
    }

    /// Task 2 verify: the non-sensible-symbol predicate.
    #[test]
    fn test_is_non_sensible_symbol() {
        // Wrong symbols that should trigger recovery.
        assert!(is_non_sensible_symbol("\u{00AC}")); // ¬ logicalnot
        assert!(is_non_sensible_symbol("\u{2192}")); // → rightwards arrow
        assert!(is_non_sensible_symbol("\u{2212}")); // − minus sign (math operator)
                                                     // Sensible punctuation / digits / letters → false.
        assert!(!is_non_sensible_symbol("."));
        assert!(!is_non_sensible_symbol(","));
        assert!(!is_non_sensible_symbol("-"));
        assert!(!is_non_sensible_symbol("5"));
        assert!(!is_non_sensible_symbol("A"));
        // Empty / multi-char never qualifies.
        assert!(!is_non_sensible_symbol(""));
        assert!(!is_non_sensible_symbol("ff"));
    }

    /// Build a CMMI-like simple font with the given ToUnicode CMap bytes and a
    /// `/Differences 58 /period` side map (and matching Custom encoding entry).
    fn cmmi_like_font(to_unicode: Option<&[u8]>, custom_char_for_58: char) -> FontInfo {
        let mut diff_glyph_names: HashMap<u8, String> = HashMap::new();
        diff_glyph_names.insert(58, "period".to_string());
        let mut custom_map: HashMap<u8, char> = HashMap::new();
        custom_map.insert(58, custom_char_for_58);
        make_font(|f| {
            f.base_font = "SQLQIW+CMMI10".to_string();
            f.subtype = "Type1".to_string();
            f.flags = Some(4); // symbolic
            f.encoding = Encoding::Custom(custom_map);
            f.to_unicode = to_unicode.map(|b| LazyCMap::new(b.to_vec()));
            f.diff_glyph_names = diff_glyph_names;
        })
    }

    /// Task 3 verify (Interception A): a non-sensible ToUnicode hit (U+00AC) for
    /// a `/period`-named code is recovered to `.`.
    #[test]
    fn test_interception_a_tounicode_non_sensible_symbol_recovered() {
        // ToUnicode maps 0x3A → U+00AC (¬), a non-sensible symbol for /period.
        let cmap = b"beginbfchar\n<003A> <00AC>\nendbfchar";
        let font = cmmi_like_font(Some(cmap), '.');
        assert_eq!(font.char_to_unicode(0x3A), Some(".".to_string()));
    }

    /// Task 4 verify (Interception B): no ToUnicode, Custom encoding resolves 58
    /// to a wrong symbol, but the /Differences /period name overrides to `.`.
    #[test]
    fn test_interception_b_custom_encoding_punctuation_override() {
        // Custom map deliberately resolves code 58 to ¬ (the wrong symbol);
        // diff_glyph_names[58] = "period" must win.
        let font = cmmi_like_font(None, '\u{00AC}');
        assert_eq!(font.char_to_unicode(0x3A), Some(".".to_string()));
    }

    /// Task 5 regression guard: correctly-mapped fonts and genuine symbols are
    /// untouched by the punctuation-recovery interceptions.
    #[test]
    fn test_punctuation_recovery_regression_guard() {
        // (a) A correctly-mapped period via ToUnicode (0x2E → U+002E) with no
        //     special glyph name stays `.` — the hit is already sensible so
        //     Interception A never fires.
        let cmap_ok = b"beginbfchar\n<002E> <002E>\nendbfchar";
        let font_ok = make_font(|f| {
            f.to_unicode = Some(LazyCMap::new(cmap_ok.to_vec()));
        });
        assert_eq!(font_ok.char_to_unicode(0x2E), Some(".".to_string()));

        // (b) A genuine `logicalnot` glyph (¬) must stay ¬: its /Differences
        //     name is NOT in the punctuation closed set, so neither
        //     interception fires even though the resolved char is a symbol.
        let cmap_not = b"beginbfchar\n<0021> <00AC>\nendbfchar";
        let mut diff_glyph_names: HashMap<u8, String> = HashMap::new();
        diff_glyph_names.insert(0x21, "logicalnot".to_string());
        let font_not = make_font(|f| {
            f.base_font = "NSCCOE+txexs".to_string();
            f.flags = Some(4);
            f.to_unicode = Some(LazyCMap::new(cmap_not.to_vec()));
            f.diff_glyph_names = diff_glyph_names;
        });
        assert_eq!(font_not.char_to_unicode(0x21), Some("\u{00AC}".to_string()));
    }

    /// Build a synthetic single-page PDF whose object 4 is a Type0 font with
    /// the given base name, /Encoding name, descendant /CIDSystemInfo
    /// Ordering, and (optionally) an extra raw entry spliced into the
    /// descendant's FontDescriptor (e.g. `/FontFile3 99 0 R` pointing at a
    /// non-existent object to model a present-but-unextractable program).
    fn build_predefined_cidfont_pdf(
        base_font: &str,
        encoding_name: &str,
        ordering: &str,
        descriptor_extra: &str,
    ) -> Vec<u8> {
        let mut pdf = Vec::new();
        pdf.extend_from_slice(b"%PDF-1.4\n");

        let o1 = pdf.len();
        pdf.extend_from_slice(b"1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n");
        let o2 = pdf.len();
        pdf.extend_from_slice(b"2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n");
        let o3 = pdf.len();
        pdf.extend_from_slice(
            b"3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
              /Resources << /Font << /F1 4 0 R >> >> >> endobj\n",
        );
        let o4 = pdf.len();
        pdf.extend_from_slice(
            format!(
                "4 0 obj << /Type /Font /Subtype /Type0 /BaseFont /{base_font} \
                 /Encoding /{encoding_name} /DescendantFonts [5 0 R] >> endobj\n"
            )
            .as_bytes(),
        );
        let o5 = pdf.len();
        pdf.extend_from_slice(
            format!(
                "5 0 obj << /Type /Font /Subtype /CIDFontType0 /BaseFont /{base_font} \
                 /CIDSystemInfo << /Registry (Adobe) /Ordering ({ordering}) /Supplement 6 >> \
                 /FontDescriptor 6 0 R /DW 1000 >> endobj\n"
            )
            .as_bytes(),
        );
        let o6 = pdf.len();
        pdf.extend_from_slice(
            format!(
                "6 0 obj << /Type /FontDescriptor /FontName /{base_font} /Flags 6 \
                 /FontBBox [-170 -331 1024 903] /ItalicAngle 0 /Ascent 723 \
                 /Descent -241 /CapHeight 709 /StemV 69 {descriptor_extra} >> endobj\n"
            )
            .as_bytes(),
        );

        let xref = pdf.len();
        pdf.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
        for off in [o1, o2, o3, o4, o5, o6] {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        pdf.extend_from_slice(
            format!("trailer << /Size 7 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", xref).as_bytes(),
        );
        pdf
    }

    /// Parse object 4 of a [`build_predefined_cidfont_pdf`] document through
    /// the real `FontInfo::from_dict` path and return the resulting FontInfo.
    fn parse_predefined_cidfont(
        base_font: &str,
        encoding_name: &str,
        ordering: &str,
        descriptor_extra: &str,
    ) -> FontInfo {
        let pdf =
            build_predefined_cidfont_pdf(base_font, encoding_name, ordering, descriptor_extra);
        let doc = crate::document::PdfDocument::from_bytes(pdf).expect("synthetic PDF must parse");
        let font_obj = doc
            .load_object(crate::object::ObjectRef::new(4, 0))
            .expect("load Type0 font dict");
        FontInfo::from_dict(&font_obj, &doc).expect("FontInfo::from_dict")
    }

    /// Control: an Identity-H predefined name with no font program is flagged
    /// for substitution under the collection derived from the name.
    #[test]
    fn cjk_substitution_flags_identity_h_predefined_name() {
        let info = parse_predefined_cidfont("Ryumin-Light", "Identity-H", "Japan1", "");
        assert_eq!(
            info.cjk_substitution,
            Some(super::super::predefined_cidfont::CharacterCollection::AdobeJapan1)
        );
    }

    /// A descriptor that *declares* a font program (here a /FontFile3 whose
    /// target object doesn't exist, so extraction fails) must NOT be
    /// substituted: the document intended to embed outlines and the decode
    /// failure should surface as a warning, not be masked by a silent
    /// sans-serif substitution.
    #[test]
    fn cjk_substitution_declined_when_font_program_key_present_but_unextractable() {
        let info =
            parse_predefined_cidfont("Ryumin-Light", "Identity-H", "Japan1", "/FontFile3 99 0 R");
        assert!(info.embedded_font_data.is_none(), "extraction must have failed");
        assert_eq!(
            info.cjk_substitution, None,
            "substitution must not mask a failed embedded-font decode"
        );
    }

    /// Non-Identity predefined CMaps (90ms-RKSJ-H, GBK-EUC-H, …) carry raw
    /// legacy multi-byte codes, not CIDs. Until a charcode→CID CMap pass is
    /// wired, such fonts must not be substituted — interpreting a Shift-JIS
    /// code as an Adobe-Japan1 CID paints wrong glyphs with no diagnostic.
    #[test]
    fn cjk_substitution_requires_identity_cmap_encoding() {
        let info =
            parse_predefined_cidfont("Ryumin-Light-90ms-RKSJ-H", "90ms-RKSJ-H", "Japan1", "");
        assert_eq!(
            info.cjk_substitution, None,
            "non-Identity CMap codes are not CIDs; substitution must decline"
        );
    }

    /// When the descendant's /CIDSystemInfo names a known collection that
    /// disagrees with the one derived from the base-font name, the explicit
    /// CIDSystemInfo wins — it is authoritative for CID semantics per
    /// ISO 32000-1 §9.7.3.
    #[test]
    fn cjk_substitution_prefers_cid_system_info_ordering_over_name() {
        let info = parse_predefined_cidfont("Ryumin-Light", "Identity-H", "GB1", "");
        assert_eq!(
            info.cjk_substitution,
            Some(super::super::predefined_cidfont::CharacterCollection::AdobeGB1),
            "explicit /CIDSystemInfo Ordering must override the name-derived collection"
        );
    }

    /// An Identity (or unknown) Ordering carries no collection semantics; the
    /// name-derived collection remains the best available signal.
    #[test]
    fn cjk_substitution_keeps_name_collection_for_identity_ordering() {
        let info = parse_predefined_cidfont("Ryumin-Light", "Identity-H", "Identity", "");
        assert_eq!(
            info.cjk_substitution,
            Some(super::super::predefined_cidfont::CharacterCollection::AdobeJapan1)
        );
    }
}
