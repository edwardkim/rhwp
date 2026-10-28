//! PDF → `office_oxide::DocumentIR` conversion.
//!
//! Format-agnostic: each PDF page becomes one IR `Section` carrying its
//! source `MediaBox` under `page_setup` and a `NextPage` break from the
//! second section onward. Spans are grouped into lines, then lines into
//! paragraphs by vertical-gap heuristic, and the dominant font-size
//! ratio against the page's median assigns heading levels (H1/H2/H3).
//!
//! The same IR feeds DOCX, PPTX, and XLSX writers (via
//! `office_oxide::create::ir_to_{docx,pptx,xlsx}`); the caller passes
//! the target `DocumentFormat` so `metadata.format` is set correctly
//! once on the way out.
//!
//! Pipeline:
//!   `PdfDocument::extract_spans(page)`  →  `Vec<TextSpan>`
//!       →  `pdf_to_ir(doc, format, opts)`  →  `office_oxide::DocumentIR`
//!       →  `office_oxide::create::ir_to_<format>(&ir)`  →  format bytes

use crate::error::Result;
use crate::layout::text_block::{Color, TextChar, TextSpan};
use office_oxide::format::DocumentFormat;
use office_oxide::ir::{
    ColumnLayout, DocumentIR, Element, Heading, Image, ImageFormat, ImagePositioning,
    InlineContent, Metadata, PageSetup, Paragraph, Section, SectionBreakType, TextSpan as IrSpan,
};
use std::collections::HashMap;

/// 1 PDF point = 20 twips (1 inch = 72 pt = 1440 twips).
const PT_TO_TWIPS: f32 = 20.0;
/// Default page margin for the IR output, in points. Each PDF page's
/// MediaBox sets the section size; these margins describe the
/// renderable inset inside that page. 36 pt = 0.5 in is tighter than
/// the OfficeConfig default of 72 pt / 1 in and avoids the "single
/// page in Word" symptom: dense academic source PDFs use slim
/// internal margins, so 1 in target margins shrink the usable text
/// area below source and Word reflows content into more pages than
/// the PDF source had.
const DEFAULT_MARGIN_PT: f32 = 36.0;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Options for the PDF → IR conversion.
#[derive(Debug, Clone)]
pub struct PdfToIrOptions {
    /// Font-size ratios (vs body median) that trigger H1 / H2 / H3.
    pub heading_ratios: [f32; 3],
    /// Inter-line gap (in line-heights) that starts a new paragraph.
    pub paragraph_gap_factor: f32,
}

impl Default for PdfToIrOptions {
    fn default() -> Self {
        Self {
            heading_ratios: [1.75, 1.35, 1.15],
            paragraph_gap_factor: 1.2,
        }
    }
}

/// Convert a whole `PdfDocument` to an `office_oxide::DocumentIR`
/// tagged for the given target format.
///
/// The `format` argument feeds `metadata.format` on the resulting IR
/// — it doesn't influence layout, but downstream writers
/// (`ir_to_docx`, `ir_to_pptx`, `ir_to_xlsx`) read it as a sanity
/// check and to inform default styling. Each PDF page becomes one
/// `Section`. The first section starts as `Continuous`; subsequent
/// sections carry `NextPage` so any paginated renderer (Word's page
/// view, PPTX slides, paginated PDF re-render) starts a fresh page
/// per source page.
pub fn pdf_to_ir(
    doc: &crate::document::PdfDocument,
    format: DocumentFormat,
    options: &PdfToIrOptions,
) -> Result<DocumentIR> {
    let page_count = doc.page_count()?;
    let mut sections: Vec<Section> = Vec::with_capacity(page_count);

    // First pass: extract spans for every page so we can build a
    // document-wide color histogram before mapping to IR. The histogram
    // feeds the link-annotation-color leak heuristic in `span_to_ir`:
    // the canonical PDF link blue (#0000FF) is dropped unconditionally,
    // and other suspiciously-canonical colors (#808080, #FF0000) are
    // dropped only when fewer than 3 spans use them across the entire
    // document — meaning they're almost certainly annotation-bleed
    // rather than intentional styling.
    let mut all_spans: Vec<Vec<TextSpan>> = Vec::with_capacity(page_count);
    for page_idx in 0..page_count {
        let (_x1, _y1, _x2, _y2) = doc.get_page_media_box(page_idx)?;
        let page_h_for_filter = (_y2 - _y1).abs();
        let mut spans = doc.extract_spans(page_idx)?;
        // Drop rotated text (typically arxiv-style page-edge watermarks
        // or rotated table headers). `TextSpan` carries no rotation
        // field, but `extract_chars` does — we identify rotated spans
        // by overlapping their bbox with chars whose
        // `rotation_degrees` is non-zero. Without this filter the
        // span's text lands in the IR as a horizontal paragraph with a
        // bbox that places it at the rotated frame's center, causing
        // visible text-on-text overlay (e.g. arxiv ID stamped over the
        // title in flow-mode round-trips). Layout-mode writers also
        // benefit since rotated text would otherwise emit a horizontal
        // shape in the wrong place.
        if let Ok(chars) = doc.extract_chars(page_idx) {
            // `span_overlaps_rotated_chars` drops a span only when its nearest
            // char is rotated (>= 5deg). When the page has NO rotated char at
            // all (the overwhelming majority), the per-span nearest-char scan
            // would run O(spans x chars) only to never drop anything. Gate the
            // whole retain behind one O(chars) precheck — byte-identical output,
            // removes the quadratic on every unrotated page.
            let any_rotated = chars.iter().any(|c| c.rotation_degrees.abs() >= 5.0);
            if any_rotated {
                let horiz = chars
                    .iter()
                    .filter(|c| c.rotation_degrees.abs() < 5.0)
                    .count();
                let chars_horizontal_dominant = chars.is_empty() || horiz * 4 >= chars.len() * 3;
                spans
                    .retain(|s| !span_overlaps_rotated_chars(s, &chars, chars_horizontal_dominant));
            }
        }
        // Drop page-pagination artifacts: PDF marked-content tags
        // headers / footers / watermarks via `Artifact` BDC blocks
        // (PDF spec 14.8.2.2). Without this, CFR-style pages bleed
        // their per-page footer noise ("skersey on DSK4WB1RN3PROD
        // with CFR…") into the body of every cover/section slide.
        // We also catch headers and watermarks for the same reason.
        // Many real-world PDFs don't tag artifacts via marked
        // content though; fall back to a geometric heuristic for
        // tiny-font text in the page's top/bottom 5% strip.
        spans.retain(|s| !is_page_artifact(s) && !is_geometric_footer(s, page_h_for_filter));
        all_spans.push(spans);
    }
    let color_counts = build_color_histogram(&all_spans);

    // Per-page resource-id → real-face-name lookup so span font
    // references like "F2" / "F12" get rewritten into the actual PDF
    // BaseFont (e.g. "AvantGarde-Book"). Without this, every span on
    // a CFR-style PDF lands in the IR with `font_name=None` and the
    // round-trip falls back to Helvetica regardless of which fonts
    // the source-PDF actually used.
    let face_lookups = doc.page_font_face_lookups().unwrap_or_default();

    for (page_idx, spans) in all_spans.iter().enumerate() {
        let (x1, y1, x2, y2) = doc.get_page_media_box(page_idx)?;
        let page_w = (x2 - x1).abs();
        let page_h = (y2 - y1).abs();

        // Every section ends with a hard page break: each PDF page maps
        // to one section, and we want Word/PowerPoint/Excel to render
        // them as separate pages on the way back. The OOXML semantic
        // is that section N's `<w:type>` controls how section N+1
        // starts — so `NextPage` everywhere means "section N+1 starts
        // on a fresh page". The very last section's type is
        // immaterial (no section follows) but using `NextPage` keeps
        // all sectPrs uniform.
        let break_type = SectionBreakType::NextPage;

        // Extract embedded raster images on this page. Each image becomes
        // an inline `Element::Image` appended to the section so DOCX/PPTX
        // writers re-emit it as `<w:drawing>` / `<p:pic>`. Failures
        // (corrupted XObject, missing decoder) are silently dropped — a
        // missing image is better than a failed conversion.
        let images = extract_page_images(doc, page_idx, page_h);
        let face_lookup = face_lookups.get(page_idx).cloned().unwrap_or_default();
        // Extract horizontal rule paths (thin wide strokes / filled
        // rects). Used by `page_to_section` to emit `ThematicBreak`
        // elements at the correct y position between paragraphs —
        // preserves source rules like CFR cover's underline beneath
        // "Title 7 / Agriculture". Each entry is (y_top_pdf, width)
        // so the section builder can sort them with paragraphs.
        let rules = extract_horizontal_rules(doc, page_idx, page_w, page_h);

        sections.push(page_to_section(
            spans,
            page_w,
            page_h,
            break_type,
            &images,
            &rules,
            options,
            &color_counts,
            &face_lookup,
        ));
    }

    let mut metadata = Metadata {
        format,
        ..Default::default()
    };
    populate_metadata_from_pdf_info(doc, &mut metadata);

    Ok(DocumentIR { metadata, sections })
}

