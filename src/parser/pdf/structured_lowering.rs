//! Materialize geometry-confirmed grids as editable rhwp tables. Uncertain grids stay positional.
mod body;
mod composition;
mod native_borders;
use super::{
    lowering::hu,
    recognition::{ParagraphProposal, ParagraphState},
    text_analysis::{reconstruct_line, LineJoin, ParagraphLayout, TextAlignment, TextLine},
    ExtractedPage,
};
use crate::model::{
    control::Control,
    document::Document,
    paragraph::{CharShapeRef, LineSeg, Paragraph},
    shape::{CommonObjAttr, HorzRelTo, ShapeObject, TextWrap, VertRelTo},
    style::{
        border_width_index, Alignment, BorderLineType, LineSpacingType, TabDef, TabItem,
        PARA_ATTR2_LINE_WRAP_MASK,
    },
    table::{Cell, Table, TablePageBreak, VerticalAlign},
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn source_baseline_position(
    page: &ExtractedPage,
    doc: &Document,
    line: &TextLine,
    span: usize,
    base_id: u32,
) -> Result<(i8, f64), String> {
    let source_delta = page.media_box.3 - page.spans[span].bbox.y - line.baseline;
    let size = doc.doc_info.char_shapes[base_id as usize].base_size as f64 / 100.;
    let position = (f64::from(source_delta) / size * 100.).round();
    if !position.is_finite() || !(-100. ..=100.).contains(&position) {
        return Err("source baseline displacement exceeds HWP character position range".into());
    }
    Ok((
        position as i8,
        (size * position / 100. - f64::from(source_delta)).abs(),
    ))
}

/// Source baseline is independent of whether horizontal reconstruction succeeds.
/// Both the legacy tab frame and ordinary paragraphs must retain this property.
fn positioned_span_style(
    page: &ExtractedPage,
    doc: &mut Document,
    line: &TextLine,
    span: usize,
    base_id: u32,
    cache: &mut BTreeMap<(u32, u8, i8, i8), u32>,
) -> Result<u32, String> {
    let (position, _) = source_baseline_position(page, doc, line, span, base_id)?;
    if position == 0 {
        return Ok(base_id);
    }
    let base = &doc.doc_info.char_shapes[base_id as usize];
    let key = (base_id, base.ratios[0], base.spacings[0], position);
    if let Some(id) = cache.get(&key) {
        return Ok(*id);
    }
    if doc.doc_info.char_shapes.len() >= u16::MAX as usize {
        return Err("PDF character style budget exceeded".into());
    }
    let mut style = base.clone();
    style.raw_data = None;
    style.char_offsets = [position; 7];
    let id = doc.doc_info.char_shapes.len() as u32;
    doc.doc_info.char_shapes.push(style);
    cache.insert(key, id);
    Ok(id)
}

/// Translate observed glyph metrics into the ordinary HWP character contract.
/// Source-font loss is explicit: width-compatible fallback does not recover its outline.
fn observed_line_text(
    page: &ExtractedPage,
    doc: &mut Document,
    line: &TextLine,
    shape_ids: &[Option<u32>],
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    cache: &mut BTreeMap<(u32, u8, i8, i8), u32>,
) -> Result<(String, Vec<(u32, u32)>, Value), String> {
    use crate::renderer::{
        layout::{estimate_text_width_exact, resolved_to_text_style},
        style_resolver::detect_lang_category,
    };
    let content = reconstruct_line(&page.spans, &line.indices)?;
    let mut runs = Vec::new();
    let mut offset = 0;
    let mut maximum_error: f64 = 0.;
    let mut maximum_baseline_error: f64 = 0.;
    for glyph in &content.characters {
        let base_id = shape_ids[glyph.span].ok_or("missing source style")?;
        let source = &page.spans[glyph.span];
        // PDF font size already includes script scaling. Preserve it and encode
        // only the observed displacement in ordinary HWP CHAROFFSET; setting
        // the superscript flag would shrink a small PDF glyph a second time.
        let (position, baseline_error) =
            source_baseline_position(page, doc, line, glyph.span, base_id)?;
        maximum_baseline_error = maximum_baseline_error.max(baseline_error);
        let mut style =
            resolved_to_text_style(styles, base_id, detect_lang_category(glyph.character));
        let text = glyph.character.to_string();
        // Glyph reconstruction and actual run placement must use the same
        // fractional width. Rounding each scalar can invent an unsupported
        // ratio (e.g. a 2.5pt TJ separator measured as 7 instead of 6.667px).
        let base_width = estimate_text_width_exact(&text, &style);
        if !base_width.is_finite() || base_width <= 0. {
            return Err("glyph advance needs separate shaping".into());
        }
        let fallback = source.font_name.is_empty() || source.font_name == "Unknown";
        // A missing source font needs metric-compatible fallback; declared fonts
        // retain their Tz scaling. Spaces have no outline and own their measured extent.
        let ratio = if fallback || glyph.character.is_whitespace() {
            (f64::from(glyph.nominal_advance) * 96. / 72. / base_width * 100.).round()
        } else {
            f64::from(source.horizontal_scaling).round()
        };
        if !(50. ..=200.).contains(&ratio) {
            return Err("source glyph width exceeds HWP ratio range".into());
        }
        let ratio = ratio as u8;
        style.ratio = f64::from(ratio) / 100.;
        if let Some(latin) = &mut style.ascii_punct_latin {
            latin.ratio = style.ratio;
        }
        let nominal = estimate_text_width_exact(&text, &style);
        let desired = f64::from(glyph.advance) * 96. / 72.;
        let guess = (desired / nominal - 1.) * 100.;
        if !guess.is_finite() || !(-52. ..=52.).contains(&guess) {
            return Err("source displacement exceeds HWP spacing range".into());
        }
        let mut best: Option<(i8, f64)> = None;
        for spacing in (guess.floor() as i32 - 2)..=(guess.ceil() as i32 + 2) {
            if !(-50..=50).contains(&spacing) {
                continue;
            }
            style.letter_spacing = style.font_size * f64::from(spacing) / 100.;
            if let Some(latin) = &mut style.ascii_punct_latin {
                latin.letter_spacing = style.letter_spacing;
            }
            let error = (estimate_text_width_exact(&text, &style) - desired).abs();
            if best.is_none_or(|(_, previous)| error < previous) {
                best = Some((spacing as i8, error));
            }
        }
        let (spacing, error) = best.ok_or("source displacement exceeds HWP spacing range")?;
        maximum_error = maximum_error.max(error * 72. / 96.);
        let key = (base_id, ratio, spacing, position);
        let id = if ratio == 100 && spacing == 0 && position == 0 {
            base_id
        } else if let Some(id) = cache.get(&key) {
            *id
        } else {
            if doc.doc_info.char_shapes.len() >= u16::MAX as usize {
                return Err("PDF character style budget exceeded".into());
            }
            let mut character_style = doc.doc_info.char_shapes[base_id as usize].clone();
            character_style.raw_data = None;
            character_style.ratios = [ratio; 7];
            character_style.spacings = [spacing; 7];
            character_style.char_offsets = [position; 7];
            let id = doc.doc_info.char_shapes.len() as u32;
            doc.doc_info.char_shapes.push(character_style);
            cache.insert(key, id);
            id
        };
        if runs.last().is_none_or(|&(_, previous)| previous != id) {
            runs.push((offset, id));
        }
        offset += glyph.character.len_utf16() as u32;
    }
    Ok((
        content.text,
        runs,
        json!({
            "characters":content.characters,
            "maximumAdvanceQuantizationErrorPt":maximum_error,
            "maximumBaselineQuantizationErrorPt":maximum_baseline_error,
            "lineBaselinePt":line.baseline,
            "status":"observed_origins_with_nominal_widths",
            "hwpQuantization":"integer-percent ratio, spacing and character position; no source-font outline recovery"
        }),
    ))
}

fn array(v: &Value) -> Result<&Vec<Value>, String> {
    v.as_array()
        .ok_or_else(|| "invalid recognition array".into())
}
fn number(v: &Value) -> Result<f32, String> {
    v.as_f64()
        .filter(|n| n.is_finite())
        .map(|n| n as f32)
        .ok_or_else(|| "invalid recognition coordinate".into())
}
fn index(v: &Value) -> Result<usize, String> {
    v.as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| "invalid recognition index".into())
}
fn refs(v: &Value) -> Result<Vec<usize>, String> {
    array(v)?.iter().map(index).collect()
}

