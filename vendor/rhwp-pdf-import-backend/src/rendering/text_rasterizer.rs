//! Text rasterizer - renders PDF text using tiny-skia.
//!
//! Text rendering in PDF is complex because:
//! - Fonts may be embedded or use standard PDF fonts
//! - Character encoding varies (identity-H, MacRoman, custom ToUnicode, etc.)
#![allow(clippy::collapsible_if, clippy::vec_box)]
//! - Glyph positioning is explicit via TJ arrays
//!
//! This module provides a text rendering implementation that:
//! - Uses system fonts as fallback when embedded fonts aren't available
//! - Renders text using harfrust for shaping and tiny-skia for drawing glyph paths

use super::{create_fill_paint, guarded_fill_path};
use crate::content::operators::TextElement;
use crate::content::GraphicsState;
use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::fonts::unicode_decode::{
    char_codes_with_widths, get_byte_mode, ByteMode, DecodePolicy, TextCharIter,
};
use crate::object::Object;
use std::collections::HashMap;
use std::sync::Arc;

use tiny_skia::{Paint, PathBuilder, Pixmap, Transform};
use ttf_parser::OutlineBuilder;

/// Outline builder that converts ttf-parser paths to tiny-skia paths.
struct SkiaOutlineBuilder<'a>(&'a mut PathBuilder);

impl<'a> OutlineBuilder for SkiaOutlineBuilder<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.0.close();
    }
}

/// Classify an embedded font's cmap tables in a single parse pass.
///
/// Returns `(is_byte_indexed_only, has_unicode_cmap)`:
/// - `is_byte_indexed_only`: only a Macintosh byte-indexed cmap present →
///   use `render_cid_direct` rather than Unicode shaping.
/// - `has_unicode_cmap`: a Unicode/Windows cmap is present → Unicode shaping
///   is likely to produce non-.notdef glyphs; use `render_unicode_text`.
///
/// This is a zero-copy `ttf_parser` table probe (no glyph parsing, no
/// shaping), cheap enough to run per call. It was previously memoised in a
/// process-wide `HashMap` keyed on `Arc::as_ptr(data)`, but that key is
/// unsound under concurrency: when an `Arc<Vec<u8>>` font buffer is dropped
/// (font-cache eviction / per-page renderer reset) and the allocator
/// recycles its address for an unrelated font, the stale entry was returned,
/// flipping the render branch and surfacing as an intermittent
/// `ParseException [1000]` under concurrent rendering (issue #505).
/// Computing it locally removes the shared mutable state entirely.
fn classify_embedded_font(data: &Arc<Vec<u8>>) -> (bool, bool) {
    (|| {
        let face = ttf_parser::Face::parse(data, 0).ok()?;
        let cmap = face.tables().cmap?;
        let mut saw_byte_indexed = false;
        let mut saw_unicode = false;
        for sub in cmap.subtables {
            use ttf_parser::PlatformId;
            match sub.platform_id {
                PlatformId::Unicode => saw_unicode = true,
                PlatformId::Windows if sub.encoding_id == 1 || sub.encoding_id == 10 => {
                    saw_unicode = true;
                },
                PlatformId::Macintosh if sub.encoding_id == 0 => saw_byte_indexed = true,
                _ => {},
            }
        }
        Some((saw_byte_indexed && !saw_unicode, saw_unicode))
    })()
    .unwrap_or((false, false))
}

/// Resolve a single PDF content byte to a GID by consulting the font's
/// own cmap subtables. Prefers a byte-indexed (Macintosh Roman) subtable
/// when present; falls back to ttf-parser's default Unicode resolution
/// for ASCII-range bytes if no byte-indexed subtable exists.
fn cmap_byte_to_gid(face: &ttf_parser::Face, byte: u8) -> Option<u16> {
    if let Some(cmap) = face.tables().cmap {
        for sub in cmap.subtables {
            use ttf_parser::PlatformId;
            if matches!(sub.platform_id, PlatformId::Macintosh) && sub.encoding_id == 0 {
                if let Some(gid) = sub.glyph_index(byte as u32) {
                    return Some(gid.0);
                }
            }
        }
    }
    face.glyph_index(byte as char).map(|g| g.0)
}

/// Process-wide cache for the system font database.
///
/// `fontdb::Database::load_system_fonts()` walks every font directory on
/// the host and parses each face it finds, which typically takes several
/// seconds on first call. Before this cache was introduced, every
/// `TextRasterizer::new()` (and therefore every `PageRenderer::new()`)
/// paid that cost, and callers who constructed a fresh `PageRenderer`
/// per page — which is the obvious first-draft usage from the Python /
/// CLI surface — hit the scan once per page. A cold-cache ORAFOL 5400
/// render took ~4.1 s on a warm machine for a single page because of
/// this. See issue #331.
///
/// Switching to a process-wide `OnceLock<Arc<fontdb::Database>>` loads
/// the database exactly once per process, and every subsequent
/// `TextRasterizer` constructor takes a cheap `Arc::clone`. Wrapping
/// in `Arc` is important so that the cache is still cheaply shareable
/// across `TextRasterizer` instances in different rendering contexts
/// without re-copying the full parsed font metadata. Callers that want
/// a private / modified database can still construct one by hand and
/// bypass this cache via `TextRasterizer::with_fontdb()`.
static SYSTEM_FONTDB: std::sync::OnceLock<std::sync::Arc<fontdb::Database>> =
    std::sync::OnceLock::new();

fn system_fontdb() -> std::sync::Arc<fontdb::Database> {
    SYSTEM_FONTDB
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            // Guarantee a CJK-covering face exists no matter which fonts the
            // host has installed. Registered under its real family name
            // ("Droid Sans Fallback"), so the existing fallback resolver only
            // reaches it as a last resort — after any system CJK font (Noto
            // Sans/Serif CJK, SimSun, …). Without this, a composite (Type 0)
            // font that references a glyph collection but embeds no outlines
            // renders blank on CJK-fontless hosts. ISO 32000-2 §9.7.5.2: a
            // processor shall support the Adobe predefined character
            // collections even when the PDF embeds no outlines for them.
            #[cfg(feature = "cjk-render-fallback")]
            db.load_font_data(
                crate::fonts::form_fallback::font_bytes(crate::fonts::form_fallback::Fallback::Cjk)
                    .to_vec(),
            );
            std::sync::Arc::new(db)
        })
        .clone()
}

/// Process-wide cache mapping fontdb::ID → (font bytes, face index).
///
/// Without this cache, `load_font_data` calls `with_face_data(...to_vec())`
/// which clones the entire font binary (often 300–500 KB for Liberation Serif
/// or Times New Roman) on every `render_text` call. A two-page text PDF can
/// trigger hundreds of such clones per render pass. This cache reduces each
/// subsequent access to a cheap `Arc::clone`.
static FONT_BYTES_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<fontdb::ID, (Arc<Vec<u8>>, u32)>>,
> = std::sync::OnceLock::new();

fn cached_font_bytes(id: fontdb::ID, db: &fontdb::Database) -> Option<(Arc<Vec<u8>>, u32)> {
    let cache =
        FONT_BYTES_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    {
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get(&id) {
            return Some(entry.clone());
        }
    }
    let mut result: Option<(Arc<Vec<u8>>, u32)> = None;
    db.with_face_data(id, |data, index| {
        result = Some((Arc::new(data.to_vec()), index));
    });
    if let Some(ref entry) = result {
        let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        guard.insert(id, entry.clone());
    }
    result
}

/// Parsed font faces cached by fontdb ID.
///
/// harfrust's `FontRef` and `ttf_parser::Face` both borrow the backing bytes.
/// We use a self-referential pattern (backed Arc keeps bytes alive) with
/// unsafe 'static transmute so we can store them in a process-wide cache
/// and reuse them across hundreds of render_text calls for the same font.
/// `ShaperData` caches the shaping tables derived from the font (owned, no
/// borrow) so each render only has to build a cheap `Shaper` from it.
///
/// # Safety
/// The `font` and `ttf_face` fields borrow `_data`'s heap allocation (not the
/// Arc pointer itself, so no double-free on Arc drop). They are only ever
/// accessed while `_data` is alive — i.e., while this struct exists.
/// Because the struct is behind `Arc`, it lives at least as long as any
/// caller that holds a clone of that Arc.
struct CachedFace {
    _data: Arc<Vec<u8>>,
    font: harfrust::FontRef<'static>,
    shaper_data: harfrust::ShaperData,
    ttf_face: ttf_parser::Face<'static>,
    pub units_per_em: f32,
}

// SAFETY: harfrust::FontRef and ttf_parser::Face only borrow immutable bytes;
// harfrust::ShaperData owns the cached tables it derives from the font.
unsafe impl Send for CachedFace {}
unsafe impl Sync for CachedFace {}

impl CachedFace {
    fn new(data: Arc<Vec<u8>>, index: u32) -> Option<Self> {
        let font: harfrust::FontRef<'_> = harfrust::FontRef::from_index(&data, index).ok()?;
        let ttf_face: ttf_parser::Face<'_> = ttf_parser::Face::parse(&data, index).ok()?;
        let units_per_em = ttf_face.units_per_em() as f32;
        // SAFETY: both the font and ttf_face borrow the data slice. We store an
        // Arc to that data in `_data`, ensuring the bytes stay alive for this
        // struct's lifetime.
        let font: harfrust::FontRef<'static> = unsafe { std::mem::transmute(font) };
        let ttf_face: ttf_parser::Face<'static> = unsafe { std::mem::transmute(ttf_face) };
        // ShaperData owns its derived tables (no borrow of `font`), so it is
        // safe to build from the now-'static font and store it alongside.
        let shaper_data = harfrust::ShaperData::new(&font);
        Some(CachedFace {
            _data: data,
            font,
            shaper_data,
            ttf_face,
            units_per_em,
        })
    }
}

static FACE_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<(fontdb::ID, u32), Arc<CachedFace>>>,
> = std::sync::OnceLock::new();

fn cached_face(id: fontdb::ID, data: Arc<Vec<u8>>, index: u32) -> Option<Arc<CachedFace>> {
    let cache = FACE_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    {
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get(&(id, index)) {
            return Some(entry.clone());
        }
    }
    let face = CachedFace::new(data, index)?;
    let arc = Arc::new(face);
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    guard.insert((id, index), arc.clone());
    Some(arc)
}

/// Process-wide CJK fallback font — loaded once per process, shared by all
/// TextRasterizer instances.
///
/// Before this cache, every glyph that fell through to the CJK path called
/// `load_cjk_fallback()`, which iterated 7+ fontdb queries and cloned a
/// 10–20 MB Noto CJK binary. Now that work is done exactly once.
static CJK_FALLBACK: std::sync::OnceLock<Option<(fontdb::ID, Arc<Vec<u8>>, u32)>> =
    std::sync::OnceLock::new();

fn get_cjk_fallback_cached(db: &fontdb::Database) -> Option<(fontdb::ID, Arc<Vec<u8>>, u32)> {
    CJK_FALLBACK
        .get_or_init(|| {
            let prioritized_variants = [
                "Noto Sans CJK SC",
                "Noto Serif CJK SC",
                "Droid Sans Fallback",
                "SimSun",
                "WenQuanYi Micro Hei",
                "Noto Sans CJK JP",
                "Noto Serif CJK JP",
            ];
            for variant in prioritized_variants {
                let query = fontdb::Query {
                    families: &[fontdb::Family::Name(variant)],
                    weight: fontdb::Weight::NORMAL,
                    stretch: fontdb::Stretch::Normal,
                    style: fontdb::Style::Normal,
                };
                if let Some(id) = db.query(&query) {
                    if let Some((arc, idx)) = cached_font_bytes(id, db) {
                        log::debug!(
                            "CJK fallback: matched '{}', idx={}, size={} bytes",
                            variant,
                            idx,
                            arc.len()
                        );
                        return Some((id, arc, idx));
                    }
                }
            }
            let query = fontdb::Query {
                families: &[fontdb::Family::SansSerif],
                weight: fontdb::Weight::NORMAL,
                stretch: fontdb::Stretch::Normal,
                style: fontdb::Style::Normal,
            };
            if let Some(id) = db.query(&query) {
                if let Some((arc, idx)) = cached_font_bytes(id, db) {
                    return Some((id, arc, idx));
                }
            }
            None
        })
        .as_ref()
        .map(|(id, arc, idx)| (*id, Arc::clone(arc), *idx))
}

