//! Render-only derived state shared by pagination, measurement, and layout.
//!
//! The editable document IR remains authoritative.  Logical paths are the
//! durable cache keys; pointer indexes are rebuilt from the current source IR
//! and exist only as a fast lookup surface for renderer hot paths.

pub(crate) mod cached_frame_owner;
mod stored_nested_text_frames;
pub(crate) use stored_nested_text_frames::{StoredNestedTextFrames, StoredTextFrame};

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::table::{Cell, Table};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) mod math_table;
pub(crate) mod math_textbox;
pub(crate) mod stored_text_metrics;
pub(crate) mod stored_word_tail;

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

impl crate::storage::CloneWithStorage for RenderPathEntry {
    fn clone_with_storage<S: crate::storage::Storage>(
        &self,
        storage: &S,
    ) -> Result<Self, S::Error> {
        let _ = storage;
        Ok(self.clone())
    }
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderPath {
    pub section_index: usize,
    pub parent_paragraph_index: usize,
    pub entries: Vec<RenderPathEntry>,
    pub target_control_index: Option<usize>,
}
crate::storage::clone_fields!(RenderPath, {section_index, parent_paragraph_index, entries, target_control_index});

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
    /// 투영된 표의 내용 상자가 owner viewport를 사용하는지 나타낸다.
    /// 부모·자식의 선언 높이만으로 이 소유 관계를 추정하지 않는다.
    pub use_owner_content_box: bool,
    table_pointer: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RenderNormalizationOverlay {
    nested_table_widths_by_path: HashMap<RenderPath, Arc<NestedTableWidthProjection>>,
    nested_table_widths_by_pointer: HashMap<usize, Arc<NestedTableWidthProjection>>,
    text_reflowed_tables_by_pointer: HashSet<usize>,
    rendered_cell_frame_changes: HashMap<usize, HashSet<(u16, u16)>>,
    rebased_table_frames_by_pointer: HashSet<usize>,
    expanded_inline_host_tables_by_pointer: HashSet<usize>,
    nested_tables_with_reflowed_hosts: HashSet<usize>,
    stored_nested_text_frames_by_pointer: HashMap<usize, Arc<StoredNestedTextFrames>>,
}

impl RenderNormalizationOverlay {
    pub fn from_document(document: &Document) -> Self {
        Self::from_document_reusing(document, &Self::default())
    }

