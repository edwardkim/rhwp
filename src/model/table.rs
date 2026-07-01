//! 표 (Table, Cell, Row)

use super::paragraph::Paragraph;
use super::shape::{common_obj_offsets, Caption};
use super::*;

pub const CELL_FLAG_HAS_MARGIN: u16 = 0x0001;
pub const CELL_FLAG_PROTECT: u16 = 0x0002;
pub const CELL_FLAG_HEADER: u16 = 0x0004;
pub const CELL_FLAG_EDITABLE_IN_FORM: u16 = 0x0008;

/// 표 개체 (HWPTAG_TABLE)
#[derive(Debug, Default, Clone)]
pub struct Table {
    /// 속성 비트 플래그
    pub attr: u32,
    /// 행 수
    pub row_count: u16,
    /// 열 수
    pub col_count: u16,
    /// 셀 간격
    pub cell_spacing: HwpUnit16,
    /// 안쪽 여백
    pub padding: Padding,
    /// 행별 셀 수 (HWP 스펙: UINT16[NRows])
    pub row_sizes: Vec<HwpUnit16>,
    /// 테두리/배경 ID 참조
    pub border_fill_id: u16,
    /// 영역 속성 목록
    pub zones: Vec<TableZone>,
    /// 셀 목록 (행 우선 순서)
    pub cells: Vec<Cell>,
    /// 2D 그리드 인덱스: grid[row * col_count + col] = Some(cell_idx)
    /// 병합 셀의 span 영역 전체가 앵커 셀 인덱스를 가리킴
    pub cell_grid: Vec<Option<usize>>,
    /// 쪽 경계에서 나눔 (0: 나누지 않음, 1: 셀 단위로 나눔)
    pub page_break: TablePageBreak,
    /// 제목 줄 자동 반복
    pub repeat_header: bool,
    /// 캡션 정보
    pub caption: Option<Caption>,
    /// 공통 객체 속성 (위치, 배치, 크기 등)
    pub common: crate::model::shape::CommonObjAttr,
    /// 바깥 여백 (CommonObjAttr의 오브젝트 바깥 4방향 여백)
    pub outer_margin_left: i16,
    pub outer_margin_right: i16,
    pub outer_margin_top: i16,
    pub outer_margin_bottom: i16,
    /// CTRL_HEADER ctrl_data의 4바이트(attr) 이후 추가 바이트 (라운드트립 보존용)
    pub raw_ctrl_data: Vec<u8>,
    /// HWPTAG_TABLE 레코드의 원본 속성 값 (라운드트립 보존용, 0이면 재구성)
    pub raw_table_record_attr: u32,
    /// HWPTAG_TABLE 레코드의 border_fill_id 이후 추가 바이트 (라운드트립 보존용)
    pub raw_table_record_extra: Vec<u8>,
    /// 구조/내용 변경 시 true → 재측정 필요 (Default: false)
    #[doc(hidden)]
    pub dirty: bool,
    /// Studio 보상 resize로 행별 독립 가로 경계를 보존해야 하는 행.
    #[doc(hidden)]
    pub local_resize_rows: Vec<u16>,
    /// Studio 보상 resize로 열별 독립 세로 경계를 보존해야 하는 열.
    #[doc(hidden)]
    pub local_resize_cols: Vec<u16>,
    /// Studio 로컬 가로 resize 후 셀별 목표 표시 폭(HWPUNIT).
    #[doc(hidden)]
    pub local_resize_cell_widths: Vec<(usize, u32)>,
    /// Studio 로컬 세로 resize 후 셀별 목표 표시 높이(HWPUNIT).
    #[doc(hidden)]
    pub local_resize_cell_heights: Vec<(usize, u32)>,
}

/// 표 쪽 나눔 종류
/// bit 0-1: 0=나누지 않음, 1=셀 단위로 나눔, 2=나눔(행 단위)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TablePageBreak {
    /// 나누지 않음 (0)
    #[default]
    None,
    /// 셀 단위로 나눔 (1) — 행 내부(인트라-로우) 분할 허용
    CellBreak,
    /// 나눔 (2) — 행 경계에서만 나눔 (인트라-로우 분할 없음)
    RowBreak,
}

/// 표 영역 속성
#[derive(Debug, Clone, Default)]
pub struct TableZone {
    /// 시작 열 주소
    pub start_col: u16,
    /// 시작 행 주소
    pub start_row: u16,
    /// 끝 열 주소
    pub end_col: u16,
    /// 끝 행 주소
    pub end_row: u16,
    /// 테두리/배경 ID 참조
    pub border_fill_id: u16,
}

/// 표 셀 (HWPTAG_LIST_HEADER + 셀 속성)
#[derive(Debug, Default, Clone)]
pub struct Cell {
    /// 셀 열 주소 (0부터 시작)
    pub col: u16,
    /// 셀 행 주소 (0부터 시작)
    pub row: u16,
    /// 열 병합 개수
    pub col_span: u16,
    /// 행 병합 개수
    pub row_span: u16,
    /// 셀 폭
    pub width: HwpUnit,
    /// 셀 높이
    pub height: HwpUnit,
    /// 셀 여백
    pub padding: Padding,
    /// 테두리/배경 ID 참조
    pub border_fill_id: u16,
    /// 셀 내 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// LIST_HEADER의 텍스트 영역 폭 참조 (라운드트립 보존용)
    pub list_header_width_ref: u16,
    /// 텍스트 방향 (0: 가로, 1: 세로)
    pub text_direction: u8,
    /// 세로 정렬 (0: top, 1: center, 2: bottom)
    pub vertical_align: VerticalAlign,
    /// 안 여백 지정 여부 (list_attr bit 16)
    /// true: 셀 고유 padding 사용, false: 표 기본 padding 사용
    pub apply_inner_margin: bool,
    /// 제목 셀 여부 (list_attr bit 18)
    pub is_header: bool,
    /// LIST_HEADER 레코드의 34바이트 이후 추가 바이트 (라운드트립 보존용)
    pub raw_list_extra: Vec<u8>,
    /// 셀 필드 이름 (한컴 셀 속성 → 필드 → 필드 이름)
    /// raw_list_extra의 offset 14-15(name_len) + offset 16~(UTF-16LE)에서 추출
    pub field_name: Option<String>,
}

/// 표 셀 행/열 바꿈 복사 데이터.
///
/// `cells[row][col]` 은 원본 범위의 셀 문단 목록이다.
#[derive(Debug, Clone, Default)]
pub struct TableTransposeData {
    pub source_rows: u16,
    pub source_cols: u16,
    pub cells: Vec<Vec<Vec<Paragraph>>>,
}

fn distribute_hwp_units(total: HwpUnit, count: u16) -> Vec<HwpUnit> {
    if count == 0 {
        return Vec::new();
    }
    let base = total / count as u32;
    let remainder = total % count as u32;
    (0..count)
        .map(|idx| base + u32::from(idx < remainder as u16))
        .collect()
}

/// 세로 정렬
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

impl Cell {
    pub fn set_apply_inner_margin(&mut self, value: bool) {
        self.apply_inner_margin = value;
        self.set_list_header_flag(CELL_FLAG_HAS_MARGIN, value);
    }

    pub fn cell_protect(&self) -> bool {
        self.list_header_width_ref & CELL_FLAG_PROTECT != 0
    }

    pub fn set_cell_protect(&mut self, value: bool) {
        self.set_list_header_flag(CELL_FLAG_PROTECT, value);
    }

