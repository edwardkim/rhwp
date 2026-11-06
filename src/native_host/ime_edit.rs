//! Bounded, atomic UTF-16 IME deltas over one plain body paragraph.
//!
//! Context text includes the active native preview. Qt adapters translate their
//! preview-excluding offsets using `preedit`; Flutter deltas use this text as
//! their oldText. Offset conversion never clamps a surrogate or context boundary.

use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::{Composition, HistoryEntry, NativeEditOutcome, NativeEditor};
use crate::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use crate::native_host::accessibility::{native_scalar_to_utf16, native_utf16_to_scalar};
use crate::native_host::commands::{NativeImeContext, NATIVE_IME_CONTEXT_SCALARS};
use crate::native_host::NativeHostError;
use crate::DocumentCore;

pub const MAX_IME_EDIT_BYTES: usize = 16 * 1024;
pub const MAX_IME_EDIT_SCALARS: usize = NATIVE_IME_CONTEXT_SCALARS;

/// Half-open UTF-16 offsets in the context specified by the containing request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeImeUtf16Range {
    pub start: u32,
    pub end: u32,
}

/// Direction-preserving UTF-16 selection; offsets refer to the resulting text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeImeUtf16Selection {
    pub anchor: u32,
    pub focus: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeImeEditContext {
    pub revision: u64,
    pub font_generation: u64,
    pub context: NativeImeContext,
    pub selection: NativeTextSelection,
    pub composition_id: Option<u64>,
    /// The entire active preview inside `context.text`, including an unchanged
    /// original selection when beginComposition has not received a preview yet.
    pub preedit: Option<NativeImeUtf16Range>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeImeReplacement {
    pub range: NativeImeUtf16Range,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeImePreedit {
    /// Range in the committed result, before this new preview is inserted.
    pub replacement: NativeImeReplacement,
    /// Selection in the final result; both ends must lie inside the new preview.
    pub selection: Option<NativeImeUtf16Selection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NativeImeEdit {
    /// Replace only within the active preview, preserving its original undo and
    /// cancel baseline. Selection refers to the resulting context text.
    Continue {
        replacement: NativeImeReplacement,
        selection: Option<NativeImeUtf16Selection>,
    },
    /// Commit a context replacement and any existing composition as one undo
    /// entry. `selection` is in the committed result. An optional new preedit
    /// has its own cancellation baseline after that commit.
    Commit {
        replacement: NativeImeReplacement,
        selection: Option<NativeImeUtf16Selection>,
        preedit: Option<NativeImePreedit>,
    },
}

struct Replacement {
    range: NativeTextSelection,
    scalars: Range<usize>,
    result: String,
    end: usize,
    changed: bool,
}

struct PlannedPreedit {
    range: NativeTextSelection,
    text: String,
    original: String,
    selection: NativeTextSelection,
    changed: bool,
}

impl NativeEditor {
    pub fn ime_edit_context(
        &self,
        expected_revision: u64,
    ) -> Result<NativeImeEditContext, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        Self::validate_selection(&self.session.core, &self.selection)?;
        same_paragraph(&self.selection)?;
        Self::check_body_text_span(&self.session.core, &self.selection)?;
        let context = self.ime_context(expected_revision)?;
        if context.anchor_scalar.is_none() || context.selection_truncated {
            return Err(limit("IME selection is outside the bounded context"));
        }
        let preedit = if let Some(composition) = &self.composition {
            same_paragraph(&composition.replacement_range)?;
            check_text_bound(&composition.original_text)?;
            let text = composition
                .preview_text
                .as_deref()
                .unwrap_or(&composition.original_text);
            check_text_bound(text)?;
            let start = Self::ordered(&composition.replacement_range).0;
            if (start.section, start.paragraph)
                != (context.position.section, context.position.paragraph)
            {
                return Err(invalid("composition is outside the IME paragraph"));
            }
            let local = start
                .scalar_offset
                .checked_sub(context.absolute_start_scalar)
                .ok_or_else(|| limit("active preedit is outside the bounded context"))?;
            let end = local
                .checked_add(text.chars().count() as u32)
                .ok_or_else(|| limit("active preedit exceeds scalar addresses"))?;
            let range = NativeImeUtf16Range {
                start: native_scalar_to_utf16(&context.text, local)?,
                end: native_scalar_to_utf16(&context.text, end)?,
            };
            let scalars = scalar_range(&context.text, range)?;
            if scalar_text(&context.text, scalars) != text {
                return Err(invalid("active preview and IME context disagree"));
            }
            Some(range)
        } else {
            None
        };
        Ok(NativeImeEditContext {
            revision: self.session.revision(),
            font_generation: self.session.font_generation(),
            context,
            selection: self.selection.clone(),
            composition_id: self.composition_id(),
            preedit,
        })
    }

    pub fn apply_ime_edit(
        &mut self,
        expected_revision: u64,
        expected: &NativeImeEditContext,
        edit: NativeImeEdit,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        // Reject large untrusted strings before counting scalars or comparing a
        // caller's entire context. The exact current snapshot remains authority.
        check_text_bound(&expected.context.text)?;
        check_text_bound(&expected.context.selected_text)?;
        Self::validate_selection(&self.session.core, &expected.selection)?;
        if expected != &self.ime_edit_context(expected_revision)? {
            return Err(invalid(
                "stale IME context, selection or composition identity",
            ));
        }
        match edit {
            NativeImeEdit::Continue {
                replacement,
                selection,
            } => self.continue_ime_edit(expected, replacement, selection),
            NativeImeEdit::Commit {
                replacement,
                selection,
                preedit,
            } => self.commit_ime_edit(expected, replacement, selection, preedit),
        }
    }

    fn continue_ime_edit(
        &mut self,
        expected: &NativeImeEditContext,
        replacement: NativeImeReplacement,
        selection: Option<NativeImeUtf16Selection>,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        check_input(&replacement.text, None)?;
        let preview_range = expected
            .preedit
            .ok_or_else(|| invalid("no active IME composition"))?;
        let preview_scalars = scalar_range(&expected.context.text, preview_range)?;
        let replacement = plan_replacement(expected, &expected.context.text, &replacement)?;
        if replacement.scalars.start < preview_scalars.start
            || replacement.scalars.end > preview_scalars.end
        {
            return Err(invalid(
                "continuing IME delta must stay inside its active preedit",
            ));
        }
        let removed = replacement.scalars.end - replacement.scalars.start;
        let inserted = replacement.end - replacement.scalars.start;
        let preview_end = preview_scalars.end - removed + inserted;
        let preview = scalar_text(&replacement.result, preview_scalars.start..preview_end);
        let selected = plan_selection(expected, &replacement.result, selection, preview_end)?;
        check_preview_selection(expected, &selected, preview_scalars.start..preview_end)?;
        self.check_next_ime_window(
            expected,
            &replacement.result,
            &selected,
            Some(preview_scalars.start..preview_end),
        )?;
        let composition = self
            .composition
            .as_ref()
            .expect("validated composition context");
        let baseline = composition.before;
        let range = composition.replacement_range.clone();
        let changed = preview != composition.original_text;
        let mutation = replacement.changed;
        let (_, rollback) = self.ime_transaction(usize::from(mutation), mutation, |core, _| {
            if mutation {
                core.restore_snapshot_native(baseline)?;
                if changed {
                    Self::apply_replacement(core, &range, &preview)?;
                }
            }
            Ok(())
        })?;
        if let Some(rollback) = rollback {
            self.session.core.discard_snapshot_native(rollback);
        }
        self.selection = selected;
        let composition = self
            .composition
            .as_mut()
            .expect("validated composition context");
        composition.preview_text = Some(preview);
        composition.changed = changed;
        Ok(self.outcome(mutation, false))
    }

    fn commit_ime_edit(
        &mut self,
        expected: &NativeImeEditContext,
        replacement: NativeImeReplacement,
        selection: Option<NativeImeUtf16Selection>,
        preedit: Option<NativeImePreedit>,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        check_input(
            &replacement.text,
            preedit
                .as_ref()
                .map(|preview| preview.replacement.text.as_str()),
        )?;
        let committed = plan_replacement(expected, &expected.context.text, &replacement)?;
        let committed_selection =
            plan_selection(expected, &committed.result, selection, committed.end)?;
        self.check_next_ime_window(expected, &committed.result, &committed_selection, None)?;
        let baseline = if let Some(composition) = &self.composition {
            let range = scalar_range(
                &expected.context.text,
                expected.preedit.expect("composition context"),
            )?;
            replace_scalars(&expected.context.text, range, &composition.original_text)
        } else {
            expected.context.text.clone()
        };
        let history_changed = committed.result != baseline;
        let restore_baseline =
            !history_changed && self.composition.as_ref().is_some_and(|value| value.changed);
        let preview = preedit
            .map(|preview| {
                let replacement =
                    plan_replacement(expected, &committed.result, &preview.replacement)?;
                // The next context query validates this cancellation baseline
                // too. A short preview can replace a span larger than either
                // bounded input after the commit has expanded the context.
                let original = scalar_text(&committed.result, replacement.scalars.clone());
                check_text_bound(&original)?;
                let selected = plan_selection(
                    expected,
                    &replacement.result,
                    preview.selection,
                    replacement.end,
                )?;
                let span = replacement.scalars.start..replacement.end;
                check_preview_selection(expected, &selected, span.clone())?;
                self.check_next_ime_window(expected, &replacement.result, &selected, Some(span))?;
                Ok::<_, NativeHostError>(PlannedPreedit {
                    range: replacement.range,
                    original,
                    text: preview.replacement.text,
                    selection: selected,
                    changed: replacement.changed,
                })
            })
            .transpose()?;
        let next_id = if preview.is_some() {
            Some(
                self.next_composition_id
                    .checked_add(1)
                    .ok_or_else(|| limit("composition identifiers exhausted"))?,
            )
        } else {
            None
        };
        let mutation = committed.changed
            || restore_baseline
            || preview.as_ref().is_some_and(|value| value.changed);
        let slots =
            usize::from(mutation) + usize::from(history_changed) + usize::from(preview.is_some());
        let old_before = self.composition.as_ref().map(|value| value.before);
        let undo_selection = self
            .composition
            .as_ref()
            .map(|value| value.selection_before.clone())
            .unwrap_or_else(|| self.selection.clone());
        let ((after, preview_before), rollback) =
            self.ime_transaction(slots, mutation, |core, allocated| {
                if restore_baseline {
                    core.restore_snapshot_native(old_before.expect("restoring composition"))?;
                } else if committed.changed {
                    Self::apply_replacement(core, &committed.range, &replacement.text)?;
                }
                let after = history_changed.then(|| retain_snapshot(core, allocated));
                let preview_before = preview.as_ref().map(|_| retain_snapshot(core, allocated));
                if let Some(preview) = &preview {
                    if preview.changed {
                        Self::apply_replacement(core, &preview.range, &preview.text)?;
                    }
                }
                Ok((after, preview_before))
            })?;
        // Every fallible operation precedes publication of editor-owned state.
        let old = self.composition.take();
        if history_changed {
            self.record(HistoryEntry {
                before: old_before.or(rollback).expect("changing action baseline"),
                after: after.expect("changing action snapshot"),
                selection_before: undo_selection,
                selection_after: committed_selection.clone(),
            });
        } else if let Some(old) = &old {
            self.session.core.discard_snapshot_native(old.before);
        }
        if let Some(rollback) = rollback {
            if old_before.is_some() || !history_changed {
                self.session.core.discard_snapshot_native(rollback);
            }
        }
        self.selection = committed_selection.clone();
        if let Some(preview) = preview {
            self.composition = Some(Composition {
                id: self.next_composition_id,
                before: preview_before.expect("new composition baseline"),
                selection_before: committed_selection,
                replacement_range: preview.range,
                original_text: preview.original,
                preview_text: Some(preview.text),
                changed: preview.changed,
            });
            self.next_composition_id = next_id.expect("new composition identifier");
            self.selection = preview.selection;
        }
        Ok(self.outcome(mutation, history_changed))
    }

    fn check_next_ime_window(
        &self,
        expected: &NativeImeEditContext,
        result: &str,
        selection: &NativeTextSelection,
        preview: Option<Range<usize>>,
    ) -> Result<(), NativeHostError> {
        let position = &expected.context.position;
        let paragraph = &self.session.core.document().sections[position.section as usize]
            .paragraphs[position.paragraph as usize]
            .text;
        let length = paragraph
            .chars()
            .count()
            .checked_sub(expected.context.text.chars().count())
            .and_then(|value| value.checked_add(result.chars().count()))
            .ok_or_else(|| limit("IME paragraph length overflow"))?;
        if u32::try_from(length).is_err() {
            return Err(limit("IME paragraph exceeds native scalar addresses"));
        }
        let start =
            (selection.focus.scalar_offset as usize).saturating_sub(NATIVE_IME_CONTEXT_SCALARS / 2);
        let end = length.min(start.saturating_add(NATIVE_IME_CONTEXT_SCALARS));
        let contains = |offset: u32| (start..=end).contains(&(offset as usize));
        if !contains(selection.anchor.scalar_offset) || !contains(selection.focus.scalar_offset) {
            return Err(limit(
                "resulting IME selection exceeds the next bounded context",
            ));
        }
        if let Some(preview) = preview {
            if !contains(native_position(expected, preview.start)?.scalar_offset)
                || !contains(native_position(expected, preview.end)?.scalar_offset)
            {
                return Err(limit("resulting preedit exceeds the next bounded context"));
            }
        }
        Ok(())
    }

    /// The core event log remains append-only, matching edit_transaction. Only
    /// new snapshots are discarded on error; old composition/history/save state
    /// is untouched, and the native revision is advanced once after success.
    fn ime_transaction<T>(
        &mut self,
        snapshot_slots: usize,
        mutation: bool,
        edit: impl FnOnce(&mut DocumentCore, &mut Vec<u32>) -> Result<T, NativeHostError>,
    ) -> Result<(T, Option<u32>), NativeHostError> {
        self.check_snapshot_slots(snapshot_slots)?;
        if mutation {
            self.check_mutation_capacity()?;
        }
        let rollback = mutation.then(|| self.session.core.save_snapshot_native());
        let mut allocated = Vec::new();
        let result = edit(&mut self.session.core, &mut allocated).and_then(|result| {
            if mutation {
                self.session.did_mutate()?;
            }
            Ok(result)
        });
        match result {
            Ok(result) => Ok((result, rollback)),
            Err(error) => {
                for snapshot in allocated {
                    self.session.core.discard_snapshot_native(snapshot);
                }
                Err(match rollback {
                    Some(snapshot) => self.rollback_and_discard(snapshot, error),
                    None => error,
                })
            }
        }
    }
}

fn retain_snapshot(core: &mut DocumentCore, allocated: &mut Vec<u32>) -> u32 {
    let id = core.save_snapshot_native();
    allocated.push(id);
    id
}

fn same_paragraph(selection: &NativeTextSelection) -> Result<(), NativeHostError> {
    if selection.anchor.section != selection.focus.section
        || selection.anchor.paragraph != selection.focus.paragraph
        || !selection.anchor.cell_path.is_empty()
        || !selection.focus.cell_path.is_empty()
    {
        return Err(NativeHostError::Unsupported(
            "IME deltas require one plain body paragraph".into(),
        ));
    }
    Ok(())
}

fn check_text_bound(text: &str) -> Result<(), NativeHostError> {
    if text.len() > MAX_IME_EDIT_BYTES || text.chars().count() > MAX_IME_EDIT_SCALARS {
        return Err(limit("IME text exceeds the bounded input limit"));
    }
    Ok(())
}

fn check_input(text: &str, preview: Option<&str>) -> Result<(), NativeHostError> {
    if text.len().saturating_add(preview.map_or(0, str::len)) > MAX_IME_EDIT_BYTES {
        return Err(limit(
            "combined IME commit and preedit exceed the input limit",
        ));
    }
    check_text_bound(text)?;
    NativeEditor::validate_text(text)?;
    if let Some(preview) = preview {
        check_text_bound(preview)?;
        NativeEditor::validate_text(preview)?;
        if text.len() + preview.len() > MAX_IME_EDIT_BYTES
            || text.chars().count() + preview.chars().count() > MAX_IME_EDIT_SCALARS
        {
            return Err(limit(
                "combined IME commit and preedit exceed the input limit",
            ));
        }
    }
    Ok(())
}

fn scalar_range(text: &str, range: NativeImeUtf16Range) -> Result<Range<usize>, NativeHostError> {
    if range.start > range.end {
        return Err(invalid("IME replacement range is reversed"));
    }
    Ok(native_utf16_to_scalar(text, range.start)? as usize
        ..native_utf16_to_scalar(text, range.end)? as usize)
}

fn scalar_text(text: &str, range: Range<usize>) -> String {
    text.chars()
        .skip(range.start)
        .take(range.end - range.start)
        .collect()
}

fn replace_scalars(text: &str, range: Range<usize>, replacement: &str) -> String {
    let mut result: String = text.chars().take(range.start).collect();
    result.push_str(replacement);
    result.extend(text.chars().skip(range.end));
    result
}

fn native_position(
    expected: &NativeImeEditContext,
    local: usize,
) -> Result<NativeTextPosition, NativeHostError> {
    let mut position = expected.context.position.clone();
    position.scalar_offset = expected
        .context
        .absolute_start_scalar
        .checked_add(u32::try_from(local).map_err(|_| limit("IME scalar position overflow"))?)
        .ok_or_else(|| limit("IME scalar position overflow"))?;
    Ok(position)
}

fn plan_replacement(
    expected: &NativeImeEditContext,
    context: &str,
    replacement: &NativeImeReplacement,
) -> Result<Replacement, NativeHostError> {
    let scalars = scalar_range(context, replacement.range)?;
    let changed = scalar_text(context, scalars.clone()) != replacement.text;
    let result = replace_scalars(context, scalars.clone(), &replacement.text);
    let end = scalars.start + replacement.text.chars().count();
    Ok(Replacement {
        range: NativeTextSelection {
            anchor: native_position(expected, scalars.start)?,
            focus: native_position(expected, scalars.end)?,
        },
        scalars,
        result,
        end,
        changed,
    })
}

fn plan_selection(
    expected: &NativeImeEditContext,
    result: &str,
    selection: Option<NativeImeUtf16Selection>,
    default: usize,
) -> Result<NativeTextSelection, NativeHostError> {
    let (anchor, focus) = if let Some(selection) = selection {
        (
            native_utf16_to_scalar(result, selection.anchor)? as usize,
            native_utf16_to_scalar(result, selection.focus)? as usize,
        )
    } else {
        (default, default)
    };
    Ok(NativeTextSelection {
        anchor: native_position(expected, anchor)?,
        focus: native_position(expected, focus)?,
    })
}

fn check_preview_selection(
    expected: &NativeImeEditContext,
    selection: &NativeTextSelection,
    preview: Range<usize>,
) -> Result<(), NativeHostError> {
    let start = native_position(expected, preview.start)?.scalar_offset;
    let end = native_position(expected, preview.end)?.scalar_offset;
    if [
        selection.anchor.scalar_offset,
        selection.focus.scalar_offset,
    ]
    .into_iter()
    .any(|value| value < start || value > end)
    {
        return Err(invalid("IME selection must remain inside its preedit"));
    }
    Ok(())
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}

fn limit(message: &str) -> NativeHostError {
    NativeHostError::LimitExceeded(message.into())
}
