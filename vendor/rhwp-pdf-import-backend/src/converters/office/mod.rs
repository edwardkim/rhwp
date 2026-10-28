//! Office document format utilities.
//!
//! DOCX/XLSX/PPTX → PDF conversion is implemented via the office_oxide IR
//! pipeline. For PDF → DOCX export, use `PdfDocument::to_docx()`.

use crate::error::{Error, Result};
use crate::geometry::Rect;
use crate::writer::PageSize;
use crate::writer::{
    DocumentBuilder, FluentPageBuilder, ListStyle, StreamingColumn, StreamingTableConfig, TextRun,
    TextRunStyle,
};
use office_oxide::ir::{
    CodeBlock, DocumentIR, Element, Heading, Image, ImagePositioning, InlineContent, List,
    ListStyle as IrListStyle, Paragraph, ParagraphAlignment, Section, SectionBreakType, Shape,
    ShapeGeom, Table as IrTable, TextBox, TextSpan,
};
use office_oxide::{Document, DocumentFormat};
use std::path::Path;

// ── Office unit conversions ───────────────────────────────────────────────────
//
// OOXML mixes a few measurement units. These f32 constants are the conversions
// pdf_oxide needs for round-trip work; office_oxide's `core::units::Emu`
// carries the same numbers as `i64` for its own typed-API (`Emu::PER_PT` etc.).

/// Twips per point. 1 pt = 20 twips (Word's basic unit).
const TWIPS_PER_PT: f32 = 20.0;
/// EMUs per point. 914 400 EMU/inch ÷ 72 pt/inch = 12 700 EMU/pt.
const EMU_PER_PT: f32 = 12_700.0;

/// Page margins in points (1 inch = 72 points).
#[derive(Debug, Clone, Copy)]
pub struct Margins {
    /// Top margin in points
    pub top: f32,
    /// Bottom margin in points
    pub bottom: f32,
    /// Left margin in points
    pub left: f32,
    /// Right margin in points
    pub right: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 72.0,
            bottom: 72.0,
            left: 72.0,
            right: 72.0,
        }
    }
}

impl Margins {
    /// Create margins with equal values on all sides.
    pub fn uniform(margin: f32) -> Self {
        Self {
            top: margin,
            bottom: margin,
            left: margin,
            right: margin,
        }
    }

    /// Create margins with no spacing.
    pub fn none() -> Self {
        Self::uniform(0.0)
    }
}

/// Configuration for Office to PDF conversion.
#[derive(Debug, Clone)]
pub struct OfficeConfig {
    /// Page size for output PDF
    pub page_size: PageSize,
    /// Margins in points
    pub margins: Margins,
    /// Whether to embed fonts
    pub embed_fonts: bool,
    /// Default font for text
    pub default_font: String,
    /// Default font size in points
    pub default_font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Whether to include images
    pub include_images: bool,
}

impl Default for OfficeConfig {
    fn default() -> Self {
        Self {
            page_size: PageSize::Letter,
            margins: Margins::default(),
            embed_fonts: false,
            default_font: "Helvetica".to_string(),
            default_font_size: 11.0,
            line_height: 1.2,
            include_images: true,
        }
    }
}

impl OfficeConfig {
    /// Create config with A4 page size.
    pub fn a4() -> Self {
        Self {
            page_size: PageSize::A4,
            ..Default::default()
        }
    }

    /// Create config with Letter page size.
    pub fn letter() -> Self {
        Self::default()
    }
}

/// Converter for Office documents to PDF.
#[derive(Debug, Clone, Default)]
pub struct OfficeConverter {
    config: OfficeConfig,
}

impl OfficeConverter {
    /// Create a new converter with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a converter with custom configuration.
    pub fn with_config(config: OfficeConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &OfficeConfig {
        &self.config
    }

    /// Convert a DOCX file to PDF bytes.
    pub fn convert_docx(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let bytes = std::fs::read(path).map_err(|e| Error::InvalidOperation(e.to_string()))?;
        self.convert_docx_bytes(&bytes)
    }

    /// Convert DOCX bytes to PDF bytes.
    ///
    /// Pipeline: DOCX → IR → PDF via `ir_to_pdf_bytes`. Mirrors the
    /// PPTX/XLSX read paths: `docx_to_ir` recovers per-section
    /// `page_setup` from each `<w:sectPr>` (inline + body-level), and
    /// `ir_to_pdf_bytes` honours those geometries so a PDF→DOCX→PDF
    /// round-trip preserves source page count and page dimensions
    /// instead of reflowing onto Letter-sized pages.
    ///
    /// If the DOCX uses absolute frame positioning (e.g. produced by
    /// pdf_oxide's `to_docx_bytes_layout`), render via the layout-aware
    /// path that honours every paragraph's `frame_position` for visual
    /// fidelity.
    ///
    /// Embedded font programs under `word/fonts/` are not yet plumbed
    /// into the IR renderer; that's tracked separately.
    pub fn convert_docx_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let doc = Document::from_reader(cursor, DocumentFormat::Docx)
            .map_err(|e| Error::InvalidOperation(format!("DOCX parse: {e}")))?;
        let mut extra_fonts: Vec<(String, Vec<u8>)> = doc
            .as_docx()
            .map(|d| d.embedded_fonts.clone())
            .unwrap_or_default();

        let ir = doc.to_ir();
        let _ = maybe_load_unicode_fallback(&ir, &mut extra_fonts);
        let _ = maybe_load_cjk_fallback(&ir, &mut extra_fonts);
        if has_positional_layout(&ir) {
            return render_positional_ir(&ir, &self.config, &extra_fonts);
        }
        ir_to_pdf_bytes(&ir, &self.config, &extra_fonts)
    }

    /// Convert an XLSX file to PDF bytes.
    pub fn convert_xlsx(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let bytes = std::fs::read(path).map_err(|e| Error::InvalidOperation(e.to_string()))?;
        self.convert_xlsx_bytes(&bytes)
    }

    /// Convert XLSX bytes to PDF bytes.
    ///
    /// Pipeline: XLSX → IR → PDF via `ir_to_pdf_bytes`. The XLSX→IR path
    /// already promotes single-column "prose" worksheets (the kind a
    /// PDF→XLSX export produces) to `Paragraph` elements so they flow
    /// like body text and honour their per-cell font sizes; genuine
    /// grids stay as tables and go through `render_table`.
    pub fn convert_xlsx_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let doc = Document::from_reader(cursor, DocumentFormat::Xlsx)
            .map_err(|e| Error::InvalidOperation(format!("XLSX parse: {e}")))?;
        let mut extra_fonts: Vec<(String, Vec<u8>)> = doc
            .as_xlsx()
            .map(|d| d.embedded_fonts.clone())
            .unwrap_or_default();
        let ir = doc.to_ir();
        let _ = maybe_load_unicode_fallback(&ir, &mut extra_fonts);
        let _ = maybe_load_cjk_fallback(&ir, &mut extra_fonts);
        ir_to_pdf_bytes(&ir, &self.config, &extra_fonts)
    }

    /// Convert a PPTX file to PDF bytes.
    pub fn convert_pptx(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let bytes = std::fs::read(path).map_err(|e| Error::InvalidOperation(e.to_string()))?;
        self.convert_pptx_bytes(&bytes)
    }

    /// Convert PPTX bytes to PDF bytes.
    ///
    /// Pipeline: PPTX → IR → PDF via `ir_to_pdf_bytes`, which draws real
    /// PDF tables (cells with borders) for slide tables instead of
    /// flattening them to monospace pipe-text the way the markdown
    /// pipeline does. Embedded fonts under `ppt/fonts/` (e.g. produced
    /// by `PdfDocument::to_pptx_bytes`) are registered with the
    /// renderer so the original typeface is preserved.
    pub fn convert_pptx_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let doc = Document::from_reader(cursor, DocumentFormat::Pptx)
            .map_err(|e| Error::InvalidOperation(format!("PPTX parse: {e}")))?;
        let mut extra_fonts: Vec<(String, Vec<u8>)> = doc
            .as_pptx()
            .map(|d| d.embedded_fonts.clone())
            .unwrap_or_default();
        let ir = doc.to_ir();
        let _ = maybe_load_unicode_fallback(&ir, &mut extra_fonts);
        let _ = maybe_load_cjk_fallback(&ir, &mut extra_fonts);
        // Slides are inherently positional. When the IR carries shape
        // positions (every PPTX section now wraps shape content in
        // `Element::TextBox` with EMU coordinates) render each slide
        // as a single page with each shape painted at its absolute
        // rectangle. Falls back to flow rendering only when no
        // positional metadata survived parsing.
        if pptx_has_positional(&ir) {
            return render_pptx_positional(&ir, &self.config, &extra_fonts);
        }
        ir_to_pdf_bytes(&ir, &self.config, &extra_fonts)
    }

    /// Auto-detect format and convert to PDF.
    pub fn convert(&self, path: impl AsRef<Path>) -> Result<Vec<u8>> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "docx" => self.convert_docx(path),
            "doc" => {
                let bytes =
                    std::fs::read(&path).map_err(|e| Error::InvalidOperation(e.to_string()))?;
                let cursor = std::io::Cursor::new(bytes);
                let doc = Document::from_reader(cursor, DocumentFormat::Doc)
                    .map_err(|e| Error::InvalidOperation(format!("DOC parse: {e}")))?;
                ir_to_pdf_bytes(&doc.to_ir(), &self.config, &[])
            },
            "xlsx" | "xls" => self.convert_xlsx(path),
            "pptx" | "ppt" => self.convert_pptx(path),
            _ => Err(Error::InvalidPdf(format!("Unsupported file format: {ext}"))),
        }
    }
}

/// Does the IR carry frame-position metadata on at least a few
/// paragraphs? Used by `convert_docx_bytes` to pick between the
/// markdown pipeline (flowing text) and the positional renderer
/// (visual-fidelity layout).
///
/// Threshold: ≥3 positioned paragraphs. A handful of incidental
/// `<w:framePr>` uses in a normal-flow document shouldn't trigger
/// the positional path.
fn has_positional_layout(ir: &DocumentIR) -> bool {
    // Only switch to the positional renderer when the IR carries
    // *paragraph-level* absolute positioning (the layout-preserving
    // DOCX writer's `<w:framePr>` paragraphs, or vector shapes the
    // positional renderer needs to draw). Floating images alone do
    // NOT trigger this path: the flow renderer's `render_image`
    // already paints `ImagePositioning::Floating` images at their
    // anchor, and the positional renderer would drop every
    // non-positioned paragraph alongside them — leaving a 134-page
    // arxiv DOCX with all its body text silently missing.
    let mut count = 0usize;
    for sec in &ir.sections {
        for el in &sec.elements {
            let positioned = match el {
                Element::Paragraph(p) => p.frame_position.is_some(),
                Element::Heading(h) => h.frame_position.is_some(),
                Element::Shape(_) => true,
                _ => false,
            };
            if positioned {
                count += 1;
                if count >= 3 {
                    return true;
                }
            }
        }
    }
    false
}