    pub fn set_header(&mut self, value: bool) {
        self.is_header = value;
        self.set_list_header_flag(CELL_FLAG_HEADER, value);
    }

    pub fn editable_in_form(&self) -> bool {
        self.list_header_width_ref & CELL_FLAG_EDITABLE_IN_FORM != 0
    }

    pub fn set_editable_in_form(&mut self, value: bool) {
        self.set_list_header_flag(CELL_FLAG_EDITABLE_IN_FORM, value);
    }

    fn set_list_header_flag(&mut self, flag: u16, value: bool) {
        if value {
            self.list_header_width_ref |= flag;
        } else {
            self.list_header_width_ref &= !flag;
        }
    }

    /// 빈 셀을 생성한다 (빈 문단 1개 포함).
    pub fn new_empty(
        col: u16,
        row: u16,
        width: HwpUnit,
        height: HwpUnit,
        border_fill_id: u16,
    ) -> Self {
        Cell {
            col,
            row,
            col_span: 1,
            row_span: 1,
            width,
            height,
            border_fill_id,
            paragraphs: vec![Paragraph::new_empty()],
            ..Default::default()
        }
    }

    /// 기존 셀을 템플릿으로 사용하여 빈 셀을 생성한다.
    ///
    /// raw_list_extra, padding, vertical_align 등 메타데이터를 복사하고,
    /// 첫 문단의 raw_header_extra, char_shapes, line_segs 구조를 복사한다.
    pub fn new_from_template(
        col: u16,
        row: u16,
        width: HwpUnit,
        height: HwpUnit,
        template: &Cell,
    ) -> Self {
        // 템플릿 문단의 구조를 복사하되 텍스트는 비움
        let para = if let Some(tpl_para) = template.paragraphs.first() {
            // instanceId를 0으로 초기화 (새 셀의 문단은 고유 ID 불필요)
            let mut raw_header_extra = tpl_para.raw_header_extra.clone();
            if raw_header_extra.len() >= 10 {
                // raw_header_extra[6..10] = instanceId
                raw_header_extra[6..10].copy_from_slice(&[0, 0, 0, 0]);
            }

            Paragraph {
                char_count: 1,        // 빈 문단: 끝 마커(0x000D) 포함
                char_count_msb: true, // 셀 문단은 항상 MSB 설정
                text: String::new(),
                char_shapes: tpl_para.char_shapes.iter().take(1).cloned().collect(),
                line_segs: tpl_para.line_segs.iter().take(1).cloned().collect(),
                para_shape_id: tpl_para.para_shape_id,
                style_id: tpl_para.style_id,
                raw_header_extra,
                has_para_text: false, // 빈 셀은 PARA_TEXT 불필요
                ..Default::default()
            }
        } else {
            Paragraph::new_empty()
        };

        Cell {
            col,
            row,
            col_span: 1,
            row_span: 1,
            width,
            height,
            border_fill_id: template.border_fill_id,
            padding: template.padding,
            list_header_width_ref: template.list_header_width_ref,
            text_direction: template.text_direction,
            vertical_align: template.vertical_align,
            apply_inner_margin: template.apply_inner_margin,
            is_header: template.is_header,
            raw_list_extra: template.raw_list_extra.clone(),
            field_name: None,
            paragraphs: vec![para],
        }
    }
}

impl Table {
    /// [Task #1716] 반복 제목행으로 재사용할 **표 상단의 연속 제목행 블록** `0..H` 를 반환한다.
    ///
    /// 행 r 이 제목행 ⟺ header 셀(`is_header`, rowspan 덮개 포함)이 r 을 덮음. 상단(행 0)부터
    /// 제목행이 연속되는 최대 구간만 반환하고, 표 중간·하단에 흩어진 `is_header` 행은 제외한다.
    /// (일부 문서는 본문 행에도 `header="1"` 을 다수 부여한다. 그 행들까지 반복 대상으로 잡으면
    /// 연속 페이지마다 반복 overhead 가 누적되어 가용 높이가 0이 되고 페이지당 1행 폭주가 발생.)
    pub fn leading_header_rows(&self) -> Vec<usize> {
        let rc = self.row_count as usize;
        if rc == 0 {
            return Vec::new();
        }
        let mut is_header_row = vec![false; rc];
        for cell in &self.cells {
            if !cell.is_header {
                continue;
            }
            let start = cell.row as usize;
            let span = (cell.row_span as usize).max(1);
            for r in start..(start + span).min(rc) {
                is_header_row[r] = true;
            }
        }
        let mut h = 0usize;
        while h < rc && is_header_row[h] {
            h += 1;
        }
        (0..h).collect()
    }

    /// 저장/복구 후 Studio 런타임 힌트가 사라진 행 단위 가로 resize를 보수적으로 추론한다.
    ///
    /// 한컴 HWP5에는 `local_resize_rows` 같은 rhwp 내부 힌트를 저장할 곳이 없다. 따라서
    /// 같은 셀 배치 패턴을 공유하는 행들 중 다수의 폭 벡터와 다른 소수 행만 행 단위
    /// resize 결과로 간주한다. 병합 패턴이 유일한 행은 원본 문서 구조일 가능성이 높아
    /// 추론 대상에서 제외한다.
    pub fn inferred_local_resize_rows(&self) -> Vec<u16> {
        let col_count = self.col_count as usize;
        if col_count == 0 || self.row_count == 0 {
            return Vec::new();
        }

        let explicit_rows = self
            .local_resize_rows
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let mut grouped_rows =
            std::collections::BTreeMap::<Vec<(u16, u16)>, Vec<(u16, Vec<u32>)>>::new();

        for row in 0..self.row_count {
            if explicit_rows.contains(&row) {
                continue;
            }

            let mut row_cells = self
                .cells
                .iter()
                .filter(|cell| cell.row == row && cell.row_span == 1)
                .collect::<Vec<_>>();
            row_cells.sort_by_key(|cell| cell.col);

            let mut next_col = 0u16;
            let mut pattern = Vec::new();
            let mut widths = Vec::new();
            let mut valid = !row_cells.is_empty();

            for cell in row_cells {
                let span = cell.col_span.max(1);
                let end_col = cell.col.saturating_add(span);
                if cell.col != next_col || end_col <= cell.col || end_col as usize > col_count {
                    valid = false;
                    break;
                }

                pattern.push((cell.col, span));
                widths.push(cell.width);
                next_col = end_col;
            }

            if !valid || next_col as usize != col_count {
                continue;
            }

            grouped_rows.entry(pattern).or_default().push((row, widths));
        }

        let mut inferred = std::collections::BTreeSet::new();
        for rows in grouped_rows.values() {
            if rows.len() < 2 {
                continue;
            }

            let mut width_counts = std::collections::BTreeMap::<Vec<u32>, usize>::new();
            for (_, widths) in rows {
                *width_counts.entry(widths.clone()).or_default() += 1;
            }

            let Some((dominant_widths, dominant_count)) =
                width_counts.iter().max_by_key(|(_, count)| **count)
            else {
                continue;
            };
            if *dominant_count < 2 {
                continue;
            }

            let dominant_is_tied = width_counts
                .values()
                .filter(|count| **count == *dominant_count)
                .count()
                > 1;
            if dominant_is_tied {
                continue;
            }

            for (row, widths) in rows {
                if widths != dominant_widths {
                    inferred.insert(*row);
                }
            }
        }

        inferred.into_iter().collect()
    }