    pub fn from_document_reusing(document: &Document, previous: &Self) -> Self {
        match Self::from_document_reusing_with_storage(
            document,
            previous,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(crate) fn from_document_reusing_with_storage<S: crate::storage::Storage>(
        document: &Document,
        previous: &Self,
        storage: &S,
    ) -> Result<Self, S::Error> {
        Ok({
            let mut overlay = Self {
                stored_nested_text_frames_by_pointer:
                    stored_nested_text_frames::collect_with_storage(document, storage)?,
                ..Self::default()
            };
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
                                && stored_first_cell_line_says_shrunk(
                                    table,
                                    body_width,
                                    source_width,
                                )
                            {
                                let mut top_path = crate::storage::try_clone(&path, storage)?;
                                top_path.target_control_index = Some(control_index);
                                let table_pointer = table.as_ref() as *const Table as usize;
                                let projection = {
                                    storage
                                        .bytes(std::mem::size_of::<NestedTableWidthProjection>())?;
                                    Arc::new(NestedTableWidthProjection {
                                        path: crate::storage::try_clone(&top_path, storage)?,
                                        source_width,
                                        effective_width: body_width,
                                        width_scale: f64::from(body_width)
                                            / f64::from(source_width),
                                        use_owner_content_box: false,
                                        table_pointer,
                                    })
                                };
                                storage
                                    .reserve_map(&mut overlay.nested_table_widths_by_pointer, 1)?;
                                overlay
                                    .nested_table_widths_by_pointer
                                    .insert(table_pointer, Arc::clone(&projection));
                                storage.reserve_map(&mut overlay.nested_table_widths_by_path, 1)?;
                                overlay
                                    .nested_table_widths_by_path
                                    .insert(top_path, projection);
                            }
                        }
                        overlay.collect_nested_tables_with_storage(
                            table,
                            path,
                            control_index,
                            previous,
                            storage,
                        )?;
                    }
                }
            }
            overlay
        })
    }

    pub(crate) fn stored_nested_text_frames(
        &self,
        table: &Table,
    ) -> Option<Arc<StoredNestedTextFrames>> {
        if self.table_text_reflowed(table) {
            return None;
        }
        self.stored_nested_text_frames_by_pointer
            .get(&(table as *const Table as usize))
            .cloned()
    }

    fn collect_nested_tables(
        &mut self,
        owner_table: &Table,
        path: RenderPath,
        owner_control_index: usize,
        previous: &Self,
    ) {
        match self.collect_nested_tables_with_storage(
            owner_table,
            path,
            owner_control_index,
            previous,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(crate) fn collect_nested_tables_with_storage<S: crate::storage::Storage>(
        &mut self,
        owner_table: &Table,
        path: RenderPath,
        owner_control_index: usize,
        previous: &Self,
        storage: &S,
    ) -> Result<(), S::Error> {
        for (cell_index, cell) in owner_table.cells.iter().enumerate() {
            if cell.width >= 0x8000_0000 {
                continue;
            }
            // 폭을 바꿔 저장한 부모 셀의 상속 프레임은 자식의 현재 총높이를
            // 인증하지 않는다. 자식의 셀·글줄은 보존하되 그 선언 총높이에
            // 맞추려고 내용/행 선언으로 얻은 자연 높이를 다시 축소하지 않는다.
            let host_reflowed = crate::renderer::typeset::is_reparsed_single_column_cell_split_row(
                owner_table,
                usize::from(cell.row),
            );
            for (paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
                for (control_index, control) in paragraph.controls.iter().enumerate() {
                    let Control::Table(nested) = control else {
                        continue;
                    };
                    if host_reflowed {
                        storage.reserve_set(&mut self.nested_tables_with_reflowed_hosts, 1)?;
                        self.nested_tables_with_reflowed_hosts
                            .insert(nested.as_ref() as *const Table as usize);
                    }

                    let mut nested_path = crate::storage::try_clone(&path, storage)?;
                    storage.push(
                        &mut nested_path.entries,
                        RenderPathEntry::TableCell {
                            control_index: owner_control_index,
                            cell_index,
                            paragraph_index,
                        },
                    )?;
                    nested_path.target_control_index = Some(control_index);

                    let source_width = nested.common.width;
                    // 비-TAC 중첩 표의 절대 폭은 원본 개체가 소유한다. 부모보다 높은
                    // 마지막 1×1 자식이라는 형상은 부모 폭으로 늘릴 근거가 아니다.
                    let keeps_legacy_near_fit_projection = !nested.common.treat_as_char
                        && source_width > 0
                        && u64::from(source_width) < u64::from(cell.width)
                        && f64::from(source_width)
                            >= f64::from(cell.width) * NESTED_STRETCH_MIN_RATIO;
                    if keeps_legacy_near_fit_projection {
                        let effective_width = cell.width;
                        let table_pointer = nested.as_ref() as *const Table as usize;
                        let projection = if let Some(projection) = previous
                            .nested_table_widths_by_path
                            .get(&nested_path)
                            .filter(|projection| {
                                projection.source_width == source_width
                                    && projection.effective_width == effective_width
                                    && !projection.use_owner_content_box
                                    && projection.table_pointer == table_pointer
                            })
                            .map(Arc::clone)
                        {
                            projection
                        } else {
                            storage.bytes(std::mem::size_of::<NestedTableWidthProjection>())?;
                            Arc::new(NestedTableWidthProjection {
                                path: crate::storage::try_clone(&nested_path, storage)?,
                                source_width,
                                effective_width,
                                width_scale: f64::from(effective_width) / f64::from(source_width),
                                use_owner_content_box: false,
                                table_pointer,
                            })
                        };
                        storage.reserve_map(&mut self.nested_table_widths_by_pointer, 1)?;
                        self.nested_table_widths_by_pointer
                            .insert(projection.table_pointer, Arc::clone(&projection));
                        storage.reserve_map(&mut self.nested_table_widths_by_path, 1)?;
                        self.nested_table_widths_by_path.insert(
                            crate::storage::try_clone(&nested_path, storage)?,
                            projection,
                        );
                    }

                    self.collect_nested_tables_with_storage(
                        nested,
                        nested_path,
                        control_index,
                        previous,
                        storage,
                    )?;
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub fn nested_table_width_scale(&self, table: &Table) -> f64 {
        let key = table as *const Table as usize;
        self.nested_table_widths_by_pointer
            .get(&key)
            .map(|projection| projection.width_scale)
            .unwrap_or(1.0)
    }

    /// 현재 원본 경로에 등록된 투영의 내용 상자 계약을 조회한다.
    /// child의 선언 높이·near-fit 비율로 owner 폭을 대신 선택하지 않는다.
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
        match self.register_text_reflowed_table_with_storage(table, &crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(crate) fn register_text_reflowed_table_with_storage<S: crate::storage::Storage>(
        &mut self,
        table: &Table,
        storage: &S,
    ) -> Result<(), S::Error> {
        storage.reserve_set(&mut self.text_reflowed_tables_by_pointer, 1)?;
        // 편집 명령이 표 전체를 다시 조판한 경우에는 셀별 render-only 범위로
        // 축소하지 않는다. 이 명시적 등록은 아래의 관측 결과보다 우선한다.
        self.rendered_cell_frame_changes
            .remove(&(table as *const Table as usize));
        self.text_reflowed_tables_by_pointer
            .insert(table as *const Table as usize);

        Ok(())
    }

    /// render-only 재조판으로 문자 줄 소속이 달라진 표는 원본 물리 컷의
    /// 캐시를 다시 인증할 수 없다. 같은 결과를 쓰는 원본·파생 포인터와
    /// 그 조각을 포함한 조상 표를 함께 갱신한다. 편집 IR은 변경하지 않는다.
    pub(crate) fn register_rendered_text_partitions(
        &mut self,
        source: &[crate::model::paragraph::Paragraph],
        rendered: &[crate::model::paragraph::Paragraph],
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> (bool, bool) {
        match self.register_rendered_text_partitions_with_storage(
            source,
            rendered,
            styles,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    pub(crate) fn register_rendered_text_partitions_with_storage<S: crate::storage::Storage>(
        &mut self,
        source: &[crate::model::paragraph::Paragraph],
        rendered: &[crate::model::paragraph::Paragraph],
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        storage: &S,
    ) -> Result<(bool, bool), S::Error> {
        let mut changed = source.len() != rendered.len();
        let mut flow_changed = changed;
        let mut upstream_flow_changed = false;
        for (original, current) in source.iter().zip(rendered) {
            if original.column_type != crate::model::paragraph::ColumnBreakType::None
                || styles
                    .para_styles
                    .get(original.para_shape_id as usize)
                    .is_some_and(|style| style.page_break_before)
            {
                upstream_flow_changed = false;
            }
            changed |= original.line_segs.len() != current.line_segs.len()
                || (0..original.line_segs.len()).any(|index| {
                    original.line_seg_text_start(index) != current.line_seg_text_start(index)
                });
            let extent = |paragraph: &crate::model::paragraph::Paragraph| {
                paragraph
                    .line_segs
                    .first()
                    .zip(paragraph.line_segs.last())
                    .map(|(first, last)| {
                        i64::from(last.vertical_pos)
                            + i64::from(last.line_height)
                            + i64::from(last.line_spacing)
                            - i64::from(first.vertical_pos)
                    })
            };
            let mut paragraph_flow_changed = extent(original) != extent(current)
                || original.line_segs.len() != current.line_segs.len()
                || original
                    .line_segs
                    .iter()
                    .zip(&current.line_segs)
                    .any(|(a, b)| {
                        a.line_height != b.line_height || a.line_spacing != b.line_spacing
                    });
            for (old, new) in original.controls.iter().zip(&current.controls) {
                let (Control::Table(old), Control::Table(new)) = (old, new) else {
                    continue;
                };
                let old_pointer = old.as_ref() as *const Table as usize;
                let new_pointer = new.as_ref() as *const Table as usize;
                // 짧은 저장 줄을 실제 인라인 상자로 확장한 출처를 보존한다.
                // 투영 뒤에는 줄 높이가 표를 덮으므로 숫자만 다시 비교하면
                // 원본의 누락 밴드를 정상 저장 프레임으로 오인한다.
                if old.common.treat_as_char && new.common.treat_as_char {
                    if let ([before], [after]) =
                        (original.line_segs.as_slice(), current.line_segs.as_slice())
                    {
                        let source_box_height = i64::from(old.common.height)
                            + i64::from(old.outer_margin_top)
                            + i64::from(old.outer_margin_bottom);
                        if i64::from(before.line_height) < source_box_height
                            && after.line_height > before.line_height
                        {
                            storage.insert_set(
                                &mut self.expanded_inline_host_tables_by_pointer,
                                old_pointer,
                            )?;
                            storage.insert_set(
                                &mut self.expanded_inline_host_tables_by_pointer,
                                new_pointer,
                            )?;
                        }
                    }
                }
                if !self.rendered_cell_frame_changes.contains_key(&old_pointer) {
                    storage.reserve_map(&mut self.rendered_cell_frame_changes, 1)?;
                }
                self.rendered_cell_frame_changes
                    .entry(old_pointer)
                    .or_default();
                if !self.rendered_cell_frame_changes.contains_key(&new_pointer) {
                    storage.reserve_map(&mut self.rendered_cell_frame_changes, 1)?;
                }
                self.rendered_cell_frame_changes
                    .entry(new_pointer)
                    .or_default();
                let mut table_changed = old.cells.len() != new.cells.len();
                let mut table_flow_changed = table_changed;
                for (old_cell, new_cell) in old.cells.iter().zip(&new.cells) {
                    let (text, flow) = self.register_rendered_text_partitions_with_storage(
                        &old_cell.paragraphs,
                        &new_cell.paragraphs,
                        styles,
                        storage,
                    )?;
                    if text || flow {
                        storage.insert_set(
                            self.rendered_cell_frame_changes
                                .get_mut(&old_pointer)
                                .unwrap(),
                            (old_cell.row, old_cell.col),
                        )?;
                        storage.insert_set(
                            self.rendered_cell_frame_changes
                                .get_mut(&new_pointer)
                                .unwrap(),
                            (new_cell.row, new_cell.col),
                        )?;
                    }
                    table_changed |= text;
                    table_flow_changed |= flow;
                }
                if table_changed {
                    storage.insert_set(&mut self.text_reflowed_tables_by_pointer, old_pointer)?;
                    storage.insert_set(&mut self.text_reflowed_tables_by_pointer, new_pointer)?;
                }
                if upstream_flow_changed {
                    self.register_rebased_source_frames_with_storage(old, new, storage)?;
                }
                changed |= table_changed;
                paragraph_flow_changed |= table_flow_changed;
            }
            flow_changed |= paragraph_flow_changed;
            upstream_flow_changed |= paragraph_flow_changed;
        }
        Ok((changed, flow_changed))
    }

    /// 앞 문단의 점유가 바뀌면 후속 block 표의 저장 쪽 컷도 같은 흐름의
    /// 결과가 아니다. 명시적 쪽/단 경계까지 그 의존성을 전달한다.
    /// 인라인 표의 내부 상자와 쪽 컷이 없는 표의 독립 기하는 유지한다.
    fn register_rebased_source_frames(&mut self, source: &Table, rendered: &Table) -> bool {
        match self.register_rebased_source_frames_with_storage(
            source,
            rendered,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    fn register_rebased_source_frames_with_storage<S: crate::storage::Storage>(
        &mut self,
        source: &Table,
        rendered: &Table,
        storage: &S,
    ) -> Result<bool, S::Error> {
        if source.common.treat_as_char
            || source.common.text_wrap != crate::model::shape::TextWrap::TopAndBottom
            || source.page_break != crate::model::table::TablePageBreak::RowBreak
        {
            return Ok(false);
        }
        let mut frames = source.cells.iter().any(|cell| {
            let mut previous = None;
            cell.paragraphs
                .iter()
                .flat_map(|paragraph| &paragraph.line_segs)
                .any(|line| {
                    let boundary = previous.is_some_and(|vpos| {
                        line.is_first_line_of_page()
                            || line.is_first_line_of_column()
                            || line.vertical_pos < vpos
                    });
                    previous = Some(line.vertical_pos);
                    boundary
                })
        });
        for (old_cell, new_cell) in source.cells.iter().zip(&rendered.cells) {
            for (old_para, new_para) in old_cell.paragraphs.iter().zip(&new_cell.paragraphs) {
                for (old, new) in old_para.controls.iter().zip(&new_para.controls) {
                    if let (Control::Table(old), Control::Table(new)) = (old, new) {
                        frames |=
                            self.register_rebased_source_frames_with_storage(old, new, storage)?;
                    }
                }
            }
        }
        if frames {
            // 앞 흐름의 변화는 표의 절대 쪽 프레임에 전파된다. 실제로 바뀐
            // 셀의 집합은 유지하여 이웃 셀의 상대 줄 원점·정렬을 보존한다.
            for table in [source, rendered] {
                let pointer = table as *const Table as usize;
                storage.insert_set(&mut self.text_reflowed_tables_by_pointer, pointer)?;
                storage.insert_set(&mut self.rebased_table_frames_by_pointer, pointer)?;
            }
        }
        Ok(frames)
    }

    #[inline]
    pub(crate) fn table_text_reflowed(&self, table: &Table) -> bool {
        self.text_reflowed_tables_by_pointer
            .contains(&(table as *const Table as usize))
    }

    /// 실제 표 상자로 확장한 줄 뒤에서는 옛 저장 줄의 spill 허용을 재사용하지 않는다.
    pub(crate) fn inline_host_frame_expanded(&self, table: &Table) -> bool {
        self.expanded_inline_host_tables_by_pointer
            .contains(&(table as *const Table as usize))
    }

    /// 셀 범위가 확인되지 않은 전체 재조판은 원본 개체 프레임도 무효화한다.
    /// 셀별 읽기 전용 정규화는 글줄 출처만 바꾸므로 호스트 앵커가 증명하는
    /// 본문 종료선과 구분한다. 명시적 편집 등록은 셀 범위를 지워 우선한다.
    pub(crate) fn table_frame_reflowed_without_cell_scope(&self, table: &Table) -> bool {
        self.rebased_table_frames_by_pointer
            .contains(&(table as *const Table as usize))
            || (self.table_text_reflowed(table)
                && !self
                    .rendered_cell_frame_changes
                    .contains_key(&(table as *const Table as usize)))
    }

    /// 앞 흐름에서 이월된 표의 저장 쪽 프레임은 셀 글줄이 같아도 무효다.
    /// 상대 글줄 프로그램이 같은지는 별도 질의로 보존한다.
    pub(crate) fn cell_frame_reflowed(
        &self,
        table: &Table,
        cell: &crate::model::table::Cell,
    ) -> bool {
        self.rebased_table_frames_by_pointer
            .contains(&(table as *const Table as usize))
            || self.cell_text_partition_reflowed(table, cell)
    }

    /// 상대 글줄 소속의 변경만 확인한다. 앞 흐름의 쪽 원점 이동을 이웃 셀의
    /// 문자 재조판으로 취급하지 않는다. 명시적 전체 편집은 기존 fallback을 쓴다.
    pub(crate) fn cell_text_partition_reflowed(&self, table: &Table, cell: &Cell) -> bool {
        self.rendered_cell_frame_changes
            .get(&(table as *const Table as usize))
            .map_or_else(
                || self.table_text_reflowed(table),
                |cells| cells.contains(&(cell.row, cell.col)),
            )
    }

    /// 실제 render-only 줄 구성 변경의 양성 출처다. 편집·출처 미확정
    /// 전체 재조판은 이 집합에 없으므로 현재 프로그램의 셀 프레임으로 승격하지 않는다.
    pub(crate) fn cell_has_current_render_partition(&self, table: &Table, cell: &Cell) -> bool {
        self.rendered_cell_frame_changes
            .get(&(table as *const Table as usize))
            .is_some_and(|cells| cells.contains(&(cell.row, cell.col)))
    }

    /// 물리 행을 덮는 셀만 그 행의 저장 프레임을 무효화할 수 있다.
    pub(crate) fn row_frame_reflowed(&self, table: &Table, row: usize) -> bool {
        table.cells.iter().any(|cell| {
            usize::from(cell.row) <= row
                && row < usize::from(cell.row) + usize::from(cell.row_span)
                && self.cell_frame_reflowed(table, cell)
        })
    }

    /// 인증할 첫 물리 상자에 실제로 들어가는 행의 출처만 확인한다.
    pub(crate) fn frame_prefix_reflowed(
        &self,
        table: &Table,
        rows: &[f64],
        spacing: f64,
        height: f64,
    ) -> bool {
        let mut top = 0.0;
        for (row, row_height) in rows.iter().enumerate() {
            if top >= height {
                break;
            }
            if self.row_frame_reflowed(table, row) {
                return true;
            }
            top += row_height + spacing;
        }
        false
    }

    /// 글자 재조판 여부와 별개인 부모 호스트 프레임의 무효화 출처다.
    #[inline]
    pub(crate) fn nested_table_has_reflowed_host(&self, table: &Table) -> bool {
        self.nested_tables_with_reflowed_hosts
            .contains(&(table as *const Table as usize))
    }

    /// 부모 폭 변경 전의 개체 선언은 현재 첫 조각의 물리 프레임을 인증하지 않는다.
    /// 측정·행 유닛·실제 배치는 같은 출처 판정을 거친 결과를 소비한다.
    pub(crate) fn stored_opening_row_frame(
        &self,
        table: &Table,
    ) -> Option<super::float_placement::StoredOpeningRowFrame> {
        if self.table_text_reflowed(table) || self.nested_table_has_reflowed_host(table) {
            return None;
        }
        super::float_placement::stored_plain_opening_row_frame(table)
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
    use crate::model::shape::{TextWrap, VertRelTo};
    use crate::model::table::TablePageBreak;

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
    fn parent_and_child_height_order_does_not_expand_absolute_child_width() {
        let document = short_rowbreak_document(1_912, 2_000, 1_000, 2_000);
        let overlay = RenderNormalizationOverlay::from_document(&document);
        let nested = nested_table_at_final_row(&document);
        assert!(overlay
            .projection_for_path(&short_rowbreak_nested_path())
            .is_none());
        assert_eq!(nested.common.width, 1_912, "원본 절대 폭 보존");
        assert_eq!(overlay.nested_table_width_scale(nested), 1.0);
        assert!(!overlay.uses_owner_content_box(nested));

        // 높이의 대소관계를 뒤집어도 자식의 절대 폭 계약은 같다.
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