fn physical_margin(points: f32) -> Result<i32, String> {
    // Odd IR margin values encode CHAR units, not sub-HWPUNIT PDF precision.
    i32::try_from(hu(points)?)
        .ok()
        .and_then(|value| value.checked_mul(2))
        .ok_or_else(|| "physical paragraph measure exceeds format range".into())
}

// PDF TJ displacements can express word spaces without a U+0020 glyph.
// Positional boxes and reconstructed table paragraphs must recover the same
// text from the same glyph origins, before either path measures the line.
pub(super) fn span_text_with_spaces(span: &pdf_oxide::layout::TextSpan) -> String {
    if let Ok(content) = reconstruct_line(std::slice::from_ref(span), &[0]) {
        return content.text;
    }
    // Preserve the pre-existing positional path when complete glyph origins
    // are unavailable. This fallback does not assert recovered word boundaries.
    let mut text = String::new();
    for (j, ch) in span.text.chars().enumerate() {
        if j > 0 {
            if let (Some(x), Some(prev), Some(width)) = (
                span.char_x_offsets.get(j),
                span.char_x_offsets.get(j - 1),
                span.char_widths.get(j - 1),
            ) {
                if x - prev - width > span.font_size * 0.2
                    && !text.ends_with(char::is_whitespace)
                    && !ch.is_whitespace()
                {
                    text.push(' ');
                }
            }
        }
        text.push(ch);
    }
    text
}

fn text_with_spaces(page: &ExtractedPage, indices: &[usize]) -> (String, Vec<(u32, usize)>) {
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut end = None;
    for &i in indices {
        let span = &page.spans[i];
        if end.is_some_and(|x| span.bbox.x - x > span.font_size * 0.2)
            && !text.ends_with(char::is_whitespace)
            && !span.text.starts_with(char::is_whitespace)
        {
            text.push(' ');
        }
        runs.push((text.encode_utf16().count() as u32, i));
        text.push_str(&span_text_with_spaces(span));
        end = Some(span.bbox.x + span.bbox.width);
    }
    (text, runs)
}

type LineText = (String, Vec<(u32, usize)>, Vec<TabItem>, Vec<[u16; 7]>);

fn text_with_tabs(
    page: &ExtractedPage,
    indices: &[usize],
    cell_x: f32,
) -> Result<LineText, String> {
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut tabs = Vec::new();
    let mut extensions = Vec::new();
    let mut end = None;
    for &i in indices {
        let span = &page.spans[i];
        if let Some(previous_end) = end.filter(|x| span.bbox.x - x > span.font_size * 0.2) {
            text.push('\t');
            tabs.push(TabItem {
                position: u32::try_from(physical_margin(span.bbox.x - cell_x)?)
                    .map_err(|_| "invalid PDF tab position")?,
                ..Default::default()
            });
            let width = u16::try_from(hu(span.bbox.x - previous_end)?)
                .map_err(|_| "PDF span gap exceeds inline tab width")?;
            extensions.push([width, 0, 0, 0, 0, 0, 9]);
        }
        runs.push((text.encode_utf16().count() as u32, i));
        text.push_str(&text_with_spaces(page, &[i]).0);
        end = Some(span.bbox.x + span.bbox.width);
    }
    Ok((text, runs, tabs, extensions))
}

/// Recognition uses text-content UTF-16 positions. Publish every IR position
/// on the same HWP paragraph stream axis, where an extended tab owns 8 units.
/// Otherwise save/reopen expands tabs but leaves later styles and line starts
/// pointing into the preceding line. Floating paint slots are attached later.
fn finalize_text_axis(para: &mut Paragraph) -> Result<(), String> {
    let mut content_to_stream = BTreeMap::new();
    let mut offsets = Vec::with_capacity(para.text.chars().count());
    let (mut content, mut stream) = (0, 0);
    for ch in para.text.chars() {
        content_to_stream.insert(content, stream);
        offsets.push(stream);
        content += ch.len_utf16() as u32;
        stream += Paragraph::char_stream_len(ch);
    }
    content_to_stream.insert(content, stream);
    let convert = |pos| {
        content_to_stream
            .get(&pos)
            .copied()
            .ok_or_else(|| "PDF text metadata is not on a scalar boundary".to_string())
    };
    for shape in &mut para.char_shapes {
        shape.start_pos = convert(shape.start_pos)?;
    }
    for line in &mut para.line_segs {
        line.text_start = convert(line.text_start)?;
    }
    para.char_offsets = offsets;
    para.char_count = stream + 1;
    Ok(())
}

struct FlowItem {
    top: f32,
    bottom: f32,
    paragraph: Paragraph,
}