    /// 2D 그리드 인덱스를 재구축한다.
    /// 구조 변경(파싱, 행/열 추가/삭제, 병합/분할) 후 호출해야 한다.
    pub fn rebuild_grid(&mut self) {
        let rc = self.row_count as usize;
        let cc = self.col_count as usize;
        self.cell_grid = vec![None; rc * cc];
        for (idx, cell) in self.cells.iter().enumerate() {
            for r in cell.row..(cell.row + cell.row_span) {
                for c in cell.col..(cell.col + cell.col_span) {
                    let gi = (r as usize) * cc + (c as usize);
                    if gi < self.cell_grid.len() {
                        self.cell_grid[gi] = Some(idx);
                    }
                }
            }
        }
    }

    /// O(1) 셀 인덱스 조회. rebuild_grid() 호출 후 사용해야 한다.
    pub fn cell_index_at(&self, row: u16, col: u16) -> Option<usize> {
        let idx = (row as usize) * (self.col_count as usize) + (col as usize);
        self.cell_grid.get(idx)?.as_ref().copied()
    }

    /// O(1) 셀 접근 (불변). rebuild_grid() 호출 후 사용해야 한다.
    pub fn cell_at(&self, row: u16, col: u16) -> Option<&Cell> {
        let idx = (row as usize) * (self.col_count as usize) + (col as usize);
        let &cell_idx = self.cell_grid.get(idx)?.as_ref()?;
        self.cells.get(cell_idx)
    }

    /// O(1) 셀 접근 (가변). rebuild_grid() 호출 후 사용해야 한다.
    pub fn cell_at_mut(&mut self, row: u16, col: u16) -> Option<&mut Cell> {
        let idx = (row as usize) * (self.col_count as usize) + (col as usize);
        let &cell_idx = self.cell_grid.get(idx)?.as_ref()?;
        self.cells.get_mut(cell_idx)
    }