/// Process-wide cache for the bundled Droid Sans Fallback face used by the
/// page-render substitution path (ISO 32000-2 §9.7.5.2 — predefined CIDFont
/// glyph supply).
///
/// Gated on `cjk-render-fallback`; when the feature is off the constant is
/// not compiled in and substitution falls through to the existing
/// system-font path. Cached once per process so we don't re-parse the 3.4 MB
/// font on every glyph paint.
#[cfg(feature = "cjk-render-fallback")]
static RENDER_CJK_FALLBACK_FACE: std::sync::OnceLock<Option<Arc<CachedFace>>> =
    std::sync::OnceLock::new();

#[cfg(feature = "cjk-render-fallback")]
fn render_cjk_fallback_face() -> Option<Arc<CachedFace>> {
    RENDER_CJK_FALLBACK_FACE
        .get_or_init(|| {
            let bytes: &'static [u8] = crate::fonts::form_fallback::render_cjk_fallback_bytes();
            // `CachedFace::new` takes `Arc<Vec<u8>>`. The bundled font is
            // `'static` data baked into the binary by `include_bytes!`;
            // copying it into a `Vec<u8>` once at first use is acceptable
            // because it happens at most once per process. After that the
            // `Arc<CachedFace>` is shared via the `OnceLock`.
            CachedFace::new(Arc::new(bytes.to_vec()), 0).map(Arc::new)
        })
        .clone()
}

/// Glyphs that painted nothing while the cursor advanced, tallied for one
/// text run.
///
/// A dropped glyph is indistinguishable from real whitespace downstream: OCR
/// transcribes the gap-filled render faithfully and the loss reaches a search
/// index unnoticed (#991). One warning per font per page names the first
/// offending code and glyph id and how many followed in the run that
/// triggered it — enough to identify a broken font without a log line per
/// glyph.
#[derive(Default)]
struct GlyphDropTally {
    count: usize,
    first: Option<(&'static str, u32, u16)>,
}

/// Whether a run is one for which "painted nothing" is the *expected* outcome,
/// so a drop tally would be a false positive.
///
/// `outline_glyph` returns `None` for an **empty** glyph as well as a missing
/// one, so the tally cannot by itself tell "the font gave us nothing" from
/// "the font says paint nothing". Two populations are legitimately the latter:
///
/// - Invisible text, ISO 32000-1:2008 §9.3.6 render modes 3 and 7 — an OCR
///   text layer sitting under a scanned page image, which is meant to be
///   searchable and not drawn.
/// - The glyphless fonts those OCR tools emit. Tesseract and ocrmypdf ship a
///   synthetic font — conventionally named with GLYPHLESS — mapping every CID
///   to a non-zero glyph id with an empty outline and a correct /ToUnicode.
///   Every such page rendered and extracted perfectly and still reported one
///   warning per page claiming invisible data loss.
///
/// These are exactly the signals `src/extractors/text.rs` already gates on for
/// the same population; the diagnostic gated on neither. A diagnostic that
/// fires on the healthy common case trains callers to ignore the channel.
fn drop_tally_is_expected(gs: &GraphicsState, base_font: &str) -> bool {
    gs.render_mode == 3 || gs.render_mode == 7 || base_font.to_uppercase().contains("GLYPHLESS")
}

impl GlyphDropTally {
    fn record(&mut self, reason: &'static str, char_code: u32, gid: u16) {
        self.count += 1;
        self.first.get_or_insert((reason, char_code, gid));
    }

    /// The structured warning for this run, or `None` when nothing dropped.
    fn warning(&self, font_name: &str) -> Option<crate::extractors::warnings::Warning> {
        let (reason, char_code, gid) = self.first?;
        Some(crate::extractors::warnings::Warning {
            category: crate::extractors::warnings::WarningCategory::GlyphDropped,
            page: None,
            message: format!(
                "font '{font_name}' painted nothing for {} glyph(s) while advancing the cursor; \
                 first was code 0x{char_code:X} (glyph {gid}): {reason}. The page renders with \
                 a gap that reads as whitespace downstream.",
                self.count
            ),
            spec_section: None,
        })
    }

    fn report(&self, font_name: &str, rasterizer: &TextRasterizer) {
        let Some(warning) = self.warning(font_name) else {
            return;
        };
        if !rasterizer.first_report_for(font_name) {
            return;
        }
        log::warn!(target: "pdf_oxide::fonts", "{}", warning.message);
        crate::extractors::warnings::push_global_warning(warning);
    }
}

/// Rasterizer for PDF text operations.
pub struct TextRasterizer {
    /// Font database for system font fallback.
    ///
    /// Shared across rasterizers via a process-wide `OnceLock` cache so
    /// we don't re-scan the system font directories on every new
    /// `PageRenderer`. See the `SYSTEM_FONTDB` docstring for the
    /// measurement that motivated the switch.
    fontdb: std::sync::Arc<fontdb::Database>,

    /// Fonts already named in a glyph-drop warning on the current page.
    ///
    /// Page-scoped like `PageRenderer::k_zero_warning_emitted`: the renderer
    /// clears it at the start of every page via `reset_page_warnings`, so bulk
    /// ingestion warns on every page a broken font paints. Reporting is per
    /// text run, and the global sink is never drained by render-only callers,
    /// so a broken font would otherwise push one warning per Tj/TJ element
    /// (#991 asked for once per font).
    warned_fonts: std::sync::Mutex<std::collections::HashSet<String>>,
}

impl TextRasterizer {
    /// Create a new text rasterizer using the cached system font database.
    pub fn new() -> Self {
        Self {
            fontdb: system_fontdb(),
            warned_fonts: Default::default(),
        }
    }

    /// Construct with a caller-supplied font database. Bypasses the
    /// process-wide cache — useful for tests or callers that need to
    /// pre-populate the database with non-system fonts.
    #[allow(dead_code)]
    pub fn with_fontdb(fontdb: std::sync::Arc<fontdb::Database>) -> Self {
        Self {
            fontdb,
            warned_fonts: Default::default(),
        }
    }

    /// Forget which fonts a glyph-drop warning has named. `PageRenderer`
    /// calls this at the start of every page, making the warning
    /// once-per-font-per-page rather than once per process.
    pub(crate) fn reset_page_warnings(&self) {
        if let Ok(mut warned) = self.warned_fonts.lock() {
            warned.clear();
        }
    }

    /// True the first time `font_name` is seen since the last
    /// `reset_page_warnings` call.
    fn first_report_for(&self, font_name: &str) -> bool {
        match self.warned_fonts.lock() {
            Ok(mut warned) => warned.insert(font_name.to_string()),
            Err(_) => false,
        }
    }

    /// Render a text string (Tj operator).
    /// Returns the total horizontal advance in PDF points.
    ///
    /// `color_override` carries the resolution-pipeline output: the
    /// fill RGBA replaces the value `gs` would supply when present, so
    /// the operator arm doesn't have to clone `gs` purely to splice a
    /// colour. Stroke override is accepted for forward compatibility —
    /// the text rasteriser does not currently paint stroked glyphs, so
    /// the stroke channel is recorded but not yet observable on the
    /// pixmap.
    #[allow(unused_variables)]
    pub fn render_text(
        &self,
        pixmap: &mut Pixmap,
        text: &[u8],
        base_transform: Transform,
        gs: &GraphicsState,
        color_override: Option<&crate::rendering::page_renderer::ResolvedColors>,
        _resources: &Object,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
        font_cache: &HashMap<String, Arc<crate::fonts::FontInfo>>,
    ) -> Result<f32> {
        // Get font info from cache
        let font_info = if let Some(font_name) = &gs.font_name {
            font_cache.get(font_name).cloned()
        } else {
            None
        };

        // Convert raw PDF bytes to Unicode string using font encoding
        let unicode_text = self.decode_text_to_unicode(text, font_info.as_deref());
        log::debug!("Decoded text: '{}' (font={:?})", unicode_text, gs.font_name);

        // Create paint from fill color, then apply the pipeline-resolved
        // override when present. `create_fill_paint` reads gs.fill_*
        // unconditionally; the override stamp afterwards is the only
        // place the resolved RGBA needs to land for visible-glyph paint.
        let mut paint = create_fill_paint(gs, "Normal");
        if let Some(overrides) = color_override {
            if let Some((r, g, b, a)) = overrides.fill {
                paint.set_color(
                    tiny_skia::Color::from_rgba(r, g, b, a).unwrap_or(tiny_skia::Color::BLACK),
                );
            }
        }
        // Text rendering mode 3 = invisible text (searchable OCR layers).
        // Mode 7 = add-to-clip-path only, with NO painting (ISO 32000-1
        // §9.3.6); it previously fell through and painted glyphs visibly. The
        // clip-path accumulation itself (modes 4–7) is not yet applied, but
        // mode-7 glyphs must at minimum not paint. (WS1.5)
        if gs.render_mode == 3 || gs.render_mode == 7 {
            paint.set_color(tiny_skia::Color::from_rgba(0.0, 0.0, 0.0, 0.0).unwrap());
        }

        // Predefined CIDFont substitution path: if this font was flagged
        // at load time as an Adobe predefined CIDFont with no embedded
        // outlines (Ryumin-Light, GothicBBB-Medium, STSong-Light,
        // MHei-Medium, HYSMyeongJo-Medium, …), route the paint through
        // the bundled Droid Sans Fallback. ISO 32000-2 §9.7.5.2 requires a
        // conforming reader to materialise glyphs for these collections;
        // pre-substitution the renderer dropped every glyph to .notdef and
        // produced a blank page. Gated on `cjk-render-fallback` — when the
        // feature is off the substitution field is still populated (so the
        // metadata is observable to embedders) but no bundled font ships and
        // we fall through to the existing routing, which in practice means
        // the document still renders blank for the substituted font but the
        // rest of the page paints normally.
        #[cfg(feature = "cjk-render-fallback")]
        if let Some(ref info) = font_info {
            if let Some(collection) = info.cjk_substitution {
                log::debug!(
                    "Routing font '{}' through CJK substitution path (collection {:?})",
                    info.base_font,
                    collection
                );
                return self.render_substituted_cjk(
                    pixmap,
                    text,
                    info,
                    collection,
                    &paint,
                    base_transform,
                    gs,
                    clip_mask,
                );
            }
        }

        // Find and load font - prioritize embedded font data
        let pdf_font_name = gs.font_name.as_deref().unwrap_or("Helvetica");
        let font_data_and_index: Option<(Option<fontdb::ID>, Arc<Vec<u8>>, u32, bool)> =
            if let Some(ref info) = font_info {
                if let Some(ref embedded) = info.embedded_font_data {
                    // Simple (non-Type0) TrueType subsets whose sole cmap subtable
                    // is a byte-indexed table must be rendered by feeding the raw
                    // PDF content bytes to the embedded cmap directly — the PDF
                    // byte is the cmap input under the font's declared encoding
                    // (ISO 32000-1 §9.6.6.4). Unicode shaping against these fonts
                    // is unreliable: even if a space or punctuation happens to
                    // share a codepoint with a cmap key, shaping for letters
                    // resolves to .notdef and the system-font fallback picks up
                    // unrelated glyphs. Bypass the Unicode shaping path entirely
                    // for this subtype so the byte→GID route is taken for every
                    // `Tj` / `TJ` call, not just the ones whose decoded Unicode
                    // happens to miss the cmap.
                    // Classify the embedded font's cmap tables. Computed
                    // locally on every call — a cheap zero-copy `ttf_parser`
                    // probe; the process-wide memoisation was removed as
                    // unsound under concurrency (issue #505).
                    let (is_byte_indexed, has_unicode_cmap) = classify_embedded_font(embedded);
                    if info.subtype != "Type0" && is_byte_indexed {
                        log::debug!(
                        "Using embedded font '{}' with byte-indexed cmap (simple TrueType subset)",
                        info.base_font
                    );
                        return self.render_cid_direct(
                            pixmap,
                            text,
                            info,
                            embedded,
                            0,
                            &paint,
                            base_transform,
                            gs,
                            clip_mask,
                        );
                    }

                    // A CIDFontType0's codes are CIDs, and §9.7.4.2
                    // (`docs/spec/pdf.md`:18643-18652) routes them through the
                    // CFF, not through the sfnt cmap: with CIDFont operators in
                    // the Top DICT "the CIDs shall be used to determine the GID
                    // value ... using the charset table in the CFF program",
                    // and without them "the CIDs shall be used directly as GID
                    // values". The cmap belongs to the Type 2 mechanism, which
                    // the same clause describes separately as TrueType's way of
                    // mapping "character codes to glyph indices".
                    //
                    // The trap is that Table 126 (:19786) *requires* an
                    // OpenType CIDFontType0 to include a "cmap" table. So its
                    // presence says nothing about how the codes should be
                    // resolved, yet it made `has_unicode_cmap` true and won the
                    // dispatch below — sending CIDs through a Unicode lookup,
                    // where they all resolved to glyph 0. One page whose whole
                    // content was a single Tj rendered blank for that reason,
                    // its CIDs coming from an embedded CMap with a private
                    // `GrpOne` ordering for which no predefined table exists.
                    let cid_keyed_cff = info.subtype == "Type0"
                        && info.cid_font_type.as_deref() == Some("CIDFontType0");
                    if has_unicode_cmap && !cid_keyed_cff {
                        log::debug!("Using embedded font data for '{}'", info.base_font);
                        Some((None, Arc::clone(embedded), 0, false))
                    } else if info.subtype == "Type0"
                        && info.cid_to_gid_map.is_some()
                        && info.cid_font_type.as_deref() == Some("CIDFontType2")
                    {
                        // CIDFontType2 (TrueType) with CIDToGIDMap — use direct GID rendering.
                        log::debug!(
                            "Using embedded font '{}' with CIDToGIDMap (CIDFontType2)",
                            info.base_font
                        );
                        Some((None, Arc::clone(embedded), 0, true))
                    } else if info.cff_gid_map.is_some()
                        || (info.subtype == "Type0"
                            && info.cid_font_type.as_deref() == Some("CIDFontType0"))
                    {
                        // CFF font — use direct GID rendering.
                        //
                        // For simple (non-Type0) CFF fonts the `cff_gid_map` is
                        // built at load time by
                        // [`crate::fonts::cff_encoding::parse_cff_gid_mapping_with_pdf_encoding`],
                        // which uses the PDF font dictionary's `/Encoding`
                        // (typically WinAnsi) as the byte → glyph-name source
                        // and the CFF Charset as the glyph-name → GID resolver
                        // (ISO 32000-1 §9.6.6). The subsetter's own CFF Encoding
                        // table is *not* consulted directly — sparse subsetter
                        // CFF Encoding tables would silently drop most content
                        // bytes to `.notdef` otherwise.
                        //
                        // Type0 + CIDFontType0 (CFF / OpenType-CFF): Identity-H
                        // emission means the content-stream's 2-byte codes ARE
                        // the GIDs in the CFF charset; bypass harfrust Unicode
                        // shaping (which round-trips CID→Unicode→GID through
                        // the patched cmap and can drift on CFF charset
                        // positions) and feed the raw codes to
                        // render_cid_direct (G3-h). ttf-parser handles CFF
                        // outlines for sfnt-wrapped OpenType-CFF (OTTO); raw
                        // CFF streams were already wrapped by
                        // `font_dict::wrap_cff_in_opentype` at load time.
                        log::debug!(
                            "Using embedded CFF font '{}' with direct GID mapping",
                            info.base_font
                        );
                        Some((None, Arc::clone(embedded), 0, true))
                    } else {
                        log::debug!(
                            "Embedded font '{}' lacks usable cmap, falling back to system font",
                            info.base_font
                        );
                        self.load_font_data(&info.base_font)
                            .map(|(id, d, i)| (Some(id), d, i, false))
                    }
                } else {
                    self.load_font_data(&info.base_font)
                        .map(|(id, d, i)| (Some(id), d, i, false))
                }
            } else {
                self.load_font_data(pdf_font_name)
                    .map(|(id, d, i)| (Some(id), d, i, false))
            };

        if let Some((font_id, font_data, index, use_cid_to_gid)) = font_data_and_index {
            if use_cid_to_gid {
                // Direct CIDToGIDMap/CFF rendering — bypass harfrust, use ttf-parser for glyph outlines
                match self.render_cid_direct(
                    pixmap,
                    text,
                    font_info.as_deref().unwrap(),
                    &font_data,
                    index,
                    &paint,
                    base_transform,
                    gs,
                    clip_mask,
                ) {
                    Ok(advance) => return Ok(advance),
                    Err(e) => {
                        // Fall back to system font if embedded parsing fails
                        log::warn!(
                            "Direct CID/CFF rendering failed: {}, falling back to system font",
                            e
                        );
                        if let Some((fb_id, fallback_data, fallback_idx)) =
                            self.load_font_data(pdf_font_name)
                        {
                            return self.render_unicode_text(
                                pixmap,
                                &unicode_text,
                                text,
                                font_info.as_deref(),
                                Some(fb_id),
                                fallback_data,
                                fallback_idx,
                                &paint,
                                base_transform,
                                gs,
                                clip_mask,
                                pdf_font_name,
                                false,
                            );
                        }
                    },
                }
            }
            Ok(self.render_unicode_text(
                pixmap,
                &unicode_text,
                text, // raw bytes
                font_info.as_deref(),
                font_id,
                font_data,
                index,
                &paint,
                base_transform,
                gs,
                clip_mask,
                pdf_font_name,
                true, // allow_fallback
            )?)
        } else {
            let font_name = font_info
                .as_ref()
                .map(|i| i.base_font.as_str())
                .unwrap_or("unknown");
            log::warn!(
                "No font found for '{}', text may render incorrectly. \
                 Install common fonts (e.g., liberation-fonts, dejavu-fonts, or noto-fonts).",
                font_name
            );
            // Fallback to simple rendering if font not found
            Ok(self.render_text_fallback(
                pixmap,
                &unicode_text,
                &paint,
                base_transform,
                gs,
                clip_mask,
            )?)
        }
    }