/// Pull `/Title`, `/Author`, `/Subject`, `/Keywords`, `/Creator`,
/// `/Producer`, `/CreationDate`, `/ModDate` from the source PDF's
/// trailer Info dictionary into the IR `Metadata`. Downstream
/// writers fan these out to `docProps/core.xml` (DOCX/PPTX/XLSX).
/// Empty strings or missing entries leave the IR metadata
/// untouched so callers that already populated it (e.g. `from_markdown`)
/// don't get silently overwritten.
/// A horizontal-rule path detected on a PDF page. Carries the
/// PDF-y of the rule's top edge so `page_to_section` can interleave
/// rules with paragraphs in source y-order.
#[derive(Debug, Clone)]
struct HorizontalRule {
    /// PDF y of the rule (bottom-up coords). Used as the rule's
    /// vertical position for interleaving with paragraphs.
    y_pdf: f32,
}

/// Extract horizontal rules from the page: thin filled rects or
/// straight horizontal stroked lines that are wide enough to count
/// as document-level separators (≥ 30% of page width). Used to
/// recover decorative underlines that the source PDF draws as
/// vector paths beneath title-block text.
fn extract_horizontal_rules(
    doc: &crate::document::PdfDocument,
    page_idx: usize,
    page_w_pt: f32,
    _page_h_pt: f32,
) -> Vec<HorizontalRule> {
    let paths = match doc.extract_paths(page_idx) {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    let min_w = page_w_pt * 0.3;
    for p in paths {
        // Length from the RENDERED extents so a stroke-width-encoded rule
        // (speck segment, page-wide stroke) qualifies;
        // thinness stays geometric — the drawn thickness of a rule is its
        // stroke, not a disqualifier.
        let rendered = p.rendered_bbox();
        let w = rendered.width;
        let h = p.bbox.height;
        // Two patterns: very thin rect (filled or stroked) or a
        // horizontal stroked line.
        let thin_rect = w >= min_w && h <= 2.0 && h > 0.0;
        let h_line = p.is_straight_line() && w >= min_w && h <= 1.0;
        if thin_rect || h_line {
            // Use the centre of the rendered rule as its y-coordinate so
            // it interleaves cleanly between paragraphs above and below.
            out.push(HorizontalRule {
                y_pdf: rendered.y + rendered.height * 0.5,
            });
        }
    }
    out
}

/// True when this span is geometrically positioned like a footer
/// (or header) — small font in the bottom (or top) 5% strip of the
/// page. Backstop for source PDFs that don't tag pagination
/// artifacts via marked content; without this, CFR-style filename /
/// date footers bleed into body text.
fn is_geometric_footer(span: &TextSpan, page_h_pt: f32) -> bool {
    if page_h_pt <= 0.0 {
        return false;
    }
    let footer_strip = page_h_pt * 0.05;
    let header_strip_low = page_h_pt - page_h_pt * 0.05;
    let in_footer = span.bbox.y < footer_strip;
    let in_header = span.bbox.y > header_strip_low;
    let small_font = span.font_size < 8.0;
    (in_footer || in_header) && small_font
}

/// True when this span is part of a page-pagination artifact
/// (header / footer / page number / watermark). Source PDFs mark
/// these via the `Artifact` marked-content BDC operator (PDF spec
/// 14.8.2.2); pdf_oxide surfaces the kind in `TextSpan.artifact_type`.
/// Filtering them at the IR boundary removes the per-page noise like
/// the CFR "skersey on DSK4WB1RN3PROD with CFR…" footers that would
/// otherwise overlap body content in the rendered round-trip.
fn is_page_artifact(span: &TextSpan) -> bool {
    use crate::extractors::text::{ArtifactType, PaginationSubtype};
    matches!(
        span.artifact_type,
        Some(ArtifactType::Pagination(_))
            | Some(ArtifactType::Page)
            | Some(ArtifactType::Background)
    ) || matches!(span.artifact_type, Some(ArtifactType::Pagination(PaginationSubtype::Watermark)))
}

/// True if `span` matches a rotated char (>= 5° off horizontal) by
/// origin proximity. The span's bbox origin is correct for rotated text
/// even when its width/height are wrong, so we identify the originating
/// char via origin distance and consult its `rotation_degrees`. Only
/// active when `chars_horizontal_dominant` (per-char rotation field
/// trustworthy); otherwise return `false` to avoid dropping legitimate
/// spans on pages where the rotation extractor misfires.
pub(crate) fn span_overlaps_rotated_chars(
    span: &TextSpan,
    chars: &[TextChar],
    chars_horizontal_dominant: bool,
) -> bool {
    if !chars_horizontal_dominant {
        return false;
    }
    if chars.is_empty() {
        return false;
    }
    let bx = span.bbox.x;
    let by = span.bbox.y;
    let mut best_idx: Option<usize> = None;
    let mut best_d2 = f32::INFINITY;
    for (i, c) in chars.iter().enumerate() {
        let dx = c.origin_x - bx;
        let dy = c.origin_y - by;
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            best_d2 = d2;
            best_idx = Some(i);
        }
    }
    // Origin-match tolerance: if the nearest char is more than ~5pt
    // away the span has no clear originating char — be conservative
    // and keep it.
    const MAX_ORIGIN_DIST: f32 = 5.0;
    if best_d2 > MAX_ORIGIN_DIST * MAX_ORIGIN_DIST {
        return false;
    }
    match best_idx {
        Some(i) => chars[i].rotation_degrees.abs() >= 5.0,
        None => false,
    }
}

