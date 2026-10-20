//! Detects music-notation regions on a PDF page so the layout-mode
//! writers can rasterize them and suppress the underlying spans/shapes
//! that would otherwise emit as garbage glyph substitutions or be
//! dropped entirely.
//!
//! Hymnal / sheet-music PDFs encode their staves, noteheads, stems and
//! accidentals as a mixture of music-notation font glyphs (Maestro,
//! Bravura, Sonata, …) and stroked horizontal staff lines. Through a
//! layout-mode round-trip neither survives intact: the music fonts
//! aren't installed where the docx/pptx/xlsx is later rendered, so
//! Word / PowerPoint / Calc substitute Times / Calibri and the
//! noteheads come out as random Latin letters; the staff lines
//! survive as vector shapes but no longer align with the (now
//! garbled) noteheads.
//!
//! This module detects the bounding boxes that contain music notation
//! so the layout writers can: (a) rasterize those regions as bitmaps
//! and embed them as images, and (b) drop the underlying spans /
//! horizontal staff-line shapes that would otherwise overlap the
//! image with garbled content.

use crate::document::PdfDocument;
use crate::geometry::Rect;

/// Music-notation font names commonly seen in hymnals / sheet music.
/// Matching is **substring**, case-insensitive, on the source PDF's
/// font BaseFont (post-subset-prefix stripping). Limited to fonts
/// that are unambiguously music notation — listing general-purpose
/// fonts here would suppress legitimate text.
pub(crate) const MUSIC_FONT_NEEDLES: &[&str] = &[
    "maestro",        // Finale's default music font
    "bravura",        // SMuFL reference font
    "petrucci",       // Sibelius (historical)
    "opus",           // Sibelius
    "sonata",         // Adobe music font
    "emmentaler",     // LilyPond's default
    "musicalsymbols", // Unicode-block fallback
    "engravertext",   // Finale text-on-staff (often paired with Maestro)
    "noteheadgroup",  // SMuFL noteheads-only fonts
];

/// Returns `true` when `font_name` looks like a music-notation font.
pub(crate) fn is_music_font_name(font_name: &str) -> bool {
    let lower = font_name.to_ascii_lowercase();
    // Strip 6-letter subset prefix (e.g. "XVSURQ+Maestro").
    let core = lower.split_once('+').map(|(_, r)| r).unwrap_or(&lower);
    MUSIC_FONT_NEEDLES.iter().any(|n| core.contains(n))
}