fn paragraph(
    page: &ExtractedPage,
    doc: &mut Document,
    proposal: &ParagraphProposal,
    cell_box: &Value,
    shape_ids: &[Option<u32>],
    fixed_line_height: bool,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    typography: &std::cell::RefCell<Vec<Value>>,
) -> Result<FlowItem, String> {
    let lines = &proposal.lines;
    let first = lines.first().ok_or("paragraph has no lines")?;
    let cell_x = number(&cell_box["x"])?;
    let cell_width = number(&cell_box["width"])?;
    let top = first.baseline - first.font * 0.95;
    let layout = proposal
        .layout
        .as_ref()
        .ok_or("paragraph layout deferred")?;
    let centered = layout.alignment == TextAlignment::Center
        || (!fixed_line_height
            && lines.iter().all(|l| {
                ((l.left + l.right) / 2. - cell_x - cell_width / 2.).abs() <= l.font * 0.75
            }));
    let mut para = Paragraph::new_empty();
    para.char_count_msb = true;
    para.char_shapes.clear();
    para.line_segs.clear();
    let mut last_end = top;
    let mut utf16 = 0;
    let mut tabs = Vec::new();
    let mut glyph_styles = BTreeMap::new();
    for (line_no, line) in lines.iter().enumerate() {
        let mut materialization = None;
        let indices = &line.indices;
        if indices.is_empty() {
            return Err("empty recognized line".into());
        }
        let baseline = line.baseline;
        let size = line.font;
        let line_top = baseline - size * 0.95;
        let next_top = if let Some(next) = lines.get(line_no + 1) {
            next.baseline - next.font * 0.95
        } else {
            line_top + size
        };
        let (text, runs): (String, Vec<(u32, u32)>) = if fixed_line_height {
            let before_styles = doc.doc_info.char_shapes.len();
            match observed_line_text(page, doc, line, shape_ids, styles, &mut glyph_styles) {
                Ok((text, runs, evidence)) => {
                    materialization = Some(evidence);
                    (text, runs)
                }
                Err(reason) => {
                    doc.doc_info.char_shapes.truncate(before_styles);
                    glyph_styles.retain(|_, id| (*id as usize) < before_styles);
                    materialization = Some(
                        json!({"status":"unverified_legacy_line_frame","reason":reason,
                            "baselinePolicy":"observed_source_position_preserved_independently_of_horizontal_metrics"}),
                    );
                    let (text, runs, line_tabs, extensions) =
                        text_with_tabs(page, indices, cell_x)?;
                    tabs.extend(line_tabs);
                    para.tab_extended.extend(extensions);
                    (
                        text,
                        runs.into_iter()
                            .map(|(offset, i)| {
                                let base = shape_ids[i].ok_or("missing source style")?;
                                positioned_span_style(page, doc, line, i, base, &mut glyph_styles)
                                    .map(|id| (offset, id))
                            })
                            .collect::<Result<_, _>>()?,
                    )
                }
            }
        } else {
            let (text, runs) = text_with_spaces(page, indices);
            (
                text,
                runs.into_iter()
                    .map(|(offset, i)| {
                        let base = shape_ids[i].ok_or("missing source style")?;
                        positioned_span_style(page, doc, line, i, base, &mut glyph_styles)
                            .map(|id| (offset, id))
                    })
                    .collect::<Result<_, _>>()?,
            )
        };
        // A visual wrap does not terminate an editable paragraph or insert a hard break.
        // This inferred word separator is recorded separately from source whitespace.
        if matches!(
            proposal.line_joins.get(line_no),
            Some(LineJoin::InferredWordSpace)
        ) {
            para.text.push(' ');
            utf16 += 1;
        }
        let line_start = utf16;
        if let Some(mut evidence) = materialization {
            evidence["context"] = json!(proposal.context);
            evidence["spans"] = json!(indices);
            evidence["lineIndex"] = json!(line_no);
            evidence["paragraphUtf16Start"] = json!(line_start);
            typography.borrow_mut().push(evidence);
        }
        for (run_offset, style) in runs {
            para.char_shapes.push(CharShapeRef {
                start_pos: utf16 + run_offset,
                char_shape_id: style,
            });
        }
        para.text.push_str(&text);
        utf16 += text.encode_utf16().count() as u32;
        para.line_segs.push(LineSeg {
            text_start: line_start,
            vertical_pos: hu(line_top - top)? as i32,
            line_height: hu(size)? as i32,
            text_height: hu(size)? as i32,
            baseline_distance: hu(size * 0.95)? as i32,
            line_spacing: hu((next_top - line_top - size).max(0.))? as i32,
            column_start: if centered {
                0
            } else {
                hu(line.left - cell_x)? as i32
            },
            segment_width: if centered {
                hu(cell_width)? as i32
            } else {
                hu(cell_width - (line.left - cell_x))? as i32
            },
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        });
        last_end = line_top + size;
    }
    finalize_text_axis(&mut para)?;
    para.has_para_text = true;
    let mut style = doc.doc_info.para_shapes[0].clone();
    style.raw_data = None;
    if fixed_line_height {
        style.attr1 &= !(1 << 8);
    }
    style.attr2 = (style.attr2 & !PARA_ATTR2_LINE_WRAP_MASK) | 1;
    style.alignment = if centered {
        Alignment::Center
    } else {
        Alignment::Left
    };
    style.margin_left = if centered {
        0
    } else {
        physical_margin(layout.margin_left)?
    };
    style.margin_right = 0;
    style.indent = if centered {
        0
    } else {
        let value = physical_margin(layout.indent.abs())?;
        if layout.indent < 0. {
            -value
        } else {
            value
        }
    };
    style.spacing_before = 0;
    style.spacing_after = 0;
    if !tabs.is_empty() {
        tabs.sort_by_key(|t| t.position);
        tabs.dedup_by_key(|t| t.position);
        style.tab_def_id = u16::try_from(doc.doc_info.tab_defs.len())
            .map_err(|_| "too many PDF line tab definitions")?;
        doc.doc_info.tab_defs.push(TabDef {
            tabs,
            ..Default::default()
        });
    }
    style.line_spacing_type = if fixed_line_height {
        LineSpacingType::Fixed
    } else {
        LineSpacingType::Percent
    };
    style.line_spacing = if fixed_line_height {
        physical_margin(
            lines
                .get(1)
                .map_or(first.font, |next| next.baseline - first.baseline),
        )?
    } else if lines.len() > 1 {
        ((lines[1].baseline - first.baseline) / first.font * 100.).round() as i32
    } else {
        100
    };
    para.para_shape_id = doc.doc_info.para_shapes.len() as u16;
    doc.doc_info.para_shapes.push(style);
    Ok(FlowItem {
        top,
        bottom: last_end,
        paragraph: para,
    })
}