fn populate_metadata_from_pdf_info(doc: &crate::document::PdfDocument, metadata: &mut Metadata) {
    let trailer = doc.trailer();
    let info_ref = match trailer
        .as_dict()
        .and_then(|d| d.get("Info"))
        .and_then(|o| o.as_reference())
    {
        Some(r) => r,
        None => return,
    };
    let info_obj = match doc.load_object(info_ref) {
        Ok(o) => o,
        Err(_) => return,
    };
    let info = crate::editor::DocumentInfo::from_object(&info_obj);
    if metadata.title.is_none() {
        metadata.title = info.title.clone();
    }
    if metadata.author.is_none() {
        metadata.author = info.author.clone();
    }
    if metadata.subject.is_none() {
        metadata.subject = info.subject.clone();
    }
    if metadata.keywords.is_empty() {
        if let Some(kw) = &info.keywords {
            metadata.keywords = kw
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    // PDF Info /CreationDate uses the format `D:YYYYMMDDHHmmSSOHH'mm'`
    // — leave the parse-to-W3CDTF for a follow-up; office writers
    // tolerate any string for the dcterms:created element.
    if metadata.created.is_none() {
        metadata.created = info.creation_date.clone();
    }
    if metadata.modified.is_none() {
        metadata.modified = info.mod_date.clone();
    }
}

// ---------------------------------------------------------------------------
// Per-page conversion
// ---------------------------------------------------------------------------

/// Pull every embedded image off `page_idx`, decode each to PNG bytes,
/// and lift the (bbox, png) into an `Image` IR element. Images with no
/// usable bbox or that fail to decode are skipped — the goal is "as
/// many figures as possible survive", not "all-or-nothing".
/// IR `Image` plus the absolute EMU anchor recovered from the PDF's
/// content-stream image bbox. Downstream callers that care about
/// position fidelity (the flow IR→PDF renderer's `render_text_box`
/// path) wrap each entry in `Element::TextBox` with these coords so
/// images land at their source x/y instead of inline-after-text.
struct PositionedImage {
    image: Image,
    x_emu: i64,
    y_emu: i64,
    cx_emu: i64,
    cy_emu: i64,
}

/// Convert a `PositionedImage` into an `Element::Image` with
/// `ImagePositioning::Floating` carrying the absolute EMU anchor.
/// The IR→PDF positional renderer (`render_positional_ir`, layout
/// mode) and the IR→PDF flow renderer (`ir_to_pdf_bytes` →
/// `render_image`) both honour this, painting the figure at the
/// source bbox rather than inline-after-text. The DOCX and PPTX
/// writers also pass `Floating` images through to their respective
/// `<wp:anchor>` / `<p:pic>` emitters with the anchor coords.
fn positioned_image_to_element(pi: &PositionedImage) -> Element {
    let mut image = pi.image.clone();
    image.positioning = ImagePositioning::Floating(office_oxide::ir::FloatingImage {
        x_emu: pi.x_emu,
        y_emu: pi.y_emu,
        width_emu: pi.cx_emu.max(0) as u64,
        height_emu: pi.cy_emu.max(0) as u64,
        h_anchor: office_oxide::ir::FloatAnchor::default(),
        v_anchor: office_oxide::ir::FloatAnchor::default(),
        text_wrap: office_oxide::ir::TextWrap::default(),
        allow_overlap: true,
    });
    Element::Image(image)
}

fn extract_page_images(
    doc: &crate::document::PdfDocument,
    page_idx: usize,
    page_h_pt: f32,
) -> Vec<PositionedImage> {
    // 1 pt = 12 700 EMU.
    const EMU_PER_PT: f64 = 12_700.0;

    let raw = match doc.extract_images(page_idx) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::with_capacity(raw.len());
    for img in raw {
        let bbox = match img.bbox() {
            Some(b) => *b,
            None => continue,
        };
        let png = match img.to_png_bytes() {
            Ok(b) if !b.is_empty() => b,
            _ => continue,
        };
        let w_emu = ((bbox.width as f64).max(1.0) * EMU_PER_PT) as u64;
        let h_emu = ((bbox.height as f64).max(1.0) * EMU_PER_PT) as u64;
        // PDF coords: y up from bottom-left. Office EMU: y down from
        // top-left. Convert: y_top_pt = page_h - bbox.y - bbox.height.
        let x_emu = (bbox.x.max(0.0) as f64 * EMU_PER_PT) as i64;
        let y_top_pt = (page_h_pt - bbox.y - bbox.height).max(0.0);
        let y_emu = (y_top_pt as f64 * EMU_PER_PT) as i64;
        let image = Image {
            data: Some(png),
            format: Some(ImageFormat::Png),
            display_width_emu: Some(w_emu),
            display_height_emu: Some(h_emu),
            pixel_width: Some(img.width()),
            pixel_height: Some(img.height()),
            // Inline positioning (the IR Image variant). The
            // surrounding `PositionedImage` carries the absolute EMU
            // anchor; downstream emission wraps the inline Image in a
            // TextBox with that anchor when fidelity requires it.
            positioning: ImagePositioning::Inline,
            ..Default::default()
        };
        out.push(PositionedImage {
            image,
            x_emu,
            y_emu,
            cx_emu: w_emu as i64,
            cy_emu: h_emu as i64,
        });
    }

    // Supplemental: capture Form XObject and inline-image regions
    // by rasterising the page sub-region they occupy. Source PDFs
    // commonly use these for tiny vector logos (GPO badge,
    // accessibility marks, gov agency logos) that aren't surfaced
    // by `extract_images` (which only handles `/Subtype /Image`).
    // Without this, the office round-trip drops those decorations
    // silently. Requires the `rendering` feature; gracefully no-ops
    // when absent.
    #[cfg(feature = "rendering")]
    {
        let existing_rects: Vec<(f32, f32, f32, f32)> = out
            .iter()
            .map(|pi| {
                let x = pi.x_emu as f32 / EMU_PER_PT as f32;
                let y_top = pi.y_emu as f32 / EMU_PER_PT as f32;
                let w = pi.cx_emu as f32 / EMU_PER_PT as f32;
                let h = pi.cy_emu as f32 / EMU_PER_PT as f32;
                let y_pdf = (page_h_pt - y_top - h).max(0.0);
                (x, y_pdf, w, h)
            })
            .collect();
        let regions = crate::converters::form_xobject_finder::rasterize_form_and_inline_regions(
            doc,
            page_idx,
            page_h_pt,
            &existing_rects,
        );
        for ((x_pdf, y_pdf, w, h), png) in regions {
            let w_emu = ((w as f64).max(1.0) * EMU_PER_PT) as u64;
            let h_emu = ((h as f64).max(1.0) * EMU_PER_PT) as u64;
            let x_emu = (x_pdf.max(0.0) as f64 * EMU_PER_PT) as i64;
            let y_top_pt = (page_h_pt - y_pdf - h).max(0.0);
            let y_emu = (y_top_pt as f64 * EMU_PER_PT) as i64;
            let image = Image {
                data: Some(png),
                format: Some(ImageFormat::Png),
                display_width_emu: Some(w_emu),
                display_height_emu: Some(h_emu),
                positioning: ImagePositioning::Inline,
                ..Default::default()
            };
            out.push(PositionedImage {
                image,
                x_emu,
                y_emu,
                cx_emu: w_emu as i64,
                cy_emu: h_emu as i64,
            });
        }
    }

    out
}

fn page_to_section(
    spans: &[TextSpan],
    page_w_pt: f32,
    page_h_pt: f32,
    break_type: SectionBreakType,
    images: &[PositionedImage],
    rules: &[HorizontalRule],
    options: &PdfToIrOptions,
    color_counts: &HashMap<[u8; 3], u32>,
    face_lookup: &HashMap<String, String>,
) -> Section {
    let margin_twips = (DEFAULT_MARGIN_PT * PT_TO_TWIPS) as u32;
    let page_setup = PageSetup {
        width_twips: (page_w_pt * PT_TO_TWIPS) as u32,
        height_twips: (page_h_pt * PT_TO_TWIPS) as u32,
        margin_top_twips: margin_twips,
        margin_bottom_twips: margin_twips,
        margin_left_twips: margin_twips,
        margin_right_twips: margin_twips,
        landscape: page_w_pt > page_h_pt,
        ..Default::default()
    };

    if spans.is_empty() {
        // Even an image-only page (cover, logo plate, scanned form
        // without OCR text) should propagate its figures into the IR.
        let elements = images.iter().map(positioned_image_to_element).collect();
        return Section {
            elements,
            page_setup: Some(page_setup),
            break_type,
            ..Default::default()
        };
    }

    let median_pt = median_font_size(spans);
    // `group_into_paragraphs` returns paragraphs *and* the line
    // structure within each paragraph so we can apply line-boundary
    // text fix-ups (drop end-of-line soft hyphens, insert missing
    // inter-line spaces) before the spans collapse into a single flat
    // run sequence.
    let (para_lines, all_lines) =
        group_into_paragraphs_with_lines(spans, options.paragraph_gap_factor);

    let columns = detect_columns(&all_lines, page_w_pt);

    let mut elements: Vec<Element> = Vec::with_capacity(para_lines.len() + images.len());
    // Emit images FIRST so they form a background layer; subsequent
    // paragraphs paint on top in the flow renderer's draw order. The
    // alternative (images last) caused text-under-logo bleed on
    // CFR-style cover pages where the shield bbox happens to overlap
    // body text below it: image drew last → image painted over the
    // already-rendered text. Source PDFs typically draw images
    // before text in their content stream, so this also matches
    // typical z-order.
    for pi in images {
        elements.push(positioned_image_to_element(pi));
    }
    // Track the previous paragraph's bottom-most line bbox y (PDF
    // bottom-up coords) so we can detect oversized vertical gaps —
    // common on cover pages where the source distributes content
    // across the full page height. Without this the round-trip
    // packs everything against the previous paragraph using the
    // renderer's default line spacing and the cover collapses to
    // the upper portion of the page.
    // Sort horizontal rules by descending y so we can pop them off
    // as we iterate paragraphs top-to-bottom. Each rule is emitted
    // when we cross its y-position (i.e. between the paragraph above
    // it and the paragraph below it).
    let mut rules_top_down: Vec<f32> = rules.iter().map(|r| r.y_pdf).collect();
    rules_top_down.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a));
    let mut rules_iter = rules_top_down.into_iter().peekable();

    let mut prev_para_min_y: Option<f32> = None;
    let mut prev_para_avg_pt: Option<f32> = None;
    for lines in &para_lines {
        if lines.is_empty() {
            continue;
        }

        // Top y of this paragraph's first line (PDF y-up: top edge =
        // bbox.y + height, since bbox.y is the baseline).
        let this_top_y = lines
            .first()
            .map(|line| {
                line.iter()
                    .map(|s| s.bbox.y + s.bbox.height)
                    .fold(f32::MIN, f32::max)
            })
            .unwrap_or(0.0);
        let this_bottom_y = lines
            .last()
            .map(|line| line.iter().map(|s| s.bbox.y).fold(f32::MAX, f32::min))
            .unwrap_or(0.0);

        // Emit any horizontal rules that fall above this paragraph
        // (after the previous paragraph). PDF y-up: rule_y between
        // prev_bottom and this_top.
        while let Some(&rule_y) = rules_iter.peek() {
            let after_prev = prev_para_min_y.is_none_or(|prev| rule_y < prev);
            let before_this = rule_y > this_top_y;
            if after_prev && before_this {
                elements.push(Element::ThematicBreak);
                rules_iter.next();
            } else if rule_y >= prev_para_min_y.unwrap_or(f32::INFINITY) {
                // Above the previous paragraph already (i.e. we
                // already passed it); discard.
                rules_iter.next();
            } else {
                break;
            }
        }
        // Average font size for this paragraph (proxy for line height).
        let this_avg_pt = {
            let mut sum = 0.0_f32;
            let mut n = 0_u32;
            for line in lines {
                for s in line {
                    sum += s.font_size;
                    n += 1;
                }
            }
            if n > 0 {
                sum / n as f32
            } else {
                median_pt
            }
        };

        // Compute extra-gap and translate it into a single empty
        // Paragraph spacer carrying `space_before_twips` equal to the
        // exact excess gap. The flow renderer honours
        // `space_before_twips` by advancing the cursor by that
        // amount before returning early on empty content. This
        // reproduces the source's vertical rhythm precisely instead
        // of approximating with N×line-height spacers (which was
        // both coarse and lossy through PPTX round-trip, where empty
        // <a:p> count varies with PowerPoint's autofit logic).
        let mut excess_pt: f32 = 0.0;
        if let (Some(prev_y), Some(prev_avg)) = (prev_para_min_y, prev_para_avg_pt) {
            let gap_pt = prev_y - this_top_y;
            let line_h_pt = prev_avg.max(this_avg_pt) * 1.2;
            if gap_pt > line_h_pt * 1.5 {
                excess_pt = (gap_pt - line_h_pt).max(0.0);
                // Sanity cap — gaps > 600pt would be a single-page
                // boundary or garbage Y position; treat them as
                // bounded.
                if excess_pt > 600.0 {
                    excess_pt = 600.0;
                }
            }
        }
        if excess_pt > 0.5 {
            let twips = (excess_pt * PT_TO_TWIPS) as u32;
            elements.push(Element::Paragraph(Paragraph {
                space_before_twips: Some(twips),
                ..Default::default()
            }));
        }

        // If this paragraph is a multi-line CENTERED block of short
        // lines (typical for title-page bibliographic blocks: each
        // metadata line is its own visual element, NOT a wrapped
        // continuation of a long paragraph), emit one IR Element per
        // line so each line preserves its own alignment and any
        // inter-line vertical gaps inside the block survive too.
        // Without this, `merge_lines_into_spans` joins consecutive
        // lines with a single space and the round-trip renders e.g.
        // "Agriculture\nParts 1 to 26" as "Agriculture Parts 1 to 26"
        // on one line, losing the source's deliberate two-line title
        // break.
        let alignment = detect_paragraph_alignment(lines, page_w_pt);
        let is_centered_block =
            matches!(alignment, Some(office_oxide::ir::ParagraphAlignment::Center));
        let lines_short = lines.iter().all(|line| {
            if line.is_empty() {
                return true;
            }
            let left = line.iter().map(|s| s.bbox.x).fold(f32::MAX, f32::min);
            let right = line
                .iter()
                .map(|s| s.bbox.x + s.bbox.width)
                .fold(f32::MIN, f32::max);
            let line_w = (right - left).max(0.0);
            line_w < page_w_pt * 0.75
        });

        if is_centered_block && lines.len() > 1 && lines_short {
            // Emit one element per source line. Re-detect heading
            // level per line so a 28pt "Title 7" can become H1 while
            // the smaller "Agriculture" line below it becomes H2 or
            // a Paragraph based on its own font-size ratio.
            let mut prev_inner_min_y: Option<f32> = prev_para_min_y;
            let mut prev_inner_avg_pt: Option<f32> = prev_para_avg_pt;
            for line in lines {
                if line.is_empty() {
                    continue;
                }
                let single = std::slice::from_ref(line);
                let inner_top_y = line
                    .iter()
                    .map(|s| s.bbox.y + s.bbox.height)
                    .fold(f32::MIN, f32::max);
                let inner_bottom_y = line.iter().map(|s| s.bbox.y).fold(f32::MAX, f32::min);
                let inner_avg_pt = if line.is_empty() {
                    median_pt
                } else {
                    line.iter().map(|s| s.font_size).sum::<f32>() / line.len() as f32
                };

                // Emit any horizontal rules that fall above THIS
                // inner line and below the previous inner line.
                while let Some(&rule_y) = rules_iter.peek() {
                    let after_prev = prev_inner_min_y.is_none_or(|prev| rule_y < prev);
                    let before_this = rule_y > inner_top_y;
                    if after_prev && before_this {
                        elements.push(Element::ThematicBreak);
                        rules_iter.next();
                    } else if rule_y >= prev_inner_min_y.unwrap_or(f32::INFINITY) {
                        rules_iter.next();
                    } else {
                        break;
                    }
                }

                let mut inner_excess_pt: f32 = 0.0;
                if let (Some(prev_y), Some(prev_avg)) = (prev_inner_min_y, prev_inner_avg_pt) {
                    let gap_pt = prev_y - inner_top_y;
                    let line_h_pt = prev_avg.max(inner_avg_pt) * 1.2;
                    if gap_pt > line_h_pt * 1.5 {
                        inner_excess_pt = (gap_pt - line_h_pt).max(0.0);
                        if inner_excess_pt > 600.0 {
                            inner_excess_pt = 600.0;
                        }
                    }
                }
                if inner_excess_pt > 0.5 {
                    let twips = (inner_excess_pt * PT_TO_TWIPS) as u32;
                    elements.push(Element::Paragraph(Paragraph {
                        space_before_twips: Some(twips),
                        ..Default::default()
                    }));
                }
                let inner_element = lines_to_element(
                    single,
                    median_pt,
                    options,
                    color_counts,
                    page_w_pt,
                    face_lookup,
                );
                elements.push(inner_element);
                prev_inner_min_y = Some(inner_bottom_y);
                prev_inner_avg_pt = Some(inner_avg_pt);
            }
            // Carry the inner loop's final position back to the
            // outer loop so subsequent paragraphs measure their gap
            // from the last actual line emitted.
            prev_para_min_y = prev_inner_min_y;
            prev_para_avg_pt = prev_inner_avg_pt;
            continue;
        } else {
            let element =
                lines_to_element(lines, median_pt, options, color_counts, page_w_pt, face_lookup);
            elements.push(element);
        }
        prev_para_min_y = Some(this_bottom_y);
        prev_para_avg_pt = Some(this_avg_pt);
    }

    Section {
        elements,
        page_setup: Some(page_setup),
        break_type,
        columns,
        ..Default::default()
    }
}