/// Compute axis-aligned bboxes for the music-notation regions on
/// `page_idx`. Each returned `Rect` is in PDF user space (origin
/// bottom-left, y-up). Empty Vec when no music regions are detected.
///
/// Detection combines two cheap signals so we don't false-positive:
/// (1) at least one span on the page uses a music-notation font
///     (per [`is_music_font_name`]);
/// (2) at least 5 horizontal stroked paths (staff lines) cluster
///     into a tight y-band (≤ 25 pt).
///
/// Both signals must be present somewhere on the page. Then we
/// expand each cluster by 25 pt top/bottom to capture noteheads,
/// stems, slurs above and below the staff, and union overlapping
/// clusters into "music systems".
pub(crate) fn find_music_regions(doc: &PdfDocument, page_idx: usize) -> Vec<Rect> {
    // Signal 1: any span on the page using a music-notation font?
    // `span.font_name` is the PDF resource id (e.g. "TT0"); resolve
    // through `page_font_face_lookups` to the actual BaseFont before
    // checking. Without this step the music-font allowlist never
    // matches because resource ids look nothing like font names.
    let spans = match doc.extract_spans(page_idx) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let lookups = doc.page_font_face_lookups().unwrap_or_default();
    let page_lookup = lookups.get(page_idx);
    let resolve = |resource_id: &str| -> String {
        page_lookup
            .and_then(|m| m.get(resource_id).cloned())
            .unwrap_or_else(|| resource_id.to_string())
    };
    let has_music_font = spans
        .iter()
        .any(|s| is_music_font_name(&resolve(&s.font_name)));
    if !has_music_font {
        return Vec::new();
    }

    // Signal 2: collect horizontal stroked staff-line candidates.
    // A staff line has near-zero height and substantial width.
    let paths = match doc.extract_paths(page_idx) {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    #[derive(Clone, Copy)]
    struct HLine {
        x_min: f32,
        x_max: f32,
        y: f32,
    }
    let mut hlines: Vec<HLine> = Vec::new();
    for p in &paths {
        // Geometric thinness (a staff line has height ≤ 1 pt) with the
        // length taken from the RENDERED extents so stroke-width-encoded
        // lines qualify too. Works whether the source emits
        // a MoveTo+LineTo pair or a degenerate rect.
        let rendered = p.rendered_bbox();
        if p.bbox.height <= 1.0 && rendered.width > 50.0 {
            hlines.push(HLine {
                x_min: rendered.x,
                x_max: rendered.x + rendered.width,
                y: rendered.y + rendered.height * 0.5,
            });
        }
    }
    if hlines.len() < 5 {
        return Vec::new();
    }

    // Sort lines by y (ascending = bottom-up in PDF space) and group
    // consecutive lines whose vertical gap is ≤ 6 pt.
    hlines.sort_by(|a, b| crate::utils::safe_float_cmp(a.y, b.y));

    struct Cluster {
        y_min: f32,
        y_max: f32,
        x_min: f32,
        x_max: f32,
        n: usize,
    }
    let mut clusters: Vec<Cluster> = Vec::new();
    for hl in &hlines {
        let extend = clusters.last_mut().is_some_and(|c| (hl.y - c.y_max) <= 6.0);
        if extend {
            let c = clusters.last_mut().unwrap();
            c.y_max = hl.y;
            // X extent of a staff is the intersection of its lines'
            // x ranges — gives the actual staff width, not the union
            // including stray ledger-line marks.
            c.x_min = c.x_min.max(hl.x_min);
            c.x_max = c.x_max.min(hl.x_max);
            c.n += 1;
        } else {
            clusters.push(Cluster {
                y_min: hl.y,
                y_max: hl.y,
                x_min: hl.x_min,
                x_max: hl.x_max,
                n: 1,
            });
        }
    }

    // A staff is exactly 5 lines, so require ≥ 5 lines per cluster
    // and a vertical span ≤ 25 pt (otherwise it's noise, not a staff).
    let staves: Vec<Cluster> = clusters
        .into_iter()
        .filter(|c| c.n >= 5 && (c.y_max - c.y_min) <= 25.0 && c.x_max > c.x_min)
        .collect();
    if staves.is_empty() {
        return Vec::new();
    }

    // Expand each staff by 25 pt above and below to capture
    // noteheads, stems, slurs, dynamics text, etc.
    let mut regions: Vec<Rect> = staves
        .into_iter()
        .map(|c| Rect::new(c.x_min, c.y_min - 25.0, c.x_max - c.x_min, (c.y_max - c.y_min) + 50.0))
        .collect();

    // Union staves that overlap (or nearly overlap) in y to form
    // "music systems" (treble + bass staves on one line of music).
    // We DON'T union across the 80-pt vertical gap between systems —
    // the 5-pt slack here is well below that gap.
    regions.sort_by(|a, b| crate::utils::safe_float_cmp(a.y, b.y));
    let mut merged: Vec<Rect> = Vec::new();
    for r in regions {
        let unioned = merged.last_mut().and_then(|m| {
            // y-overlap within 5 pt slack?
            let m_top = m.y + m.height;
            let r_top = r.y + r.height;
            if r.y <= m_top + 5.0 {
                let new_y = m.y.min(r.y);
                let new_top = m_top.max(r_top);
                let new_x = m.x.min(r.x);
                let new_right = (m.x + m.width).max(r.x + r.width);
                *m = Rect::new(new_x, new_y, new_right - new_x, new_top - new_y);
                Some(())
            } else {
                None
            }
        });
        if unioned.is_none() {
            merged.push(r);
        }
    }

    merged
}

/// Rasterise the music regions found on `page_idx` at 150 DPI as
/// PNG bytes. Returns each region paired with its PDF user-space
/// bbox `(x_pdf, y_pdf, w, h)` so callers can convert to
/// office-coord systems (y-down) themselves.
///
/// Mirrors `form_xobject_finder::rasterize_form_and_inline_regions`:
/// renders the page once, crops in image space per region. Caller
/// gets the same `(bbox, png)` shape so plumbing into the layout
/// writers' existing image-emission code is trivial.
#[cfg(feature = "rendering")]
pub(crate) fn rasterize_music_regions(
    doc: &PdfDocument,
    page_idx: usize,
    page_h_pt: f32,
) -> Vec<((f32, f32, f32, f32), Vec<u8>)> {
    use crate::rendering::{render_page, ImageFormat as RFmt, RenderOptions};

    let regions = find_music_regions(doc, page_idx);
    if regions.is_empty() {
        return Vec::new();
    }

    // Re-open the source bytes as a mutable doc — `render_page`
    // takes `&mut PdfDocument` because rendering mutates internal
    // caches. Same pattern as the form-xobject rasterizer.
    let bytes = doc.source_bytes.clone();
    if bytes.is_empty() {
        return Vec::new();
    }
    let doc_mut = match crate::document::PdfDocument::from_bytes(bytes) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let dpi: u32 = 150;
    let opts = RenderOptions {
        dpi,
        format: RFmt::Png,
        ..Default::default()
    };
    let full = match render_page(&doc_mut, page_idx, &opts) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };
    let full_img = match image::load_from_memory(&full.data) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };
    let scale = dpi as f32 / 72.0;
    let img_w = full_img.width();
    let img_h = full_img.height();

    let mut out = Vec::with_capacity(regions.len());
    for r in regions {
        let (x_pdf, y_pdf, w, h) = (r.x, r.y, r.width, r.height);
        // PDF y-up → image y-down. Image origin is top-left.
        let top_y_pt = page_h_pt - (y_pdf + h);
        let cx = (x_pdf * scale).round().max(0.0) as u32;
        let cy = (top_y_pt * scale).round().max(0.0) as u32;
        let cw = (w * scale).round().max(1.0) as u32;
        let ch = (h * scale).round().max(1.0) as u32;
        let x = cx.min(img_w.saturating_sub(1));
        let y = cy.min(img_h.saturating_sub(1));
        let cw = cw.min(img_w - x);
        let ch = ch.min(img_h - y);
        if cw == 0 || ch == 0 {
            continue;
        }
        let cropped = full_img.crop_imm(x, y, cw, ch);
        let mut buf = Vec::new();
        use image::codecs::png::{CompressionType, FilterType, PngEncoder};
        use image::ImageEncoder;
        if PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, FilterType::Sub)
            .write_image(cropped.as_bytes(), cw, ch, cropped.color().into())
            .is_err()
        {
            continue;
        }
        if buf.is_empty() {
            continue;
        }
        out.push(((x_pdf, y_pdf, w, h), buf));
    }
    out
}

