//! Read-only convenience addresses resolve to the exact same fill target contract.
use super::{
    template::select, validation, ParagraphBlockPathStep as Step,
    ParagraphBlockValidationError as Error, TemplateBinding, TemplateFillTarget, TemplateScope,
};
use crate::{
    document_core::DocumentCore,
    model::{control::Control, paragraph::Paragraph},
};

type LocatedParagraph<'a> = (Vec<Step>, &'a Paragraph);

impl DocumentCore {
    fn template_scope_paragraphs(
        &self,
        scope: &TemplateScope,
    ) -> Result<Vec<LocatedParagraph<'_>>, Error> {
        let inspection = scope.inspection();
        self.validate_paragraph_block_native(&inspection)?;
        validation::paragraphs(
            &self.document.sections[scope.section_index].paragraphs[scope.start..scope.end],
            &inspection,
        )
    }

    /// Resolve only closed ClickHere fields in this scope, in owned-tree traversal order.
    /// Occurrence is zero-based. None requires exactly one match, never the first match.
    /// Returned paths are request-local snapshots; later fill revalidates them.
    pub fn template_field_target_native(
        &self,
        scope: &TemplateScope,
        name: &str,
        occurrence: Option<usize>,
    ) -> Result<TemplateFillTarget, Error> {
        if name.is_empty() || name.len() > 8 * 1024 * 1024 {
            return Err(validation::reject(
                &[],
                "fillName",
                "empty or oversized field name",
            ));
        }
        let paras = self.template_scope_paragraphs(scope)?;
        let mut matches = 0usize;
        let mut selected = None;
        for (path, para) in &paras {
            for (index, range) in para.field_ranges.iter().enumerate() {
                let Some(Control::Field(field)) = para.controls.get(range.control_idx) else {
                    continue;
                };
                if field.field_name() != Some(name) {
                    continue;
                }
                if matches == occurrence.unwrap_or(0) {
                    selected = Some((path, *para, index));
                }
                matches += 1;
            }
        }
        if occurrence.is_none() && matches > 1 {
            return Err(validation::reject(
                &[],
                "fillAmbiguous",
                "multiple scoped fields require occurrence",
            ));
        }
        let (path, para, field_range_index) = selected.ok_or_else(|| {
            validation::reject(&[], "fillName", "scoped field occurrence not found")
        })?;
        let target = TemplateFillTarget::Field {
            path: path.clone(),
            field_range_index,
        };
        select(
            &TemplateBinding {
                key: name.into(),
                target: target.clone(),
            },
            para,
        )?;
        Ok(target)
    }

    /// Table path ends in Control(table_index); row/column must be a real cell anchor.
    /// Select one cell paragraph and scalar range, never flatten a multi-paragraph cell.
    pub fn template_cell_target_native(
        &self,
        scope: &TemplateScope,
        table_path: &[Step],
        anchor: (u16, u16),
        paragraph_index: usize,
        range: std::ops::Range<usize>,
    ) -> Result<TemplateFillTarget, Error> {
        if table_path.len() > scope.limits.max_depth {
            return Err(validation::reject(
                &[],
                "fillBudget",
                "table path depth exceeded",
            ));
        }
        let paras = self.template_scope_paragraphs(scope)?;
        let Some((Step::Control(index), owner_path)) = table_path.split_last() else {
            return Err(validation::reject(
                table_path,
                "fillPath",
                "table path must end in Control",
            ));
        };
        let owner = paras
            .iter()
            .find(|(path, _)| path == owner_path)
            .ok_or_else(|| validation::reject(table_path, "fillPath", "table owner not in scope"))?
            .1;
        let Some(Control::Table(table)) = owner.controls.get(*index) else {
            return Err(validation::reject(
                table_path,
                "fillPath",
                "target is not a table",
            ));
        };
        let (row, col) = anchor;
        let mut covering = table.cells.iter().enumerate().filter(|(_, cell)| {
            u32::from(row) >= u32::from(cell.row)
                && u32::from(row) < u32::from(cell.row) + u32::from(cell.row_span)
                && u32::from(col) >= u32::from(cell.col)
                && u32::from(col) < u32::from(cell.col) + u32::from(cell.col_span)
        });
        let (index, cell) = covering
            .next()
            .ok_or_else(|| validation::reject(table_path, "fillCell", "cell anchor not found"))?;
        if covering.next().is_some() || (cell.row, cell.col) != anchor {
            return Err(validation::reject(
                table_path,
                "fillCell",
                "ambiguous or merged-covered cell coordinate",
            ));
        }
        let para = cell.paragraphs.get(paragraph_index).ok_or_else(|| {
            validation::reject(table_path, "fillPath", "cell paragraph not found")
        })?;
        let mut path = table_path.to_vec();
        path.extend([Step::Cell(index), Step::Paragraph(paragraph_index)]);
        if !paras.iter().any(|(owned, _)| owned == &path) {
            return Err(validation::reject(
                &path,
                "fillPath",
                "cell paragraph is not supported",
            ));
        }
        let target = TemplateFillTarget::TextRange {
            path,
            start: range.start,
            end: range.end,
        };
        select(
            &TemplateBinding {
                key: "cell".into(),
                target: target.clone(),
            },
            para,
        )?;
        Ok(target)
    }
}
