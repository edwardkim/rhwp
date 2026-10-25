//! Editable layout: recognized body paragraphs leave their positioned frames
//! and become the section's own text flow.
//!
//! The fidelity layout keeps one positioned text box per recognized body
//! region, so a paragraph cannot continue from one column or page into the
//! next. This layout stores the same paragraphs the way a word processor
//! does: one section per run of equally sized pages, page margins and column
//! definitions measured from the observed text, a page break per source page
//! and a column break per source column. Stored line positions are rebased
//! from each frame to its column, so the first layout matches the source.
//!
//! Everything that is not a recognized body paragraph (retained spans,
//! tables, pictures, rules) keeps its paper position and is anchored to the
//! page's first paragraph, a minimal empty line at the top of the body.
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::page::{ColumnDef, ColumnDirection, ColumnType};
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::shape::ShapeObject;
use crate::model::style::LineSpacingType;
use serde_json::{json, Value};

/// A committed body frame and its observed text extent in page points.
pub(super) struct BodyFrame {
    pub inst_id: u32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Frames committed on one output section (one source page).
pub(super) struct PageFrames {
    pub section: usize,
    pub frames: Vec<BodyFrame>,
}

/// Frames from one page's lowering report (`bodyParagraphMapping`).
pub(super) fn page_frames(section: usize, report: &Value) -> PageFrames {
    let frames = report["bodyParagraphMapping"]["frames"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| f["status"] == "paragraph_ir")
        .filter_map(|f| {
            let b = f["textBox"].as_array()?;
            let n = |i: usize| b.get(i)?.as_f64().map(|v| v as f32);
            Some(BodyFrame {
                inst_id: u32::try_from(f["instanceId"].as_u64()?).ok()?,
                left: n(0)?,
                top: n(1)?,
                right: n(2)?,
                bottom: n(3)?,
            })
        })
        .collect();
    PageFrames { section, frames }
}

fn hu(points: f32) -> i32 {
    (points * 100.0).round() as i32
}

/// Column of each frame on one page, or `None` for a frame that spans
/// several columns (a title band, a wide box) and stays positioned. Tall
/// frames found the columns first, so a short spanning line cannot merge two
/// columns into one.
fn page_columns(frames: &[BodyFrame]) -> (Vec<(f32, f32)>, Vec<Option<usize>>) {
    let mut order: Vec<usize> = (0..frames.len()).collect();
    order.sort_by(|&a, &b| {
        let h = |i: usize| frames[i].bottom - frames[i].top;
        h(b).total_cmp(&h(a))
    });
    let mut columns: Vec<(f32, f32)> = Vec::new();
    let mut owner = vec![None; frames.len()];
    for i in order {
        let f = &frames[i];
        let hits: Vec<usize> = (0..columns.len())
            .filter(|&c| f.left < columns[c].1 && columns[c].0 < f.right)
            .collect();
        match hits.as_slice() {
            [] => {
                columns.push((f.left, f.right));
                owner[i] = Some(columns.len() - 1);
            }
            [c] => {
                columns[*c].0 = columns[*c].0.min(f.left);
                columns[*c].1 = columns[*c].1.max(f.right);
                owner[i] = Some(*c);
            }
            _ => {}
        }
    }
    // Number columns left to right.
    let mut rank: Vec<usize> = (0..columns.len()).collect();
    rank.sort_by(|&a, &b| columns[a].0.total_cmp(&columns[b].0));
    let mut position = vec![0; columns.len()];
    for (to, &from) in rank.iter().enumerate() {
        position[from] = to;
    }
    let sorted = rank.iter().map(|&c| columns[c]).collect();
    (
        sorted,
        owner.into_iter().map(|c| c.map(|c| position[c])).collect(),
    )
}

fn take_frame(paragraph: &mut Paragraph, inst_id: u32) -> Option<Vec<Paragraph>> {
    let at = paragraph.controls.iter().position(|c| {
        matches!(c, Control::Shape(s) if matches!(&**s, ShapeObject::Rectangle(r)
            if r.drawing.inst_id == inst_id && r.drawing.text_box.is_some()))
    })?;
    if paragraph.ctrl_data_records.len() == paragraph.controls.len() {
        paragraph.ctrl_data_records.remove(at);
    }
    let Control::Shape(shape) = paragraph.controls.remove(at) else {
        unreachable!()
    };
    let ShapeObject::Rectangle(rect) = *shape else {
        unreachable!()
    };
    // The anchor's control count shrinks with the removed frame.
    paragraph.char_count = paragraph.char_count.saturating_sub(8);
    rect.drawing.text_box.map(|b| b.paragraphs)
}

/// Move every control of `anchor` to the front of `paragraph`. Each extended
/// control takes eight stream units ahead of the first character, so text,
/// character shape and line start positions shift with it.
fn attach_leading_controls(paragraph: &mut Paragraph, mut anchor: Paragraph) {
    let count = anchor.controls.len();
    if count == 0 {
        return;
    }
    let mut records = std::mem::take(&mut anchor.ctrl_data_records);
    records.resize(count, None);
    let mut own = std::mem::take(&mut paragraph.ctrl_data_records);
    own.resize(paragraph.controls.len(), None);
    records.extend(own);
    let mut controls = std::mem::take(&mut anchor.controls);
    controls.append(&mut paragraph.controls);
    paragraph.controls = controls;
    paragraph.ctrl_data_records = records;
    paragraph.char_count += 8 * count as u32;
    paragraph.reserve_leading_extended_control_slots(count);
}

/// Columns as (left, right) boxes in HWPUNIT from the page edge. Unequal
/// columns store widths and gaps as proportions of the body (sum 32768), the
/// HWP 5.0 and HWPX convention; absolute values would overflow 16 bits.
/// Whether `next`, the first paragraph of a column or page, continues
/// `previous` from the column or page before. The column boundary cut a
/// paragraph when `next` starts on the continuation edge of `previous` without
/// an item marker, and either `previous` stops mid-sentence or it is an
/// indented paragraph, whose successors would start indented.
fn continues(doc: &Document, previous: &Paragraph, next: &Paragraph) -> bool {
    if previous.text.trim().is_empty()
        || next.text.trim().is_empty()
        || !next.controls.is_empty()
        || !next.tab_extended.is_empty()
        || super::recognition::starts_item(&next.text)
    {
        return false;
    }
    let shapes = &doc.doc_info.para_shapes;
    let (Some(p), Some(n)) = (
        shapes.get(previous.para_shape_id as usize),
        shapes.get(next.para_shape_id as usize),
    ) else {
        return false;
    };
    // Measures are twice HWPUNIT. A negative indent advances later lines.
    let continuation_left = p.margin_left + (-p.indent).max(0);
    let first_left = n.margin_left + n.indent.max(0);
    if (first_left - continuation_left).abs() > 200 {
        return false;
    }
    let ends_sentence = previous.text.trim_end().ends_with(|c: char| {
        matches!(
            c,
            '.' | '?' | '!' | ':' | '"' | '”' | '’' | '」' | '』' | ')' | '）'
        )
    });
    !ends_sentence || p.indent > 0
}

/// Append `next` to `previous`, keeping the stored rows of both: the rows of
/// `next` keep their own (column-relative) positions, as a paragraph that
/// continues in the next column stores them.
fn join(previous: &mut Paragraph, next: &Paragraph) {
    use super::text_analysis::{line_join, LineJoin};
    let last_line = previous.text.lines().last().unwrap_or("").to_string();
    let mut rows = std::mem::take(&mut previous.line_segs);
    if matches!(
        line_join(&last_line, &next.text),
        LineJoin::InferredWordSpace
    ) {
        let end = previous.char_offsets.last().map_or(0, |&o| {
            o + previous
                .text
                .chars()
                .last()
                .map_or(1, |c| c.len_utf16() as u32)
        });
        previous.text.push(' ');
        previous.char_offsets.push(end);
    }
    let at = previous.merge_from(next);
    let shift = previous.char_offsets.get(at).copied().unwrap_or(0);
    rows.extend(next.line_segs.iter().map(|seg| {
        let mut seg = seg.clone();
        seg.text_start += shift;
        seg
    }));
    previous.line_segs = rows;
}

/// Lay the pages' paragraphs into one flow. Paragraphs cut by a column or
/// page boundary are joined. Each page's positioned content rides on the
/// first paragraph that starts on that page, or on its own anchor line.
fn assemble(
    doc: &mut Document,
    out: &mut Vec<Paragraph>,
    pages: Vec<(Paragraph, Vec<Paragraph>, bool)>,
) -> usize {
    let mut joined = 0;
    for (page_no, (mut anchor, mut body, keep_anchor)) in pages.into_iter().enumerate() {
        if !keep_anchor && page_no > 0 {
            if let Some(first) = body.first_mut() {
                first.column_type = ColumnBreakType::Page;
            }
        }
        let mut cut = vec![false; body.len()];
        for j in 0..body.len() {
            if !matches!(
                body[j].column_type,
                ColumnBreakType::Column | ColumnBreakType::Page
            ) {
                continue;
            }
            let previous = if j > 0 { body.get(j - 1) } else { out.last() };
            cut[j] = previous.is_some_and(|p| continues(doc, p, &body[j]));
        }
        let host = (!keep_anchor)
            .then(|| cut.iter().position(|c| !c))
            .flatten();
        match host {
            Some(h) => {
                attach_leading_controls(&mut body[h], std::mem::take(&mut anchor));
            }
            None => {
                // The anchor opens the page, so nothing joins across its start.
                if page_no > 0 {
                    anchor.column_type = ColumnBreakType::Page;
                }
                if let Some(first) = cut.first_mut() {
                    *first = false;
                }
                if let Some(first) = body.first_mut() {
                    if first.column_type == ColumnBreakType::Page {
                        first.column_type = ColumnBreakType::None;
                    }
                }
                out.push(anchor);
            }
        }
        for (j, paragraph) in body.into_iter().enumerate() {
            match out.last_mut() {
                Some(previous) if cut[j] => {
                    // The joined paragraph ends where `paragraph` ended: its
                    // spacing to the following paragraph comes along.
                    let after =
                        doc.doc_info.para_shapes[paragraph.para_shape_id as usize].spacing_after;
                    doc.doc_info.para_shapes[previous.para_shape_id as usize].spacing_after = after;
                    join(previous, &paragraph);
                    joined += 1;
                }
                _ => out.push(paragraph),
            }
        }
    }
    joined
}

fn column_def(boxes: &[(i32, i32)]) -> ColumnDef {
    const TOTAL: i64 = 32768;
    let body = i64::from((boxes[boxes.len() - 1].1 - boxes[0].0).max(1));
    let mut parts = Vec::new();
    for (i, &(left, right)) in boxes.iter().enumerate() {
        parts.push(i64::from(right - left));
        parts.push(boxes.get(i + 1).map_or(0, |next| i64::from(next.0 - right)));
    }
    // Cumulative rounding keeps the proportions summing to exactly 32768.
    let mut edges = Vec::with_capacity(parts.len());
    let mut acc = 0;
    for part in &parts {
        acc += part;
        edges.push(acc * TOTAL / body);
    }
    let mut previous = 0;
    let scaled: Vec<i16> = edges
        .iter()
        .map(|&edge| {
            let value = (edge - previous).clamp(0, i64::from(i16::MAX)) as i16;
            previous = edge;
            value
        })
        .collect();
    let widths: Vec<i16> = scaled.iter().step_by(2).copied().collect();
    let gaps: Vec<i16> = scaled.iter().skip(1).step_by(2).copied().collect();
    let spacing = boxes.windows(2).next().map_or(0, |w| {
        (w[1].0 - w[0].1).clamp(0, i32::from(i16::MAX)) as i16
    });
    ColumnDef {
        column_type: ColumnType::Normal,
        column_count: boxes.len() as u16,
        direction: ColumnDirection::LeftToRight,
        same_width: false,
        spacing,
        widths,
        gaps,
        proportional_widths: true,
        ..Default::default()
    }
}

/// Replace each page's positioned body frames with section text flow.
pub(super) fn reflow(doc: &mut Document, pages: &[PageFrames]) -> Result<Value, String> {
    if pages.is_empty() {
        return Ok(json!({"layout":"editable","sections":0}));
    }
    let base = pages[0].section;
    // Runs of consecutive pages with the same paper become one section each.
    let mut runs: Vec<Vec<&PageFrames>> = Vec::new();
    for page in pages {
        let pd = &doc.sections[page.section].section_def.page_def;
        let same = runs.last().and_then(|run| run.last()).is_some_and(|prev| {
            let q = &doc.sections[prev.section].section_def.page_def;
            (q.width, q.height, q.landscape) == (pd.width, pd.height, pd.landscape)
        });
        if same {
            runs.last_mut().unwrap().push(page);
        } else {
            runs.push(vec![page]);
        }
    }
    let mut old: Vec<_> = doc.sections.drain(base..).collect();
    let mut report = Vec::new();
    let mut anchor_shape = None;
    for run in runs {
        let first = run[0].section - base;
        let pd = old[first].section_def.page_def.clone();
        let width = if pd.landscape { pd.height } else { pd.width } as i32;
        let height = if pd.landscape { pd.width } else { pd.height } as i32;
        let layouts: Vec<_> = run.iter().map(|p| page_columns(&p.frames)).collect();
        // The section's column grid is the most common page grid.
        let mut counts = std::collections::BTreeMap::new();
        for (columns, _) in &layouts {
            if !columns.is_empty() {
                *counts.entry(columns.len()).or_insert(0usize) += 1;
            }
        }
        let Some((&count, _)) = counts.iter().max_by_key(|(n, c)| (**c, **n)) else {
            // Nothing recognized: keep the positioned pages as they were.
            for page in &run {
                doc.sections.push(old[page.section - base].clone());
            }
            continue;
        };
        let grid: Vec<&Vec<(f32, f32)>> = layouts
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.len() == count)
            .collect();
        let median = |values: Vec<f32>| {
            let mut v = values;
            v.sort_by(f32::total_cmp);
            v[v.len() / 2]
        };
        let lefts: Vec<i32> = (0..count)
            .map(|c| hu(median(grid.iter().map(|g| g[c].0).collect())))
            .collect();
        let rights: Vec<i32> = (0..count)
            .map(|c| hu(median(grid.iter().map(|g| g[c].1).collect())))
            .collect();
        let gap = if count > 1 {
            (0..count - 1)
                .map(|c| lefts[c + 1] - rights[c])
                .sum::<i32>()
                / (count as i32 - 1)
        } else {
            0
        };
        // Each column keeps its observed text edges. The renderer admits the
        // stored rows only for this exact frame; a wider box re-breaks them.
        let boxes: Vec<(i32, i32)> = (0..count)
            .map(|c| (lefts[c].max(0), rights[c].min(width)))
            .collect();
        let margin_left = boxes[0].0;
        let margin_right = (width - boxes[count - 1].1).max(0);
        let column_left = |c: usize| boxes[c].0;
        let column_width_of = |c: usize| boxes[c].1 - boxes[c].0;
        // A frame joins the flow when its column maps onto one grid column.
        let mut placed: Vec<Vec<Option<usize>>> = run
            .iter()
            .zip(&layouts)
            .map(|(page, (columns, owner))| {
                let grid_of = |c: usize| {
                    let center = hu((columns[c].0 + columns[c].1) / 2.);
                    (0..count).min_by_key(|&g| ((boxes[g].0 + boxes[g].1) / 2 - center).abs())
                };
                let mapped: Vec<Option<usize>> = (0..columns.len()).map(grid_of).collect();
                page.frames
                    .iter()
                    .zip(owner)
                    .map(|(frame, o)| {
                        let c = (*o)?;
                        let g = mapped[c]?;
                        // Text outside the column box (a page number in the
                        // gutter) is not column flow; it keeps its position.
                        let (left, right) = boxes[g];
                        if hu(frame.left) < left - 100 || hu(frame.right) > right + 100 {
                            return None;
                        }
                        // Two page columns on one grid column are not a column flow.
                        (mapped.iter().filter(|m| **m == Some(g)).count() == 1).then_some(g)
                    })
                    .collect()
            })
            .collect();
        // A short frame that recurs at one height on half the pages or more
        // is a running header or footer (page number, booklet marks). It
        // keeps its page position instead of opening a column.
        let band = |f: &BodyFrame| (hu(f.top) / 200, hu(f.bottom) / 200);
        let mut recurring = std::collections::BTreeMap::new();
        for page in &run {
            let bands: std::collections::BTreeSet<_> = page
                .frames
                .iter()
                .filter(|f| f.bottom - f.top < 30.0)
                .map(band)
                .collect();
            for b in bands {
                *recurring.entry(b).or_insert(0usize) += 1;
            }
        }
        let running = run.len() >= 2;
        for (page, owners) in run.iter().zip(placed.iter_mut()) {
            for (frame, owner) in page.frames.iter().zip(owners.iter_mut()) {
                let repeats = recurring.get(&band(frame)).copied().unwrap_or(0);
                if running && frame.bottom - frame.top < 30.0 && 2 * repeats >= run.len() {
                    *owner = None;
                }
            }
        }
        // Body text starts at one height on most pages. A cover page whose
        // text starts higher keeps that text positioned, so every column top
        // is the page body top and a paragraph continued into the next column
        // starts there, as stored rows of a continued paragraph do.
        let page_tops: Vec<f32> = run
            .iter()
            .zip(&placed)
            .filter_map(|(p, owners)| {
                p.frames
                    .iter()
                    .zip(owners)
                    .filter(|(_, g)| g.is_some())
                    .map(|(f, _)| f.top)
                    .reduce(f32::min)
            })
            .collect();
        let body_top = median(page_tops);
        for (page, owners) in run.iter().zip(placed.iter_mut()) {
            for (frame, owner) in page.frames.iter().zip(owners.iter_mut()) {
                if frame.top < body_top - 2.0 {
                    *owner = None;
                }
            }
        }
        let flowing = || {
            run.iter()
                .zip(&placed)
                .flat_map(|(p, owners)| p.frames.iter().zip(owners))
                .filter(|(_, g)| g.is_some())
                .map(|(f, _)| f)
        };
        let margin_top = hu(body_top).max(0);
        let margin_bottom =
            (height - hu(flowing().map(|f| f.bottom).fold(f32::MIN, f32::max))).max(0);
        let mut section = old[first].clone();
        section.paragraphs.clear();
        {
            let pd = &mut section.section_def.page_def;
            pd.margin_left = margin_left as u32;
            pd.margin_right = margin_right as u32;
            pd.margin_top = margin_top as u32;
            pd.margin_bottom = margin_bottom as u32;
            pd.margin_header = 0;
            pd.margin_footer = 0;
        }
        // An anchor line that reserves almost no height: fixed 1pt spacing.
        let anchor_shape_id = *anchor_shape.get_or_insert_with(|| {
            let mut shape = doc.doc_info.para_shapes[0].clone();
            shape.raw_data = None;
            shape.line_spacing_type = LineSpacingType::Fixed;
            shape.line_spacing = 200;
            shape.spacing_before = 0;
            shape.spacing_after = 0;
            doc.doc_info.para_shapes.push(shape);
            (doc.doc_info.para_shapes.len() - 1) as u16
        });
        let def = column_def(&boxes);
        let mut pages_report = Vec::new();
        let mut pages_flow = Vec::new();
        for (page_no, page) in run.iter().enumerate() {
            let mut anchor = std::mem::take(&mut old[page.section - base].paragraphs[0]);
            if page_no == 0 {
                // The section opens with its column grid, after the section
                // definition when the anchor carries one.
                anchor
                    .controls
                    .retain(|c| !matches!(c, Control::ColumnDef(_)));
                let at = anchor
                    .controls
                    .iter()
                    .position(|c| !matches!(c, Control::SectionDef(_)))
                    .unwrap_or(anchor.controls.len());
                anchor.controls.insert(at, Control::ColumnDef(def.clone()));
                anchor.char_count += 8;
            } else {
                // One section definition and one column grid open the section.
                let before = anchor.controls.len();
                anchor
                    .controls
                    .retain(|c| !matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)));
                let removed = (before - anchor.controls.len()) as u32;
                anchor.char_count = anchor.char_count.saturating_sub(8 * removed);
                anchor.column_type = ColumnBreakType::Page;
            }
            // Positioned content rides on the page's first column-0 paragraph.
            // A separate anchor line is kept only when column 0 has none.
            let keep_anchor = !placed[page_no].contains(&Some(0));
            anchor.para_shape_id = anchor_shape_id;
            for seg in &mut anchor.line_segs {
                seg.vertical_pos = 0;
                seg.line_height = 100;
                seg.text_height = 100;
                seg.baseline_distance = 85;
                seg.line_spacing = 100;
                seg.column_start = 0;
                seg.segment_width = column_width_of(0);
            }
            let mut body = Vec::new();
            let mut moved = 0;
            let mut current_column = 0;
            for g in 0..count {
                let mut members: Vec<usize> = (0..page.frames.len())
                    .filter(|&i| placed[page_no][i] == Some(g))
                    .collect();
                members.sort_by(|&a, &b| page.frames[a].top.total_cmp(&page.frames[b].top));
                // The anchor line occupies the top of the first column.
                let mut previous_bottom = margin_top + if g == 0 && keep_anchor { 200 } else { 0 };
                let col_left = column_left(g);
                for &i in &members {
                    let frame = &page.frames[i];
                    let Some(owner_left) = anchor.controls.iter().find_map(|c| match c {
                        Control::Shape(s) => match &**s {
                            ShapeObject::Rectangle(r) if r.drawing.inst_id == frame.inst_id => {
                                Some(r.common.horizontal_offset as i32)
                            }
                            _ => None,
                        },
                        _ => None,
                    }) else {
                        continue;
                    };
                    let Some(mut paragraphs) = take_frame(&mut anchor, frame.inst_id) else {
                        continue;
                    };
                    let top = hu(frame.top);
                    let shift_x = owner_left - col_left;
                    for (pi, p) in paragraphs.iter_mut().enumerate() {
                        let shape = &mut doc.doc_info.para_shapes[p.para_shape_id as usize];
                        // Paragraph measures are stored as twice HWPUNIT.
                        shape.margin_left = (shape.margin_left + 2 * shift_x).max(0);
                        shape.margin_right = 0;
                        if pi == 0 {
                            shape.spacing_before = 2 * (top - previous_bottom).max(0);
                        }
                        // Body rows reproduce the frame the renderer derives
                        // from the column and the paragraph shape: every row
                        // starts at the left margin (the indent is applied
                        // inside the row) and ends at the column's right edge
                        // snapped down to 4 HWPUNIT. Rows that miss this key
                        // are broken again with substitute font metrics.
                        let start = shape.margin_left / 2;
                        let end = column_width_of(g)
                            - column_width_of(g).rem_euclid(4)
                            - shape.margin_right / 2;
                        for seg in &mut p.line_segs {
                            seg.vertical_pos += top - margin_top;
                            seg.column_start = start;
                            seg.segment_width = (end - start).max(0);
                        }
                    }
                    if g > current_column {
                        // Each skipped grid column needs its own column break.
                        for _ in current_column + 1..g {
                            let mut empty = Paragraph::new_empty();
                            empty.column_type = ColumnBreakType::Column;
                            body.push(empty);
                        }
                        paragraphs[0].column_type = ColumnBreakType::Column;
                        current_column = g;
                    }
                    previous_bottom = hu(frame.bottom);
                    moved += paragraphs.len();
                    body.extend(paragraphs);
                }
            }
            let positioned =
                page.frames.len() - placed[page_no].iter().filter(|g| g.is_some()).count();
            pages_report.push(json!({"sourceSection":page.section,"columns":count,"bodyParagraphs":moved,"positionedFrames":positioned}));
            pages_flow.push((anchor, body, keep_anchor));
        }
        let joined = assemble(doc, &mut section.paragraphs, pages_flow);
        report.push(json!({"pages":pages_report,"columns":count,"columnGap":gap,"joinedParagraphs":joined,"margins":[margin_left,margin_top,margin_right,margin_bottom]}));
        doc.sections.push(section);
    }
    Ok(json!({"layout":"editable","sections":report}))
}