    /// Decode raw PDF text bytes to a Unicode string based on font type.
    /// Decode a show string the way the render path paints it.
    ///
    /// Rendering expands a ligature so the shaper sees ordinary glyphs, drops
    /// a code no table resolves rather than writing a question mark into the
    /// page, and never keeps an unmapped code.
    ///
    /// No GlyphDropTally here: this decode runs before render_text routes the
    /// run, and the byte-to-GID / direct-CID paths paint correctly from raw
    /// bytes for exactly the font classes whose Unicode decode fails. Only the
    /// chosen paint path may record a drop.
    fn decode_text_to_unicode(
        &self,
        bytes: &[u8],
        font: Option<&crate::fonts::FontInfo>,
    ) -> String {
        crate::fonts::unicode_decode::decode_text_to_unicode(
            bytes,
            font,
            DecodePolicy {
                preserve_unmapped: false,
                decompose_ligatures: true,
                question_mark_for_invalid: false,
            },
        )
    }

    /// Measure-only: compute the horizontal advance of a Tj text string
    /// without painting any glyphs.
    ///
    /// Used by the operator loop when a text-showing operator falls inside an
    /// excluded OCG scope: glyphs must not be rasterised, but the text matrix
    /// still needs to advance so that any subsequent visible text in the same
    /// BT/ET block paints at the correct X position.
    ///
    /// Implements the PDF text advance formula `tx = ((w0 * Tfs) + Tc + Tw) * Th`
    /// per ISO 32000-1 §9.4.4, summing across the source-character widths exposed
    /// by [`crate::fonts::FontInfo::get_glyph_width`].
    pub fn measure_text(
        &self,
        text: &[u8],
        gs: &GraphicsState,
        font_cache: &HashMap<String, Arc<crate::fonts::FontInfo>>,
    ) -> f32 {
        let font_info = gs
            .font_name
            .as_ref()
            .and_then(|n| font_cache.get(n).cloned());
        measure_text_bytes(text, gs, font_info.as_deref())
    }

    /// Measure-only: compute the total advance of a TJ array along the
    /// active writing axis (x for WMode 0, y for WMode 1), without
    /// painting any glyphs.
    pub fn measure_tj_array(
        &self,
        array: &[TextElement],
        gs: &GraphicsState,
        font_cache: &HashMap<String, Arc<crate::fonts::FontInfo>>,
    ) -> f32 {
        let font_info = gs
            .font_name
            .as_ref()
            .and_then(|n| font_cache.get(n).cloned());
        let mut total: f32 = 0.0;
        for element in array {
            match element {
                TextElement::String(text) => {
                    total += measure_text_bytes(text, gs, font_info.as_deref());
                },
                TextElement::Offset(offset) => {
                    // PDF numeric offsets in a TJ array shift the cursor by
                    // -offset/1000 * font_size along the active writing
                    // axis. The axis swap is applied by the caller via
                    // advance_text_matrix; here we just accumulate the
                    // scalar magnitude.
                    let shift = (-offset / 1000.0) * gs.font_size;
                    total += shift;
                },
            }
        }
        total
    }

    /// Render a TJ array (text with positioning adjustments).
    ///
    /// Returns the total advance along the active writing axis (x for
    /// WMode 0, y for WMode 1) in PDF text-space units. The axis swap is
    /// applied by the caller via [`GraphicsState::advance_text_matrix`];
    /// the rasterizer never constructs a horizontal-translation matrix
    /// directly.
    ///
    /// `color_override` carries the resolution-pipeline output. It is
    /// threaded into each inner `render_text` call so the per-element
    /// paint colour is the resolved RGBA rather than the `gs.fill_*`
    /// field the operator stack carried. The existing per-call
    /// `current_gs.clone()` (needed to advance `text_matrix` between TJ
    /// elements) is the only `GraphicsState` allocation on the TJ path
    /// — the operator-arm-side clone is eliminated.
    pub fn render_tj_array(
        &self,
        pixmap: &mut Pixmap,
        array: &[TextElement],
        base_transform: Transform,
        gs: &GraphicsState,
        color_override: Option<&crate::rendering::page_renderer::ResolvedColors>,
        resources: &Object,
        doc: &PdfDocument,
        clip_mask: Option<&tiny_skia::Mask>,
        font_cache: &HashMap<String, Arc<crate::fonts::FontInfo>>,
    ) -> Result<f32> {
        let mut current_gs = gs.clone();
        let mut total_advance: f32 = 0.0;

        for element in array {
            match element {
                TextElement::String(text) => {
                    let advance = self.render_text(
                        pixmap,
                        text,
                        base_transform,
                        &current_gs,
                        color_override,
                        resources,
                        doc,
                        clip_mask,
                        font_cache,
                    )?;
                    current_gs.advance_text_matrix(advance);
                    total_advance += advance;
                },
                TextElement::Offset(offset) => {
                    let shift = (-offset / 1000.0) * current_gs.font_size;
                    current_gs.advance_text_matrix(shift);
                    total_advance += shift;
                },
            }
        }
        Ok(total_advance)
    }

    /// Get font info for a specific font name from resources.
    #[allow(dead_code)]
    fn get_font_info(
        &self,
        doc: &PdfDocument,
        resources: &Object,
        font_name: &str,
    ) -> Result<crate::fonts::FontInfo> {
        if let Object::Dictionary(res_dict) = resources {
            if let Some(Object::Dictionary(fonts)) = res_dict.get("Font") {
                if let Some(font_ref) = fonts.get(font_name) {
                    let font_obj = doc.resolve_object(font_ref)?;
                    let info = crate::fonts::FontInfo::from_dict(&font_obj, doc)?;
                    log::debug!("Resolved font '{}': subtype={}, encoding={:?}, has_to_unicode={}, has_embedded={}", 
                        info.base_font, info.subtype, info.encoding, info.to_unicode.is_some(), info.embedded_font_data.is_some());
                    return Ok(info);
                }
            }
        }
        Err(Error::InvalidPdf(format!("Font {} not found", font_name)))
    }

