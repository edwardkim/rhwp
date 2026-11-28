//! Top-level table objects addressed via the existing native cell caret.

use serde::{Deserialize, Serialize};

use crate::document_core::native_interaction::{NativeInteractionBounds, NativeTextPosition};
use crate::model::{
    control::Control,
    shape::SizeCriterion,
    table::{Table, MIN_RESIZED_CELL_SIZE},
};
use crate::{DocumentCore, HwpError};

use super::commands::MAX_CREATE_TABLE_CELLS;
use super::editor::{NativeEditOutcome, NativeEditor};
use super::{NativeDocumentSession, NativeHostError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTableObjectAddress {
    pub section: u32,
    pub paragraph: u32,
    pub control: u32,
}

impl NativeTableObjectAddress {
    fn from_cell(position: &NativeTextPosition) -> Result<Self, NativeHostError> {
        let [cell] = position.cell_path.as_slice() else {
            return Err(unsupported(
                "table object commands require a top-level table cell",
            ));
        };
        Ok(Self {
            section: position.section,
            paragraph: position.paragraph,
            control: cell.control,
        })
    }

    fn indices(self) -> (usize, usize, usize) {
        (
            self.section as usize,
            self.paragraph as usize,
            self.control as usize,
        )
    }

    fn resolve(self, core: &DocumentCore) -> Result<&Table, NativeHostError> {
        let (section, paragraph, control) = self.indices();
        let object = core
            .document()
            .sections
            .get(section)
            .and_then(|entry| entry.paragraphs.get(paragraph))
            .and_then(|entry| entry.controls.get(control));
        match object {
            Some(Control::Table(table)) => Ok(table),
            _ => Err(invalid("table address is outside the document")),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTableObjectProperties {
    pub table_width: u32,
    pub table_height: u32,
    pub cell_spacing: i16,
    pub padding_left: i16,
    pub padding_right: i16,
    pub padding_top: i16,
    pub padding_bottom: i16,
    pub outer_left: i16,
    pub outer_right: i16,
    pub outer_top: i16,
    pub outer_bottom: i16,
    pub page_break: u8,
    pub repeat_header: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTableObjectPropertyChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_spacing: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outer_left: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outer_right: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outer_top: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outer_bottom: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_break: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_header: Option<bool>,
}

impl NativeTableObjectPropertyChanges {
    fn changed(self, current: &NativeTableObjectProperties) -> Self {
        Self {
            cell_spacing: self
                .cell_spacing
                .filter(|value| *value != current.cell_spacing),
            padding_left: self
                .padding_left
                .filter(|value| *value != current.padding_left),
            padding_right: self
                .padding_right
                .filter(|value| *value != current.padding_right),
            padding_top: self
                .padding_top
                .filter(|value| *value != current.padding_top),
            padding_bottom: self
                .padding_bottom
                .filter(|value| *value != current.padding_bottom),
            outer_left: self.outer_left.filter(|value| *value != current.outer_left),
            outer_right: self
                .outer_right
                .filter(|value| *value != current.outer_right),
            outer_top: self.outer_top.filter(|value| *value != current.outer_top),
            outer_bottom: self
                .outer_bottom
                .filter(|value| *value != current.outer_bottom),
            page_break: self.page_break.filter(|value| *value != current.page_break),
            repeat_header: self
                .repeat_header
                .filter(|value| *value != current.repeat_header),
        }
    }

    fn validate(&self) -> Result<(), NativeHostError> {
        if self.page_break.is_some_and(|value| value > 2) {
            return Err(invalid(
                "table pageBreak must be 0 (none), 1 (cell) or 2 (row)",
            ));
        }
        if [
            self.cell_spacing,
            self.padding_left,
            self.padding_right,
            self.padding_top,
            self.padding_bottom,
            self.outer_left,
            self.outer_right,
            self.outer_top,
            self.outer_bottom,
        ]
        .into_iter()
        .flatten()
        .any(|value| value < 0)
        {
            return Err(invalid(
                "table spacing and margins must be nonnegative HWPUNIT values",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTableObject {
    pub address: NativeTableObjectAddress,
    pub row_count: u16,
    pub column_count: u16,
    pub bounds: Option<NativeInteractionBounds>,
    pub properties: NativeTableObjectProperties,
    pub editable: bool,
    pub can_resize: bool,
}

impl NativeDocumentSession {
    pub fn table_object(
        &self,
        revision: u64,
        position: &NativeTextPosition,
        page: u32,
    ) -> Result<NativeTableObject, NativeHostError> {
        self.check_revision(revision)?;
        if page >= self.document().page_count() {
            return Err(invalid("table bounds page is outside the document"));
        }
        let address = NativeTableObjectAddress::from_cell(position)?;
        let core = self.document();
        core.validate_native_text_position(position)?;
        let table = address.resolve(core)?;
        let editable = can_edit_table(table);
        let (section, paragraph, control) = address.indices();
        let bounds = core
            .find_top_level_table_bounds(section, paragraph, control, page)?
            .map(|rect| NativeInteractionBounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            });
        Ok(NativeTableObject {
            address,
            row_count: table.row_count,
            column_count: table.col_count,
            bounds,
            properties: table_properties(core, address)?,
            editable,
            can_resize: editable && fixed_grid(table).is_some(),
        })
    }
}

impl NativeEditor {
    fn editable_table_object(
        &self,
        address: NativeTableObjectAddress,
    ) -> Result<&Table, NativeHostError> {
        let selection = self.selection();
        if selection.anchor != selection.focus
            || NativeTableObjectAddress::from_cell(&selection.focus)? != address
        {
            return Err(invalid(
                "the selected table no longer matches the table at the caret",
            ));
        }
        let table = address.resolve(self.session().document())?;
        if !can_edit_table(table) {
            return Err(unsupported(
                "locked tables and tables containing protected cells cannot be edited",
            ));
        }
        Ok(table)
    }

    pub(super) fn set_table_object_properties(
        &mut self,
        revision: u64,
        address: NativeTableObjectAddress,
        properties: NativeTableObjectPropertyChanges,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        self.editable_table_object(address)?;
        let properties = properties.changed(&table_properties(self.session().document(), address)?);
        properties.validate()?;
        if properties == NativeTableObjectPropertyChanges::default() {
            return Ok(self.outcome(false, false));
        }
        let json = serde_json::to_string(&properties).map_err(table_json_error)?;
        let (section, paragraph, control) = address.indices();
        let selection = self.selection().clone();
        self.edit_explicit_ranges_transaction(revision, |core| {
            core.set_table_properties_native(section, paragraph, control, &json)?;
            Ok(selection)
        })
    }

    pub(super) fn resize_table_object(
        &mut self,
        revision: u64,
        address: NativeTableObjectAddress,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let table = self.editable_table_object(address)?;
        let (columns, rows) = fixed_grid(table).ok_or_else(|| {
            unsupported("table resizing requires a consistent rectangular, absolute-size grid")
        })?;
        let width = width.unwrap_or(table.common.width);
        let height = height.unwrap_or(table.common.height);
        if height > i32::MAX as u32 {
            return Err(invalid("requested dimensions exceed the table size range"));
        }
        let width_changed = width != table.common.width;
        let height_changed = height != table.common.height;
        if !width_changed && !height_changed {
            return Ok(self.outcome(false, false));
        }
        let columns = if width_changed {
            scaled_sizes(&columns, width)?
        } else {
            columns
        };
        let rows = if height_changed {
            // These rows have stored fixed sizes. The shared resize command
            // preserves the table's existing outer height beyond their sum.
            let content_height = height
                .checked_sub(table.row_height_surplus(&[]))
                .ok_or_else(|| {
                    invalid("requested height is smaller than the table's outer height allowance")
                })?;
            scaled_sizes(&rows, content_height)?
        } else {
            rows
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CellResize {
            cell_idx: usize,
            width_delta: i32,
            height_delta: i32,
        }
        let updates: Vec<_> = table
            .cells
            .iter()
            .enumerate()
            .map(|(cell_idx, cell)| CellResize {
                cell_idx,
                width_delta: columns[usize::from(cell.col)] as i32 - cell.width as i32,
                height_delta: rows[usize::from(cell.row)] as i32 - cell.height as i32,
            })
            .collect();
        let unchanged_single_row_height = (height_changed
            && rows.len() == 1
            && updates.iter().all(|update| update.height_delta == 0))
        .then_some(height);
        let json = serde_json::to_string(&updates).map_err(table_json_error)?;
        let (section, paragraph, control) = address.indices();
        let selection = self.selection().clone();
        self.edit_explicit_ranges_transaction(revision, |core| {
            core.resize_top_level_table_cells(
                section,
                paragraph,
                control,
                &json,
                unchanged_single_row_height,
            )?;
            let changed = address.resolve(core)?;
            if changed.common.width != width || changed.common.height != height {
                return Err(invalid(
                    "requested dimensions cannot be written at the table's minimum cell sizes",
                ));
            }
            Ok(selection)
        })
    }
}

fn table_properties(
    core: &DocumentCore,
    address: NativeTableObjectAddress,
) -> Result<NativeTableObjectProperties, NativeHostError> {
    let (section, paragraph, control) = address.indices();
    let json = core.get_table_properties_native(section, paragraph, control)?;
    serde_json::from_str(&json).map_err(table_json_error)
}

fn can_edit_table(table: &Table) -> bool {
    !table.common.locked && !table.cells.iter().any(|cell| cell.cell_protect())
}

/// The shared cell-delta command preserves an aligned grid. Require the same
/// stored size in every cell on each axis to avoid overwriting independent
/// row boundaries, merged areas or content-fit height placeholders.
fn fixed_grid(table: &Table) -> Option<(Vec<u32>, Vec<u32>)> {
    let cells = table.cells.len();
    let expected = usize::from(table.row_count) * usize::from(table.col_count);
    if cells == 0
        || cells != expected
        || cells > MAX_CREATE_TABLE_CELLS as usize
        || table.common.size_protect
        || table.common.width_criterion != SizeCriterion::Absolute
        || table.common.height_criterion != SizeCriterion::Absolute
        || table
            .cells
            .iter()
            .any(|cell| cell.col_span != 1 || cell.row_span != 1)
    {
        return None;
    }
    let columns = table.get_column_widths();
    let rows = table.get_raw_row_heights();
    let width: u64 = columns.iter().copied().map(u64::from).sum();
    let height: u64 = rows.iter().copied().map(u64::from).sum();
    if width != u64::from(table.common.width)
        || height > u64::from(table.common.height)
        || table.common.width > i32::MAX as u32
        || table.common.height > i32::MAX as u32
        || table.cells.iter().enumerate().any(|(index, cell)| {
            cell.height == 0
                || rows.get(usize::from(cell.row)) != Some(&cell.height)
                || columns.get(usize::from(cell.col)) != Some(&cell.width)
                || table.cell_index_at(cell.row, cell.col) != Some(index)
        })
    {
        return None;
    }
    Some((columns, rows))
}

fn scaled_sizes(sizes: &[u32], target: u32) -> Result<Vec<u32>, NativeHostError> {
    if target > i32::MAX as u32 {
        return Err(invalid("requested dimensions exceed the table size range"));
    }
    let minimum = u64::from(MIN_RESIZED_CELL_SIZE);
    let mut available = u64::from(target);
    if available < minimum * sizes.len() as u64 {
        return Err(invalid(
            "requested dimensions are below the table's minimum cell sizes",
        ));
    }
    // Reserve the minimum for undersized rows or columns first, then allocate
    // the remaining extent proportionally from cumulative edges.
    let mut total: u64 = sizes.iter().copied().map(u64::from).sum();
    let mut by_size: Vec<_> = (0..sizes.len()).collect();
    by_size.sort_unstable_by_key(|&index| sizes[index]);
    let mut at_minimum = vec![false; sizes.len()];
    for index in by_size {
        let original = u64::from(sizes[index]);
        if original * available >= minimum * total {
            break;
        }
        at_minimum[index] = true;
        available -= minimum;
        total -= original;
    }
    let mut old_edge = 0;
    let mut new_edge = 0;
    Ok(sizes
        .iter()
        .zip(at_minimum)
        .map(|(&size, at_minimum)| {
            if at_minimum {
                return MIN_RESIZED_CELL_SIZE;
            }
            old_edge += u64::from(size);
            let edge = old_edge * available / total;
            let scaled = (edge - new_edge) as u32;
            new_edge = edge;
            scaled
        })
        .collect())
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}

fn unsupported(message: &str) -> NativeHostError {
    NativeHostError::Unsupported(message.into())
}

fn table_json_error(error: serde_json::Error) -> NativeHostError {
    HwpError::RenderError(format!("invalid table object properties: {error}")).into()
}