/// Returns `true` when the centre point `(cx, cy)` falls inside `region`.
pub(crate) fn rect_contains_point(region: &Rect, cx: f32, cy: f32) -> bool {
    cx >= region.x
        && cx <= region.x + region.width
        && cy >= region.y
        && cy <= region.y + region.height
}

/// Returns `true` when the centre of `bbox` falls inside `region`.
/// Centre-point containment (not full-bbox-inside) — lyrics that
/// extend partly under a staff are kept, while noteheads / glyphs
/// whose centre lies inside the staff region are suppressed.
pub(crate) fn rect_contains_bbox(region: &Rect, bbox: &Rect) -> bool {
    let cx = bbox.x + bbox.width * 0.5;
    let cy = bbox.y + bbox.height * 0.5;
    cx >= region.x
        && cx <= region.x + region.width
        && cy >= region.y
        && cy <= region.y + region.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_font_needles_match_case_insensitive() {
        assert!(is_music_font_name("Maestro"));
        assert!(is_music_font_name("maestro"));
        assert!(is_music_font_name("MAESTRO"));
        assert!(is_music_font_name("XVSURQ+Maestro"));
        assert!(is_music_font_name("ABCDEF+Bravura-Regular"));
        assert!(is_music_font_name("Sonata"));
        assert!(is_music_font_name("Emmentaler-20"));
    }

    #[test]
    fn music_font_needles_reject_general_fonts() {
        assert!(!is_music_font_name("Times New Roman"));
        assert!(!is_music_font_name("Calibri"));
        assert!(!is_music_font_name("Helvetica"));
        assert!(!is_music_font_name("ABCDEF+TeXGyreTermes-Regular"));
        assert!(!is_music_font_name("Arial"));
        assert!(!is_music_font_name(""));
    }
}