fn is_grid_rule(shape: &ShapeObject, grid: &Value) -> Result<bool, String> {
    let (x1, y1, x2, y2) = match shape {
        ShapeObject::Line(line) => (
            (line.common.horizontal_offset as f32 + line.start.x as f32) / 100.,
            (line.common.vertical_offset as f32 + line.start.y as f32) / 100.,
            (line.common.horizontal_offset as f32 + line.end.x as f32) / 100.,
            (line.common.vertical_offset as f32 + line.end.y as f32) / 100.,
        ),
        ShapeObject::Rectangle(r)
            if r.drawing.text_box.is_none() && r.drawing.fill.solid.is_some() =>
        {
            let c = &r.common;
            let (x, y, w, h) = (
                c.horizontal_offset as f32 / 100.,
                c.vertical_offset as f32 / 100.,
                c.width as f32 / 100.,
                c.height as f32 / 100.,
            );
            if w < h {
                (x + w / 2., y, x + w / 2., y + h)
            } else {
                (x, y + h / 2., x + w, y + h / 2.)
            }
        }
        _ => return Ok(false),
    };
    // Only consume axes within the reconstructed grid, never arbitrary interior artwork.
    let b = &grid["bbox"];
    let inside = x1.min(x2) >= number(&b["x"])? - 0.5
        && x1.max(x2) <= number(&b["x"])? + number(&b["width"])? + 0.5
        && y1.min(y2) >= number(&b["y"])? - 0.5
        && y1.max(y2) <= number(&b["y"])? + number(&b["height"])? + 0.5;
    if !inside {
        return Ok(false);
    }
    let on_axis = |values: &Value, at: f32| -> Result<bool, String> {
        Ok(array(values)?
            .iter()
            .any(|v| number(v).is_ok_and(|n| (n - at).abs() <= 0.5)))
    };
    Ok(
        ((y1 - y2).abs() <= 0.5 && on_axis(&grid["yBoundaries"], y1)?)
            || ((x1 - x2).abs() <= 0.5 && on_axis(&grid["xBoundaries"], x1)?),
    )
}

struct TableBuilder<'a> {
    table_id_base: u32,
    page: &'a ExtractedPage,
    controls: &'a [Control],
    grids: &'a [Value],
    recognized: &'a [Value],
    span_controls: &'a [Option<usize>],
    shape_ids: &'a [Option<u32>],
    image_controls: &'a [usize],
    rule_controls: &'a [Option<usize>],
    styles: &'a crate::renderer::style_resolver::ResolvedStyleSet,
    typography: &'a std::cell::RefCell<Vec<Value>>,
}

fn cell_border(doc: &mut Document, controls: &[Control], bounds: &Value) -> Result<u16, String> {
    let (x, y, w, h) = (
        number(&bounds["x"])?,
        number(&bounds["y"])?,
        number(&bounds["width"])?,
        number(&bounds["height"])?,
    );
    let mut border = doc.doc_info.border_fills[1].clone();
    border.raw_data = None;
    for (side, (horizontal, at, from, to)) in [
        (false, x, y, y + h),
        (false, x + w, y, y + h),
        (true, y, x, x + w),
        (true, y + h, x, x + w),
    ]
    .into_iter()
    .enumerate()
    {
        let mut segments = Vec::new();
        let mut styles = BTreeSet::new();
        for control in controls {
            let Control::Shape(shape) = control else {
                continue;
            };
            let ShapeObject::Rectangle(r) = shape.as_ref() else {
                continue;
            };
            if r.drawing.text_box.is_some() {
                continue;
            }
            let Some(fill) = &r.drawing.fill.solid else {
                continue;
            };
            let c = &r.common;
            let (xx, yy, ww, hh) = (
                c.horizontal_offset as f32 / 100.,
                c.vertical_offset as f32 / 100.,
                c.width as f32 / 100.,
                c.height as f32 / 100.,
            );
            let (rule_h, rule_at, a, b) = if ww >= hh {
                (true, yy + hh / 2., xx, xx + ww)
            } else {
                (false, xx + ww / 2., yy, yy + hh)
            };
            let (a, b) = (a.max(from), b.min(to));
            if rule_h == horizontal && (rule_at - at).abs() <= 0.5 && b - a > 0.5 {
                segments.push((a, b));
                styles.insert((
                    fill.background_color,
                    border_width_index(ww.min(hh) as f64 * 25.4 / 72.),
                ));
            }
        }
        segments.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut end = from;
        for (a, b) in segments {
            if a > end + 0.5 {
                return Err("cell border coverage gap".into());
            }
            end = end.max(b);
        }
        if end < to - 0.5 || styles.len() != 1 {
            return Err("nonuniform or missing cell side style".into());
        }
        let &(color, width) = styles.first().unwrap();
        border.borders[side].line_type = BorderLineType::Solid;
        border.borders[side].color = color;
        border.borders[side].width = width;
    }
    if let Some(i) = doc.doc_info.border_fills.iter().position(|b| b == &border) {
        return u16::try_from(i + 1).map_err(|e| e.to_string());
    }
    doc.doc_info.border_fills.push(border);
    u16::try_from(doc.doc_info.border_fills.len()).map_err(|e| e.to_string())
}