/// Fold the per-line span vectors of a paragraph into a single span
/// sequence with line-boundary text fix-ups applied:
///   - **End-of-line soft hyphen** (e.g. `"captur-"` followed by
///     `"ing"`): drop the trailing `-` from the previous span and
///     concatenate without inserting whitespace.
///   - **Missing inter-line space** (e.g. `"the"` followed by
///     `"Chinese"`): when neither side has whitespace at the seam,
///     append a single space to the previous span so downstream text
///     extractors don't fuse the words.
fn merge_lines_into_spans(lines: &[Vec<TextSpan>]) -> Vec<TextSpan> {
    let mut out: Vec<TextSpan> = Vec::new();
    for (li, line) in lines.iter().enumerate() {
        if li > 0 {
            // Look at the seam between out's last span and the first
            // span of the new line.
            if let (Some(prev), Some(next)) = (out.last_mut(), line.first()) {
                let prev_text = prev.text.trim_end_matches([' ', '\t']);
                let prev_ends_ws = prev.text.chars().last().is_none_or(|c| c.is_whitespace());
                let next_starts_ws = next.text.chars().next().is_none_or(|c| c.is_whitespace());
                let ends_hyphen = prev_text.ends_with('-')
                    && prev_text
                        .chars()
                        .rev()
                        .nth(1)
                        .is_some_and(|c| c.is_alphabetic());
                let starts_lower = next
                    .text
                    .trim_start()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_lowercase());
                if ends_hyphen {
                    // A hyphen at the end of a line is an incidental layout
                    // artifact, not a word break — so this seam never gets a
                    // separating space inserted. When the continuation starts
                    // lowercase it is a single word divided across the line
                    // (a soft hyphen, e.g. "captur-" / "ing"): drop the
                    // trailing '-' and join. When it starts uppercase it is a
                    // hard hyphen in a wrapped compound (e.g. "sub-" /
                    // "Neptune"): keep the hyphen, still no space. Mirrors the
                    // `merge_hyphenated_spans` heuristic in
                    // `docx_layout::merge_hyphenated_spans`.
                    if starts_lower {
                        let trimmed: String = prev_text[..prev_text.len() - 1].to_string()
                            + &prev.text[prev_text.len()..]; // preserve any trailing ws (none expected)
                        prev.text = trimmed;
                    }
                } else if !prev_ends_ws && !next_starts_ws {
                    // Neither end carries whitespace; downstream
                    // concatenation would fuse the words. Append a
                    // single space.
                    prev.text.push(' ');
                }
            }
        }
        out.extend(line.iter().cloned());
    }
    out
}