    fn validate_unmerged_rect(
        &self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> Result<(), String> {
        if start_row > end_row || start_col > end_col {
            return Err("셀 범위가 유효하지 않습니다".to_string());
        }
        if end_row >= self.row_count || end_col >= self.col_count {
            return Err(format!(
                "셀 범위 ({},{})~({},{})가 표 크기 {}×{}를 초과합니다",
                start_row, start_col, end_row, end_col, self.row_count, self.col_count
            ));
        }

        for row in start_row..=end_row {
            for col in start_col..=end_col {
                let cell_idx = self
                    .cell_index_at(row, col)
                    .ok_or_else(|| format!("셀 ({},{})을 찾을 수 없습니다", row, col))?;
                let cell = &self.cells[cell_idx];
                if cell.row != row || cell.col != col || cell.row_span != 1 || cell.col_span != 1 {
                    return Err(format!(
                        "병합 셀 ({},{})은 행/열 바꿈 범위에 포함할 수 없습니다",
                        cell.row, cell.col
                    ));
                }
            }
        }

        Ok(())
    }

    /// 직사각형 범위의 셀 문단을 행/열 바꿈 복사용 데이터로 복사한다.
    pub fn copy_transpose_range(
        &self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> Result<TableTransposeData, String> {
        self.validate_unmerged_rect(start_row, start_col, end_row, end_col)?;

        let source_rows = end_row - start_row + 1;
        let source_cols = end_col - start_col + 1;
        let mut cells = Vec::with_capacity(source_rows as usize);

        for row in start_row..=end_row {
            let mut row_cells = Vec::with_capacity(source_cols as usize);
            for col in start_col..=end_col {
                let cell_idx = self
                    .cell_index_at(row, col)
                    .ok_or_else(|| format!("셀 ({},{})을 찾을 수 없습니다", row, col))?;
                row_cells.push(self.cells[cell_idx].paragraphs.clone());
            }
            cells.push(row_cells);
        }

        Ok(TableTransposeData {
            source_rows,
            source_cols,
            cells,
        })
    }

    /// 행/열 바꿈 복사 데이터를 대상 시작 셀부터 붙여넣는다.
    ///
    /// 반환값은 내용이 교체된 `(cell_idx, paragraph_count)` 목록이다.
    pub fn paste_transposed_cells(
        &mut self,
        start_row: u16,
        start_col: u16,
        data: &TableTransposeData,
    ) -> Result<Vec<(usize, usize)>, String> {
        if data.source_rows == 0 || data.source_cols == 0 || data.cells.is_empty() {
            return Err("행/열 바꿈 복사 데이터가 비어 있습니다".to_string());
        }
        if data.cells.len() != data.source_rows as usize
            || data
                .cells
                .iter()
                .any(|row| row.len() != data.source_cols as usize)
        {
            return Err("행/열 바꿈 복사 데이터의 행/열 크기가 일치하지 않습니다".to_string());
        }

        let target_rows = data.source_cols;
        let target_cols = data.source_rows;
        let end_row = start_row
            .checked_add(target_rows - 1)
            .ok_or_else(|| "대상 행 범위가 너무 큽니다".to_string())?;
        let end_col = start_col
            .checked_add(target_cols - 1)
            .ok_or_else(|| "대상 열 범위가 너무 큽니다".to_string())?;
        self.validate_unmerged_rect(start_row, start_col, end_row, end_col)?;

        let mut changed = Vec::with_capacity((target_rows as usize) * (target_cols as usize));
        for source_row in 0..data.source_rows {
            for source_col in 0..data.source_cols {
                let target_row = start_row + source_col;
                let target_col = start_col + source_row;
                let cell_idx = self.cell_index_at(target_row, target_col).ok_or_else(|| {
                    format!("셀 ({},{})을 찾을 수 없습니다", target_row, target_col)
                })?;
                let mut paragraphs = data.cells[source_row as usize][source_col as usize].clone();
                if paragraphs.is_empty() {
                    paragraphs.push(Paragraph::new_empty());
                }
                let para_count = paragraphs.len();
                self.cells[cell_idx].paragraphs = paragraphs;
                changed.push((cell_idx, para_count));
            }
        }

        self.dirty = true;
        Ok(changed)
    }

    /// 병합 없는 전체 표를 제자리에서 전치한다.
    pub fn transpose_unmerged_table_in_place(&mut self) -> Result<Vec<(usize, usize)>, String> {
        if self.row_count == 0 || self.col_count == 0 {
            return Err("행/열을 바꿀 표가 비어 있습니다".to_string());
        }
        self.validate_unmerged_rect(0, 0, self.row_count - 1, self.col_count - 1)?;

        let source_rows = self.row_count;
        let source_cols = self.col_count;
        let source_cells = (0..source_rows)
            .map(|row| {
                (0..source_cols)
                    .map(|col| {
                        let cell_idx = self
                            .cell_index_at(row, col)
                            .ok_or_else(|| format!("셀 ({},{})을 찾을 수 없습니다", row, col))?;
                        Ok(self.cells[cell_idx].clone())
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .collect::<Result<Vec<_>, String>>()?;

        let target_rows = source_cols;
        let target_cols = source_rows;
        let total_width: HwpUnit = self.get_column_widths().iter().sum();
        let target_widths = distribute_hwp_units(total_width.max(1800), target_cols);
        let row_height = self
            .get_row_heights()
            .into_iter()
            .max()
            .unwrap_or(400)
            .max(400);

        let mut cells = Vec::with_capacity((target_rows as usize) * (target_cols as usize));
        for target_row in 0..target_rows {
            for target_col in 0..target_cols {
                let mut cell = source_cells[target_col as usize][target_row as usize].clone();
                cell.row = target_row;
                cell.col = target_col;
                cell.row_span = 1;
                cell.col_span = 1;
                cell.width = target_widths[target_col as usize];
                cell.height = row_height;
                if cell.paragraphs.is_empty() {
                    cell.paragraphs.push(Paragraph::new_empty());
                }
                cells.push(cell);
            }
        }

        self.row_count = target_rows;
        self.col_count = target_cols;
        self.row_sizes = vec![target_cols as i16; target_rows as usize];
        self.cells = cells;
        self.zones.clear();
        self.local_resize_rows.clear();
        self.local_resize_cols.clear();
        self.local_resize_cell_widths.clear();
        self.local_resize_cell_heights.clear();
        self.update_ctrl_dimensions();
        self.rebuild_grid();
        self.dirty = true;

        Ok(self
            .cells
            .iter()
            .enumerate()
            .map(|(idx, cell)| (idx, cell.paragraphs.len()))
            .collect())
    }

    /// raw_ctrl_data 내 CommonObjAttr의 width/height를 재계산하여 갱신한다 (의도된 dual).
    ///
    /// **Dual maintenance 의 이유** (Picture/Shape 와 다른 점):
    /// - **Table**: `serializer/control.rs:461` 가 `table.raw_ctrl_data` 를 그대로 기록 →
    ///   `raw_ctrl_data` 가 **source-of-truth**. 본 함수가 cell 조절 후 갱신 필수.
    /// - **Picture/Shape**: `serializer/control.rs:895` 가 `&serialize_common_obj_attr(&pic.common)`
    ///   으로 매번 재생성 → `self.common` 이 source-of-truth, raw bytes 는 derived.
    ///
    /// Table 만 dual 인 것은 serializer 의 source-of-truth 정책 차이로 의도된 구조.
    /// 추후 모델 통일 (Picture/Shape 정합으로 Table 전환) 시 본 함수도 단순화 가능.
    ///
    /// raw_ctrl_data 레이아웃 (parse_common_obj_attr 정합):
    ///   [0..4] flags, [4..8] v_offset, [8..12] h_offset,
    ///   [12..16] width, [16..20] height, [20..24] z_order,
    ///   [24..32] outer_margin (i16×4), [32..36] instance_id
    pub fn update_ctrl_dimensions(&mut self) {
        if self.raw_ctrl_data.len() < common_obj_offsets::HEIGHT.end {
            return;
        }
        let total_width: HwpUnit = self.get_column_widths().iter().sum();
        let total_height: HwpUnit = self.get_row_heights().iter().sum();
        // (1) serialize source — raw_ctrl_data bytes (HWP 직렬화 시 사용).
        self.raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
        self.raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
        // (2) [Task #1151 v6] paragraph_layout cache — self.common.width/height.
        // v3 helper (calc_sibling_topandbottom_table_reserved_hu) 가 self.common.height 사용.
        // dual maintenance 가 필수 — 한쪽만 갱신 시 stale 결함.
        self.common.width = total_width;
        self.common.height = total_height;
    }

    fn sync_ctrl_height(&mut self, height: HwpUnit) {
        self.common.height = height;
        if self.raw_ctrl_data.len() >= common_obj_offsets::HEIGHT.end {
            self.raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&height.to_le_bytes());
        }
    }

    fn stretched_row_heights(&self) -> Option<Vec<HwpUnit>> {
        let mut heights = self.get_row_heights();
        let raw_sum: u64 = heights.iter().map(|h| *h as u64).sum();
        let target = self.common.height as u64;
        if heights.is_empty() || raw_sum == 0 || target <= raw_sum {
            return None;
        }

        let mut scaled_sum = 0u64;
        for height in &mut heights {
            let scaled = ((*height as u64 * target) + raw_sum / 2) / raw_sum;
            *height = scaled.max(1).min(u32::MAX as u64) as HwpUnit;
            scaled_sum += *height as u64;
        }

        if let Some(last) = heights.last_mut() {
            match target.cmp(&scaled_sum) {
                std::cmp::Ordering::Greater => {
                    let delta = (target - scaled_sum).min(u32::MAX as u64);
                    *last = last.saturating_add(delta as HwpUnit);
                }
                std::cmp::Ordering::Less => {
                    let delta = (scaled_sum - target).min(*last as u64);
                    *last = last.saturating_sub(delta as HwpUnit).max(1);
                }
                std::cmp::Ordering::Equal => {}
            }
        }

        Some(heights)
    }

    /// 열별 폭을 추출한다 (col_span==1인 셀 기준).
    pub fn get_column_widths(&self) -> Vec<HwpUnit> {
        let mut widths = vec![0u32; self.col_count as usize];
        for cell in &self.cells {
            if cell.col_span == 1 && (cell.col as usize) < widths.len() {
                if cell.width > widths[cell.col as usize] {
                    widths[cell.col as usize] = cell.width;
                }
            }
        }
        // 폭이 0인 열은 기본값 1800 HWPUNIT (약 6.35mm)
        for w in &mut widths {
            if *w == 0 {
                *w = 1800;
            }
        }
        widths
    }

    /// 열별 폭(HWPUNIT)을 절대값으로 설정한다.
    ///
    /// `widths.len()` 은 `col_count` 와 같아야 한다. 병합 셀(`col_span > 1`)은
    /// 걸친 열들의 폭 합으로 설정된다. 설정 후 표 전체 크기
    /// (`update_ctrl_dimensions`)와 그리드 인덱스(`rebuild_grid`)를 갱신한다.
    ///
    /// `insert_column` 이 기준 열 폭을 복제해 표를 넓히는 것과 달리, 이 메서드는
    /// 입력한 폭들의 합이 그대로 표 전체 폭이 된다. 페이지 폭에 맞추려면
    /// 합이 본문 폭 이하가 되도록 전달한다.
    pub fn set_column_widths(&mut self, widths: &[HwpUnit]) -> Result<(), String> {
        if widths.len() != self.col_count as usize {
            return Err(format!(
                "열 폭 개수 {} 가 표의 열 수 {} 와 다릅니다",
                widths.len(),
                self.col_count
            ));
        }
        for cell in &mut self.cells {
            let c = cell.col as usize;
            if c >= widths.len() {
                continue;
            }
            let end = (c + cell.col_span as usize).min(widths.len());
            cell.width = widths[c..end].iter().sum();
        }
        self.update_ctrl_dimensions();
        self.rebuild_grid();
        Ok(())
    }

    /// 행별 높이를 추출한다 (row_span==1인 셀 기준).
    /// 높이가 0인 행은 기본값 400으로 대체 (새 셀 생성용).
    pub fn get_row_heights(&self) -> Vec<HwpUnit> {
        let mut heights = self.get_raw_row_heights();
        // 높이가 0인 행은 기본값 400 HWPUNIT
        for h in &mut heights {
            if *h == 0 {
                *h = 400;
            }
        }
        heights
    }

    /// 행별 높이를 추출한다 (fallback 없이 원본 값 그대로).
    /// 병합 시 원본 height=0 (자동 맞춤) 보존용.
    pub fn get_raw_row_heights(&self) -> Vec<HwpUnit> {
        let mut heights = vec![0u32; self.row_count as usize];
        for cell in &self.cells {
            if cell.row_span == 1 && (cell.row as usize) < heights.len() {
                if cell.height > heights[cell.row as usize] {
                    heights[cell.row as usize] = cell.height;
                }
            }
        }
        heights
    }

    /// row_sizes를 행별 실제 셀 개수로 재계산한다.
    fn rebuild_row_sizes(&mut self) {
        self.row_sizes = (0..self.row_count)
            .map(|r| self.cells.iter().filter(|c| c.row == r).count() as i16)
            .collect();
    }

    /// 행을 삽입한다.
    ///
    /// `row_idx`: 기준 행 인덱스, `below`: true면 아래에, false면 위에 삽입.
    /// 반환: Ok(()) 또는 에러 메시지.
    pub fn insert_row(&mut self, row_idx: u16, below: bool) -> Result<(), String> {
        if row_idx >= self.row_count {
            return Err(format!(
                "행 인덱스 {} 범위 초과 (총 {}행)",
                row_idx, self.row_count
            ));
        }

        let stretched_new_row_height = self
            .stretched_row_heights()
            .and_then(|heights| heights.get(row_idx as usize).copied());
        let original_height = self.common.height;
        let target_row = if below { row_idx + 1 } else { row_idx };
        let col_widths = self.get_column_widths();

        // 셀 height용
        let row_heights = self.get_row_heights();
        let new_cell_height: HwpUnit = if (row_idx as usize) < row_heights.len() {
            row_heights[row_idx as usize]
        } else {
            400
        };

        // 병합 셀 확장 + 기존 셀 시프트 (커버리지 맵 생성용으로 먼저 처리)
        // 삽입 지점을 걸치는 병합 셀 추적
        let mut covered_cols = vec![false; self.col_count as usize];

        for cell in &mut self.cells {
            // 병합 셀이 삽입 지점을 걸치는 경우: row_span 확장
            if cell.row < target_row && cell.row + cell.row_span > target_row {
                cell.row_span += 1;
                // 이 셀이 커버하는 열 표시
                for c in cell.col..(cell.col + cell.col_span).min(self.col_count) {
                    covered_cols[c as usize] = true;
                }
            }
            // target_row 이상의 셀은 1행 아래로 시프트
            if cell.row >= target_row {
                cell.row += 1;
            }
        }

        // 새 셀 생성: 병합 셀에 의해 커버되지 않는 열에만
        // 삽입 지점 아래 행의 셀을 템플릿으로 우선 사용 (헤더 행 대신 데이터 행)
        // target_row 아래(+1)의 셀이 원래 데이터 행이므로 먼저 시도, 없으면 위(-1), 그래도 없으면 아무 셀
        for c in 0..self.col_count {
            if !covered_cols[c as usize] {
                let width = col_widths[c as usize];
                let template = self
                    .cells
                    .iter()
                    .find(|cell| cell.col == c && cell.col_span == 1 && cell.row == target_row + 1)
                    .or_else(|| {
                        if target_row > 0 {
                            self.cells.iter().find(|cell| {
                                cell.col == c && cell.col_span == 1 && cell.row == target_row - 1
                            })
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        self.cells
                            .iter()
                            .find(|cell| cell.col == c && cell.col_span == 1)
                    });
                let new_cell = if let Some(tpl) = template {
                    Cell::new_from_template(c, target_row, width, new_cell_height, tpl)
                } else {
                    Cell::new_empty(c, target_row, width, new_cell_height, self.border_fill_id)
                };
                self.cells.push(new_cell);
            }
        }

        // row_count 갱신 및 row_sizes 재계산 (행별 셀 개수)
        self.row_count += 1;
        self.rebuild_row_sizes();

        // 행 우선 순서 정렬
        self.cells.sort_by_key(|c| (c.row, c.col));

        // CommonObjAttr 크기 갱신
        self.update_ctrl_dimensions();
        if let Some(new_row_height) = stretched_new_row_height {
            // 일반 표는 셀 저장 height보다 큰 표시 height를 별도로 가진다.
            // 행 추가 시 표시 기준 행 높이를 더해 표가 납작해지지 않도록 보존한다.
            self.sync_ctrl_height(original_height.saturating_add(new_row_height));
        }

        // 그리드 인덱스 재구축
        self.rebuild_grid();

        Ok(())
    }

    /// 열을 삽입한다.
    ///
    /// `col_idx`: 기준 열 인덱스, `right`: true면 오른쪽에, false면 왼쪽에 삽입.
    pub fn insert_column(&mut self, col_idx: u16, right: bool) -> Result<(), String> {
        if col_idx >= self.col_count {
            return Err(format!(
                "열 인덱스 {} 범위 초과 (총 {}열)",
                col_idx, self.col_count
            ));
        }

        let original_height = self.common.height;
        let target_col = if right { col_idx + 1 } else { col_idx };
        let col_widths = self.get_column_widths();
        let row_heights = self.get_row_heights();
        let new_col_width = col_widths[col_idx as usize];

        // 병합 셀 확장 + 기존 셀 시프트
        let mut covered_rows = vec![false; self.row_count as usize];

        for cell in &mut self.cells {
            // 병합 셀이 삽입 지점을 걸치는 경우: col_span 확장
            if cell.col < target_col && cell.col + cell.col_span > target_col {
                cell.col_span += 1;
                cell.width += new_col_width;
                // 이 셀이 커버하는 행 표시
                for r in cell.row..(cell.row + cell.row_span).min(self.row_count) {
                    covered_rows[r as usize] = true;
                }
            }
            // target_col 이상의 셀은 1열 오른쪽으로 시프트
            if cell.col >= target_col {
                cell.col += 1;
            }
        }

        // 새 셀 생성: 병합 셀에 의해 커버되지 않는 행에만
        // 삽입 지점 오른쪽 열의 셀을 템플릿으로 우선 사용, 없으면 왼쪽, 그래도 없으면 아무 셀
        for r in 0..self.row_count {
            if !covered_rows[r as usize] {
                let height = row_heights[r as usize];
                let template = self
                    .cells
                    .iter()
                    .find(|cell| cell.row == r && cell.row_span == 1 && cell.col == target_col + 1)
                    .or_else(|| {
                        if target_col > 0 {
                            self.cells.iter().find(|cell| {
                                cell.row == r && cell.row_span == 1 && cell.col == target_col - 1
                            })
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        self.cells
                            .iter()
                            .find(|cell| cell.row == r && cell.row_span == 1)
                    });
                let new_cell = if let Some(tpl) = template {
                    Cell::new_from_template(target_col, r, new_col_width, height, tpl)
                } else {
                    Cell::new_empty(target_col, r, new_col_width, height, self.border_fill_id)
                };
                self.cells.push(new_cell);
            }
        }

        // col_count 갱신 및 row_sizes 재계산 (행별 셀 개수)
        self.col_count += 1;
        self.rebuild_row_sizes();

        // 행 우선 순서 정렬
        self.cells.sort_by_key(|c| (c.row, c.col));

        // CommonObjAttr 크기 갱신
        self.update_ctrl_dimensions();
        if original_height > 0 {
            // 열 추가는 행 수를 바꾸지 않으므로 표 외곽 높이는 기존 값을 유지한다.
            self.common.height = original_height;
            if self.raw_ctrl_data.len() >= common_obj_offsets::HEIGHT.end {
                self.raw_ctrl_data[common_obj_offsets::HEIGHT]
                    .copy_from_slice(&original_height.to_le_bytes());
            }
        }

        // 그리드 인덱스 재구축
        self.rebuild_grid();

        Ok(())
    }

    /// 행을 삭제한다.
    ///
    /// `row_idx`: 삭제할 행 인덱스. 최소 1행은 유지 (row_count == 1이면 에러).
    pub fn delete_row(&mut self, row_idx: u16) -> Result<(), String> {
        if row_idx >= self.row_count {
            return Err(format!(
                "행 인덱스 {} 범위 초과 (총 {}행)",
                row_idx, self.row_count
            ));
        }
        if self.row_count <= 1 {
            return Err("최소 1행은 유지해야 합니다".to_string());
        }

        let stretched_deleted_row_height = self
            .stretched_row_heights()
            .and_then(|heights| heights.get(row_idx as usize).copied());
        let original_height = self.common.height;

        // 삭제 행을 걸치는 병합 셀: row_span 축소
        for cell in &mut self.cells {
            if cell.row < row_idx && cell.row + cell.row_span > row_idx {
                cell.row_span -= 1;
            }
        }

        // 삭제 대상 행의 셀 제거 (해당 행에 앵커가 있고 row_span==1인 셀)
        self.cells
            .retain(|cell| !(cell.row == row_idx && cell.row_span == 1));

        // 삭제 행에 앵커가 있지만 row_span > 1인 병합 셀: 다음 행으로 이동, row_span 축소
        for cell in &mut self.cells {
            if cell.row == row_idx && cell.row_span > 1 {
                cell.row_span -= 1;
            }
        }

        // 삭제 행 아래 셀: row -= 1
        for cell in &mut self.cells {
            if cell.row > row_idx {
                cell.row -= 1;
            }
        }

        // row_count 갱신 및 row_sizes 재계산
        self.row_count -= 1;
        self.rebuild_row_sizes();

        // 행 우선 순서 정렬
        self.cells.sort_by_key(|c| (c.row, c.col));

        // CommonObjAttr 크기 갱신
        self.update_ctrl_dimensions();
        if let Some(deleted_row_height) = stretched_deleted_row_height {
            // 일반 표의 표시 높이는 셀 저장 height 합보다 크므로 삭제 행의 표시
            // 높이만큼 외곽 height를 줄여 한컴식 비례를 유지한다.
            let raw_sum: HwpUnit = self.get_row_heights().iter().sum();
            self.sync_ctrl_height(
                original_height
                    .saturating_sub(deleted_row_height)
                    .max(raw_sum),
            );
        }

        // 그리드 인덱스 재구축
        self.rebuild_grid();

        Ok(())
    }

    /// 열을 삭제한다.
    ///
    /// `col_idx`: 삭제할 열 인덱스. 최소 1열은 유지 (col_count == 1이면 에러).
    pub fn delete_column(&mut self, col_idx: u16) -> Result<(), String> {
        if col_idx >= self.col_count {
            return Err(format!(
                "열 인덱스 {} 범위 초과 (총 {}열)",
                col_idx, self.col_count
            ));
        }
        if self.col_count <= 1 {
            return Err("최소 1열은 유지해야 합니다".to_string());
        }

        let original_height = self.common.height;

        // 삭제 열의 폭 (셀 width 축소용)
        let col_widths = self.get_column_widths();
        let deleted_width = col_widths[col_idx as usize];

        // 삭제 열을 걸치는 병합 셀: col_span 축소, width 축소
        for cell in &mut self.cells {
            if cell.col < col_idx && cell.col + cell.col_span > col_idx {
                cell.col_span -= 1;
                if cell.width >= deleted_width {
                    cell.width -= deleted_width;
                }
            }
        }

        // 삭제 대상 열의 셀 제거 (해당 열에 앵커가 있고 col_span==1인 셀)
        self.cells
            .retain(|cell| !(cell.col == col_idx && cell.col_span == 1));

        // 삭제 열에 앵커가 있지만 col_span > 1인 병합 셀: 다음 열로 이동, col_span 축소
        for cell in &mut self.cells {
            if cell.col == col_idx && cell.col_span > 1 {
                cell.col_span -= 1;
                if cell.width >= deleted_width {
                    cell.width -= deleted_width;
                }
            }
        }

        // 삭제 열 오른쪽 셀: col -= 1
        for cell in &mut self.cells {
            if cell.col > col_idx {
                cell.col -= 1;
            }
        }

        // col_count 갱신 및 row_sizes 재계산
        self.col_count -= 1;
        self.rebuild_row_sizes();

        // 행 우선 순서 정렬
        self.cells.sort_by_key(|c| (c.row, c.col));

        // CommonObjAttr 크기 갱신
        self.update_ctrl_dimensions();
        if original_height > 0 {
            // 열 삭제는 행 수를 바꾸지 않으므로 표 외곽 높이는 기존 값을 유지한다.
            self.common.height = original_height;
            if self.raw_ctrl_data.len() >= common_obj_offsets::HEIGHT.end {
                self.raw_ctrl_data[common_obj_offsets::HEIGHT]
                    .copy_from_slice(&original_height.to_le_bytes());
            }
        }

        // 그리드 인덱스 재구축
        self.rebuild_grid();

        Ok(())
    }

    /// 직사각형 범위의 셀을 병합한다.
    ///
    /// 범위: (start_col, start_row) ~ (end_col, end_row) (모두 포함).
    /// 좌상단 셀이 병합 결과가 되고, 나머지 셀은 제거된다.
    pub fn merge_cells(
        &mut self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> Result<(), String> {
        // 범위 유효성 검증
        if start_row > end_row || start_col > end_col {
            return Err("병합 범위가 유효하지 않습니다".to_string());
        }
        if end_row >= self.row_count || end_col >= self.col_count {
            return Err(format!(
                "병합 범위 ({},{})~({},{})가 표 크기 {}×{}를 초과합니다",
                start_row, start_col, end_row, end_col, self.row_count, self.col_count
            ));
        }

        // 범위 내 셀이 모두 범위 안에 들어오는지 확인 (부분 겹침 방지)
        for cell in &self.cells {
            let cell_end_row = cell.row + cell.row_span - 1;
            let cell_end_col = cell.col + cell.col_span - 1;

            // 셀이 범위와 겹치는지 확인
            let overlaps = cell.col <= end_col
                && cell_end_col >= start_col
                && cell.row <= end_row
                && cell_end_row >= start_row;

            if overlaps {
                // 겹치는 셀은 범위 안에 완전히 포함되어야 함
                let contained = cell.col >= start_col
                    && cell_end_col <= end_col
                    && cell.row >= start_row
                    && cell_end_row <= end_row;
                if !contained {
                    return Err(format!(
                        "셀 ({},{}) span ({},{})이 병합 범위를 벗어납니다",
                        cell.row, cell.col, cell.row_span, cell.col_span
                    ));
                }
            }
        }

        // 주 셀 존재 확인
        if !self
            .cells
            .iter()
            .any(|c| c.col == start_col && c.row == start_row)
        {
            return Err(format!(
                "주 셀 ({},{})을 찾을 수 없습니다",
                start_row, start_col
            ));
        }

        // 열폭/행높이 합산 (원본 값 보존: 0은 fallback 없이 그대로 유지)
        let col_widths = self.get_column_widths();
        let raw_row_heights = self.get_raw_row_heights();
        let new_width: HwpUnit = (start_col..=end_col)
            .map(|c| col_widths.get(c as usize).copied().unwrap_or(0))
            .sum();
        let new_height: HwpUnit = (start_row..=end_row)
            .map(|r| raw_row_heights.get(r as usize).copied().unwrap_or(0))
            .sum();

        // 비주 셀의 비어있지 않은 문단 수집 (모든 메타데이터 보존)
        let mut extra_paragraphs: Vec<Paragraph> = Vec::new();
        for cell in &self.cells {
            if cell.col == start_col && cell.row == start_row {
                continue; // 주 셀 스킵
            }
            let in_range = cell.col >= start_col
                && cell.col <= end_col
                && cell.row >= start_row
                && cell.row <= end_row;
            if in_range {
                for para in &cell.paragraphs {
                    if !para.text.is_empty() {
                        extra_paragraphs.push(Paragraph {
                            text: para.text.clone(),
                            char_count: para.char_count,
                            char_count_msb: para.char_count_msb,
                            control_mask: para.control_mask,
                            char_offsets: para.char_offsets.clone(),
                            char_shapes: para.char_shapes.clone(),
                            line_segs: para.line_segs.clone(),
                            range_tags: para.range_tags.clone(),
                            para_shape_id: para.para_shape_id,
                            style_id: para.style_id,
                            raw_header_extra: para.raw_header_extra.clone(),
                            has_para_text: para.has_para_text,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // 비주 셀 제거 (한컴 오피스와 동일하게 셀을 실제로 제거)
        self.cells.retain(|cell| {
            if cell.col == start_col && cell.row == start_row {
                return true; // 주 셀 유지
            }
            let in_range = cell.col >= start_col
                && cell.col <= end_col
                && cell.row >= start_row
                && cell.row <= end_row;
            !in_range // 범위 밖 셀 유지, 범위 내 비주 셀 제거
        });

        // 주 셀 갱신
        let primary = self
            .cells
            .iter_mut()
            .find(|c| c.col == start_col && c.row == start_row)
            .expect("주 셀이 retain 후에도 존재해야 합니다");

        primary.col_span = end_col - start_col + 1;
        primary.row_span = end_row - start_row + 1;
        // raw_list_extra[0..4]에 참조 폭이 저장되어 있으면 갱신
        if primary.raw_list_extra.len() >= 4 {
            let old_ref_width =
                u32::from_le_bytes(primary.raw_list_extra[0..4].try_into().unwrap());
            if old_ref_width == primary.width {
                primary.raw_list_extra[0..4].copy_from_slice(&new_width.to_le_bytes());
            }
        }
        primary.width = new_width;
        primary.height = new_height;

        // 비어있지 않은 문단 추가
        for para in extra_paragraphs {
            primary.paragraphs.push(para);
        }

        // 행 우선 순서 정렬
        self.cells.sort_by_key(|c| (c.row, c.col));

        // row_sizes 갱신 (행별 실제 셀 개수)
        self.rebuild_row_sizes();

        // 그리드 인덱스 재구축
        self.rebuild_grid();

        Ok(())
    }

    /// 병합된 셀을 나눈다 (merge_cells의 역연산).
    ///
    /// 대상 셀의 col_span > 1 또는 row_span > 1이어야 한다.
    /// 원본 셀은 (target_col, target_row)에 col_span=1, row_span=1로 축소되고,
    /// 나머지 위치에 새 빈 셀이 생성된다.
    pub fn split_cell(&mut self, target_row: u16, target_col: u16) -> Result<(), String> {
        // 대상 셀 찾기 및 검증
        let cell_idx = self
            .cells
            .iter()
            .position(|c| c.col == target_col && c.row == target_row)
            .ok_or_else(|| format!("셀 ({},{})을 찾을 수 없습니다", target_row, target_col))?;

        let orig_col_span = self.cells[cell_idx].col_span;
        let orig_row_span = self.cells[cell_idx].row_span;
        let orig_width = self.cells[cell_idx].width;
        let orig_height = self.cells[cell_idx].height;

        if orig_col_span <= 1 && orig_row_span <= 1 {
            return Err("병합되지 않은 셀은 나눌 수 없습니다".to_string());
        }

        // 열폭 계산: 다른 행의 col_span==1 셀에서 실제 폭 추출, 없으면 균등 분배
        let col_widths = self.get_column_widths();
        let split_col_widths: Vec<HwpUnit> = {
            let has_real = (target_col..target_col + orig_col_span).all(|c| {
                self.cells.iter().any(|cell| {
                    cell.col == c
                        && cell.col_span == 1
                        && !(cell.col == target_col && cell.row == target_row)
                })
            });
            if has_real {
                (target_col..target_col + orig_col_span)
                    .map(|c| col_widths[c as usize])
                    .collect()
            } else {
                let each = orig_width / orig_col_span as u32;
                vec![each; orig_col_span as usize]
            }
        };

        // 행높이 계산: 다른 열의 row_span==1 셀에서 실제 높이 추출, 없으면 균등 분배
        let raw_row_heights = self.get_raw_row_heights();
        let split_row_heights: Vec<HwpUnit> = {
            let has_real = (target_row..target_row + orig_row_span).all(|r| {
                self.cells.iter().any(|cell| {
                    cell.row == r
                        && cell.row_span == 1
                        && !(cell.col == target_col && cell.row == target_row)
                })
            });
            if has_real {
                (target_row..target_row + orig_row_span)
                    .map(|r| raw_row_heights[r as usize])
                    .collect()
            } else {
                let each = orig_height / orig_row_span as u32;
                vec![each; orig_row_span as usize]
            }
        };

        // 주 셀 축소
        let new_width = split_col_widths[0];
        let primary = &mut self.cells[cell_idx];
        primary.col_span = 1;
        primary.row_span = 1;
        if primary.raw_list_extra.len() >= 4 {
            let old_ref = u32::from_le_bytes(primary.raw_list_extra[0..4].try_into().unwrap());
            if old_ref == primary.width {
                primary.raw_list_extra[0..4].copy_from_slice(&new_width.to_le_bytes());
            }
        }
        primary.width = new_width;
        primary.height = split_row_heights[0];

        // 새 셀 생성: 범위 내 (target_col, target_row) 제외한 모든 위치
        for ri in 0..orig_row_span {
            for ci in 0..orig_col_span {
                let r = target_row + ri;
                let c = target_col + ci;
                if r == target_row && c == target_col {
                    continue; // 주 셀 위치 스킵
                }
                let w = split_col_widths[ci as usize];
                let h = split_row_heights[ri as usize];
                let new_cell = Cell::new_from_template(c, r, w, h, &self.cells[cell_idx]);
                self.cells.push(new_cell);
            }
        }

        // 행 우선 순서 정렬
        self.cells.sort_by_key(|c| (c.row, c.col));

        // row_sizes 갱신 (행별 실제 셀 개수)
        self.rebuild_row_sizes();

        // 그리드 인덱스 재구축
        self.rebuild_grid();

        Ok(())
    }

    /// 셀을 N줄 × M칸으로 분할한다.
    ///
    /// 기존 `split_cell()`은 병합 해제만 지원하지만, 이 메서드는 임의 셀을
    /// 지정한 행/열 수로 분할한다. 테이블 그리드에 새 행/열이 추가되고,
    /// 인접 셀은 col_span/row_span이 확장되어 기존 형태를 유지한다.
    pub fn split_cell_into(
        &mut self,
        target_row: u16,
        target_col: u16,
        n_rows: u16,
        m_cols: u16,
        equal_row_height: bool,
        merge_first: bool,
    ) -> Result<(), String> {
        if n_rows < 1 || m_cols < 1 {
            return Err("분할 행/열 수는 1 이상이어야 합니다".to_string());
        }
        if n_rows == 1 && m_cols == 1 {
            return Ok(()); // no-op
        }

        // 대상 셀 찾기
        let cell_idx = self
            .cells
            .iter()
            .position(|c| c.col == target_col && c.row == target_row)
            .ok_or_else(|| format!("셀 ({},{})을 찾을 수 없습니다", target_row, target_col))?;

        let cs = self.cells[cell_idx].col_span;
        let rs = self.cells[cell_idx].row_span;

        // 병합 셀이면서 merge_first 옵션 → 먼저 병합 해제
        if merge_first && (cs > 1 || rs > 1) {
            self.split_cell(target_row, target_col)?;
            // split_cell 후 셀 인덱스 변경됨 → 재탐색
        }

        // 대상 셀 재탐색 (병합 해제 후 span=1x1)
        let cell_idx = self
            .cells
            .iter()
            .position(|c| c.col == target_col && c.row == target_row)
            .ok_or_else(|| {
                format!(
                    "분할 대상 셀 ({},{})을 찾을 수 없습니다",
                    target_row, target_col
                )
            })?;

        let target_width = self.cells[cell_idx].width;
        let target_height = self.cells[cell_idx].height;
        let cs = self.cells[cell_idx].col_span;
        let rs = self.cells[cell_idx].row_span;

        // 현재 span 기준으로 추가 열/행 계산
        // (다중 셀 분할 시 이전 분할로 span이 확장된 경우 extra=0)
        let extra_cols = if m_cols > cs { m_cols - cs } else { 0 };
        let extra_rows = if n_rows > rs { n_rows - rs } else { 0 };

        // 서브셀이 차지할 그리드 열/행 수
        let grid_cols = cs + extra_cols; // = max(m_cols, cs)
        let grid_rows = rs + extra_rows; // = max(n_rows, rs)

        // 폭 분배: 균등 분배 (나머지는 첫 셀에 가산)
        let base_w = target_width / m_cols as u32;
        let remainder_w = target_width - base_w * m_cols as u32;
        let sub_widths: Vec<HwpUnit> = (0..m_cols)
            .map(|i| base_w + if i == 0 { remainder_w } else { 0 })
            .collect();

        // 높이 분배
        let sub_heights: Vec<HwpUnit> = if equal_row_height || n_rows > 1 {
            let base_h = target_height / n_rows as u32;
            let remainder_h = target_height - base_h * n_rows as u32;
            (0..n_rows)
                .map(|i| base_h + if i == 0 { remainder_h } else { 0 })
                .collect()
        } else {
            vec![target_height]
        };

        // 서브셀의 col_span/row_span 분배 (grid_cols를 m_cols개에 분배)
        let base_cspan = grid_cols / m_cols;
        let cspan_rem = grid_cols - base_cspan * m_cols;
        let sub_cspans: Vec<u16> = (0..m_cols)
            .map(|i| base_cspan + if i < cspan_rem { 1 } else { 0 })
            .collect();
        let base_rspan = grid_rows / n_rows;
        let rspan_rem = grid_rows - base_rspan * n_rows;
        let sub_rspans: Vec<u16> = (0..n_rows)
            .map(|i| base_rspan + if i < rspan_rem { 1 } else { 0 })
            .collect();

        // 서브셀의 그리드 col 오프셋 계산 (col_span 누적)
        let mut sub_col_offsets: Vec<u16> = vec![0; m_cols as usize];
        for i in 1..m_cols as usize {
            sub_col_offsets[i] = sub_col_offsets[i - 1] + sub_cspans[i - 1];
        }
        let mut sub_row_offsets: Vec<u16> = vec![0; n_rows as usize];
        for i in 1..n_rows as usize {
            sub_row_offsets[i] = sub_row_offsets[i - 1] + sub_rspans[i - 1];
        }

        // 기존 셀 조정 (대상 셀 제외)
        for i in 0..self.cells.len() {
            if i == cell_idx {
                continue;
            }
            let cell = &mut self.cells[i];

            // --- 열 방향 조정 ---
            if extra_cols > 0 {
                if cell.col > target_col {
                    cell.col += extra_cols;
                } else if cell.col == target_col {
                    cell.col_span += extra_cols;
                } else if cell.col < target_col && cell.col + cell.col_span > target_col {
                    cell.col_span += extra_cols;
                }
            }

            // --- 행 방향 조정 ---
            if extra_rows > 0 {
                if cell.row > target_row {
                    cell.row += extra_rows;
                } else if cell.row == target_row {
                    cell.row_span += extra_rows;
                } else if cell.row < target_row && cell.row + cell.row_span > target_row {
                    cell.row_span += extra_rows;
                }
            }
        }

        // 주 셀(0,0) 축소
        let template = self.cells[cell_idx].clone();
        let primary = &mut self.cells[cell_idx];
        primary.width = sub_widths[0];
        primary.height = sub_heights[0];
        primary.col_span = sub_cspans[0];
        primary.row_span = sub_rspans[0];
        if primary.raw_list_extra.len() >= 4 {
            primary.raw_list_extra[0..4].copy_from_slice(&sub_widths[0].to_le_bytes());
        }

        // 나머지 서브셀 생성
        for ri in 0..n_rows {
            for ci in 0..m_cols {
                if ri == 0 && ci == 0 {
                    continue;
                } // 주 셀 스킵
                let r = target_row + sub_row_offsets[ri as usize];
                let c = target_col + sub_col_offsets[ci as usize];
                let w = sub_widths[ci as usize];
                let h = sub_heights[ri as usize];
                let mut new_cell = Cell::new_from_template(c, r, w, h, &template);
                new_cell.col_span = sub_cspans[ci as usize];
                new_cell.row_span = sub_rspans[ri as usize];
                if new_cell.raw_list_extra.len() >= 4 {
                    new_cell.raw_list_extra[0..4].copy_from_slice(&w.to_le_bytes());
                }
                self.cells.push(new_cell);
            }
        }

        // 테이블 메타 갱신
        self.col_count += extra_cols;
        self.row_count += extra_rows;

        self.cells.sort_by_key(|c| (c.row, c.col));
        self.rebuild_row_sizes();
        self.update_ctrl_dimensions();
        self.rebuild_grid();

        Ok(())
    }

    /// 범위 내 셀들을 각각 N줄 × M칸으로 분할한다.
    ///
    /// 우측→좌측, 하단→상단 순서로 처리하여 그리드 시프트가
    /// 아직 처리되지 않은 셀에 영향을 주지 않도록 한다.
    pub fn split_cells_in_range(
        &mut self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
        n_rows: u16,
        m_cols: u16,
        equal_row_height: bool,
    ) -> Result<(), String> {
        if n_rows < 1 || m_cols < 1 {
            return Err("분할 행/열 수는 1 이상이어야 합니다".to_string());
        }
        if n_rows == 1 && m_cols == 1 {
            return Ok(());
        }

        // 열 우선 순서: 우측→좌측 열, 각 열 내에서 하단→상단
        // 같은 열 내 분할은 col을 시프트하지 않고 col_span만 확장하므로 안전.
        // 우측 열 처리 후 좌측 열의 셀 col은 아직 원래 값을 유지한다.
        for c in (start_col..=end_col).rev() {
            // 행 분할: 하단→상단 (같은 행 내 분할은 row_span만 확장)
            for r in (start_row..=end_row).rev() {
                if !self.cells.iter().any(|cell| cell.col == c && cell.row == r) {
                    continue;
                }
                self.split_cell_into(r, c, n_rows, m_cols, equal_row_height, false)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
