//! Opt-in native editor state. The document engine itself remains history-free.
//!
//! This first surface edits body text only. Offsets are Unicode scalar indices;
//! cell editing, object-bearing paragraph mutations, inline-control affinity
//! and tabs are rejected explicitly. Read-only selection may span floating objects.
//! Paragraph split/join and grapheme navigation live in `commands`. OS IME events
//! and screen coordinates belong to the host, not to this session.

use std::collections::VecDeque;

use crate::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use crate::model::control::Control;
use crate::DocumentCore;

use super::{NativeDocumentSession, NativeHostError};

const DEFAULT_HISTORY_CAPACITY: usize = 16;

/// The host must choose what losing input focus does to an active composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeCompositionFocusLoss {
    Commit,
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeEditOutcome {
    pub revision: u64,
    pub selection: NativeTextSelection,
    /// This call changed document contents, including an IME preview or rollback.
    pub document_changed: bool,
    /// This call changed undo/redo history; committing a preview need not repaint.
    pub history_changed: bool,
    /// Pass this identifier back with IME events to reject late events from a
    /// previous composition even when the document revision has not changed.
    pub composition_id: Option<u64>,
}

struct HistoryEntry {
    before: u32,
    after: u32,
    selection_before: NativeTextSelection,
    selection_after: NativeTextSelection,
}

struct Composition {
    id: u64,
    before: u32,
    selection_before: NativeTextSelection,
    original_text: String,
    preview_text: Option<String>,
    changed: bool,
}

/// A single owner of document edits, selection, composition and bounded history.
///
/// A snapshot pair is deliberately retained for each committed action in this
/// initial API. The capacity bounds snapshot count, not document byte size.
/// Existing snapshots owned by the caller are preserved. Two free slots cover
/// either a new action's before/after pair or composition plus rollback state.
/// The core event log keeps its existing semantics; this is not a total-memory
/// budget or a promise to rewind previously emitted events on undo.
pub struct NativeEditor {
    session: NativeDocumentSession,
    selection: NativeTextSelection,
    undo: VecDeque<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    history_capacity: usize,
    composition: Option<Composition>,
    next_composition_id: u64,
}

impl NativeEditor {
    pub fn new(
        session: NativeDocumentSession,
        selection: NativeTextSelection,
    ) -> Result<Self, NativeHostError> {
        Self::try_new(session, selection).map_err(|failed| {
            let (_, error) = *failed;
            error
        })
    }

    /// Enter editing without losing the viewer session when its selected target
    /// is unsupported. The returned session is unchanged on every error path.
    /// Hosts that retain an open document should use this instead of `new`.
    pub fn try_new(
        session: NativeDocumentSession,
        selection: NativeTextSelection,
    ) -> Result<Self, Box<(NativeDocumentSession, NativeHostError)>> {
        let available = Self::available_history_capacity(&session);
        Self::try_with_history_capacity(session, selection, DEFAULT_HISTORY_CAPACITY.min(available))
    }

    pub fn with_history_capacity(
        session: NativeDocumentSession,
        selection: NativeTextSelection,
        history_capacity: usize,
    ) -> Result<Self, NativeHostError> {
        Self::try_with_history_capacity(session, selection, history_capacity).map_err(|failed| {
            let (_, error) = *failed;
            error
        })
    }

    fn try_with_history_capacity(
        session: NativeDocumentSession,
        selection: NativeTextSelection,
        history_capacity: usize,
    ) -> Result<Self, Box<(NativeDocumentSession, NativeHostError)>> {
        if session.core.batch_mode {
            return Err(Box::new((
                session,
                NativeHostError::Unsupported(
                    "finish the document batch before opening a native editor".into(),
                ),
            )));
        }
        if let Err(error) = Self::validate_selection(&session.core, &selection) {
            return Err(Box::new((session, error)));
        }
        let available = Self::available_history_capacity(&session);
        if history_capacity == 0 || history_capacity > available {
            return Err(Box::new((
                session,
                NativeHostError::LimitExceeded(format!(
                    "native editor history capacity must be in 1..={available}"
                )),
            )));
        }
        Ok(Self {
            session,
            selection,
            undo: VecDeque::new(),
            redo: Vec::new(),
            history_capacity,
            composition: None,
            next_composition_id: 1,
        })
    }

    pub fn session(&self) -> &NativeDocumentSession {
        &self.session
    }