fn lines_to_element(
    lines: &[Vec<TextSpan>],
    median_pt: f32,
    opts: &PdfToIrOptions,
    color_counts: &HashMap<[u8; 3], u32>,
    page_w_pt: f32,
    face_lookup: &HashMap<String, String>,
) -> Element {
    let group = merge_lines_into_spans(lines);
    let avg_pt = if group.is_empty() {
        median_pt
    } else {
        group.iter().map(|s| s.font_size).sum::<f32>() / group.len() as f32
    };
    let ratio = avg_pt / median_pt.max(1.0);

    let alignment = detect_paragraph_alignment(lines, page_w_pt);
    let inline = spans_to_inline(&group, color_counts, face_lookup);

    if ratio >= opts.heading_ratios[0] {
        Element::Heading(Heading {
            level: 1,
            content: inline,
            alignment,
            ..Default::default()
        })
    } else if ratio >= opts.heading_ratios[1] {
        Element::Heading(Heading {
            level: 2,
            content: inline,
            alignment,
            ..Default::default()
        })
    } else if ratio >= opts.heading_ratios[2] {
        Element::Heading(Heading {
            level: 3,
            content: inline,
            alignment,
            ..Default::default()
        })
    } else {
        Element::Paragraph(Paragraph {
            content: inline,
            alignment,
            ..Default::default()
        })
    }
}

