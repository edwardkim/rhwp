//! PDF → XLSX with **layout-preserving** drawing-anchored shapes.
//!
//! Each PDF page becomes one worksheet. Each PDF text span is emitted
//! as an `<xdr:sp>` text shape inside `xl/drawings/drawingN.xml` at
//! its exact source EMU coordinates. Embedded raster images are
//! emitted as `<xdr:pic>` shapes alongside the text shapes.
//!
//! Excel, LibreOffice Calc, and the OpenXML SDK all honor absolute
//! drawing anchors so the round-trip
//! `convert_xlsx_bytes` → `ir_to_pdf_bytes` → positional `render_text_box`
//! reproduces the source page near-pixel-identically.
//!
//! Trade-offs vs the flow path (`to_xlsx_bytes` flow mode):
//! - Better visual fidelity (every span renders at source x/y).
//! - The worksheet's cell grid is empty — content lives entirely in
//!   the drawing layer. Spreadsheet-aware tooling (filters, sorts,
//!   formulas) sees no data; this is intentional for a "PDF as
//!   XLSX" view.
//! - One worksheet per PDF page.

use crate::error::{Error, Result};
use office_oxide::xlsx::write::{PageSetup, XlsxWriter};

/// EMUs per point. 914400 EMU/inch ÷ 72 pt/inch = 12700 EMU/pt.
const EMU_PER_PT: f32 = 12_700.0;
/// Twips per point. 1 pt = 20 twips.
const TWIPS_PER_PT: f32 = 20.0;

