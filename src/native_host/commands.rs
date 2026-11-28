//! Editing commands over scalar addresses and extended Unicode graphemes.
//!
//! Left/right are logical text order, not visual bidi navigation. Home/End mean
//! paragraph boundaries; Up/Down consume the existing core visual-line query.
//! SelectAll selects the current body section or the current cell paragraph.
//! Hosts must commit/cancel composition before issuing these commands.

use std::ops::Range;

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::document_core::native_interaction::{
    legacy_cell_path, NativeTextPosition, NativeTextSelection, NativeVerticalDirection,
};
use crate::model::{paragraph::Paragraph, style::Alignment};

use super::editor::{NativeEditOutcome, NativeEditor};
use super::NativeHostError;

#[path = "cell_paragraph_commands.rs"]
mod cell_paragraph_commands;
#[path = "table_navigation.rs"]
mod table_navigation;

/// Tab moves to the next editable cell, Shift+Tab to the previous cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeTableCellDirection {
    Next,
    Previous,
}

/// Paragraph alignment on the Native command wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeParagraphAlignment {
    Justify,
    Left,
    Right,
    Center,
    Distribute,
    Split,
}

impl From<NativeParagraphAlignment> for Alignment {
    fn from(alignment: NativeParagraphAlignment) -> Self {
        match alignment {
            NativeParagraphAlignment::Justify => Self::Justify,
            NativeParagraphAlignment::Left => Self::Left,
            NativeParagraphAlignment::Right => Self::Right,
            NativeParagraphAlignment::Center => Self::Center,
            NativeParagraphAlignment::Distribute => Self::Distribute,
            NativeParagraphAlignment::Split => Self::Split,
        }
    }
}

impl From<Alignment> for NativeParagraphAlignment {
    fn from(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Justify => Self::Justify,
            Alignment::Left => Self::Left,
            Alignment::Right => Self::Right,
            Alignment::Center => Self::Center,
            Alignment::Distribute => Self::Distribute,
            Alignment::Split => Self::Split,
        }
    }
}

/// A property can be uniform even when the selected text uses several shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum NativeFormattingValue<T> {
    Uniform(T),
    Mixed,
}