/// Detect paragraph alignment from line geometry. Centered when every
/// line's left and right margins are roughly symmetric (within 8 % of
/// page width). Right-aligned when every line's right margin is small
/// while the left margin is large. Otherwise None (Left, the default).
fn detect_paragraph_alignment(
    lines: &[Vec<TextSpan>],
    page_w_pt: f32,
) -> Option<office_oxide::ir::ParagraphAlignment> {
    use office_oxide::ir::ParagraphAlignment;
    if lines.is_empty() || page_w_pt <= 0.0 {
        return None;
    }
    // Looser tolerance — title-block text often left-aligns each line
    // around a centred *block*, which produces a small left margin
    // and a much larger right margin per line. Treat any group whose
    // mean centre x is within 8 % of page mid-x and whose left
    // margin is at least 10 % of page width as centred.
    let mid = page_w_pt * 0.5;
    let centre_tol = page_w_pt * 0.08;
    let min_left = page_w_pt * 0.10;
    let mut all_centered = true;
    let mut all_right = true;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let left = line.iter().map(|s| s.bbox.x).fold(f32::MAX, f32::min);
        let right = line
            .iter()
            .map(|s| s.bbox.x + s.bbox.width)
            .fold(f32::MIN, f32::max);
        let left_margin = left.max(0.0);
        let right_margin = (page_w_pt - right).max(0.0);
        let centre = (left + right) * 0.5;
        // Centred if the line's centre is within tolerance of the
        // page mid-x AND both margins are real (content sits inside
        // both edges, not flush against either). The right-margin
        // check distinguishes centred title text from regular body
        // text that happens to span the full width.
        if (centre - mid).abs() > centre_tol || left_margin < min_left || right_margin < min_left {
            all_centered = false;
        }
        // Right-aligned when the right margin is very small and the
        // left margin is large.
        if left_margin <= page_w_pt * 0.25 || right_margin > page_w_pt * 0.10 {
            all_right = false;
        }
    }
    if all_centered {
        Some(ParagraphAlignment::Center)
    } else if all_right {
        Some(ParagraphAlignment::Right)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Grouping: spans → lines → paragraphs
// ---------------------------------------------------------------------------

/// Group spans into lines, then cluster adjacent lines into paragraphs.
///
/// Returns:
/// - `paragraphs`: each entry is a `Vec<Line>` retaining the
///   line-by-line structure so callers can apply line-boundary fix-ups
///   (soft-hyphen merge, missing inter-line space).
/// - `all_lines`: every detected line on the page in reading order;
///   used by `detect_columns` to inspect the page-wide x distribution.
fn group_into_paragraphs_with_lines(
    spans: &[TextSpan],
    gap_factor: f32,
) -> (Vec<Vec<Vec<TextSpan>>>, Vec<Vec<TextSpan>>) {
    // 1. Sort top-to-bottom (PDF Y increases bottom→top, so descending Y = top first).
    let mut sorted: Vec<&TextSpan> = spans.iter().collect();
    sorted.sort_by(|a, b| {
        let ay = a.bbox.y + a.bbox.height * 0.5;
        let by = b.bbox.y + b.bbox.height * 0.5;
        crate::utils::safe_float_cmp(by, ay)
    });

    if sorted.is_empty() {
        return (Vec::new(), Vec::new());
    }

    // 2. Cluster into lines: spans whose Y-centers are within 0.8 × line_height.
    let mut lines: Vec<Vec<&TextSpan>> = Vec::new();
    let mut cur_line: Vec<&TextSpan> = vec![sorted[0]];

    for span in sorted.iter().skip(1) {
        let last = cur_line.last().unwrap();
        let last_cy = last.bbox.y + last.bbox.height * 0.5;
        let span_cy = span.bbox.y + span.bbox.height * 0.5;
        let lh = last.font_size.max(span.font_size);

        if (last_cy - span_cy).abs() < lh * 0.8 {
            cur_line.push(span);
        } else {
            lines.push(std::mem::take(&mut cur_line));
            cur_line = vec![span];
        }
    }
    lines.push(cur_line);

    // 3. Sort each line left-to-right by X.
    for line in &mut lines {
        line.sort_by(|a, b| crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x));
    }

    // Materialise lines as owned `Vec<TextSpan>` for downstream use.
    let owned_lines: Vec<Vec<TextSpan>> = lines
        .iter()
        .map(|l| l.iter().map(|s| (*s).clone()).collect())
        .collect();

    // 4. Cluster adjacent lines into paragraphs.
    //    New paragraph when: vertical gap > gap_factor × line_height,
    //    or the dominant font size changes significantly.
    let mut paragraphs: Vec<Vec<Vec<TextSpan>>> = Vec::new();
    let mut cur_para: Vec<Vec<TextSpan>> = Vec::new();

    for i in 0..lines.len() {
        let line_owned = owned_lines[i].clone();

        if i == 0 {
            cur_para.push(line_owned);
            continue;
        }

        let prev = &lines[i - 1];
        let cur = &lines[i];
        // Bottom of previous line (lowest Y of all spans in it, i.e. min bbox.y).
        let prev_bottom = prev.iter().map(|s| s.bbox.y).fold(f32::MAX, f32::min);
        // Top of current line.
        let cur_top = cur
            .iter()
            .map(|s| s.bbox.y + s.bbox.height)
            .fold(f32::MIN, f32::max);
        let lh = cur.iter().map(|s| s.font_size).fold(0.0_f32, f32::max);

        // Positive gap = white space between lines.
        let gap = prev_bottom - cur_top;

        let prev_avg = prev.iter().map(|s| s.font_size).sum::<f32>() / prev.len() as f32;
        let cur_avg = cur.iter().map(|s| s.font_size).sum::<f32>() / cur.len() as f32;
        let size_jump = (cur_avg - prev_avg).abs() > 2.0;

        if (gap > lh * gap_factor || size_jump) && !cur_para.is_empty() {
            paragraphs.push(std::mem::take(&mut cur_para));
        }
        cur_para.push(line_owned);
    }
    if !cur_para.is_empty() {
        paragraphs.push(cur_para);
    }

    (paragraphs, owned_lines)
}

