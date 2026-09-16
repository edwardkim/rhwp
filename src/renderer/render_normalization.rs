//! Render-only derived state shared by pagination, measurement, and layout.
//!
//! The editable document IR remains authoritative.  Logical paths are the
//! durable cache keys; pointer indexes are rebuilt from the current source IR
//! and exist only as a fast lookup surface for renderer hot paths.

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::shape::{TextWrap, VertRelTo};
use crate::model::table::{Cell, Table, TablePageBreak};
use std::collections::{HashMap, HashSet};
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
    text_reflowed_tables_by_pointer: HashSet<usize>,
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
                    // [#6590] 최상위 글자처럼(TAC) 표의 선언 폭이 본문 폭을 근소 초과하면
                    // 한/글은 본문 폭으로 비례 축소해 그린다. 근거는 host 문단의 저장
                    // lineseg 다 — `samples/basic/BlogForm_BookReview.hwp` 실측: 표 선언
                    // 폭 35719HU > 본문 폭 35149HU 인데 host 줄의 저장 `segment_width`
                    // 는 35148HU 로 본문 폭과 같다(한/글이 표를 본문 폭 줄박스에
                    // 실었다는 직접 증거). 그대로 그리면 표 우단이 본문 우단을 넘는다
                    // (같은 표본 7.6px).
                    //
                    // near-fit(축소율 0.9 이상)에만 적용한다 — 본문보다 훨씬 넓은 표는
                    // 저작 의도가 다르고(가로 넘침 허용·다단 등) 기존 경로가 맡는다.
                    //
                    // [#7059] 위 근거(host 줄이 본문 폭)는 **축소의 증거가 아니다.** host 줄이
                    // 본문 폭인 것은 "표가 그 줄을 오른쪽으로 넘친다"는 뜻이기도 하다. 실제로
                    // 한/글은 이 형상에서 표를 **축소하지 않고 본문 우단을 넘겨 그린다** —
                    // 위 표본의 정본 세 판본(`pdf/basic/BlogForm_BookReview-hwp-2020.pdf` ·
                    // `-2022.pdf` · `pdf/BlogForm_BookReview-2020.pdf`)이 글자 단위로 같다.
                    //
                    //   본문 영역     22.68 .. 374.17 pt
                    //   선언 표 폭    357.19 pt (35719 HU)
                    //   정본 괘선     22.51 .. 379.37 pt (폭 356.86) — 본문 우단 +5.2 pt
                    //
                    // 갈림은 **표 첫 칸의 저장 `LINE_SEG`** 다. 표가 정말 축소됐다면 칸 안
                    // 줄도 축소 폭이어야 한다. 위 표본은 34696 + 여백 1020 = 35716 으로 선언
                    // 35719 에 수렴하고(축소 35149 와는 567 차이), `#7059` 가 신고한
                    // 3194097 도 50440 + 282 = 50722 = 선언 폭이다. 둘 다 정본이 "축소 안 함"
                    // 이다.
                    //
                    // 사다리가 축소 폭을 말하는 표는 종전대로 축소한다 — samples 373건 +
                    // 행정규칙 855건에서 판정 가능한 표 30개 중 18개가 그쪽이다.
                    if hwp5_stored_pagination_layout && table.common.treat_as_char {
                        let page_def = &section.section_def.page_def;
                        let body_width = page_def
                            .width
                            .saturating_sub(page_def.margin_left)
                            .saturating_sub(page_def.margin_right);
                        let source_width = table.common.width;
                        if body_width > 0
                            && source_width > body_width
                            && f64::from(body_width) >= f64::from(source_width) * 0.9
                            && stored_first_cell_line_says_shrunk(table, body_width, source_width)
                        {
                            let mut top_path = path.clone();
                            top_path.target_control_index = Some(control_index);
                            let table_pointer = table.as_ref() as *const Table as usize;
                            let projection = Arc::new(NestedTableWidthProjection {
                                path: top_path.clone(),
                                source_width,
                                effective_width: body_width,
                                width_scale: f64::from(body_width) / f64::from(source_width),
                                use_owner_content_box: false,
                                table_pointer,
                            });
                            overlay
                                .nested_table_widths_by_pointer
                                .insert(table_pointer, Arc::clone(&projection));
                            overlay
                                .nested_table_widths_by_path
                                .insert(top_path, projection);
                        }
                    }
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
                        // An owner-derived viewport starts at the host cell's
                        // padded origin. Its width must therefore be the same
                        // content box, not the unpadded border-box width.
                        // Resolve table-default versus per-cell margins through
                        // the shared model rule (including explicit zero).
                        // Keep this in the overlay so measurement, fragment
                        // composition and paint consume the same width scale.
                        let padding = cell.effective_padding(&owner_table.padding);
                        let effective_width = (i64::from(cell.width)
                            - i64::from(padding.left)
                            - i64::from(padding.right))
                        .clamp(0, i64::from(u32::MAX))
                            as u32;
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

    /// Native HWP5 `RowBreak` parent의 마지막 1×1 child에 owner 기반 폭을 선택하는
    /// 기존 호환 조건. 실제 viewport는 collect_nested_tables에서 유효 안 여백을
    /// 제외한다. 76076 p81의 38,245HU는 content box가 아니라 parent cell 전폭이다.
    /// 일반 near-fit nested table에는 적용하지 않는다.
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

    /// Register one table from the current source or normalized snapshot.
    /// Logical-path validation happens in DocumentCore before this hot-path
    /// pointer is published.
    pub(crate) fn register_text_reflowed_table(&mut self, table: &Table) {
        self.text_reflowed_tables_by_pointer
            .insert(table as *const Table as usize);
    }

    #[inline]
    pub(crate) fn table_text_reflowed(&self, table: &Table) -> bool {
        self.text_reflowed_tables_by_pointer
            .contains(&(table as *const Table as usize))
    }
}