/// Render an IR with frame positions to PDF bytes. Each positioned
/// paragraph lands at its absolute coordinates on the page; text uses
/// the run properties (font, size, bold/italic) from the IR.
///
/// Pages are derived from the IR section list; per-section page
/// breaks (the IR carries the source's hard `<w:br w:type="page"/>`
/// breaks as `Element::ThematicBreak`) advance to a new page sharing
/// the section's geometry, so an 8-page layout-preserving PDF→DOCX
/// round-trips back to 8 PDF pages with each page's positioned
/// paragraphs landing on the correct page.
fn render_positional_ir(
    ir: &DocumentIR,
    config: &OfficeConfig,
    extra_fonts: &[(String, Vec<u8>)],
) -> Result<Vec<u8>> {
    use crate::writer::{DocumentBuilder, EmbeddedFont, PageSize};

    let mut builder = DocumentBuilder::new().compress_streams(true);
    let mut registered: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Register embedded fonts so positioned text can resolve them by
    // name. Skip fonts that:
    //   - fail to parse (corrupt or non-TrueType program — Type 1
    //     PostScript fonts from arxiv-style LaTeX PDFs)
    //   - parse but lack a usable Unicode→GID cmap (CID-encoded
    //     subsets from re-embedded PDF font programs); registering
    //     them would route every glyph through GID 0 (.notdef) and
    //     make text invisible. Falling through here lets text emission
    //     pick the base 14 by name (or substitute Helvetica) so glyphs
    //     still render — wrong typeface but readable, which beats
    //     missing-glyph boxes.
    for (name, data) in extra_fonts {
        match EmbeddedFont::from_data(Some(name.clone()), data.clone()) {
            Ok(font) if font.has_usable_unicode_cmap() => {
                builder = builder.register_embedded_font(name.clone(), font);
                registered.insert(name.clone());
            },
            Ok(_) => {
                eprintln!(
                    "  [font] skipped {} ({} bytes): no Unicode cmap (CID-only subset)",
                    name,
                    data.len()
                );
            },
            Err(e) => {
                eprintln!("  [font] register failed: {} ({} bytes): {}", name, data.len(), e);
            },
        }
    }
    let unicode_fallback =
        if registered.contains(crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME) {
            Some(crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME.to_string())
        } else {
            None
        };
    let cjk_fallback =
        if registered.contains(crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME) {
            Some(crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME.to_string())
        } else {
            None
        };

    // Page geometry is derived per-section so a multi-section DOCX
    // with mixed page sizes round-trips with each section's pages at
    // its declared dimensions. Falls back to the `OfficeConfig`
    // default if the section has no `page_setup`. DOCX twips → PDF
    // points (÷ 20).
    let default_page = {
        let (w, h) = config.page_size.dimensions();
        (w, h)
    };

    // Iterate paragraphs and emit each at its frame position. Page
    // breaks in the source DOCX (parsed as `Element::ThematicBreak`
    // because that's how `convert_docx::split_at_page_break` represents
    // them) reset to a fresh page so multi-page layout DOCX renders
    // each section's content on its own page.
    //
    // Wrap the emission in `with_registered_fonts_full` so spans
    // with CJK / Hebrew / Arabic text route through the registered
    // Unicode fallback faces (`Pdfox-UnicodeFallback`,
    // `Pdfox-UnicodeFallback-CJK`). Without this the positional
    // layout DOCX path bypasses `resolve_font_for_text` and lands
    // back on the source PDF's CID-only face name, which has no
    // mapping in the writer and silently falls back to Helvetica
    // (no Han / Hebrew / Arabic glyphs).
    with_registered_fonts_full(registered, unicode_fallback, cjk_fallback, || -> Result<()> {
        for section in &ir.sections {
            let (page_w_pt, page_h_pt) = section
                .page_setup
                .as_ref()
                .map(|ps| {
                    (ps.width_twips as f32 / TWIPS_PER_PT, ps.height_twips as f32 / TWIPS_PER_PT)
                })
                .unwrap_or(default_page);
            let page_size = PageSize::Custom(page_w_pt, page_h_pt);
            let mut page = builder.page(page_size);
            for el in &section.elements {
                match el {
                    Element::ThematicBreak => {
                        page = page.new_page_same_size();
                    },
                    Element::Paragraph(p) => {
                        if let Some(fp) = p.frame_position.as_ref() {
                            // DOCX origin top-left, PDF origin bottom-left.
                            let x_pt = fp.x_twips as f32 / TWIPS_PER_PT;
                            let y_top_pt = fp.y_twips as f32 / TWIPS_PER_PT;
                            let h_pt = fp.height_twips as f32 / TWIPS_PER_PT;
                            // Convert to PDF y (baseline). Approximate: bottom of
                            // the frame = page_h - (y_top + h); add ~0.8×h for
                            // baseline within the frame.
                            let y_baseline_pt = page_h_pt - y_top_pt - h_pt * 0.8;

                            let (text, font_name, size_pt, _bold, _italic) =
                                flatten_paragraph_run(&p.content);
                            if text.is_empty() {
                                continue;
                            }
                            let resolved = resolve_font_for_text(&font_name, &text);
                            page = page
                                .font(&resolved, size_pt)
                                .at(x_pt, y_baseline_pt)
                                .text(&text);
                        }
                    },
                    Element::Heading(h) => {
                        if let Some(fp) = h.frame_position.as_ref() {
                            let x_pt = fp.x_twips as f32 / TWIPS_PER_PT;
                            let y_top_pt = fp.y_twips as f32 / TWIPS_PER_PT;
                            let h_pt = fp.height_twips as f32 / TWIPS_PER_PT;
                            let y_baseline_pt = page_h_pt - y_top_pt - h_pt * 0.8;
                            let (text, font_name, size_pt, _bold, _italic) =
                                flatten_paragraph_run(&h.content);
                            if text.is_empty() {
                                continue;
                            }
                            let resolved = resolve_font_for_text(&font_name, &text);
                            page = page
                                .font(&resolved, size_pt)
                                .at(x_pt, y_baseline_pt)
                                .text(&text);
                        }
                    },
                    Element::Image(img) => {
                        if let ImagePositioning::Floating(f) = &img.positioning {
                            if let Some(data) = img.data.as_ref() {
                                let x_pt = f.x_emu as f32 / EMU_PER_PT;
                                let w_pt = f.width_emu as f32 / EMU_PER_PT;
                                let h_im = f.height_emu as f32 / EMU_PER_PT;
                                // DOCX top-left → PDF bottom-left baseline.
                                let y_top_pt = f.y_emu as f32 / EMU_PER_PT;
                                let y_pt = page_h_pt - y_top_pt - h_im;
                                page = page
                                    .image_from_bytes(data, Rect::new(x_pt, y_pt, w_pt, h_im))
                                    .map_err(|e| {
                                        Error::InvalidOperation(format!("image float: {e}"))
                                    })?;
                            }
                        }
                    },
                    Element::Shape(shape) => {
                        page = render_shape(page, shape, page_h_pt);
                    },
                    _ => {},
                }
            }
            page.done();
        }
        Ok(())
    })?;

    builder
        .build()
        .map_err(|e| Error::InvalidOperation(format!("positional PDF build: {e}")))
}

/// Render a vector shape (line / rectangle) onto the current page at
/// its absolute coordinates. Coordinates arrive in DOCX/EMU with a
/// top-left origin and are converted to PDF's bottom-left baseline
/// using `page_h_pt`.
fn render_shape<'a>(
    page: FluentPageBuilder<'a>,
    shape: &Shape,
    page_h_pt: f32,
) -> FluentPageBuilder<'a> {
    use crate::writer::LineStyle;

    let x_pt = shape.x_emu as f32 / EMU_PER_PT;
    let y_top_pt = shape.y_emu as f32 / EMU_PER_PT;
    let w_pt = shape.width_emu as f32 / EMU_PER_PT;
    let h_pt = shape.height_emu as f32 / EMU_PER_PT;
    let y_pt = page_h_pt - y_top_pt - h_pt;

    let stroke_w = shape
        .stroke_w_emu
        .map(|w| (w as f32 / EMU_PER_PT).max(0.1))
        .unwrap_or(0.75);
    let stroke = shape
        .stroke_rgb
        .map(|[r, g, b]| (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
        .unwrap_or((0.0, 0.0, 0.0));

    match shape.kind {
        ShapeGeom::Line => page.stroke_line(
            x_pt,
            y_pt + h_pt,
            x_pt + w_pt,
            y_pt,
            LineStyle {
                width: stroke_w,
                color: stroke,
                dash: None,
            },
        ),
        ShapeGeom::Rect => {
            let mut p = page;
            if let Some([r, g, b]) = shape.fill_rgb {
                p = p.filled_rect(
                    x_pt,
                    y_pt,
                    w_pt,
                    h_pt,
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                );
            }
            if shape.stroke_rgb.is_some() {
                p = p.stroke_rect(
                    x_pt,
                    y_pt,
                    w_pt,
                    h_pt,
                    LineStyle {
                        width: stroke_w,
                        color: stroke,
                        dash: None,
                    },
                );
            }
            p
        },
    }
}

/// Reduce a paragraph's inline content to a single (text, font, size,
/// bold, italic) tuple — used by the positional renderer where each
/// paragraph is a single positioned text run.
fn flatten_paragraph_run(content: &[InlineContent]) -> (String, String, f32, bool, bool) {
    let mut text = String::new();
    let mut font_name = "Helvetica".to_string();
    let mut size_pt = 12.0_f32;
    let mut bold = false;
    let mut italic = false;
    let mut took_style = false;
    for ic in content {
        if let InlineContent::Text(span) = ic {
            text.push_str(&span.text);
            if !took_style {
                if let Some(name) = span.font_name.as_ref() {
                    font_name = name.clone();
                }
                if let Some(half_pt) = span.font_size_half_pt {
                    size_pt = half_pt as f32 / 2.0;
                }
                bold = span.bold;
                italic = span.italic;
                took_style = true;
            }
        }
    }
    (text, font_name, size_pt, bold, italic)
}

// ── IR → PDF rendering ────────────────────────────────────────────────────────

/// How many columns fit on the page at the StreamingTable minimum column
/// width (20 pt) given the current margins. Tables wider than this overflow
/// the right margin and lose data.
fn portrait_col_cap(config: &OfficeConfig) -> usize {
    const MIN_COL_PT: f32 = 20.0;
    let (w, _h) = config.page_size.dimensions();
    let usable = (w - config.margins.left - config.margins.right).max(0.0);
    (usable / MIN_COL_PT).floor() as usize
}

/// Does this section contain any table whose column count exceeds the
/// portrait page cap? Used to flip such sections to landscape so wide
/// spreadsheets render their full data instead of being silently
/// truncated by `render_table`.
fn section_needs_landscape(section: &Section, config: &OfficeConfig) -> bool {
    let cap = portrait_col_cap(config);
    section.elements.iter().any(|e| {
        if let Element::Table(t) = e {
            t.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0) > cap
        } else {
            false
        }
    })
}

/// Swap width/height of a `PageSize` to get its landscape form.
fn landscape(size: PageSize) -> PageSize {
    let (w, h) = size.dimensions();
    PageSize::Custom(h.max(w), w.min(h))
}

/// Convert a `DocumentIR` to PDF bytes using the FluentPageBuilder pipeline.
/// Public alias of the internal `ir_to_pdf_bytes` for diagnostic /
/// instrumentation use only — lets in-tree tools render an IR
/// straight to PDF, bypassing the PPTX hop.
pub fn ir_to_pdf_bytes_pub(ir: &DocumentIR, config: &OfficeConfig) -> Result<Vec<u8>> {
    ir_to_pdf_bytes(ir, config, &[])
}

fn ir_to_pdf_bytes(
    ir: &DocumentIR,
    config: &OfficeConfig,
    extra_fonts: &[(String, Vec<u8>)],
) -> Result<Vec<u8>> {
    use crate::writer::EmbeddedFont;
    let mut doc = DocumentBuilder::new().compress_streams(true);
    let mut registered: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Register embedded source-PDF fonts so flow-mode rendering can
    // resolve face names back to the original typeface program.
    // Same Unicode-cmap guard as `render_positional_ir` — registering
    // a CID-only subset routes every glyph through GID 0 and makes
    // text invisible.
    for (name, data) in extra_fonts {
        match EmbeddedFont::from_data(Some(name.clone()), data.clone()) {
            Ok(font) if font.has_usable_unicode_cmap() => {
                doc = doc.register_embedded_font(name.clone(), font);
                registered.insert(name.clone());
            },
            Ok(_) => {
                eprintln!(
                    "  [font] skipped {} ({} bytes): no Unicode cmap (CID-only subset)",
                    name,
                    data.len()
                );
            },
            Err(e) => {
                eprintln!("  [font] register failed: {} ({} bytes): {}", name, data.len(), e);
            },
        }
    }

    if let Some(ref t) = ir.metadata.title {
        doc = doc.title(t);
    }
    if let Some(ref a) = ir.metadata.author {
        doc = doc.author(a);
    }
    if let Some(ref s) = ir.metadata.subject {
        doc = doc.subject(s);
    }

    let unicode_fallback =
        if registered.contains(crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME) {
            Some(crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME.to_string())
        } else {
            None
        };
    let cjk_fallback =
        if registered.contains(crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME) {
            Some(crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME.to_string())
        } else {
            None
        };
    let result = with_registered_fonts_full(
        registered,
        unicode_fallback,
        cjk_fallback,
        || -> Result<Vec<u8>> {
            // Per-section page geometry: a section that carries its own
            // `page_setup` (e.g. PDF→PPTX→PDF where every slide knows its
            // source MediaBox) overrides the OfficeConfig default. This
            // keeps a 660-page Letter PDF round-tripping back to 660
            // Letter pages instead of overflowing onto the default size.
            let section_page_size = |section: &Section| -> PageSize {
                if let Some(ps) = section.page_setup.as_ref() {
                    let w_pt = ps.width_twips as f32 / TWIPS_PER_PT;
                    let h_pt = ps.height_twips as f32 / TWIPS_PER_PT;
                    if w_pt > 0.0 && h_pt > 0.0 {
                        return PageSize::Custom(w_pt, h_pt);
                    }
                }
                if section_needs_landscape(section, config) {
                    landscape(config.page_size)
                } else {
                    config.page_size
                }
            };

            let first_size = ir
                .sections
                .first()
                .map(section_page_size)
                .unwrap_or(config.page_size);
            let mut page = doc.page(first_size);
            let mut cur_size = first_size;

            for (si, section) in ir.sections.iter().enumerate() {
                let want_size = section_page_size(section);

                if si > 0 {
                    let force_break = !matches!(section.break_type, SectionBreakType::Continuous);
                    let size_changed = want_size.dimensions() != cur_size.dimensions();
                    if force_break || size_changed {
                        page = page.done().page(want_size);
                        cur_size = want_size;
                    }
                }
                // Multi-column sections (DOCX `<w:cols num="2">`,
                // pdf_to_ir's `detect_columns`) reflow paragraphs into N
                // bounded column rectangles and advance to the next
                // column when one fills. Without this guard a 2-column
                // arxiv-style paper rendered as a single narrow strip on
                // the left half of every page. Falls back to the flat
                // flow path for single-column sections.
                let col_count = section.columns.as_ref().map(|c| c.count).unwrap_or(1);
                if col_count >= 2 {
                    let (page_w_pt, page_h_pt) = match cur_size {
                        PageSize::Custom(w, h) => (w, h),
                        other => other.dimensions(),
                    };
                    page = render_section_columned(
                        page, section, col_count, page_w_pt, page_h_pt, config,
                    )?;
                } else {
                    let (_, page_h_pt) = match cur_size {
                        PageSize::Custom(w, h) => (w, h),
                        other => other.dimensions(),
                    };
                    // Walk floating images first to find the lowest
                    // bottom edge among any that anchor in the top half
                    // of the page. Once we know that, we'll snap the
                    // text cursor below it before emitting paragraphs.
                    // Without this, flow-mode body text starts at the
                    // page's top margin and lands on top of source
                    // top-of-page logos (UNIVERSITY OF ICELAND header,
                    // CFR shield, journal mastheads).
                    let top_image_floor = top_floating_image_floor(&section.elements, page_h_pt);
                    let mut text_cursor_pinned = false;
                    for element in &section.elements {
                        if let Some(floor_y) = top_image_floor {
                            let is_text_like = matches!(
                                element,
                                Element::Paragraph(_)
                                    | Element::Heading(_)
                                    | Element::List(_)
                                    | Element::Table(_)
                                    | Element::CodeBlock(_)
                            );
                            if is_text_like && !text_cursor_pinned {
                                // Push the cursor down to the bottom of
                                // the lowest top-of-page floating image
                                // (with a small visual gap) before flow
                                // text begins. PDF y-coords increase
                                // upward, so "lower on page" means
                                // smaller y. Only adjust when the image
                                // floor is actually below the current
                                // cursor.
                                let cur_y = page.cursor_y();
                                let new_y = floor_y - 6.0;
                                if new_y < cur_y && new_y > 0.0 {
                                    let cur_x = page.cursor_x();
                                    page = page.at(cur_x, new_y);
                                }
                                text_cursor_pinned = true;
                            }
                        }
                        page = render_ir_element(page, element, config)?;
                    }
                }
            }

            page.done();
            doc.build()
                .map_err(|e| Error::InvalidOperation(format!("PDF build: {e}")))
        },
    );
    result
}