    /// Find and load font data from system. Returns a `fontdb::ID` alongside
    /// the `Arc`-wrapped bytes so callers can look up the parsed-face cache.
    fn load_font_data(&self, pdf_font_name: &str) -> Option<(fontdb::ID, Arc<Vec<u8>>, u32)> {
        // Strip subset prefix (e.g., "ABCDEF+FontName" -> "FontName")
        let clean_name = if let Some(plus_idx) = pdf_font_name.find('+') {
            &pdf_font_name[plus_idx + 1..]
        } else {
            pdf_font_name
        };

        // Handle common CJK names and encoding markers
        let is_cjk_probability = clean_name.contains("GB2312") 
            || clean_name.contains("Identity")
            || clean_name.contains("楷体") 
            || clean_name.contains("æ¥·ä½") // Mojibake variant
            || clean_name.contains("宋体")
            || clean_name.contains("å®\u{008b}ä½") // Mojibake variant
            || clean_name.contains("黑体")
            || clean_name.contains("é»\u{0091}ä½") // Mojibake variant
            || clean_name.contains("FangSong")
            || clean_name.contains("SimSun")
            || clean_name.contains("SimHei")
            || clean_name.contains("KaiTi")
            || pdf_font_name == "F1";

        let final_name = if clean_name.contains("楷体")
            || clean_name.contains("æ¥·ä½")
            || clean_name.contains("KaiTi")
        {
            "KaiTi"
        } else if clean_name.contains("宋体")
            || clean_name.contains("å®\u{008b}ä½")
            || clean_name.contains("SimSun")
        {
            "SimSun"
        } else if clean_name.contains("黑体")
            || clean_name.contains("é»\u{0091}ä½")
            || clean_name.contains("SimHei")
        {
            "SimHei"
        } else {
            clean_name
        };

        // Map well-known PDF/LaTeX font names to system font equivalents
        let mut variants = vec![final_name.to_string()];

        // URW/TeX font mappings to URW base35 system fonts
        if clean_name.contains("URWPalladioL") || clean_name.contains("Palatino") {
            variants.insert(0, "P052".to_string());
            variants.push("Palatino Linotype".to_string());
            variants.push("TeX Gyre Pagella".to_string());
        } else if clean_name.contains("NimbusRomNo9L") || clean_name.contains("NimbusRoman") {
            variants.insert(0, "Nimbus Roman".to_string());
            variants.push("Times New Roman".to_string());
        } else if clean_name.contains("NimbusSanL") || clean_name.contains("NimbusSans") {
            variants.insert(0, "Nimbus Sans".to_string());
            variants.push("Arial".to_string());
        } else if clean_name.contains("NimbusMonL") || clean_name.contains("NimbusMono") {
            variants.insert(0, "Nimbus Mono PS".to_string());
            variants.push("Courier New".to_string());
        } else if clean_name.contains("CMSS")
            || clean_name.contains("CMR")
            || clean_name.contains("CMBX")
        {
            // Computer Modern fonts (LaTeX) — use Latin Modern or serif fallback
            variants.push("Latin Modern Roman".to_string());
            variants.push("Computer Modern".to_string());
        } else if clean_name.contains("URWBookmanL") || clean_name.contains("Bookman") {
            variants.insert(0, "Bookman URW".to_string());
        } else if clean_name.contains("CenturySchL") || clean_name.contains("NewCentury") {
            variants.insert(0, "C059".to_string());
        } else if clean_name.contains("URWChanceryL") || clean_name.contains("Chancery") {
            variants.insert(0, "Z003".to_string());
        }

        if is_cjk_probability {
            variants.push("Noto Sans CJK SC".to_string());
            variants.push("Noto Serif CJK SC".to_string());
            variants.push("WenQuanYi Micro Hei".to_string());
            variants.push("Droid Sans Fallback".to_string());
        }

        // Generic fallbacks — detect serif vs sans-serif
        let is_serif = clean_name.contains("Roman")
            || clean_name.contains("Serif")
            || clean_name.contains("Times")
            || clean_name.contains("Palladio")
            || clean_name.contains("Palatino")
            || clean_name.contains("Bookman")
            || clean_name.contains("Garamond")
            || clean_name.contains("Century")
            || clean_name.contains("Georgia")
            || clean_name.contains("CMR")
            || clean_name.contains("CMBX")
            || clean_name.contains("CMTI");
        if is_serif {
            variants.push("Times New Roman".to_string());
            variants.push("Liberation Serif".to_string());
            variants.push("DejaVu Serif".to_string());
        }
        variants.push("Arial".to_string());
        variants.push("Helvetica".to_string());
        variants.push("Liberation Sans".to_string());
        variants.push("DejaVu Sans".to_string());
        variants.push("Noto Sans".to_string());
        variants.push("FreeSans".to_string());

        let weight = if pdf_font_name.contains("Bold") || pdf_font_name.contains("Black") {
            fontdb::Weight::BOLD
        } else {
            fontdb::Weight::NORMAL
        };

        let style = if pdf_font_name.contains("Italic") || pdf_font_name.contains("Oblique") {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        };

        for variant in variants {
            let families = [
                fontdb::Family::Name(&variant),
                fontdb::Family::Serif,
                fontdb::Family::SansSerif,
            ];
            let query = fontdb::Query {
                families: &families,
                weight,
                stretch: fontdb::Stretch::Normal,
                style,
            };

            if let Some(id) = self.font_db().query(&query) {
                if let Some((arc_data, index)) = cached_font_bytes(id, self.font_db()) {
                    log::debug!(
                        "Matched system font for {}: variant={}, index={}, size={} bytes",
                        pdf_font_name,
                        variant,
                        index,
                        arc_data.len()
                    );
                    return Some((id, arc_data, index));
                }
            }
        }
        log::debug!(
            "No system font matched for '{}' after trying all fallback variants",
            pdf_font_name
        );
        None
    }

    /// Access the font database.
    fn font_db(&self) -> &fontdb::Database {
        &self.fontdb
    }