impl<T: PartialEq> NativeFormattingValue<T> {
    fn include(&mut self, value: T) {
        if matches!(self, Self::Uniform(current) if *current != value) {
            *self = Self::Mixed;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSelectionFormatting {
    pub bold: NativeFormattingValue<bool>,
    /// HWPUNIT (100 units per point), matching formatCharacters.
    pub font_size: NativeFormattingValue<i32>,
    /// Always uniform because the selection belongs to one editable paragraph.
    pub alignment: NativeParagraphAlignment,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "command",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NativeEditCommand {
    MoveLeft {
        #[serde(default)]
        extend_selection: bool,
    },
    MoveRight {
        #[serde(default)]
        extend_selection: bool,
    },
    Home {
        #[serde(default)]
        extend_selection: bool,
    },
    End {
        #[serde(default)]
        extend_selection: bool,
    },
    MoveUp {
        #[serde(default)]
        extend_selection: bool,
        preferred_x: Option<f64>,
    },
    MoveDown {
        #[serde(default)]
        extend_selection: bool,
        preferred_x: Option<f64>,
    },
    /// Move within the innermost table without changing the document.
    NavigateTableCell {
        direction: NativeTableCellDirection,
    },
    /// Delete the scalar selection, or one complete grapheme at a caret.
    /// At the paragraph start, join with the preceding plain paragraph.
    DeleteBackward {},
    /// Delete the scalar selection, or one complete grapheme at a caret.
    /// At the paragraph end, join with the following body paragraph.
    DeleteForward {},
    /// Replace the scalar selection with a paragraph boundary as one action.
    SplitParagraph {},
    /// Join the focus paragraph into its predecessor. Requires a collapsed
    /// selection and places the caret at the join. Backspace/Delete instead
    /// delete a noncollapsed selection.
    JoinPreviousParagraph {},
    SelectAll {},
    /// Change at least one character property within a nonempty selection in one
    /// plain paragraph. Font size is in HWPUNIT (100 units per point).
    FormatCharacters {
        bold: Option<bool>,
        font_size: Option<i32>,
    },
    /// Align the paragraph that owns the selection. A collapsed caret is valid.
    SetParagraphAlignment {
        alignment: NativeParagraphAlignment,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeCommandOutcome {
    pub outcome: NativeEditOutcome,
    /// Carry this into the next vertical command to keep the preferred column.
    /// Other commands return None, so the host can reset its vertical sequence.
    pub preferred_x: Option<f64>,
}

impl NativeEditor {
    pub fn execute_command(
        &mut self,
        expected_revision: u64,
        command: NativeEditCommand,
    ) -> Result<NativeCommandOutcome, NativeHostError> {
        self.check_edit_ready(expected_revision)?;
        let mut preferred_x = None;
        let outcome = match command {
            NativeEditCommand::MoveLeft { extend_selection } => {
                let target = self.horizontal_target(true, extend_selection)?;
                self.move_focus(expected_revision, target, extend_selection)?
            }
            NativeEditCommand::MoveRight { extend_selection } => {
                let target = self.horizontal_target(false, extend_selection)?;
                self.move_focus(expected_revision, target, extend_selection)?
            }
            NativeEditCommand::Home { extend_selection } => {
                let mut target = self.selection().focus.clone();
                target.scalar_offset = 0;
                self.move_focus(expected_revision, target, extend_selection)?
            }
            NativeEditCommand::End { extend_selection } => {
                let mut target = self.selection().focus.clone();
                target.scalar_offset = scalar_len(self.command_text(&target)?)?;
                self.move_focus(expected_revision, target, extend_selection)?
            }
            NativeEditCommand::MoveUp {
                extend_selection,
                preferred_x: x,
            }
            | NativeEditCommand::MoveDown {
                extend_selection,
                preferred_x: x,
            } => {
                let direction = if matches!(command, NativeEditCommand::MoveUp { .. }) {
                    NativeVerticalDirection::Up
                } else {
                    NativeVerticalDirection::Down
                };
                let movement = self.session().document().native_move_vertical(
                    &self.selection().focus,
                    direction,
                    x,
                )?;
                preferred_x = Some(movement.preferred_x);
                self.move_focus(expected_revision, movement.position, extend_selection)?
            }
            NativeEditCommand::NavigateTableCell { direction } => {
                let selection = self.table_cell_selection(direction)?;
                self.set_selection(expected_revision, selection)?
            }
            NativeEditCommand::DeleteBackward {} => {
                self.delete_grapheme(expected_revision, true)?
            }
            NativeEditCommand::DeleteForward {} => {
                self.delete_grapheme(expected_revision, false)?
            }
            NativeEditCommand::SplitParagraph {} => self.split_selection(expected_revision)?,
            NativeEditCommand::JoinPreviousParagraph {} => {
                if self.selection().anchor != self.selection().focus {
                    return Err(NativeHostError::InvalidArgument(
                        "joinPreviousParagraph requires a collapsed selection".into(),
                    ));
                }
                if self.selection().focus.cell_path.is_empty() {
                    self.join_at(expected_revision, self.selection().focus.paragraph)?
                } else {
                    self.join_previous_cell_paragraph(expected_revision)?
                }
            }
            NativeEditCommand::FormatCharacters { bold, font_size } => {
                self.format_characters(expected_revision, bold, font_size)?
            }
            NativeEditCommand::SetParagraphAlignment { alignment } => {
                self.set_paragraph_alignment(expected_revision, alignment)?
            }
            NativeEditCommand::SelectAll {} => self.select_all(expected_revision)?,
        };
        Ok(NativeCommandOutcome {
            outcome,
            preferred_x,
        })
    }

    fn formatting_paragraph(&self) -> Result<&Paragraph, NativeHostError> {
        let selection = self.selection();
        if selection.anchor.paragraph != selection.focus.paragraph {
            return Err(NativeHostError::Unsupported(
                "formatting requires a selection within one paragraph".into(),
            ));
        }
        let core = self.session().document();
        Self::check_editable_text_span(core, selection)?;
        Ok(core.native_text_paragraph(&selection.focus)?)
    }

    /// Read the current editable leaf without changing selection or history.
    pub fn selection_formatting(
        &self,
        revision: u64,
        font_generation: u64,
        selection: &NativeTextSelection,
    ) -> Result<NativeSelectionFormatting, NativeHostError> {
        self.check_edit_ready(revision)?;
        if font_generation != self.session().font_generation() {
            return Err(NativeHostError::InvalidArgument(
                "stale font generation".into(),
            ));
        }
        if selection != self.selection() {
            return Err(NativeHostError::InvalidArgument("stale selection".into()));
        }

        let paragraph = self.formatting_paragraph()?;
        let doc_info = &self.session().document().document().doc_info;
        let (from, to) = Self::ordered(selection);
        let start = from.scalar_offset as usize;
        let end = to.scalar_offset as usize;
        if end > paragraph.char_offsets.len() {
            return Err(NativeHostError::InvalidArgument(
                "invalid character format range".into(),
            ));
        }

        // At a caret use the shape at the insertion offset, including the
        // paragraph-end shape. Otherwise read only actual selected text runs.
        let ids = if start == end {
            vec![paragraph.char_shape_id_at(start).unwrap_or(0)]
        } else {
            paragraph
                .char_shape_runs(start, end)
                .into_iter()
                .map(|run| run.char_shape_id)
                .collect()
        };
        let shape_for = |id: u32| {
            doc_info.char_shapes.get(id as usize).ok_or_else(|| {
                NativeHostError::InvalidArgument("unknown character shape in selection".into())
            })
        };
        let mut ids = ids.into_iter();
        let first = ids.next().ok_or_else(|| {
            NativeHostError::InvalidArgument("invalid character format range".into())
        })?;
        let first = shape_for(first)?;
        let mut bold = NativeFormattingValue::Uniform(first.bold);
        let mut font_size = NativeFormattingValue::Uniform(first.base_size);
        for id in ids {
            let shape = shape_for(id)?;
            bold.include(shape.bold);
            font_size.include(shape.base_size);
        }
        let alignment = doc_info
            .para_shapes
            .get(paragraph.para_shape_id as usize)
            .ok_or_else(|| {
                NativeHostError::InvalidArgument("unknown paragraph shape in selection".into())
            })?
            .alignment
            .into();
        Ok(NativeSelectionFormatting {
            bold,
            font_size,
            alignment,
        })
    }

    fn format_characters(
        &mut self,
        revision: u64,
        bold: Option<bool>,
        font_size: Option<i32>,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        if bold.is_none() && font_size.is_none() {
            return Err(NativeHostError::InvalidArgument(
                "formatCharacters requires bold or fontSize".into(),
            ));
        }
        // The model stores its base size as HWPUNIT, with 100 units per point
        // and an upper bound of 4096 points.
        if font_size.is_some_and(|size| !(1..=409_600).contains(&size)) {
            return Err(NativeHostError::InvalidArgument(
                "fontSize must be within 1..=409600 HWPUNIT".into(),
            ));
        }
        let paragraph = self.formatting_paragraph()?;
        let (start, end) = Self::ordered(self.selection());
        let range = start.scalar_offset as usize..end.scalar_offset as usize;
        if range.is_empty() {
            return Err(NativeHostError::InvalidArgument(
                "formatCharacters requires a nonempty text selection".into(),
            ));
        }
        let shapes = &self.session().document().document().doc_info.char_shapes;
        let ids = paragraph.char_shape_ids_in_range(range.start, range.end);
        if ids.is_empty() {
            return Err(NativeHostError::InvalidArgument(
                "invalid character format range".into(),
            ));
        }
        let mut changes_style = false;
        for id in ids {
            let shape = shapes.get(id as usize).ok_or_else(|| {
                NativeHostError::InvalidArgument("unknown character shape in selection".into())
            })?;
            changes_style |= bold.is_some_and(|bold| shape.bold != bold)
                || font_size.is_some_and(|size| shape.base_size != size);
        }
        if !changes_style {
            return Ok(self.outcome(false, false));
        }
        let properties = serde_json::json!({"bold": bold, "fontSize": font_size});
        self.apply_format(revision, Some(range), &properties.to_string())
    }

    fn set_paragraph_alignment(
        &mut self,
        revision: u64,
        alignment: NativeParagraphAlignment,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let paragraph = self.formatting_paragraph()?;
        let shapes = &self.session().document().document().doc_info.para_shapes;
        let shape = shapes
            .get(paragraph.para_shape_id as usize)
            .ok_or_else(|| {
                NativeHostError::InvalidArgument("unknown paragraph shape in selection".into())
            })?;
        if shape.alignment == Alignment::from(alignment) {
            return Ok(self.outcome(false, false));
        }
        let properties = serde_json::json!({"alignment": alignment});
        self.apply_format(revision, None, &properties.to_string())
    }

    fn apply_format(
        &mut self,
        revision: u64,
        characters: Option<Range<usize>>,
        properties: &str,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let selection = self.selection().clone();
        let position = &selection.focus;
        let section = position.section as usize;
        let paragraph = position.paragraph as usize;
        let path = legacy_cell_path(&position.cell_path);
        self.edit_transaction(revision, move |core| {
            match (path.is_empty(), characters) {
                (true, Some(range)) => core.apply_char_format_native(
                    section,
                    paragraph,
                    range.start,
                    range.end,
                    properties,
                ),
                (false, Some(range)) => core.apply_char_format_in_cell_by_path(
                    section,
                    paragraph,
                    &path,
                    range.start,
                    range.end,
                    properties,
                ),
                (true, None) => core.apply_para_format_native(section, paragraph, properties),
                (false, None) => {
                    core.apply_para_format_in_cell_by_path(section, paragraph, &path, properties)
                }
            }?;
            Ok(selection)
        })
    }

    fn move_focus(
        &mut self,
        revision: u64,
        focus: NativeTextPosition,
        extend: bool,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let anchor = if extend {
            self.selection().anchor.clone()
        } else {
            focus.clone()
        };
        if extend
            && !anchor.cell_path.is_empty()
            && (anchor.section != focus.section
                || anchor.paragraph != focus.paragraph
                || anchor.cell_path != focus.cell_path)
        {
            return self.set_selection(revision, self.selection().clone());
        }
        self.set_selection(revision, NativeTextSelection { anchor, focus })
    }

    fn command_text(&self, position: &NativeTextPosition) -> Result<&str, NativeHostError> {
        Ok(&self
            .session()
            .document()
            .native_text_paragraph(position)?
            .text)
    }

    fn select_all(&mut self, revision: u64) -> Result<NativeEditOutcome, NativeHostError> {
        let mut start = self.selection().focus.clone();
        let mut end = start.clone();
        start.scalar_offset = 0;
        if start.cell_path.is_empty() {
            start.paragraph = 0;
            let paragraphs =
                &self.session().document().document().sections[start.section as usize].paragraphs;
            end.paragraph = u32::try_from(paragraphs.len() - 1).map_err(|_| {
                NativeHostError::LimitExceeded("paragraph address range exhausted".into())
            })?;
            end.scalar_offset = 0;
        }
        end.scalar_offset = scalar_len(self.command_text(&end)?)?;
        self.set_selection(
            revision,
            NativeTextSelection {
                anchor: start,
                focus: end,
            },
        )
    }

    fn horizontal_target(
        &self,
        backward: bool,
        extend: bool,
    ) -> Result<NativeTextPosition, NativeHostError> {
        let (start, end) = Self::ordered(self.selection());
        if !extend && start != end {
            return Ok(if backward { start } else { end }.clone());
        }
        let mut target = self.selection().focus.clone();
        let text = self.command_text(&target)?;
        let length = scalar_len(text)?;
        if target.cell_path.is_empty() && backward && target.scalar_offset == 0 {
            if target.paragraph > 0 {
                target.paragraph -= 1;
                target.scalar_offset = scalar_len(self.command_text(&target)?)?;
            }
        } else if target.cell_path.is_empty() && !backward && target.scalar_offset == length {
            let count = self.session().document().document().sections[target.section as usize]
                .paragraphs
                .len();
            if (target.paragraph as usize) + 1 < count {
                target.paragraph = target.paragraph.checked_add(1).ok_or_else(|| {
                    NativeHostError::LimitExceeded("paragraph address range exhausted".into())
                })?;
                target.scalar_offset = 0;
            }
        } else {
            let (from, to) = grapheme_span(text, target.scalar_offset, backward);
            target.scalar_offset = if backward { from } else { to };
        }
        Ok(target)
    }

    fn delete_grapheme(
        &mut self,
        revision: u64,
        backward: bool,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        if self.selection().anchor != self.selection().focus {
            return self.replace_selection(revision, "");
        }
        let focus = self.selection().focus.clone();
        let text = self.command_text(&focus)?;
        let length = scalar_len(text)?;
        if backward && focus.scalar_offset == 0 {
            return if focus.cell_path.is_empty() {
                self.join_at(revision, focus.paragraph)
            } else {
                self.join_previous_cell_paragraph(revision)
            };
        }
        if !backward && focus.scalar_offset == length {
            let count = self.session().document().document().sections[focus.section as usize]
                .paragraphs
                .len();
            if !focus.cell_path.is_empty() || (focus.paragraph as usize) + 1 == count {
                return self.set_selection(revision, self.selection().clone());
            }
            let next = focus.paragraph.checked_add(1).ok_or_else(|| {
                NativeHostError::LimitExceeded("paragraph address range exhausted".into())
            })?;
            return self.join_at(revision, next);
        }
        // A scalar caret supplied inside a grapheme deletes the entire containing
        // grapheme, never an orphan combining mark or half of an emoji sequence.
        let (from, to) = grapheme_span(text, focus.scalar_offset, backward);
        let mut anchor = focus.clone();
        anchor.scalar_offset = from;
        let mut focus = focus;
        focus.scalar_offset = to;
        self.replace_range(revision, &NativeTextSelection { anchor, focus }, "")
    }

    fn split_selection(&mut self, revision: u64) -> Result<NativeEditOutcome, NativeHostError> {
        let selection = self.selection().clone();
        if !selection.focus.cell_path.is_empty() {
            return self.split_cell_selection(revision, selection);
        }
        let (start, end) = Self::ordered(&selection);
        let mut after = start.clone();
        after.paragraph = after.paragraph.checked_add(1).ok_or_else(|| {
            NativeHostError::LimitExceeded("paragraph address range exhausted".into())
        })?;
        after.scalar_offset = 0;
        self.edit_transaction(revision, |core| {
            if start != end {
                Self::apply_replacement(core, &selection, "")?;
            }
            core.split_paragraph_native(
                start.section as usize,
                start.paragraph as usize,
                start.scalar_offset as usize,
                None,
            )?;
            Ok(NativeTextSelection {
                anchor: after.clone(),
                focus: after,
            })
        })
    }

    fn join_at(
        &mut self,
        revision: u64,
        paragraph: u32,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        if paragraph == 0 {
            return self.set_selection(revision, self.selection().clone());
        }
        let mut next = self.selection().focus.clone();
        next.paragraph = paragraph;
        next.scalar_offset = 0;
        let mut previous = next.clone();
        previous.paragraph -= 1;
        previous.scalar_offset = scalar_len(self.command_text(&previous)?)?;
        let span = NativeTextSelection {
            anchor: previous.clone(),
            focus: next.clone(),
        };
        Self::validate_selection(self.session().document(), &span)?;
        // The current caret alone does not cover the adjacent paragraph whose
        // text and controls this join would move.
        Self::check_body_text_span(self.session().document(), &span)?;
        self.edit_transaction(revision, |core| {
            core.merge_paragraph_native(next.section as usize, paragraph as usize)?;
            Ok(NativeTextSelection {
                anchor: previous.clone(),
                focus: previous,
            })
        })
    }
}

fn scalar_len(text: &str) -> Result<u32, NativeHostError> {
    u32::try_from(text.chars().count()).map_err(|_| {
        NativeHostError::LimitExceeded("paragraph exceeds the scalar address range".into())
    })
}

/// Called only after scalar_len proved that the paragraph fits the public range.
fn grapheme_span(text: &str, offset: u32, backward: bool) -> (u32, u32) {
    let mut start = 0;
    for grapheme in text.graphemes(true) {
        let end = start + grapheme.chars().count() as u32;
        if offset < end || (backward && offset == end) {
            return (start, end);
        }
        start = end;
    }
    (start, start)
}

/// The surrounding-text window and selected-text cap share this scalar limit.
pub const NATIVE_IME_CONTEXT_SCALARS: usize = 4096;

/// Bounded surrounding text for an OS adapter. Cursor/anchor are scalar offsets
/// inside `text`; the OS adapter converts them to UTF-16 where required.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeImeContext {
    pub text: String,
    pub cursor_scalar: u32,
    pub anchor_scalar: Option<u32>,
    pub selected_text: String,
    pub absolute_start_scalar: u32,
    pub position: NativeTextPosition,
    pub text_truncated: bool,
    pub selection_truncated: bool,
}

impl NativeEditor {
    /// Query also during preedit; never move selection or mutate the document.
    pub fn ime_context(&self, expected_revision: u64) -> Result<NativeImeContext, NativeHostError> {
        self.session().check_revision(expected_revision)?;
        let selection = self.selection();
        Self::validate_selection(self.session().document(), selection)?;
        let core = self.session().document();
        let focus = &selection.focus;
        if !focus.cell_path.is_empty() {
            Self::check_editable_text_span(core, selection)?;
        }
        let paragraph = core.native_text_paragraph(focus)?;
        let length = paragraph.text.chars().count();
        let start = (focus.scalar_offset as usize).saturating_sub(NATIVE_IME_CONTEXT_SCALARS / 2);
        let end = length.min(start.saturating_add(NATIVE_IME_CONTEXT_SCALARS));
        let text = paragraph
            .text
            .chars()
            .skip(start)
            .take(end - start)
            .collect();
        let anchor = &selection.anchor;
        let anchor_scalar = (anchor.paragraph == focus.paragraph
            && (anchor.scalar_offset as usize) >= start
            && (anchor.scalar_offset as usize) <= end)
            .then(|| anchor.scalar_offset - start as u32);
        let (from, to) = Self::ordered(selection);
        let mut selected = String::new();
        let mut count = 0usize;
        let mut truncated = false;
        'paragraphs: for index in from.paragraph..=to.paragraph {
            if index > from.paragraph {
                if count == NATIVE_IME_CONTEXT_SCALARS {
                    truncated = true;
                    break;
                }
                selected.push('\n');
                count += 1;
            }
            // A validated cell selection stays in the single resolved leaf. Only
            // body selections can span other paragraphs in the same section.
            let text = if index == focus.paragraph {
                &paragraph.text
            } else {
                &core.document().sections[from.section as usize].paragraphs[index as usize].text
            };
            let left = if index == from.paragraph {
                from.scalar_offset as usize
            } else {
                0
            };
            let right = if index == to.paragraph {
                to.scalar_offset as usize
            } else {
                text.chars().count()
            };
            for ch in text.chars().skip(left).take(right - left) {
                if count == NATIVE_IME_CONTEXT_SCALARS {
                    truncated = true;
                    break 'paragraphs;
                }
                selected.push(ch);
                count += 1;
            }
        }
        Ok(NativeImeContext {
            text,
            cursor_scalar: focus.scalar_offset - start as u32,
            anchor_scalar,
            selected_text: selected,
            absolute_start_scalar: start as u32,
            position: focus.clone(),
            text_truncated: start > 0 || end < length,
            selection_truncated: truncated,
        })
    }
}