/// Compute the lowest PDF-y bound (smallest y, since PDF y grows
/// upward) reached by any floating image that anchors in the top
/// half of the page. The flow renderer uses this to push the text
/// cursor below top-of-page logos / mastheads / headers — without
/// it, source PDFs that put a banner image at the page top
/// round-trip with body text overflowing into the banner area.
///
/// Returns `None` when no floating image is in the top half;
/// returns `Some(y_pdf)` (the bottom edge of the lowest such image
/// in PDF bottom-up coords) otherwise.
fn top_floating_image_floor(elements: &[Element], page_h_pt: f32) -> Option<f32> {
    let half_page = page_h_pt * 0.5;
    let mut floor: Option<f32> = None;
    for el in elements {
        let img = match el {
            Element::Image(img) => img,
            _ => continue,
        };
        let f = match &img.positioning {
            ImagePositioning::Floating(f) => f,
            _ => continue,
        };
        let y_top_pt = f.y_emu as f32 / EMU_PER_PT;
        let h_pt = f.height_emu as f32 / EMU_PER_PT;
        // PDF coords: image top in PDF y = page_h - y_top_pt.
        // Image bottom in PDF y = page_h - y_top_pt - h_pt.
        let pdf_top = page_h_pt - y_top_pt;
        let pdf_bottom = pdf_top - h_pt;
        // "Top half of the page" = PDF y above the page midline.
        // For an image whose top edge is in the upper half, we
        // assume it's a banner/logo that body text should sit below.
        if pdf_top >= half_page {
            floor = match floor {
                Some(prev) => Some(prev.min(pdf_bottom)),
                None => Some(pdf_bottom),
            };
        }
    }
    floor
}

/// Render a multi-column section. Lays paragraphs and headings into
/// `n` column rectangles (split horizontally with a gutter) by
/// estimating each block's wrapped height up front and advancing the
/// column cursor downward. When a block doesn't fit in the current
/// column the cursor moves to the next column on the same page; once
/// the rightmost column fills, a new page begins and the cursor
/// resets to the leftmost column.
///
/// First-cut scope: handles `Heading`, `Paragraph`, `Image`, and
/// `ColumnBreak` / `PageBreak`. Tables, lists, and code blocks fall
/// through to a no-op for now — they're rare in the multi-column
/// scientific-paper / news-article sources this exists to fix, and
/// adding column-aware width to them is a separate effort.
fn render_section_columned<'a>(
    mut page: FluentPageBuilder<'a>,
    section: &Section,
    col_count: u32,
    page_w_pt: f32,
    page_h_pt: f32,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    use crate::writer::TextAlign;

    let n = col_count.max(1) as usize;

    // Margins: prefer per-section page_setup (twips) when present, else
    // fall back to the OfficeConfig defaults the flow renderer uses.
    let (margin_l, margin_r, margin_t, margin_b) = section
        .page_setup
        .as_ref()
        .map(|ps| {
            (
                ps.margin_left_twips as f32 / TWIPS_PER_PT,
                ps.margin_right_twips as f32 / TWIPS_PER_PT,
                ps.margin_top_twips as f32 / TWIPS_PER_PT,
                ps.margin_bottom_twips as f32 / TWIPS_PER_PT,
            )
        })
        .unwrap_or((
            config.margins.left,
            config.margins.right,
            config.margins.top,
            config.margins.bottom,
        ));

    // Column-gutter from `Section.columns.space_twips` when supplied,
    // else 18pt (~ 1/4 inch) — Word's default `<w:cols space=...>`
    // hovers around this when authors don't override it.
    let gutter = section
        .columns
        .as_ref()
        .and_then(|c| c.space_twips)
        .map(|sp| sp as f32 / TWIPS_PER_PT)
        .unwrap_or(18.0);

    let usable_w = (page_w_pt - margin_l - margin_r).max(1.0);
    let total_gutter = gutter * (n as f32 - 1.0).max(0.0);
    let col_w = ((usable_w - total_gutter) / n as f32).max(1.0);

    let usable_top = page_h_pt - margin_t;
    let usable_bottom = margin_b;

    let column_x = |c: usize| margin_l + (col_w + gutter) * c as f32;

    let mut cur_col = 0usize;
    let mut cursor_y = usable_top;

    for el in &section.elements {
        match el {
            Element::PageBreak => {
                page = page.new_page_same_size();
                cur_col = 0;
                cursor_y = usable_top;
                continue;
            },
            Element::ColumnBreak => {
                cur_col += 1;
                if cur_col >= n {
                    page = page.new_page_same_size();
                    cur_col = 0;
                }
                cursor_y = usable_top;
                continue;
            },
            _ => {},
        }

        // Plain (text, font, size, alignment) tuple. None means we
        // don't know how to size this element in column flow; fall
        // through to the flat renderer so it still emits something.
        let block: Option<(String, f32, String, TextAlign)> = match el {
            Element::Heading(h) => {
                let t = inline_content_to_text(&h.content);
                if t.trim().is_empty() {
                    None
                } else {
                    let sz = match h.level {
                        1 => 18.0,
                        2 => 16.0,
                        3 => 14.0,
                        _ => 12.0,
                    };
                    let align = match h.alignment {
                        Some(ParagraphAlignment::Center) => TextAlign::Center,
                        Some(ParagraphAlignment::Right) => TextAlign::Right,
                        _ => TextAlign::Left,
                    };
                    let font_owned = first_inline_font_name(&h.content)
                        .map(|n| resolve_font_for_text(&n, &t))
                        .unwrap_or_else(|| resolve_font_for_text("Helvetica-Bold", &t));
                    Some((t, sz, font_owned, align))
                }
            },
            Element::Paragraph(p) => {
                let t = inline_content_to_text(&p.content);
                if t.trim().is_empty() {
                    None
                } else {
                    let sz = office_oxide::ir::first_inline_font_size_pt(&p.content)
                        .unwrap_or(config.default_font_size);
                    let bold = p.content.iter().any(|ic| match ic {
                        InlineContent::Text(s) => s.bold,
                        _ => false,
                    });
                    let italic = p.content.iter().any(|ic| match ic {
                        InlineContent::Text(s) => s.italic,
                        _ => false,
                    });
                    let default = match (bold, italic) {
                        (true, true) => "Helvetica-BoldOblique",
                        (true, false) => "Helvetica-Bold",
                        (false, true) => "Helvetica-Oblique",
                        (false, false) => "Helvetica",
                    };
                    let font_owned = first_inline_font_name(&p.content)
                        .map(|n| resolve_font_for_text(&n, &t))
                        .unwrap_or_else(|| resolve_font_for_text(default, &t));
                    let align = match p.alignment {
                        Some(ParagraphAlignment::Center) => TextAlign::Center,
                        Some(ParagraphAlignment::Right) => TextAlign::Right,
                        Some(ParagraphAlignment::Justify) => TextAlign::Left,
                        _ => TextAlign::Left,
                    };
                    Some((t, sz, font_owned, align))
                }
            },
            Element::Image(img) => {
                if let Some(ref data) = img.data {
                    if !data.is_empty() {
                        // Size: prefer the IR's intrinsic display size in
                        // EMU; cap to column width so figures don't burst
                        // out of their column. Aspect ratio preserved.
                        let cx_pt = img
                            .display_width_emu
                            .map(|w| w as f32 / EMU_PER_PT)
                            .unwrap_or(col_w);
                        let cy_pt = img
                            .display_height_emu
                            .map(|h| h as f32 / EMU_PER_PT)
                            .unwrap_or(col_w * 0.75);
                        let scale = (col_w / cx_pt.max(1.0)).min(1.0);
                        let draw_w = cx_pt * scale;
                        let draw_h = cy_pt * scale;
                        // Advance column if it doesn't fit.
                        if cursor_y - draw_h < usable_bottom {
                            cur_col += 1;
                            if cur_col >= n {
                                page = page.new_page_same_size();
                                cur_col = 0;
                            }
                            cursor_y = usable_top;
                        }
                        let rect = Rect::new(column_x(cur_col), cursor_y - draw_h, draw_w, draw_h);
                        page = page.image_from_bytes(data, rect).map_err(|e| {
                            Error::InvalidOperation(format!("image_from_bytes (column flow): {e}"))
                        })?;
                        cursor_y -= draw_h + 4.0;
                    }
                }
                continue;
            },
            _ => None,
        };

        let (text, size_pt, font, align) = match block {
            Some(b) => b,
            None => continue,
        };

        let line_h = size_pt * config.line_height;
        let lines = wrap_estimate(&text, col_w, size_pt);
        let block_h = line_h * lines.max(1) as f32;

        // If the paragraph won't fit in the current column, advance.
        if cursor_y - block_h < usable_bottom {
            cur_col += 1;
            if cur_col >= n {
                page = page.new_page_same_size();
                cur_col = 0;
            }
            cursor_y = usable_top;

            // Edge case: a paragraph taller than a full column. Render
            // anyway at the top of the new column; subsequent text
            // will continue past it (text_in_rect already does best-
            // effort wrapping inside the rect — overflow is silently
            // truncated by the layout engine, matching how Word
            // handles oversized blocks at column boundaries).
        }

        let rect = Rect::new(column_x(cur_col), cursor_y - block_h, col_w, block_h);
        page = page.font(&font, size_pt);
        page = page.text_in_rect(rect, &text, align);
        cursor_y -= block_h;
    }

    Ok(page)
}

fn render_ir_element<'a>(
    page: FluentPageBuilder<'a>,
    element: &Element,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    match element {
        Element::Heading(h) => Ok(render_heading(page, h)),
        Element::Paragraph(p) => Ok(render_paragraph(page, p)),
        Element::List(l) => Ok(render_list(page, l)),
        Element::Table(t) => Ok(render_table(page, t, config)),
        Element::Image(img) => render_image(page, img, config),
        Element::ThematicBreak => Ok(page.horizontal_rule()),
        Element::PageBreak => Ok(page.new_page_same_size()),
        Element::ColumnBreak => Ok(page),
        Element::CodeBlock(cb) => Ok(render_code_block(page, cb)),
        Element::TextBox(tb) => render_text_box(page, tb, config),
        Element::Footnote(n) | Element::Endnote(n) => {
            let text = note_to_text(&n.content);
            let marker = n.marker.clone().unwrap_or_else(|| n.id.to_string());
            Ok(page.footnote(&marker, &text))
        },
        _ => Ok(page),
    }
}

fn render_heading<'a>(page: FluentPageBuilder<'a>, h: &Heading) -> FluentPageBuilder<'a> {
    // Honour the source's per-run font size when present. The fixed
    // 24/20/16/14 pt scale baked into `FluentPageBuilder::heading()`
    // is correct only when the source carried no size override; for
    // PDF→IR conversions where the run carries an explicit `<w:sz>`,
    // forcing the level-default would silently drop a 40-pt title
    // down to 20 pt.
    let text = inline_content_to_text(&h.content);
    if text.is_empty() {
        return page;
    }
    // Prefer the heading's *own* source font_name (via base-14
    // mapping) over the page's currently-bound font. Without this,
    // a serif heading after a Helvetica paragraph would render in
    // Helvetica-Bold instead of Times-Bold — wrong family.
    let bold_font = first_inline_font_name(&h.content)
        .map(|n| {
            let resolved = resolve_font_for_text(&n, &text);
            match resolved.as_str() {
                "Helvetica" | "Helvetica-Oblique" => "Helvetica-Bold".to_string(),
                "Times-Roman" | "Times-Italic" => "Times-Bold".to_string(),
                "Courier" | "Courier-Oblique" => "Courier-Bold".to_string(),
                other => other.to_string(),
            }
        })
        .unwrap_or_else(|| {
            let cur_font = page.text_config_font_name().to_string();
            let cur_resolved = resolve_font_for_text(&cur_font, &text);
            match cur_resolved.as_str() {
                "Helvetica" | "Helvetica-Oblique" => "Helvetica-Bold".to_string(),
                "Times-Roman" | "Times-Italic" => "Times-Bold".to_string(),
                "Courier" | "Courier-Oblique" => "Courier-Bold".to_string(),
                other => other.to_string(),
            }
        });
    let size_pt =
        office_oxide::ir::first_inline_font_size_pt(&h.content).unwrap_or(match h.level {
            1 => 24.0,
            2 => 20.0,
            3 => 16.0,
            _ => 14.0,
        });
    let page = page.font(&bold_font, size_pt);

    // Honour `h.alignment` so a centred / right-aligned source
    // heading survives the round-trip. `paragraph()` only does
    // left-alignment; non-Left alignments need to drop down to
    // `text_in_rect` (same approach as `render_paragraph`).
    use crate::writer::TextAlign;
    use office_oxide::ir::ParagraphAlignment;
    if matches!(h.alignment, Some(ParagraphAlignment::Center) | Some(ParagraphAlignment::Right)) {
        let cursor_x = page.cursor_x();
        let cursor_y = page.cursor_y();
        let page_w = page.page_width();
        let right_margin = 72.0_f32;
        let usable_w = (page_w - cursor_x - right_margin).max(1.0);
        let line_h = size_pt * page.text_config_line_height();
        let est_chars_per_line = (usable_w / (size_pt * 0.5)).max(1.0) as usize;
        let lines = text.chars().count().max(1).div_ceil(est_chars_per_line);
        let block_h = line_h * lines.max(1) as f32;
        let rect = Rect::new(cursor_x, cursor_y - block_h, usable_w, block_h);
        let align = match h.alignment {
            Some(ParagraphAlignment::Center) => TextAlign::Center,
            Some(ParagraphAlignment::Right) => TextAlign::Right,
            _ => TextAlign::Left,
        };
        // `text_in_rect` only emits text — it does NOT advance the
        // cursor. Without an explicit `set_cursor_y` call afterwards
        // every centred / right-aligned block draws on top of the
        // previous one (verified visually on the CFR cover page).
        let mut page = page.text_in_rect(rect, &text, align);
        let new_y = cursor_y - block_h - line_h * 0.5;
        page.set_cursor_y(new_y);
        return page;
    }
    page.paragraph(&text)
}

