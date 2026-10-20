//! Materialize geometry-confirmed grids as editable rhwp tables. Uncertain grids stay positional.
use crate::{lowering::hu, ExtractedPage};
use rhwp::model::{
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
use std::collections::BTreeSet;

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

fn offsets(text: &str) -> (Vec<u32>, u32) {
    let mut len = 0;
    let offsets = text
        .chars()
        .map(|c| {
            let at = len;
            len += c.len_utf16() as u32;
            at
        })
        .collect();
    (offsets, len)
}

struct FlowItem {
    top: f32,
    bottom: f32,
    paragraph: Paragraph,
}

fn paragraph(
    page: &ExtractedPage,
    doc: &mut Document,
    proposal: &Value,
    cell_box: &Value,
    shape_ids: &[Option<u32>],
    fixed_line_height: bool,
) -> Result<FlowItem, String> {
    let lines = array(&proposal["lines"])?;
    let first = lines.first().ok_or("paragraph has no lines")?;
    let cell_x = number(&cell_box["x"])?;
    let cell_width = number(&cell_box["width"])?;
    let top = number(&first["baseline"])? - number(&first["fontSize"])? * 0.95;
    let centered = !fixed_line_height
        && lines.iter().all(|l| {
            let b = &l["bbox"];
            let mid =
                b["x"].as_f64().unwrap_or(f64::NAN) + b["width"].as_f64().unwrap_or(f64::NAN) / 2.;
            (mid as f32 - cell_x - cell_width / 2.).abs()
                <= number(&l["fontSize"]).unwrap_or(0.) * 0.75
        });
    let mut para = Paragraph::new_empty();
    para.char_count_msb = true;
    para.char_shapes.clear();
    para.line_segs.clear();
    let mut last_end = top;
    let mut utf16 = 0;
    let mut tabs = Vec::new();
    for (line_no, line) in lines.iter().enumerate() {
        let indices = refs(&line["spanIndices"])?;
        if indices.is_empty() {
            return Err("empty recognized line".into());
        }
        let baseline = number(&line["baseline"])?;
        let size = number(&line["fontSize"])?;
        let line_top = baseline - size * 0.95;
        let next_top = if let Some(next) = lines.get(line_no + 1) {
            number(&next["baseline"])? - number(&next["fontSize"])? * 0.95
        } else {
            line_top + size
        };
        let (text, runs) = if fixed_line_height {
            let (text, runs, line_tabs, extensions) = text_with_tabs(page, &indices, cell_x)?;
            tabs.extend(line_tabs);
            para.tab_extended.extend(extensions);
            (text, runs)
        } else {
            text_with_spaces(page, &indices)
        };
        // Preserve explicit PDF line grouping; paragraph boundaries remain heuristic.
        if line_no > 0 {
            para.text.push('\n');
            utf16 += 1;
        }
        let line_start = utf16;
        for (run_offset, i) in runs {
            let style = shape_ids[i].ok_or("span style was not generated")?;
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
                hu(number(&line["bbox"]["x"])? - cell_x)? as i32
            },
            segment_width: if centered {
                hu(cell_width)? as i32
            } else {
                hu(cell_width - (number(&line["bbox"]["x"])? - cell_x))? as i32
            },
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        });
        last_end = line_top + size;
    }
    let (char_offsets, count) = offsets(&para.text);
    para.char_offsets = char_offsets;
    para.char_count = count + 1;
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
    } else if fixed_line_height {
        physical_margin(number(&first["bbox"]["x"])? - cell_x)?
    } else {
        hu(2. * (number(&first["bbox"]["x"])? - cell_x))? as i32
    };
    style.margin_right = 0;
    style.indent = 0;
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
        physical_margin(number(&first["fontSize"])?)?
    } else if lines.len() > 1 {
        ((number(&lines[1]["baseline"])? - number(&first["baseline"])?)
            / number(&first["fontSize"])?
            * 100.)
            .round() as i32
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
    page: &'a ExtractedPage,
    controls: &'a [Control],
    grids: &'a [Value],
    recognized: &'a [Value],
    span_controls: &'a [Option<usize>],
    shape_ids: &'a [Option<u32>],
    image_controls: &'a [usize],
    rule_controls: &'a [Option<usize>],
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
    ) -> Result<Table, String> {
        if stack.len() >= 8 || !stack.insert(candidate) {
            return Err("invalid or too-deep nested table composition".into());
        }
        let result = self.build_inner(doc, candidate, removed, stack, family);
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
        } = self;
        let grid = grids.get(candidate).ok_or("missing recognized grid")?;
        let recognized_table = recognized
            .iter()
            .find(|r| r["candidate"].as_u64() == Some(candidate as u64))
            .ok_or("missing table recognition")?;
        if recognized_table["status"] != "GeometryConfirmed" {
            return Err("recognition_deferred".into());
        }
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
        if !family && stroke_styles.len() != 1 {
            return Err("mixed or missing rule styles require further lowering".into());
        }
        let &(color, width) = stroke_styles.first().ok_or("missing rule style")?;
        let mut border = doc
            .doc_info
            .border_fills
            .get(1)
            .ok_or("missing scaffold solid border")?
            .clone();
        border.raw_data = None;
        for side in &mut border.borders {
            if family {
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
        let mut table = Table {
            row_count: u16::try_from(index(&grid["rows"])?).map_err(|e| e.to_string())?,
            col_count: u16::try_from(index(&grid["columns"])?).map_err(|e| e.to_string())?,
            border_fill_id: border_id,
            page_break: TablePageBreak::None,
            common: CommonObjAttr {
                width: hu(number(&b["width"])?)?,
                height: hu(number(&b["height"])?)?,
                horizontal_offset: hu(number(&b["x"])?)?,
                vertical_offset: hu(number(&b["y"])?)?,
                instance_id: u32::try_from(controls.len() + 2 + candidate * 2)
                    .map_err(|e| e.to_string())?,
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
            let cell_border_id = if family {
                cell_border(doc, controls, cb)?
            } else {
                border_id
            };
            let mut cell = Cell::new_empty(
                index(&source["column"])? as u16,
                index(&source["row"])? as u16,
                hu(number(&cb["width"])?)?,
                hu(number(&cb["height"])?)?,
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
            for proposal in array(&recognized_cell["recognition"]["paragraphs"])? {
                if family {
                    // PDF line geometry is evidence even when authored paragraph grouping is unresolved.
                    for line in array(&proposal["lines"])? {
                        flow.push(paragraph(
                            page,
                            doc,
                            &json!({"lines":[line]}),
                            cb,
                            shape_ids,
                            true,
                        )?);
                    }
                } else if proposal["status"] == "Confirmed" {
                    flow.push(paragraph(page, doc, proposal, cb, shape_ids, false)?);
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
                let mut child = self.build(doc, child_id, removed, stack, true)?;
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
                if family {
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
                if family {
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
                if family && flow_index == 0 {
                    // Cell inset is shared by cached placement and a consumer's fresh reflow.
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
                        doc.doc_info.para_shapes[style as usize].spacing_after = if family {
                            physical_margin(gap)?
                        } else {
                            hu(2. * gap)? as i32
                        };
                    } else {
                        cell.padding.top = i16::try_from(hu(gap)?)
                            .map_err(|_| "cell padding exceeds format range")?;
                    }
                } else {
                    doc.doc_info.para_shapes[item.paragraph.para_shape_id as usize]
                        .spacing_before = if family {
                        physical_margin(gap)?
                    } else {
                        hu(2. * gap)? as i32
                    };
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
        if family {
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

pub(crate) fn lower(page: &ExtractedPage) -> Result<(Document, Value), String> {
    let (mut doc, mut report) = crate::lowering::lower(page, true)?;
    let controls = std::mem::take(&mut doc.sections[0].paragraphs[0].controls);
    let mut span_controls = vec![None; page.spans.len()];
    let mut shape_ids = vec![None; page.spans.len()];
    let mut image_controls = Vec::new();
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
            Control::Picture(_) => image_controls.push(control_idx),
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
    let builder = TableBuilder {
        page,
        controls: &controls,
        grids,
        recognized,
        span_controls: &span_controls,
        shape_ids: &shape_ids,
        image_controls: &image_controls,
        rule_controls: &rule_controls,
    };
    let mut removed = BTreeSet::new();
    let mut tables = Vec::new();
    let mut held = Vec::new();
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
        let border_start = doc.doc_info.border_fills.len();
        let tab_start = doc.doc_info.tab_defs.len();
        let mut candidate_removed = BTreeSet::new();
        let built = builder.build(
            &mut doc,
            candidate,
            &mut candidate_removed,
            &mut BTreeSet::new(),
            family,
        );
        match built {
            Ok(table) => {
                removed.extend(candidate_removed);
                tables.push(Control::Table(Box::new(table)));
            }
            Err(reason) => {
                doc.doc_info.para_shapes.truncate(style_start);
                doc.doc_info.border_fills.truncate(border_start);
                doc.doc_info.tab_defs.truncate(tab_start);
                held.push(json!({"candidate":candidate,"reason":reason}));
            }
        }
    }
    let root_count = tables.len();
    let table_count = table_count(&tables);
    let mut retained: Vec<_> = controls
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| (!removed.contains(&i)).then_some(c))
        .collect();
    retained.extend(tables);
    let owner = &mut doc.sections[0].paragraphs[0];
    owner.controls = retained;
    owner.char_count = 1 + owner.controls.len() as u32 * 8;
    report["mode"] = json!("structured-tables-with-positional-fallback");
    report["semanticTables"] = json!(table_count);
    report["semanticRootTables"] = json!(root_count);
    report["semanticNestedTables"] = json!(table_count - root_count);
    report["nestedParagraphPolicy"] =
        json!("PDF lines retained separately; authored paragraph grouping is not asserted");
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
        "Only selected source page is converted; Hancom editor interoperability is unverified"
    ]);
    Ok((doc, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_oxide::elements::{PathContent, PathOperation};

    #[test]
    fn pdf_physical_measure_cannot_turn_into_character_units() {
        assert_eq!(physical_margin(3.725).unwrap(), 746);
        assert_eq!(physical_margin(0.).unwrap(), 0);
        assert!(physical_margin(f32::INFINITY).is_err());
        assert!(physical_margin(20_000_000.).is_err());
    }

    fn grid() -> Vec<PathContent> {
        [
            (10., 10., 70., 10.),
            (10., 40., 70., 40.),
            (10., 70., 70., 70.),
            (10., 10., 10., 70.),
            (40., 10., 40., 70.),
            (70., 10., 70., 70.),
        ]
        .into_iter()
        .map(|(x, y, xx, yy)| {
            PathContent::from_operations(vec![
                PathOperation::MoveTo(x, y),
                PathOperation::LineTo(xx, yy),
            ])
        })
        .collect()
    }
    fn page(paths: Vec<PathContent>) -> ExtractedPage {
        let media_box = (0., 0., 200., 200.);
        let detection =
            crate::table_detection::detect_ruled_tables(&[], &paths, &[], media_box).unwrap();
        let recognition = crate::recognition::recognize_page(
            &[],
            &[],
            &detection,
            media_box,
            crate::recognition::Policy::default(),
        )
        .unwrap();
        ExtractedPage {
            media_box,
            spans: vec![],
            images: vec![],
            paths,
            snapshot: json!({"tableDetection":detection,"recognition":recognition,"tables":[]}),
        }
    }
    #[test]
    fn empty_confirmed_grid_becomes_cells_and_merged_ownership_is_preserved() {
        let mut paths = grid();
        paths[4] = PathContent::from_operations(vec![
            PathOperation::MoveTo(40., 10.),
            PathOperation::LineTo(40., 40.),
        ]);
        let (doc, report) = lower(&page(paths)).unwrap();
        assert_eq!(report["semanticTables"], 1);
        let Control::Table(t) = &doc.sections[0].paragraphs[0].controls[0] else {
            panic!("actual table required")
        };
        assert_eq!((t.row_count, t.col_count, t.cells.len()), (2, 2, 3));
        assert!(t.cells.iter().any(|c| c.col_span == 2));
        assert_eq!(t.cell_grid[0], t.cell_grid[1]);
        assert!(t.cells.iter().all(|c| c.paragraphs.len() == 1));
    }
    #[test]
    fn deferred_candidate_keeps_every_positional_control() {
        let mut p = page(grid());
        p.snapshot["recognition"]["tables"][0]["status"] = json!("Deferred");
        let (baseline, _) = crate::lowering::lower(&p, true).unwrap();
        let (doc, report) = lower(&p).unwrap();
        assert_eq!(report["semanticTables"], 0);
        assert_eq!(report["deferredTables"].as_array().unwrap().len(), 1);
        assert_eq!(
            serde_json::to_value(&doc.sections[0].paragraphs).unwrap(),
            serde_json::to_value(&baseline.sections[0].paragraphs).unwrap()
        );
    }

    #[test]
    fn rowspan_keeps_all_logical_slots() {
        let mut paths = grid();
        paths[1] = PathContent::from_operations(vec![
            PathOperation::MoveTo(40., 40.),
            PathOperation::LineTo(70., 40.),
        ]);
        let (doc, report) = lower(&page(paths)).unwrap();
        assert_eq!(report["semanticTables"], 1);
        let Control::Table(t) = &doc.sections[0].paragraphs[0].controls[0] else {
            panic!("actual table required")
        };
        assert_eq!(t.cells.len(), 3);
        assert!(t.cells.iter().any(|c| c.row_span == 2));
        assert_eq!(t.cell_grid[0], t.cell_grid[2]);
    }

    #[test]
    fn mixed_rule_styles_are_deferred_without_color_loss() {
        let mut paths = grid();
        paths[0].stroke_color = Some(pdf_oxide::layout::Color {
            r: 1.,
            g: 0.,
            b: 0.,
        });
        let p = page(paths);
        let (baseline, _) = crate::lowering::lower(&p, true).unwrap();
        let (doc, report) = lower(&p).unwrap();
        assert_eq!(report["semanticTables"], 0);
        assert_eq!(report["deferredTables"].as_array().unwrap().len(), 1);
        assert_eq!(
            serde_json::to_value(&doc.sections[0].paragraphs).unwrap(),
            serde_json::to_value(&baseline.sections[0].paragraphs).unwrap()
        );
    }
    #[test]
    fn failed_candidate_does_not_drop_rules_or_leave_styles() {
        let mut p = page(grid());
        p.snapshot["tableDetection"]["tables"][0]["bbox"]["width"] = json!(-1);
        let (baseline, _) = crate::lowering::lower(&p, true).unwrap();
        let (doc, report) = lower(&p).unwrap();
        assert_eq!(report["semanticTables"], 0);
        assert_eq!(
            serde_json::to_value(&doc.doc_info.para_shapes).unwrap(),
            serde_json::to_value(&baseline.doc_info.para_shapes).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&doc.sections[0].paragraphs).unwrap(),
            serde_json::to_value(&baseline.sections[0].paragraphs).unwrap()
        );
    }
}