/// [#7059] 표 **첫 칸의 저장 `LINE_SEG`** 가 축소된 폭을 말하는가.
///
/// `#6590` 의 근사-축소는 host 문단의 저장 줄이 본문 폭인 것을 근거로 삼았는데, 그것은
/// "표가 그 줄을 오른쪽으로 넘친다"는 뜻이기도 해서 축소를 가르지 못한다. 표가 정말 축소돼
/// 저장됐다면 **칸 안 줄**도 축소 폭이어야 한다 — 그쪽이 갈림이다.
///
/// 칸의 안쪽 폭은 `칸 폭 - 실효 좌우 여백`이다(`Cell::effective_padding` — 렌더·측정과 같은
/// 단일 출처). 축소 가정의 안쪽 폭과 선언 가정의 안쪽 폭 중 저장 줄이 가까운 쪽을 고른다.
///
/// 합성 줄(`TAG_IMPLEMENTATION_PROPERTY`)은 저장으로 치지 않는다 — 우리가 만든 값으로
/// 우리 규칙을 판정하면 순환이다. 저장 줄이 없으면 축소하지 않는다(근거 없음).
fn stored_first_cell_line_says_shrunk(table: &Table, body_width: u32, source_width: u32) -> bool {
    let Some(cell) = table.cells.iter().find(|c| c.row == 0 && c.col == 0) else {
        return false;
    };
    let Some(seg) = cell
        .paragraphs
        .iter()
        .find_map(|paragraph| paragraph.line_segs.first())
    else {
        return false;
    };
    if seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 {
        return false;
    }
    let padding = cell.effective_padding(&table.padding);
    let padding_h = i64::from(padding.left) + i64::from(padding.right);
    let declared_inner = i64::from(cell.width) - padding_h;
    let scale = f64::from(body_width) / f64::from(source_width);
    let shrunk_inner = (f64::from(cell.width) * scale) as i64 - padding_h;
    let stored = i64::from(seg.segment_width);
    (stored - shrunk_inner).abs() < (stored - declared_inner).abs()
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