impl TableBuilder<'_> {
    fn build(
        &self,
        doc: &mut Document,
        candidate: usize,
        removed: &mut BTreeSet<usize>,
        stack: &mut BTreeSet<usize>,
        family: bool,
        source_paint: bool,
    ) -> Result<Table, String> {
        if stack.len() >= 8 || !stack.insert(candidate) {
            return Err("invalid or too-deep nested table composition".into());
        }
        let result = self.build_inner(doc, candidate, removed, stack, family, source_paint);
        stack.remove(&candidate);
        result
    }

    fn build_inner(
        &self,
        doc: &mut Document,
        candidate: usize,
        removed: &mut BTreeSet<usize>,
        stack: &mut BTreeSet<usize>,
        family: bool,
        source_paint: bool,
    ) -> Result<Table, String> {
        let Self {
            page,
            controls,
            grids,
            recognized,
            span_controls,
            shape_ids,
            image_controls,
            rule_controls,
            ..
        } = self;
        let grid = grids.get(candidate).ok_or("missing recognized grid")?;
        let recognized_table = recognized
            .iter()
            .find(|r| r["candidate"].as_u64() == Some(candidate as u64))
            .ok_or("missing table recognition")?;
        if recognized_table["status"] != "GeometryConfirmed" {
            return Err("recognition_deferred".into());
        }
        let preserve_paint = source_paint;
        let fixed_lines = family || preserve_paint;
        let mut stroke_styles = BTreeSet::new();
        for c in *controls {
            if let Control::Shape(s) = c {
                if is_grid_rule(s, grid)? {
                    match s.as_ref() {
                        ShapeObject::Line(l) => {
                            stroke_styles.insert((
                                l.drawing.border_line.color,
                                l.drawing.border_line.width as u32,
                            ));
                        }
                        ShapeObject::Rectangle(r) => {
                            stroke_styles.insert((
                                r.drawing
                                    .fill
                                    .solid
                                    .as_ref()
                                    .ok_or("missing rule fill")?
                                    .background_color,
                                r.common.width.min(r.common.height),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
        if !family && !preserve_paint && stroke_styles.len() != 1 {
            return Err("mixed or missing rule styles require further lowering".into());
        }
        let (color, width) = if preserve_paint {
            (0, 0)
        } else {
            *stroke_styles.first().ok_or("missing rule style")?
        };
        let mut border = doc
            .doc_info
            .border_fills
            .get(1)
            .ok_or("missing scaffold solid border")?
            .clone();
        border.raw_data = None;
        for side in &mut border.borders {
            if family || preserve_paint {
                side.line_type = BorderLineType::None;
            } else {
                side.color = color;
                side.width = border_width_index(width as f64 / 100. * 25.4 / 72.);
            }
        }
        doc.doc_info.border_fills.push(border);
        let border_id =
            u16::try_from(doc.doc_info.border_fills.len()).map_err(|e| e.to_string())?;
        let b = &grid["bbox"];
        let extent = if preserve_paint {
            Some(super::table_paint::dimensions(grid, None)?)
        } else {
            None
        };
        let mut table = Table {
            row_count: u16::try_from(index(&grid["rows"])?).map_err(|e| e.to_string())?,
            col_count: u16::try_from(index(&grid["columns"])?).map_err(|e| e.to_string())?,
            border_fill_id: border_id,
            page_break: TablePageBreak::None,
            common: CommonObjAttr {
                width: extent.map_or_else(|| hu(number(&b["width"])?), |(w, _)| Ok(w))?,
                height: extent.map_or_else(|| hu(number(&b["height"])?), |(_, h)| Ok(h))?,
                horizontal_offset: hu(number(&b["x"])?)?,
                vertical_offset: hu(number(&b["y"])?)?,
                instance_id: self
                    .table_id_base
                    .checked_add(u32::try_from(candidate).map_err(|e| e.to_string())?)
                    .ok_or("table instance ID overflow")?,
                text_wrap: TextWrap::InFrontOfText,
                allow_overlap: true,
                ..Default::default()
            },
            ..Default::default()
        };
        table.row_sizes = vec![table.col_count as i16; table.row_count as usize];
        for recognized_cell in array(&recognized_table["cells"])? {
            let cell_idx = index(&recognized_cell["cell"])?;
            let source = array(&grid["cells"])?
                .get(cell_idx)
                .ok_or("missing cell geometry")?;
            let cb = &source["bbox"];
            let cell_border_id = if family && !preserve_paint {
                cell_border(doc, controls, cb)?
            } else {
                border_id
            };
            let extent = if preserve_paint {
                Some(super::table_paint::dimensions(grid, Some(source))?)
            } else {
                None
            };
            let mut cell = Cell::new_empty(
                index(&source["column"])? as u16,
                index(&source["row"])? as u16,
                extent.map_or_else(|| hu(number(&cb["width"])?), |(w, _)| Ok(w))?,
                extent.map_or_else(|| hu(number(&cb["height"])?), |(_, h)| Ok(h))?,
                cell_border_id,
            );
            cell.col_span = index(&source["colspan"])? as u16;
            cell.row_span = index(&source["rowspan"])? as u16;
            cell.vertical_align = VerticalAlign::Top;
            cell.apply_inner_margin = true;
            cell.paragraphs[0].char_shapes = vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }];
            let mut flow = Vec::new();
            let recognized_text = self
                .page
                .recognition
                .cells
                .get(&(candidate, cell_idx))
                .ok_or("missing typed cell text recognition")?;
            for proposal in &recognized_text.paragraphs {
                if proposal.status == ParagraphState::Confirmed {
                    flow.push(paragraph(
                        page,
                        doc,
                        proposal,
                        cb,
                        shape_ids,
                        fixed_lines,
                        self.styles,
                        self.typography,
                    )?);
                } else if fixed_lines {
                    // Preserve unresolved appearance without claiming recovered paragraph grouping.
                    for line in &proposal.lines {
                        let mut unresolved_line = proposal.clone();
                        unresolved_line.lines = vec![line.clone()];
                        unresolved_line.line_joins = vec![LineJoin::Start];
                        unresolved_line.layout = Some(ParagraphLayout {
                            alignment: TextAlignment::Left,
                            margin_left: line.left - number(&cb["x"])?,
                            indent: 0.,
                        });
                        unresolved_line.span_indices = line.indices.clone();
                        flow.push(paragraph(
                            page,
                            doc,
                            &unresolved_line,
                            cb,
                            shape_ids,
                            true,
                            self.styles,
                            self.typography,
                        )?);
                    }
                } else {
                    return Err("recognition_deferred".into());
                }
            }
            let children = recognized_cell["childTableIndices"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for child_ref in children {
                let child_id = index(&child_ref)?;
                let child_grid = grids.get(child_id).ok_or("missing child grid")?;
                if child_grid["hierarchy"]["parent"] != json!({"table":candidate,"cell":cell_idx}) {
                    return Err("nested cell-slot ownership mismatch".into());
                }
                let mut child = self.build(doc, child_id, removed, stack, true, source_paint)?;
                let child_box = &child_grid["bbox"];
                let top = number(&child_box["y"])?;
                let bottom = top + number(&child_box["height"])?;
                let left = hu(number(&child_box["x"])? - number(&cb["x"])?)?;
                child.common.treat_as_char = true;
                child.common.horz_rel_to = HorzRelTo::Para;
                child.common.vert_rel_to = VertRelTo::Para;
                child.common.horizontal_offset = 0;
                child.common.vertical_offset = 0;
                child.common.text_wrap = TextWrap::TopAndBottom;
                child.common.allow_overlap = false;
                child.common.flow_with_text = true;
                child.outer_margin_left =
                    i16::try_from(left).map_err(|_| "nested inset exceeds format range")?;
                let height = child.common.height;
                let mut host = Paragraph::new_empty();
                host.char_count_msb = true;
                host.char_count = 9;
                host.control_mask = 0x800;
                host.char_shapes = vec![CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                }];
                let mut ps = doc.doc_info.para_shapes[0].clone();
                ps.raw_data = None;
                ps.attr1 &= !(1 << 8);
                ps.margin_left = 0;
                ps.margin_right = 0;
                ps.indent = 0;
                ps.spacing_before = 0;
                ps.spacing_after = 0;
                ps.line_spacing_type = LineSpacingType::SpaceOnly;
                ps.line_spacing = 0;
                host.para_shape_id =
                    u16::try_from(doc.doc_info.para_shapes.len()).map_err(|e| e.to_string())?;
                doc.doc_info.para_shapes.push(ps);
                host.line_segs = vec![LineSeg {
                    line_height: height as i32,
                    text_height: height as i32,
                    baseline_distance: (height as f32 * 0.85).round() as i32,
                    segment_width: cell.width as i32,
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    ..Default::default()
                }];
                host.controls.push(Control::Table(Box::new(child)));
                flow.push(FlowItem {
                    top,
                    bottom,
                    paragraph: host,
                });
            }
            for image_idx in refs(&source["imageIndices"])? {
                let control_idx = *image_controls
                    .get(image_idx)
                    .ok_or("image/control map mismatch")?;
                let Control::Picture(original) = &controls[control_idx] else {
                    unreachable!()
                };
                let mut picture = original.clone();
                let top = picture.common.vertical_offset as f32 / 100.;
                picture.common.treat_as_char = true;
                picture.common.text_wrap = TextWrap::TopAndBottom;
                picture.common.allow_overlap = false;
                if fixed_lines {
                    picture.common.flow_with_text = true;
                }
                picture.common.vert_rel_to = VertRelTo::Para;
                picture.common.horz_rel_to = HorzRelTo::Para;
                picture.common.horizontal_offset = 0;
                picture.common.vertical_offset = 0;
                let mut para = Paragraph::new_empty();
                para.char_count_msb = true;
                para.char_count = 9;
                para.control_mask = 0x800;
                para.char_shapes = vec![CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                }];
                let mut ps = doc.doc_info.para_shapes[0].clone();
                ps.raw_data = None;
                ps.margin_left =
                    2 * (original.common.horizontal_offset as i32 - hu(number(&cb["x"])?)? as i32);
                ps.line_spacing = 100;
                if fixed_lines {
                    ps.line_spacing_type = LineSpacingType::Fixed;
                    ps.line_spacing = i32::try_from(picture.common.height)
                        .ok()
                        .and_then(|height| height.checked_mul(2))
                        .ok_or("picture line height exceeds format range")?;
                }
                ps.spacing_before = 0;
                ps.spacing_after = 0;
                para.para_shape_id = doc.doc_info.para_shapes.len() as u16;
                doc.doc_info.para_shapes.push(ps);
                para.line_segs = vec![LineSeg {
                    line_height: picture.common.height as i32,
                    text_height: picture.common.height as i32,
                    baseline_distance: picture.common.height as i32,
                    segment_width: cell.width as i32,
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    ..Default::default()
                }];
                let bottom = top + picture.common.height as f32 / 100.;
                para.controls.push(Control::Picture(picture));
                flow.push(FlowItem {
                    top,
                    bottom,
                    paragraph: para,
                });
                removed.insert(control_idx);
            }
            flow.sort_by(|a, b| a.top.total_cmp(&b.top));
            let mut cursor = number(&cb["y"])?;
            let mut previous_style: Option<u16> = None;
            for (flow_index, item) in flow.iter_mut().enumerate() {
                if item.top < cursor - 0.5 {
                    return Err("overlapping cell flow requires additional lowering".into());
                }
                let gap = (item.top - cursor).max(0.);
                if fixed_lines && flow_index == 0 {
                    // Derived content-frame inset, not a claim about the author's padding.
                    // Both cached placement and fresh reflow consume this same frame.
                    cell.padding.top =
                        i16::try_from(hu(gap)?).map_err(|_| "cell padding exceeds format range")?;
                    cursor = item.bottom;
                    previous_style = Some(item.paragraph.para_shape_id);
                    continue;
                }
                let picture = item.paragraph.controls.first().is_some_and(|c| {
                    matches!(
                        c,
                        Control::Picture(_) | Control::Table(_) | Control::Shape(_)
                    )
                });
                if picture {
                    // Place the inter-item gap before the inline image's flow origin.
                    if let Some(style) = previous_style {
                        doc.doc_info.para_shapes[style as usize].spacing_after =
                            physical_margin(gap)?;
                    } else {
                        cell.padding.top = i16::try_from(hu(gap)?)
                            .map_err(|_| "cell padding exceeds format range")?;
                    }
                } else {
                    doc.doc_info.para_shapes[item.paragraph.para_shape_id as usize]
                        .spacing_before = physical_margin(gap)?;
                }
                if flow_index == 0 && item.paragraph.controls.is_empty() {
                    // Stored cell starts own their lead in the frame, not flow_before.
                    let lead = hu((item.top - cursor).max(0.))? as i32;
                    for segment in &mut item.paragraph.line_segs {
                        segment.vertical_pos += lead;
                    }
                }
                cursor = item.bottom;
                previous_style = Some(item.paragraph.para_shape_id);
            }
            if !flow.is_empty() {
                cell.paragraphs = flow.into_iter().map(|i| i.paragraph).collect();
            }
            for i in refs(&source["spanIndices"])? {
                if let Some(c) = span_controls[i] {
                    removed.insert(c);
                }
            }
            table.cells.push(cell);
        }
        table.rebuild_grid();
        table.row_sizes = (0..table.row_count)
            .map(|row| table.cells.iter().filter(|c| c.row == row).count() as i16)
            .collect();
        if preserve_paint {
            // Page-wide partition attaches paint only after all accepted cell owners are known.
        } else if family {
            for path in refs(&grid["sourcePathIndices"])? {
                let control = rule_controls
                    .get(path)
                    .ok_or("invalid source rule reference")?
                    .ok_or("source boundary has no lowering representation")?;
                removed.insert(control);
            }
        } else {
            for (i, c) in controls.iter().enumerate() {
                if let Control::Shape(s) = c {
                    if is_grid_rule(s, grid)? {
                        removed.insert(i);
                    }
                }
            }
        }
        Ok(table)
    }
}

fn table_count(controls: &[Control]) -> usize {
    controls
        .iter()
        .map(|c| match c {
            Control::Table(t) => {
                1 + t
                    .cells
                    .iter()
                    .flat_map(|c| &c.paragraphs)
                    .map(|p| table_count(&p.controls))
                    .sum::<usize>()
            }
            _ => 0,
        })
        .sum()
}

fn paint_frame(control: &Control) -> Option<&CommonObjAttr> {
    match control {
        Control::Shape(shape) => Some(shape.common()),
        Control::Picture(picture) => Some(&picture.common),
        Control::Table(table) => Some(&table.common),
        _ => None,
    }
}

/// Body table paint can use the flow plane, whereas positional source controls
/// paint in front of text. A retained backdrop can consequently cover table
/// labels even with a larger table z-order. Promote only when no earlier
/// overlapping positional paint would cross that plane boundary.
fn table_paint_layer(
    controls: &[Control],
    consumed: &BTreeSet<usize>,
    bounds: &Value,
) -> Result<i32, String> {
    let last = consumed
        .iter()
        .filter_map(|&i| controls.get(i).and_then(paint_frame))
        .map(|c| c.z_order)
        .max()
        .ok_or("table has no source paint")?;
    let (x, y, width, height) = (
        number(&bounds["x"])?,
        number(&bounds["y"])?,
        number(&bounds["width"])?,
        number(&bounds["height"])?,
    );
    for (i, control) in controls.iter().enumerate() {
        if consumed.contains(&i) {
            continue;
        }
        let Some(frame) = paint_frame(control) else {
            continue;
        };
        if frame.z_order > last {
            continue;
        }
        let cx = (frame.horizontal_offset as i32) as f32 / 100.;
        let cy = (frame.vertical_offset as i32) as f32 / 100.;
        if cx < x + width
            && cx + frame.width as f32 / 100. > x
            && cy < y + height
            && cy + frame.height as f32 / 100. > y
        {
            return Err("positional backdrop cannot share inferred table flow plane".into());
        }
    }
    Ok(last)
}

pub(super) fn lower(
    page: &ExtractedPage,
    doc: &mut Document,
    next_id: &mut u32,
) -> Result<Value, String> {
    let mut report = super::lowering::lower(page, doc, next_id)?;
    let section = doc.sections.len() - 1;
    let controls = std::mem::take(&mut doc.sections[section].paragraphs[0].controls);
    let mut span_controls = vec![None; page.spans.len()];
    let mut shape_ids = vec![None; page.spans.len()];
    let mut image_controls = vec![0; page.images.len()];
    for source in array(&report["imageSourceControls"])? {
        let image = index(&source["image"])?;
        let control = index(&source["control"])?;
        *image_controls
            .get_mut(image)
            .ok_or("invalid image/control mapping")? = control;
    }
    let mut rule_controls = vec![None; page.paths.len()];
    for source in array(&report["ruleSourceControls"])? {
        let path = index(&source["path"])?;
        let control = index(&source["control"])?;
        *rule_controls
            .get_mut(path)
            .ok_or("invalid rule/control mapping")? = Some(control);
    }
    let nonempty: Vec<_> = page
        .spans
        .iter()
        .enumerate()
        .filter_map(|(i, s)| (!s.text.is_empty()).then_some(i))
        .collect();
    let mut span_cursor = 0;
    for (control_idx, c) in controls.iter().enumerate() {
        match c {
            Control::Picture(_) => {}
            Control::Shape(s) => {
                if let Some(b) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                    let source = *nonempty
                        .get(span_cursor)
                        .ok_or("span/control map mismatch")?;
                    span_controls[source] = Some(control_idx);
                    shape_ids[source] = Some(b.paragraphs[0].char_shapes[0].char_shape_id);
                    span_cursor += 1;
                }
            }
            _ => {}
        }
    }
    let grids = array(&page.snapshot["tableDetection"]["tables"])?;
    let recognized = array(&page.snapshot["recognition"]["tables"])?;
    let fragment_ids = std::cell::Cell::new(
        next_id
            .checked_add(u32::try_from(grids.len()).map_err(|e| e.to_string())?)
            .ok_or("table instance ID overflow")?,
    );
    let styles = crate::renderer::style_resolver::resolve_styles(&doc.doc_info, 96.);
    let typography = std::cell::RefCell::new(Vec::new());
    let builder = TableBuilder {
        table_id_base: *next_id,
        page,
        controls: &controls,
        grids,
        recognized,
        span_controls: &span_controls,
        shape_ids: &shape_ids,
        image_controls: &image_controls,
        rule_controls: &rule_controls,
        styles: &styles,
        typography: &typography,
    };
    *next_id = next_id
        .checked_add(u32::try_from(grids.len()).map_err(|e| e.to_string())?)
        .ok_or("table instance ID overflow")?;
    let mut removed = BTreeSet::new();
    let mut tables = Vec::new();
    let mut table_candidates = Vec::new();
    let mut table_removed = Vec::new();
    let mut held = Vec::new();
    let mut preserve_indices = Vec::new();
    for (candidate, grid) in grids.iter().enumerate() {
        if !grid["hierarchy"]["parent"].is_null() {
            continue;
        }
        let family = array(&grid["cells"])?.iter().any(|c| {
            c["childTableIndices"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        });
        let style_start = doc.doc_info.para_shapes.len();
        let char_style_start = doc.doc_info.char_shapes.len();
        let typography_start = typography.borrow().len();
        let border_start = doc.doc_info.border_fills.len();
        let tab_start = doc.doc_info.tab_defs.len();
        let mut candidate_removed = BTreeSet::new();
        let source_paint = composition::source_paint_family(grids, candidate)?;
        let built = builder
            .build(
                doc,
                candidate,
                &mut candidate_removed,
                &mut BTreeSet::new(),
                family,
                source_paint,
            )
            .and_then(|mut table| {
                if !source_paint {
                    table.common.z_order =
                        table_paint_layer(&controls, &candidate_removed, &grid["bbox"])?;
                }
                Ok(table)
            });
        match built {
            Ok(table) => {
                if source_paint {
                    preserve_indices.push(tables.len());
                }
                table_removed.push(candidate_removed);
                table_candidates.push(candidate);
                tables.push(Control::Table(Box::new(table)));
            }
            Err(reason) => {
                doc.doc_info.para_shapes.truncate(style_start);
                doc.doc_info.char_shapes.truncate(char_style_start);
                typography.borrow_mut().truncate(typography_start);
                doc.doc_info.border_fills.truncate(border_start);
                doc.doc_info.tab_defs.truncate(tab_start);
                held.push(json!({"candidate":candidate,"reason":reason}));
            }
        }
    }
    let paint_owners =
        composition::owners(&tables, &preserve_indices, grids, builder.table_id_base)?;
    let native_borders = paint_owners
        .iter()
        .map(|owner| native_borders::plan(page, &controls, &rule_controls, &grids[owner.candidate]))
        .collect::<Vec<_>>();
    let targets = paint_owners
        .iter()
        .zip(&native_borders)
        .map(|(owner, native)| super::table_paint::Target {
            grid: &grids[owner.candidate],
            native_border_controls: native.as_ref().map(|n| n.controls.as_slice()),
        })
        .collect::<Vec<_>>();
    let mut extra_paint = Vec::new();
    let mut paint_fragments = 0;
    let mut native_border_cells = 0;
    let mut native_border_fragments = 0;
    if !targets.is_empty() {
        let plan = super::table_paint::partition(
            &controls,
            &rule_controls,
            &targets,
            &page.snapshot["tableDetection"],
        )
        .and_then(|plan| {
            let consumed = table_removed
                .iter()
                .flatten()
                .copied()
                .chain(plan.removed.iter().copied())
                .collect::<BTreeSet<_>>();
            for &ti in &preserve_indices {
                table_paint_layer(&controls, &consumed, &grids[table_candidates[ti]]["bbox"])?;
            }
            Ok(plan)
        });
        match plan {
            Ok(mut plan) => {
                native_border_fragments = plan.native_border_fragments;
                removed.extend(plan.removed);
                let consumed = table_removed
                    .iter()
                    .flatten()
                    .copied()
                    .chain(removed.iter().copied())
                    .collect::<BTreeSet<_>>();
                for &ti in &preserve_indices {
                    let Control::Table(table) = &mut tables[ti] else {
                        unreachable!()
                    };
                    table.common.z_order = table_paint_layer(
                        &controls,
                        &consumed,
                        &grids[table_candidates[ti]]["bbox"],
                    )?;
                }
                for (target, owner) in paint_owners.iter().enumerate() {
                    let table = composition::table_mut(&mut tables[owner.root], owner.instance_id)
                        .ok_or("source paint owner missing from actual table hierarchy")?;
                    let grid = &grids[owner.candidate];
                    for (ci, cell) in table.cells.iter_mut().enumerate() {
                        if let Some(native) = &native_borders[target] {
                            let border = &native.borders[ci];
                            let id = if let Some(i) =
                                doc.doc_info.border_fills.iter().position(|b| b == border)
                            {
                                i + 1
                            } else {
                                doc.doc_info.border_fills.push(border.clone());
                                doc.doc_info.border_fills.len()
                            };
                            cell.border_fill_id = u16::try_from(id).map_err(|e| e.to_string())?;
                            native_border_cells += usize::from(
                                border
                                    .borders
                                    .iter()
                                    .any(|b| b.line_type != BorderLineType::None),
                            );
                        }
                        for mut paint in std::mem::take(&mut plan.cells[target][ci]) {
                            let id = fragment_ids.get();
                            fragment_ids
                                .set(id.checked_add(1).ok_or("cell paint instance ID overflow")?);
                            super::table_paint::anchor(
                                &mut paint,
                                cell,
                                &grid["cells"][ci]["bbox"],
                                id,
                            )?;
                            let para = &mut cell.paragraphs[0];
                            para.controls.push(paint);
                            para.control_mask |= 0x800;
                            para.char_count = para
                                .char_count
                                .checked_add(8)
                                .ok_or("cell paint text count overflow")?;
                            paint_fragments += 1;
                        }
                    }
                }
                for mut paint in plan.retained {
                    let id = fragment_ids.get();
                    fragment_ids.set(id.checked_add(1).ok_or("page paint instance ID overflow")?);
                    super::table_paint::assign_id(&mut paint, id);
                    extra_paint.push(paint);
                }
            }
            Err(reason) => {
                let mut kept = Vec::new();
                let mut kept_removed = Vec::new();
                for (ti, (table, consumed)) in tables.into_iter().zip(table_removed).enumerate() {
                    if preserve_indices.contains(&ti) {
                        held.push(json!({"candidate":table_candidates[ti],"reason":reason}));
                    } else {
                        kept.push(table);
                        kept_removed.push(consumed);
                    }
                }
                tables = kept;
                table_removed = kept_removed;
            }
        }
    }
    removed.extend(table_removed.into_iter().flatten());
    let root_count = tables.len();
    *next_id = fragment_ids.get();
    let table_count = table_count(&tables);
    let (body_frames, body_mapping) = body::lower(
        page,
        doc,
        &controls,
        &span_controls,
        &shape_ids,
        &styles,
        &typography,
        &mut removed,
        next_id,
    );
    let mut retained: Vec<_> = controls
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| (!removed.contains(&i)).then_some(c))
        .collect();
    retained.extend(extra_paint);
    retained.extend(tables);
    retained.extend(body_frames);
    let owner = &mut doc.sections[section].paragraphs[0];
    owner.controls = retained;
    owner.char_count = 1 + owner.controls.len() as u32 * 8;
    report["mode"] = json!("structured-tables-with-positional-fallback");
    report["semanticTables"] = json!(table_count);
    report["semanticRootTables"] = json!(root_count);
    report["semanticNestedTables"] = json!(table_count - root_count);
    report["nestedParagraphPolicy"] =
        json!("confirmed proposals share Paragraph IR; deferred line frames do not assert authored grouping");
    report["paragraphMappingPolicy"] = json!({
        "confirmed": "one proposal becomes one Paragraph with multiple source LineSegs",
        "deferred": "line frames retained; paragraph grouping unverified",
        "lineJoins": "inferred wrap connections; original whitespace retained",
        "bodyRegions": "confirmed owner-local paragraphs share the cell paragraph adapter; ambiguous source spans retained"
    });
    report["bodyParagraphMapping"] = body_mapping;
    report["sourceTypography"] = json!({
        "lines": typography.into_inner(),
        "policy": "recorded glyph origins; inferred spaces independent of span boundaries; HWP integer-percent character metrics",
        "fontRecovery": "unidentified source fonts use explicit metric-compatible fallback; original outlines remain unverified"
    });
    report["boundaryPaintPolicy"] = json!("standard borders or source paint partitioned into cell-owned drawings; no page-level duplicate");
    report["cellOwnedPaintFragments"] = json!(paint_fragments);
    report["nativeBorderCells"] = json!(native_border_cells);
    report["nativeBorderPaintFragments"] = json!(native_border_fragments);
    report["ruledTableCandidates"] = json!(grids.len());
    report["tableSemantics"] = json!("inferred-from-geometry-not-authored-metadata");
    report["textBoxes"] = json!(owner
        .controls
        .iter()
        .filter(|c| matches!(c,
        Control::Shape(s) if s.drawing().is_some_and(|d| d.text_box.is_some())))
        .count());
    report["straightRules"] = json!(owner
        .controls
        .iter()
        .filter(|c| matches!(c,
        Control::Shape(s) if matches!(s.as_ref(), ShapeObject::Line(_))))
        .count());
    report["paintedRuleRectangles"]=json!(owner.controls.iter().filter(|c|matches!(c,
        Control::Shape(s) if matches!(s.as_ref(),ShapeObject::Rectangle(r) if r.drawing.text_box.is_none() && r.drawing.fill.solid.is_some()))).count());
    report["deferredTables"] = json!(held);
    report["hancomCompatibility"] = json!("unverified-awaiting-editor-test");
    report["warnings"] = json!([
        "PDF table/paragraph semantics are inferred, not recovered authored metadata",
        "PDF fonts, exact glyph advances and paragraph formatting are not fully reconstructed",
        "Hancom editor interoperability must be checked independently"
    ]);
    Ok(report)
}