    /// Keep history and selection while replacing fonts; finish IME first.
    pub fn set_font_sources(
        &mut self,
        expected_revision: u64,
        paths: &[std::path::PathBuf],
        environment: Option<crate::renderer::font_environment::FontEnvironment>,
    ) -> Result<bool, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        self.session.set_font_sources(paths, environment)
    }

    /// Save without committing an unfinished IME preview. HWP stores current
    /// body focus; HWPX retains its existing source-settings/default policy.
    pub fn prepare_save(
        &mut self,
        format: super::save::NativeSaveFormat,
    ) -> Result<super::save::PreparedSave, NativeHostError> {
        self.require_no_composition()?;
        match format {
            super::save::NativeSaveFormat::Hwp => self
                .session
                .prepare_save_with_caret(format, &self.selection.focus),
            super::save::NativeSaveFormat::Hwpx => self.session.prepare_save(format),
        }
    }

    pub fn acknowledge_save(
        &mut self,
        token: super::save::NativeSaveToken,
    ) -> Result<super::save::NativeSaveStatus, NativeHostError> {
        self.session.acknowledge_save(token)
    }

    pub fn cancel_save(
        &mut self,
        token: super::save::NativeSaveToken,
    ) -> Result<super::save::NativeSaveStatus, NativeHostError> {
        self.session.cancel_save(token)
    }

    pub fn selection(&self) -> &NativeTextSelection {
        &self.selection
    }

    pub fn can_undo(&self) -> bool {
        self.composition.is_none() && !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        self.composition.is_none() && !self.redo.is_empty()
    }

    pub fn history_capacity(&self) -> usize {
        self.history_capacity
    }

    pub fn composition_id(&self) -> Option<u64> {
        self.composition.as_ref().map(|composition| composition.id)
    }

    /// Return the session after releasing this editor's undo resources. An active
    /// composition returns the editor intact so the host can commit or cancel it.
    pub fn into_session(mut self) -> Result<NativeDocumentSession, Box<Self>> {
        if self.composition.is_some() {
            return Err(Box::new(self));
        }
        self.discard_history();
        Ok(self.session)
    }

    pub fn set_selection(
        &mut self,
        expected_revision: u64,
        selection: NativeTextSelection,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        Self::validate_selection(&self.session.core, &selection)?;
        self.selection = selection;
        Ok(self.outcome(false, false))
    }

    /// Insert at a collapsed selection, delete with an empty string, or replace a
    /// range. The selected body paragraphs may span one section. Newline and tab
    /// input are not silently interpreted as paragraph/control insertion.
    pub fn replace_selection(
        &mut self,
        expected_revision: u64,
        text: &str,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let selection = self.selection.clone();
        self.replace_range(expected_revision, &selection, text)
    }

    /// Shared scalar range replacement for editor commands and host adapters.
    /// The undo selection is the caller's actual selection, not this edit range.
    pub(super) fn replace_range(
        &mut self,
        expected_revision: u64,
        selection: &NativeTextSelection,
        text: &str,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.check_edit_ready(expected_revision)?;
        Self::validate_text(text)?;
        Self::validate_selection(&self.session.core, selection)?;
        let selection_after = Self::replacement_end(selection, text)?;
        if self.selected_text(selection) == text {
            self.selection = selection_after;
            return Ok(self.outcome(false, false));
        }
        self.edit_transaction(expected_revision, |core| {
            Self::apply_replacement(core, selection, text)?;
            Ok(selection_after)
        })
    }

    pub(super) fn check_edit_ready(&self, expected_revision: u64) -> Result<(), NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        Self::validate_selection(&self.session.core, &self.selection)
    }

    /// Commit one known-changing action. The closure can mutate only the core;
    /// editor selection/history are published after every fallible step succeeds.
    /// This does not rewind the core's already-emitted event log on rollback.
    pub(super) fn edit_transaction(
        &mut self,
        expected_revision: u64,
        edit: impl FnOnce(&mut DocumentCore) -> Result<NativeTextSelection, NativeHostError>,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.check_edit_ready(expected_revision)?;
        self.check_mutation_capacity()?;
        self.check_snapshot_slots(2)?;
        let selection_before = self.selection.clone();
        let before = self.session.core.save_snapshot_native();
        let result = edit(&mut self.session.core).and_then(|selection| {
            Self::validate_selection(&self.session.core, &selection)?;
            Ok(selection)
        });
        let selection_after = match result {
            Ok(selection) => selection,
            Err(error) => return Err(self.rollback_and_discard(before, error)),
        };
        if let Err(error) = self.session.did_mutate() {
            return Err(self.rollback_and_discard(before, error));
        }
        let after = self.session.core.save_snapshot_native();
        self.selection = selection_after.clone();
        self.record(HistoryEntry {
            before,
            after,
            selection_before,
            selection_after,
        });
        Ok(self.outcome(true, true))
    }

    pub fn begin_composition(
        &mut self,
        expected_revision: u64,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        Self::validate_selection(&self.session.core, &self.selection)?;
        self.check_snapshot_slots(2)?;
        let next_id = self.next_composition_id.checked_add(1).ok_or_else(|| {
            NativeHostError::LimitExceeded("composition identifiers exhausted".into())
        })?;
        let original_text = self.selected_text(&self.selection);
        let before = self.session.core.save_snapshot_native();
        self.composition = Some(Composition {
            id: self.next_composition_id,
            before,
            selection_before: self.selection.clone(),
            original_text,
            preview_text: None,
            changed: false,
        });
        self.next_composition_id = next_id;
        Ok(self.outcome(false, false))
    }

    /// Preview from the original selection each time. This preserves formatting
    /// on composition cancellation and never records intermediate Korean jamo.
    pub fn update_composition(
        &mut self,
        expected_revision: u64,
        composition_id: u64,
        text: &str,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        let composition = self.require_composition(composition_id)?;
        Self::validate_text(text)?;
        if composition.preview_text.as_deref() == Some(text) {
            return Ok(self.outcome(false, false));
        }
        let selection_before = composition.selection_before.clone();
        let selection_after = Self::replacement_end(&selection_before, text)?;
        let before = composition.before;
        let changed = text != composition.original_text;
        let needs_mutation = changed || composition.changed;
        if needs_mutation {
            self.check_mutation_capacity()?;
            self.check_snapshot_slots(1)?;
            let rollback = self.session.core.save_snapshot_native();
            let result = self
                .session
                .core
                .restore_snapshot_native(before)
                .map_err(NativeHostError::from)
                .and_then(|_| {
                    if changed {
                        Self::apply_replacement(&mut self.session.core, &selection_before, text)
                    } else {
                        Ok(())
                    }
                })
                .and_then(|_| self.session.did_mutate().map(|_| ()));
            if let Err(error) = result {
                return Err(self.rollback_and_discard(rollback, error));
            }
            self.session.core.discard_snapshot_native(rollback);
        }
        self.selection = selection_after;
        let composition = self.composition.as_mut().expect("validated composition");
        composition.preview_text = Some(text.to_owned());
        composition.changed = changed;
        Ok(self.outcome(needs_mutation, false))
    }

    /// Record already-applied preedit exactly once. The host sends its final
    /// committed text through update_composition before calling this method.
    pub fn commit_composition(
        &mut self,
        expected_revision: u64,
        composition_id: u64,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        let changed = self.require_composition(composition_id)?.changed;
        if changed {
            self.check_snapshot_slots(1)?;
        }
        let composition = self.composition.take().expect("validated composition");
        if changed {
            let after = self.session.core.save_snapshot_native();
            self.record(HistoryEntry {
                before: composition.before,
                after,
                selection_before: composition.selection_before,
                selection_after: self.selection.clone(),
            });
        } else {
            self.session
                .core
                .discard_snapshot_native(composition.before);
        }
        Ok(self.outcome(false, changed))
    }

    pub fn cancel_composition(
        &mut self,
        expected_revision: u64,
        composition_id: u64,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        let composition = self.require_composition(composition_id)?;
        let changed = composition.changed;
        let before = composition.before;
        if changed {
            self.check_mutation_capacity()?;
            self.session.core.restore_snapshot_native(before)?;
            self.session.did_mutate()?;
        }
        let composition = self.composition.take().expect("validated composition");
        self.selection = composition.selection_before;
        self.session.core.discard_snapshot_native(before);
        Ok(self.outcome(changed, false))
    }

    pub fn focus_lost(
        &mut self,
        expected_revision: u64,
        policy: NativeCompositionFocusLoss,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        match (self.composition_id(), policy) {
            (Some(id), NativeCompositionFocusLoss::Commit) => {
                self.commit_composition(expected_revision, id)
            }
            (Some(id), NativeCompositionFocusLoss::Cancel) => {
                self.cancel_composition(expected_revision, id)
            }
            (None, _) => Ok(self.outcome(false, false)),
        }
    }

    pub fn undo(&mut self, expected_revision: u64) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        let Some(entry) = self.undo.back() else {
            return Ok(self.outcome(false, false));
        };
        self.check_mutation_capacity()?;
        self.session.core.restore_snapshot_native(entry.before)?;
        self.session.did_mutate()?;
        let entry = self.undo.pop_back().expect("checked history entry");
        self.selection = entry.selection_before.clone();
        self.redo.push(entry);
        Ok(self.outcome(true, true))
    }

    pub fn redo(&mut self, expected_revision: u64) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        let Some(entry) = self.redo.last() else {
            return Ok(self.outcome(false, false));
        };
        self.check_mutation_capacity()?;
        self.session.core.restore_snapshot_native(entry.after)?;
        self.session.did_mutate()?;
        let entry = self.redo.pop().expect("checked history entry");
        self.selection = entry.selection_after.clone();
        self.undo.push_back(entry);
        Ok(self.outcome(true, true))
    }

    pub fn clear_history(
        &mut self,
        expected_revision: u64,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.session.check_revision(expected_revision)?;
        self.require_no_composition()?;
        let changed = !self.undo.is_empty() || !self.redo.is_empty();
        self.discard_history();
        Ok(self.outcome(false, changed))
    }

    fn available_history_capacity(session: &NativeDocumentSession) -> usize {
        DocumentCore::MAX_SNAPSHOTS.saturating_sub(session.core.snapshot_store.len() + 2) / 2
    }

    fn check_snapshot_slots(&self, count: usize) -> Result<(), NativeHostError> {
        if self.session.core.snapshot_store.len() + count > DocumentCore::MAX_SNAPSHOTS
            || self
                .session
                .core
                .next_snapshot_id
                .checked_add(count as u32)
                .is_none()
        {
            return Err(NativeHostError::LimitExceeded(
                "native editor snapshot budget exhausted".into(),
            ));
        }
        Ok(())
    }

    fn check_mutation_capacity(&self) -> Result<(), NativeHostError> {
        self.session
            .revision()
            .checked_add(1)
            .ok_or_else(|| NativeHostError::LimitExceeded("document revision exhausted".into()))
            .map(|_| ())
    }

    fn require_no_composition(&self) -> Result<(), NativeHostError> {
        if self.composition.is_some() {
            Err(NativeHostError::InvalidArgument(
                "commit or cancel the active composition first".into(),
            ))
        } else {
            Ok(())
        }
    }

    fn require_composition(&self, id: u64) -> Result<&Composition, NativeHostError> {
        self.composition
            .as_ref()
            .filter(|composition| composition.id == id)
            .ok_or_else(|| {
                NativeHostError::InvalidArgument("inactive or stale composition identifier".into())
            })
    }

    fn validate_text(text: &str) -> Result<(), NativeHostError> {
        if text.chars().any(char::is_control) {
            return Err(NativeHostError::Unsupported(
                "body text input does not insert control characters, tabs or paragraphs".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn validate_selection(
        core: &DocumentCore,
        selection: &NativeTextSelection,
    ) -> Result<(), NativeHostError> {
        if !selection.anchor.cell_path.is_empty() || !selection.focus.cell_path.is_empty() {
            return Err(NativeHostError::Unsupported(
                "native editor currently supports body text only; cell editing is not supported"
                    .into(),
            ));
        }
        for position in [&selection.anchor, &selection.focus] {
            core.validate_native_text_position(position)
                .map_err(|error| NativeHostError::InvalidArgument(error.to_string()))?;
        }
        if selection.anchor.section != selection.focus.section {
            return Err(NativeHostError::Unsupported(
                "cross-section text replacement is not supported".into(),
            ));
        }
        let (start, end) = Self::ordered(selection);
        for paragraph in start.paragraph..=end.paragraph {
            core.check_native_interaction_support(&NativeTextPosition {
                section: start.section,
                paragraph,
                scalar_offset: 0,
                cell_path: Vec::new(),
            })
            .map_err(|error| NativeHostError::Unsupported(error.to_string()))?;
        }
        Ok(())
    }

    pub(super) fn ordered(
        selection: &NativeTextSelection,
    ) -> (&NativeTextPosition, &NativeTextPosition) {
        if (selection.anchor.paragraph, selection.anchor.scalar_offset)
            <= (selection.focus.paragraph, selection.focus.scalar_offset)
        {
            (&selection.anchor, &selection.focus)
        } else {
            (&selection.focus, &selection.anchor)
        }
    }

    /// Scalar body edits cannot preserve semantic controls when removing a
    /// paragraph. Check the full validated span, including middle paragraphs
    /// and floating objects that read-only interaction queries may traverse.
    /// Selection changes and identical-text no-ops do not mutate those controls.
    pub(super) fn check_body_text_span(
        core: &DocumentCore,
        selection: &NativeTextSelection,
    ) -> Result<(), NativeHostError> {
        let (start, end) = Self::ordered(selection);
        for paragraph in &core.document().sections[start.section as usize].paragraphs
            [start.paragraph as usize..=end.paragraph as usize]
        {
            if paragraph.controls.iter().any(|control| {
                // Core text editing preserves the section/column definitions
                // carried by blank documents. Other controls are not plain text.
                !matches!(control, Control::SectionDef(_) | Control::ColumnDef(_))
            }) {
                return Err(NativeHostError::Unsupported(
                    "body text edits cannot include semantic-control paragraphs".into(),
                ));
            }
        }
        Ok(())
    }

    fn replacement_end(
        selection: &NativeTextSelection,
        text: &str,
    ) -> Result<NativeTextSelection, NativeHostError> {
        let (start, _) = Self::ordered(selection);
        let length = u32::try_from(text.chars().count()).map_err(|_| {
            NativeHostError::LimitExceeded("input text exceeds the scalar address range".into())
        })?;
        let mut end = start.clone();
        end.scalar_offset = end.scalar_offset.checked_add(length).ok_or_else(|| {
            NativeHostError::LimitExceeded("input text exceeds the scalar address range".into())
        })?;
        Ok(NativeTextSelection {
            anchor: end.clone(),
            focus: end,
        })
    }

    pub(super) fn selected_text(&self, selection: &NativeTextSelection) -> String {
        let (start, end) = Self::ordered(selection);
        let paragraphs = &self.session.core.document().sections[start.section as usize].paragraphs;
        (start.paragraph..=end.paragraph)
            .map(|paragraph| {
                let text = &paragraphs[paragraph as usize].text;
                let from = if paragraph == start.paragraph {
                    start.scalar_offset as usize
                } else {
                    0
                };
                let to = if paragraph == end.paragraph {
                    end.scalar_offset as usize
                } else {
                    text.chars().count()
                };
                text.chars().skip(from).take(to - from).collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) fn apply_replacement(
        core: &mut DocumentCore,
        selection: &NativeTextSelection,
        text: &str,
    ) -> Result<(), NativeHostError> {
        // IME previews reach this path independently of replace_selection.
        // Reject unsupported spans before either core replacement primitive.
        Self::check_body_text_span(core, selection)?;
        let (start, end) = Self::ordered(selection);
        if start.paragraph == end.paragraph
            && end.scalar_offset - start.scalar_offset <= 8
            && text.chars().count() <= 8
        {
            core.replace_body_text_local_native(
                start.section as usize,
                start.paragraph as usize,
                start.scalar_offset as usize,
                (end.scalar_offset - start.scalar_offset) as usize,
                text,
            )?;
            // Expose a settled document to native geometry/save consumers. This
            // session does not yet own Studio's deferred-pagination scheduler.
            core.paginate_if_needed();
            return Ok(());
        }
        if start != end {
            core.delete_range_native(
                start.section as usize,
                start.paragraph as usize,
                start.scalar_offset as usize,
                end.paragraph as usize,
                end.scalar_offset as usize,
                None,
            )?;
        }
        if !text.is_empty() {
            core.insert_text_native(
                start.section as usize,
                start.paragraph as usize,
                start.scalar_offset as usize,
                text,
            )?;
        }
        Ok(())
    }

    fn rollback_and_discard(
        &mut self,
        snapshot: u32,
        original_error: NativeHostError,
    ) -> NativeHostError {
        let restored = self.session.core.restore_snapshot_native(snapshot);
        self.session.core.discard_snapshot_native(snapshot);
        match restored {
            Ok(_) => original_error,
            Err(error) => NativeHostError::Core(error),
        }
    }

    fn record(&mut self, entry: HistoryEntry) {
        while let Some(entry) = self.redo.pop() {
            self.discard_entry(entry);
        }
        self.undo.push_back(entry);
        while self.undo.len() > self.history_capacity {
            let entry = self.undo.pop_front().expect("history over capacity");
            self.discard_entry(entry);
        }
    }

    fn discard_entry(&mut self, entry: HistoryEntry) {
        self.session.core.discard_snapshot_native(entry.before);
        self.session.core.discard_snapshot_native(entry.after);
    }

    fn discard_history(&mut self) {
        while let Some(entry) = self.undo.pop_front() {
            self.discard_entry(entry);
        }
        while let Some(entry) = self.redo.pop() {
            self.discard_entry(entry);
        }
    }

    fn outcome(&self, document_changed: bool, history_changed: bool) -> NativeEditOutcome {
        NativeEditOutcome {
            revision: self.session.revision(),
            selection: self.selection.clone(),
            document_changed,
            history_changed,
            composition_id: self.composition_id(),
        }
    }
}