    /// Render Unicode text using shaped glyphs.
    /// Returns the total horizontal advance in PDF points.
    fn render_unicode_text(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        bytes: &[u8],
        font_info: Option<&crate::fonts::FontInfo>,
        font_id: Option<fontdb::ID>,
        font_data: Arc<Vec<u8>>,
        index: u32,
        paint: &Paint,
        base_transform: Transform,
        gs: &GraphicsState,
        clip_mask: Option<&tiny_skia::Mask>,
        pdf_font_name: &str,
        allow_fallback: bool,
    ) -> Result<f32> {
        let font_size = gs.font_size;
        let h_scale = gs.horizontal_scaling / 100.0;

        // 1. Resolve faces — prefer process-wide cache to avoid re-parsing font tables
        //    on every text segment.  Embedded fonts (font_id == None) are not cached
        //    because they are unique per-PDF and typically only rendered once.
        let cached_arc: Option<Arc<CachedFace>> =
            font_id.and_then(|id| cached_face(id, Arc::clone(&font_data), index));

        // Storage for locally-created faces when there is no cache entry
        // (embedded fonts, first-ever render of a system font).
        let _local_font: Option<harfrust::FontRef<'_>>;
        let _local_ttf: Option<ttf_parser::Face<'_>>;
        let _local_shaper_data: Option<harfrust::ShaperData>;

        let font_ref: &harfrust::FontRef<'_>;
        let shaper_data_ref: &harfrust::ShaperData;
        let ttf_face_ref: &ttf_parser::Face<'_>;
        let units_per_em: f32;

        if let Some(ref c) = cached_arc {
            _local_font = None;
            _local_ttf = None;
            _local_shaper_data = None;
            font_ref = &c.font;
            shaper_data_ref = &c.shaper_data;
            ttf_face_ref = &c.ttf_face;
            units_per_em = c.units_per_em;
        } else {
            let font_opt = harfrust::FontRef::from_index(&font_data, index).ok();
            if font_opt.is_none() {
                if allow_fallback {
                    log::warn!("Failed to create harfrust font from embedded data for '{}', falling back to system font", pdf_font_name);
                    if let Some((fb_id, fallback_data, fallback_index)) =
                        self.load_font_data(pdf_font_name)
                    {
                        return self.render_unicode_text(
                            pixmap,
                            text,
                            bytes,
                            font_info,
                            Some(fb_id),
                            fallback_data,
                            fallback_index,
                            paint,
                            base_transform,
                            gs,
                            clip_mask,
                            pdf_font_name,
                            false, // don't allow infinite fallback
                        );
                    }
                }
                return self.render_text_fallback(
                    pixmap,
                    text,
                    paint,
                    base_transform,
                    gs,
                    clip_mask,
                );
            }
            _local_font = font_opt;
            _local_ttf = ttf_parser::Face::parse(&font_data, index).ok();
            if _local_ttf.is_none() {
                return Err(Error::InvalidPdf(format!("Failed to parse font: {}", pdf_font_name)));
            }
            font_ref = _local_font.as_ref().unwrap();
            ttf_face_ref = _local_ttf.as_ref().unwrap();
            units_per_em = ttf_face_ref.units_per_em() as f32;
            // ShaperData owns its derived tables (no borrow of `font_ref`).
            _local_shaper_data = Some(harfrust::ShaperData::new(font_ref));
            shaper_data_ref = _local_shaper_data.as_ref().unwrap();
        }

        // 2. Buffer setup
        let mut buffer = harfrust::UnicodeBuffer::new();
        buffer.push_str(text);

        // Explicitly set script and direction for better CJK shaping
        if text
            .chars()
            .any(|c| (c as u32) >= 0x4E00 && (c as u32) <= 0x9FFF)
        {
            if let Some(script) = harfrust::Script::from_iso15924_tag(harfrust::Tag::new(b"Hani")) {
                buffer.set_script(script);
            }
        }
        buffer.set_direction(harfrust::Direction::LeftToRight);
        // Fill in any still-unset segment properties (script for non-CJK,
        // language) so the shaper picks the right GSUB/GPOS rules. The
        // explicit direction and CJK script set above are preserved.
        buffer.guess_segment_properties();

        // 3. Shape the text
        let shaper = shaper_data_ref.shaper(font_ref).instance(None).build();
        let glyphs = shaper.shape(buffer, harfrust::ShapeOptions::new());
        let info = glyphs.glyph_infos();
        let pos = glyphs.glyph_positions();

        let scale = font_size / units_per_em;
        log::debug!(
            "render_unicode_text: pdf_font={}, units_per_em={}, font_size={}, scale={}",
            pdf_font_name,
            units_per_em,
            font_size,
            scale
        );

        // 4. Transform setup - include full text matrix [Tm]
        let text_transform = Transform::from_row(
            gs.text_matrix.a,
            gs.text_matrix.b,
            gs.text_matrix.c,
            gs.text_matrix.d,
            gs.text_matrix.e,
            gs.text_matrix.f,
        );
        // Transform from text space to pixel space: P_pixel = base_transform * text_transform * P_text
        let combined_base = base_transform.pre_concat(text_transform);

        let mut x_cursor: f32 = 0.0; // In text space units
                                     // y_cursor tracks the cursor along the y-axis. It stays at 0 in
                                     // horizontal mode (the default) and accumulates `w1y*font_size/1000`
                                     // per glyph when WMode 1 is active. Single cursor variable keeps the
                                     // hot loop simple — the branch on `gs.text_wmode` only flips which
                                     // axis receives the advance and how the glyph is positioned
                                     // relative to its horizontal origin.
        let mut y_cursor: f32 = 0.0;
        // A glyph that paints nothing here still advances the cursor, leaving
        // a gap indistinguishable from whitespace (#991).
        let mut unicode_dropped = GlyphDropTally::default();
        let mut last_fallback_cluster: Option<usize> = None;
        let wmode = gs.text_wmode;
        // Per ISO 32000-1:2008 §9.3.3, Tw applies only to the single-byte
        // character code 32 — never to the byte value 32 inside a
        // multi-byte code (e.g. CID 32 under Identity-H/V, always 2 bytes).
        // `font_info` is constant for the whole call, so resolve this once.
        let word_space_eligible = get_byte_mode(font_info) != ByteMode::TwoByte;

        // Pre-resolve CIDs for Type0 fonts using our iterator
        let cids: Vec<u16> = if let Some(info) = font_info {
            if info.subtype == "Type0" {
                TextCharIter::new(bytes, Some(info))
                    .map(|(cid, _)| cid)
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Build mapping from Unicode byte offset → character index for correct CID lookup.
        // Rustybuzz clusters are byte offsets into the Unicode string, but we need
        // the character index to map to the corresponding CID.
        let cluster_to_char_idx: HashMap<usize, usize> = text
            .char_indices()
            .enumerate()
            .map(|(char_idx, (byte_offset, _))| (byte_offset, char_idx))
            .collect();

        // 5. Iterate through shaped glyphs
        for i in 0..info.len() {
            let glyph_id = info[i].glyph_id;
            let cluster = info[i].cluster as usize;

            // Get character at this cluster (byte offset)
            let char_at_pos = text[cluster..].chars().next().unwrap_or(' ');

            // Map cluster (Unicode byte offset) to character index
            let char_idx = cluster_to_char_idx.get(&cluster).copied().unwrap_or(0);

            // Determine how many *source* characters this glyph represents.
            // For normal 1:1 glyphs, cluster_chars == 1. For shaped
            // ligatures like the "ffi" glyph (#331 R2), one glyph covers
            // multiple characters and harfrust reports them with the
            // same cluster index on every glyph of the cluster. Since we
            // advance the output cursor by the sum of the PDF-declared
            // widths of the *source* characters (per PDF §9.2.4 text-
            // showing advance), we must add the widths of every source
            // character in the ligature cluster to the cursor, not just
            // the first character's width. Otherwise a ligature glyph
            // draws wide but only advances by one character's worth, and
            // subsequent glyphs overwrite the tail of the ligature —
            // exactly the `Efficient` → `Effi ert` symptom reported in
            // #331 on arxiv-style LaTeX-embedded fonts.
            let next_cluster_byte: usize = info
                .get(i + 1)
                .map(|n| n.cluster as usize)
                .unwrap_or(text.len());
            let cluster_chars: usize = text[cluster..next_cluster_byte.min(text.len())]
                .chars()
                .count()
                .max(1);

            // PDF Spec: tx = ((w0 * Tfs) + Tc + Tw) * Th
            // Priority:
            // 1. Explicit /W or /DW from FontInfo (in 1000ths of em),
            //    summed across every source character in the cluster
            //    so ligatures advance by the full cluster's width.
            // 2. Shaped advance from harfrust (fallback, already
            //    reflects the ligature's real width because it comes
            //    from the font's horizontal metrics table).
            let pdf_width = if let Some(font_info_ref) = font_info {
                let mut sum = 0.0_f32;
                for k in 0..cluster_chars {
                    let idx = char_idx + k;
                    let char_code = if font_info_ref.subtype == "Type0" {
                        *cids.get(idx).unwrap_or(&0)
                    } else {
                        *bytes.get(idx).unwrap_or(&0) as u16
                    };
                    sum += font_info_ref.get_glyph_width(char_code);
                }
                sum
            } else {
                // No FontInfo, use shaped advance
                pos[i].x_advance as f32 / font_size * 1000.0
            };

            let x_advance = pdf_width * font_size / 1000.0;
            let x_offset = pos[i].x_offset as f32 / units_per_em * font_size;
            let y_offset = pos[i].y_offset as f32 / units_per_em * font_size;

            let mut x_advance_override: Option<f32> = None;

            // Resolve vertical-mode displacement and origin offset once per
            // glyph. Horizontal mode: y_step = 0, paint_origin_dx/dy = 0 —
            // the same code path as before. Vertical mode: y_step =
            // w1y*Tfs/1000 (typically -font_size), and (paint_origin_dx,
            // paint_origin_dy) shifts the glyph so its vertical origin
            // (v_x, v_y) lands at the current cursor.
            //
            // For composite (Type0) vertical text the per-glyph metrics
            // come from /W2 + /DW2. Simple fonts in vertical mode are not
            // a real-world case but the helper still produces spec-default
            // metrics, keeping the math safe.
            let (y_step, paint_origin_dx, paint_origin_dy) = if wmode == 1 {
                if let Some(font_info_ref) = font_info {
                    // Sum w1y across the source-character cluster, matching the
                    // horizontal path's `pdf_width` accumulation. Use the
                    // primary glyph's vertical-origin offset (v_x, v_y) for
                    // painting — clusters share a single origin per spec.
                    let mut w1y_sum = 0.0_f32;
                    let mut head_v_x = 0.0_f32;
                    let mut head_v_y = 0.0_f32;
                    for k in 0..cluster_chars {
                        let idx = char_idx + k;
                        let cid = if font_info_ref.subtype == "Type0" {
                            *cids.get(idx).unwrap_or(&0)
                        } else {
                            *bytes.get(idx).unwrap_or(&0) as u16
                        };
                        let m = font_info_ref.get_vertical_metrics(cid);
                        w1y_sum += m.w1y;
                        if k == 0 {
                            head_v_x = m.v_x;
                            head_v_y = m.v_y;
                        }
                    }
                    let y_advance_v = w1y_sum * font_size / 1000.0;
                    let dx = -head_v_x * font_size / 1000.0;
                    let dy = -head_v_y * font_size / 1000.0;
                    (y_advance_v, dx, dy)
                } else {
                    // No FontInfo + vertical mode: spec defaults (-1000, 500, 880).
                    let m = crate::fonts::VerticalMetrics::SPEC_DEFAULT;
                    (
                        m.w1y * font_size / 1000.0,
                        -m.v_x * font_size / 1000.0,
                        -m.v_y * font_size / 1000.0,
                    )
                }
            } else {
                (0.0, 0.0, 0.0)
            };

            // Try to get glyph from primary font
            let mut pb = PathBuilder::new();
            let mut builder = SkiaOutlineBuilder(&mut pb);
            let mut has_outline = ttf_face_ref
                .outline_glyph(ttf_parser::GlyphId(glyph_id as u16), &mut builder)
                .is_some();

            if has_outline && glyph_id != 0 {
                if let Some(path) = pb.finish() {
                    // Vertical mode shifts the glyph by (-v_x, -v_y) so its
                    // vertical origin lands at the current cursor, and uses
                    // y_cursor in place of the y=0 baseline. text_rise (Ts)
                    // continues to offset perpendicular to the writing axis
                    // per §9.3.5 — horizontal in vertical mode.
                    let (rise_x, rise_y) = if wmode == 0 {
                        (0.0, gs.text_rise)
                    } else {
                        (gs.text_rise, 0.0)
                    };
                    let px = (x_cursor + x_offset + paint_origin_dx) * h_scale + rise_x;
                    let py = y_cursor + y_offset + paint_origin_dy + rise_y;
                    let glyph_transform =
                        combined_base.pre_translate(px, py).pre_scale(scale, scale);

                    guarded_fill_path(
                        pixmap,
                        &path,
                        paint,
                        tiny_skia::FillRule::Winding,
                        glyph_transform,
                        clip_mask,
                    );
                }
            } else {
                // FALLBACK PATH: If primary font fails, use the cluster offset to find the original character
                // char_at_pos already retrieved above using byte offset

                // Skip empty glyphs for spaces — advance along the active
                // writing axis (x in horizontal mode, y in vertical mode).
                if char_at_pos.is_whitespace() {
                    if wmode == 0 {
                        x_cursor += x_advance + gs.char_space;
                        if char_at_pos == ' ' && word_space_eligible {
                            x_cursor += gs.word_space;
                        }
                    } else {
                        y_cursor += y_step + gs.char_space;
                        if char_at_pos == ' ' && word_space_eligible {
                            y_cursor += gs.word_space;
                        }
                    }
                    continue;
                }

                // IMPORTANT: Only render fallback character ONCE per cluster
                if last_fallback_cluster == Some(cluster) {
                    if wmode == 0 {
                        x_cursor += x_advance;
                    } else {
                        y_cursor += y_step;
                    }
                    continue;
                }
                last_fallback_cluster = Some(cluster);

                // Try to find character in fallback CJK fonts.
                // get_cjk_fallback_cached() hits a process-wide OnceLock after the
                // first call — no fontdb queries or font clones on subsequent glyphs.
                if let Some((cjk_id, cjk_arc, cjk_index)) = get_cjk_fallback_cached(self.font_db())
                {
                    if let Some(cjk_cached) = cached_face(cjk_id, cjk_arc, cjk_index) {
                        if let Some(cjk_glyph_id) = cjk_cached.ttf_face.glyph_index(char_at_pos) {
                            let mut cjk_pb = PathBuilder::new();
                            let mut cjk_builder = SkiaOutlineBuilder(&mut cjk_pb);
                            if cjk_cached
                                .ttf_face
                                .outline_glyph(cjk_glyph_id, &mut cjk_builder)
                                .is_some()
                            {
                                if let Some(cjk_path) = cjk_pb.finish() {
                                    let cjk_scale = font_size / cjk_cached.units_per_em;
                                    let (rise_x, rise_y) = if wmode == 0 {
                                        (0.0, gs.text_rise)
                                    } else {
                                        (gs.text_rise, 0.0)
                                    };
                                    let px =
                                        (x_cursor + x_offset + paint_origin_dx) * h_scale + rise_x;
                                    let py = y_cursor + y_offset + paint_origin_dy + rise_y;
                                    // No y reflection here. ISO 32000-1:2008 9.4.4 makes
                                    // the rendering matrix Tfs/Th/Trise x Tm x CTM the whole
                                    // glyph-space-to-device transform, and the page's single
                                    // y-flip already lives in page_base_transform. The primary
                                    // glyph path above and the CID and substituted-CJK paths
                                    // all compose this same `combined_base` with a positive
                                    // scale; this fallback branch was the only place that
                                    // negated it, so a run routed here came out mirrored.
                                    let cjk_transform = combined_base
                                        .pre_translate(px, py)
                                        .pre_scale(cjk_scale, cjk_scale);
                                    guarded_fill_path(
                                        pixmap,
                                        &cjk_path,
                                        paint,
                                        tiny_skia::FillRule::Winding,
                                        cjk_transform,
                                        clip_mask,
                                    );
                                    has_outline = true;

                                    if let Some(adv) =
                                        cjk_cached.ttf_face.glyph_hor_advance(cjk_glyph_id)
                                    {
                                        x_advance_override =
                                            Some(adv as f32 / cjk_cached.units_per_em * font_size);
                                    }
                                }
                            }
                        }
                    }
                }

                if !has_outline {
                    let reason = if glyph_id == 0 {
                        "not mapped by font or CJK fallback"
                    } else {
                        "no outline in font or CJK fallback"
                    };
                    unicode_dropped.record(reason, char_at_pos as u32, glyph_id as u16);
                }
            }

            // Advance cursor in text space per ISO 32000-1:2008 §9.4.4.
            // Horizontal mode: tx = ((w0 * Tfs) + Tc + Tw) * Th
            // Vertical mode:  ty = (w1y * Tfs) + Tc + Tw (Tw applied at the
            // space CID just as in horizontal mode).
            // x_advance / y_step already include w0*Tfs / w1y*Tfs.
            if wmode == 0 {
                x_cursor += x_advance_override.unwrap_or(x_advance);
                x_cursor += gs.char_space;
                if char_at_pos == ' ' && word_space_eligible {
                    x_cursor += gs.word_space;
                }
            } else {
                y_cursor += y_step;
                y_cursor += gs.char_space;
                if char_at_pos == ' ' && word_space_eligible {
                    y_cursor += gs.word_space;
                }
            }
        }

        // Suppress the tally for runs where painting nothing is the expected
        // outcome — see `drop_tally_is_expected`.
        let base_font = font_info
            .map(|f| f.base_font.as_str())
            .unwrap_or("<system fallback>");
        if !drop_tally_is_expected(gs, base_font) {
            unicode_dropped.report(base_font, self);
        }

        // Return the magnitude of the accumulated advance along the active
        // writing axis. Callers that drive the text matrix forward consume
        // this as a scalar; in vertical mode the cursor advances in y but
        // the magnitude is identically meaningful to the matrix-update
        // helper (which itself handles the axis swap).
        Ok(if wmode == 0 { x_cursor } else { y_cursor })
    }
    /// Render text using direct CID-to-GID mapping, bypassing harfrust shaping.
    /// Used for CID subset fonts that have embedded data but no usable Unicode cmap.
    /// Per PDF spec section 9.7.4, CIDToGIDMap maps CIDs to glyph indices in the TrueType font.
    fn render_cid_direct(
        &self,
        pixmap: &mut Pixmap,
        bytes: &[u8],
        font_info: &crate::fonts::FontInfo,
        font_data: &[u8],
        index: u32,
        paint: &Paint,
        base_transform: Transform,
        gs: &GraphicsState,
        clip_mask: Option<&tiny_skia::Mask>,
    ) -> Result<f32> {
        let font_size = gs.font_size;
        let h_scale = gs.horizontal_scaling / 100.0;

        let ttf_face = ttf_parser::Face::parse(font_data, index)
            .map_err(|e| Error::InvalidPdf(format!("Failed to parse embedded font: {}", e)))?;
        let units_per_em = ttf_face.units_per_em() as f32;
        let scale = font_size / units_per_em;

        let text_transform = Transform::from_row(
            gs.text_matrix.a,
            gs.text_matrix.b,
            gs.text_matrix.c,
            gs.text_matrix.d,
            gs.text_matrix.e,
            gs.text_matrix.f,
        );
        let combined_base = base_transform.pre_concat(text_transform);

        let mut x_cursor: f32 = 0.0;
        let mut y_cursor: f32 = 0.0;
        let wmode = gs.text_wmode;
        // A glyph that paints nothing while the cursor still advances leaves an
        // invisible gap, and a caller cannot tell that from real whitespace
        // (#991). Counted per run, reported once per font, so a broken font is
        // visible without one line per glyph.
        let mut dropped = GlyphDropTally::default();

        // Iterate over character codes from the raw bytes
        for (char_code, bytes_consumed) in TextCharIter::new(bytes, Some(font_info)) {
            // Map character code to GID based on font type:
            // - Type0 (CID-keyed) without CIDToGIDMap → CID is GID
            //   (Identity-H/Identity-V emission, the case our writer
            //   uses for CFF subsets re-embedded with a synthesised
            //   cmap). The cff_gid_map only applies when the font is
            //   a SIMPLE Type1/CFF font — i.e. `subtype != "Type0"`.
            // - CIDFontType2: CIDToGIDMap maps CID → GID.
            // - CFF simple font (Type1, non-Type0): cff_gid_map maps
            //   byte → GID.
            // - Simple TrueType: consult the embedded font's cmap
            //   directly (the PDF content byte is the cmap input
            //   under the font's declared encoding; ISO 32000-1
            //   §9.6.6.4).
            // - Default: identity mapping.
            let gid = if font_info.subtype == "Type0" {
                match &font_info.cid_to_gid_map {
                    Some(crate::fonts::CIDToGIDMap::Identity) => char_code,
                    Some(crate::fonts::CIDToGIDMap::Explicit(map)) => {
                        *map.get(char_code as usize).unwrap_or(&0)
                    },
                    // §9.7.4.2 (`docs/spec/pdf.md`:18641-18644): when the
                    // embedded CFF's Top DICT uses CIDFont operators, "the CIDs
                    // shall be used to determine the GID value for the glyph
                    // procedure using the charset table in the CFF program",
                    // and the NOTE at :18646 warns the two "may differ". Only
                    // when it does NOT use those operators (:18649-18650) may
                    // "the CIDs be used directly as GID values".
                    //
                    // Treating the CID as the GID unconditionally blanked every
                    // glyph of a CID-keyed subset whose charset renumbers: one
                    // headline asked for GID 13391 where the font maps that CID
                    // to GID 107, and 13391 is past the end of a 315-glyph
                    // CharStrings INDEX, so all 23 glyphs resolved to no
                    // outline and the line rendered blank.
                    None => font_info
                        .cff_cid_to_gid
                        .as_ref()
                        .and_then(|m| m.get(&char_code).copied())
                        .unwrap_or(char_code),
                }
            } else if let Some(cff_map) = &font_info.cff_gid_map {
                *cff_map.get(&(char_code as u8)).unwrap_or(&0)
            } else if font_info.cid_to_gid_map.is_none() {
                cmap_byte_to_gid(&ttf_face, char_code as u8).unwrap_or(0)
            } else {
                match &font_info.cid_to_gid_map {
                    Some(crate::fonts::CIDToGIDMap::Identity) => char_code,
                    Some(crate::fonts::CIDToGIDMap::Explicit(map)) => {
                        *map.get(char_code as usize).unwrap_or(&0)
                    },
                    None => char_code,
                }
            };
            let cid = char_code; // For width lookup

            // Get width from PDF metrics (horizontal) and vertical advance
            // + origin offset (vertical mode). Both lookups read from
            // FontInfo's hot caches; the vertical lookup is only consulted
            // when wmode==1, keeping the horizontal fast path unchanged.
            let pdf_width = font_info.get_glyph_width(cid);
            let x_advance = pdf_width * font_size / 1000.0;
            let (y_step, paint_origin_dx, paint_origin_dy) = if wmode == 1 {
                let m = font_info.get_vertical_metrics(cid);
                (
                    m.w1y * font_size / 1000.0,
                    -m.v_x * font_size / 1000.0,
                    -m.v_y * font_size / 1000.0,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            // Get Unicode character for space/word-space detection.
            // Use '\0' as the sentinel for "no mapping" so that bytes without a
            // Unicode entry (e.g. ligatures and accented chars in symbolic TrueType
            // fonts that use the Mac Roman cmap path) are not silently treated as
            // spaces and dropped from the rendered output.
            let char_str = font_info.char_to_unicode(cid as u32).unwrap_or_default();
            let char_at_pos = char_str.chars().next().unwrap_or('\0');

            // Draw glyph outline
            if gid == 0 && !char_at_pos.is_whitespace() {
                dropped.record("no glyph id", u32::from(char_code), gid);
            }
            if gid != 0 || char_at_pos.is_whitespace() {
                if !char_at_pos.is_whitespace() {
                    let mut pb = PathBuilder::new();
                    let mut builder = SkiaOutlineBuilder(&mut pb);
                    // Outlined once: `outline_glyph` appends to the builder,
                    // so calling it twice would draw the glyph twice.
                    let outlined = ttf_face
                        .outline_glyph(ttf_parser::GlyphId(gid), &mut builder)
                        .is_some();
                    if !outlined {
                        dropped.record("no outline", u32::from(char_code), gid);
                    }
                    if outlined {
                        if let Some(path) = pb.finish() {
                            let (rise_x, rise_y) = if wmode == 0 {
                                (0.0, gs.text_rise)
                            } else {
                                (gs.text_rise, 0.0)
                            };
                            let px = (x_cursor + paint_origin_dx) * h_scale + rise_x;
                            let py = y_cursor + paint_origin_dy + rise_y;
                            let glyph_transform =
                                combined_base.pre_translate(px, py).pre_scale(scale, scale);
                            guarded_fill_path(
                                pixmap,
                                &path,
                                paint,
                                tiny_skia::FillRule::Winding,
                                glyph_transform,
                                clip_mask,
                            );
                        }
                    }
                }
            }

            // Per ISO 32000-1:2008 §9.3.3, Tw applies only to the
            // single-byte character code 32 — a 2-byte CID 32 (0x0020)
            // under Identity-H/V or another multi-byte CMap must not
            // take Tw.
            let word_space_eligible = bytes_consumed == 1 && char_code == 32;
            if wmode == 0 {
                x_cursor += x_advance + gs.char_space;
                if word_space_eligible {
                    x_cursor += gs.word_space;
                }
            } else {
                y_cursor += y_step + gs.char_space;
                if word_space_eligible {
                    y_cursor += gs.word_space;
                }
            }
        }
        if !drop_tally_is_expected(gs, &font_info.base_font) {
            dropped.report(&font_info.base_font, self);
        }

        Ok(if wmode == 0 { x_cursor } else { y_cursor })
    }

    /// Paint a Tj / TJ string for an Adobe predefined CIDFont whose source
    /// PDF doesn't embed glyph outlines (ISO 32000-2 §9.7.5.2 — Ryumin-Light,
    /// GothicBBB-Medium, STSong-Light, MHei-Medium, HYSMyeongJo-Medium, …).
    ///
    /// Routes each CID through the appropriate Adobe character-collection
    /// table to a Unicode code point, then through Droid Sans Fallback's
    /// Unicode `cmap` to a glyph_id, then paints the outline. Advance widths
    /// come from the PDF's own metrics (`/W`, `/DW`) so the layout matches the
    /// original document even though the glyph shapes are sans-serif
    /// substitutes for whichever face the producer requested.
    ///
    /// When a CID has no Unicode mapping under the resolved collection (rare
    /// — both real-world fixtures probe every CID in the Adobe-Japan1 table
    /// without a miss) or when Droid Sans Fallback has no glyph for the
    /// resolved Unicode (sparse for archaic CJK ideographs at the edges of
    /// the Adobe collections), the paint is skipped but the advance is
    /// preserved so subsequent glyphs land at the correct text-space
    /// position.
    ///
    /// Honours vertical writing mode (`gs.text_wmode == 1`): the text cursor
    /// advances along y, the glyph origin is offset by the PDF's `(v_x, v_y)`
    /// from `/W2` / `/DW2`, and the glyph itself is painted with the same
    /// shape Droid Sans Fallback supplies (vertical-form variant glyphs are
    /// not provided — sans-serif glyph integrity is preferable to a blank
    /// column).
    #[cfg(feature = "cjk-render-fallback")]
    fn render_substituted_cjk(
        &self,
        pixmap: &mut Pixmap,
        bytes: &[u8],
        font_info: &crate::fonts::FontInfo,
        collection: crate::fonts::predefined_cidfont::CharacterCollection,
        paint: &Paint,
        base_transform: Transform,
        gs: &GraphicsState,
        clip_mask: Option<&tiny_skia::Mask>,
    ) -> Result<f32> {
        let face = match render_cjk_fallback_face() {
            Some(f) => f,
            None => {
                log::warn!(
                    "Font '{}': CJK predefined-CIDFont substitution unavailable — \
                     bundled Droid Sans Fallback face failed to load. Falling back \
                     to .notdef paint with advance-only.",
                    font_info.base_font
                );
                return self.measure_only_advance(bytes, font_info, gs);
            },
        };
        let ttf_face = &face.ttf_face;
        let font_size = gs.font_size;
        let h_scale = gs.horizontal_scaling / 100.0;
        let units_per_em = face.units_per_em;
        let scale = font_size / units_per_em;

        let text_transform = Transform::from_row(
            gs.text_matrix.a,
            gs.text_matrix.b,
            gs.text_matrix.c,
            gs.text_matrix.d,
            gs.text_matrix.e,
            gs.text_matrix.f,
        );
        let combined_base = base_transform.pre_concat(text_transform);

        let mut x_cursor: f32 = 0.0;
        let mut y_cursor: f32 = 0.0;
        let wmode = gs.text_wmode;

        let mut glyphs_painted: usize = 0;
        let mut glyphs_missing: usize = 0;

        for (char_code, bytes_consumed) in TextCharIter::new(bytes, Some(font_info)) {
            // code == CID holds by construction: the load-time gate in
            // `FontInfo::from_dict` only sets `cjk_substitution` when the
            // /Encoding resolved to `Encoding::Identity` (Identity-H/V or an
            // Adobe-collection identity CMap stream). Non-Identity predefined
            // CMaps (90ms-RKSJ-H, GBK-EUC-H, …) carry raw legacy multi-byte
            // codes and are never routed here.
            let cid = char_code;

            // PDF advance metrics (font's own /W array) — paint position is
            // independent of the substituted glyph's native advance.
            let pdf_width = font_info.get_glyph_width(cid);
            let x_advance = pdf_width * font_size / 1000.0;
            let (y_step, paint_origin_dx, paint_origin_dy) = if wmode == 1 {
                let m = font_info.get_vertical_metrics(cid);
                (
                    m.w1y * font_size / 1000.0,
                    -m.v_x * font_size / 1000.0,
                    -m.v_y * font_size / 1000.0,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            // CID → Unicode → glyph_id. The PDF's CID is resolved to a
            // Unicode code point, then the bundled font's `cmap` maps that
            // point to a glyph_id. Source of the CID → Unicode mapping,
            // in priority order:
            //   1. the font's /ToUnicode CMap — authoritative for this
            //      font's CIDs (§9.10.2), and the only correct mapping for
            //      an Identity-encoded subset whose CIDs are not the Adobe
            //      collection's CIDs;
            //   2. the Adobe character collection table (e.g. UniJIS-UCS2-H
            //      for Adobe-Japan1) — the common case for the real
            //      predefined CIDFonts (Ryumin-Light, …) this substitution
            //      targets, which usually ship no /ToUnicode.
            // Either step can miss for CIDs outside both sources or Unicode
            // points outside Droid Sans Fallback's coverage — then we paint
            // nothing but still advance the cursor so the rest lands right.
            let mut gid: u16 = 0;
            let mut ch: char = '\0';
            let unicode = font_info
                .to_unicode
                .as_ref()
                .and_then(|lazy| lazy.get())
                .and_then(|cmap| cmap.get(&(cid as u32)).and_then(|s| s.chars().next()))
                .filter(|c| !matches!(*c, '\u{FFFD}' | '\u{FFFE}' | '\u{FFFF}'))
                .or_else(|| collection.cid_to_unicode(cid).and_then(char::from_u32));
            if let Some(c) = unicode {
                ch = c;
                if let Some(g) = ttf_face.glyph_index(c) {
                    gid = g.0;
                }
            }

            // Treat ASCII whitespace as advance-only: the glyph shape is a
            // blank box in DroidSans and would paint nothing anyway, but
            // routing through `outline_glyph` for every space costs a
            // path-build allocation we can skip.
            let is_whitespace = ch.is_whitespace();
            if gid != 0 && !is_whitespace {
                let mut pb = PathBuilder::new();
                let mut builder = SkiaOutlineBuilder(&mut pb);
                if ttf_face
                    .outline_glyph(ttf_parser::GlyphId(gid), &mut builder)
                    .is_some()
                {
                    if let Some(path) = pb.finish() {
                        let (rise_x, rise_y) = if wmode == 0 {
                            (0.0, gs.text_rise)
                        } else {
                            (gs.text_rise, 0.0)
                        };
                        let px = (x_cursor + paint_origin_dx) * h_scale + rise_x;
                        let py = y_cursor + paint_origin_dy + rise_y;
                        let glyph_transform =
                            combined_base.pre_translate(px, py).pre_scale(scale, scale);
                        guarded_fill_path(
                            pixmap,
                            &path,
                            paint,
                            tiny_skia::FillRule::Winding,
                            glyph_transform,
                            clip_mask,
                        );
                        glyphs_painted += 1;
                    }
                }
            } else if !is_whitespace {
                glyphs_missing += 1;
            }

            // Per ISO 32000-1:2008 §9.3.3, Tw applies only to the
            // single-byte character code 32. This substitution path is
            // only reached for Identity-encoded CIDFonts (see the
            // `code == CID` comment above), which are always 2-byte, so
            // `bytes_consumed == 1` never holds today — kept explicit
            // (rather than dropping Tw unconditionally) so this stays
            // correct if this path is ever reached for a 1-byte codespace.
            let word_space_eligible = bytes_consumed == 1 && ch == ' ';
            if wmode == 0 {
                x_cursor += x_advance + gs.char_space;
                if word_space_eligible {
                    x_cursor += gs.word_space;
                }
            } else {
                y_cursor += y_step + gs.char_space;
                if word_space_eligible {
                    y_cursor += gs.word_space;
                }
            }
        }

        if glyphs_missing > 0 {
            log::debug!(
                "Font '{}': CJK substitution painted {} glyphs, skipped {} \
                 (no Unicode mapping or no glyph in Droid Sans Fallback)",
                font_info.base_font,
                glyphs_painted,
                glyphs_missing
            );
        }
        // §9.4.4: tx = ((w0·Tfs)+Tc+Tw)·Th, ty has no Th factor. The paint
        // loop above defers Th to the per-glyph `px` computation, so the
        // returned text-space advance applies it here — matching
        // `measure_text_bytes`, which this function falls back to when the
        // bundled face is unavailable.
        Ok(if wmode == 0 {
            x_cursor * h_scale
        } else {
            y_cursor
        })
    }

    /// Advance-only fallback used when CJK substitution is requested but the
    /// bundled face is unavailable (feature off at the include site, or the
    /// loader failed). Returns the cumulative advance along the active writing
    /// axis so downstream text continues at the correct position even though
    /// no glyph was painted.
    #[cfg(feature = "cjk-render-fallback")]
    fn measure_only_advance(
        &self,
        bytes: &[u8],
        font_info: &crate::fonts::FontInfo,
        gs: &GraphicsState,
    ) -> Result<f32> {
        Ok(measure_text_bytes(bytes, gs, Some(font_info)))
    }

    /// Fallback simple rendering if no font found.
    /// Returns the total horizontal advance in PDF points.
    fn render_text_fallback(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        paint: &Paint,
        base_transform: Transform,
        gs: &GraphicsState,
        clip_mask: Option<&tiny_skia::Mask>,
    ) -> Result<f32> {
        // Just draw rectangles for now as very last resort
        let font_size = gs.font_size;
        let char_width = font_size * 0.6;
        let mut x_cursor: f32 = 0.0;
        let h_scale = gs.horizontal_scaling / 100.0;

        let text_transform = Transform::from_row(
            gs.text_matrix.a,
            gs.text_matrix.b,
            gs.text_matrix.c,
            gs.text_matrix.d,
            gs.text_matrix.e,
            gs.text_matrix.f,
        );
        let transform = base_transform.pre_concat(text_transform);

        for c in text.chars() {
            if !c.is_whitespace() {
                let mut pb = PathBuilder::new();
                if let Some(rect) = tiny_skia::Rect::from_xywh(
                    x_cursor * h_scale,
                    0.0,
                    char_width * 0.8,
                    font_size * 0.8,
                ) {
                    pb.push_rect(rect);
                    if let Some(path) = pb.finish() {
                        guarded_fill_path(
                            pixmap,
                            &path,
                            paint,
                            tiny_skia::FillRule::Winding,
                            transform,
                            clip_mask,
                        );
                    }
                }
            }

            x_cursor += (char_width + gs.char_space) / h_scale;
            if c == ' ' {
                x_cursor += gs.word_space / h_scale;
            }
        }

        Ok(x_cursor * h_scale)
    }
}

impl Default for TextRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the PDF-spec text advance for `bytes` without painting,
/// returning the scalar magnitude along the active writing axis.
///
/// Mirrors the advance math in [`TextRasterizer::render_unicode_text`] but
/// without any glyph outline work. Per ISO 32000-1 §9.4.4:
///
/// - Horizontal mode (`gs.text_wmode == 0`):
///   `tx = ((w0 * Tfs) + Tc + Tw) * Th`
/// - Vertical mode (`gs.text_wmode == 1`):
///   `ty = (w1y * Tfs) + Tc + Tw`
///
/// `w0` / `w1y` are in 1000ths of an em, `Tfs` is the font size, `Tc` is
/// `char_space`, `Tw` is `word_space` (applied at the space CID 0x20), and
/// `Th` is `horizontal_scaling / 100` (used in horizontal mode only — per
/// §9.3.4 horizontal scaling is along the writing direction).
///
/// When no font metrics are available we fall back to a half-em estimate per
/// character — same constant `render_text_fallback` uses for the visible path,
/// so the suppressed branch stays consistent with the painted branch.
fn measure_text_bytes(
    bytes: &[u8],
    gs: &GraphicsState,
    font_info: Option<&crate::fonts::FontInfo>,
) -> f32 {
    let font_size = gs.font_size;
    let h_scale = gs.horizontal_scaling / 100.0;
    let wmode = gs.text_wmode;
    let mut advance: f32 = 0.0;

    if let Some(font) = font_info {
        // `char_codes_with_widths` keeps the segmentation identical to the
        // painted path's width lookups (UTF-8 CMaps are variable-width), so
        // the measured advance matches what rendering would have produced,
        // while still exposing each code's byte width for the Tw gate below.
        for (code, nbytes) in char_codes_with_widths(bytes, font) {
            let char_code = u16::try_from(code).unwrap_or(0);
            // Per ISO 32000-1 §9.4.4 the advance formula differs by writing
            // mode:
            //   horizontal: tx = ((w0 * Tfs) + Tc + Tw) * Th
            //   vertical:   ty = (w1y * Tfs) + Tc + Tw       (NO Th)
            // Tz is defined as glyph stretching along the *horizontal*
            // direction only (§9.3.4); it does not scale vertical w1y or
            // vertical Tc / Tw.
            // Per §9.3.3, Tw applies only to the single-byte code 32 — a
            // 2-byte CID 0x0020 under Identity-H/V or another multi-byte
            // CMap must not take Tw. This also covers UTF-8-codespace CMaps:
            // a UTF-8 code 0x20 with width 1 is exactly the plain-ASCII-space
            // case Tw is meant to cover.
            let word_space_eligible = nbytes == 1 && code == 0x20;
            if wmode == 0 {
                let glyph_adv = font.get_glyph_width(char_code) * font_size / 1000.0;
                advance += (glyph_adv + gs.char_space) * h_scale;
                if word_space_eligible {
                    advance += gs.word_space * h_scale;
                }
            } else {
                let w1y = font.get_vertical_metrics(char_code).w1y;
                let glyph_adv = w1y * font_size / 1000.0;
                advance += glyph_adv + gs.char_space;
                if word_space_eligible {
                    advance += gs.word_space;
                }
            }
        }
    } else {
        // No font info — half-em estimate per byte. Match the wmode-aware
        // arm above by omitting h_scale in vertical mode.
        let char_width = font_size * 0.6;
        for &b in bytes {
            if wmode == 0 {
                advance += (char_width + gs.char_space) * h_scale;
                if b == 0x20 {
                    advance += gs.word_space * h_scale;
                }
            } else {
                advance += char_width + gs.char_space;
                if b == 0x20 {
                    advance += gs.word_space;
                }
            }
        }
    }
    advance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::graphics_state::GraphicsState;
    use crate::fonts::{Encoding, FontInfo, VerticalMetrics};
    use std::collections::HashMap;

    /// A run with no drops must produce no warning.
    #[test]
    fn empty_glyph_drop_tally_produces_no_warning() {
        assert!(GlyphDropTally::default().warning("AnyFont").is_none());
    }

    /// The warning names the font, the first dropped glyph, and the count.
    #[test]
    fn glyph_drop_warning_names_font_first_glyph_and_count() {
        let mut tally = GlyphDropTally::default();
        tally.record("no outline", 0x41, 7);
        tally.record("no glyph id", 0x42, 0);
        tally.record("no outline", 0x43, 9);
        let warning = tally
            .warning("AAAAAA+Broken")
            .expect("recorded drops must warn");
        assert_eq!(warning.category, crate::extractors::warnings::WarningCategory::GlyphDropped);
        assert!(warning.message.contains("AAAAAA+Broken"));
        assert!(warning.message.contains("3 glyph(s)"));
        assert!(warning.message.contains("0x41"));
        assert!(warning.message.contains("(glyph 7)"));
        assert!(warning.message.contains("no outline"));
    }

    /// A font is named once per page, not once per text run (#991) and not
    /// once per process: the latch clears with `reset_page_warnings`, so the
    /// next page names the same broken font again.
    #[test]
    fn glyph_drop_report_is_once_per_font_per_page() {
        let rasterizer = TextRasterizer::with_fontdb(std::sync::Arc::new(fontdb::Database::new()));
        assert!(rasterizer.first_report_for("OncePerPage+UniqueA"));
        assert!(!rasterizer.first_report_for("OncePerPage+UniqueA"));
        assert!(rasterizer.first_report_for("OncePerPage+UniqueB"));

        rasterizer.reset_page_warnings();
        assert!(rasterizer.first_report_for("OncePerPage+UniqueA"));
    }

    /// Query helper: the family name the CJK fallback resolver looks up.
    #[cfg(feature = "cjk-render-fallback")]
    fn query_droid_fallback(db: &fontdb::Database) -> Option<fontdb::ID> {
        db.query(&fontdb::Query {
            families: &[fontdb::Family::Name("Droid Sans Fallback")],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        })
    }

    /// The bundled CJK fallback (feature `cjk-render-fallback`) must provide
    /// real glyph coverage for the Adobe predefined character collections even
    /// on a host with NO fonts installed. Build a database holding only the
    /// bundled face — deliberately skipping `load_system_fonts` — and confirm
    /// it is both discoverable under the family name the resolver queries and
    /// able to outline representative CJK glyphs. Host-independent: it never
    /// consults system fonts.
    #[cfg(feature = "cjk-render-fallback")]
    #[test]
    fn bundled_cjk_fallback_covers_cjk_without_system_fonts() {
        let mut db = fontdb::Database::new();
        db.load_font_data(
            crate::fonts::form_fallback::font_bytes(crate::fonts::form_fallback::Fallback::Cjk)
                .to_vec(),
        );
        let id = query_droid_fallback(&db)
            .expect("bundled Droid Sans Fallback must be queryable by family name");

        // Representative Japanese kanji / Chinese hanzi / Korean hangul from
        // the Adobe-Japan1 / Adobe-GB1 / Adobe-Korea1 collections.
        let covered = db
            .with_face_data(id, |data, index| {
                let face = ttf_parser::Face::parse(data, index).expect("parse bundled face");
                ['東', '中', '가']
                    .iter()
                    .all(|&c| face.glyph_index(c).is_some())
            })
            .expect("bundled face data must be present");
        assert!(covered, "bundled CJK fallback must cover representative CJK glyphs");
    }

    /// The process-wide system font database must always expose a CJK-capable
    /// "Droid Sans Fallback" face when the feature is on — the guaranteed
    /// last-resort lookup key `get_cjk_fallback_cached` and `load_font_data`
    /// rely on. Without the feature this would be absent on a CJK-fontless host
    /// and composite fonts with no embedded outlines would render blank.
    #[cfg(feature = "cjk-render-fallback")]
    #[test]
    fn system_fontdb_registers_cjk_fallback() {
        assert!(
            query_droid_fallback(&system_fontdb()).is_some(),
            "system_fontdb must expose Droid Sans Fallback under cjk-render-fallback"
        );
    }

    /// Build a minimal Type0 FontInfo for advance-measurement tests.
    /// All horizontal widths are 1000 (one full em) and vertical metrics
    /// default to [`VerticalMetrics::SPEC_DEFAULT`] (`w1y = -1000`,
    /// `v_x = 500`, `v_y = 880`). Identity-V signals vertical writing.
    fn make_vertical_test_font() -> FontInfo {
        FontInfo {
            base_font: "TestVertical".to_string(),
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
            truetype_cmap: std::sync::OnceLock::new(),
            import_budget: None,
            embedded_glyph_names: std::sync::OnceLock::new(),
            is_truetype_font: false,
            widths: None,
            first_char: None,
            last_char: None,
            font_matrix_a: 0.001,
            default_width: 1000.0,
            cid_to_gid_map: Some(crate::fonts::CIDToGIDMap::Identity),
            cid_system_info: None,
            cid_font_type: Some("CIDFontType2".to_string()),
            cid_widths: None,
            cid_default_width: 1000.0,
            has_explicit_dw: true,
            cff_gid_map: None,
            cff_cid_to_gid: None,
            multi_char_map: HashMap::new(),
            byte_to_char_table: std::sync::OnceLock::new(),
            type0_unicode_memo: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            byte_to_width_table: std::sync::OnceLock::new(),
            weight_memo: std::sync::OnceLock::new(),
            italic_memo: std::sync::OnceLock::new(),
            std14_memo: std::sync::OnceLock::new(),
            diff_glyph_names: HashMap::new(),
            wmode: 1,
            cid_vertical_metrics: None,
            cid_default_vertical_metrics: VerticalMetrics::SPEC_DEFAULT,
            cjk_substitution: None,
        }
    }

    /// `measure_text_bytes` must return |w1y * font_size / 1000| per glyph
    /// in vertical mode — independent of the horizontal width, which would
    /// drive the answer in WMode 0. Two-byte Identity-V CIDs `<0001 0002>`
    /// at font size 12 advance by `|-1000 * 12 / 1000| * 2 = 24.0`.
    #[test]
    fn measure_text_bytes_advances_along_y_in_vertical_mode() {
        let font = make_vertical_test_font();
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 1;

        let bytes: &[u8] = &[0x00, 0x01, 0x00, 0x02];
        let advance = measure_text_bytes(bytes, &gs, Some(&font));

        // |w1y| = 1000, two glyphs, font size 12 ⇒ 24.0 magnitude.
        assert!(
            (advance.abs() - 24.0).abs() < 0.01,
            "expected ~|24.0| advance in vertical mode, got {}",
            advance
        );
        // Sign: w1y is negative, so the displacement is negative.
        assert!(
            advance < 0.0,
            "vertical advance must be negative (spec default w1y = -1000), got {}",
            advance
        );
    }

    /// Same font in horizontal mode (toggle wmode to 0) advances by the
    /// horizontal width — `1000 * 12 / 1000 = 12` per glyph, total 24.
    #[test]
    fn measure_text_bytes_advances_along_x_in_horizontal_mode() {
        let font = make_vertical_test_font();
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 0;

        let bytes: &[u8] = &[0x00, 0x01, 0x00, 0x02];
        let advance = measure_text_bytes(bytes, &gs, Some(&font));

        assert!(
            (advance - 24.0).abs() < 0.01,
            "expected ~24.0 advance in horizontal mode, got {}",
            advance
        );
        assert!(advance > 0.0, "horizontal advance must be positive");
    }

    /// `measure_text_bytes` MUST NOT apply Tz (horizontal scaling) to
    /// vertical w1y advances. Per ISO 32000-1 §9.4.4 the vertical formula
    /// is `ty = w1y * Tfs + Tc + Tw` with no Th factor; §9.3.4 defines Tz
    /// as glyph stretching along the horizontal direction only.
    #[test]
    fn measure_text_bytes_ignores_tz_in_vertical_mode() {
        let font = make_vertical_test_font();
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 1;
        gs.horizontal_scaling = 200.0; // Tz=200: would double an H advance

        let bytes: &[u8] = &[0x00, 0x01];
        let advance = measure_text_bytes(bytes, &gs, Some(&font));

        // |w1y * fs / 1000| = 12.0 (NOT 24.0 — Tz must not apply).
        assert!(
            (advance.abs() - 12.0).abs() < 0.01,
            "Tz=200 must NOT scale vertical advance: expected 12, got {}",
            advance.abs()
        );
    }

    /// Char spacing (Tc) and word spacing (Tw) in vertical mode also
    /// ignore Tz per §9.4.4.
    #[test]
    fn measure_text_bytes_vertical_tc_tw_skip_tz() {
        let font = make_vertical_test_font();
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 1;
        gs.horizontal_scaling = 200.0;
        gs.char_space = 3.0;

        // Single CID: advance = w1y*fs/1000 + Tc = -12 + 3 = -9 (Tz ignored)
        // If Tz applied, the result would be (-12 + 3) * 2 = -18.
        let bytes: &[u8] = &[0x00, 0x01];
        let advance = measure_text_bytes(bytes, &gs, Some(&font));
        assert!(
            ((-advance) - 9.0).abs() < 0.01,
            "vertical Tc must NOT pick up Tz: expected -9, got {}",
            advance
        );
    }

    /// Minimal simple (non-Type0) FontInfo for word-spacing tests — every
    /// content byte is inherently single-byte, so `get_byte_mode` always
    /// resolves to `ByteMode::OneByte` for it.
    fn make_simple_test_font() -> FontInfo {
        let mut font = make_vertical_test_font();
        font.subtype = "Type1".to_string();
        font.encoding = Encoding::Standard("WinAnsiEncoding".to_string());
        font.cid_to_gid_map = None;
        font.cid_font_type = None;
        font.wmode = 0;
        font
    }

    /// Per ISO 32000-1:2008 §9.3.3, Tw applies only to the single-byte
    /// character code 32 — never to the byte value 32 inside a multi-byte
    /// code. A 2-byte Identity CID `<0020>` (code 32, but a 2-byte code)
    /// must NOT receive word spacing, even though the raw code equals 32.
    #[test]
    fn measure_text_bytes_skips_tw_for_multibyte_cid_32() {
        let font = make_vertical_test_font(); // Type0, Identity (2-byte codes)
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 0;
        gs.word_space = 100.0;

        let bytes: &[u8] = &[0x00, 0x20]; // 2-byte CID 32
        let advance = measure_text_bytes(bytes, &gs, Some(&font));

        // Glyph width only (1000/1000 * 12 = 12); Tw must be excluded.
        assert!(
            (advance - 12.0).abs() < 0.01,
            "Tw must not apply to a 2-byte CID 32, expected 12.0, got {}",
            advance
        );
    }

    /// Control for the test above: a *simple* font's single-byte code 32
    /// is exactly the case §9.3.3 targets, so Tw must still apply there.
    #[test]
    fn measure_text_bytes_applies_tw_for_single_byte_code_32() {
        let font = make_simple_test_font();
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 0;
        gs.word_space = 100.0;

        let bytes: &[u8] = &[0x20]; // single-byte code 32
        let advance = measure_text_bytes(bytes, &gs, Some(&font));

        // Glyph width (12.0) + Tw (100.0) = 112.0.
        assert!(
            (advance - 112.0).abs() < 0.01,
            "Tw must apply to a single-byte code 32, expected 112.0, got {}",
            advance
        );
    }

    /// Two-glyph TJ array under WMode 1 reports the same magnitude as the
    /// sum of per-glyph w1y * fs / 1000 — proving `measure_tj_array`
    /// inherits `measure_text_bytes`' axis awareness rather than treating
    /// the scalar as horizontal.
    #[test]
    fn measure_tj_array_aggregates_vertical_advance() {
        use crate::content::TextElement;

        let font = make_vertical_test_font();
        let mut font_cache: HashMap<String, Arc<crate::fonts::FontInfo>> = HashMap::new();
        font_cache.insert("F1".to_string(), Arc::new(font));

        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 1;
        gs.font_name = Some("F1".to_string());

        let rasterizer = TextRasterizer::new();
        let array = vec![
            TextElement::String(vec![0x00, 0x01]),
            // -250 offset shifts the cursor forward (negative in y for V).
            TextElement::Offset(-250.0),
            TextElement::String(vec![0x00, 0x02]),
        ];
        let total = rasterizer.measure_tj_array(&array, &gs, &font_cache);

        // Two glyphs: 2 * (w1y * fs / 1000) = 2 * -12 = -24
        // Offset:    -(-250)/1000 * 12 = +3
        // Total:     -21
        assert!(
            (total - (-21.0)).abs() < 0.01,
            "measure_tj_array total should be -21 in vertical mode, got {}",
            total
        );
    }

    /// The advance returned by the CJK substitution paint path must include
    /// Th (horizontal scaling) per ISO 32000-1 §9.4.4
    /// (`tx = ((w0·Tfs)+Tc+Tw)·Th`), matching `measure_text_bytes` — which
    /// the same function falls back to when the bundled face is unavailable.
    /// Halving Tz must halve the returned advance.
    #[cfg(feature = "cjk-render-fallback")]
    #[test]
    fn substituted_cjk_advance_applies_horizontal_scaling() {
        let mut font = make_vertical_test_font();
        font.wmode = 0;
        font.cjk_substitution =
            Some(crate::fonts::predefined_cidfont::CharacterCollection::AdobeJapan1);

        let rasterizer = TextRasterizer::with_fontdb(std::sync::Arc::new(fontdb::Database::new()));
        let mut pixmap = Pixmap::new(16, 16).expect("pixmap");
        let paint = Paint::default();
        // CID 1200 (一) twice — default width 1000 ⇒ 10 pt per glyph at Tfs 10.
        let bytes: &[u8] = &[0x04, 0xB0, 0x04, 0xB0];

        let advance_at = |h_scaling: f32, pixmap: &mut Pixmap| {
            let mut gs = GraphicsState::new();
            gs.font_size = 10.0;
            gs.text_wmode = 0;
            gs.horizontal_scaling = h_scaling;
            rasterizer
                .render_substituted_cjk(
                    pixmap,
                    bytes,
                    &font,
                    crate::fonts::predefined_cidfont::CharacterCollection::AdobeJapan1,
                    &paint,
                    Transform::identity(),
                    &gs,
                    None,
                )
                .expect("substituted render")
        };

        let full = advance_at(100.0, &mut pixmap);
        let half = advance_at(50.0, &mut pixmap);

        assert!((full - 20.0).abs() < 0.01, "Th=100% advance should be 20.0, got {full}");
        assert!(
            (half - 10.0).abs() < 0.01,
            "Th=50% must halve the returned advance (§9.4.4 tx·Th): got {half}, full was {full}"
        );
    }

    /// A Type0 font whose CMap uses the UTF-8-codespace convention, as
    /// opposed to `make_vertical_test_font`'s fixed 2-byte Identity CMap.
    fn make_utf8_cmap_test_font() -> FontInfo {
        let mut font = make_vertical_test_font();
        font.encoding = Encoding::Standard("UniFull-UTF8-H".to_string());
        font.wmode = 0;
        font
    }

    /// `measure_text_bytes` must segment with `char_codes_with_widths`, so a
    /// UTF-8-codespace CMap's single-byte code 0x20 still receives Tw
    /// (ISO 32000-1 §9.3.3) even though the font routes through the UTF-8
    /// branch rather than `TextCharIter`.
    #[test]
    fn measure_text_bytes_applies_tw_for_utf8_cmap_single_byte_space() {
        let font = make_utf8_cmap_test_font();
        let mut gs = GraphicsState::new();
        gs.font_size = 12.0;
        gs.text_wmode = 0;
        gs.word_space = 100.0;

        // 0xC3 0xA9 ("é", 2-byte UTF-8 code) followed by a literal 1-byte
        // space (0x20) — the space must stay segmented as width 1 and
        // receive Tw despite following a multi-byte code in the same run.
        let bytes: &[u8] = &[0xC3, 0xA9, 0x20];
        let advance = measure_text_bytes(bytes, &gs, Some(&font));

        // Glyph widths only matter as a constant offset here; what this
        // test asserts is the *delta* Tw contributes — isolate it by
        // diffing against the same bytes with word_space forced to 0.
        let mut gs_no_tw = gs.clone();
        gs_no_tw.word_space = 0.0;
        let advance_no_tw = measure_text_bytes(bytes, &gs_no_tw, Some(&font));

        assert!(
            (advance - advance_no_tw - 100.0).abs() < 0.01,
            "Tw must apply once for the trailing single-byte space in a UTF-8-CMap run, \
             got delta {}",
            advance - advance_no_tw
        );
    }
}
