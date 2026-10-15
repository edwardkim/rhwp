//! Body editing commands over scalar addresses and extended Unicode graphemes.
//!
//! Left/right are logical text order, not visual bidi navigation. Home/End mean
//! paragraph boundaries; Up/Down consume the existing core visual-line query.
//! SelectAll selects the current body section, never cells or other sections.
//! Hosts must commit/cancel composition before issuing these commands.

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::document_core::native_interaction::{
    NativeTextPosition, NativeTextSelection, NativeVerticalDirection,
};

use super::editor::{NativeEditOutcome, NativeEditor};
use super::NativeHostError;

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
    /// Delete the scalar selection, or one complete grapheme at a caret.
    /// At the paragraph start, join with the preceding body paragraph.
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
                target.scalar_offset = scalar_len(self.command_text(&target))?;
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
                self.join_at(expected_revision, self.selection().focus.paragraph)?
            }
            NativeEditCommand::SelectAll {} => {
                let mut start = self.selection().focus.clone();
                start.paragraph = 0;
                start.scalar_offset = 0;
                let paragraphs = &self.session().document().document().sections
                    [start.section as usize]
                    .paragraphs;
                let mut end = start.clone();
                end.paragraph = u32::try_from(paragraphs.len() - 1).map_err(|_| {
                    NativeHostError::LimitExceeded("paragraph address range exhausted".into())
                })?;
                end.scalar_offset = scalar_len(&paragraphs[end.paragraph as usize].text)?;
                self.set_selection(
                    expected_revision,
                    NativeTextSelection {
                        anchor: start,
                        focus: end,
                    },
                )?
            }
        };
        Ok(NativeCommandOutcome {
            outcome,
            preferred_x,
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
        self.set_selection(revision, NativeTextSelection { anchor, focus })
    }

    fn command_text(&self, position: &NativeTextPosition) -> &str {
        &self.session().document().document().sections[position.section as usize].paragraphs
            [position.paragraph as usize]
            .text
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
        let text = self.command_text(&target);
        let length = scalar_len(text)?;
        if backward && target.scalar_offset == 0 {
            if target.paragraph > 0 {
                target.paragraph -= 1;
                target.scalar_offset = scalar_len(self.command_text(&target))?;
            }
        } else if !backward && target.scalar_offset == length {
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
        let text = self.command_text(&focus);
        let length = scalar_len(text)?;
        if backward && focus.scalar_offset == 0 {
            return self.join_at(revision, focus.paragraph);
        }
        if !backward && focus.scalar_offset == length {
            let count = self.session().document().document().sections[focus.section as usize]
                .paragraphs
                .len();
            if (focus.paragraph as usize) + 1 == count {
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
        previous.scalar_offset = scalar_len(self.command_text(&previous))?;
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

/// Bounded surrounding text for an OS adapter. Cursor/anchor are scalar offsets
/// inside `text`; the OS adapter converts them to UTF-16 where required.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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
        let focus = &selection.focus;
        let document = self.session().document().document();
        let paragraph =
            &document.sections[focus.section as usize].paragraphs[focus.paragraph as usize];
        let length = paragraph.text.chars().count();
        let start = (focus.scalar_offset as usize).saturating_sub(2048);
        let end = length.min(start.saturating_add(4096));
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
                if count == 4096 {
                    truncated = true;
                    break;
                }
                selected.push('\n');
                count += 1;
            }
            let text = &document.sections[from.section as usize].paragraphs[index as usize].text;
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
                if count == 4096 {
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
