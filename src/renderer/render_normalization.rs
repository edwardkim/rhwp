//! Render-only derived state shared by pagination, measurement, and layout.
//!
//! The editable document IR remains authoritative.  Logical paths are the
//! durable cache keys; pointer indexes are rebuilt from the current source IR
//! and exist only as a fast lookup surface for renderer hot paths.

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::shape::{TextWrap, VertRelTo};
use crate::model::table::{Cell, Table, TablePageBreak};
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

/// 비-TAC 중첩 표는 저장 폭을 조판 폭으로 쓴다. 한컴 PDF는 부모 셀보다 근소하게
/// 좁은 1×1 표도 자동 확장하지 않는다(76076 p34: 36,572HU 유지).
///
/// 과거의 0.9 하한 스트레치는 페이지 수만 맞춘 보정이었다. 저장 폭을 넓히면
/// continuation 가용폭이 달라져 PDF 줄바꿈과 표 조각 경계가 모두 어긋난다.
pub(crate) const NESTED_STRETCH_MIN_RATIO: f64 = 1.0;

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
    /// Native HWP5 short RowBreak child는 parent viewport를 content box로도
    /// 사용한다. 이는 일반 non-TAC nested-table의 저장 cell margin 보존(#2308)과
    /// 구분되는, owner fragment 전용 projection이다.
    pub use_owner_content_box: bool,
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
        let hwp5_stored_pagination_layout =
            document.layout_profile().hwp5_stored_pagination_layout();
        for (section_index, section) in document.sections.iter().enumerate() {
            for (parent_paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
                for (control_index, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(table) = control else {
                        continue;
                    };
                    let path = RenderPath::top_level(section_index, parent_paragraph_index);
                    overlay.collect_nested_tables(
                        table,
                        path,
                        control_index,
                        hwp5_stored_pagination_layout,
                        previous,
                    );
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
        hwp5_stored_pagination_layout: bool,
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
                    // 비-TAC nested table은 한컴 PDF가 저장 폭을 유지한다. 단, native
                    // HWP5 RowBreak parent의 short-tail 1×1 child는 parent cell 폭을
                    // 쓰는 별도 저장 계약이다(76076 p81). p34의 일반 1×1 child에는
                    // 적용하지 않도록 구조·viewport·near-fit 조건을 모두 요구한다.
                    let keeps_legacy_near_fit_projection = !nested.common.treat_as_char
                        && source_width > 0
                        && u64::from(source_width) < u64::from(cell.width)
                        && f64::from(source_width)
                            >= f64::from(cell.width) * NESTED_STRETCH_MIN_RATIO;
                    let short_rowbreak_child_projection = hwp5_stored_pagination_layout
                        && Self::is_native_short_rowbreak_child_near_fit(
                            owner_table,
                            cell,
                            nested,
                            source_width,
                        );
                    if keeps_legacy_near_fit_projection || short_rowbreak_child_projection {
                        let effective_width = cell.width;
                        let table_pointer = nested.as_ref() as *const Table as usize;
                        let projection = previous
                            .nested_table_widths_by_path
                            .get(&nested_path)
                            .filter(|projection| {
                                projection.source_width == source_width
                                    && projection.effective_width == effective_width
                                    && projection.use_owner_content_box
                                        == short_rowbreak_child_projection
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
                                    use_owner_content_box: short_rowbreak_child_projection,
                                    table_pointer,
                                })
                            });
                        self.nested_table_widths_by_pointer
                            .insert(projection.table_pointer, Arc::clone(&projection));
                        self.nested_table_widths_by_path
                            .insert(nested_path.clone(), projection);
                    }

                    self.collect_nested_tables(
                        nested,
                        nested_path,
                        control_index,
                        hwp5_stored_pagination_layout,
                        previous,
                    );
                }
            }
        }
    }

    /// Native HWP5 `RowBreak` parent의 마지막 1×1 child만 parent cell 폭으로
    /// 투영한다. 이 source 형상은 `76076_regulatory_analysis` p81에서 child의
    /// 저장 폭(36,572HU)보다 parent cell 폭(38,245HU)을 line-wrap viewport로
    /// 사용하는 한컴 PDF 계약이다. 일반 near-fit nested table에는 적용하지 않는다.
    fn is_native_short_rowbreak_child_near_fit(
        owner: &Table,
        host_cell: &Cell,
        child: &Table,
        source_width: u32,
    ) -> bool {
        let owner_height = owner.common.height;
        let child_height = child.common.height;
        !owner.common.treat_as_char
            && matches!(owner.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(owner.common.vert_rel_to, VertRelTo::Para)
            && matches!(owner.page_break, TablePageBreak::RowBreak)
            && owner.row_count > 1
            && owner.cells.iter().all(|cell| cell.row_span == 1)
            && host_cell.row_span == 1
            && host_cell.row as usize + 1 == owner.row_count as usize
            && host_cell.paragraphs.first().is_some_and(|host| {
                host.text.trim().is_empty()
                    && host
                        .controls
                        .iter()
                        .filter(|control| matches!(control, Control::Table(_)))
                        .count()
                        == 1
            })
            && host_cell.paragraphs.iter().skip(1).all(|paragraph| {
                paragraph.text.trim().is_empty()
                    && paragraph.controls.is_empty()
                    && paragraph.line_segs.len() <= 1
            })
            && !child.common.treat_as_char
            && child.row_count == 1
            && child.col_count == 1
            && child.cells.len() == 1
            && child.cells[0].paragraphs.len() <= 3
            && owner_height > 0
            // 76076 p81 is the only candidate whose stored child viewport
            // (12,846HU) exceeds its RowBreak parent viewport (8,304HU).
            // This excludes p33 pi=511 (14,406 <= 24,456) and the p34
            // stored-width counterexample pi=336 (9,350 <= 19,400).
            && child_height > owner_height
            && source_width > 0
            && source_width < host_cell.width
            && u64::from(source_width) * 100 >= u64::from(host_cell.width) * 95
    }

    #[inline]
    pub fn nested_table_width_scale(&self, table: &Table) -> f64 {
        let key = table as *const Table as usize;
        self.nested_table_widths_by_pointer
            .get(&key)
            .map(|projection| projection.width_scale)
            .unwrap_or(1.0)
    }

    /// 76076 p81처럼 native HWP5 `RowBreak` parent가 마지막 1×1 child의
    /// source 폭뿐 아니라 content box를 parent owner viewport로 해석한 경우다.
    /// 일반 non-TAC child는 false여서 저장된 small cell margin을 계속 보존한다.
    #[inline]
    pub fn uses_owner_content_box(&self, table: &Table) -> bool {
        let key = table as *const Table as usize;
        self.nested_table_widths_by_pointer
            .get(&key)
            .is_some_and(|projection| projection.use_owner_content_box)
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

    /// 비-TAC nested table은 근소 미달도 포함해 선언 폭을 유지한다.
    #[test]
    fn nested_tables_keep_declared_width() {
        let narrow = document_with_nested_table(1_358, 2_000); // 0.679 — 직인 fixture 실측 비율
        let overlay = RenderNormalizationOverlay::from_document(&narrow);
        assert_eq!(overlay.nested_table_projection_count(), 0);

        // 0.956처럼 부모 셀에 거의 맞더라도 한컴 PDF는 저장 폭을 넓히지 않는다
        // (76076 p34의 36,572HU nested table).
        let near_fit = document_with_nested_table(1_912, 2_000); // 0.956
        let overlay = RenderNormalizationOverlay::from_document(&near_fit);
        assert_eq!(overlay.nested_table_projection_count(), 0);
    }

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

    #[test]
    fn short_native_rowbreak_child_projects_only_inside_short_owner_viewport() {
        let document = short_rowbreak_document(1_912, 2_000, 1_000, 2_000);
        let overlay = RenderNormalizationOverlay::from_document(&document);
        let nested = nested_table_at_final_row(&document);
        assert!(overlay
            .projection_for_path(&short_rowbreak_nested_path())
            .is_some());
        assert!(overlay.nested_table_width_scale(nested) > 1.0);
        assert!(overlay.uses_owner_content_box(nested));

        // p34's long owner viewport is a near-fit 1×1 nested-table counterexample.
        let long_owner = short_rowbreak_document(1_912, 2_000, 5_000, 1_000);
        let overlay = RenderNormalizationOverlay::from_document(&long_owner);
        let nested = nested_table_at_final_row(&long_owner);
        assert!(overlay
            .projection_for_path(&short_rowbreak_nested_path())
            .is_none());
        assert!(!overlay.uses_owner_content_box(nested));
    }

    fn short_rowbreak_document(
        source_width: u32,
        parent_width: u32,
        parent_height: u32,
        child_height: u32,
    ) -> Document {
        let mut nested = Table::default();
        nested.row_count = 1;
        nested.col_count = 1;
        nested.common.width = source_width;
        nested.common.height = child_height;
        nested.cells.push(Cell {
            col_span: 1,
            row_span: 1,
            width: source_width,
            paragraphs: vec![Paragraph::default()],
            ..Cell::default()
        });

        let mut host = Paragraph::default();
        host.controls.push(Control::Table(Box::new(nested)));

        let mut owner = Table::default();
        owner.row_count = 2;
        owner.col_count = 1;
        owner.page_break = TablePageBreak::RowBreak;
        owner.common.width = parent_width;
        owner.common.height = parent_height;
        owner.common.text_wrap = TextWrap::TopAndBottom;
        owner.common.vert_rel_to = VertRelTo::Para;
        owner.cells.push(Cell {
            row: 0,
            col_span: 1,
            row_span: 1,
            width: parent_width,
            paragraphs: vec![Paragraph::default()],
            ..Cell::default()
        });
        owner.cells.push(Cell {
            row: 1,
            col_span: 1,
            row_span: 1,
            width: parent_width,
            paragraphs: vec![host, Paragraph::default()],
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

    fn nested_table_at_final_row(document: &Document) -> &Table {
        let Control::Table(owner) = &document.sections[0].paragraphs[0].controls[0] else {
            panic!("owner table");
        };
        let Control::Table(nested) = &owner.cells[1].paragraphs[0].controls[0] else {
            panic!("nested table");
        };
        nested
    }

    fn short_rowbreak_nested_path() -> RenderPath {
        RenderPath {
            section_index: 0,
            parent_paragraph_index: 0,
            entries: vec![RenderPathEntry::TableCell {
                control_index: 0,
                cell_index: 1,
                paragraph_index: 0,
            }],
            target_control_index: Some(0),
        }
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
    fn near_fit_nested_table_has_no_render_width_projection() {
        let document = document_with_nested_table(1_900, 2_000);
        let overlay = RenderNormalizationOverlay::from_document(&document);
        let nested = nested_table(&document);

        assert_eq!(nested.common.width, 1_900, "source width must not change");
        assert_eq!(nested.cells[0].width, 1_900, "source cell width");
        assert!(
            (overlay.nested_table_width_scale(nested) - 1.0).abs() < f64::EPSILON,
            "stored nested-table width must be used without parent-cell projection"
        );
        assert!(overlay.projection_for_path(&nested_path()).is_none());
    }

    #[test]
    fn repeated_normalization_has_no_stale_width_projection() {
        let document = document_with_nested_table(1_900, 2_000);
        let first = RenderNormalizationOverlay::from_document(&document);
        let second = RenderNormalizationOverlay::from_document_reusing(&document, &first);

        assert_eq!(first.nested_table_projection_count(), 0);
        assert_eq!(second.nested_table_projection_count(), 0);
        assert!(first.projection_for_path(&nested_path()).is_none());
        assert!(second.projection_for_path(&nested_path()).is_none());
    }

    #[test]
    fn removed_source_path_does_not_reuse_stale_projection() {
        let mut document = document_with_nested_table(1_900, 2_000);
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
