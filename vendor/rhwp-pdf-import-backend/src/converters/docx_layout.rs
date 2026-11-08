//! PDF → DOCX with **layout-preserving** text frames.
//!
//! Each PDF text span becomes a `<w:p>` paragraph wrapped in a
//! `<w:framePr>` with absolute coordinates anchored to the page. Word /
//! LibreOffice / OnlyOffice all honor this and render the result
//! visually similar to the source PDF — same fonts, sizes, colors, and
//! positions, but the text remains real selectable / editable text
//! (unlike the rasterization approach which produces pixel-only output).
//!
//! Trade-offs:
//!   - Works well for text-heavy documents with simple layouts
//!     (academic papers, forms, single/multi-column reports).
//!   - Doesn't reconstruct table grids, vector graphics, or embedded
//!     images (those would need additional work).
//!   - Word's frame engine handles positioning; large numbers of
//!     overlapping frames can be slow to lay out.
//!
//! The output DOCX is a flat zip containing exactly the parts Word needs:
//! `[Content_Types].xml`, `_rels/.rels`, `word/document.xml`, and
//! `word/_rels/document.xml.rels`.

use crate::error::{Error, Result};
use crate::layout::text_block::TextSpan;
use office_oxide::core::opc::{OpcWriter, PartName};
use office_oxide::core::relationships::rel_types;
use std::collections::HashMap;
use std::io::Cursor;

/// Twips per point. 1 pt = 20 twips (Word's basic unit).
const TWIPS_PER_PT: f32 = 20.0;
/// EMUs per point. 1 pt = 12 700 EMU.
const EMU_PER_PT: f64 = 12_700.0;