/// First non-empty `font_name` field on any `Text` inline run.
/// Mirrors `office_oxide::ir::first_inline_font_size_pt` but for the
/// face name. None means the caller should keep its current font.
fn first_inline_font_name(content: &[InlineContent]) -> Option<String> {
    for ic in content {
        if let InlineContent::Text(span) = ic {
            if let Some(name) = span.font_name.as_ref() {
                if !name.is_empty() {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Map a source-PDF font name to its closest PDF base-14 family.
///
/// Returns `None` when the source name is unrecognised (caller should
/// pass it through to the renderer as-is — if registered under that
/// name as an embedded program, it will resolve directly; otherwise
/// the writer's own fallback chain takes over). When recognised, the
/// returned `&'static str` is one of the 14 standard PDF font names
/// the renderer guarantees to have available.
///
/// Used so source PDFs whose fonts can't be re-embedded (Type 1
/// LaTeX programs, CFF subsets without a Unicode cmap) still pick a
/// metrically and visually closer fallback than the blanket
/// "Helvetica regardless" default — Times for serif sources, Courier
/// for monospace, Helvetica only for sans-serif sources.
fn map_to_base14(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    // newtx math italic uses the convention `NewTXMI` (math italic),
    // `NewTXBMI` (bold math italic) — the bold marker is the leading
    // `B` in `BMI`, not a "bold" substring. Detect explicitly.
    let newtx_bold_mi = (lower.starts_with("newtxbmi") || lower.starts_with("newtxb"))
        || lower.contains("-newtxbmi");
    let bold = lower.contains("bold")
        || lower.contains("-bd")
        || lower.contains("medium")
        || lower.contains("black")
        || lower.contains("heavy")
        || lower.contains("demi")
        || newtx_bold_mi;
    // newtx math italic implies italic regardless of substring; same
    // for `*MI` / `*-MI` shapes used by LaTeX math italic packages.
    let newtx_mi = lower.starts_with("newtxmi")
        || lower.starts_with("newtxbmi")
        || lower.contains("-newtxmi")
        || lower.contains("-newtxbmi");
    let math_italic_shape = lower.contains("txmia")
        || lower.contains("-mi")
        || lower.ends_with("mi")
        || lower.contains("mathitalic")
        || lower.contains("math-italic");
    let italic = lower.contains("italic")
        || lower.contains("oblique")
        || lower.contains("-it")
        || lower.ends_with("-i")
        || newtx_mi
        || math_italic_shape;
    // Math symbol fonts (LaTeX `txsy`, `txex`, `cmsy`, `cmex`,
    // `msam`/`msbm`, `mathsymb`*) carry mathematical operator and
    // delimiter glyphs that overlap PDF's Symbol font. Map them
    // there so the round-trip at least uses the correct glyph set
    // instead of Helvetica's missing-glyph squares.
    if lower.contains("symbol")
        || lower.starts_with("txsy")
        || lower.starts_with("txex")
        || lower.starts_with("cmsy")
        || lower.starts_with("cmex")
        || lower.starts_with("msam")
        || lower.starts_with("msbm")
        || lower.contains("mathsymbols")
        || lower.contains("mathoperator")
    {
        return Some("Symbol");
    }
    if lower.contains("zapfdingbats") || lower.contains("dingbats") {
        return Some("ZapfDingbats");
    }
    let mono = lower.contains("courier")
        || lower.contains("nimbusmon")
        || lower.contains("texgyrecursor")
        || lower.contains("cmtt")
        || lower.contains("cmtex")
        || lower.contains("lmmono")
        || lower.contains("latinmodernmono")
        || lower.contains("typewriter")
        || lower.contains("monospace")
        || lower.contains("mono ");
    let serif = if mono {
        false
    } else {
        {
            lower.contains("times")
            || lower.contains("nimbusrom")
            || lower.contains("texgyretermes") || lower.contains("texgyrepagella")
            || lower.contains("texgyrebonum") || lower.contains("texgyreschola")
            || lower.starts_with("cmr") || lower.starts_with("cmb") || lower.starts_with("cmm")
            || lower.starts_with("cmsl") || lower.starts_with("cmti")
            || lower.contains("lmroman") || lower.contains("latinmodernroman")
            || lower.contains("lmmath") || lower.contains("latinmodernmath")
            || lower.contains("stix") || lower.contains("xits")
            // newtx math italic (NewTXMI, NewTXBMI, txmia, etc.)
            // is a Times-Italic-shaped math italic font. Maps to
            // the Times family so inline math italics in LaTeX
            // papers don't fall back to Helvetica.
            || lower.starts_with("newtx") || lower.starts_with("newpx")
            || lower.starts_with("txmia") || lower.starts_with("txmi")
            || lower.contains("newcenturyschlbk") || lower.contains("schoolbook")
            || lower.contains("garamond") || lower.contains("palatino")
            || lower.contains("bookman") || lower.contains("georgia")
            || lower.contains("serif")
            || lower.starts_with("rm") // generic LaTeX serif default
        }
    };
    let sans = !mono
        && !serif
        && (
            lower.contains("helvetica")
                || lower.contains("arial")
                || lower.contains("nimbussan")
                || lower.contains("texgyreheros")
                || lower.contains("avantgarde")
                || lower.contains("avant garde")
                || lower.contains("cmss")
                || lower.contains("lmsans")
                || lower.contains("latinmodernsans")
                || lower.contains("verdana")
                || lower.contains("tahoma")
                || lower.contains("sans")
                || lower.starts_with("sf")
            // generic LaTeX sans default
        );
    let family = if mono {
        "Courier"
    } else if serif {
        "Times"
    } else if sans {
        "Helvetica"
    } else {
        return None;
    };
    Some(match (family, bold, italic) {
        ("Times", false, false) => "Times-Roman",
        ("Times", true, false) => "Times-Bold",
        ("Times", false, true) => "Times-Italic",
        ("Times", true, true) => "Times-BoldItalic",
        ("Helvetica", false, false) => "Helvetica",
        ("Helvetica", true, false) => "Helvetica-Bold",
        ("Helvetica", false, true) => "Helvetica-Oblique",
        ("Helvetica", true, true) => "Helvetica-BoldOblique",
        ("Courier", false, false) => "Courier",
        ("Courier", true, false) => "Courier-Bold",
        ("Courier", false, true) => "Courier-Oblique",
        ("Courier", true, true) => "Courier-BoldOblique",
        _ => "Helvetica",
    })
}

/// Resolve the renderer-facing font name for a source PDF face.
///
/// When the source name matches a registered embedded font we keep
/// it (preserves the original typeface). When it doesn't — typical
/// for Type 1 / CFF subsets that fail the writer's cmap guard — we
/// map by family heuristic to a closer base-14 (`Times*` for serif
/// sources, `Courier*` for mono, `Helvetica*` for sans). Without
/// this, every unembeddable source falls back to Helvetica
/// regardless of source family, producing a noticeable typeface
/// mismatch on serif-heavy documents (academic papers, federal
/// regs).
///
/// The registered-font set is set per-conversion via
/// [`with_registered_fonts`] so the deeply-nested render helpers
/// don't need an extra parameter on every call.
fn resolve_font_name(source_name: &str) -> String {
    if source_name.is_empty() {
        return source_name.to_string();
    }
    let registered_match = REGISTERED_FONTS.with(|cell| {
        cell.borrow()
            .as_ref()
            .is_some_and(|set| set.contains(source_name))
    });
    if registered_match {
        return source_name.to_string();
    }
    if let Some(b14) = map_to_base14(source_name) {
        return b14.to_string();
    }
    source_name.to_string()
}

thread_local! {
    /// Per-conversion set of font names that the writer accepted as
    /// real embedded programs. Populated at the top of
    /// `ir_to_pdf_bytes` / `render_pptx_positional` and consulted by
    /// `resolve_font_name` so unembeddable face names map to a closer
    /// base-14 family while embeddable ones keep their source name.
    static REGISTERED_FONTS: std::cell::RefCell<Option<std::collections::HashSet<String>>>
        = const { std::cell::RefCell::new(None) };

    /// Name under which a Unicode-capable system font was registered
    /// for this conversion (or `None` if no such font was loaded /
    /// no non-Latin text was found). Consulted by
    /// `resolve_font_for_text` to route Hebrew / Arabic / Latin
    /// Extended spans away from base-14 fonts that can't render them.
    static UNICODE_FALLBACK_FONT: std::cell::RefCell<Option<String>>
        = const { std::cell::RefCell::new(None) };

    /// Name under which a CJK-capable system font was registered for
    /// this conversion (or `None` if no CJK text was found / no CJK
    /// font is available on the system). The CJK fallback is
    /// separate from the general Unicode fallback because typical
    /// Unicode-capable fonts (DejaVu Sans / FreeSans) don't include
    /// Han / Hiragana / Hangul glyphs — Chinese / Japanese / Korean
    /// text needs to route to a dedicated CJK face.
    static UNICODE_FALLBACK_CJK_FONT: std::cell::RefCell<Option<String>>
        = const { std::cell::RefCell::new(None) };
}

/// Run `body` with the full font-resolution context: registered
/// embedded faces, plus optional Unicode + CJK fallback names. Used
/// to scope `resolve_font_name` / `resolve_font_for_text` answers to
/// one PDF render pass.
fn with_registered_fonts_full<R>(
    set: std::collections::HashSet<String>,
    unicode_fallback: Option<String>,
    cjk_fallback: Option<String>,
    body: impl FnOnce() -> R,
) -> R {
    REGISTERED_FONTS.with(|cell| cell.replace(Some(set)));
    UNICODE_FALLBACK_FONT.with(|cell| cell.replace(unicode_fallback));
    UNICODE_FALLBACK_CJK_FONT.with(|cell| cell.replace(cjk_fallback));
    let r = body();
    REGISTERED_FONTS.with(|cell| cell.replace(None));
    UNICODE_FALLBACK_FONT.with(|cell| cell.replace(None));
    UNICODE_FALLBACK_CJK_FONT.with(|cell| cell.replace(None));
    r
}

/// Resolve the font name for a span carrying `text`. When the text
/// contains characters outside the base-14 Latin-1 range AND a
/// Unicode-capable fallback was registered for this conversion,
/// route the span to that font; otherwise fall back to the
/// family-aware [`resolve_font_name`] mapping.
///
/// Without this, Hebrew / Arabic / Latin Extended source text would
/// route through Helvetica / Times — neither covers those Unicode
/// blocks, so the renderer emits `.notdef` (`?` or missing-glyph
/// boxes) for every non-Latin character. With it, the span draws
/// in the system Unicode font registered by [`maybe_load_unicode_fallback`].
fn resolve_font_for_text(source_name: &str, text: &str) -> String {
    // CJK first: a CJK-capable face also covers Latin, but the
    // general Unicode fallback (DejaVu Sans / FreeSans) doesn't
    // cover Han / Hiragana / Hangul. Route any text with CJK
    // codepoints to the dedicated CJK fallback if one was loaded.
    if crate::fonts::unicode_fallback::needs_cjk_fallback(text) {
        let fallback = UNICODE_FALLBACK_CJK_FONT.with(|cell| cell.borrow().clone());
        if let Some(name) = fallback {
            return name;
        }
    }
    if crate::fonts::unicode_fallback::needs_unicode_fallback(text) {
        let fallback = UNICODE_FALLBACK_FONT.with(|cell| cell.borrow().clone());
        if let Some(name) = fallback {
            return name;
        }
    }
    resolve_font_name(source_name)
}

/// If the IR carries any text outside the base-14 Latin-1 range AND
/// a system Unicode font is available, load it once and append to
/// `extra_fonts` under [`crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME`].
/// Returns the name to advertise to `resolve_font_for_text`
/// (or `None` if no non-Latin text was found / no font could be
/// loaded).
fn maybe_load_unicode_fallback(
    ir: &DocumentIR,
    extra_fonts: &mut Vec<(String, Vec<u8>)>,
) -> Option<String> {
    if !ir_has_non_latin_text(ir) {
        return None;
    }
    let bytes = crate::fonts::unicode_fallback::load_unicode_fallback_bytes()?;
    let name = crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME.to_string();
    // Avoid double-registering if the caller already added it.
    if !extra_fonts.iter().any(|(n, _)| n == &name) {
        extra_fonts.push((name.clone(), bytes));
    }
    Some(name)
}

/// If the IR contains CJK text AND a system CJK font is available,
/// load it once and append to `extra_fonts` under
/// [`crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME`].
/// Returns the name to advertise to `resolve_font_for_text` (or
/// `None` if no CJK text was found / no CJK font is available).
fn maybe_load_cjk_fallback(
    ir: &DocumentIR,
    extra_fonts: &mut Vec<(String, Vec<u8>)>,
) -> Option<String> {
    if !ir_has_cjk_text(ir) {
        return None;
    }
    let bytes = crate::fonts::unicode_fallback::load_cjk_fallback_bytes()?;
    let name = crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME.to_string();
    if !extra_fonts.iter().any(|(n, _)| n == &name) {
        extra_fonts.push((name.clone(), bytes));
    }
    Some(name)
}

/// Recurse into the IR's text-bearing elements, return `true` as
/// soon as any character code-point exceeds the base-14 Latin-1
/// range. Cheap — short-circuits on the first hit.
fn ir_has_non_latin_text(ir: &DocumentIR) -> bool {
    for section in &ir.sections {
        if section_has_non_latin(&section.elements) {
            return true;
        }
    }
    false
}

fn section_has_non_latin(elements: &[Element]) -> bool {
    for el in elements {
        if element_has_non_latin(el) {
            return true;
        }
    }
    false
}

fn element_has_non_latin(el: &Element) -> bool {
    match el {
        Element::Paragraph(p) => inline_has_non_latin(&p.content),
        Element::Heading(h) => inline_has_non_latin(&h.content),
        Element::List(l) => l.items.iter().any(|it| section_has_non_latin(&it.content)),
        Element::Table(t) => t.rows.iter().any(|row| {
            row.cells
                .iter()
                .any(|cell| section_has_non_latin(&cell.content))
        }),
        Element::TextBox(tb) => section_has_non_latin(&tb.content),
        _ => false,
    }
}

fn inline_has_non_latin(content: &[InlineContent]) -> bool {
    content.iter().any(|ic| match ic {
        InlineContent::Text(s) => crate::fonts::unicode_fallback::needs_unicode_fallback(&s.text),
        _ => false,
    })
}

/// Mirror of [`ir_has_non_latin_text`] for the CJK heuristic. Walks
/// the same element tree and returns `true` as soon as any text
/// span contains a CJK codepoint. Used to gate loading the larger
/// CJK fallback font.
fn ir_has_cjk_text(ir: &DocumentIR) -> bool {
    fn section(elements: &[Element]) -> bool {
        elements.iter().any(element)
    }
    fn element(el: &Element) -> bool {
        match el {
            Element::Paragraph(p) => inline(&p.content),
            Element::Heading(h) => inline(&h.content),
            Element::List(l) => l.items.iter().any(|it| section(&it.content)),
            Element::Table(t) => t
                .rows
                .iter()
                .any(|row| row.cells.iter().any(|cell| section(&cell.content))),
            Element::TextBox(tb) => section(&tb.content),
            _ => false,
        }
    }
    fn inline(content: &[InlineContent]) -> bool {
        content.iter().any(|ic| match ic {
            InlineContent::Text(s) => crate::fonts::unicode_fallback::needs_cjk_fallback(&s.text),
            _ => false,
        })
    }
    ir.sections.iter().any(|s| section(&s.elements))
}

fn render_paragraph<'a>(page: FluentPageBuilder<'a>, p: &Paragraph) -> FluentPageBuilder<'a> {
    // PPTX-encoded ThematicBreak: paragraph whose content is one or
    // more runs of U+2500 only. office_oxide emits ThematicBreak
    // through PPTX as a centered paragraph of box-drawing horizontal
    // characters because PPTX has no <a:pPr>-level border syntax
    // analogous to DOCX <w:pBdr>. Recover the rule here so it
    // renders as a real horizontal_rule() instead of literal glyphs.
    {
        let only_thematic_chars = !p.content.is_empty()
            && p.content.iter().all(|ic| match ic {
                InlineContent::Text(s) => {
                    !s.text.is_empty() && s.text.chars().all(|c| c == '\u{2500}')
                },
                _ => false,
            });
        if only_thematic_chars {
            return page.horizontal_rule();
        }
    }
    let runs = inline_content_to_runs(&p.content);
    let mut page = page;
    // Honour `space_before_twips` so source-PDF cover pages where
    // paragraphs are distributed across the page height with large
    // vertical gaps survive the round-trip with similar gaps. Without
    // this, the renderer defaults to a single-line gap between
    // paragraphs and crams cover content tightly under the banner.
    if let Some(twips) = p.space_before_twips {
        if twips > 0 {
            let pt = twips as f32 / TWIPS_PER_PT;
            let cur_y = page.cursor_y();
            let new_y = cur_y - pt;
            if new_y > 0.0 {
                page.set_cursor_y(new_y);
            }
        }
    }
    if runs.is_empty() {
        // Empty paragraph with a bottom border = horizontal rule
        // (the conventional DOCX representation of an `<hr/>`).
        // pdf_to_ir's ThematicBreak round-trips through DOCX as
        // exactly this shape; recover it here so the rendered PDF
        // shows a visible rule instead of silently swallowing it.
        if let Some(ref border) = p.border {
            if border.bottom.is_some() {
                return page.horizontal_rule();
            }
        }
        // Empty paragraphs without a border serve as vertical
        // spacers (used by pdf_to_ir between cover-page paragraphs
        // that have a large y-gap in the source). Advance the
        // cursor by a single line height so they actually
        // contribute vertical space; without this they're silent
        // no-ops.
        let line_h = page.text_config_font_size() * page.text_config_line_height();
        let cur_y = page.cursor_y();
        let new_y = cur_y - line_h;
        if new_y > 0.0 {
            page.set_cursor_y(new_y);
        }
        return page;
    }
    // Honour the IR's per-paragraph font name and size. Without
    // pulling the font name forward from the IR span, every
    // paragraph falls back to the page builder's current font
    // (Helvetica), so PDF→DOCX→PDF round-trips lose every
    // typeface — TeXGyreTermesX, NimbusSanL, NewCenturySchlbk —
    // even though the source-PDF font program is embedded in the
    // PDF under the same face name. Pairing the embedded-font
    // registration in `ir_to_pdf_bytes` with this per-paragraph
    // `font(...)` call is what actually makes the embed reachable.
    let span_size_pt = office_oxide::ir::first_inline_font_size_pt(&p.content);
    let span_font_name = first_inline_font_name(&p.content);
    if span_size_pt.is_some() || span_font_name.is_some() {
        let size_pt = span_size_pt.unwrap_or_else(|| page.text_config_font_size());
        let body_text = inline_content_to_text(&p.content);
        let face = span_font_name
            .map(|n| resolve_font_for_text(&n, &body_text))
            .unwrap_or_else(|| {
                let cur = page.text_config_font_name().to_string();
                resolve_font_for_text(&cur, &body_text)
            });
        page = page.font(&face, size_pt);
    }

    // Centered or right-aligned paragraphs need explicit positioning
    // — `paragraph()` / `rich_paragraph()` only do left alignment.
    // Drop down to `text_in_rect` with the page's full width minus
    // current margins so the source's centered title pages survive
    // the round-trip.
    use crate::writer::TextAlign;
    use office_oxide::ir::ParagraphAlignment;
    if matches!(p.alignment, Some(ParagraphAlignment::Center) | Some(ParagraphAlignment::Right)) {
        let text = runs.iter().map(|r| r.text.as_str()).collect::<String>();
        if !text.is_empty() {
            let cursor_x = page.cursor_x();
            let cursor_y = page.cursor_y();
            let page_w = page.page_width();
            let right_margin = 72.0_f32;
            let usable_w = (page_w - cursor_x - right_margin).max(1.0);
            let font_size = page.text_config_font_size();
            let line_h = font_size * page.text_config_line_height();
            // Estimate line count for height; text_in_rect wraps inside.
            let est_chars_per_line = (usable_w / (font_size * 0.5)).max(1.0) as usize;
            let lines = text.chars().count().max(1).div_ceil(est_chars_per_line);
            let block_h = line_h * lines.max(1) as f32;
            let rect = Rect::new(cursor_x, cursor_y - block_h, usable_w, block_h);
            let align = match p.alignment {
                Some(ParagraphAlignment::Center) => TextAlign::Center,
                Some(ParagraphAlignment::Right) => TextAlign::Right,
                _ => TextAlign::Left,
            };
            // `text_in_rect` is a positional emit and doesn't move
            // the cursor — without this manual advance the next
            // paragraph would draw on top of the previous one.
            let mut page = page.text_in_rect(rect, &text, align);
            let new_y = cursor_y - block_h - line_h * 0.5;
            page.set_cursor_y(new_y);
            return page;
        }
    }

    if runs.len() == 1 && runs[0].style == TextRunStyle::Normal {
        return page.paragraph(&runs[0].text.clone());
    }
    // `rich_paragraph` already finishes with a full line-height advance
    // plus a 0.5-line trailing gap. Chaining `.newline()` (one more
    // full line-height advance) inserts a 1.5-line dead zone between
    // every paragraph — over a long section that compounds into ~1
    // extra page per ~30 paragraphs and was a major contributor to
    // round-trip page-count inflation.
    page.rich_paragraph(&runs)
}

fn render_list<'a>(page: FluentPageBuilder<'a>, list: &List) -> FluentPageBuilder<'a> {
    let items: Vec<String> = list
        .items
        .iter()
        .map(|item| elements_to_text(&item.content))
        .collect();
    if items.is_empty() {
        return page;
    }
    if list.ordered {
        let style = match list.style.as_ref() {
            Some(IrListStyle::LowerRoman) | Some(IrListStyle::UpperRoman) => ListStyle::RomanLower,
            Some(IrListStyle::LowerAlpha) | Some(IrListStyle::UpperAlpha) => ListStyle::AlphaLower,
            _ => ListStyle::Decimal,
        };
        page.numbered_list(&items, style)
    } else {
        page.bullet_list(&items)
    }
}

fn render_table<'a>(
    page: FluentPageBuilder<'a>,
    t: &IrTable,
    config: &OfficeConfig,
) -> FluentPageBuilder<'a> {
    if t.rows.is_empty() {
        return page;
    }

    // What's the maximum column count across all rows? Title rows can be
    // narrower than data rows, so taking max is correct.
    let total_cols = t.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
    if total_cols == 0 {
        return page;
    }

    // Cap columns to those that physically fit on the page at minimum width
    // (20 pt). We query the *actual* page (which may be landscape if
    // `ir_to_pdf_bytes` flipped this section) rather than `config.page_size`,
    // so wide tables get the larger cap automatically.
    const MIN_COL_PT: f32 = 20.0;
    let (page_w, _) = page.page_dimensions();
    let usable_w = page_w - config.margins.left - config.margins.right;
    let max_cols = (usable_w / MIN_COL_PT).floor() as usize;

    // If the table fits on one page width, render it as a single chunk.
    // Otherwise split columns into chunks of `max_cols` and emit each chunk
    // as its own sub-table on a fresh page sequence. The first column (often
    // a row key/identifier) is repeated as the leftmost column of every
    // chunk so a reader can correlate rows across pages.
    if total_cols <= max_cols {
        return render_table_chunk(page, t, config, 0, total_cols, false);
    }

    // Wide-table case: split into column groups.
    let has_header = t.rows.first().is_some_and(|r| r.is_header);
    // Reserve one slot for the repeated row-key column.
    let chunk_size = max_cols.saturating_sub(1).max(1);
    // First chunk includes col 0..chunk_size (no repetition since it owns col 0).
    // Subsequent chunks repeat col 0 followed by chunk_size additional columns.
    let mut page = page;
    let mut start = 0usize;
    let mut first = true;
    while start < total_cols {
        let end = if first {
            (start + max_cols).min(total_cols)
        } else {
            (start + chunk_size).min(total_cols)
        };
        if !first {
            // Each subsequent chunk starts on a fresh page so the reader sees
            // a clean continuation header.
            page = page.new_page_same_size();
            // Note that this is "table N continued" so a future improvement
            // can add a small caption — for now the repeated header column
            // suffices as the linkage cue.
            let _ = has_header; // silence unused-after-edit warning if any
        }
        page = render_table_chunk(page, t, config, start, end, !first);
        start = end;
        first = false;
    }
    page
}