/// Detect a 2-column layout from the page's **line-start** x distribution.
///
/// In a 2-column layout, every line of body text starts at one of two
/// x positions: the column-1 left margin or the column-2 left margin.
/// The line-cluster pass in `group_into_paragraphs_with_lines` groups
/// spans by y-center, so a left-column line and right-column line at
/// the same baseline collapse into one merged "line"; that's actually
/// useful here because each merged line then carries spans from *both*
/// columns and the **leftmost span of each y-baseline cluster** lands
/// at the column-1 margin while the **leftmost span past the midline**
/// (within the same merged line) lands at the column-2 margin.
///
/// Algorithm:
///   - For every merged line, record:
///       - the leftmost span's x (left-margin candidate);
///       - the leftmost span whose x ≥ mid (column-2 margin candidate),
///         if any.
///   - Histogram both lists in 5 pt bins.
///   - The dominant bin in each list is taken as the column edge.
///   - The right edge of column 1 is estimated as the max right-x of
///     any span on a line that *also* contains a column-2 candidate
///     (otherwise the line is single-column).
///   - 2-column requires:
///       - column-2 candidate appeared on ≥ 25% of lines,
///       - dominant bins captured ≥ 25% of their list each,
///       - col2_left − col1_right ≥ 36 pt.
fn detect_columns(all_lines: &[Vec<TextSpan>], page_w_pt: f32) -> Option<ColumnLayout> {
    const MIN_GUTTER_PT: f32 = 36.0; // 0.5 inch
    const BIN_PT: f32 = 5.0;

    if all_lines.len() < 8 {
        return None;
    }
    let mid = page_w_pt * 0.5;

    let mut col1_lefts: Vec<f32> = Vec::new();
    let mut col2_lefts: Vec<f32> = Vec::new();
    // Right-x of the rightmost left-of-mid span on a line that also has
    // a right-of-mid span. This is the strongest signal for "where
    // column 1 ends" because such lines are confirmed 2-column.
    let mut col1_rights_on_two_col_lines: Vec<f32> = Vec::new();

    for line in all_lines {
        if line.is_empty() {
            continue;
        }
        let lx = line.iter().map(|s| s.bbox.x).fold(f32::MAX, f32::min);
        col1_lefts.push(lx);

        // Leftmost span past the midline (if any).
        let mut col2_l: Option<f32> = None;
        for s in line {
            if s.bbox.x >= mid {
                let v = s.bbox.x;
                col2_l = Some(col2_l.map_or(v, |cur| cur.min(v)));
            }
        }
        if let Some(c2l) = col2_l {
            col2_lefts.push(c2l);
            // Rightmost left-of-mid span end on the same line.
            let r1 = line
                .iter()
                .filter(|s| s.bbox.x + s.bbox.width <= mid)
                .map(|s| s.bbox.x + s.bbox.width)
                .fold(f32::MIN, f32::max);
            if r1.is_finite() {
                col1_rights_on_two_col_lines.push(r1);
            }
        }
    }

    let total_lines = all_lines.len() as f32;
    if (col2_lefts.len() as f32) < total_lines * 0.25 {
        return None;
    }

    let mode = |xs: &[f32]| -> Option<(f32, usize)> {
        if xs.is_empty() {
            return None;
        }
        let mut bins: HashMap<i32, usize> = HashMap::new();
        for &x in xs {
            *bins.entry((x / BIN_PT).round() as i32).or_insert(0) += 1;
        }
        // Explicit tie-break, not `max_by_key`: it returns the LAST maximal
        // element, and over a `HashMap` that is the per-process-randomized
        // iteration order, so two equally-populated bins picked a different
        // column left-edge per process. Ties resolve to the smaller bin.
        let (&best_b, &best_n) = bins
            .iter()
            .max_by(|(a_bin, a_n), (b_bin, b_n)| a_n.cmp(b_n).then_with(|| b_bin.cmp(a_bin)))?;
        Some((best_b as f32 * BIN_PT, best_n))
    };

    let (col1_left, col1_n) = mode(&col1_lefts)?;
    let (col2_left, col2_n) = mode(&col2_lefts)?;
    if (col1_n as f32) < (col1_lefts.len() as f32) * 0.25 {
        return None;
    }
    if (col2_n as f32) < (col2_lefts.len() as f32) * 0.25 {
        return None;
    }

    // Estimate column-1 right edge. Use the 90th percentile of
    // confirmed two-column-line right-x's; fall back to col2_left −
    // MIN_GUTTER_PT if too few samples.
    let col1_right = if col1_rights_on_two_col_lines.len() >= 5 {
        let mut v = col1_rights_on_two_col_lines.clone();
        v.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        v[(v.len() as f32 * 0.9) as usize]
    } else {
        col2_left - MIN_GUTTER_PT
    };

    let gutter = col2_left - col1_right;
    if gutter < MIN_GUTTER_PT {
        return None;
    }
    if col2_left <= col1_left + MIN_GUTTER_PT * 2.0 {
        return None;
    }

    // Symmetric column widths around the gutter.
    let total_block_w = ((col2_left - col1_left) * 2.0).max(1.0);
    let col_w = ((total_block_w - gutter) * 0.5).max(1.0);

    Some(ColumnLayout {
        count: 2,
        space_twips: Some((gutter * PT_TO_TWIPS) as u32),
        separator: false,
        column_widths_twips: vec![(col_w * PT_TO_TWIPS) as u32, (col_w * PT_TO_TWIPS) as u32],
    })
}

/// Tally exact RGB (post-rounding to u8) frequencies across every page's
/// spans. Used by `span_to_ir` to decide whether a span's color is
/// "rare and suspiciously canonical" enough to be dropped — a
/// heuristic that catches PDF link-annotation color leaks where the
/// annotation's `Color` array (typically `[0,0,1]`) gets propagated
/// onto neighboring text spans by the extractor.
fn build_color_histogram(all_spans: &[Vec<TextSpan>]) -> HashMap<[u8; 3], u32> {
    let mut counts: HashMap<[u8; 3], u32> = HashMap::new();
    for spans in all_spans {
        for s in spans {
            let r = (s.color.r * 255.0).round() as u8;
            let g = (s.color.g * 255.0).round() as u8;
            let b = (s.color.b * 255.0).round() as u8;
            if r == 0 && g == 0 && b == 0 {
                continue;
            }
            *counts.entry([r, g, b]).or_insert(0) += 1;
        }
    }
    counts
}

// ---------------------------------------------------------------------------
// Span mapping
// ---------------------------------------------------------------------------

fn spans_to_inline(
    spans: &[TextSpan],
    color_counts: &HashMap<[u8; 3], u32>,
    face_lookup: &HashMap<String, String>,
) -> Vec<InlineContent> {
    spans
        .iter()
        .map(|s| InlineContent::Text(span_to_ir(s, color_counts, face_lookup)))
        .collect()
}

fn span_to_ir(
    span: &TextSpan,
    color_counts: &HashMap<[u8; 3], u32>,
    face_lookup: &HashMap<String, String>,
) -> IrSpan {
    // Resolve resource-id aliases ("F2", "TT12") to the real PDF
    // BaseFont via the per-page font lookup before falling back to
    // `real_font_name` — which would strip the alias as a placeholder
    // and leave `font_name=None`. With the lookup in place, a CFR
    // span whose `font_name` reads "F4" surfaces in the IR as
    // "AvantGarde-Book" so the round-trip can match it against the
    // embedded font program shipped under `word/fonts/`.
    let resolved_name = face_lookup
        .get(&span.font_name)
        .cloned()
        .unwrap_or_else(|| span.font_name.clone());
    IrSpan {
        text: span.text.clone(),
        bold: span.font_weight.is_bold(),
        italic: span.is_italic,
        font_name: real_font_name(&resolved_name),
        font_size_half_pt: Some((span.font_size * 2.0).round() as u32),
        color: color_opt(&span.color, color_counts),
        char_spacing_half_pt: char_spacing_opt(span.char_spacing),
        ..Default::default()
    }
}

