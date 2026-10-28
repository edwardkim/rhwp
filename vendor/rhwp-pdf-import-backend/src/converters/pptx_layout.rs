//! PDF → PPTX with **layout-preserving** text shapes.
//!
//! Each PDF text span becomes a `<p:sp>` shape with `<a:xfrm>` at its
//! exact source EMU coordinates. PowerPoint, LibreOffice Impress, and
//! Keynote all honor this — slides are inherently positional, so each
//! shape lands at the source PDF span's (x, y, w, h) and the
//! round-trip back through `convert_pptx_bytes` →
//! `render_pptx_positional` reproduces the page near-pixel-identically
//! to the source.
//!
//! Trade-offs vs the flow path (`to_pptx_bytes` flow mode):
//! - Better visual fidelity (text positions match source exactly).
//! - Each span is its own shape — slide editing in PowerPoint becomes
//!   awkward because text isn't grouped into editable paragraphs.
//! - Currently emits one slide per source PDF page; very long PDFs
//!   produce decks above PowerPoint's "fix-the-content?" 250-slide
//!   threshold (the flow path collapses them via heading-bounded
//!   compaction; layout mode preserves 1:1).
//!
//! Tables, vector graphics, and embedded images are carried through
//! when present (images via the existing `add_image` API at the
//! source bbox; vector shapes are not yet round-tripped).

use crate::error::{Error, Result};
use office_oxide::pptx::write::{PptxWriter, Run};

/// EMUs per point. 914400 EMU/inch ÷ 72 pt/inch = 12700 EMU/pt.
const EMU_PER_PT: f32 = 12_700.0;