/// Render a single column chunk `[col_start, col_end)` of a table.
///
/// `repeat_first_col`: if true, the leftmost cell of every emitted row is the
/// table's column 0 (row key) regardless of the chunk's start. Used by
/// column-group splitting so rows stay identifiable on continuation pages.
fn render_table_chunk<'a>(
    page: FluentPageBuilder<'a>,
    t: &IrTable,
    _config: &OfficeConfig,
    col_start: usize,
    col_end: usize,
    repeat_first_col: bool,
) -> FluentPageBuilder<'a> {
    if col_end <= col_start {
        return page;
    }
    let chunk_n = col_end - col_start + if repeat_first_col { 1 } else { 0 };
    let has_header = t.rows.first().is_some_and(|r| r.is_header);

    const MIN_COL_PT: f32 = 20.0;
    // Wider cell padding (default is 4pt) so adjacent cell text doesn't run
    // together when extracted: PDF text extraction insets a space when the
    // gap between two text fragments exceeds a fraction of the font size,
    // and 4pt of padding falls just below that threshold for default 11pt
    // body text. 6pt on each side (12pt total between cells) gives
    // extractors a reliable signal.
    let mut cfg = StreamingTableConfig::new()
        .repeat_header(true)
        .cell_padding(6.0, 2.0, 2.0)
        .mode_sample(50, MIN_COL_PT, 300.0);

    let header_row = if has_header { Some(&t.rows[0]) } else { None };
    let resolve_col_idx = |i: usize| -> usize {
        if repeat_first_col {
            if i == 0 {
                0
            } else {
                col_start + (i - 1)
            }
        } else {
            col_start + i
        }
    };
    for i in 0..chunk_n {
        let actual = resolve_col_idx(i);
        let mut name = header_row
            .and_then(|r| r.cells.get(actual))
            .map(|c| elements_to_text(&c.content))
            .unwrap_or_else(|| format!("Col {}", actual + 1));
        // Same trailing-space trick as for body cells: ensures the PDF
        // text extractor sees a word boundary between adjacent header cells.
        if !name.is_empty() && !name.ends_with(' ') {
            name.push(' ');
        }
        cfg = cfg.column(StreamingColumn::new(name));
    }

    let mut st = page.streaming_table(cfg);

    const MAX_CELL_CHARS: usize = 500;
    let body_rows = if has_header {
        &t.rows[1..]
    } else {
        &t.rows[..]
    };
    for row in body_rows {
        let cells: Vec<String> = (0..chunk_n)
            .map(|i| {
                let actual = resolve_col_idx(i);
                let text = row
                    .cells
                    .get(actual)
                    .map(|c| elements_to_text(&c.content))
                    .unwrap_or_default();
                let mut t = if text.len() > MAX_CELL_CHARS {
                    let mut s = text.chars().take(MAX_CELL_CHARS).collect::<String>();
                    s.push('…');
                    s
                } else {
                    text
                };
                // Trailing space guarantees the PDF text extractor sees a
                // word boundary between adjacent cells. Without this, two
                // cells like "2021-01-01" and "15" come out as
                // "2021-01-0115" because the column-padding gap is below
                // the extractor's space-detection threshold (~½ × glyph
                // advance) for many fonts.
                if !t.is_empty() && !t.ends_with(' ') {
                    t.push(' ');
                }
                t
            })
            .collect();
        let _ = st.push_row(|r| {
            for cell in &cells {
                r.cell(cell.as_str());
            }
        });
    }

    st.finish()
}

