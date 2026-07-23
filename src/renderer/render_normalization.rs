//! Render-only derived state shared by pagination, measurement, and layout.
//!
//! The editable document IR remains authoritative.  Logical paths are the
//! durable cache keys; pointer indexes are rebuilt from the current source IR
//! and exist only as a fast lookup surface for renderer hot paths.

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::table::Table;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RenderPathEntry {
    TableCell {
        control_index: usize,
        cell_index: usize,
        paragraph_index: usize,
    },
    TableCaption {
        control_index: usize,
        paragraph_index: usize,
    },
    ShapeTextBox {
        control_index: usize,
        paragraph_index: usize,
    },
    PictureCaption {
        control_index: usize,
        paragraph_index: usize,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderPath {
    pub section_index: usize,
    pub parent_paragraph_index: usize,
    pub entries: Vec<RenderPathEntry>,
    pub target_control_index: Option<usize>,
}

impl RenderPath {
    pub fn top_level(section_index: usize, parent_paragraph_index: usize) -> Self {
        Self {
            section_index,
            parent_paragraph_index,
            entries: Vec::new(),
            target_control_index: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NestedTableWidthProjection {
    pub path: RenderPath,
    pub source_width: u32,
    pub effective_width: u32,
    pub width_scale: f64,
    table_pointer: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RenderNormalizationOverlay {
    nested_table_widths_by_path: HashMap<RenderPath, Arc<NestedTableWidthProjection>>,
    nested_table_widths_by_pointer: HashMap<usize, Arc<NestedTableWidthProjection>>,
}

impl RenderNormalizationOverlay {
    pub fn from_document(document: &Document) -> Self {
        Self::from_document_reusing(document, &Self::default())
    }

    pub fn from_document_reusing(document: &Document, previous: &Self) -> Self {
        let mut overlay = Self::default();
        for (section_index, section) in document.sections.iter().enumerate() {
            for (parent_paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_index, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    let path = RenderPath::top_level(section_index, parent_paragraph_index);
                    overlay.collect_nested_tables(table, path, control_index, previous);
                }
            }
        }
        overlay
    }

    fn collect_nested_tables(
        &mut self,
        owner_table: &Table,
        path: RenderPath,
        owner_control_index: usize,
        previous: &Self,
    ) {
        for (cell_index, cell) in owner_table.cells.iter().enumerate() {
            if cell.width >= 0x8000_0000 {
                continue;
            }
            for (paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
                for (control_index, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(nested) = control else {
                        continue;
                    };

                    let mut nested_path = path.clone();
                    nested_path.entries.push(RenderPathEntry::TableCell {
                        control_index: owner_control_index,
                        cell_index,
                        paragraph_index,
                    });
                    nested_path.target_control_index = Some(control_index);

                    let source_width = nested.common.width;
                    if !nested.common.treat_as_char
                        && source_width > 0
                        && u64::from(source_width) < u64::from(cell.width)
                    {
                        let effective_width = cell.width;
                        let table_pointer = nested.as_ref() as *const Table as usize;
                        let projection = previous
                            .nested_table_widths_by_path
                            .get(&nested_path)
                            .filter(|projection| {
                                projection.source_width == source_width
                                    && projection.effective_width == effective_width
                                    && projection.table_pointer == table_pointer
                            })
                            .map(Arc::clone)
                            .unwrap_or_else(|| {
                                Arc::new(NestedTableWidthProjection {
                                    path: nested_path.clone(),
                                    source_width,
                                    effective_width,
                                    width_scale: f64::from(effective_width)
                                        / f64::from(source_width),
                                    table_pointer,
                                })
                            });
                        self.nested_table_widths_by_pointer
                            .insert(projection.table_pointer, Arc::clone(&projection));
                        self.nested_table_widths_by_path
                            .insert(nested_path.clone(), projection);
                    }

                    self.collect_nested_tables(nested, nested_path, control_index, previous);
                }
            }
        }
    }

    #[inline]
    pub fn nested_table_width_scale(&self, table: &Table) -> f64 {
        let key = table as *const Table as usize;
        self.nested_table_widths_by_pointer
            .get(&key)
            .map(|projection| projection.width_scale)
            .unwrap_or(1.0)
    }

    pub fn projection_for_path(
        &self,
        path: &RenderPath,
    ) -> Option<Arc<NestedTableWidthProjection>> {
        self.nested_table_widths_by_path.get(path).map(Arc::clone)
    }

    pub fn nested_table_projection_count(&self) -> usize {
        self.nested_table_widths_by_path.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Section;
    use crate::model::paragraph::Paragraph;
    use crate::model::table::Cell;

    fn document_with_nested_table(source_width: u32, parent_width: u32) -> Document {
        let mut nested = Table::default();
        nested.row_count = 1;
        nested.col_count = 1;
        nested.common.width = source_width;
        nested.cells.push(Cell {
            col_span: 1,
            row_span: 1,
            width: source_width,
            paragraphs: vec![Paragraph::default()],
            ..Cell::default()
        });

        let mut cell_paragraph = Paragraph::default();
        cell_paragraph
            .controls
            .push(Control::Table(Box::new(nested)));

        let mut owner = Table::default();
        owner.row_count = 1;
        owner.col_count = 1;
        owner.common.width = parent_width;
        owner.cells.push(Cell {
            col_span: 1,
            row_span: 1,
            width: parent_width,
            paragraphs: vec![cell_paragraph],
            ..Cell::default()
        });

        let mut parent = Paragraph::default();
        parent.controls.push(Control::Table(Box::new(owner)));
        let mut section = Section::default();
        section.paragraphs.push(parent);
        let mut document = Document::default();
        document.sections.push(section);
        document
    }

    fn nested_table(document: &Document) -> &Table {
        let Control::Table(owner) = &document.sections[0].paragraphs[0].controls[0] else {
            panic!("owner table");
        };
        let Control::Table(nested) = &owner.cells[0].paragraphs[0].controls[0] else {
            panic!("nested table");
        };
        nested
    }

    fn nested_path() -> RenderPath {
        RenderPath {
            section_index: 0,
            parent_paragraph_index: 0,
            entries: vec![RenderPathEntry::TableCell {
                control_index: 0,
                cell_index: 0,
                paragraph_index: 0,
            }],
            target_control_index: Some(0),
        }
    }

    #[test]
    fn nested_width_projection_keeps_source_ir_immutable() {
        let document = document_with_nested_table(1_000, 2_000);
        let overlay = RenderNormalizationOverlay::from_document(&document);
        let nested = nested_table(&document);
        let projection = overlay
            .projection_for_path(&nested_path())
            .expect("nested width projection");

        assert_eq!(nested.common.width, 1_000, "source width must not change");
        assert_eq!(nested.cells[0].width, 1_000, "source cell width");
        assert_eq!(projection.source_width, 1_000);
        assert_eq!(projection.effective_width, 2_000);
        assert!((overlay.nested_table_width_scale(nested) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stable_projection_reuses_arc_identity() {
        let document = document_with_nested_table(1_000, 2_000);
        let first = RenderNormalizationOverlay::from_document(&document);
        let second = RenderNormalizationOverlay::from_document_reusing(&document, &first);
        let first_projection = first
            .projection_for_path(&nested_path())
            .expect("first projection");
        let second_projection = second
            .projection_for_path(&nested_path())
            .expect("second projection");

        assert!(Arc::ptr_eq(&first_projection, &second_projection));
    }

    #[test]
    fn unrelated_path_edit_reuses_nested_projection_identity() {
        let mut document = document_with_nested_table(1_000, 2_000);
        let first = RenderNormalizationOverlay::from_document(&document);
        let first_projection = first
            .projection_for_path(&nested_path())
            .expect("initial projection");

        document.sections[0].paragraphs[0].text = "unrelated edit".to_string();
        let second = RenderNormalizationOverlay::from_document_reusing(&document, &first);
        let second_projection = second
            .projection_for_path(&nested_path())
            .expect("projection after unrelated edit");

        assert!(
            Arc::ptr_eq(&first_projection, &second_projection),
            "an unrelated stable-path edit must preserve the nested projection Arc"
        );
    }

    #[test]
    fn removed_source_path_does_not_reuse_stale_projection() {
        let mut document = document_with_nested_table(1_000, 2_000);
        let first = RenderNormalizationOverlay::from_document(&document);
        let Control::Table(owner) = &mut document.sections[0].paragraphs[0].controls[0] else {
            panic!("owner table");
        };
        owner.cells[0].paragraphs[0].controls.clear();

        let second = RenderNormalizationOverlay::from_document_reusing(&document, &first);

        assert_eq!(second.nested_table_projection_count(), 0);
        assert!(
            second.projection_for_path(&nested_path()).is_none(),
            "a missing logical source path must never fall back to the previous projection"
        );
    }
}
