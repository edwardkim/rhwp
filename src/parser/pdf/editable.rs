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
use super::{allocation::Allocation, value_macro::request_json, PdfImportError};
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::page::{ColumnDef, ColumnDirection, ColumnType};
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::shape::ShapeObject;
use crate::model::style::LineSpacingType;
use crate::storage::Storage;
use serde_json::Value;

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
pub(super) fn page_frames(
    section: usize,
    report: &Value,
    allocation: Allocation<'_>,
) -> Result<PageFrames, PdfImportError> {
    let frames = allocation.collect(
        report["bodyParagraphMapping"]["frames"]
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
            }),
    )?;
    Ok(PageFrames { section, frames })
}

/// Height in HWPUNIT that an anchor line (no text) takes in its column.
const ANCHOR_HEIGHT: i32 = 100;

fn hu(points: f32) -> i32 {
    (points * 100.0).round() as i32
}

/// Column of each frame on one page, or `None` for a frame that spans
/// several columns (a title band, a wide box) and stays positioned. Tall
/// frames found the columns first, so a short spanning line cannot merge two
/// columns into one.
fn page_columns(
    frames: &[BodyFrame],
    allocation: Allocation<'_>,
) -> Result<(Vec<(f32, f32)>, Vec<Option<usize>>), PdfImportError> {
    let mut order = allocation.collect(0..frames.len())?;
    allocation.sort_by(&mut order, |&a, &b| {
        let h = |i: usize| frames[i].bottom - frames[i].top;
        h(b).total_cmp(&h(a))
    })?;
    let mut columns: Vec<(f32, f32)> = Vec::new();
    let mut owner = allocation.filled(frames.len(), None)?;
    for i in order {
        let f = &frames[i];
        let hits = allocation.collect(
            (0..columns.len()).filter(|&c| f.left < columns[c].1 && columns[c].0 < f.right),
        )?;
        match hits.as_slice() {
            [] => {
                allocation.push(&mut columns, (f.left, f.right))?;
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
    let mut rank = allocation.collect(0..columns.len())?;
    allocation.sort_by(&mut rank, |&a, &b| columns[a].0.total_cmp(&columns[b].0))?;
    let mut position = allocation.filled(columns.len(), 0)?;
    for (to, &from) in rank.iter().enumerate() {
        position[from] = to;
    }
    let sorted = allocation.collect(rank.iter().map(|&c| columns[c]))?;
    Ok((
        sorted,
        allocation.collect(owner.into_iter().map(|c| c.map(|c| position[c])))?,
    ))
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

/// A retained text owner in the column's leading band must travel with its
/// first flow paragraph. Otherwise joining that paragraph to the previous
/// column/page discards the band when pagination selects a continuation origin.
fn take_positioned_prefix(
    anchor: &mut Paragraph,
    committed: &std::collections::BTreeSet<u32>,
    column: (i32, i32),
    body_top: i32,
    flow_top: i32,
    allocation: Allocation<'_>,
) -> Result<Option<Paragraph>, PdfImportError> {
    use crate::model::shape::{HorzRelTo, VertRelTo};
    let owns_prefix = |control: &Control| {
        let Control::Shape(shape) = control else {
            return false;
        };
        let ShapeObject::Rectangle(rect) = &**shape else {
            return false;
        };
        if rect.drawing.text_box.is_none()
            || committed.contains(&rect.drawing.inst_id)
            || rect.common.vert_rel_to != VertRelTo::Paper
            || rect.common.horz_rel_to != HorzRelTo::Paper
        {
            return false;
        }
        let left = i64::from(rect.common.horizontal_offset);
        let top = i64::from(rect.common.vertical_offset);
        let right = left + i64::from(rect.common.width);
        let bottom = top + i64::from(rect.common.height);
        left < i64::from(column.1)
            && right > i64::from(column.0)
            && top < i64::from(flow_top)
            && bottom > i64::from(body_top)
            && bottom <= i64::from(flow_top)
    };
    let count = anchor.controls.iter().filter(|c| owns_prefix(c)).count();
    if count == 0 {
        return Ok(None);
    }
    let mut prefix = Paragraph::new_empty();
    allocation.reserve(&mut prefix.controls, count)?;
    let aligned_records = anchor.ctrl_data_records.len() == anchor.controls.len();
    if aligned_records {
        allocation.reserve(&mut prefix.ctrl_data_records, count)?;
    }
    for index in (0..anchor.controls.len()).rev() {
        if owns_prefix(&anchor.controls[index]) {
            prefix.controls.push(anchor.controls.remove(index));
            if aligned_records {
                prefix
                    .ctrl_data_records
                    .push(anchor.ctrl_data_records.remove(index));
            }
        }
    }
    prefix.controls.reverse();
    prefix.ctrl_data_records.reverse();
    let slots = 8 * count as u32;
    prefix.char_count += slots;
    anchor.char_count -= slots;
    Ok(Some(prefix))
}

/// Move every control of `anchor` to the front of `paragraph`. Each extended
/// control takes eight stream units ahead of the first character, so text,
/// character shape and line start positions shift with it.
fn attach_leading_controls(
    paragraph: &mut Paragraph,
    mut anchor: Paragraph,
    allocation: Allocation<'_>,
) -> Result<(), PdfImportError> {
    let count = anchor.controls.len();
    if count == 0 {
        return Ok(());
    }
    let mut records = std::mem::take(&mut anchor.ctrl_data_records);
    let extra = count.saturating_sub(records.len());
    allocation.reserve(&mut records, extra)?;
    records.resize(count, None);
    let mut own = std::mem::take(&mut paragraph.ctrl_data_records);
    let extra = paragraph.controls.len().saturating_sub(own.len());
    allocation.reserve(&mut own, extra)?;
    own.resize(paragraph.controls.len(), None);
    allocation.extend(&mut records, own)?;
    let mut controls = std::mem::take(&mut anchor.controls);
    allocation.reserve(&mut controls, paragraph.controls.len())?;
    controls.append(&mut paragraph.controls);
    paragraph.controls = controls;
    paragraph.ctrl_data_records = records;
    paragraph.char_count += 8 * count as u32;
    paragraph.reserve_leading_extended_control_slots(count);
    Ok(())
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
fn join(
    previous: &mut Paragraph,
    next: &Paragraph,
    allocation: Allocation<'_>,
) -> Result<(), PdfImportError> {
    use super::text_analysis::{line_join, LineJoin};
    let last_line = previous.text.lines().last().unwrap_or("");
    let mut rows = std::mem::take(&mut previous.line_segs);
    if matches!(
        line_join(last_line, &next.text),
        LineJoin::InferredWordSpace
    ) {
        let end = previous.char_offsets.last().map_or(0, |&o| {
            o + previous
                .text
                .chars()
                .last()
                .map_or(1, |c| c.len_utf16() as u32)
        });
        allocation.push_char(&mut previous.text, ' ')?;
        allocation.push(&mut previous.char_offsets, end)?;
    }
    let at = previous.merge_from_with_storage(next, &allocation)?;
    let shift = previous.char_offsets.get(at).copied().unwrap_or(0);
    allocation.extend(
        &mut rows,
        next.line_segs.iter().map(|seg| {
            let mut seg = seg.clone();
            seg.text_start += shift;
            seg
        }),
    )?;
    previous.line_segs = rows;
    Ok(())
}

/// Body rows take the paragraph indent from the row tag, not from their
/// start position: a positive indent marks the first row, a negative
/// (hanging) indent the rows after it. Frame rows stored the indent in
/// their start instead, and joined rows came from separate paragraphs.
fn mark_indented_rows(doc: &Document, paragraphs: &mut [Paragraph]) {
    use crate::model::paragraph::LineSeg;
    for p in paragraphs.iter_mut() {
        let Some(shape) = doc.doc_info.para_shapes.get(p.para_shape_id as usize) else {
            continue;
        };
        let indent = shape.indent;
        for (row, seg) in p.line_segs.iter_mut().enumerate() {
            let indented = (indent > 0 && row == 0) || (indent < 0 && row > 0);
            if indented {
                seg.tag |= LineSeg::TAG_INDENTATION;
            } else {
                seg.tag &= !LineSeg::TAG_INDENTATION;
            }
        }
    }
}

/// Lay the pages' paragraphs into one flow. Paragraphs cut by a column or
/// page boundary are joined. Each page's positioned content rides on the
/// first paragraph that starts on that page, or on its own anchor line.
fn assemble(
    doc: &mut Document,
    out: &mut Vec<Paragraph>,
    pages: Vec<(Paragraph, Vec<Paragraph>, bool)>,
    allocation: Allocation<'_>,
) -> Result<usize, PdfImportError> {
    let mut joined = 0;
    for (page_no, (mut anchor, mut body, keep_anchor)) in pages.into_iter().enumerate() {
        if page_no == 0 {
            // The section's first paragraph carries only the section and
            // column definitions on one minimal line, as stored sections do.
            // HWPX stores the first paragraph's row starts on an axis without
            // those two definitions, so text rows never start in it.
            let lead = anchor
                .controls
                .iter()
                .take_while(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))
                .count();
            let mut rest = Paragraph::new_empty();
            allocation.payload::<Control>(anchor.controls.len() - lead)?;
            rest.controls = anchor.controls.split_off(lead);
            if anchor.ctrl_data_records.len() > lead {
                allocation.bytes(
                    (anchor.ctrl_data_records.len() - lead)
                        * std::mem::size_of_val(&anchor.ctrl_data_records[0]),
                )?;
                rest.ctrl_data_records = anchor.ctrl_data_records.split_off(lead);
            }
            anchor.char_count -= 8 * rest.controls.len() as u32;
            rest.para_shape_id = anchor.para_shape_id;
            rest.char_shapes = allocation.clone_record(&anchor.char_shapes)?;
            rest.line_segs = allocation.clone_record(&anchor.line_segs)?;
            rest.char_count = 1 + 8 * rest.controls.len() as u32;
            allocation.push(out, anchor)?;
            anchor = rest;
        }
        if !keep_anchor && page_no > 0 {
            if let Some(first) = body.first_mut() {
                first.column_type = ColumnBreakType::Page;
            }
        }
        let mut cut = allocation.filled(body.len(), false)?;
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
                attach_leading_controls(&mut body[h], std::mem::take(&mut anchor), allocation)?;
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
                allocation.push(out, anchor)?;
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
                    join(previous, &paragraph, allocation)?;
                    joined += 1;
                }
                _ => allocation.push(out, paragraph)?,
            }
        }
    }
    Ok(joined)
}

fn column_def(
    boxes: &[(i32, i32)],
    allocation: Allocation<'_>,
) -> Result<ColumnDef, PdfImportError> {
    const TOTAL: i64 = 32768;
    let body = i64::from((boxes[boxes.len() - 1].1 - boxes[0].0).max(1));
    let mut parts = Vec::new();
    for (i, &(left, right)) in boxes.iter().enumerate() {
        allocation.push(&mut parts, i64::from(right - left))?;
        allocation.push(
            &mut parts,
            boxes.get(i + 1).map_or(0, |next| i64::from(next.0 - right)),
        )?;
    }
    // Cumulative rounding keeps the proportions summing to exactly 32768.
    let mut edges = allocation.vec(parts.len())?;
    let mut acc = 0;
    for part in &parts {
        acc += part;
        allocation.push(&mut edges, acc * TOTAL / body)?;
    }
    let mut previous = 0;
    let scaled = allocation.collect(edges.iter().map(|&edge| {
        let value = (edge - previous).clamp(0, i64::from(i16::MAX)) as i16;
        previous = edge;
        value
    }))?;
    let widths = allocation.collect(scaled.iter().step_by(2).copied())?;
    let gaps = allocation.collect(scaled.iter().skip(1).step_by(2).copied())?;
    let spacing = boxes.windows(2).next().map_or(0, |w| {
        (w[1].0 - w[0].1).clamp(0, i32::from(i16::MAX)) as i16
    });
    Ok(ColumnDef {
        column_type: ColumnType::Normal,
        column_count: boxes.len() as u16,
        direction: ColumnDirection::LeftToRight,
        same_width: false,
        spacing,
        widths,
        gaps,
        proportional_widths: true,
        ..Default::default()
    })
}

/// Replace each page's positioned body frames with section text flow.
pub(super) fn reflow(
    doc: &mut Document,
    pages: &[PageFrames],
    allocation: Allocation<'_>,
) -> Result<Value, PdfImportError> {
    if pages.is_empty() {
        return Ok(request_json!(allocation, {"layout":"editable","sections":0})?);
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
            allocation.push(runs.last_mut().unwrap(), page)?;
        } else {
            let run = allocation.collect([page])?;
            allocation.push(&mut runs, run)?;
        }
    }
    let mut old = allocation.collect(doc.sections.drain(base..))?;
    let mut report = Vec::new();
    let mut anchor_shape = None;
    for run in runs {
        let first = run[0].section - base;
        let pd = allocation.clone_record(&old[first].section_def.page_def)?;
        let width = if pd.landscape { pd.height } else { pd.width } as i32;
        let height = if pd.landscape { pd.width } else { pd.height } as i32;
        let layouts =
            allocation.try_collect(run.iter().map(|p| page_columns(&p.frames, allocation)))?;
        // The section's column grid is the most common page grid.
        let mut counts = std::collections::BTreeMap::new();
        for (columns, _) in &layouts {
            if !columns.is_empty() {
                let value = counts.get(&columns.len()).copied().unwrap_or(0usize) + 1;
                allocation.insert(&mut counts, columns.len(), value)?;
            }
        }
        let Some((&count, _)) = counts.iter().max_by_key(|(n, c)| (**c, **n)) else {
            // Nothing recognized: keep the positioned pages as they were.
            for page in &run {
                let section = allocation.clone_section(&old[page.section - base])?;
                allocation.push(&mut doc.sections, section)?;
            }
            continue;
        };
        let grid =
            allocation.collect(layouts.iter().map(|(c, _)| c).filter(|c| c.len() == count))?;
        let median = |values: Vec<f32>| -> Result<f32, PdfImportError> {
            let mut v = values;
            allocation.sort_by(&mut v, f32::total_cmp)?;
            Ok(v[v.len() / 2])
        };
        let lefts = allocation.try_collect((0..count).map(|c| -> Result<i32, PdfImportError> {
            Ok(hu(median(
                allocation.collect(grid.iter().map(|g| g[c].0))?,
            )?))
        }))?;
        let rights =
            allocation.try_collect((0..count).map(|c| -> Result<i32, PdfImportError> {
                Ok(hu(median(
                    allocation.collect(grid.iter().map(|g| g[c].1))?,
                )?))
            }))?;
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
        let boxes =
            allocation.collect((0..count).map(|c| (lefts[c].max(0), rights[c].min(width))))?;
        let margin_left = boxes[0].0;
        let margin_right = (width - boxes[count - 1].1).max(0);
        let column_left = |c: usize| boxes[c].0;
        let column_width_of = |c: usize| boxes[c].1 - boxes[c].0;
        // A frame joins the flow when its column maps onto one grid column.
        let mut placed =
            allocation.try_collect(run.iter().zip(&layouts).map(|(page, (columns, owner))| {
                let grid_of = |c: usize| {
                    let center = hu((columns[c].0 + columns[c].1) / 2.);
                    (0..count).min_by_key(|&g| ((boxes[g].0 + boxes[g].1) / 2 - center).abs())
                };
                let mapped = allocation.collect((0..columns.len()).map(grid_of))?;
                allocation.collect(page.frames.iter().zip(owner).map(|(frame, o)| {
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
                }))
            }))?;
        // A short frame that recurs at one height on half the pages or more
        // is a running header or footer (page number, booklet marks). It
        // keeps its page position instead of opening a column.
        let band = |f: &BodyFrame| (hu(f.top) / 200, hu(f.bottom) / 200);
        let mut recurring = std::collections::BTreeMap::new();
        for page in &run {
            let bands = allocation.set_from(
                page.frames
                    .iter()
                    .filter(|f| f.bottom - f.top < 30.0)
                    .map(band),
            )?;
            for b in bands {
                let value = recurring.get(&b).copied().unwrap_or(0usize) + 1;
                allocation.insert(&mut recurring, b, value)?;
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
        let page_tops = allocation.collect(run.iter().zip(&placed).filter_map(|(p, owners)| {
            p.frames
                .iter()
                .zip(owners)
                .filter(|(_, g)| g.is_some())
                .map(|(f, _)| f.top)
                .reduce(f32::min)
        }))?;
        let body_top = median(page_tops)?;
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
        let mut flow_bottoms = allocation
            .try_collect(run.iter().map(|page| {
                allocation.collect(page.frames.iter().map(|frame| hu(frame.bottom)))
            }))?;
        let mut tail_bands = Vec::new();
        // The ordinary reflow witness also infers a tail after its last row.
        // Reconcile that tail with the next observed frame, preserving both
        // accepted line boxes and the confirmed body paragraph owners.
        for (page_index, (page, owners)) in run.iter().zip(&mut placed).enumerate() {
            for column in 0..count {
                let mut order = allocation
                    .collect((0..page.frames.len()).filter(|&i| owners[i] == Some(column)))?;
                allocation.sort_by(&mut order, |&a, &b| {
                    page.frames[a].top.total_cmp(&page.frames[b].top)
                })?;
                for pair in order.windows(2) {
                    let index = pair[0];
                    let next_top = hu(page.frames[pair[1]].top);
                    let bottom = &mut flow_bottoms[page_index][index];
                    if next_top >= *bottom {
                        continue;
                    }
                    let last = old[page.section - base].paragraphs[0]
                        .controls
                        .iter_mut()
                        .find_map(|control| match control {
                            Control::Shape(shape) => match &mut **shape {
                                ShapeObject::Rectangle(rect)
                                    if rect.drawing.inst_id == page.frames[index].inst_id =>
                                {
                                    rect.drawing
                                        .text_box
                                        .as_mut()?
                                        .paragraphs
                                        .last_mut()?
                                        .line_segs
                                        .last_mut()
                                }
                                _ => None,
                            },
                            _ => None,
                        });
                    if let Some(last) = last {
                        let content_end = i64::from(*bottom) - i64::from(last.line_spacing);
                        // A paragraph's inferred interline tail is not
                        // observed occupied content. Grant only the available
                        // source band, without cutting any accepted line box.
                        if last.line_spacing > 0 && i64::from(next_top) >= content_end {
                            let before = last.line_spacing;
                            last.line_spacing = (i64::from(next_top) - content_end) as i32;
                            *bottom = next_top;
                            allocation.push(&mut tail_bands, request_json!(allocation, {"sourceSection":page.section,"instanceId":page.frames[index].inst_id,"contentEndHu":content_end,"sourceBandEndHu":next_top,"inferredTailHu":before,"acceptedTailHu":last.line_spacing})?)?;
                        }
                    }
                }
            }
        }
        let mut section = allocation.clone_section(&old[first])?;
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
        // Anchor lines hold no text. A 1pt face on a fixed 1pt line keeps
        // the height they take in the column known: `ANCHOR_HEIGHT`.
        let (anchor_shape_id, anchor_char_id) = if let Some(ids) = anchor_shape {
            ids
        } else {
            let mut shape = allocation.clone_record(&doc.doc_info.para_shapes[0])?;
            shape.raw_data = None;
            shape.line_spacing_type = LineSpacingType::Fixed;
            shape.line_spacing = 2 * ANCHOR_HEIGHT;
            shape.spacing_before = 0;
            shape.spacing_after = 0;
            allocation.push(&mut doc.doc_info.para_shapes, shape)?;
            let mut face = allocation.clone_record(&doc.doc_info.char_shapes[0])?;
            face.raw_data = None;
            face.base_size = ANCHOR_HEIGHT;
            allocation.push(&mut doc.doc_info.char_shapes, face)?;
            let ids = (
                (doc.doc_info.para_shapes.len() - 1) as u16,
                (doc.doc_info.char_shapes.len() - 1) as u32,
            );
            anchor_shape = Some(ids);
            ids
        };
        let def = column_def(&boxes, allocation)?;
        let mut pages_report = Vec::new();
        let mut pages_flow = Vec::new();
        for (page_no, page) in run.iter().enumerate() {
            let committed = allocation.set_from(
                page.frames
                    .iter()
                    .zip(&placed[page_no])
                    .filter(|(_, owner)| owner.is_some())
                    .map(|(frame, _)| frame.inst_id),
            )?;
            let mut anchor = std::mem::take(&mut old[page.section - base].paragraphs[0]);
            if page_no == 0 {
                // The section opens with its definition and column grid as the
                // first two extended controls, as stored HWP sections do. Both
                // take stream slots ahead of the first character; savers write
                // the section definition there even when the model omits it.
                let before = anchor.controls.len();
                anchor
                    .controls
                    .retain(|c| !matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)));
                anchor.char_count -= 8 * (before - anchor.controls.len()) as u32;
                let section_def = allocation.clone_section_def(&section.section_def)?;
                let section_def = allocation.boxed(section_def)?;
                let column = allocation.clone_record(&def)?;
                allocation.reserve(&mut anchor.controls, 2)?;
                anchor.controls.insert(0, Control::SectionDef(section_def));
                anchor.controls.insert(1, Control::ColumnDef(column));
                anchor.char_count += 16;
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
            anchor.char_shapes = allocation.collect([crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: anchor_char_id,
            }])?;
            for seg in &mut anchor.line_segs {
                seg.vertical_pos = 0;
                seg.line_height = ANCHOR_HEIGHT;
                seg.text_height = ANCHOR_HEIGHT;
                seg.baseline_distance = ANCHOR_HEIGHT * 85 / 100;
                seg.line_spacing = 0;
                seg.column_start = 0;
                seg.segment_width = column_width_of(0);
            }
            let mut body = Vec::new();
            let mut moved = 0;
            let mut current_column = 0;
            for g in 0..count {
                let mut members = allocation
                    .collect((0..page.frames.len()).filter(|&i| placed[page_no][i] == Some(g)))?;
                allocation.sort_by(&mut members, |&a, &b| {
                    page.frames[a].top.total_cmp(&page.frames[b].top)
                })?;
                // The anchor line occupies the top of the first column.
                // Anchor lines (the section opener, a page anchor) occupy the
                // top of the first column.
                let anchors = i32::from(page_no == 0) + i32::from(keep_anchor);
                let mut previous_bottom =
                    margin_top + if g == 0 { ANCHOR_HEIGHT * anchors } else { 0 };
                let col_left = column_left(g);
                let mut first_in_column = true;
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
                    if first_in_column {
                        if let Some(first) = paragraphs.first_mut() {
                            if let Some(prefix) = take_positioned_prefix(
                                &mut anchor,
                                &committed,
                                boxes[g],
                                margin_top,
                                top,
                                allocation,
                            )? {
                                attach_leading_controls(first, prefix, allocation)?;
                            }
                            first_in_column = false;
                        }
                    }
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
                            allocation.push(&mut body, empty)?;
                        }
                        paragraphs[0].column_type = ColumnBreakType::Column;
                        current_column = g;
                    }
                    previous_bottom = flow_bottoms[page_no][i];
                    moved += paragraphs.len();
                    allocation.extend(&mut body, paragraphs)?;
                }
            }
            let positioned =
                page.frames.len() - placed[page_no].iter().filter(|g| g.is_some()).count();
            allocation.push(&mut pages_report, request_json!(allocation, {"sourceSection":page.section,"columns":count,"bodyParagraphs":moved,"positionedFrames":positioned})?)?;
            allocation.push(&mut pages_flow, (anchor, body, keep_anchor))?;
        }
        let joined = assemble(doc, &mut section.paragraphs, pages_flow, allocation)?;
        mark_indented_rows(doc, &mut section.paragraphs);
        allocation.push(&mut report, request_json!(allocation, {"pages":pages_report,"columns":count,"columnGap":gap,"joinedParagraphs":joined,"margins":[margin_left,margin_top,margin_right,margin_bottom],"observedTailBands":tail_bands})?)?;
        allocation.push(&mut doc.sections, section)?;
    }
    Ok(request_json!(allocation, {"layout":"editable","sections":report})?)
}