/// Convert a `PdfDocument` to layout-preserving DOCX bytes.
///
/// Iterates every page, extracts text spans with their bounding boxes,
/// and emits each as a positioned `w:framePr` paragraph in `word/document.xml`.
/// The DOCX page size is set to match the source PDF so frame coordinates
/// (in twips, 1/20 pt) map 1:1.
pub fn to_docx_bytes_layout(doc: &crate::document::PdfDocument) -> Result<Vec<u8>> {
    let n_pages = doc.page_count()?;
    if n_pages == 0 {
        return Err(Error::InvalidOperation("PDF has zero pages".into()));
    }

    // Build a per-page resource-name → real-face-name lookup so spans
    // referencing PDF font resource ids (e.g. "F75") can be emitted with
    // the actual font face that ships in the source PDF (e.g.
    // "TeXGyreTermesX-Regular"). `extract_spans` returns the resource id;
    // `page_font_face_lookups` resolves those to BaseFont names.
    let lookups = doc.page_font_face_lookups().unwrap_or_default();

    // Initialize OPC writer up-front so we can reserve image rIds in order
    // and use them in the body XML below.
    let cursor = Cursor::new(Vec::new());
    let mut opc =
        OpcWriter::new(cursor).map_err(|e| Error::InvalidOperation(format!("opc init: {e}")))?;
    let doc_part = PartName::new("/word/document.xml")
        .map_err(|e| Error::InvalidOperation(format!("part name: {e}")))?;
    opc.add_package_rel(rel_types::OFFICE_DOCUMENT, "word/document.xml");

    let mut pages: Vec<PageSpans> = Vec::with_capacity(n_pages);
    let mut media_idx = 0usize;
    for i in 0..n_pages {
        let (x1, y1, x2, y2) = doc.get_page_media_box(i)?;
        let w_pt = (x2 - x1).abs();
        let h_pt = (y2 - y1).abs();
        let mut spans = doc.extract_spans(i).unwrap_or_default();
        // Drop rotated text (page-edge watermarks like the arxiv
        // ID, rotated table headers). TextSpan carries no rotation
        // field but `extract_chars` does; identify rotated spans by
        // overlapping their bbox with rotated chars and drop them.
        // Mirrors the same filter in `pdf_to_ir`. Without this the
        // arxiv watermark renders as a horizontal text frame in the
        // wrong place.
        if let Ok(chars) = doc.extract_chars(i) {
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
        // Detect music-notation regions (hymnals, sheet music). When
        // present, drop spans whose centre falls inside a music
        // region — their music-font glyphs would otherwise emit as
        // garbled Latin letters because the recipient lacks the
        // Maestro / Bravura / Sonata face. The regions are
        // rasterised below and embedded as bitmap images instead.
        let music_regions = crate::converters::music_region_finder::find_music_regions(doc, i);
        if !music_regions.is_empty() {
            spans.retain(|s| {
                !music_regions
                    .iter()
                    .any(|r| crate::converters::music_region_finder::rect_contains_bbox(r, &s.bbox))
            });
        }
        merge_hyphenated_spans(&mut spans);
        // Stamp heading_level on spans whose font size is a heading-class
        // ratio above the page's body text. Tagged-PDF struct_role would
        // be more precise but isn't currently exposed on TextSpan; the
        // ratio heuristic matches what the existing markdown / pdf_to_ir
        // converters use (1.75 / 1.35 / 1.15× modal body size).
        annotate_heading_levels(&mut spans);
        let font_lookup = lookups.get(i).cloned().unwrap_or_default();

        // Extract images on this page; for each, write the PNG to
        // `word/media/imageN.png` and register an IMAGE relationship.
        // The returned rId is what the body XML's `r:embed="…"` references.
        let mut page_images: Vec<PageImage> = Vec::new();
        if let Ok(imgs) = doc.extract_images(i) {
            for img in imgs {
                let bbox = match img.bbox() {
                    Some(b) => *b,
                    None => continue,
                };
                let png = match img.to_png_bytes() {
                    Ok(b) if !b.is_empty() => b,
                    _ => continue,
                };
                media_idx += 1;
                let target = format!("/word/media/image{}.png", media_idx);
                let part = PartName::new(&target)
                    .map_err(|e| Error::InvalidOperation(format!("part name {target}: {e}")))?;
                opc.add_part(&part, "image/png", &png)
                    .map_err(|e| Error::InvalidOperation(format!("opc add_part {target}: {e}")))?;
                let rid = opc.add_part_rel(
                    &doc_part,
                    rel_types::IMAGE,
                    &format!("media/image{}.png", media_idx),
                );
                page_images.push(PageImage { bbox, rid });
            }
        }

        // Rasterise Form-XObject + inline-image regions (academic-
        // paper figures, agency logos, accessibility marks) into
        // `word/media/` so the round-trip preserves them. Without
        // this the layout-mode DOCX path drops every figure drawn as
        // a vector Form XObject. Mirrors the supplemental pass in
        // `pdf_to_ir::extract_page_images`.
        #[cfg(feature = "rendering")]
        {
            let existing: Vec<(f32, f32, f32, f32)> = page_images
                .iter()
                .map(|pi| (pi.bbox.x, pi.bbox.y, pi.bbox.width, pi.bbox.height))
                .collect();
            let regions = crate::converters::form_xobject_finder::rasterize_form_and_inline_regions(
                doc, i, h_pt, &existing,
            );
            for ((x_pdf, y_pdf, w, h), png) in regions {
                media_idx += 1;
                let target = format!("/word/media/image{}.png", media_idx);
                let part = PartName::new(&target)
                    .map_err(|e| Error::InvalidOperation(format!("part name {target}: {e}")))?;
                opc.add_part(&part, "image/png", &png)
                    .map_err(|e| Error::InvalidOperation(format!("opc add_part {target}: {e}")))?;
                let rid = opc.add_part_rel(
                    &doc_part,
                    rel_types::IMAGE,
                    &format!("media/image{}.png", media_idx),
                );
                page_images.push(PageImage {
                    bbox: crate::geometry::Rect::new(x_pdf, y_pdf, w, h),
                    rid,
                });
            }
        }

        // Extract drawing paths and reduce to simple shapes (lines / rects).
        // Table borders, underlines, dividers, page frames all live here.
        let mut shapes: Vec<SimpleShape> = Vec::new();
        if let Ok(paths) = doc.extract_paths(i) {
            for p in paths {
                shapes.extend(simplify_path(&p));
            }
        }
        // Drop shape artefacts from `extract_paths` that would
        // wipe the DOCX page:
        // (a) Bbox dramatically larger than the page (e.g. 538×2340pt
        //     on an 841pt page). 20% bleed margin handles legitimate
        //     page-frame rects.
        // (b) Solid-black filled rect covering >25% of the page —
        //     `extract_paths` sometimes reports a near-full-page
        //     filled rect for what's actually a clipping / frame
        //     path; the source slide has a white background, so the
        //     "black" fill is a parser artefact.
        // (c) Solid-white filled rect covering >50% of the page — a
        //     few source PDFs emit a full-page white rect as the
        //     page background BEFORE the text, but the docx round-
        //     trip emits shapes AFTER text so the white rect
        //     occludes the rendered text. The page is already white
        //     in the round-trip output, so dropping this is the
        //     cleanest fix.
        let page_w_max = w_pt * 1.2;
        let page_h_max = h_pt * 1.2;
        let page_area = w_pt * h_pt;
        shapes.retain(|s| match s {
            SimpleShape::Rect { bbox, fill_rgb, .. } => {
                if bbox.width > page_w_max || bbox.height > page_h_max {
                    return false;
                }
                let area = bbox.width * bbox.height;
                if page_area <= 0.0 {
                    return true;
                }
                let area_ratio = area / page_area;
                if matches!(fill_rgb, Some((0, 0, 0))) && area_ratio > 0.25 {
                    return false;
                }
                if matches!(fill_rgb, Some((255, 255, 255))) && area_ratio > 0.5 {
                    return false;
                }
                true
            },
            _ => true,
        });

        // Suppress shapes (e.g. stroked horizontal staff lines, slurs
        // rendered as Bezier paths, beam rects) whose centre falls
        // inside a music region — they'd otherwise visually conflict
        // with the rasterised music bitmap.
        if !music_regions.is_empty() {
            shapes.retain(|s| {
                let (cx, cy) = match s {
                    SimpleShape::Line {
                        x1_pt,
                        y1_pt,
                        x2_pt,
                        y2_pt,
                        ..
                    } => ((x1_pt + x2_pt) * 0.5, (y1_pt + y2_pt) * 0.5),
                    SimpleShape::Rect { bbox, .. } => {
                        (bbox.x + bbox.width * 0.5, bbox.y + bbox.height * 0.5)
                    },
                };
                !music_regions
                    .iter()
                    .any(|r| crate::converters::music_region_finder::rect_contains_point(r, cx, cy))
            });
        }

        // Rasterise each music region as a 150 DPI PNG and emit it
        // as a positional image. This is what actually preserves
        // staves / noteheads / stems through the round-trip — the
        // span/shape suppression above just keeps garbage from
        // overlapping the bitmap.
        #[cfg(feature = "rendering")]
        if !music_regions.is_empty() {
            let regions =
                crate::converters::music_region_finder::rasterize_music_regions(doc, i, h_pt);
            for ((x_pdf, y_pdf, w, h), png) in regions {
                media_idx += 1;
                let target = format!("/word/media/image{}.png", media_idx);
                let part = PartName::new(&target)
                    .map_err(|e| Error::InvalidOperation(format!("part name {target}: {e}")))?;
                opc.add_part(&part, "image/png", &png)
                    .map_err(|e| Error::InvalidOperation(format!("opc add_part {target}: {e}")))?;
                let rid = opc.add_part_rel(
                    &doc_part,
                    rel_types::IMAGE,
                    &format!("media/image{}.png", media_idx),
                );
                page_images.push(PageImage {
                    bbox: crate::geometry::Rect::new(x_pdf, y_pdf, w, h),
                    rid,
                });
            }
        }

        pages.push(PageSpans {
            w_pt,
            h_pt,
            spans,
            font_lookup,
            images: page_images,
            shapes,
        });
    }

    let (page_w_pt, page_h_pt) = (pages[0].w_pt, pages[0].h_pt);

    let body = build_document_body(&pages, page_w_pt, page_h_pt);
    let document_xml = wrap_document(&body);

    opc.add_part(
        &doc_part,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        document_xml.as_bytes(),
    )
    .map_err(|e| Error::InvalidOperation(format!("opc add_part document.xml: {e}")))?;

    // styles.xml — minimal scaffold defining the heading styles the
    // body references via `<w:pStyle w:val="HeadingN"/>`. Without
    // this, Word silently swallows the pStyle reference and the
    // affected paragraphs render as plain text.
    let styles_part = PartName::new("/word/styles.xml")
        .map_err(|e| Error::InvalidOperation(format!("part name styles.xml: {e}")))?;
    opc.add_part_rel(&doc_part, rel_types::STYLES, "styles.xml");
    opc.add_part(
        &styles_part,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        layout_styles_xml().as_bytes(),
    )
    .map_err(|e| Error::InvalidOperation(format!("opc add_part styles.xml: {e}")))?;

    // Embed the source PDF's font programs (typeface preservation),
    // and wire them up via fontTable.xml + rels so Word actually
    // picks them up. Without the manifest plumbing the TTFs ship
    // but Word silently substitutes Calibri.
    let embedded_fonts = doc.extract_embedded_fonts().unwrap_or_default();
    if !embedded_fonts.is_empty() {
        let font_table_part = PartName::new("/word/fontTable.xml")
            .map_err(|e| Error::InvalidOperation(format!("part name fontTable.xml: {e}")))?;
        opc.add_part_rel(&doc_part, rel_types::FONT_TABLE, "fontTable.xml");

        let mut font_entries: Vec<(String, String)> = Vec::with_capacity(embedded_fonts.len());
        for (idx, (name, data)) in embedded_fonts.iter().enumerate() {
            let n = idx + 1;
            let safe_name = sanitize_font_filename(name);
            let target_rel = format!("fonts/font_{n}_{safe_name}.ttf");
            let target_abs = format!("/word/fonts/font_{n}_{safe_name}.ttf");
            let part = PartName::new(&target_abs)
                .map_err(|e| Error::InvalidOperation(format!("part name {target_abs}: {e}")))?;
            opc.add_part(&part, "application/x-font-ttf", data)
                .map_err(|e| Error::InvalidOperation(format!("opc add_part {target_abs}: {e}")))?;
            let rid = opc.add_part_rel(&font_table_part, rel_types::FONT, &target_rel);
            font_entries.push((name.clone(), rid));
        }

        opc.add_part(
            &font_table_part,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml",
            layout_font_table_xml(&font_entries).as_bytes(),
        )
        .map_err(|e| Error::InvalidOperation(format!("opc add_part fontTable.xml: {e}")))?;
    }

    let result = opc
        .finish()
        .map_err(|e| Error::InvalidOperation(format!("opc finish: {e}")))?;
    Ok(result.into_inner())
}

/// Minimal `word/styles.xml` shipped by the layout writer. Defines
/// `Normal` and `Heading1..6` so the body's `<w:pStyle>` references
/// resolve. Run-property defaults are intentionally bare — the
/// layout writer already inlines `<w:rPr>` per run with explicit
/// font/size/colour, so heading styling is driven by those, not by
/// the stylesheet.
fn layout_styles_xml() -> String {
    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    out.push_str(
        r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    out.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/></w:style>"#,
    );
    for level in 1..=6 {
        let outline = level - 1;
        out.push_str(&format!(
            concat!(
                r#"<w:style w:type="paragraph" w:styleId="Heading{lvl}">"#,
                r#"<w:name w:val="heading {lvl}"/>"#,
                r#"<w:basedOn w:val="Normal"/>"#,
                r#"<w:pPr><w:outlineLvl w:val="{ol}"/></w:pPr>"#,
                r#"</w:style>"#
            ),
            lvl = level,
            ol = outline,
        ));
    }
    out.push_str(r#"</w:styles>"#);
    out
}

/// Build `word/fontTable.xml` for the layout writer. Lists each
/// embedded font with `<w:embedRegular r:id="rIdN"/>` so Word
/// recognises the package's font programs.
fn layout_font_table_xml(entries: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    out.push_str(r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#);
    for (name, rid) in entries {
        out.push_str(&format!(
            r#"<w:font w:name="{name}"><w:embedRegular r:id="{rid}"/></w:font>"#,
            name = xml_escape(name),
            rid = rid,
        ));
    }
    out.push_str(r#"</w:fonts>"#);
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Build the inner contents of `<w:body>` — one positioned paragraph per
/// span, plus a final `<w:sectPr>` for page geometry.
fn build_document_body(pages: &[PageSpans], page_w_pt: f32, page_h_pt: f32) -> String {
    let mut out = String::with_capacity(64 * 1024);
    for (pi, page) in pages.iter().enumerate() {
        // Page break before every page except the first. Word treats a
        // hard break inside the body as the start of a new page.
        if pi > 0 {
            out.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
        }

        // Group spans into lines (by Y position). Each line becomes
        // ONE positioned `<w:framePr>` paragraph carrying all the
        // line's spans as separate runs. Without this grouping, Word
        // emits each span at its absolute frame and the slightly
        // wider fallback font (when the source TeXGyre/NewTXMI face
        // isn't installed) makes adjacent frames overflow into each
        // other — the visible "text on top of text" the user reported.
        let lines = crate::converters::layout_lines::group_spans_into_lines(page.spans.clone());
        for line in &lines {
            // PDF coords: origin bottom-left, y increases upward.
            // DOCX framePr coords: origin top-left, y increases downward.
            let x_pt = line.x_pt.max(0.0).min(page.w_pt);
            let y_pt = (page.h_pt - line.y_pt - line.height_pt)
                .max(0.0)
                .min(page.h_pt);

            let x_twip = (x_pt * TWIPS_PER_PT) as i32;
            let y_twip = (y_pt * TWIPS_PER_PT) as i32;
            // Pad line width 1.5× the source bbox so fallback fonts
            // (Helvetica is wider than TeXGyre) don't clip the
            // trailing run; clamp to remaining page width.
            let line_w_pt = line.width_pt;
            let max_pad_w = (page.w_pt - x_pt).max(line_w_pt + 4.0);
            let w_pt = (line_w_pt * 1.5).max(line_w_pt + 16.0).min(max_pad_w);
            let w_twip = ((w_pt * TWIPS_PER_PT) as i32).max(40);
            let max_font = line
                .spans
                .iter()
                .map(|s| s.font_size)
                .fold(0.0_f32, f32::max)
                .max(line.height_pt);
            let line_h_pt = max_font * 1.4;
            let h_twip = (line_h_pt * TWIPS_PER_PT) as i32;

            // First span's heading_level marks the whole line as a
            // heading (rare for a heading to span multiple distinct
            // <w:p>s on one line; usually the entire line is the
            // heading). Falls back to plain body when None.
            let pstyle_xml = match line.spans.first().and_then(|s| s.heading_level) {
                Some(n) if (1..=6).contains(&n) => {
                    format!("<w:pStyle w:val=\"Heading{}\"/>", n)
                },
                _ => String::new(),
            };

            let mut runs_xml = String::with_capacity(256);
            let mut prev_right_pt: Option<f32> = None;
            for span in &line.spans {
                let text = span.text.trim_matches('\u{0000}');
                if text.is_empty() {
                    continue;
                }

                // Insert a synthetic space when there's a visible
                // gap and neither side already carries whitespace.
                if let Some(prev_right) = prev_right_pt {
                    let gap = span.bbox.x - prev_right;
                    let needs_space = gap > span.font_size * 0.25
                        && !runs_xml.ends_with(' ')
                        && !runs_xml.ends_with("&#160;")
                        && !text.starts_with(|c: char| c.is_whitespace());
                    if needs_space {
                        runs_xml.push_str(r#"<w:r><w:t xml:space="preserve"> </w:t></w:r>"#);
                    }
                }

                let sz_half_pt = (span.font_size * 2.0).round() as i32;
                let color_hex =
                    if crate::color::is_document_black(span.color.r, span.color.g, span.color.b) {
                        String::new()
                    } else {
                        format!(
                            "<w:color w:val=\"{:02X}{:02X}{:02X}\"/>",
                            (span.color.r * 255.0).round().clamp(0.0, 255.0) as u8,
                            (span.color.g * 255.0).round().clamp(0.0, 255.0) as u8,
                            (span.color.b * 255.0).round().clamp(0.0, 255.0) as u8,
                        )
                    };
                let bold = if span.font_weight.is_bold() {
                    "<w:b/><w:bCs/>"
                } else {
                    ""
                };
                let italic = if span.is_italic { "<w:i/><w:iCs/>" } else { "" };

                let raw_font = &span.font_name;
                let resolved: String = if let Some(real) = page.font_lookup.get(raw_font) {
                    real.clone()
                } else {
                    let stripped = raw_font
                        .split_once('+')
                        .map(|(_, rest)| rest)
                        .unwrap_or(raw_font);
                    let looks_synthetic = stripped.len() < 12
                        && (stripped.bytes().next() == Some(b'F'))
                        && stripped.bytes().skip(1).all(|b| b.is_ascii_digit());
                    if looks_synthetic {
                        if span.is_monospace {
                            "Courier New"
                        } else {
                            "Times New Roman"
                        }
                        .to_string()
                    } else {
                        stripped.to_string()
                    }
                };
                let font = escape_xml(&resolved);

                runs_xml.push_str(&format!(
                    r#"<w:r><w:rPr><w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>{bold}{italic}<w:sz w:val="{sz}"/><w:szCs w:val="{sz}"/>{color}</w:rPr><w:t xml:space="preserve">{text}</w:t></w:r>"#,
                    font = font,
                    bold = bold, italic = italic,
                    sz = sz_half_pt, color = color_hex,
                    text = escape_xml(text),
                ));
                prev_right_pt = Some(span.bbox.x + span.bbox.width);
            }

            if runs_xml.is_empty() {
                continue;
            }

            out.push_str(&format!(
                r#"<w:p><w:pPr>{pstyle}<w:framePr w:w="{w}" w:h="{h}" w:hRule="exact" w:hSpace="0" w:vSpace="0" w:wrap="none" w:vAnchor="page" w:hAnchor="page" w:x="{x}" w:y="{y}"/><w:spacing w:before="0" w:after="0" w:line="240" w:lineRule="auto"/></w:pPr>{runs}</w:p>"#,
                pstyle = pstyle_xml,
                w = w_twip, h = h_twip, x = x_twip, y = y_twip,
                runs = runs_xml,
            ));
        }

        // Emit each vector shape (table border, divider, frame) as a
        // DrawingML wsp shape anchored to the page. Lines use prstGeom
        // line; rects use prstGeom rect with optional fill/stroke.
        for (sh_idx, shape) in page.shapes.iter().enumerate() {
            let id = pi * 10_000 + 5_000 + sh_idx + 1;
            match shape {
                SimpleShape::Line {
                    x1_pt,
                    y1_pt,
                    x2_pt,
                    y2_pt,
                    stroke_rgb,
                    stroke_w_pt,
                } => {
                    let xa = (*x1_pt as f64).min(*x2_pt as f64);
                    let xb = (*x1_pt as f64).max(*x2_pt as f64);
                    let ya = (*y1_pt as f64).min(*y2_pt as f64);
                    let yb = (*y1_pt as f64).max(*y2_pt as f64);
                    // Y flip
                    let top_y = page.h_pt as f64 - yb;
                    let off_x = (xa * EMU_PER_PT) as i64;
                    let off_y = (top_y * EMU_PER_PT) as i64;
                    let cx = (((xb - xa).max(1.0)) * EMU_PER_PT) as i64;
                    let cy = (((yb - ya).max(1.0)) * EMU_PER_PT) as i64;
                    let stroke_w_emu = ((*stroke_w_pt as f64).max(0.25) * EMU_PER_PT) as i64;
                    let (r, g, b) = stroke_rgb;
                    out.push_str(&format!(
                        r#"<w:p><w:r><w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" behindDoc="0" distT="0" distB="0" distL="0" distR="0" simplePos="0" locked="0" layoutInCell="1" allowOverlap="1" relativeHeight="0"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page"><wp:posOffset>{ox}</wp:posOffset></wp:positionH><wp:positionV relativeFrom="page"><wp:posOffset>{oy}</wp:posOffset></wp:positionV><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="{id}" name="Line{id}"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:cNvSpPr/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="line"><a:avLst/></a:prstGeom><a:noFill/><a:ln w="{sw}"><a:solidFill><a:srgbClr val="{r:02X}{g:02X}{b:02X}"/></a:solidFill></a:ln></wps:spPr><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>"#,
                        ox = off_x, oy = off_y, cx = cx, cy = cy,
                        id = id, sw = stroke_w_emu, r = r, g = g, b = b,
                    ));
                },
                SimpleShape::Rect {
                    bbox,
                    stroke_rgb,
                    fill_rgb,
                    stroke_w_pt,
                } => {
                    let x_pt = (bbox.x as f64).max(0.0);
                    let y_pt = (page.h_pt as f64 - bbox.y as f64 - bbox.height as f64).max(0.0);
                    let off_x = (x_pt * EMU_PER_PT) as i64;
                    let off_y = (y_pt * EMU_PER_PT) as i64;
                    let cx = ((bbox.width as f64).max(1.0) * EMU_PER_PT) as i64;
                    let cy = ((bbox.height as f64).max(1.0) * EMU_PER_PT) as i64;
                    let fill_xml = match fill_rgb {
                        Some((r, g, b)) => format!(
                            "<a:solidFill><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:solidFill>",
                            r, g, b
                        ),
                        None => "<a:noFill/>".to_string(),
                    };
                    let stroke_xml = match stroke_rgb {
                        Some((r, g, b)) => {
                            let stroke_w_emu =
                                ((*stroke_w_pt as f64).max(0.25) * EMU_PER_PT) as i64;
                            format!(
                                "<a:ln w=\"{}\"><a:solidFill><a:srgbClr val=\"{:02X}{:02X}{:02X}\"/></a:solidFill></a:ln>",
                                stroke_w_emu, r, g, b
                            )
                        },
                        None => "<a:ln><a:noFill/></a:ln>".to_string(),
                    };
                    out.push_str(&format!(
                        r#"<w:p><w:r><w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" behindDoc="1" distT="0" distB="0" distL="0" distR="0" simplePos="0" locked="0" layoutInCell="1" allowOverlap="1" relativeHeight="0"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page"><wp:posOffset>{ox}</wp:posOffset></wp:positionH><wp:positionV relativeFrom="page"><wp:posOffset>{oy}</wp:posOffset></wp:positionV><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="{id}" name="Rect{id}"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:cNvSpPr/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom>{fill}{stroke}</wps:spPr><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>"#,
                        ox = off_x, oy = off_y, cx = cx, cy = cy,
                        id = id, fill = fill_xml, stroke = stroke_xml,
                    ));
                },
            }
        }

        // Emit each PDF image as a DrawingML floating picture anchored to the
        // page at the source coordinates. Each picture lives inside its own
        // host paragraph (Word requires drawings to sit inside a `<w:p>`).
        for (img_idx, img) in page.images.iter().enumerate() {
            let x_pt = img.bbox.x.max(0.0).min(page.w_pt) as f64;
            let y_pt = (page.h_pt - img.bbox.y - img.bbox.height).max(0.0) as f64;
            let y_pt = y_pt.min(page.h_pt as f64);
            let cx = (img.bbox.width.max(1.0) as f64 * EMU_PER_PT) as i64;
            let cy = (img.bbox.height.max(1.0) as f64 * EMU_PER_PT) as i64;
            let off_x = (x_pt * EMU_PER_PT) as i64;
            let off_y = (y_pt * EMU_PER_PT) as i64;
            let id = pi * 10_000 + img_idx + 1;
            out.push_str(&format!(
                r#"<w:p><w:r><w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" behindDoc="1" distT="0" distB="0" distL="0" distR="0" simplePos="0" locked="0" layoutInCell="1" allowOverlap="1" relativeHeight="0"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page"><wp:posOffset>{ox}</wp:posOffset></wp:positionH><wp:positionV relativeFrom="page"><wp:posOffset>{oy}</wp:posOffset></wp:positionV><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="{id}" name="Image{id}"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="{id}" name="Image{id}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>"#,
                ox = off_x, oy = off_y, cx = cx, cy = cy, id = id, rid = img.rid,
            ));
        }
    }

    // Final section: page size + zero margins (frames are already absolutely
    // positioned). DOCX requires a sectPr at the end of the body to set page
    // geometry for the whole document.
    let pw_twip = (page_w_pt * TWIPS_PER_PT) as i32;
    let ph_twip = (page_h_pt * TWIPS_PER_PT) as i32;
    out.push_str(&format!(
        r#"<w:sectPr><w:pgSz w:w="{pw}" w:h="{ph}"/><w:pgMar w:top="0" w:right="0" w:bottom="0" w:left="0" w:header="0" w:footer="0" w:gutter="0"/></w:sectPr>"#,
        pw = pw_twip, ph = ph_twip,
    ));

    out
}

struct PageSpans {
    w_pt: f32,
    h_pt: f32,
    spans: Vec<TextSpan>,
    /// PDF font-resource-name → real face name (subset prefix stripped).
    /// e.g. `"F75"` → `"TeXGyreTermesX-Regular"`.
    font_lookup: HashMap<String, String>,
    images: Vec<PageImage>,
    shapes: Vec<SimpleShape>,
}

struct PageImage {
    bbox: crate::geometry::Rect,
    rid: String,
}

/// Simplified vector primitive — just lines and rectangles. Anything more
/// complex (Bezier curves, multi-segment paths) is approximated by its
/// bounding box rectangle. Sufficient for table borders, underlines,
/// frames, and other rule-style decorations.
enum SimpleShape {
    Line {
        x1_pt: f32,
        y1_pt: f32,
        x2_pt: f32,
        y2_pt: f32,
        stroke_rgb: (u8, u8, u8),
        stroke_w_pt: f32,
    },
    Rect {
        bbox: crate::geometry::Rect,
        stroke_rgb: Option<(u8, u8, u8)>,
        fill_rgb: Option<(u8, u8, u8)>,
        stroke_w_pt: f32,
    },
}

fn wrap_document(body_xml: &str) -> String {
    // Note: w:document MUST include the namespace declarations Word checks
    // for. Missing one → "Word found unreadable content" dialog.
    // Image emission also requires `wp` (DrawingML wordprocessing),
    // `a` (DrawingML core), and `pic` (DrawingML picture).
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document
    xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
    xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
    xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
    xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
    xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
<w:body>{}</w:body>
</w:document>"#,
        body_xml
    )
}

fn sanitize_font_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect()
}

/// Stamp `heading_level` on spans that look like a heading by font-size
/// ratio. Matches the thresholds the existing markdown / pdf_to_ir
/// converters use (1.75 / 1.35 / 1.15 × modal body size). Only spans
/// passing simple guards (≥2 alphabetic chars, ≤120 chars, no trailing
/// period for body text) are accepted.
fn annotate_heading_levels(spans: &mut [crate::layout::text_block::TextSpan]) {
    if spans.is_empty() {
        return;
    }
    // Modal body size: most-common font_size, rounded to 0.1 pt buckets.
    let mut buckets: std::collections::HashMap<i32, usize> = Default::default();
    for s in spans.iter() {
        let key = (s.font_size * 10.0).round() as i32;
        *buckets.entry(key).or_default() += 1;
    }
    // Explicit tie-break, not `max_by_key`: it returns the LAST maximal
    // element, and over a `HashMap` that is the per-process-randomized
    // iteration order, so a page whose two font sizes carry equal span counts
    // picked a different modal body size per process — which flips the
    // heading/body ratio test below. Ties resolve to the smaller size.
    let body_size_tenth = buckets
        .iter()
        .max_by(|(a_key, a_c), (b_key, b_c)| a_c.cmp(b_c).then_with(|| b_key.cmp(a_key)))
        .map(|(k, _)| *k)
        .unwrap_or(120);
    let body = (body_size_tenth as f32) / 10.0;
    for s in spans.iter_mut() {
        if !looks_like_heading_text(&s.text) {
            continue;
        }
        let ratio = s.font_size / body.max(1.0);
        let level = if ratio >= 1.75 {
            Some(1)
        } else if ratio >= 1.35 {
            Some(2)
        } else if ratio >= 1.15 {
            Some(3)
        } else {
            None
        };
        if level.is_some() {
            s.heading_level = level;
        }
    }
}

/// Reject candidates that obviously aren't headings: too long, no
/// letters, ends in body-paragraph period.
fn looks_like_heading_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < 2 || trimmed.len() > 120 {
        return false;
    }
    if !trimmed.chars().any(|c| c.is_alphabetic()) {
        return false;
    }
    // A long sentence ending in `.` (and not a numbered heading like
    // "1.2.") is body text, not a heading.
    if trimmed.len() > 60 && trimmed.ends_with('.') {
        return false;
    }
    true
}

/// Merge end-of-line hyphenated spans. PDF text wraps long words across
/// lines as `"agree-"` followed by `"laws"` on the next baseline. The
/// layout writer would otherwise emit them as two adjacent floating
/// frames whose extracted text concatenates to `"agree-laws"` or
/// `"agreelaws"` — a word that doesn't exist. We detect:
///   - prev span text ends in `-`
///   - next span starts with a lowercase letter
///   - next span sits on a different baseline (lower y in PDF coords)
///   - next span begins horizontally near where prev would have
///
/// and merge: drop the trailing `-`, splice the next span's text onto
/// the prev span's text, drop the next span. Keeps the original bbox /
/// font of the prev span so downstream positioning is unchanged.
fn merge_hyphenated_spans(spans: &mut Vec<crate::layout::text_block::TextSpan>) {
    if spans.len() < 2 {
        return;
    }
    // Single forward pass with a running accumulator. The previous version did
    // `spans.remove(i+1)` (O(n) shift) and did not advance `i` after a merge, so
    // a long hyphenation chain was O(n^2). Here `cur` accumulates and may chain
    // into the following span exactly as before, but each span is visited once.
    let mut out: Vec<crate::layout::text_block::TextSpan> = Vec::with_capacity(spans.len());
    let mut iter = std::mem::take(spans).into_iter();
    let mut cur = iter.next().expect("len >= 2 checked above");
    for next in iter {
        let merge = {
            let cur_text = cur.text.trim_end();
            // Hyphen at end of current span, lowercase letter starting the next.
            let ends_hyphen = cur_text.ends_with('-')
                && cur_text.len() >= 2
                && cur_text
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
            // Different baseline (a hyphenation continuation lives on a lower line).
            let cur_cy = cur.bbox.y + cur.bbox.height * 0.5;
            let next_cy = next.bbox.y + next.bbox.height * 0.5;
            let line_h = cur.font_size.max(next.font_size).max(1.0);
            let new_line = (cur_cy - next_cy).abs() > line_h * 0.5;
            ends_hyphen && starts_lower && new_line
        };
        if merge {
            let next_text = next.text.trim_start().to_string();
            let mut cur_text = cur.text.trim_end().to_string();
            cur_text.pop(); // remove '-'
            cur_text.push_str(&next_text);
            cur.text = cur_text;
            let cur_bbox = cur.bbox;
            let next_bbox = next.bbox;
            let min_x = cur_bbox.x.min(next_bbox.x);
            let min_y = cur_bbox.y.min(next_bbox.y);
            let max_x = (cur_bbox.x + cur_bbox.width).max(next_bbox.x + next_bbox.width);
            let max_y = (cur_bbox.y + cur_bbox.height).max(next_bbox.y + next_bbox.height);
            cur.bbox = crate::geometry::Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
            // `cur` stays the accumulator — may chain into the next span.
        } else {
            out.push(cur);
            cur = next;
        }
    }
    out.push(cur);
    *spans = out;
}

/// Reduce a `PathContent` to one or more `SimpleShape`s.
///
/// Recognises:
/// - A single MoveTo + LineTo pair → a single line segment.
/// - A single Rectangle operation → a rect.
/// - Anything else → bounding-box rectangle (a coarse approximation,
///   but better than dropping the path entirely).
fn simplify_path(path: &crate::elements::PathContent) -> Vec<SimpleShape> {
    use crate::elements::PathOperation::*;
    let stroke_rgb = path.stroke_color.as_ref().map(|c| {
        (
            (c.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (c.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (c.b * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    });
    let fill_rgb = path.fill_color.as_ref().map(|c| {
        (
            (c.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (c.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (c.b * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    });

    // Most lines come as `MoveTo(x1,y1), LineTo(x2,y2)`.
    if let [MoveTo(x1, y1), LineTo(x2, y2)] = path.operations.as_slice() {
        if let Some(stroke) = stroke_rgb {
            return vec![SimpleShape::Line {
                x1_pt: *x1,
                y1_pt: *y1,
                x2_pt: *x2,
                y2_pt: *y2,
                stroke_rgb: stroke,
                stroke_w_pt: path.stroke_width.max(0.25),
            }];
        }
    }
    // Rectangle primitive.
    if let [Rectangle(x, y, w, h), ..] = path.operations.as_slice() {
        return vec![SimpleShape::Rect {
            bbox: crate::geometry::Rect::new(*x, *y, *w, *h),
            stroke_rgb,
            fill_rgb,
            stroke_w_pt: path.stroke_width.max(0.25),
        }];
    }
    // Fallback: render the path's bounding box. Loses curvature but
    // preserves position and footprint for things like rounded boxes.
    if path.bbox.width > 0.5 && path.bbox.height > 0.5 {
        vec![SimpleShape::Rect {
            bbox: path.bbox,
            stroke_rgb,
            fill_rgb,
            stroke_w_pt: path.stroke_width.max(0.25),
        }]
    } else {
        vec![]
    }
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        // Collapse Mathematical Alphanumeric Symbols (U+1D400-1D7FF) to
        // their plain Latin/Greek base — `𝑥` → `x`, `𝛽` → `β`. Word can't
        // typeset the math-italic codepoints from the standard fonts, and
        // downstream extraction normalises them anyway. Doing it here keeps
        // the layout DOCX clean.
        let c = crate::fonts::encoding::math_alphanumeric_base(c as u32)
            .and_then(char::from_u32)
            .unwrap_or(c);
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // Strip control chars (XML 1.0 forbids most of them).
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r' => {},
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod heading_level_tests {
    use super::*;

    fn span(text: &str, size: f32) -> TextSpan {
        TextSpan {
            text: text.to_string(),
            font_size: size,
            ..Default::default()
        }
    }

    /// The modal-body-size histogram inside `annotate_heading_levels` ties
    /// 3-to-3 between 10 pt and 20 pt. `max_by_key` over the backing
    /// `HashMap` returned whichever bucket the per-process-randomized
    /// iteration order visited last, so the body size — and therefore every
    /// heading/body ratio — flapped between runs. The tie must resolve to the
    /// SMALLER size (10 pt), which makes the 20 pt spans ratio-2.0 H1s. Had
    /// it resolved to 20 pt, those same spans would be ratio-1.0 body text
    /// and get no heading level at all.
    #[test]
    fn modal_body_size_tie_resolves_to_smaller_size() {
        let mut spans = vec![
            span("Chapter One", 20.0),
            span("Chapter Two", 20.0),
            span("Chapter Three", 20.0),
            span("Some body text", 10.0),
            span("More body text", 10.0),
            span("Further body text", 10.0),
        ];
        annotate_heading_levels(&mut spans);
        for s in spans.iter().take(3) {
            assert_eq!(
                s.heading_level,
                Some(1),
                "20 pt span against a 10 pt body must be an H1: {:?}",
                s.text
            );
        }
        for s in spans.iter().skip(3) {
            assert_eq!(s.heading_level, None, "body-size span must not be a heading: {:?}", s.text);
        }
    }

    /// The tie-break must not disturb the ordinary case: a clear modal size
    /// stays the body size regardless of the tie direction.
    #[test]
    fn unique_modal_body_size_is_unaffected_by_tie_break() {
        let mut spans = vec![
            span("Chapter One", 20.0),
            span("Some body text", 10.0),
            span("More body text", 10.0),
            span("Further body text", 10.0),
        ];
        annotate_heading_levels(&mut spans);
        assert_eq!(spans[0].heading_level, Some(1));
    }
}