fn render_image<'a>(
    page: FluentPageBuilder<'a>,
    img: &Image,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    if !config.include_images {
        return Ok(page);
    }
    let data = match &img.data {
        Some(d) if !d.is_empty() => d,
        _ => return Ok(page),
    };

    let width_pt = img
        .display_width_emu
        .map(|e| e as f32 / EMU_PER_PT)
        .unwrap_or(300.0);
    let height_pt = img
        .display_height_emu
        .map(|e| e as f32 / EMU_PER_PT)
        .unwrap_or(200.0);

    // Floating positioning: paint at the absolute (x, y) anchor in
    // page coordinates instead of consuming the cursor. The PDF→IR
    // converter sets this when the source PDF carries an image at a
    // known bbox so flow-mode round-trips don't drop the figure to
    // the bottom-after-text default.
    if let ImagePositioning::Floating(f) = &img.positioning {
        let (_page_w, page_h) = page.page_dimensions();
        let x_pt = f.x_emu as f32 / EMU_PER_PT;
        let y_top_pt = f.y_emu as f32 / EMU_PER_PT;
        let w_pt = (f.width_emu as f32 / EMU_PER_PT).max(width_pt);
        let h_pt = (f.height_emu as f32 / EMU_PER_PT).max(height_pt);
        let pdf_y_bottom = (page_h - y_top_pt - h_pt).max(0.0);
        let rect = Rect::new(x_pt, pdf_y_bottom, w_pt, h_pt);
        return page
            .image_from_bytes(data, rect)
            .map_err(|e| Error::InvalidOperation(format!("floating image: {e}")));
    }

    let x = page.cursor_x();
    let y = page.cursor_y() - height_pt;
    let rect = Rect::new(x, y, width_pt, height_pt);

    let page = page
        .image_from_bytes(data, rect)
        .map_err(|e| Error::InvalidOperation(format!("image embed: {e}")))?;

    Ok(page)
}

fn render_code_block<'a>(page: FluentPageBuilder<'a>, cb: &CodeBlock) -> FluentPageBuilder<'a> {
    let lang = cb.language.as_deref().unwrap_or("");
    page.code_block(lang, &cb.content)
}

fn render_text_box<'a>(
    mut page: FluentPageBuilder<'a>,
    tb: &TextBox,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    use crate::writer::TextAlign;

    // Positional path: when the TextBox carries an explicit
    // (x_emu, y_emu, w_emu, h_emu) rectangle, paint each inner
    // element at the absolute page rectangle:
    //   - `Element::Image` → image rendered at the rectangle
    //   - `Element::Paragraph` → text laid out within the rectangle
    //     using the run properties (font, size, bold, italic, colour)
    //     of the first inline span
    // Mixed content falls back to the flow path so we never silently
    // drop element kinds we don't handle.
    let positional = tb.x_emu.is_some()
        && tb.y_emu.is_some()
        && tb.width_emu.is_some()
        && tb.height_emu.is_some();

    if positional {
        let (_page_w, page_h) = page.page_dimensions();
        let x_pt = tb.x_emu.unwrap_or(0) as f32 / EMU_PER_PT;
        let y_top_pt = tb.y_emu.unwrap_or(0) as f32 / EMU_PER_PT;
        let w_pt = (tb.width_emu.unwrap_or(0) as f32 / EMU_PER_PT).max(1.0);
        let h_pt = (tb.height_emu.unwrap_or(0) as f32 / EMU_PER_PT).max(1.0);
        let pdf_y_bottom = (page_h - y_top_pt - h_pt).max(0.0);
        let rect = Rect::new(x_pt, pdf_y_bottom, w_pt, h_pt);

        if let [Element::Image(img)] = tb.content.as_slice() {
            if let Some(ref data) = img.data {
                if !data.is_empty() {
                    return page
                        .image_from_bytes(data, rect)
                        .map_err(|e| Error::InvalidOperation(format!("textbox image: {e}")));
                }
            }
        }
        if let [Element::Paragraph(p)] = tb.content.as_slice() {
            let text = inline_content_to_text(&p.content);
            if !text.is_empty() {
                let size_pt = office_oxide::ir::first_inline_font_size_pt(&p.content)
                    .unwrap_or(config.default_font_size);
                let (bold, italic) = p
                    .content
                    .iter()
                    .find_map(|ic| match ic {
                        InlineContent::Text(s) => Some((s.bold, s.italic)),
                        _ => None,
                    })
                    .unwrap_or((false, false));
                let default_face = match (bold, italic) {
                    (true, true) => "Helvetica-BoldOblique",
                    (true, false) => "Helvetica-Bold",
                    (false, true) => "Helvetica-Oblique",
                    (false, false) => "Helvetica",
                };
                let font_name = p
                    .content
                    .iter()
                    .find_map(|ic| match ic {
                        InlineContent::Text(s) => s.font_name.clone(),
                        _ => None,
                    })
                    .map(|n| resolve_font_for_text(&n, &text))
                    .unwrap_or_else(|| resolve_font_for_text(default_face, &text));
                page = page.font(&font_name, size_pt);
                let has_color = p
                    .content
                    .iter()
                    .any(|ic| matches!(ic, InlineContent::Text(s) if s.color.is_some()));
                if has_color {
                    // `text_in_rect` doesn't carry per-run colour;
                    // when the source span has an explicit colour
                    // (xlsx cell colour, pptx `<a:solidFill>`),
                    // drop down to `rich_paragraph` at the rect's
                    // top-left so the colour is preserved. Word-wrap
                    // becomes approximate but the colour is the
                    // user-visible bit. Mirrors the same fallback in
                    // `render_pptx_textbox_content`.
                    let runs = inline_content_to_runs(&p.content);
                    page.set_cursor_y(rect.y + rect.height);
                    return Ok(page.rich_paragraph(&runs));
                }
                return Ok(page.text_in_rect(rect, &text, TextAlign::Left));
            }
        }
    }

    let mut p = page;
    for el in &tb.content {
        p = render_ir_element(p, el, config)?;
    }
    Ok(p)
}

// ── Inline content helpers ────────────────────────────────────────────────────

fn inline_content_to_text(content: &[InlineContent]) -> String {
    content
        .iter()
        .map(|ic| match ic {
            InlineContent::Text(span) => span.text.clone(),
            InlineContent::LineBreak => "\n".to_string(),
            InlineContent::FootnoteRef(_) | InlineContent::EndnoteRef(_) => String::new(),
            _ => String::new(),
        })
        .collect()
}

fn inline_content_to_runs(content: &[InlineContent]) -> Vec<TextRun> {
    content
        .iter()
        .filter_map(|ic| match ic {
            InlineContent::Text(span) if !span.text.is_empty() => Some(span_to_run(span)),
            InlineContent::LineBreak => Some(TextRun::normal("\n")),
            _ => None,
        })
        .collect()
}

fn span_to_run(span: &TextSpan) -> TextRun {
    if let Some([r, g, b]) = span.color {
        return TextRun::color(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, &span.text);
    }
    if span.bold && span.italic {
        return TextRun::bold(&span.text);
    }
    if span.bold {
        return TextRun::bold(&span.text);
    }
    if span.italic {
        return TextRun::italic(&span.text);
    }
    TextRun::normal(&span.text)
}

