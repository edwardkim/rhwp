use std::cmp::Ordering;

use crate::document_core::native_interaction::{NativeCellPathEntry, NativeTextSelection};
use crate::document_core::TABLE_CAPTION_CELL_SENTINEL;
use crate::model::control::Control;

use super::{NativeEditor, NativeHostError, NativeTableCellDirection};

impl NativeEditor {
    pub(super) fn table_cell_selection(
        &self,
        direction: NativeTableCellDirection,
    ) -> Result<NativeTextSelection, NativeHostError> {
        let selection = self.selection();
        let mut parent = selection.focus.clone();
        let leaf = parent.cell_path.pop().ok_or_else(|| {
            NativeHostError::Unsupported("table cell navigation requires a cell selection".into())
        })?;
        parent.scalar_offset = 0;

        let core = self.session().document();
        // The current body or enclosing cell paragraph owns the innermost table.
        // An editable check on the parent still permits leaving a protected leaf.
        let owner = core.native_editable_text_paragraph(&parent)?;
        let Some(Control::Table(table)) = owner.controls.get(leaf.control as usize) else {
            return Err(NativeHostError::Unsupported(
                "table cell navigation requires an owning table".into(),
            ));
        };
        if table.common.locked {
            return Err(NativeHostError::Unsupported(
                "table cell navigation requires an unlocked table".into(),
            ));
        }

        let source = &table.cells[leaf.cell as usize];
        let source_anchor = (source.row, source.col);
        let step = match direction {
            NativeTableCellDirection::Next => Ordering::Greater,
            NativeTableCellDirection::Previous => Ordering::Less,
        };
        // Use one stored anchor per real cell. Grid entries covered by merged
        // cells would otherwise visit that same cell multiple times.
        let candidates = table.cells.iter().enumerate().filter_map(|(index, cell)| {
            let anchor = (cell.row, cell.col);
            if anchor.cmp(&source_anchor) != step
                || cell.row >= table.row_count
                || cell.col >= table.col_count
                || index == TABLE_CAPTION_CELL_SENTINEL
                || cell.cell_protect()
            {
                return None;
            }
            let paragraph = cell
                .paragraphs
                .iter()
                .position(|paragraph| paragraph.controls.is_empty())?;
            Some((anchor, index, paragraph))
        });
        let candidate = match direction {
            NativeTableCellDirection::Next => candidates.min(),
            NativeTableCellDirection::Previous => candidates.max(),
        };
        let Some((_, cell, paragraph)) = candidate else {
            return Ok(selection.clone());
        };

        let to_u32 = |index| {
            u32::try_from(index).map_err(|_| {
                NativeHostError::LimitExceeded("table cell address range exhausted".into())
            })
        };
        let mut point = parent;
        point.cell_path.push(NativeCellPathEntry {
            control: leaf.control,
            cell: to_u32(cell)?,
            paragraph: to_u32(paragraph)?,
        });
        Ok(NativeTextSelection {
            anchor: point.clone(),
            focus: point,
        })
    }
}
