//! Project HWP5 change-tracking visibility before parsing the layout model.
//!
//! ViewText stores review history, including text and control slots hidden by
//! DocInfo TRACK_CHANGE records. A hidden paragraph terminator joins paragraphs;
//! an ordinary empty paragraph still owns its line. The original stream remains
//! the section's round-trip source; only the input to the visible IR is projected.

use super::{body_text::BodyTextError, record::Record, tags};
use crate::model::document::DocInfo;

const INSERT: u32 = 0x10;
const DELETE: u32 = 0x11;

fn invalid(message: &str) -> BodyTextError {
    BodyTextError::ChangeTrackingStructure(message.into())
}

fn word(data: &[u8], offset: usize) -> Option<u32> {
    data.get(offset..offset + 4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
}

/// Record IDs are one-based in the ordered DocInfo TRACK_CHANGE table. The
/// insertion/deletion payload is kind(4), date(10), author(2), reserved(2),
/// properties(4), hide(4). Formatting changes have an additional shape ID.
/// This interpretation was cross-checked against independent Hancom HWPX
/// trackChange IDs/types/hide values, including nested insert/delete ranges.
struct Visibility(Vec<Option<(u32, bool)>>);

impl Visibility {
    fn from_doc_info(info: &DocInfo) -> Result<Self, BodyTextError> {
        let mut changes = Vec::new();
        for record in &info.extra_records {
            if record.tag_id != tags::HWPTAG_TRACK_CHANGE {
                continue;
            }
            let kind = word(&record.data, 0);
            if matches!(kind, Some(INSERT | DELETE)) {
                let hidden = match word(&record.data, 22) {
                    Some(0) => false,
                    Some(1) => true,
                    _ => return Err(invalid("invalid text change visibility")),
                };
                changes.push(Some((kind.unwrap(), hidden)));
            } else {
                changes.push(None);
            }
        }
        Ok(Self(changes))
    }

    fn hidden(&self, tag: u32) -> bool {
        let kind = tag >> 24;
        let id = tag & 0x00ff_ffff;
        id.checked_sub(1)
            .and_then(|index| self.0.get(index as usize))
            .copied()
            .flatten()
            .is_some_and(|(record_kind, hidden)| record_kind == kind && hidden)
    }
}

struct Node {
    record: Record,
    children: Vec<Node>,
}

fn read_nodes(
    records: &mut std::iter::Peekable<std::vec::IntoIter<Record>>,
    parent: Option<u16>,
    depth: usize,
) -> Result<Vec<Node>, BodyTextError> {
    // A paragraph nesting step includes its control/list wrappers as well as
    // the paragraph itself. Bound record recursion without changing that limit.
    if depth > super::body_text::MAX_HWP5_SECTION_DEPTH as usize * 4 {
        return Err(invalid("change-tracking record nesting limit"));
    }
    let mut nodes = Vec::new();
    while records
        .peek()
        .is_some_and(|r| parent.is_none_or(|level| r.level > level))
    {
        let record = records.next().unwrap();
        let children = read_nodes(records, Some(record.level), depth + 1)?;
        nodes.push(Node { record, children });
    }
    Ok(nodes)
}

fn text_child(node: &Node) -> Option<usize> {
    node.children
        .iter()
        .position(|child| child.record.tag_id == tags::HWPTAG_PARA_TEXT)
}

fn update_header(node: &mut Node) -> Result<(), BodyTextError> {
    if node.record.data.len() < 18 {
        return Err(invalid("short tracked paragraph header"));
    }
    let units = text_child(node)
        .map(|i| node.children[i].record.data.len() / 2)
        .unwrap_or(1);
    let last = word(&node.record.data, 0).unwrap() & 0x8000_0000;
    node.record.data[..4].copy_from_slice(&(last | units as u32).to_le_bytes());
    for (offset, tag, width) in [
        (12, tags::HWPTAG_PARA_CHAR_SHAPE, 8),
        (14, tags::HWPTAG_PARA_RANGE_TAG, 12),
        (16, tags::HWPTAG_PARA_LINE_SEG, 36),
    ] {
        let count: usize = node
            .children
            .iter()
            .filter(|c| c.record.tag_id == tag)
            .map(|c| c.record.data.len() / width)
            .sum();
        let count =
            u16::try_from(count).map_err(|_| invalid("tracked paragraph item count overflow"))?;
        node.record.data[offset..offset + 2].copy_from_slice(&count.to_le_bytes());
    }
    Ok(())
}

fn extended_slot(unit: u16) -> bool {
    matches!(unit, 1..=9 | 11..=12 | 14..=23)
}

fn owned_slot(unit: u16) -> bool {
    matches!(unit, 1..=3 | 11..=12 | 14..=18 | 21..=23)
}

fn retain_visible_control_bits(node: &mut Node) {
    let mut present = 0u32;
    if let Some(index) = text_child(node) {
        let bytes = &node.children[index].record.data;
        let mut offset = 0;
        while offset + 2 <= bytes.len() {
            let unit = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            if unit < 32 {
                present |= 1u32 << unit;
            }
            offset += if extended_slot(unit) { 16 } else { 2 };
        }
    }
    // A hidden control must not leave its source presence bit in visible IR.
    // Preserve the original mask convention for every surviving control.
    if let Some(mask) = word(&node.record.data, 4) {
        node.record.data[4..8].copy_from_slice(&(mask & present).to_le_bytes());
    }
}

/// Carry a hidden terminator and wholly hidden content separately. A deleted
/// paragraph can still supply the paragraph format when its end joins the next.
fn project_paragraph(
    mut node: Node,
    visibility: &Visibility,
) -> Result<(Node, bool, bool), BodyTextError> {
    let text_index = text_child(&node);
    let units = match text_index {
        Some(index) => {
            let bytes = &node.children[index].record.data;
            if !bytes.len().is_multiple_of(2) {
                return Err(invalid("odd tracked UTF-16 byte count"));
            }
            bytes
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect::<Vec<_>>()
        }
        // Empty paragraphs can have an implicit CR, with no PARA_TEXT record.
        None if word(&node.record.data, 0).map(|n| n & 0x7fff_ffff) == Some(1) => vec![13],
        None => return Ok((node, false, false)),
    };
    let mut hidden = vec![false; units.len()];
    for child in &node.children {
        if child.record.tag_id != tags::HWPTAG_PARA_RANGE_TAG {
            continue;
        }
        for range in child.record.data.chunks_exact(12) {
            if !visibility.hidden(word(range, 8).unwrap()) {
                continue;
            }
            let start = word(range, 0).unwrap() as usize;
            let end = word(range, 4).unwrap() as usize;
            let span = hidden
                .get_mut(start..end)
                .ok_or_else(|| invalid("hidden range outside paragraph source axis"))?;
            span.fill(true);
        }
    }
    let join = units.last() == Some(&13) && hidden.last() == Some(&true);
    let source_changed = hidden.iter().any(|&hide| hide);
    let mut removed_controls = Vec::new();
    let mut position = 0;
    while position < units.len() {
        let unit = units[position];
        let width = if extended_slot(unit) {
            8
        } else if (0xd800..=0xdbff).contains(&unit)
            && units
                .get(position + 1)
                .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
        {
            2
        } else {
            1
        };
        let flags = hidden
            .get(position..position + width)
            .ok_or_else(|| invalid("incomplete tracked source token"))?;
        if flags.iter().any(|&h| h != flags[0]) {
            return Err(invalid("hidden range splits a source token"));
        }
        if owned_slot(unit) {
            removed_controls.push(flags[0]);
        }
        position += width;
    }
    let mut mapping = Vec::with_capacity(units.len() + 1);
    mapping.push(0u32);
    let mut visible = Vec::new();
    for (&unit, &hide) in units.iter().zip(&hidden) {
        if !hide {
            visible.push(unit);
        }
        mapping.push(visible.len() as u32);
    }
    let fully_hidden = visible.is_empty() && join;
    if join {
        visible.push(13);
    }
    let mut control = 0;
    let mut children = Vec::new();
    for mut child in node.children {
        match child.record.tag_id {
            tags::HWPTAG_CTRL_HEADER => {
                let remove = removed_controls.get(control).copied().unwrap_or(false);
                control += 1;
                if remove {
                    continue;
                }
            }
            tags::HWPTAG_PARA_TEXT => {
                child.record.data = visible.iter().flat_map(|u| u.to_le_bytes()).collect();
            }
            // Only changed source axes invalidate saved rows. ViewText can
            // contain valid final-mode rows in an unchanged visible paragraph;
            // discarding those rows would lose the author's saved line breaks.
            tags::HWPTAG_PARA_LINE_SEG if source_changed => continue,
            tags::HWPTAG_PARA_CHAR_SHAPE => {
                let mut shapes: Vec<(u32, u32)> = Vec::new();
                for pair in child.record.data.chunks_exact(8) {
                    let old = word(pair, 0).unwrap() as usize;
                    let Some(&start) = mapping.get(old) else {
                        return Err(invalid("shape boundary outside tracked source axis"));
                    };
                    let shape = word(pair, 4).unwrap();
                    if let Some(last) = shapes.last_mut().filter(|last| last.0 == start) {
                        last.1 = shape;
                    } else {
                        shapes.push((start, shape));
                    }
                }
                child.record.data = shapes
                    .iter()
                    .flat_map(|(start, shape)| {
                        start.to_le_bytes().into_iter().chain(shape.to_le_bytes())
                    })
                    .collect();
            }
            tags::HWPTAG_PARA_RANGE_TAG => {
                let mut ranges = Vec::new();
                for range in child.record.data.chunks_exact(12) {
                    let tag = word(range, 8).unwrap();
                    let start = word(range, 0).unwrap() as usize;
                    let end = word(range, 4).unwrap() as usize;
                    // Projection consumes tracking ranges; the original raw
                    // source still retains them for an unedited round trip.
                    if matches!(tag >> 24, 0x10..=0x13) {
                        continue;
                    }
                    let (Some(&start), Some(&end)) = (mapping.get(start), mapping.get(end)) else {
                        return Err(invalid("range outside tracked source axis"));
                    };
                    if start < end {
                        ranges.extend(start.to_le_bytes());
                        ranges.extend(end.to_le_bytes());
                        ranges.extend(tag.to_le_bytes());
                    }
                }
                child.record.data = ranges;
            }
            _ => {}
        }
        children.push(child);
    }
    node.children = children;
    if source_changed {
        retain_visible_control_bits(&mut node);
    }
    let fully_hidden = fully_hidden
        && !node
            .children
            .iter()
            .any(|c| c.record.tag_id == tags::HWPTAG_CTRL_HEADER);
    update_header(&mut node)?;
    Ok((node, join, fully_hidden))
}

fn merge_paragraphs(first: &mut Node, mut next: Node) -> Result<(), BodyTextError> {
    // Joining across a hidden paragraph end changes the paragraph partition.
    first
        .children
        .retain(|c| c.record.tag_id != tags::HWPTAG_PARA_LINE_SEG);
    next.children
        .retain(|c| c.record.tag_id != tags::HWPTAG_PARA_LINE_SEG);
    fn ensure_text(node: &mut Node) -> usize {
        if let Some(index) = text_child(node) {
            return index;
        }
        node.children.insert(
            0,
            Node {
                record: Record {
                    tag_id: tags::HWPTAG_PARA_TEXT,
                    level: node.record.level + 1,
                    size: 2,
                    data: vec![13, 0],
                },
                children: Vec::new(),
            },
        );
        0
    }
    let first_text = ensure_text(first);
    let next_text = ensure_text(&mut next);
    let offset = first.children[first_text]
        .record
        .data
        .len()
        .checked_sub(2)
        .ok_or_else(|| invalid("missing paragraph terminator during join"))?;
    first.children[first_text].record.data.truncate(offset);
    first.children[first_text]
        .record
        .data
        .extend_from_slice(&next.children[next_text].record.data);
    let offset = (offset / 2) as u32;
    for mut child in next.children {
        match child.record.tag_id {
            tags::HWPTAG_PARA_TEXT => continue,
            tags::HWPTAG_PARA_CHAR_SHAPE => {
                for shape in child.record.data.chunks_exact_mut(8) {
                    let start = word(shape, 0).unwrap() + offset;
                    shape[..4].copy_from_slice(&start.to_le_bytes());
                }
                if let Some(target) = first
                    .children
                    .iter_mut()
                    .find(|c| c.record.tag_id == tags::HWPTAG_PARA_CHAR_SHAPE)
                {
                    target.record.data.extend(child.record.data);
                    continue;
                }
            }
            tags::HWPTAG_PARA_RANGE_TAG => {
                for range in child.record.data.chunks_exact_mut(12) {
                    for pos in [0, 4] {
                        let start = word(range, pos).unwrap() + offset;
                        range[pos..pos + 4].copy_from_slice(&start.to_le_bytes());
                    }
                }
                if let Some(target) = first
                    .children
                    .iter_mut()
                    .find(|c| c.record.tag_id == tags::HWPTAG_PARA_RANGE_TAG)
                {
                    target.record.data.extend(child.record.data);
                    continue;
                }
            }
            _ => {}
        }
        first.children.push(child);
    }
    // The final paragraph owns the list-end flag after the hidden boundary.
    first.record.data[3] = (first.record.data[3] & 0x7f) | (next.record.data[3] & 0x80);
    let mask = word(&first.record.data, 4).unwrap() | word(&next.record.data, 4).unwrap();
    first.record.data[4..8].copy_from_slice(&mask.to_le_bytes());
    retain_visible_control_bits(first);
    update_header(first)
}

fn project_nodes(nodes: Vec<Node>, visibility: &Visibility) -> Result<Vec<Node>, BodyTextError> {
    let mut output: Vec<Node> = Vec::new();
    let mut join = false;
    let mut pending: Option<Node> = None;
    for mut node in nodes {
        node.children = project_nodes(node.children, visibility)?;
        if node.record.tag_id != tags::HWPTAG_PARA_HEADER {
            // A separate list/control is an ownership boundary for paragraph joins.
            join = false;
            pending = None;
            if node.record.tag_id == tags::HWPTAG_LIST_HEADER && node.record.data.len() >= 2 {
                let count = node
                    .children
                    .iter()
                    .filter(|c| c.record.tag_id == tags::HWPTAG_PARA_HEADER)
                    .count();
                node.record.data[..2].copy_from_slice(
                    &u16::try_from(count)
                        .map_err(|_| invalid("tracked list count overflow"))?
                        .to_le_bytes(),
                );
            }
            output.push(node);
            continue;
        }
        let (mut node, hidden_end, fully_hidden) = project_paragraph(node, visibility)?;
        if join
            && output
                .last()
                .is_some_and(|n| n.record.tag_id == tags::HWPTAG_PARA_HEADER)
        {
            merge_paragraphs(output.last_mut().unwrap(), node)?;
        } else if fully_hidden {
            // No visible line is emitted. Retain the first paragraph's format
            // until a visible sibling supplies content across the deleted CR.
            if pending.is_none() {
                pending = Some(node);
            }
            continue;
        } else {
            if let Some(mut first) = pending.take() {
                merge_paragraphs(&mut first, node)?;
                node = first;
            }
            output.push(node);
        }
        join = hidden_end;
    }
    Ok(output)
}

fn write_nodes(nodes: &[Node], bytes: &mut Vec<u8>) {
    for node in nodes {
        let record = &node.record;
        let size = record.data.len() as u32;
        bytes.extend(
            (record.tag_id as u32 | (record.level as u32) << 10 | size.min(0xfff) << 20)
                .to_le_bytes(),
        );
        if size >= 0xfff {
            bytes.extend(size.to_le_bytes());
        }
        bytes.extend_from_slice(&record.data);
        write_nodes(&node.children, bytes);
    }
}

pub(crate) fn project(data: &[u8], info: &DocInfo) -> Result<Option<Vec<u8>>, BodyTextError> {
    let visibility = Visibility::from_doc_info(info)?;
    if !visibility.0.iter().flatten().any(|&(_, hidden)| hidden) {
        return Ok(None);
    }
    let records = Record::read_all(data).map_err(|e| BodyTextError::RecordError(e.to_string()))?;
    let uses_hidden_ranges = records
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_PARA_RANGE_TAG)
        .any(|r| {
            r.data
                .chunks_exact(12)
                .any(|tag| visibility.hidden(word(tag, 8).unwrap()))
        });
    if !uses_hidden_ranges {
        return Ok(None);
    }
    let nodes = read_nodes(&mut records.into_iter().peekable(), None, 0)?;
    let nodes = project_nodes(nodes, &visibility)?;
    let mut bytes = Vec::new();
    write_nodes(&nodes, &mut bytes);
    Ok(Some(bytes))
}