/// Filter out PDF resource-dictionary aliases ("F1", "F2", "TT1", etc.)
/// that pdf_oxide's extractor sometimes returns instead of the
/// PostScript / BaseFont name. Office writers happily forward whatever
/// font name they see into `<w:rFonts>` / `<a:rPr>` / cell-style
/// `font.name`; downstream readers (Word, PowerPoint, Excel) try to
/// look up the face in their installed-font cache, miss, and
/// substitute a fallback — sometimes silently, sometimes with a
/// "font is not installed" warning that scares the user. Returning
/// `None` for placeholder names lets the writers omit the font
/// attribute entirely so the document inherits the default style.
///
/// Heuristic: a placeholder is short (≤4 chars), starts with one or
/// two ASCII alphabetic letters, and the rest is purely digits.
/// Examples that match: `F1`, `F12`, `TT1`, `C0`. Examples that
/// don't: `Helvetica`, `Times-Roman`, `MIonic`, `AGMedFont`.
fn real_font_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() <= 4 {
        let bytes = trimmed.as_bytes();
        let alpha_prefix = bytes.iter().take_while(|b| b.is_ascii_alphabetic()).count();
        let digit_suffix = bytes[alpha_prefix..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if (1..=2).contains(&alpha_prefix)
            && digit_suffix >= 1
            && alpha_prefix + digit_suffix == bytes.len()
        {
            return None;
        }
    }
    Some(trimmed.to_string())
}

/// Map a PDF color to an optional RGB triple, dropping link-annotation
/// color leaks. The PDF spec lets link annotations carry a `Color`
/// array (typically `[0,0,1]` for the canonical bright blue underlined
/// link); some extractors propagate that color onto neighbouring text
/// spans even when the spans aren't actually inside the annotation
/// rectangle, producing PPTX/DOCX output where author affiliations,
/// abstract text, and body paragraphs render in saturated blue.
///
/// Heuristic:
///   - Black (`[0,0,0]`) → `None` (the IR default; saves bytes).
///   - Pure link blue (`[0,0,255]`) → `None`, always. Real intentional
///     `(0,0,255)` body text is essentially never seen in
///     business-document corpora; the false-positive cost is tiny.
///   - Other suspiciously-canonical colors (`[128,128,128]` mid-grey,
///     pure red `[255,0,0]`) → `None` only when the document-wide
///     count is `<3`, the threshold under which the color is almost
///     certainly an annotation-bleed singleton rather than intentional
///     styling (e.g. red highlight, grey caption).
///   - Anything else → carried through.
fn color_opt(c: &Color, counts: &HashMap<[u8; 3], u32>) -> Option<[u8; 3]> {
    let r = (c.r * 255.0).round() as u8;
    let g = (c.g * 255.0).round() as u8;
    let b = (c.b * 255.0).round() as u8;
    if r == 0 && g == 0 && b == 0 {
        return None;
    }
    let rgb = [r, g, b];
    // Always-drop list: the canonical PDF link blue.
    if rgb == [0, 0, 255] {
        return None;
    }
    // Drop other canonical annotation colors only when rare in the doc.
    let suspicious_canonical =
        matches!(rgb, [0x80, 0x80, 0x80] | [0xC0, 0xC0, 0xC0] | [0xFF, 0, 0] | [0, 0xFF, 0]);
    if suspicious_canonical {
        let cnt = counts.get(&rgb).copied().unwrap_or(0);
        if cnt < 3 {
            return None;
        }
    }
    Some(rgb)
}

fn char_spacing_opt(spacing_pt: f32) -> Option<i32> {
    if spacing_pt.abs() > 0.005 {
        Some((spacing_pt * 2.0).round() as i32)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn median_font_size(spans: &[TextSpan]) -> f32 {
    let mut sizes: Vec<f32> = spans.iter().map(|s| s.font_size).collect();
    sizes.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let n = sizes.len();
    if n == 0 {
        return 12.0;
    }
    if n.is_multiple_of(2) {
        (sizes[n / 2 - 1] + sizes[n / 2]) / 2.0
    } else {
        sizes[n / 2]
    }
}

#[cfg(test)]
mod merge_lines_tests {
    use super::*;

    /// One span on its own line, default geometry. `merge_lines_into_spans`
    /// only inspects `.text` at the seam, so geometry is irrelevant here.
    fn line(text: &str) -> Vec<TextSpan> {
        vec![TextSpan {
            text: text.to_string(),
            ..Default::default()
        }]
    }

    fn merged_text(lines: &[Vec<TextSpan>]) -> String {
        merge_lines_into_spans(lines)
            .iter()
            .map(|s| s.text.as_str())
            .collect()
    }

    /// §14.8.2.2.3 soft-hyphen line break of a single word: the hyphen is an
    /// incidental layout artifact — drop it and join with no space.
    #[test]
    fn soft_hyphen_lowercase_continuation_drops_hyphen() {
        assert_eq!(merged_text(&[line("captur-"), line("ing")]), "capturing");
    }

    /// A hard hyphen in a compound proper noun that happened to wrap at the
    /// hyphen (e.g. "sub-Neptune" split into "sub-" / "Neptune"). The hyphen
    /// is real content (§14.8.2.2.3 distinguishes it from the soft hyphen):
    /// keep it and, crucially, do NOT insert a space at the line seam.
    #[test]
    fn line_end_hyphen_uppercase_continuation_joins_without_space() {
        assert_eq!(merged_text(&[line("sub-"), line("Neptune")]), "sub-Neptune");
    }

    /// Two ordinary words split across lines with no whitespace on either
    /// side must gain a single separating space so they are not fused.
    #[test]
    fn missing_inter_line_space_is_inserted() {
        assert_eq!(merged_text(&[line("the"), line("Chinese")]), "the Chinese");
    }

    /// A line seam where one side already carries whitespace must be left
    /// untouched (no double space).
    #[test]
    fn existing_whitespace_seam_unchanged() {
        assert_eq!(merged_text(&[line("hello "), line("world")]), "hello world");
    }

    /// The column-left histogram inside `detect_columns` ties 4-to-4 between
    /// the x=50 and x=100 bins. `max_by_key` over the backing `HashMap`
    /// returned whichever bin the per-process-randomized iteration order
    /// visited last, so the derived column width flapped between runs. The
    /// tie must resolve to the SMALLER bin (x=50), giving a 250 pt column
    /// (5000 twips) rather than the 200 pt (4000 twips) the other bin yields.
    #[test]
    fn column_left_histogram_tie_resolves_to_smaller_bin() {
        fn span_at(x: f32, w: f32) -> TextSpan {
            TextSpan {
                text: "x".to_string(),
                bbox: crate::geometry::Rect::new(x, 0.0, w, 12.0),
                ..Default::default()
            }
        }
        // Eight two-column lines. Four start column 1 at x=50, four at x=100
        // — an exact tie. Column 1 always ends at x=250; column 2 always
        // starts at x=350, so the gutter and midline gates all pass.
        let mut lines: Vec<Vec<TextSpan>> = Vec::new();
        for i in 0..8 {
            let left_x = if i < 4 { 50.0 } else { 100.0 };
            lines.push(vec![span_at(left_x, 250.0 - left_x), span_at(350.0, 200.0)]);
        }
        let layout = detect_columns(&lines, 612.0).expect("two-column layout expected");
        assert_eq!(
            layout.column_widths_twips,
            vec![5000, 5000],
            "tie must resolve to the smaller column-left bin (x=50 → 250 pt columns)"
        );
    }
}