/// Convert a `PdfDocument` to layout-preserving XLSX bytes.
///
/// One worksheet per source PDF page. Within each worksheet, every
/// text span is emitted as a positionally-anchored `<xdr:sp>` text
/// shape carrying the span's font/size/weight/italic/colour run
/// properties. Embedded raster images survive at their source bbox.
pub fn to_xlsx_bytes_layout(doc: &crate::document::PdfDocument) -> Result<Vec<u8>> {
    let n_pages = doc.page_count()?;
    if n_pages == 0 {
        return Err(Error::InvalidOperation("PDF has zero pages".into()));
    }

    let lookups = doc.page_font_face_lookups().unwrap_or_default();

    let mut writer = XlsxWriter::new();

    if let Ok(fonts) = doc.extract_embedded_fonts() {
        for (name, data) in fonts {
            writer.embed_font(name, data);
        }
    }

    for page_idx in 0..n_pages {
        let (x1, y1, x2, y2) = doc.get_page_media_box(page_idx)?;
        let page_w_pt = (x2 - x1).abs();
        let page_h_pt = (y2 - y1).abs();
        let mut spans = doc.extract_spans(page_idx).unwrap_or_default();
        // Drop rotated text (page-edge watermarks, rotated table
        // headers). Mirrors the filter in `pdf_to_ir`.
        if let Ok(chars) = doc.extract_chars(page_idx) {
            let chars_horizontal_dominant = if chars.is_empty() {
                true
            } else {
                let horiz = chars
                    .iter()
                    .filter(|c| c.rotation_degrees.abs() < 5.0)
                    .count();
                horiz * 4 >= chars.len() * 3
            };
            spans.retain(|s| {
                !crate::converters::pdf_to_ir::span_overlaps_rotated_chars(
                    s,
                    &chars,
                    chars_horizontal_dominant,
                )
            });
        }
        // Detect music-notation regions; suppress spans whose centre
        // lies inside one. See `docx_layout.rs` for rationale.
        let music_regions =
            crate::converters::music_region_finder::find_music_regions(doc, page_idx);
        if !music_regions.is_empty() {
            spans.retain(|s| {
                !music_regions
                    .iter()
                    .any(|r| crate::converters::music_region_finder::rect_contains_bbox(r, &s.bbox))
            });
        }
        merge_hyphenated_spans(&mut spans);

        let font_lookup = lookups.get(page_idx).cloned().unwrap_or_default();

        let sheet_name = format!("Page {}", page_idx + 1);
        let mut sheet = writer.add_sheet(&sheet_name);

        // Per-worksheet page setup: source MediaBox so a PDF→XLSX→PDF
        // round-trip preserves dimensions instead of snapping to
        // Letter. Margins are zero so positional shapes can land
        // anywhere on the page.
        sheet.set_page_setup(PageSetup {
            width_twips: (page_w_pt * TWIPS_PER_PT) as u32,
            height_twips: (page_h_pt * TWIPS_PER_PT) as u32,
            margin_top_twips: 0,
            margin_bottom_twips: 0,
            margin_left_twips: 0,
            margin_right_twips: 0,
            header_distance_twips: 0,
            footer_distance_twips: 0,
            landscape: page_w_pt > page_h_pt,
        });

        // Group spans into lines by Y position. One `<xdr:sp>` per
        // line carrying the joined-and-spaced text — see
        // `layout_lines::group_spans_into_lines` for rationale.
        // XLSX text shapes don't currently support multi-run styling
        // through `add_text_shape`; we fall back to using the
        // dominant style on the line (first non-empty span) so the
        // line still renders correctly. Multi-run styled XLSX shapes
        // would need a richer writer API and are tracked separately.
        let lines = crate::converters::layout_lines::group_spans_into_lines(spans);
        for line in &lines {
            // Build the line's joined text with synthetic spaces
            // bridging visible inter-span gaps the source PDF didn't
            // explicitly encode.
            let mut joined = String::new();
            let mut prev_right_pt: Option<f32> = None;
            let mut style_span: Option<&crate::layout::text_block::TextSpan> = None;
            for span in &line.spans {
                let text = span.text.trim_matches('\u{0000}');
                if text.is_empty() {
                    continue;
                }
                if let Some(prev_right) = prev_right_pt {
                    let gap = span.bbox.x - prev_right;
                    let needs_space = gap > span.font_size * 0.25
                        && !joined
                            .chars()
                            .last()
                            .map(|c| c.is_whitespace())
                            .unwrap_or(false)
                        && !text.starts_with(|c: char| c.is_whitespace());
                    if needs_space {
                        joined.push(' ');
                    }
                }
                joined.push_str(text);
                prev_right_pt = Some(span.bbox.x + span.bbox.width);
                if style_span.is_none() {
                    style_span = Some(span);
                }
            }

            let trimmed_joined = joined.trim_matches('\u{0000}');
            if trimmed_joined.is_empty() {
                continue;
            }
            let style = match style_span {
                Some(s) => s,
                None => continue,
            };

            let x_pt = line.x_pt.max(0.0).min(page_w_pt);
            let y_top_pt = (page_h_pt - line.y_pt - line.height_pt)
                .max(0.0)
                .min(page_h_pt);
            // Pad width 1.5× to absorb fallback-font widening and
            // avoid clipping the trailing glyphs.
            let w_pt = (line.width_pt * 1.5).max(line.width_pt + 16.0).max(8.0);
            let h_pt = line.height_pt.max(style.font_size * 1.4);

            let raw_font = style.font_name.as_str();
            let resolved: String = if let Some(real) = font_lookup.get(raw_font) {
                real.clone()
            } else {
                let stripped = raw_font
                    .split_once('+')
                    .map(|(_, rest)| rest)
                    .unwrap_or(raw_font);
                if !stripped.is_empty() && stripped.chars().any(char::is_alphabetic) {
                    stripped.to_string()
                } else if style.is_monospace {
                    "Courier New".to_string()
                } else {
                    "Times New Roman".to_string()
                }
            };

            let color_hex =
                if crate::color::is_document_black(style.color.r, style.color.g, style.color.b) {
                    None
                } else {
                    Some(format!(
                        "{:02X}{:02X}{:02X}",
                        (style.color.r * 255.0).round().clamp(0.0, 255.0) as u8,
                        (style.color.g * 255.0).round().clamp(0.0, 255.0) as u8,
                        (style.color.b * 255.0).round().clamp(0.0, 255.0) as u8,
                    ))
                };

            sheet.add_text_shape(
                trimmed_joined,
                resolved,
                style.font_size,
                style.font_weight.is_bold(),
                style.is_italic,
                color_hex,
                (x_pt * EMU_PER_PT) as i64,
                (y_top_pt * EMU_PER_PT) as i64,
                (w_pt * EMU_PER_PT) as i64,
                (h_pt * EMU_PER_PT) as i64,
            );
        }

        // Per-page images at bbox. Track raster bboxes (PDF y-up)
        // so the Form-XObject pass below can skip duplicates.
        let mut existing_rects_pdf: Vec<(f32, f32, f32, f32)> = Vec::new();
        if let Ok(imgs) = doc.extract_images(page_idx) {
            for img in imgs {
                let bbox = match img.bbox() {
                    Some(b) => *b,
                    None => continue,
                };
                let png = match img.to_png_bytes() {
                    Ok(b) if !b.is_empty() => b,
                    _ => continue,
                };
                let x_pt = bbox.x.max(0.0).min(page_w_pt);
                let y_top_pt = (page_h_pt - bbox.y - bbox.height).max(0.0);
                let y_top_pt = y_top_pt.min(page_h_pt);
                let w_pt = bbox.width.max(1.0);
                let h_pt = bbox.height.max(1.0);
                existing_rects_pdf.push((bbox.x, bbox.y, bbox.width, bbox.height));
                sheet.add_image(
                    png,
                    "png",
                    (x_pt * EMU_PER_PT) as i64,
                    (y_top_pt * EMU_PER_PT) as i64,
                    (w_pt * EMU_PER_PT) as i64,
                    (h_pt * EMU_PER_PT) as i64,
                );
            }
        }

        // Rasterise Form-XObject + inline-image regions onto the
        // sheet so vector figures survive the round-trip. Mirrors
        // the supplemental pass in `pdf_to_ir`, `docx_layout`,
        // `pptx_layout`.
        #[cfg(feature = "rendering")]
        {
            let regions = crate::converters::form_xobject_finder::rasterize_form_and_inline_regions(
                doc,
                page_idx,
                page_h_pt,
                &existing_rects_pdf,
            );
            for ((x_pdf, y_pdf, w, h), png) in regions {
                let x_pt = x_pdf.max(0.0).min(page_w_pt);
                let y_top_pt = (page_h_pt - y_pdf - h).max(0.0).min(page_h_pt);
                let w_pt = w.max(1.0);
                let h_pt = h.max(1.0);
                sheet.add_image(
                    png,
                    "png",
                    (x_pt * EMU_PER_PT) as i64,
                    (y_top_pt * EMU_PER_PT) as i64,
                    (w_pt * EMU_PER_PT) as i64,
                    (h_pt * EMU_PER_PT) as i64,
                );
            }
        }

        // Rasterise music-notation regions (hymnals, sheet music).
        // The matching spans have already been dropped above so the
        // bitmap is what the recipient sees. See `docx_layout.rs`
        // for rationale.
        #[cfg(feature = "rendering")]
        if !music_regions.is_empty() {
            let regions = crate::converters::music_region_finder::rasterize_music_regions(
                doc, page_idx, page_h_pt,
            );
            for ((x_pdf, y_pdf, w, h), png) in regions {
                let x_pt = x_pdf.max(0.0).min(page_w_pt);
                let y_top_pt = (page_h_pt - y_pdf - h).max(0.0).min(page_h_pt);
                let w_pt = w.max(1.0);
                let h_pt = h.max(1.0);
                sheet.add_image(
                    png,
                    "png",
                    (x_pt * EMU_PER_PT) as i64,
                    (y_top_pt * EMU_PER_PT) as i64,
                    (w_pt * EMU_PER_PT) as i64,
                    (h_pt * EMU_PER_PT) as i64,
                );
            }
        }
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    writer
        .write_to(&mut buf)
        .map_err(|e| Error::InvalidOperation(format!("XLSX layout export: {e}")))?;
    Ok(buf.into_inner())
}

/// Merge consecutive spans that form a hyphenated word at a line
/// boundary. Mirrors the helper in `docx_layout` / `pptx_layout`.
fn merge_hyphenated_spans(spans: &mut Vec<crate::layout::text_block::TextSpan>) {
    if spans.len() < 2 {
        return;
    }
    // Single forward pass with a running accumulator (was O(n^2) via
    // Vec::remove + no-advance re-scan). Byte-identical to the loop above.
    let mut out: Vec<crate::layout::text_block::TextSpan> = Vec::with_capacity(spans.len());
    let mut iter = std::mem::take(spans).into_iter();
    let mut cur = iter.next().expect("len >= 2 checked above");
    for next in iter {
        let curr_ends_hyphen = cur.text.ends_with('-');
        let same_size = (cur.font_size - next.font_size).abs() < 0.01;
        let next_starts_lower = next
            .text
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false);
        if curr_ends_hyphen && same_size && next_starts_lower {
            let merged_text = format!("{}{}", &cur.text[..cur.text.len() - 1], next.text);
            cur.text = merged_text;
            cur.bbox.width += next.bbox.width;
        } else {
            out.push(cur);
            cur = next;
        }
    }
    out.push(cur);
    *spans = out;
}