/// Convert a `PdfDocument` to layout-preserving PPTX bytes.
///
/// One slide per source PDF page. Within each slide, every text span
/// is emitted as a positionally-anchored shape carrying the span's
/// font/size/weight/italic/colour run properties. Embedded raster
/// images survive at their source bbox.
pub fn to_pptx_bytes_layout(doc: &crate::document::PdfDocument) -> Result<Vec<u8>> {
    let n_pages = doc.page_count()?;
    if n_pages == 0 {
        return Err(Error::InvalidOperation("PDF has zero pages".into()));
    }

    let lookups = doc.page_font_face_lookups().unwrap_or_default();

    let mut writer = PptxWriter::new();

    // Embed source-PDF fonts so PPTX viewers can render with the
    // original typeface (mirrors the flow path's behaviour).
    if let Ok(fonts) = doc.extract_embedded_fonts() {
        for (name, data) in fonts {
            writer.embed_font(name, data);
        }
    }

    // Set presentation slide size from the first page so a single
    // PDF→PPTX→PDF round-trip preserves source MediaBox dimensions.
    let (x1_0, y1_0, x2_0, y2_0) = doc.get_page_media_box(0)?;
    let page0_w_pt = (x2_0 - x1_0).abs();
    let page0_h_pt = (y2_0 - y1_0).abs();
    let pres_cx = (page0_w_pt * EMU_PER_PT) as u64;
    let pres_cy = (page0_h_pt * EMU_PER_PT) as u64;
    writer.set_presentation_size(pres_cx, pres_cy);

    for page_idx in 0..n_pages {
        let (x1, y1, x2, y2) = doc.get_page_media_box(page_idx)?;
        let page_w_pt = (x2 - x1).abs();
        let page_h_pt = (y2 - y1).abs();
        let mut spans = doc.extract_spans(page_idx).unwrap_or_default();
        // Drop rotated text (page-edge watermarks, rotated table
        // headers). Mirrors the filter in `pdf_to_ir` and
        // `docx_layout` so the arxiv watermark doesn't render as a
        // horizontal frame at the bottom of every slide.
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

        let slide = writer.add_slide();

        // Group spans into lines (by Y position). Each line becomes
        // ONE positioned text shape carrying all the line's runs in
        // order. This eliminates inter-shape overflow when PowerPoint
        // substitutes a wider fallback font for missing source faces.
        let lines = crate::converters::layout_lines::group_spans_into_lines(spans);
        for line in &lines {
            // Convert PDF y-up to PPTX y-down.
            let x_pt = line.x_pt.max(0.0).min(page_w_pt);
            let y_top_pt = (page_h_pt - line.y_pt - line.height_pt)
                .max(0.0)
                .min(page_h_pt);
            // Width: leave headroom past the source bbox so the
            // wider fallback font doesn't clip the trailing run.
            let w_pt = (line.width_pt * 1.5).max(line.width_pt + 16.0).max(8.0);
            let h_pt = line.height_pt.max(
                line.spans
                    .iter()
                    .map(|s| s.font_size)
                    .fold(0.0_f32, f32::max)
                    * 1.4,
            );

            let x_emu = (x_pt * EMU_PER_PT) as i64;
            let y_emu = (y_top_pt * EMU_PER_PT) as i64;
            let cx_emu = (w_pt * EMU_PER_PT) as i64;
            let cy_emu = (h_pt * EMU_PER_PT) as i64;

            let mut runs: Vec<Run> = Vec::with_capacity(line.spans.len());
            let mut prev_right_pt: Option<f32> = None;
            for (i, span) in line.spans.iter().enumerate() {
                let text = span.text.trim_matches('\u{0000}');
                if text.is_empty() {
                    continue;
                }

                // Insert a space when there's a visible horizontal
                // gap between this span and the previous one and
                // neither side already carries trailing/leading
                // whitespace. PDF source frequently emits adjacent
                // glyphs on the same line as separate spans without
                // an explicit space character.
                if let Some(prev_right) = prev_right_pt {
                    let gap = span.bbox.x - prev_right;
                    let needs_space = gap > span.font_size * 0.25
                        && !runs
                            .last()
                            .and_then(|r| r.text.chars().last())
                            .map(|c| c.is_whitespace())
                            .unwrap_or(false)
                        && !text.starts_with(|c: char| c.is_whitespace());
                    if needs_space && i > 0 {
                        runs.push(Run::new(" "));
                    }
                }

                let raw_font = span.font_name.as_str();
                let resolved: String = if let Some(real) = font_lookup.get(raw_font) {
                    real.clone()
                } else {
                    let stripped = raw_font
                        .split_once('+')
                        .map(|(_, rest)| rest)
                        .unwrap_or(raw_font);
                    if !stripped.is_empty() && stripped.chars().any(char::is_alphabetic) {
                        stripped.to_string()
                    } else if span.is_monospace {
                        "Courier New".to_string()
                    } else {
                        "Times New Roman".to_string()
                    }
                };

                let mut run = Run::new(text)
                    .font(resolved)
                    .font_size(span.font_size as f64);
                if span.font_weight.is_bold() {
                    run = run.bold();
                }
                if span.is_italic {
                    run = run.italic();
                }
                if !crate::color::is_document_black(span.color.r, span.color.g, span.color.b) {
                    let hex = format!(
                        "{:02X}{:02X}{:02X}",
                        (span.color.r * 255.0).round().clamp(0.0, 255.0) as u8,
                        (span.color.g * 255.0).round().clamp(0.0, 255.0) as u8,
                        (span.color.b * 255.0).round().clamp(0.0, 255.0) as u8,
                    );
                    run = run.color(hex);
                }

                runs.push(run);
                prev_right_pt = Some(span.bbox.x + span.bbox.width);
            }

            if !runs.is_empty() {
                slide.add_rich_text_box(&runs, x_emu, y_emu, cx_emu, cy_emu);
            }
        }

        // Images: copy each PDF image at its bbox onto the slide.
        // Track existing raster bboxes (PDF y-up) so the
        // Form-XObject pass below can skip duplicates.
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
                slide.add_image(
                    png,
                    office_oxide::ir::ImageFormat::Png,
                    (x_pt * EMU_PER_PT) as i64,
                    (y_top_pt * EMU_PER_PT) as i64,
                    (w_pt * EMU_PER_PT) as u64,
                    (h_pt * EMU_PER_PT) as u64,
                );
            }
        }

        // Rasterise Form-XObject + inline-image regions onto the
        // slide. Required to preserve vector figures (charts,
        // logos drawn as Form XObjects) — without this the
        // layout-mode PPTX path drops every such figure.
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
                slide.add_image(
                    png,
                    office_oxide::ir::ImageFormat::Png,
                    (x_pt * EMU_PER_PT) as i64,
                    (y_top_pt * EMU_PER_PT) as i64,
                    (w_pt * EMU_PER_PT) as u64,
                    (h_pt * EMU_PER_PT) as u64,
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
                slide.add_image(
                    png,
                    office_oxide::ir::ImageFormat::Png,
                    (x_pt * EMU_PER_PT) as i64,
                    (y_top_pt * EMU_PER_PT) as i64,
                    (w_pt * EMU_PER_PT) as u64,
                    (h_pt * EMU_PER_PT) as u64,
                );
            }
        }
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    writer
        .write_to(&mut buf)
        .map_err(|e| Error::InvalidOperation(format!("PPTX layout export: {e}")))?;
    Ok(buf.into_inner())
}

/// Merge consecutive spans that form a hyphenated word at a line
/// boundary (e.g. `["captur-", "ing"]` → `"capturing"`). Mirrors the
/// helper in `docx_layout` but kept private here to avoid a
/// cross-module dependency that would require making the helper `pub`.
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
            // Merge: drop trailing '-', concatenate, expand bbox.
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
