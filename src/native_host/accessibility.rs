//! Bounded semantic input for a native accessibility bridge, not OS registration
//! or evidence of screen-reader support. Text follows structural IR order, not
//! inferred visual reading order. Keys are session-local revision + address;
//! they deliberately make no cross-revision identity promise.

use serde::{Deserialize, Serialize};

use super::{NativeDocumentSession, NativeHostError};
use crate::document_core::native_interaction::{
    NativeCaretRect, NativeCellPathEntry, NativeInteractionError, NativeSelectionRect,
    NativeTextPosition, NativeTextSelection, NATIVE_CELL_PATH_MAX_DEPTH,
};
use crate::model::{control::Control, paragraph::Paragraph};

pub const MAX_ACCESSIBILITY_NODES: usize = 1_000;
pub const MAX_ACCESSIBILITY_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_ACCESSIBILITY_DEPTH: usize = NATIVE_CELL_PATH_MAX_DEPTH;
pub const MAX_ACCESSIBILITY_WORK_ITEMS: usize = 10_000;
pub const MAX_ACCESSIBILITY_GEOMETRY_QUERIES: usize = 128;
pub const MAX_ACCESSIBILITY_SELECTION_RECTS: usize = 256;
/// Optional geometry uses existing core queries, not a layout CPU/time budget.
/// Keep each queried paragraph and selection bounded before calling them.
pub const MAX_ACCESSIBILITY_GEOMETRY_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_ACCESSIBILITY_SELECTION_SCALARS: u32 = 4096;
pub const MAX_ACCESSIBILITY_SELECTION_PARAGRAPHS: u32 = 64;

/// Half-open top-level body paragraph range, including its table-cell subtrees.
/// A page filter is intentionally absent: paragraphs can span multiple pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeAccessibilityRange {
    pub section: u32,
    pub start_paragraph: u32,
    pub end_paragraph: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NativeAccessibilityRequest {
    pub range: Option<NativeAccessibilityRange>,
    /// False performs no caret/selection geometry queries.
    pub geometry: bool,
    pub max_nodes: usize,
    pub max_text_bytes: usize,
    /// Zero requests body paragraphs only; skipped table subtrees are reported.
    pub max_depth: usize,
    pub max_geometry_queries: usize,
    pub max_selection_rects: usize,
}

