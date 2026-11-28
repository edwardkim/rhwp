use crate::document_core::native_interaction::{
    legacy_cell_path, NativeTextPosition, NativeTextSelection,
};
use crate::model::paragraph::Paragraph;
use crate::DocumentCore;

use super::{scalar_len, NativeEditOutcome, NativeEditor, NativeHostError};

impl NativeEditor {
    pub(super) fn split_cell_selection(
        &mut self,
        revision: u64,
        selection: NativeTextSelection,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let (start, end) = selection.ordered();
        if start.leaf_paragraph() != end.leaf_paragraph() {
            return Err(NativeHostError::Unsupported(
                "splitting a cell paragraph requires a selection within that paragraph".into(),
            ));
        }
        check_structural_cell_paragraph(self.session().document(), start)?;
        let path = legacy_cell_path(&start.cell_path);
        let mut after = start.clone();
        let leaf = after.cell_path.last_mut().ok_or_else(cell_required)?;
        leaf.paragraph = leaf.paragraph.checked_add(1).ok_or_else(|| {
            NativeHostError::LimitExceeded("cell paragraph address range exhausted".into())
        })?;
        after.scalar_offset = 0;
        self.edit_transaction(revision, |core| {
            if start != end {
                Self::apply_replacement(core, &selection, "")?;
            }
            core.split_paragraph_in_cell_by_path(
                start.section as usize,
                start.paragraph as usize,
                &path,
                start.scalar_offset as usize,
                None,
            )?;
            Ok(NativeTextSelection {
                anchor: after.clone(),
                focus: after,
            })
        })
    }

    pub(super) fn join_previous_cell_paragraph(
        &mut self,
        revision: u64,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let focus = self.selection().focus.clone();
        let mut previous = focus.clone();
        let leaf = previous.cell_path.last_mut().ok_or_else(cell_required)?;
        if leaf.paragraph == 0 {
            return self.set_selection(revision, self.selection().clone());
        }
        leaf.paragraph -= 1;
        previous.scalar_offset = 0;
        let core = self.session().document();
        check_structural_cell_paragraph(core, &focus)?;
        let paragraph = check_structural_cell_paragraph(core, &previous)?;
        previous.scalar_offset = scalar_len(&paragraph.text)?;
        let path = legacy_cell_path(&focus.cell_path);
        self.edit_transaction(revision, |core| {
            core.merge_paragraph_in_cell_by_path(
                focus.section as usize,
                focus.paragraph as usize,
                &path,
            )?;
            Ok(NativeTextSelection {
                anchor: previous.clone(),
                focus: previous,
            })
        })
    }
}

fn cell_required() -> NativeHostError {
    NativeHostError::Unsupported("cell paragraph commands require a table-cell position".into())
}

fn check_structural_cell_paragraph<'a>(
    core: &'a DocumentCore,
    position: &NativeTextPosition,
) -> Result<&'a Paragraph, NativeHostError> {
    let paragraph = core.native_editable_text_paragraph(position)?;
    if !paragraph.controls.is_empty()
        || !paragraph.field_ranges.is_empty()
        || !paragraph.orphan_field_ends.is_empty()
    {
        return Err(NativeHostError::Unsupported(
            "cell paragraph commands require text without semantic controls or field boundaries"
                .into(),
        ));
    }
    Ok(paragraph)
}
