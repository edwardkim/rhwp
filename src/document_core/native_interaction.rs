//! Typed interaction queries over the existing layout and cursor implementation.
//!
//! Positions use Unicode scalar offsets, never UTF-16 code units or logical
//! inline-object slots. Body text and nested table cells are supported. A target
//! paragraph containing logical inline controls or character overlaps is rejected:
//! the legacy cursor contract mixes text and object slots, and this facade has no
//! object-side affinity. Text boxes, captions, notes and header/footer editing are
//! also outside this initial contract. Queries do not change the document/caret.

use super::DocumentCore;
use crate::error::HwpError;
use crate::model::{control::Control, paragraph::Paragraph};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum nested table address depth accepted by the native facade.
pub const NATIVE_CELL_PATH_MAX_DEPTH: usize = 32;

/// One table/cell/paragraph descent, relative to the preceding paragraph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeCellPathEntry {
    pub control: u32,
    pub cell: u32,
    pub paragraph: u32,
}

/// `paragraph` always identifies a top-level body paragraph. For a nonempty
/// `cell_path`, its last entry identifies the leaf paragraph being addressed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTextPosition {
    pub section: u32,
    pub paragraph: u32,
    pub scalar_offset: u32,
    pub cell_path: Vec<NativeCellPathEntry>,
}