/// Flatten a slice of IR Elements to plain text (used for table cells, notes).
fn elements_to_text(elements: &[Element]) -> String {
    elements
        .iter()
        .map(|el| match el {
            Element::Paragraph(p) => inline_content_to_text(&p.content),
            Element::Heading(h) => inline_content_to_text(&h.content),
            Element::CodeBlock(cb) => cb.content.clone(),
            Element::List(l) => l
                .items
                .iter()
                .map(|item| elements_to_text(&item.content))
                .collect::<Vec<_>>()
                .join(", "),
            Element::Table(t) => t
                .rows
                .iter()
                .flat_map(|r| r.cells.iter())
                .map(|c| elements_to_text(&c.content))
                .collect::<Vec<_>>()
                .join(" "),
            Element::TextBox(tb) => elements_to_text(&tb.content),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Extract plain text from a note body (footnote / endnote).
fn note_to_text(elements: &[Element]) -> String {
    elements_to_text(elements)
}

// ── PPTX positional rendering ─────────────────────────────────────────────────
//
// PPTX slides are inherently positional: every shape lives at an absolute
// EMU rectangle on the slide canvas. The flow-based `ir_to_pdf_bytes`
// pipeline ignores those rectangles and stacks shapes vertically, which
// destroys the visual layout for any non-trivial deck (positioned
// callouts, side-by-side tables, picture frames, themed backgrounds).
//
// `render_pptx_positional` walks `ir.sections` (one section per slide),
// paints each section's solid background colour as a full-page rectangle
// (when present), then renders each `Element::TextBox` at its
// `(x_emu, y_emu, width_emu, height_emu)` rectangle. This is the
// minimal "shape-aware" path landed in v0.3.42 — it does not yet honour
// theme fonts, layout / master backgrounds, or rotation.

/// Does any section in this IR carry positional shape data (TextBox
/// with EMU coordinates) or a section background?
fn pptx_has_positional(ir: &DocumentIR) -> bool {
    for sec in &ir.sections {
        if sec.background_rgb.is_some() {
            return true;
        }
        for el in &sec.elements {
            if let Element::TextBox(tb) = el {
                if tb.x_emu.is_some() && tb.y_emu.is_some() {
                    return true;
                }
            }
        }
    }
    false
}

/// Render a PPTX `DocumentIR` to PDF where each section becomes one
/// page sized to the slide geometry, the section background (if any)
/// fills the page, and each positioned `TextBox` lands at its EMU
/// rectangle.
fn render_pptx_positional(
    ir: &DocumentIR,
    config: &OfficeConfig,
    extra_fonts: &[(String, Vec<u8>)],
) -> Result<Vec<u8>> {
    use crate::writer::EmbeddedFont;

    let mut builder = DocumentBuilder::new().compress_streams(true);

    if let Some(ref t) = ir.metadata.title {
        builder = builder.title(t);
    }
    if let Some(ref a) = ir.metadata.author {
        builder = builder.author(a);
    }

    // Same Unicode-cmap guard the other rendering paths use:
    // CID-only subsets registered here would route every text run
    // through GID 0 (.notdef) — invisible glyphs across whole
    // slide bodies. Falling through gives base 14 with proper
    // glyphs (wrong typeface but readable).
    let mut registered: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (name, data) in extra_fonts {
        match EmbeddedFont::from_data(Some(name.clone()), data.clone()) {
            Ok(font) if font.has_usable_unicode_cmap() => {
                builder = builder.register_embedded_font(name.clone(), font);
                registered.insert(name.clone());
            },
            _ => {},
        }
    }

    let default_page = {
        let (w, h) = config.page_size.dimensions();
        (w, h)
    };

    let unicode_fallback =
        if registered.contains(crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME) {
            Some(crate::fonts::unicode_fallback::UNICODE_FALLBACK_NAME.to_string())
        } else {
            None
        };
    let cjk_fallback =
        if registered.contains(crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME) {
            Some(crate::fonts::unicode_fallback::UNICODE_FALLBACK_CJK_NAME.to_string())
        } else {
            None
        };
    let result = with_registered_fonts_full(
        registered,
        unicode_fallback,
        cjk_fallback,
        || -> Result<Vec<u8>> {
            for section in &ir.sections {
                let (page_w_pt, page_h_pt) = section
                    .page_setup
                    .as_ref()
                    .map(|ps| {
                        (
                            ps.width_twips as f32 / TWIPS_PER_PT,
                            ps.height_twips as f32 / TWIPS_PER_PT,
                        )
                    })
                    .unwrap_or(default_page);
                let page_size = PageSize::Custom(page_w_pt.max(1.0), page_h_pt.max(1.0));
                let mut page = builder.page(page_size);

                // Slide background (solid fill only). Painted before any shape
                // so positioned content sits on top of the chrome.
                if let Some([r, g, b]) = section.background_rgb {
                    page = page.filled_rect(
                        0.0,
                        0.0,
                        page_w_pt,
                        page_h_pt,
                        r as f32 / 255.0,
                        g as f32 / 255.0,
                        b as f32 / 255.0,
                    );
                }

                // Optional title heading rendered at the top — keeps section
                // titles visible when the title placeholder had no `<a:xfrm>`
                // (in which case it's still in `section.title` but not present
                // as a positioned shape).
                // Two-pass z-order: positioned shapes (TextBox at EMU coords)
                // paint first as the slide-chrome layer (banners, logos,
                // background imagery), then flow content (Heading, Paragraph)
                // paints on top. Without this, the Heading rendered at the
                // top-of-page cursor gets covered by a top-of-page banner
                // TextBox that's iterated AFTER it in section.elements.
                // We also advance the cursor past the lowest banner so the
                // first flow Heading lands under the banner instead of
                // overlapping it.
                let mut top_banner_floor_pt: Option<f32> = None;
                for el in &section.elements {
                    if let Element::TextBox(tb) = el {
                        page = render_pptx_element(page, el, page_w_pt, page_h_pt, config)?;
                        if let (Some(y_emu), Some(h_emu)) = (tb.y_emu, tb.height_emu) {
                            let y_top_pt = y_emu as f32 / EMU_PER_PT;
                            let h_pt = h_emu as f32 / EMU_PER_PT;
                            // Treat any TextBox anchored in the top half of the
                            // page as a banner whose bottom edge gates the
                            // first flow line. PDF y-up: cursor is the
                            // distance from the page bottom; banner bottom
                            // (in PDF coords) is `page_h_pt - y_top_pt - h_pt`.
                            if y_top_pt < page_h_pt * 0.5 {
                                let banner_bottom_pdf_y = (page_h_pt - y_top_pt - h_pt).max(0.0);
                                top_banner_floor_pt =
                                    Some(top_banner_floor_pt.map_or(banner_bottom_pdf_y, |c| {
                                        c.min(banner_bottom_pdf_y)
                                    }));
                            }
                        }
                    }
                }
                if let Some(floor_y) = top_banner_floor_pt {
                    // Advance the cursor below the banner with a small gap.
                    let target = (floor_y - 12.0).max(36.0);
                    if page.cursor_y() > target {
                        page.set_cursor_y(target);
                    }
                }
                for el in &section.elements {
                    if !matches!(el, Element::TextBox(_)) {
                        page = render_pptx_element(page, el, page_w_pt, page_h_pt, config)?;
                    }
                }

                page.done();
            }

            builder
                .build()
                .map_err(|e| Error::InvalidOperation(format!("PPTX positional PDF build: {e}")))
        },
    );
    result
}

/// Render a single section-level IR element on a positioned slide
/// page. Handles `TextBox` (positioned shape), `Heading` (title at top
/// of page), and `Paragraph` (slide-level prose like notes). Other
/// variants are dropped silently — they fall outside the positional
/// model PPTX needs.
fn render_pptx_element<'a>(
    page: FluentPageBuilder<'a>,
    element: &Element,
    page_w_pt: f32,
    page_h_pt: f32,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    match element {
        Element::TextBox(tb) => render_pptx_textbox(page, tb, page_h_pt, config),
        Element::Heading(h) => {
            // PPTX title without xfrm — render flow-style at top.
            let text = inline_content_to_text(&h.content);
            if text.is_empty() {
                Ok(page)
            } else {
                use crate::writer::TextAlign;
                use office_oxide::ir::ParagraphAlignment;
                // Honour explicit center/right alignment from the
                // title placeholder. PPTX slide titles are commonly
                // centered; without this they render left-aligned
                // even when the source PPTX carried algn="ctr".
                if matches!(
                    h.alignment,
                    Some(ParagraphAlignment::Center) | Some(ParagraphAlignment::Right)
                ) {
                    // Prefer the source span's actual font size
                    // (preserved via the inline content's first
                    // span) over the heading-level default. CFR
                    // cover "Title 7" is 28pt source — at lvl 1 the
                    // default 18pt would render it visibly smaller
                    // than source. Falls back to a level-based ratio
                    // when the IR doesn't carry a span size (e.g.
                    // synthesised headings from the convert layer).
                    let body_size = config.default_font_size;
                    let size_pt = office_oxide::ir::first_inline_font_size_pt(&h.content)
                        .unwrap_or(match h.level {
                            1 => body_size * 1.6,
                            2 => body_size * 1.4,
                            3 => body_size * 1.2,
                            _ => body_size,
                        });
                    let face = first_inline_font_name(&h.content)
                        .map(|n| resolve_font_for_text(&n, &text))
                        .unwrap_or_else(|| resolve_font_for_text("Helvetica-Bold", &text));
                    let page = page.font(&face, size_pt);
                    let cursor_x = page.cursor_x();
                    let cursor_y = page.cursor_y();
                    let right_margin = 36.0_f32;
                    let usable_w = (page_w_pt - cursor_x - right_margin).max(1.0);
                    let line_h = size_pt * page.text_config_line_height();
                    let est_chars_per_line = (usable_w / (size_pt * 0.5)).max(1.0) as usize;
                    let lines = text.chars().count().max(1).div_ceil(est_chars_per_line);
                    let block_h = line_h * lines.max(1) as f32;
                    let rect = Rect::new(cursor_x, cursor_y - block_h, usable_w, block_h);
                    let align = match h.alignment {
                        Some(ParagraphAlignment::Center) => TextAlign::Center,
                        Some(ParagraphAlignment::Right) => TextAlign::Right,
                        _ => TextAlign::Left,
                    };
                    let mut page = page.text_in_rect(rect, &text, align);
                    let new_y = cursor_y - block_h - line_h * 0.5;
                    page.set_cursor_y(new_y);
                    Ok(page)
                } else {
                    // Even left-aligned headings: prefer source span
                    // size over the level-based default.
                    let body_size = config.default_font_size;
                    let size_pt = office_oxide::ir::first_inline_font_size_pt(&h.content)
                        .unwrap_or(match h.level {
                            1 => body_size * 1.6,
                            2 => body_size * 1.4,
                            3 => body_size * 1.2,
                            _ => body_size,
                        });
                    let face = first_inline_font_name(&h.content)
                        .map(|n| resolve_font_for_text(&n, &text))
                        .unwrap_or_else(|| resolve_font_for_text("Helvetica-Bold", &text));
                    let page = page.font(&face, size_pt);
                    Ok(page.text(&text))
                }
            }
        },
        Element::Paragraph(p) => {
            let text = inline_content_to_text(&p.content);
            // PPTX-encoded ThematicBreak: paragraph of U+2500 only.
            // office_oxide emits Element::ThematicBreak through PPTX
            // as a centered paragraph of box-drawing horizontal
            // characters; re-render as a real horizontal rule.
            if !text.is_empty() && text.chars().all(|c| c == '\u{2500}' || c.is_whitespace()) {
                let line_h = page.text_config_font_size() * page.text_config_line_height();
                let cur_y = page.cursor_y();
                let pdf_y = (cur_y - line_h * 0.5).max(0.0);
                let right_margin = 36.0_f32;
                let mut new_page = page.filled_rect(
                    36.0,
                    pdf_y,
                    page_w_pt - 36.0 - right_margin,
                    0.5,
                    0.5,
                    0.5,
                    0.5,
                );
                let new_y = cur_y - line_h;
                if new_y > 0.0 {
                    new_page.set_cursor_y(new_y);
                }
                return Ok(new_page);
            }
            let _ = page_h_pt;
            if text.is_empty() {
                // Empty spacer paragraph (round-tripped through PPTX
                // from pdf_to_ir's gap-detection pass). Advance the
                // cursor by one line height so it reproduces the
                // vertical gap rather than being a no-op.
                let line_h = page.text_config_font_size() * page.text_config_line_height();
                let cur_y = page.cursor_y();
                let new_y = cur_y - line_h;
                if new_y > 0.0 {
                    let mut page = page;
                    page.set_cursor_y(new_y);
                    Ok(page)
                } else {
                    Ok(page)
                }
            } else {
                // Flow-style: continue from the cursor wherever the
                // previous element left it. The previous version of
                // this branch hard-pinned every paragraph to (36, 36),
                // overlapping all body content at the bottom-left
                // corner — that was intended only for speaker-notes
                // overflow but fired for every flat paragraph,
                // which is the common case once `convert_pptx`
                // stops wrapping content with no `<a:xfrm>` in a
                // 0×0 TextBox.
                let size_pt = office_oxide::ir::first_inline_font_size_pt(&p.content)
                    .unwrap_or(config.default_font_size);
                let face = first_inline_font_name(&p.content)
                    .map(|n| resolve_font_for_text(&n, &text))
                    .unwrap_or_else(|| resolve_font_for_text("Helvetica", &text));
                let page = page.font(&face, size_pt);
                // Honour explicit center/right alignment from the
                // source PPTX. `paragraph()` is left-only, so when
                // the IR says Center/Right we drop down to
                // `text_in_rect` with the page width less margins.
                use crate::writer::TextAlign;
                use office_oxide::ir::ParagraphAlignment;
                if matches!(
                    p.alignment,
                    Some(ParagraphAlignment::Center) | Some(ParagraphAlignment::Right)
                ) {
                    let cursor_x = page.cursor_x();
                    let cursor_y = page.cursor_y();
                    let right_margin = 36.0_f32;
                    let usable_w = (page_w_pt - cursor_x - right_margin).max(1.0);
                    let line_h = size_pt * page.text_config_line_height();
                    let est_chars_per_line = (usable_w / (size_pt * 0.5)).max(1.0) as usize;
                    let lines = text.chars().count().max(1).div_ceil(est_chars_per_line);
                    let block_h = line_h * lines.max(1) as f32;
                    let rect = Rect::new(cursor_x, cursor_y - block_h, usable_w, block_h);
                    let align = match p.alignment {
                        Some(ParagraphAlignment::Center) => TextAlign::Center,
                        Some(ParagraphAlignment::Right) => TextAlign::Right,
                        _ => TextAlign::Left,
                    };
                    let mut page = page.text_in_rect(rect, &text, align);
                    let new_y = cursor_y - block_h - line_h * 0.5;
                    page.set_cursor_y(new_y);
                    Ok(page)
                } else if p
                    .content
                    .iter()
                    .any(|ic| matches!(ic, InlineContent::Text(s) if s.color.is_some()))
                {
                    // At least one run carries an explicit colour —
                    // emit rich-text so `<a:solidFill>` from the
                    // source PPTX survives to the rendered PDF
                    // (otherwise `page.paragraph` flattens to plain
                    // text and the renderer uses the page-default
                    // black). Mirrors the docx side that already
                    // routes colour through `<w:color w:val=...>`.
                    let runs = inline_content_to_runs(&p.content);
                    Ok(page.rich_paragraph(&runs))
                } else {
                    Ok(page.paragraph(&text))
                }
            }
        },
        _ => Ok(page),
    }
}

/// Render a positioned `TextBox` on the current page. The TextBox's
/// EMU rectangle is converted to PDF points (origin top-left → PDF
/// bottom-left), then each contained element is rendered into that
/// frame: paragraphs / headings as text-in-rect (so wrapping respects
/// the shape width); tables as positioned rectangles with cell text;
/// images as image rectangles.
fn render_pptx_textbox<'a>(
    page: FluentPageBuilder<'a>,
    tb: &TextBox,
    page_h_pt: f32,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    let x_emu = tb.x_emu.unwrap_or(0);
    let y_emu = tb.y_emu.unwrap_or(0);
    let w_emu = tb.width_emu.unwrap_or(0);
    let h_emu = tb.height_emu.unwrap_or(0);

    let x_pt = x_emu as f32 / EMU_PER_PT;
    let y_top_pt = y_emu as f32 / EMU_PER_PT;
    let w_pt = (w_emu as f32 / EMU_PER_PT).max(1.0);
    let h_pt = (h_emu as f32 / EMU_PER_PT).max(1.0);

    // PDF y origin is bottom-left, so the rectangle's PDF y is
    // `page_h_pt - y_top_pt - h_pt`. We render text relative to this.
    let pdf_y_bottom = (page_h_pt - y_top_pt - h_pt).max(0.0);

    render_pptx_textbox_content(page, &tb.content, x_pt, pdf_y_bottom, w_pt, h_pt, config)
}

/// Render the inner content of a positioned shape rectangle.
///
/// `(x_pt, y_pt)` is the PDF bottom-left corner; `(w_pt, h_pt)` the
/// rectangle's size in points.
fn render_pptx_textbox_content<'a>(
    mut page: FluentPageBuilder<'a>,
    content: &[Element],
    x_pt: f32,
    y_pt: f32,
    w_pt: f32,
    h_pt: f32,
    config: &OfficeConfig,
) -> Result<FluentPageBuilder<'a>> {
    use crate::writer::TextAlign;

    // Walk the inner content and stack paragraphs from the top of the
    // shape rectangle downwards. PPTX text is anchored to the top of
    // its body box by default, with a small inset.
    let inset = 2.0_f32;
    let mut cursor_top = y_pt + h_pt - inset;
    let inner_x = x_pt + inset;
    let inner_w = (w_pt - 2.0 * inset).max(1.0);
    let avail_bottom = y_pt + inset;

    for el in content {
        if cursor_top <= avail_bottom {
            break;
        }
        match el {
            Element::Heading(h) => {
                let text = inline_content_to_text(&h.content);
                if text.is_empty() {
                    continue;
                }
                let size_pt = match h.level {
                    1 => 24.0,
                    2 => 20.0,
                    3 => 16.0,
                    _ => 14.0,
                };
                let h_face = first_inline_font_name(&h.content)
                    .map(|n| resolve_font_for_text(&n, &text))
                    .unwrap_or_else(|| resolve_font_for_text("Helvetica-Bold", &text));
                page = page.font(&h_face, size_pt);
                let rect_y = cursor_top - size_pt;
                let rect = Rect::new(inner_x, rect_y, inner_w, size_pt);
                page = page.text_in_rect(rect, &text, TextAlign::Left);
                // Advance roughly one line height. text_in_rect may
                // wrap; for headings we treat as one line (wrapping
                // would push past the rect anyway — we just advance
                // the cursor).
                let line_h = size_pt * 1.2;
                let lines = wrap_estimate(&text, inner_w, size_pt);
                cursor_top -= line_h * lines.max(1) as f32;
            },
            Element::Paragraph(p) => {
                let text = inline_content_to_text(&p.content);
                if text.is_empty() {
                    cursor_top -= config.default_font_size * config.line_height;
                    continue;
                }
                let size_pt = office_oxide::ir::first_inline_font_size_pt(&p.content)
                    .unwrap_or(config.default_font_size);
                let bold = p
                    .content
                    .iter()
                    .find_map(|ic| {
                        if let InlineContent::Text(s) = ic {
                            Some(s.bold)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);
                let italic = p
                    .content
                    .iter()
                    .find_map(|ic| {
                        if let InlineContent::Text(s) = ic {
                            Some(s.italic)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);
                let default_font = match (bold, italic) {
                    (true, true) => "Helvetica-BoldOblique",
                    (true, false) => "Helvetica-Bold",
                    (false, true) => "Helvetica-Oblique",
                    (false, false) => "Helvetica",
                };
                let p_face = first_inline_font_name(&p.content)
                    .map(|n| resolve_font_for_text(&n, &text))
                    .unwrap_or_else(|| resolve_font_for_text(default_font, &text));
                page = page.font(&p_face, size_pt);
                let lines = wrap_estimate(&text, inner_w, size_pt);
                let block_h = size_pt * 1.2 * lines.max(1) as f32;
                let rect_y = cursor_top - block_h;
                let has_color = p
                    .content
                    .iter()
                    .any(|ic| matches!(ic, InlineContent::Text(s) if s.color.is_some()));
                if has_color {
                    // `text_in_rect` doesn't carry per-run colour; if
                    // any run has an explicit `<a:solidFill>` colour
                    // (or `<w:color>` on the docx side), drop the
                    // word-wrap path and emit a `rich_paragraph` at
                    // the rect's top-left. Wrapping is approximate
                    // but the colour survives — which is the
                    // important fidelity bit for short
                    // call-out / annotation text that's the typical
                    // shape of a coloured run in a slide TextBox.
                    let runs = inline_content_to_runs(&p.content);
                    page.set_cursor_y(cursor_top);
                    page = page.rich_paragraph(&runs);
                } else {
                    let rect = Rect::new(inner_x, rect_y, inner_w, block_h);
                    page = page.text_in_rect(rect, &text, TextAlign::Left);
                }
                cursor_top -= block_h;
            },
            Element::List(l) => {
                // Flatten list to bullet text and render line by line.
                for item in &l.items {
                    let line = format!("• {}", elements_to_text(&item.content));
                    if line.trim().is_empty() {
                        continue;
                    }
                    let l_face = resolve_font_for_text("Helvetica", &line);
                    page = page.font(&l_face, config.default_font_size);
                    let lines = wrap_estimate(&line, inner_w, config.default_font_size);
                    let block_h = config.default_font_size * 1.2 * lines.max(1) as f32;
                    let rect_y = cursor_top - block_h;
                    let rect = Rect::new(inner_x, rect_y, inner_w, block_h);
                    page = page.text_in_rect(rect, &line, TextAlign::Left);
                    cursor_top -= block_h;
                    if cursor_top <= avail_bottom {
                        break;
                    }
                }
            },
            Element::Table(t) => {
                page = render_pptx_positioned_table(page, t, x_pt, y_pt, w_pt, h_pt, config);
                // Tables consume the whole shape; stop further content.
                break;
            },
            Element::Image(img) => {
                // The PPTX picture frame currently propagates only
                // alt-text; image bytes aren't carried through the IR
                // for slides yet. When they are, render at the shape
                // rectangle.
                if let Some(ref data) = img.data {
                    if !data.is_empty() {
                        let rect = Rect::new(x_pt, y_pt, w_pt, h_pt);
                        // image_from_bytes consumes `page` and returns
                        // Result; on error there's no recoverable
                        // builder, so unwrap_or_else falls back to a
                        // fresh page-positioned no-op via re-entering
                        // the loop with content fully consumed.
                        page = page
                            .image_from_bytes(data, rect)
                            .map_err(|e| Error::InvalidOperation(format!("pptx picture: {e}")))?;
                    }
                }
                break;
            },
            Element::TextBox(inner) => {
                // Nested shape (rare for PPTX). Recurse with absolute
                // coordinates inside the parent.
                page = render_pptx_textbox_content(
                    page,
                    &inner.content,
                    x_pt,
                    y_pt,
                    w_pt,
                    h_pt,
                    config,
                )?;
            },
            _ => {},
        }
    }

    Ok(page)
}

/// Render a table positioned within a PPTX shape rectangle. Cells get
/// equal-width columns spread across `w_pt`; rows get equal-height
/// stripes across `h_pt`. Each cell renders its text via
/// `text_in_rect` with a 1pt inset and a thin stroked border so the
/// table reads as a grid in the rendered PDF.
fn render_pptx_positioned_table<'a>(
    mut page: FluentPageBuilder<'a>,
    t: &IrTable,
    x_pt: f32,
    y_pt: f32,
    w_pt: f32,
    h_pt: f32,
    config: &OfficeConfig,
) -> FluentPageBuilder<'a> {
    use crate::writer::{LineStyle, TextAlign};

    if t.rows.is_empty() {
        return page;
    }
    let n_cols = t.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
    let n_rows = t.rows.len();
    if n_cols == 0 {
        return page;
    }

    let col_w = w_pt / n_cols as f32;
    let row_h = h_pt / n_rows as f32;
    let stroke = LineStyle {
        width: 0.5,
        color: (0.6, 0.6, 0.6),
        dash: None,
    };

    for (ri, row) in t.rows.iter().enumerate() {
        // Row top (PDF y, bottom-left origin): start at top of shape
        // and step downwards by row_h per row.
        let row_top = y_pt + h_pt - (ri as f32) * row_h;
        let row_bottom = row_top - row_h;
        for (ci, cell) in row.cells.iter().enumerate() {
            if ci >= n_cols {
                break;
            }
            let cell_x = x_pt + ci as f32 * col_w;
            // Border
            page = page.stroke_rect(cell_x, row_bottom, col_w, row_h, stroke.clone());
            // Cell text
            let text = elements_to_text(&cell.content);
            if text.is_empty() {
                continue;
            }
            let size_pt =
                elements_first_font_size(&cell.content).unwrap_or(config.default_font_size);
            let inset = 2.0_f32;
            let inner_rect = Rect::new(
                cell_x + inset,
                row_bottom + inset,
                (col_w - 2.0 * inset).max(1.0),
                (row_h - 2.0 * inset).max(1.0),
            );
            // Pick bold from first text span if any.
            let bold = cell.content.iter().any(|el| {
                if let Element::Paragraph(p) = el {
                    p.content.iter().any(|ic| {
                        if let InlineContent::Text(s) = ic {
                            s.bold
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            });
            let font = if bold { "Helvetica-Bold" } else { "Helvetica" };
            page = page.font(font, size_pt);
            // Anchor near the top of the cell so multi-line wraps
            // cascade downwards instead of clipping below the top
            // edge.
            let rect_top = inner_rect.y + inner_rect.height;
            let lines = wrap_estimate(&text, inner_rect.width, size_pt);
            let block_h = size_pt * 1.2 * lines.max(1) as f32;
            let rect = Rect::new(
                inner_rect.x,
                (rect_top - block_h).max(inner_rect.y),
                inner_rect.width,
                block_h.min(inner_rect.height),
            );
            page = page.text_in_rect(rect, &text, TextAlign::Left);
        }
    }

    page
}

/// Walk a slice of IR elements and return the first declared font
/// size in points (mirrors `first_inline_font_size_pt` but at element
/// level).
fn elements_first_font_size(elements: &[Element]) -> Option<f32> {
    for el in elements {
        if let Element::Paragraph(p) = el {
            if let Some(s) = office_oxide::ir::first_inline_font_size_pt(&p.content) {
                return Some(s);
            }
        }
    }
    None
}

/// Estimate how many wrapped lines `text` produces in a rectangle of
/// width `w_pt` at font size `size_pt`. Used by the positional
/// renderer to advance the per-shape cursor without round-tripping
/// through the layout engine. Approximation: assume average glyph
/// advance ≈ 0.5 × font size (true for Helvetica, close enough for
/// other proportional fonts).
fn wrap_estimate(text: &str, w_pt: f32, size_pt: f32) -> usize {
    if w_pt <= 0.0 || size_pt <= 0.0 {
        return 1;
    }
    let avg_advance = size_pt * 0.5;
    let chars_per_line = (w_pt / avg_advance).floor().max(1.0) as usize;
    let mut lines = 0usize;
    for raw in text.split('\n') {
        if raw.is_empty() {
            lines += 1;
            continue;
        }
        let n = raw.chars().count().div_ceil(chars_per_line);
        lines += n.max(1);
    }
    lines.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_margins_default() {
        let m = Margins::default();
        assert_eq!(m.top, 72.0);
        assert_eq!(m.bottom, 72.0);
        assert_eq!(m.left, 72.0);
        assert_eq!(m.right, 72.0);
    }

    #[test]
    fn test_margins_uniform() {
        let m = Margins::uniform(36.0);
        assert_eq!(m.top, 36.0);
        assert_eq!(m.bottom, 36.0);
    }

    #[test]
    fn test_config_default() {
        let c = OfficeConfig::default();
        assert_eq!(c.default_font, "Helvetica");
        assert_eq!(c.default_font_size, 11.0);
    }

    #[test]
    fn test_converter_new() {
        let c = OfficeConverter::new();
        assert_eq!(c.config().default_font, "Helvetica");
    }

    #[test]
    fn test_ir_to_pdf_empty() {
        let ir = DocumentIR::default();
        let config = OfficeConfig::default();
        let bytes = ir_to_pdf_bytes(&ir, &config, &[]).expect("ir_to_pdf_bytes");
        assert!(bytes.starts_with(b"%PDF-"), "output must be a PDF");
    }

    #[test]
    fn test_map_to_base14_latex_serif() {
        // TeX Gyre Termes (Times-compatible) bare body → Times-Roman.
        assert_eq!(map_to_base14("TeXGyreTermesX-Regular"), Some("Times-Roman"));
        assert_eq!(map_to_base14("TeXGyreTermesX-Bold"), Some("Times-Bold"));
        assert_eq!(map_to_base14("TeXGyreTermesX-Italic"), Some("Times-Italic"));
        // Latin Modern Roman.
        assert_eq!(map_to_base14("LMRoman10-Regular"), Some("Times-Roman"));
        // STIX math.
        assert_eq!(map_to_base14("STIXMath-Regular"), Some("Times-Roman"));
    }

    #[test]
    fn test_map_to_base14_latex_sans() {
        assert_eq!(map_to_base14("TeXGyreHeros-Bold"), Some("Helvetica-Bold"));
        assert_eq!(map_to_base14("LMSans10-Regular"), Some("Helvetica"));
    }

    #[test]
    fn test_map_to_base14_newtx_math_italic() {
        // newtx math italic (used for inline math in LaTeX papers)
        // is a Times-Italic-shaped face. Pre-fix it fell back to
        // Helvetica because no family heuristic recognised the name.
        assert_eq!(map_to_base14("NewTXMI"), Some("Times-Italic"));
        assert_eq!(map_to_base14("NewTXMI5"), Some("Times-Italic"));
        assert_eq!(map_to_base14("NewTXMI7"), Some("Times-Italic"));
        // Bold math italic.
        assert_eq!(map_to_base14("NewTXBMI"), Some("Times-BoldItalic"));
        // newtx math italic alphabet extension.
        assert_eq!(map_to_base14("txmiaX"), Some("Times-Italic"));
    }

    #[test]
    fn test_map_to_base14_math_symbols() {
        // txsy / txex are math symbol / extension fonts — route to
        // PDF's Symbol so glyphs like ≤, ≥, ∫, ∑ at least pick the
        // right shape instead of Helvetica's missing-glyph squares.
        assert_eq!(map_to_base14("txsys"), Some("Symbol"));
        assert_eq!(map_to_base14("txexs"), Some("Symbol"));
        assert_eq!(map_to_base14("cmsy10"), Some("Symbol"));
        assert_eq!(map_to_base14("cmex10"), Some("Symbol"));
    }

    #[test]
    fn test_map_to_base14_monospace() {
        assert_eq!(map_to_base14("LMMono10-Regular"), Some("Courier"));
        assert_eq!(map_to_base14("LMMono10-Bold"), Some("Courier-Bold"));
    }
}
