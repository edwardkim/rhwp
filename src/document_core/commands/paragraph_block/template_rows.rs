//! Copy complete row groups, fill detached cells, and replace one owning root.
use super::{
    budget, invalid, owned,
    repeat::table_keys,
    rows_geometry,
    template::{path, validate_source_fill, FillEdits},
    validation, ParagraphBlockBudget, ParagraphBlockCopy, ParagraphBlockLimits,
    ParagraphBlockMapping, ParagraphBlockPathStep as Step, RepeatParagraphBlockRequest,
    TemplateBinding,
};
use crate::{
    document_core::{DocumentCore, TableTextReflowKey},
    error::HwpError,
    model::{
        control::Control,
        event::DocumentEvent,
        identity::{used_instance_ids, Allocator},
        paragraph::Paragraph,
        table::Table,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
};

/// Coordinates refer to the input, not to the progressively growing table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepeatTableRowsRequest {
    pub section_index: usize,
    pub paragraph_index: usize,
    pub control_index: usize,
    pub start_row: u16,
    pub end_row: u16,
    pub insert_before: u16,
    /// Targets use [Paragraph(0), Control(0), Cell(n), Paragraph(p), ...].
    /// Cell(n) is the nth source cell in (row, column) order, not the whole table index.
    pub bindings: Vec<TemplateBinding>,
    /// One record per new row group. Empty maps allow copy-only repetition.
    pub records: Vec<BTreeMap<String, String>>,
    #[serde(default)]
    pub limits: ParagraphBlockLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatTableRowsResult {
    pub section_index: usize,
    pub paragraph_index: usize,
    pub control_index: usize,
    pub inserted_rows: Range<usize>,
    pub source_rows_after: Range<usize>,
    pub row_count: u16,
    /// Copy ranges are rows. Mapping source/destination paths are absolute owned paths.
    pub copies: Vec<ParagraphBlockCopy>,
}

fn table(p: &Paragraph, control: usize) -> Result<&Table, HwpError> {
    match p.controls.get(control) {
        Some(Control::Table(t)) => Ok(t),
        _ => Err(invalid(
            "rows/table: expected a top-level body table control",
        )),
    }
}
fn table_mut(p: &mut Paragraph, control: usize) -> &mut Table {
    match &mut p.controls[control] {
        Control::Table(t) => t,
        _ => unreachable!("validated table"),
    }
}
fn cap(a: usize, b: usize, limit: usize) -> Result<usize, HwpError> {
    a.checked_add(b)
        .filter(|v| *v <= limit)
        .ok_or_else(|| invalid("rows/staging budget exceeded"))
}

struct PreparedRows {
    result: RepeatTableRowsResult,
    staged: Option<Paragraph>,
    old_keys: Vec<TableTextReflowKey>,
    inherited: Vec<TableTextReflowKey>,
}

impl DocumentCore {
    /// Full detached row preparation; does not mutate even rendering provenance.
    pub fn preview_repeat_and_fill_table_rows_native(
        &self,
        request: &RepeatTableRowsRequest,
    ) -> Result<RepeatTableRowsResult, HwpError> {
        Ok(self.prepare_table_rows(request)?.result)
    }

    /// Atomic row repetition. The original table ID, rows, styles and clipboard survive.
    /// Title rows and crossing merges/zones are rejected, never silently repaired.
    pub fn repeat_and_fill_table_rows_native(
        &mut self,
        r: &RepeatTableRowsRequest,
    ) -> Result<RepeatTableRowsResult, HwpError> {
        let PreparedRows {
            result,
            staged,
            old_keys,
            inherited,
        } = self.prepare_table_rows(r)?;
        let Some(staged) = staged else {
            return Ok(result);
        };
        self.render_normalization
            .text_reflowed_tables
            .try_reserve(inherited.len())
            .map_err(|_| invalid("rows/provenance allocation failed"))?;
        self.event_log
            .try_reserve(1)
            .map_err(|_| invalid("rows/event allocation failed"))?;
        self.document.sections[r.section_index].paragraphs[r.paragraph_index] = staged;
        self.document.sections[r.section_index].raw_stream = None;
        for key in old_keys {
            self.render_normalization.text_reflowed_tables.remove(&key);
        }
        self.render_normalization
            .text_reflowed_tables
            .extend(inherited);
        self.recompose_section(r.section_index);
        self.paginate_if_needed();
        self.event_log.push(DocumentEvent::TableRowInserted {
            section: r.section_index,
            para: r.paragraph_index,
            ctrl: r.control_index,
        });
        Ok(result)
    }

    /// All fallible model work happens under an immutable core borrow.
    fn prepare_table_rows(&self, r: &RepeatTableRowsRequest) -> Result<PreparedRows, HwpError> {
        let host = self
            .document
            .sections
            .get(r.section_index)
            .and_then(|s| s.paragraphs.get(r.paragraph_index))
            .ok_or_else(|| invalid("rows/host address out of range"))?;
        let original = table(host, r.control_index)?;
        let g = rows_geometry::inspect(original, r)?;
        let count = r.records.len();
        let start = usize::from(r.insert_before);
        let shift = if r.insert_before <= r.start_row {
            usize::from(g.added)
        } else {
            0
        };
        let mut result = RepeatTableRowsResult {
            section_index: r.section_index,
            paragraph_index: r.paragraph_index,
            control_index: r.control_index,
            inserted_rows: start..start + usize::from(g.added),
            source_rows_after: usize::from(r.start_row) + shift..usize::from(r.end_row) + shift,
            row_count: g.rows,
            copies: vec![],
        };
        let inspection = RepeatParagraphBlockRequest {
            section_index: r.section_index,
            source_start: 0,
            source_end: 1,
            insert_before: 1,
            count,
            limits: r.limits,
        };
        let mut costs = ParagraphBlockBudget {
            added_paragraphs: 0,
            structure_bytes: 0,
            added_nodes: 0,
            owned_depth: 0,
            mapping_bytes: 0,
            document_nodes: 0,
            source_start_after: 0,
            source_end_after: 1,
            inserted_end: 1,
        };
        if count == 0 {
            validate_source_fill(&[], &inspection, &r.bindings, &r.records, costs)
                .map_err(|e| invalid(e.to_string()))?;
            return Ok(PreparedRows {
                result,
                staged: None,
                old_keys: vec![],
                inherited: vec![],
            });
        }
        // Before any deep clone, bound the complete owner. Two owner-sized charges
        // cover the target staging and the temporary projection construction.
        let host_cost = owned::source(
            std::slice::from_ref(host),
            r.limits.max_nodes / 2,
            r.limits.max_depth,
        )?;
        let base_bytes = budget::measure(
            std::slice::from_ref(host),
            2,
            r.limits.max_structure_bytes,
            host_cost.skipped_bytes,
        )?;
        costs.document_nodes = owned::document(&self.document, r.limits.max_document_nodes)?;
        let once = RepeatParagraphBlockRequest {
            count: 1,
            ..inspection.clone()
        };
        let old_keys = table_keys(std::slice::from_ref(host), &once)?;
        // The projection owns only the selected cells; the table wrapper itself
        // is discarded after each copy. It is never substituted for the live table.
        let mut projected = original.clone();
        let selected: HashSet<_> = g.cells.iter().copied().collect();
        let mut cells: Vec<_> = projected
            .cells
            .drain(..)
            .enumerate()
            .filter_map(|(i, c)| selected.contains(&i).then_some(c))
            .collect();
        cells.sort_by_key(|c| (c.row, c.col));
        for c in &mut cells {
            c.row -= r.start_row;
        }
        projected.cells = cells;
        projected.zones = g
            .zones
            .iter()
            .map(|i| {
                let mut z = original.zones[*i].clone();
                z.start_row -= r.start_row;
                z.end_row -= r.start_row;
                z
            })
            .collect();
        projected.row_count = r.end_row - r.start_row;
        projected.caption = None;
        rows_geometry::rebuild_row_sizes(&mut projected);
        projected.rebuild_grid();
        let source = [Paragraph {
            controls: vec![Control::Table(Box::new(projected))],
            ..Default::default()
        }];
        let source_cost = owned::source(&source, r.limits.max_nodes / count, r.limits.max_depth)?;
        cap(
            host_cost.nodes * 2,
            source_cost
                .nodes
                .checked_mul(count)
                .ok_or_else(|| invalid("rows/node overflow"))?,
            r.limits.max_nodes,
        )?;
        costs.added_nodes = source_cost.nodes * count;
        costs.owned_depth = source_cost.depth;
        costs.mapping_bytes = source_cost
            .mapping_bytes
            .checked_mul(count)
            .filter(|v| *v <= r.limits.max_mapping_bytes)
            .ok_or_else(|| invalid("rows/mapping budget exceeded"))?;
        let source_bytes = budget::measure(
            &source,
            count,
            r.limits.max_structure_bytes,
            source_cost.skipped_bytes,
        )?;
        costs.structure_bytes = cap(
            cap(base_bytes, source_bytes, r.limits.max_structure_bytes)?,
            g.grid_bytes,
            r.limits.max_structure_bytes,
        )?;
        let mut paras = 0usize;
        owned::inspect_source(
            &source,
            r.limits.max_nodes / count,
            r.limits.max_depth,
            |n| {
                if matches!(n, owned::Node::Para(_)) {
                    paras += 1;
                }
                Ok(())
            },
        )?;
        costs.added_paragraphs = (paras - 1)
            .checked_mul(count)
            .filter(|v| *v <= r.limits.max_paragraphs)
            .ok_or_else(|| invalid("rows/paragraph budget exceeded"))?;
        validation::row_source(&self.document, &source, original, &g.cells, &inspection)
            .map_err(|e| invalid(e.to_string()))?;
        for b in &r.bindings {
            if !matches!(
                path(&b.target),
                [
                    Step::Paragraph(0),
                    Step::Control(0),
                    Step::Cell(_),
                    Step::Paragraph(_),
                    ..
                ]
            ) {
                return Err(invalid(
                    "rows/fill target must belong to a source cell paragraph",
                ));
            }
        }
        let preview = validate_source_fill(&source, &inspection, &r.bindings, &r.records, costs)
            .map_err(|e| invalid(e.to_string()))?;
        let mut edits = FillEdits::prepare_source(&source, &inspection, &r.bindings, &preview)?;
        let paths = validation::paths(&source, &inspection).map_err(|e| invalid(e.to_string()))?;
        let source_keys = table_keys(&source, &inspection)?;
        // Source projection keys are fresh; inspect the actual source cell descendants.
        let mut source_flags = vec![self
            .render_normalization
            .text_reflowed_tables
            .contains(&TableTextReflowKey::from_table(original))];
        for i in &g.cells {
            for p in &original.cells[*i].paragraphs {
                source_flags.extend(
                    table_keys(std::slice::from_ref(p), &once)?
                        .iter()
                        .map(|k| self.render_normalization.text_reflowed_tables.contains(k)),
                );
            }
        }
        if source_flags.len() != source_keys.len() {
            return Err(invalid("rows/source ownership mismatch"));
        }
        let mut allocator = Allocator {
            used: used_instance_ids(&self.document),
            next: 1,
        };
        let mut staged = host.clone();
        let mut inherited: Vec<_> = old_keys
            .iter()
            .zip(table_keys(std::slice::from_ref(&staged), &once)?)
            .filter_map(|(old, new)| {
                self.render_normalization
                    .text_reflowed_tables
                    .contains(old)
                    .then_some(new)
            })
            .collect();
        let target = table_mut(&mut staged, r.control_index);
        for c in &mut target.cells {
            if c.row >= r.insert_before {
                c.row += g.added;
            }
        }
        for z in &mut target.zones {
            if z.start_row >= r.insert_before {
                z.start_row += g.added;
                z.end_row += g.added;
            }
        }
        for (index, record) in r.records.iter().enumerate() {
            let mut copy = source.to_vec();
            super::super::clone_identity::reidentify_with_allocator(&mut copy, &mut allocator)?;
            let wrapper_key = TableTextReflowKey::from_table(table(&copy[0], 0)?);
            inherited.extend(
                edits
                    .apply(&mut copy, record, r.limits, r.limits.max_nodes / count)?
                    .into_iter()
                    .filter(|k| *k != wrapper_key),
            );
            inherited.extend(
                table_keys(&copy, &inspection)?
                    .into_iter()
                    .zip(&source_flags)
                    .filter_map(|(k, flag)| (*flag && k != wrapper_key).then_some(k)),
            );
            let offset = r.insert_before + (index * (usize::from(r.end_row - r.start_row))) as u16;
            let t = table_mut(&mut copy[0], 0);
            for c in &mut t.cells {
                c.row += offset;
            }
            for z in &mut t.zones {
                z.start_row += offset;
                z.end_row += offset;
            }
            target.cells.append(&mut t.cells);
            target.zones.append(&mut t.zones);
            result.copies.push(ParagraphBlockCopy {
                range: usize::from(offset)
                    ..usize::from(offset) + usize::from(r.end_row - r.start_row),
                mappings: vec![],
            });
        }
        target.cells.sort_by_key(|c| (c.row, c.col));
        target.row_count = g.rows;
        rows_geometry::rebuild_row_sizes(target);
        target.rebuild_grid();
        target.sync_ctrl_height(g.height); // Width, position, title and split policy stay intact.
        inherited.push(TableTextReflowKey::from_table(target));
        let anchors: HashMap<_, _> = target
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| ((c.row, c.col), i))
            .collect();
        for copy in &mut result.copies {
            for p in &paths {
                let Some(Step::Cell(i)) = p.get(2) else {
                    continue;
                };
                let original_cell = &original.cells[g.cells[*i]];
                let row = copy.range.start as u16 + (original_cell.row - r.start_row);
                let destination_cell = anchors[&(row, original_cell.col)];
                let mut src = p.clone();
                src[0] = Step::Paragraph(r.paragraph_index);
                src[1] = Step::Control(r.control_index);
                src[2] = Step::Cell(g.cells[*i]);
                let mut dst = src.clone();
                dst[2] = Step::Cell(destination_cell);
                copy.mappings.push(ParagraphBlockMapping {
                    source: src,
                    destination: dst,
                });
            }
        }
        // Re-measure the grown owner too, before touching any live state.
        let final_cost = owned::source(
            std::slice::from_ref(&staged),
            r.limits.max_nodes,
            r.limits.max_depth,
        )?;
        let final_bytes = budget::measure(
            std::slice::from_ref(&staged),
            1,
            r.limits.max_structure_bytes,
            final_cost.skipped_bytes,
        )?;
        cap(final_bytes, source_bytes, r.limits.max_structure_bytes)?;
        Ok(PreparedRows {
            result,
            staged: Some(staged),
            old_keys,
            inherited,
        })
    }
}