/// Anchor/focus order is preserved; geometry normalizes their document order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTextSelection {
    pub anchor: NativeTextPosition,
    pub focus: NativeTextPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeInteractionBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Page-local layout coordinates, in the same units as the core page tree.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCaretRect {
    pub page: u32,
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub cell_bounds: Option<NativeInteractionBounds>,
    pub cell_overflowed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSelectionRect {
    pub page: u32,
    pub bounds: NativeInteractionBounds,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeHitTest {
    pub position: NativeTextPosition,
    /// None when the existing hit-test returned a position without geometry.
    pub caret: Option<NativeCaretRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeVerticalDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeVerticalMove {
    pub position: NativeTextPosition,
    /// None represents the core's explicit `rectValid:false`, not a fake rect.
    pub caret: Option<NativeCaretRect>,
    pub preferred_x: f64,
}

#[derive(Debug)]
pub enum NativeInteractionError {
    InvalidInput(&'static str),
    InvalidPosition(String),
    Unsupported(&'static str),
    InvalidCoreResult(String),
    Core(HwpError),
}

impl fmt::Display for NativeInteractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::Unsupported(message) => f.write_str(message),
            Self::InvalidPosition(message) | Self::InvalidCoreResult(message) => {
                f.write_str(message)
            }
            Self::Core(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for NativeInteractionError {}

impl From<HwpError> for NativeInteractionError {
    fn from(error: HwpError) -> Self {
        Self::Core(error)
    }
}

impl DocumentCore {
    /// Validate a body/table address and its Unicode scalar boundary. This does
    /// not imply that its paragraph is supported for geometry or editing.
    pub fn validate_native_text_position(
        &self,
        position: &NativeTextPosition,
    ) -> Result<(), NativeInteractionError> {
        self.native_text_paragraph(position).map(|_| ())
    }

    /// Check the scalar-only interaction subset, also usable by native editors.
    pub fn check_native_interaction_support(
        &self,
        position: &NativeTextPosition,
    ) -> Result<(), NativeInteractionError> {
        let paragraph = self.native_text_paragraph(position)?;
        if paragraph.controls.iter().any(|control| {
            control.is_logical_inline() || matches!(control, Control::CharOverlap(_))
        }) {
            return Err(NativeInteractionError::Unsupported(
                "inline-object affinity is not supported by scalar interaction positions",
            ));
        }
        Ok(())
    }

    pub fn native_caret_rect(
        &self,
        position: &NativeTextPosition,
    ) -> Result<NativeCaretRect, NativeInteractionError> {
        self.check_native_interaction_support(position)?;
        let json = if position.cell_path.is_empty() {
            self.get_cursor_rect_native(
                position.section as usize,
                position.paragraph as usize,
                position.scalar_offset as usize,
            )?
        } else {
            self.get_cursor_rect_by_path_native(
                position.section as usize,
                position.paragraph as usize,
                &legacy_path_json(&position.cell_path),
                position.scalar_offset as usize,
            )?
        };
        self.native_decode_caret(decode(&json)?)
    }

    pub fn native_hit_test(
        &self,
        page: u32,
        x: f64,
        y: f64,
    ) -> Result<NativeHitTest, NativeInteractionError> {
        self.native_validate_page(page)?;
        if !x.is_finite() || !y.is_finite() {
            return Err(NativeInteractionError::InvalidInput(
                "hit coordinates must be finite",
            ));
        }
        let hit: LegacyHit = decode(&self.hit_test_native(page, x, y)?)?;
        Ok(NativeHitTest {
            position: self.native_decode_position(hit.position)?,
            caret: hit
                .caret
                .map(|rect| self.native_decode_caret(rect))
                .transpose()?,
        })
    }

    /// Selections must remain within one body section or one table-cell
    /// container. Different paragraphs in that container are supported.
    pub fn native_selection_rects(
        &self,
        selection: &NativeTextSelection,
    ) -> Result<Vec<NativeSelectionRect>, NativeInteractionError> {
        let mut start = &selection.anchor;
        let mut end = &selection.focus;
        self.check_native_interaction_support(start)?;
        self.check_native_interaction_support(end)?;
        if !same_container(start, end) {
            return Err(NativeInteractionError::Unsupported(
                "selection across sections or cell containers is not supported",
            ));
        }
        if position_order(start) > position_order(end) {
            std::mem::swap(&mut start, &mut end);
        }
        // Intermediate paragraphs also need scalar-only geometry. Checking only
        // the endpoints would silently reinterpret inline slots in the middle.
        for paragraph in position_order(start).0..=position_order(end).0 {
            let mut position = start.clone();
            position.scalar_offset = 0;
            if let Some(leaf) = position.cell_path.last_mut() {
                leaf.paragraph = paragraph;
            } else {
                position.paragraph = paragraph;
            }
            self.check_native_interaction_support(&position)?;
        }
        if start == end {
            return Ok(Vec::new());
        }
        let json = if start.cell_path.is_empty() {
            self.get_selection_rects_native(
                start.section as usize,
                start.paragraph as usize,
                start.scalar_offset as usize,
                end.paragraph as usize,
                end.scalar_offset as usize,
                None,
                None,
            )?
        } else {
            self.get_selection_rects_in_cell_by_path_native(
                start.section as usize,
                start.paragraph as usize,
                &legacy_path_json(&start.cell_path),
                position_order(start).0 as usize,
                start.scalar_offset as usize,
                position_order(end).0 as usize,
                end.scalar_offset as usize,
                None,
            )?
        };
        let rects: Vec<LegacySelectionRect> = decode(&json)?;
        rects
            .into_iter()
            .map(|rect| {
                self.native_validate_page(rect.page_index)?;
                let bounds = NativeInteractionBounds {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                };
                validate_bounds(bounds)?;
                Ok(NativeSelectionRect {
                    page: rect.page_index,
                    bounds,
                })
            })
            .collect()
    }

    /// Move one visual line through the existing navigation implementation.
    /// Nested cells preserve the complete path. Their existing table-boundary
    /// behavior is retained: navigation can stay at the boundary rather than exit.
    pub fn native_move_vertical(
        &self,
        position: &NativeTextPosition,
        direction: NativeVerticalDirection,
        preferred_x: Option<f64>,
    ) -> Result<NativeVerticalMove, NativeInteractionError> {
        self.check_native_interaction_support(position)?;
        if preferred_x.is_some_and(|x| !x.is_finite() || x < 0.0) {
            return Err(NativeInteractionError::InvalidInput(
                "preferred x must be finite and nonnegative",
            ));
        }
        let delta = match direction {
            NativeVerticalDirection::Up => -1,
            NativeVerticalDirection::Down => 1,
        };
        let preferred_x = preferred_x.unwrap_or(-1.0);
        let json = if position.cell_path.is_empty() {
            self.move_vertical_native(
                position.section as usize,
                position.paragraph as usize,
                position.scalar_offset as usize,
                delta,
                preferred_x,
                None,
            )?
        } else {
            self.move_vertical_by_path_native(
                position.section as usize,
                position.paragraph as usize,
                &legacy_path_json(&position.cell_path),
                position.scalar_offset as usize,
                delta,
                preferred_x,
            )?
        };
        let movement: LegacyMove = decode(&json)?;
        if !movement.preferred_x.is_finite() || movement.preferred_x < 0.0 {
            return Err(invalid_result("navigation returned an invalid preferred x"));
        }
        Ok(NativeVerticalMove {
            position: self.native_decode_position(movement.position)?,
            caret: if movement.rect_valid {
                Some(self.native_decode_caret(movement.caret)?)
            } else {
                None
            },
            preferred_x: movement.preferred_x,
        })
    }

    fn native_text_paragraph<'a>(
        &'a self,
        position: &NativeTextPosition,
    ) -> Result<&'a Paragraph, NativeInteractionError> {
        if position.cell_path.len() > NATIVE_CELL_PATH_MAX_DEPTH {
            return Err(NativeInteractionError::InvalidInput(
                "cell path exceeds depth limit",
            ));
        }
        let invalid = || {
            NativeInteractionError::InvalidPosition("text position is outside the document".into())
        };
        let mut paragraph = self
            .document
            .sections
            .get(position.section as usize)
            .and_then(|section| section.paragraphs.get(position.paragraph as usize))
            .ok_or_else(invalid)?;
        for entry in &position.cell_path {
            let control = paragraph
                .controls
                .get(entry.control as usize)
                .ok_or_else(invalid)?;
            let Control::Table(table) = control else {
                return Err(NativeInteractionError::Unsupported(
                    "only table-cell paths are supported",
                ));
            };
            paragraph = table
                .cells
                .get(entry.cell as usize)
                .and_then(|cell| cell.paragraphs.get(entry.paragraph as usize))
                .ok_or_else(invalid)?;
        }
        if position.scalar_offset as usize > paragraph.text.chars().count() {
            return Err(invalid());
        }
        Ok(paragraph)
    }

    fn native_validate_page(&self, page: u32) -> Result<(), NativeInteractionError> {
        if page >= self.page_count() {
            return Err(NativeInteractionError::InvalidInput(
                "page index is outside the document",
            ));
        }
        Ok(())
    }

    fn native_decode_position(
        &self,
        legacy: LegacyPosition,
    ) -> Result<NativeTextPosition, NativeInteractionError> {
        if legacy.is_text_box || legacy.inner_control_idx.is_some() {
            return Err(NativeInteractionError::Unsupported(
                "object/text-box hits are not supported",
            ));
        }
        let cell_path = match (
            legacy.parent_para_index,
            legacy.control_index,
            legacy.cell_index,
            legacy.cell_para_index,
        ) {
            (None, None, None, None) if legacy.cell_path.is_none() => Vec::new(),
            (Some(_), Some(control), Some(cell), Some(paragraph)) => {
                let path = legacy.cell_path.unwrap_or_else(|| {
                    vec![LegacyPathEntry {
                        control_index: control,
                        cell_index: cell,
                        cell_para_index: paragraph,
                    }]
                });
                let Some(first) = path.first() else {
                    return Err(invalid_result("empty cell path in core position"));
                };
                if first.control_index != control
                    || first.cell_index != cell
                    || first.cell_para_index != paragraph
                {
                    return Err(invalid_result("inconsistent cell path in core position"));
                }
                path.into_iter()
                    .map(|entry| NativeCellPathEntry {
                        control: entry.control_index,
                        cell: entry.cell_index,
                        paragraph: entry.cell_para_index,
                    })
                    .collect()
            }
            _ => return Err(invalid_result("incomplete cell context in core position")),
        };
        let position = NativeTextPosition {
            section: legacy.section_index,
            paragraph: legacy.parent_para_index.unwrap_or(legacy.paragraph_index),
            scalar_offset: legacy.char_offset,
            cell_path,
        };
        self.check_native_interaction_support(&position)?;
        Ok(position)
    }

    fn native_decode_caret(
        &self,
        legacy: LegacyCaret,
    ) -> Result<NativeCaretRect, NativeInteractionError> {
        self.native_validate_page(legacy.page_index)?;
        if !legacy.x.is_finite()
            || !legacy.y.is_finite()
            || !legacy.height.is_finite()
            || legacy.height <= 0.0
        {
            return Err(invalid_result("core returned invalid caret geometry"));
        }
        let cell_bounds = legacy
            .cell_bounds
            .map(|bounds| -> Result<_, NativeInteractionError> {
                let bounds = NativeInteractionBounds {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.w,
                    height: bounds.h,
                };
                validate_bounds(bounds)?;
                Ok(bounds)
            })
            .transpose()?;
        Ok(NativeCaretRect {
            page: legacy.page_index,
            x: legacy.x,
            y: legacy.y,
            height: legacy.height,
            cell_bounds,
            cell_overflowed: legacy.cell_overflowed,
        })
    }
}

fn position_order(position: &NativeTextPosition) -> (u32, u32) {
    (
        position
            .cell_path
            .last()
            .map_or(position.paragraph, |entry| entry.paragraph),
        position.scalar_offset,
    )
}

fn same_container(a: &NativeTextPosition, b: &NativeTextPosition) -> bool {
    a.section == b.section
        && match (a.cell_path.split_last(), b.cell_path.split_last()) {
            (None, None) => true,
            (Some((a_leaf, a_prefix)), Some((b_leaf, b_prefix))) => {
                a.paragraph == b.paragraph
                    && a_prefix == b_prefix
                    && a_leaf.control == b_leaf.control
                    && a_leaf.cell == b_leaf.cell
            }
            _ => false,
        }
}

fn legacy_path_json(path: &[NativeCellPathEntry]) -> String {
    DocumentCore::format_path_json(
        &path
            .iter()
            .map(|entry| {
                (
                    entry.control as usize,
                    entry.cell as usize,
                    entry.paragraph as usize,
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn validate_bounds(bounds: NativeInteractionBounds) -> Result<(), NativeInteractionError> {
    if ![bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .all(f64::is_finite)
        || bounds.width < 0.0
        || bounds.height < 0.0
    {
        return Err(invalid_result("core returned invalid rectangle geometry"));
    }
    Ok(())
}

fn invalid_result(message: &str) -> NativeInteractionError {
    NativeInteractionError::InvalidCoreResult(message.into())
}

fn decode<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, NativeInteractionError> {
    serde_json::from_str(json)
        .map_err(|error| invalid_result(&format!("invalid core interaction result: {error}")))
}

// Required position/geometry fields have no defaults. Optional legacy metadata
// is deliberately tolerated; it is not used to invent missing coordinates.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyPosition {
    section_index: u32,
    paragraph_index: u32,
    char_offset: u32,
    parent_para_index: Option<u32>,
    control_index: Option<u32>,
    cell_index: Option<u32>,
    cell_para_index: Option<u32>,
    cell_path: Option<Vec<LegacyPathEntry>>,
    #[serde(default)]
    is_text_box: bool,
    inner_control_idx: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyPathEntry {
    control_index: u32,
    cell_index: u32,
    cell_para_index: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyHit {
    #[serde(flatten)]
    position: LegacyPosition,
    #[serde(rename = "cursorRect")]
    caret: Option<LegacyCaret>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyCaret {
    page_index: u32,
    x: f64,
    y: f64,
    height: f64,
    cell_bounds: Option<LegacyBounds>,
    #[serde(default)]
    cell_overflowed: bool,
}

#[derive(Deserialize)]
struct LegacyBounds {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyMove {
    #[serde(flatten)]
    position: LegacyPosition,
    #[serde(flatten)]
    caret: LegacyCaret,
    preferred_x: f64,
    #[serde(default = "true_value")]
    rect_valid: bool,
}

fn true_value() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacySelectionRect {
    page_index: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