impl Default for NativeAccessibilityRequest {
    fn default() -> Self {
        Self {
            range: None,
            geometry: false,
            max_nodes: MAX_ACCESSIBILITY_NODES,
            max_text_bytes: MAX_ACCESSIBILITY_TEXT_BYTES,
            max_depth: MAX_ACCESSIBILITY_DEPTH,
            max_geometry_queries: MAX_ACCESSIBILITY_GEOMETRY_QUERIES,
            max_selection_rects: MAX_ACCESSIBILITY_SELECTION_RECTS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeAccessibilityRole {
    BodyParagraph,
    TableCellParagraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeAccessibilityLimitation {
    NodeLimit,
    TextLimit,
    DepthLimit,
    TraversalLimit,
    GeometryLimit,
    GeometryUnavailable,
    SelectionGeometryUnsupported,
    InlineObjects,
    HeadersFooters,
    Notes,
    ShapesAndTextBoxes,
    Captions,
    MasterPages,
    OtherControlSemantics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeAccessibilityGeometryStatus {
    Complete,
    Unsupported,
    Unavailable,
    Limited,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeAccessibilityNodeGeometry {
    pub start_caret: Option<NativeCaretRect>,
    pub end_caret: Option<NativeCaretRect>,
    pub status: NativeAccessibilityGeometryStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeAccessibilityNode {
    pub key: String,
    pub role: NativeAccessibilityRole,
    pub start: NativeTextPosition,
    pub end: NativeTextPosition,
    /// A Unicode-boundary prefix. start/end describe exactly the returned text.
    pub text: String,
    pub text_complete: bool,
    pub geometry: Option<NativeAccessibilityNodeGeometry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeAccessibilitySelectionGeometry {
    pub focus_caret: Option<NativeCaretRect>,
    pub rectangles: Vec<NativeSelectionRect>,
    pub status: NativeAccessibilityGeometryStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeAccessibilitySnapshot {
    pub revision: u64,
    pub font_generation: u64,
    /// Always bodyAndTableCellText; this does not claim complete document semantics.
    pub scope: &'static str,
    pub range: Option<NativeAccessibilityRange>,
    pub nodes: Vec<NativeAccessibilityNode>,
    pub selection: Option<NativeTextSelection>,
    pub selection_geometry: Option<NativeAccessibilitySelectionGeometry>,
    /// Complete only within the requested scope and requested geometry contract.
    /// Encountered omitted controls/notes/headers make this false too.
    pub complete: bool,
    pub truncated: bool,
    pub limitations: Vec<NativeAccessibilityLimitation>,
}

impl NativeDocumentSession {
    /// Read body/table text with bounded nodes, UTF-8 bytes, descent and visited
    /// items. No document edits, selection changes, or revision changes occur.
    /// The optional selection must address bounded (<=1 MiB) paragraphs. Geometry
    /// is optional and its unsupported/unavailable/budget states remain explicit.
    /// Selection metadata is session-wide, even when a structural range is given.
    pub fn accessibility_snapshot(
        &self,
        request: &NativeAccessibilityRequest,
        selection: Option<&NativeTextSelection>,
    ) -> Result<NativeAccessibilitySnapshot, NativeHostError> {
        validate_request(request)?;
        let document = self.document().document();
        if let Some(range) = request.range {
            let section = document
                .sections
                .get(range.section as usize)
                .ok_or_else(|| invalid("accessibility section is out of range"))?;
            if range.start_paragraph > range.end_paragraph
                || range.end_paragraph as usize > section.paragraphs.len()
            {
                return Err(invalid("accessibility paragraph range is invalid"));
            }
        }
        if let Some(selection) = selection {
            for position in [&selection.anchor, &selection.focus] {
                let paragraph = resolve_paragraph(self, position)?;
                if paragraph.text.len() > MAX_ACCESSIBILITY_TEXT_BYTES {
                    return Err(NativeHostError::LimitExceeded(
                        "selection paragraph exceeds bounded validation limit".into(),
                    ));
                }
                self.document().validate_native_text_position(position)?;
            }
        }
        let mut walk = SnapshotWalk {
            session: self,
            request,
            snapshot: NativeAccessibilitySnapshot {
                revision: self.revision(),
                font_generation: self.font_generation(),
                scope: "bodyAndTableCellText",
                range: request.range,
                nodes: Vec::new(),
                selection: selection.cloned(),
                selection_geometry: None,
                complete: true,
                truncated: false,
                limitations: Vec::new(),
            },
            remaining_text: request.max_text_bytes,
            work: 0,
            geometry_queries: 0,
            stopped: false,
        };
        if request.geometry {
            if let Some(selection) = selection {
                walk.snapshot.selection_geometry = Some(walk.selection_geometry(selection)?);
            }
        }
        let sections = request.range.map_or(0..document.sections.len(), |range| {
            range.section as usize..range.section as usize + 1
        });
        for section_index in sections {
            if !walk.charge() {
                break;
            }
            let section = &document.sections[section_index];
            if !section.section_def.master_pages.is_empty() {
                walk.note(NativeAccessibilityLimitation::MasterPages);
            }
            let paragraphs = request.range.map_or(0..section.paragraphs.len(), |range| {
                range.start_paragraph as usize..range.end_paragraph as usize
            });
            for paragraph_index in paragraphs {
                if walk.stopped {
                    break;
                }
                walk.paragraph(
                    &section.paragraphs[paragraph_index],
                    NativeTextPosition {
                        section: address_index(section_index)?,
                        paragraph: address_index(paragraph_index)?,
                        scalar_offset: 0,
                        cell_path: Vec::new(),
                    },
                )?;
            }
        }
        Ok(walk.snapshot)
    }
}

struct SnapshotWalk<'a> {
    session: &'a NativeDocumentSession,
    request: &'a NativeAccessibilityRequest,
    snapshot: NativeAccessibilitySnapshot,
    remaining_text: usize,
    work: usize,
    geometry_queries: usize,
    stopped: bool,
}

impl SnapshotWalk<'_> {
    fn note(&mut self, limitation: NativeAccessibilityLimitation) {
        use NativeAccessibilityLimitation::*;
        self.snapshot.complete = false;
        if matches!(
            limitation,
            NodeLimit | TextLimit | DepthLimit | TraversalLimit | GeometryLimit
        ) {
            self.snapshot.truncated = true;
        }
        if !self.snapshot.limitations.contains(&limitation) {
            self.snapshot.limitations.push(limitation);
        }
    }

    fn charge(&mut self) -> bool {
        if self.stopped {
            return false;
        }
        if self.work == MAX_ACCESSIBILITY_WORK_ITEMS {
            self.note(NativeAccessibilityLimitation::TraversalLimit);
            self.stopped = true;
            return false;
        }
        self.work += 1;
        true
    }

    fn paragraph(
        &mut self,
        paragraph: &Paragraph,
        start: NativeTextPosition,
    ) -> Result<(), NativeHostError> {
        if !self.charge() {
            return Ok(());
        }
        if self.snapshot.nodes.len() == self.request.max_nodes {
            self.note(NativeAccessibilityLimitation::NodeLimit);
            self.stopped = true;
            return Ok(());
        }
        let mut byte_end = paragraph.text.len().min(self.remaining_text);
        while !paragraph.text.is_char_boundary(byte_end) {
            byte_end -= 1;
        }
        let text = paragraph.text[..byte_end].to_owned();
        self.remaining_text -= text.len();
        let mut end = start.clone();
        end.scalar_offset = text.chars().count() as u32; // <=1 MiB
        let text_complete = byte_end == paragraph.text.len();
        let geometry = self
            .request
            .geometry
            .then(|| self.node_geometry(paragraph, &start, &end));
        let role = if start.cell_path.is_empty() {
            NativeAccessibilityRole::BodyParagraph
        } else {
            NativeAccessibilityRole::TableCellParagraph
        };
        self.snapshot.nodes.push(NativeAccessibilityNode {
            key: node_key(self.session.revision(), &start),
            role,
            start: start.clone(),
            end,
            text,
            text_complete,
            geometry,
        });
        if !text_complete {
            self.note(NativeAccessibilityLimitation::TextLimit);
            self.stopped = true;
            return Ok(());
        }
        for (control_index, control) in paragraph.controls.iter().enumerate() {
            if !self.charge() {
                break;
            }
            if control.is_logical_inline() {
                self.note(NativeAccessibilityLimitation::InlineObjects);
            }
            match control {
                Control::Table(table) => {
                    if table.caption.is_some() {
                        self.note(NativeAccessibilityLimitation::Captions);
                    }
                    if start.cell_path.len() == self.request.max_depth && !table.cells.is_empty() {
                        self.note(NativeAccessibilityLimitation::DepthLimit);
                        continue;
                    }
                    for (cell_index, cell) in table.cells.iter().enumerate() {
                        if !self.charge() {
                            break;
                        }
                        for (paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
                            if self.stopped {
                                break;
                            }
                            let mut child = start.clone();
                            child.cell_path.push(NativeCellPathEntry {
                                control: address_index(control_index)?,
                                cell: address_index(cell_index)?,
                                paragraph: address_index(paragraph_index)?,
                            });
                            self.paragraph(paragraph, child)?;
                        }
                    }
                }
                Control::Header(_) | Control::Footer(_) => {
                    self.note(NativeAccessibilityLimitation::HeadersFooters)
                }
                Control::Footnote(_) | Control::Endnote(_) => {
                    self.note(NativeAccessibilityLimitation::Notes)
                }
                Control::Shape(_) => self.note(NativeAccessibilityLimitation::ShapesAndTextBoxes),
                Control::SectionDef(_) | Control::ColumnDef(_) => {}
                _ => self.note(NativeAccessibilityLimitation::OtherControlSemantics),
            }
        }
        Ok(())
    }

    fn caret(
        &mut self,
        position: &NativeTextPosition,
    ) -> (Option<NativeCaretRect>, NativeAccessibilityGeometryStatus) {
        if self.geometry_queries == self.request.max_geometry_queries {
            self.note(NativeAccessibilityLimitation::GeometryLimit);
            return (None, NativeAccessibilityGeometryStatus::Limited);
        }
        self.geometry_queries += 1;
        match self.session.document().native_caret_rect(position) {
            Ok(rect) => (Some(rect), NativeAccessibilityGeometryStatus::Complete),
            Err(error) => {
                self.note(NativeAccessibilityLimitation::GeometryUnavailable);
                (None, geometry_error_status(&error))
            }
        }
    }

    fn node_geometry(
        &mut self,
        paragraph: &Paragraph,
        start: &NativeTextPosition,
        end: &NativeTextPosition,
    ) -> NativeAccessibilityNodeGeometry {
        if paragraph.text.len() > MAX_ACCESSIBILITY_GEOMETRY_TEXT_BYTES {
            self.note(NativeAccessibilityLimitation::GeometryLimit);
            return NativeAccessibilityNodeGeometry {
                start_caret: None,
                end_caret: None,
                status: NativeAccessibilityGeometryStatus::Limited,
            };
        }
        let (start_caret, start_status) = self.caret(start);
        let (end_caret, end_status) = if start == end {
            (start_caret, start_status)
        } else {
            self.caret(end)
        };
        NativeAccessibilityNodeGeometry {
            start_caret,
            end_caret,
            status: if start_status != NativeAccessibilityGeometryStatus::Complete {
                start_status
            } else {
                end_status
            },
        }
    }

    fn selection_geometry(
        &mut self,
        selection: &NativeTextSelection,
    ) -> Result<NativeAccessibilitySelectionGeometry, NativeHostError> {
        let mut result = NativeAccessibilitySelectionGeometry {
            focus_caret: None,
            rectangles: Vec::new(),
            status: NativeAccessibilityGeometryStatus::Limited,
        };
        let paragraph = resolve_paragraph(self.session, &selection.focus)?;
        if paragraph.text.len() > MAX_ACCESSIBILITY_GEOMETRY_TEXT_BYTES {
            self.note(NativeAccessibilityLimitation::GeometryLimit);
            return Ok(result);
        }
        (result.focus_caret, result.status) = self.caret(&selection.focus);
        if selection.anchor == selection.focus {
            return Ok(result);
        }
        let Some((start, end, first, last)) = selection_container_range(selection) else {
            self.note(NativeAccessibilityLimitation::SelectionGeometryUnsupported);
            result.status = NativeAccessibilityGeometryStatus::Unsupported;
            return Ok(result);
        };
        if last - first >= MAX_ACCESSIBILITY_SELECTION_PARAGRAPHS
            || self.geometry_queries == self.request.max_geometry_queries
        {
            self.note(NativeAccessibilityLimitation::GeometryLimit);
            result.status = NativeAccessibilityGeometryStatus::Limited;
            return Ok(result);
        }
        let mut text_bytes = 0;
        let mut selected_scalars = 0;
        for index in first..=last {
            let mut position = start.clone();
            if let Some(leaf) = position.cell_path.last_mut() {
                leaf.paragraph = index;
            } else {
                position.paragraph = index;
            }
            let paragraph = resolve_paragraph(self.session, &position)?;
            if paragraph.text.len() > MAX_ACCESSIBILITY_GEOMETRY_TEXT_BYTES - text_bytes {
                self.note(NativeAccessibilityLimitation::GeometryLimit);
                result.status = NativeAccessibilityGeometryStatus::Limited;
                return Ok(result);
            }
            text_bytes += paragraph.text.len();
            let from = if index == first {
                start.scalar_offset
            } else {
                0
            };
            let to = if index == last {
                end.scalar_offset
            } else {
                paragraph.text.chars().count() as u32
            };
            selected_scalars += to - from;
            if selected_scalars > MAX_ACCESSIBILITY_SELECTION_SCALARS {
                self.note(NativeAccessibilityLimitation::GeometryLimit);
                result.status = NativeAccessibilityGeometryStatus::Limited;
                return Ok(result);
            }
        }
        self.geometry_queries += 1;
        match self.session.document().native_selection_rects(selection) {
            Ok(mut rectangles) => {
                if rectangles.len() > self.request.max_selection_rects {
                    rectangles.truncate(self.request.max_selection_rects);
                    self.note(NativeAccessibilityLimitation::GeometryLimit);
                    result.status = NativeAccessibilityGeometryStatus::Limited;
                }
                result.rectangles = rectangles;
            }
            Err(error) => {
                self.note(NativeAccessibilityLimitation::GeometryUnavailable);
                result.status = geometry_error_status(&error);
            }
        }
        Ok(result)
    }
}

fn validate_request(request: &NativeAccessibilityRequest) -> Result<(), NativeHostError> {
    if request.max_nodes == 0
        || request.max_nodes > MAX_ACCESSIBILITY_NODES
        || request.max_text_bytes == 0
        || request.max_text_bytes > MAX_ACCESSIBILITY_TEXT_BYTES
        || request.max_depth > MAX_ACCESSIBILITY_DEPTH
        || request.max_geometry_queries > MAX_ACCESSIBILITY_GEOMETRY_QUERIES
        || request.max_selection_rects == 0
        || request.max_selection_rects > MAX_ACCESSIBILITY_SELECTION_RECTS
    {
        return Err(NativeHostError::LimitExceeded(
            "accessibility request exceeds node/text/depth/geometry limits".into(),
        ));
    }
    Ok(())
}

fn resolve_paragraph<'a>(
    session: &'a NativeDocumentSession,
    position: &NativeTextPosition,
) -> Result<&'a Paragraph, NativeHostError> {
    if position.cell_path.len() > MAX_ACCESSIBILITY_DEPTH {
        return Err(invalid("selection cell path exceeds depth limit"));
    }
    let mut paragraph = session
        .document()
        .document()
        .sections
        .get(position.section as usize)
        .and_then(|section| section.paragraphs.get(position.paragraph as usize))
        .ok_or_else(|| invalid("selection address is outside the document"))?;
    for entry in &position.cell_path {
        let Some(Control::Table(table)) = paragraph.controls.get(entry.control as usize) else {
            return Err(invalid("selection path must address table cells"));
        };
        paragraph = table
            .cells
            .get(entry.cell as usize)
            .and_then(|cell| cell.paragraphs.get(entry.paragraph as usize))
            .ok_or_else(|| invalid("selection cell address is outside the document"))?;
    }
    Ok(paragraph)
}

fn node_key(revision: u64, position: &NativeTextPosition) -> String {
    use std::fmt::Write;
    let mut key = format!("r{revision}:s{}:p{}", position.section, position.paragraph);
    for entry in &position.cell_path {
        let _ = write!(
            key,
            ":t{}:c{}:p{}",
            entry.control, entry.cell, entry.paragraph
        );
    }
    key
}

fn geometry_error_status(error: &NativeInteractionError) -> NativeAccessibilityGeometryStatus {
    if matches!(error, NativeInteractionError::Unsupported(_)) {
        NativeAccessibilityGeometryStatus::Unsupported
    } else {
        NativeAccessibilityGeometryStatus::Unavailable
    }
}

/// Match the public native selection query's body/cell container contract before
/// enumerating a bounded range. Paragraphs at the final cell path may differ.
fn selection_container_range(
    selection: &NativeTextSelection,
) -> Option<(&NativeTextPosition, &NativeTextPosition, u32, u32)> {
    let a = &selection.anchor;
    let b = &selection.focus;
    if a.section != b.section {
        return None;
    }
    match (a.cell_path.split_last(), b.cell_path.split_last()) {
        (None, None) => {}
        (Some((a_leaf, a_prefix)), Some((b_leaf, b_prefix)))
            if a.paragraph == b.paragraph
                && a_prefix == b_prefix
                && a_leaf.control == b_leaf.control
                && a_leaf.cell == b_leaf.cell => {}
        _ => return None,
    }
    let a_paragraph = a
        .cell_path
        .last()
        .map_or(a.paragraph, |leaf| leaf.paragraph);
    let b_paragraph = b
        .cell_path
        .last()
        .map_or(b.paragraph, |leaf| leaf.paragraph);
    if (a_paragraph, a.scalar_offset) <= (b_paragraph, b.scalar_offset) {
        Some((a, b, a_paragraph, b_paragraph))
    } else {
        Some((b, a, b_paragraph, a_paragraph))
    }
}

fn address_index(index: usize) -> Result<u32, NativeHostError> {
    u32::try_from(index)
        .map_err(|_| NativeHostError::LimitExceeded("structural address exceeds u32".into()))
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}

/// Convert decoded text scalar boundaries to OS UTF-16 units. These are not HWP
/// raw char_offsets (which can include control slots). Inputs are <=1 MiB UTF-8.
pub fn native_scalar_to_utf16(text: &str, offset: u32) -> Result<u32, NativeHostError> {
    validate_conversion_text(text)?;
    let mut scalar = 0;
    let mut utf16 = 0;
    for character in text.chars() {
        if scalar == offset {
            return Ok(utf16);
        }
        scalar += 1;
        utf16 += character.len_utf16() as u32;
    }
    if scalar == offset {
        Ok(utf16)
    } else {
        Err(invalid("scalar offset is outside the text"))
    }
}

/// Reject UTF-16 offsets inside a surrogate pair, rather than rounding to a
/// neighboring scalar. Combining marks remain distinct scalars, not graphemes.
pub fn native_utf16_to_scalar(text: &str, offset: u32) -> Result<u32, NativeHostError> {
    validate_conversion_text(text)?;
    let mut scalar = 0;
    let mut utf16 = 0;
    for character in text.chars() {
        if utf16 == offset {
            return Ok(scalar);
        }
        let next = utf16 + character.len_utf16() as u32;
        if offset < next {
            return Err(invalid("UTF-16 offset splits a surrogate pair"));
        }
        utf16 = next;
        scalar += 1;
    }
    if utf16 == offset {
        Ok(scalar)
    } else {
        Err(invalid("UTF-16 offset is outside the text"))
    }
}

fn validate_conversion_text(text: &str) -> Result<(), NativeHostError> {
    if text.len() > MAX_ACCESSIBILITY_TEXT_BYTES {
        Err(NativeHostError::LimitExceeded(
            "UTF conversion text exceeds 1 MiB".into(),
        ))
    } else {
        Ok(())
    }
}
