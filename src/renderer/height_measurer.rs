//! 콘텐츠 높이 측정 모듈
//!
//! 페이지네이션 전에 각 콘텐츠의 실제 렌더링 높이를 측정한다.
//! LayoutEngine과 동일한 계산 로직을 사용하여 정확한 높이를 산출한다.

use crate::model::control::Control;
use crate::model::footnote::{Footnote, FootnoteShape};
use crate::model::paragraph::Paragraph;
use crate::model::shape::Caption;
use crate::model::table::{Table, TablePageBreak};
use super::composer::{ComposedParagraph, compose_paragraph};
use super::style_resolver::ResolvedStyleSet;
use super::{hwpunit_to_px, DEFAULT_DPI};

/// treat_as_char 표가 인라인(텍스트와 나란히)인지 판별
///
/// 인라인 조건:
/// 1. 텍스트가 있으면 → 표 너비가 줄 너비의 90% 미만
/// 2. 텍스트가 없어도 → 같은 문단에 TAC 표가 2개 이상이고 합산 너비가 줄 너비 이내
pub fn is_tac_table_inline(table: &Table, seg_width: i32, text: &str, controls: &[Control]) -> bool {
    let table_width: u32 = table.get_column_widths().iter().sum();

    if !text.is_empty() {
        return (table_width as i32) < (seg_width as f64 * 0.9) as i32;
    }

    // 텍스트 없는 문단: 다중 TAC 표의 합산 너비가 줄 너비 이내이면 인라인
    let tac_tables: Vec<&Table> = controls.iter()
        .filter_map(|c| match c {
            Control::Table(t) if t.common.treat_as_char => Some(t.as_ref()),
            _ => None,
        })
        .collect();

    if tac_tables.len() >= 2 {
        let total_width: u32 = tac_tables.iter()
            .map(|t| t.get_column_widths().iter().sum::<u32>())
            .sum();
        return (total_width as i32) <= seg_width;
    }

    false
}

/// 문단의 측정된 높이 정보
#[derive(Debug, Clone)]
pub struct MeasuredParagraph {
    /// 문단 인덱스
    pub para_index: usize,
    /// 총 높이 (spacing 포함, px)
    pub total_height: f64,
    /// 줄별 콘텐츠 높이 목록 (line_height만, line_spacing 미포함, px)
    pub line_heights: Vec<f64>,
    /// 줄별 줄간격 목록 (line_spacing, px)
    pub line_spacings: Vec<f64>,
    /// spacing_before (px)
    pub spacing_before: f64,
    /// spacing_after (px)
    pub spacing_after: f64,
    /// 표 컨트롤 포함 여부
    pub has_table: bool,
    /// 그림 컨트롤 포함 여부
    pub has_picture: bool,
    /// 그림 총 높이 (px)
    pub picture_height: f64,
}

impl MeasuredParagraph {
    /// 특정 줄의 전체 advance 높이 (콘텐츠 + 줄간격)를 반환한다.
    #[inline]
    pub fn line_advance(&self, line_idx: usize) -> f64 {
        self.line_heights[line_idx] + self.line_spacings[line_idx]
    }

    /// 줄 범위의 전체 advance 높이 합계를 반환한다.
    pub fn line_advances_sum(&self, range: std::ops::Range<usize>) -> f64 {
        range.into_iter()
            .map(|i| self.line_heights[i] + self.line_spacings[i])
            .sum()
    }
}

/// 표의 측정된 높이 정보
#[derive(Debug, Clone)]
pub struct MeasuredTable {
    /// 문단 인덱스
    pub para_index: usize,
    /// 컨트롤 인덱스
    pub control_index: usize,
    /// 총 높이 (px, 캡션 포함)
    pub total_height: f64,
    /// 행별 높이 목록 (px)
    pub row_heights: Vec<f64>,
    /// 캡션 높이 (px)
    pub caption_height: f64,
    /// 셀 간격 (px)
    pub cell_spacing: f64,
    /// 누적 행 높이 (cell_spacing 포함). len = row_heights.len() + 1
    /// cumulative_heights[0] = 0, cumulative_heights[i+1] = cumulative_heights[i] + row_heights[i] + cs_i
    /// cs_i = cell_spacing if i > 0, else 0
    pub cumulative_heights: Vec<f64>,
    /// 제목행 반복 여부
    pub repeat_header: bool,
    /// 행0에 제목 셀(is_header)이 있는지 여부
    pub has_header_cells: bool,
    /// 셀별 줄 단위 측정 데이터 (page_break == CellBreak일 때만 채움)
    pub cells: Vec<MeasuredCell>,
    /// 표 쪽 나눔 설정
    pub page_break: TablePageBreak,
    /// 각 행이 속한 rowspan 묶음 블록의 시작 행 (Task #398).
    /// 단일 행 블록(rowspan=1만 포함)이면 row_block_start[r] == r.
    /// 길이는 row_heights.len()와 동일. 빈 vec이면 모든 행이 단일 블록으로 간주.
    pub row_block_start: Vec<usize>,
    /// 각 행이 속한 rowspan 묶음 블록의 종료 행 (exclusive, Task #398).
    /// 단일 행 블록이면 row_block_end[r] == r + 1.
    pub row_block_end: Vec<usize>,
}

/// 셀의 줄 단위 측정 정보 (행 내부 분할용)
#[derive(Debug, Clone)]
pub struct MeasuredCell {
    /// 행 인덱스
    pub row: usize,
    /// 열 인덱스
    pub col: usize,
    /// 행 병합 수
    pub row_span: usize,
    /// 상단 패딩 (px)
    pub padding_top: f64,
    /// 하단 패딩 (px)
    pub padding_bottom: f64,
    /// 전체 줄별 높이 (모든 문단의 줄을 평탄화, px).
    /// 각 값 = line_height + line_spacing. 마지막 줄은 line_spacing 제외.
    pub line_heights: Vec<f64>,
    /// 총 콘텐츠 높이 (line_heights의 합, px)
    pub total_content_height: f64,
    /// 문단별 줄 수 (평탄화된 인덱스를 문단/줄로 역매핑용)
    pub para_line_counts: Vec<usize>,
    /// 셀 내 중첩 표 포함 여부
    pub has_nested_table: bool,
}

/// 구역 전체의 측정 결과
#[derive(Debug, Clone)]
pub struct MeasuredSection {
    /// 문단별 측정 정보
    pub paragraphs: Vec<MeasuredParagraph>,
    /// 표별 측정 정보 (문단 내 인라인 표)
    pub tables: Vec<MeasuredTable>,
}

/// 높이 측정 엔진
pub struct HeightMeasurer {
    dpi: f64,
}

impl HeightMeasurer {
    pub fn new(dpi: f64) -> Self {
        Self { dpi }
    }

    pub fn with_default_dpi() -> Self {
        Self::new(DEFAULT_DPI)
    }

    /// 구역의 모든 콘텐츠 높이를 측정한다.
    pub fn measure_section(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) -> MeasuredSection {
        let mut measured_paras = Vec::with_capacity(paragraphs.len());
        let mut measured_tables = Vec::new();

        for (para_idx, para) in paragraphs.iter().enumerate() {
            let comp = composed.get(para_idx);

            // 블록 표 컨트롤 감지 (일반 표 + treat_as_char 블록형)
            let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let has_table = para.controls.iter()
                .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char
                    || (t.common.treat_as_char && !is_tac_table_inline(t, seg_width, &para.text, &para.controls))));

            // 그림 컨트롤 감지 및 높이 측정
            let has_picture = para.controls.iter()
                .any(|c| matches!(c, Control::Picture(_) | Control::Equation(_)));
            let picture_height = self.measure_pictures_in_paragraph(para);

            // 문단 높이 측정
            let measured = self.measure_paragraph(para, comp, styles, para_idx, has_table, has_picture, picture_height);
            measured_paras.push(measured);

            // 표 높이 측정
            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(table) = ctrl {
                    let measured_table = self.measure_table(table, para_idx, ctrl_idx, styles);
                    measured_tables.push(measured_table);
                }
            }
        }

        MeasuredSection {
            paragraphs: measured_paras,
            tables: measured_tables,
        }
    }

    /// 단일 문단의 높이를 측정한다.
    fn measure_paragraph(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        para_index: usize,
        has_table: bool,
        has_picture: bool,
        picture_height: f64,
    ) -> MeasuredParagraph {
        // 문단 스타일에서 spacing 조회
        let para_style_id = composed.map(|c| c.para_style_id as usize).unwrap_or(0);
        let para_style = styles.para_styles.get(para_style_id);
        let spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);

        // 줄별 높이 계산: 콘텐츠 높이(line_height)와 줄간격(line_spacing)을 분리 저장
        // line_height = 줄의 콘텐츠 영역 높이
        // line_spacing = 현재 줄 하단에서 다음 줄 상단까지의 추가 공간
        // Y advance = line_height + line_spacing (HWP LineSeg 실증 결과)
        //
        // layout_paragraph와 동일한 보정: LineSeg line_height가 해당 줄의 최대
        // 폰트 크기보다 작으면 ParaShape 줄간격 설정으로 재계산한다.
        let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
        let ls_type = para_style.map(|s| s.line_spacing_type)
            .unwrap_or(crate::model::style::LineSpacingType::Percent);

        let (line_heights, line_spacings): (Vec<f64>, Vec<f64>) = if let Some(comp) = composed {
            comp.lines.iter()
                .map(|line| {
                    let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                    let max_fs = line.runs.iter()
                        .map(|r| {
                            styles.char_styles.get(r.char_style_id as usize)
                                .map(|cs| cs.font_size)
                                .unwrap_or(0.0)
                        })
                        .fold(0.0f64, f64::max);
                    let lh = crate::renderer::corrected_line_height(raw_lh, max_fs, ls_type, ls_val);
                    (lh, hwpunit_to_px(line.line_spacing, self.dpi))
                })
                .unzip()
        } else if !para.line_segs.is_empty() {
            // 누름틀(ClickHere) 안내문이 LINE_SEG에 포함되면 줄 수가 실제보다 많음
            // 안내문 텍스트가 차지하는 줄을 제외하여 실제 렌더링 높이를 계산
            let guide_char_count: usize = para.controls.iter()
                .filter_map(|c| {
                    if let Control::Field(f) = c {
                        f.guide_text().map(|t| t.encode_utf16().count())
                    } else { None }
                })
                .sum();
            if guide_char_count > 0 && para.line_segs.len() >= 2 {
                // 안내문이 차지하는 LINE_SEG 수:
                // 제어문자(필드 시작/끝 약 8 code units) + 안내문 길이까지의 text_start
                let guide_end = guide_char_count + 10; // 제어문자 + 안내문 + 여유
                let skip = para.line_segs.iter()
                    .position(|seg| (seg.text_start as usize) >= guide_end)
                    .unwrap_or(0);
                para.line_segs.iter().skip(skip)
                    .map(|seg| (
                        hwpunit_to_px(seg.line_height, self.dpi),
                        hwpunit_to_px(seg.line_spacing, self.dpi),
                    ))
                    .unzip()
            } else {
                para.line_segs.iter()
                    .map(|seg| (
                        hwpunit_to_px(seg.line_height, self.dpi),
                        hwpunit_to_px(seg.line_spacing, self.dpi),
                    ))
                    .unzip()
            }
        } else {
            // 빈 문단: 기본 높이
            (vec![hwpunit_to_px(400, self.dpi)], vec![0.0])
        };

        let lines_total: f64 = {
            let sum: f64 = line_heights.iter().zip(line_spacings.iter())
                .map(|(h, s)| h + s)
                .sum();
            // TAC 표 문단에서 첫 LINE_SEG의 lh가 표 높이로 확장되고
            // 마지막 SEG도 동일한 lh를 가질 때, 합산이 이중 계산됨.
            // (표 앞 텍스트가 있어 LINE_SEG가 2개인 경우 발생)
            // → vpos 기반 실제 높이와 비교하여 작은 값 사용
            if has_table && para.line_segs.len() >= 2 {
                let first = &para.line_segs[0];
                let last = &para.line_segs[para.line_segs.len() - 1];
                if first.text_height * 2 < first.line_height
                    && first.line_height == last.line_height
                {
                    let vpos_h = hwpunit_to_px(
                        last.vertical_pos + last.line_height + last.line_spacing
                            - first.vertical_pos, self.dpi);
                    vpos_h.min(sum)
                } else {
                    sum
                }
            } else {
                sum
            }
        };

        // 누름틀(ClickHere) 안내문 높이 제외
        // 안내문은 렌더링되지 않으므로 페이지네이션에서 높이를 차지하면 안 됨
        let clickhere_adjustment: f64 = para.controls.iter()
            .filter_map(|c| {
                if let Control::Field(f) = c {
                    if let Some(guide) = f.guide_text() {
                        let guide_u16_len = guide.encode_utf16().count();
                        if guide_u16_len > 0 && para.line_segs.len() >= 2 {
                            // 안내문이 차지하는 LINE_SEG 수 계산
                            let guide_end = guide_u16_len + 10; // 제어문자 여유
                            let guide_segs = para.line_segs.iter()
                                .position(|seg| (seg.text_start as usize) >= guide_end)
                                .unwrap_or(0);
                            if guide_segs > 0 {
                                let adj: f64 = para.line_segs[..guide_segs].iter()
                                    .map(|seg| hwpunit_to_px(seg.line_height + seg.line_spacing, self.dpi))
                                    .sum();
                                return Some(adj);
                            }
                        }
                    }
                }
                None
            })
            .sum();

        // 그림 높이는 문단 높이에 포함하지 않음 (별도 PageItem::Shape로 처리)
        let total_height = (spacing_before + lines_total + spacing_after - clickhere_adjustment).max(0.0);

        MeasuredParagraph {
            para_index,
            total_height,
            line_heights,
            line_spacings,
            spacing_before,
            spacing_after,
            has_table,
            has_picture,
            picture_height,
        }
    }

    /// 문단들 내 비-인라인(treat_as_char가 아닌) 그림/도형의 높이 합계를 측정한다.
    /// LINE_SEG에는 비-인라인 컨트롤 높이가 포함되지 않으므로 별도 합산이 필요하다.
    fn measure_non_inline_controls_height(&self, paragraphs: &[Paragraph]) -> f64 {
        use crate::model::shape::TextWrap;
        let mut total = 0.0;
        for para in paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Picture(pic) if !pic.common.treat_as_char
                        && matches!(pic.common.text_wrap, TextWrap::TopAndBottom) => {
                        total += hwpunit_to_px(pic.common.height as i32, self.dpi);
                    }
                    Control::Shape(shape) if !shape.common().treat_as_char
                        && matches!(shape.common().text_wrap, TextWrap::TopAndBottom) => {
                        total += hwpunit_to_px(shape.common().height as i32, self.dpi);
                    }
                    _ => {}
                }
            }
        }
        total
    }

    /// 문단 내 모든 그림/수식의 높이 합계를 측정한다.
    fn measure_pictures_in_paragraph(&self, para: &Paragraph) -> f64 {
        let mut total = 0.0;
        for ctrl in &para.controls {
            match ctrl {
                Control::Picture(pic) => {
                    total += hwpunit_to_px(pic.common.height as i32, self.dpi);
                }
                Control::Equation(eq) => {
                    total += hwpunit_to_px(eq.common.height as i32, self.dpi);
                }
                _ => {}
            }
        }
        total
    }

    /// 표의 높이를 측정한다.
    /// layout_table과 동일한 방식으로 셀 내용 높이를 고려한다.
    fn measure_table(
        &self,
        table: &Table,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
    ) -> MeasuredTable {
        self.measure_table_impl(table, para_index, control_index, styles, 0)
    }

    /// 재귀적 높이 제한
    const MAX_NESTED_DEPTH: usize = 10;

    /// 셀 내 중첩 표들의 총 높이를 계산한다.
    pub fn cell_controls_height(&self, paragraphs: &[Paragraph], styles: &ResolvedStyleSet, depth: usize) -> f64 {
        if depth >= Self::MAX_NESTED_DEPTH {
            return 0.0;
        }
        paragraphs.iter().map(|p| {
            p.controls.iter().filter_map(|ctrl| {
                if let Control::Table(nested) = ctrl {
                    let mt = self.measure_table_impl(nested, 0, 0, styles, depth + 1);
                    Some(mt.total_height)
                } else {
                    None
                }
            }).sum::<f64>()
        }).sum()
    }

    /// 표의 높이를 측정한다 (depth 기반 재귀).
    fn measure_table_impl(
        &self,
        table: &Table,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
        depth: usize,
    ) -> MeasuredTable {
        if depth >= Self::MAX_NESTED_DEPTH {
            let rc = table.row_count as usize;
            let (rbs, rbe) = compute_row_blocks(table, rc);
            return MeasuredTable {
                para_index,
                control_index,
                total_height: 0.0,
                row_heights: vec![0.0; rc],
                caption_height: 0.0,
                cell_spacing: 0.0,
                cumulative_heights: vec![0.0; rc + 1],
                repeat_header: false,
                has_header_cells: false,
                cells: Vec::new(),
                page_break: crate::model::table::TablePageBreak::None,
                row_block_start: rbs,
                row_block_end: rbe,
            };
        }
        // 1x1 래퍼 표 감지: 내부 표의 높이를 직접 측정
        if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
            let cell = &table.cells[0];
            let has_visible_text = cell.paragraphs.iter()
                .any(|p| p.text.chars().any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n'));
            if !has_visible_text {
                if let Some(nested) = cell.paragraphs.iter()
                    .flat_map(|p| p.controls.iter())
                    .find_map(|c| if let Control::Table(t) = c { Some(t.as_ref()) } else { None })
                {
                    return self.measure_table_impl(nested, para_index, control_index, styles, depth + 1);
                }
            }
        }

        let row_count = table.row_count as usize;
        let mut row_heights = vec![0.0f64; row_count];

        // 1단계: row_span==1인 셀에서 행별 최대 높이 추출
        // cell.height는 HWP가 저장한 셀 높이 (pad + content, trailing ls 미포함)
        for cell in &table.cells {
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                if cell.height < 0x80000000 {
                    let h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if h > row_heights[r] {
                        row_heights[r] = h;
                    }
                }
            }
        }

        // 2단계: 셀 내 실제 컨텐츠 높이 계산 (layout_table과 동일)
        for cell in &table.cells {
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                // 셀 패딩 — layout 의 resolve_cell_padding 과 일관성:
                //   aim=true  → cell.padding (0 도 명시값으로 존중)
                //   aim=false → 기본은 table.padding, 단 cell > table 인 비대칭 축만 cell 우선
                let prefer_cell_axis = |c: i16, t: i16| -> bool {
                    if cell.apply_inner_margin { c != 0 } else { (c as i32) > (t as i32) }
                };
                let pad_top = if prefer_cell_axis(cell.padding.top, table.padding.top) {
                    hwpunit_to_px(cell.padding.top as i32, self.dpi)
                } else {
                    hwpunit_to_px(table.padding.top as i32, self.dpi)
                };
                let pad_bottom = if prefer_cell_axis(cell.padding.bottom, table.padding.bottom) {
                    hwpunit_to_px(cell.padding.bottom as i32, self.dpi)
                } else {
                    hwpunit_to_px(table.padding.bottom as i32, self.dpi)
                };
                // [Task #671] 좌우 패딩 — recompose_for_cell_width 의 inner_width 계산용
                let pad_left = if prefer_cell_axis(cell.padding.left, table.padding.left) {
                    hwpunit_to_px(cell.padding.left as i32, self.dpi)
                } else {
                    hwpunit_to_px(table.padding.left as i32, self.dpi)
                };
                let pad_right = if prefer_cell_axis(cell.padding.right, table.padding.right) {
                    hwpunit_to_px(cell.padding.right as i32, self.dpi)
                } else {
                    hwpunit_to_px(table.padding.right as i32, self.dpi)
                };
                let cell_w_px = if cell.width < 0x80000000 {
                    hwpunit_to_px(cell.width as i32, self.dpi)
                } else {
                    0.0
                };
                let cell_inner_width = (cell_w_px - pad_left - pad_right).max(0.0);

                // 셀 내 문단들의 실제 높이 합산
                let text_height: f64 = if cell.text_direction != 0 {
                    // 세로쓰기: line_seg.segment_width가 열의 세로 길이
                    // 셀 높이 = 최대 segment_width
                    let mut max_h: f64 = 0.0;
                    for p in &cell.paragraphs {
                        for ls in &p.line_segs {
                            let h = hwpunit_to_px(ls.segment_width, self.dpi);
                            if h > max_h { max_h = h; }
                        }
                    }
                    if max_h <= 0.0 { hwpunit_to_px(400, self.dpi) } else { max_h }
                } else {
                    // 가로쓰기: spacing + line_height + line_spacing 합산
                    let cell_para_count = cell.paragraphs.len();
                    cell.paragraphs.iter()
                        .enumerate()
                        .map(|(pidx, p)| {
                            let mut comp = compose_paragraph(p);
                            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine
                            // 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할.
                            // 측정/렌더링 일관성 (layout 의 recompose_for_cell_width 호출과 동일).
                            crate::renderer::composer::recompose_for_cell_width(
                                &mut comp, p, cell_inner_width, styles,
                            );
                            let para_style = styles.para_styles.get(p.para_shape_id as usize);
                            let is_last_para = pidx + 1 == cell_para_count;
                            let spacing_before = if pidx > 0 {
                                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            let spacing_after = if !is_last_para {
                                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            if comp.lines.is_empty() {
                                spacing_before + hwpunit_to_px(400, self.dpi) + spacing_after
                            } else {
                                let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                                let cell_ls_type = para_style.map(|s| s.line_spacing_type)
                                    .unwrap_or(crate::model::style::LineSpacingType::Percent);
                                let line_count = comp.lines.len();
                                let lines_total: f64 = comp.lines.iter()
                                    .enumerate()
                                    .map(|(i, line)| {
                                        let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                                        let max_fs = line.runs.iter()
                                            .map(|r| styles.char_styles.get(r.char_style_id as usize)
                                                .map(|cs| cs.font_size).unwrap_or(0.0))
                                            .fold(0.0f64, f64::max);
                                        let h = crate::renderer::corrected_line_height(
                                            raw_lh, max_fs, cell_ls_type, cell_ls_val);
                                        let is_cell_last_line = is_last_para && i + 1 == line_count;
                                        if !is_cell_last_line {
                                            h + hwpunit_to_px(line.line_spacing, self.dpi)
                                        } else {
                                            h
                                        }
                                    })
                                    .sum();
                                spacing_before + lines_total + spacing_after
                            }
                        })
                        .sum()
                };
                // 중첩 표가 있는 셀: LINE_SEG.line_height에 중첩 표 높이가 미포함.
                // vpos 점프에만 반영되므로, 마지막 seg의 (vpos + lh)로 전체 높이를 계산.
                let has_nested_table_in_cell = cell.paragraphs.iter()
                    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
                let content_height = if has_nested_table_in_cell {
                    // 마지막 문단의 마지막 LINE_SEG의 vpos + line_height
                    let last_seg_end: i32 = cell.paragraphs.iter()
                        .flat_map(|p| p.line_segs.last())
                        .map(|s| s.vertical_pos + s.line_height)
                        .max()
                        .unwrap_or(0);
                    hwpunit_to_px(last_seg_end, self.dpi).max(text_height)
                } else {
                    // 단, 비-인라인 이미지/도형은 LINE_SEG에 미포함이므로 별도 합산
                    let non_inline_h = self.measure_non_inline_controls_height(&cell.paragraphs);
                    text_height + non_inline_h
                };

                // 패딩 포함 총 필요 높이
                // [Task #501] cell.padding 이 IR cell.height 의 절반을 초과하는 비정상
                // 케이스 (mel-001 p2 셀[21]: cell.h=1280 HU, pad.top+bottom=3400 HU) 가드:
                // 비정상 padding 이 row_heights 를 확장하면 TAC 표 비례 축소가 모든 행에
                // 영향. content_height 가 IR cell.height 안에 들어가면 IR 권위 우선.
                let total_pad = pad_top + pad_bottom;
                let cell_h_px = if cell.height < 0x80000000 {
                    hwpunit_to_px(cell.height as i32, self.dpi)
                } else {
                    0.0
                };
                let required_height = if cell_h_px > 0.0
                    && total_pad > cell_h_px * 0.5
                    && content_height <= cell_h_px
                {
                    cell_h_px
                } else {
                    content_height + total_pad
                };
                if required_height > row_heights[r] {
                    row_heights[r] = required_height;
                }
            }
        }

        // 2-b단계: 병합 셀에서 미지 행 높이를 반복적으로 해결
        {
            let mut constraints: Vec<(usize, usize, f64)> = Vec::new();
            for cell in &table.cells {
                let r = cell.row as usize;
                let span = cell.row_span as usize;
                if span > 1 && r + span <= row_count && cell.height < 0x80000000 {
                    let total_h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == r && x.1 == span) {
                        if total_h > existing.2 { existing.2 = total_h; }
                    } else {
                        constraints.push((r, span, total_h));
                    }
                }
            }
            constraints.sort_by_key(|&(_, span, _)| span);
            let max_iter = row_count + constraints.len();
            for _ in 0..max_iter {
                let mut progress = false;
                for &(r, span, total_h) in &constraints {
                    let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                    let unknown_rows: Vec<usize> = (r..r + span)
                        .filter(|&i| row_heights[i] == 0.0)
                        .collect();
                    if unknown_rows.len() == 1 {
                        let remaining = (total_h - known_sum).max(0.0);
                        row_heights[unknown_rows[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress { break; }
            }
            for &(r, span, total_h) in &constraints {
                let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                let unknown_rows: Vec<usize> = (r..r + span)
                    .filter(|&i| row_heights[i] == 0.0)
                    .collect();
                if !unknown_rows.is_empty() {
                    let remaining = (total_h - known_sum).max(0.0);
                    let per_row = remaining / unknown_rows.len() as f64;
                    for i in unknown_rows {
                        row_heights[i] = per_row;
                    }
                }
            }
        }

        // 2-c단계: 병합 셀의 실제 컨텐츠 높이가 결합 행 높이 초과 시 마지막 행 확장
        for cell in &table.cells {
            let r = cell.row as usize;
            let span = cell.row_span as usize;
            if span > 1 && r + span <= row_count {
                let (pad_top, pad_bottom) = if !cell.apply_inner_margin {
                    (hwpunit_to_px(table.padding.top as i32, self.dpi),
                     hwpunit_to_px(table.padding.bottom as i32, self.dpi))
                } else {
                    (if cell.padding.top != 0 { hwpunit_to_px(cell.padding.top as i32, self.dpi) }
                     else { hwpunit_to_px(table.padding.top as i32, self.dpi) },
                     if cell.padding.bottom != 0 { hwpunit_to_px(cell.padding.bottom as i32, self.dpi) }
                     else { hwpunit_to_px(table.padding.bottom as i32, self.dpi) })
                };
                // [Task #671] 좌우 패딩 (recompose_for_cell_width inner_width 계산용)
                let (pad_left, pad_right) = if !cell.apply_inner_margin {
                    (hwpunit_to_px(table.padding.left as i32, self.dpi),
                     hwpunit_to_px(table.padding.right as i32, self.dpi))
                } else {
                    (if cell.padding.left != 0 { hwpunit_to_px(cell.padding.left as i32, self.dpi) }
                     else { hwpunit_to_px(table.padding.left as i32, self.dpi) },
                     if cell.padding.right != 0 { hwpunit_to_px(cell.padding.right as i32, self.dpi) }
                     else { hwpunit_to_px(table.padding.right as i32, self.dpi) })
                };
                let cell_w_px = if cell.width < 0x80000000 {
                    hwpunit_to_px(cell.width as i32, self.dpi)
                } else {
                    0.0
                };
                let cell_inner_width = (cell_w_px - pad_left - pad_right).max(0.0);
                let text_height: f64 = if cell.text_direction != 0 {
                    // 세로쓰기: max(segment_width)
                    let mut max_h: f64 = 0.0;
                    for p in &cell.paragraphs {
                        for ls in &p.line_segs {
                            let h = hwpunit_to_px(ls.segment_width, self.dpi);
                            if h > max_h { max_h = h; }
                        }
                    }
                    if max_h <= 0.0 { hwpunit_to_px(400, self.dpi) } else { max_h }
                } else {
                    let cell_para_count = cell.paragraphs.len();
                    cell.paragraphs.iter()
                        .enumerate()
                        .map(|(pidx, p)| {
                            let mut comp = compose_paragraph(p);
                            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine
                            // 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할.
                            crate::renderer::composer::recompose_for_cell_width(
                                &mut comp, p, cell_inner_width, styles,
                            );
                            let para_style = styles.para_styles.get(p.para_shape_id as usize);
                            let is_last_para = pidx + 1 == cell_para_count;
                            let spacing_before = if pidx > 0 {
                                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            let spacing_after = if !is_last_para {
                                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            if comp.lines.is_empty() {
                                spacing_before + hwpunit_to_px(400, self.dpi) + spacing_after
                            } else {
                                let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                                let cell_ls_type = para_style.map(|s| s.line_spacing_type)
                                    .unwrap_or(crate::model::style::LineSpacingType::Percent);
                                let line_count = comp.lines.len();
                                let lines_total: f64 = comp.lines.iter()
                                    .enumerate()
                                    .map(|(i, line)| {
                                        let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                                        let max_fs = line.runs.iter()
                                            .map(|r| styles.char_styles.get(r.char_style_id as usize)
                                                .map(|cs| cs.font_size).unwrap_or(0.0))
                                            .fold(0.0f64, f64::max);
                                        let h = crate::renderer::corrected_line_height(
                                            raw_lh, max_fs, cell_ls_type, cell_ls_val);
                                        let is_cell_last_line = is_last_para && i + 1 == line_count;
                                        if !is_cell_last_line {
                                            h + hwpunit_to_px(line.line_spacing, self.dpi)
                                        } else {
                                            h
                                        }
                                    })
                                    .sum();
                                spacing_before + lines_total + spacing_after
                            }
                        })
                        .sum()
                };
                // LINE_SEG의 line_height에 이미 셀 내 중첩 표 높이가 반영되어 있으므로
                // controls_height를 별도로 더하면 이중 계산됨
                // 단, 비-인라인 이미지/도형은 LINE_SEG에 미포함이므로 별도 합산
                let non_inline_h = self.measure_non_inline_controls_height(&cell.paragraphs);
                let content_height = text_height + non_inline_h;
                let required_height = content_height + pad_top + pad_bottom;
                let combined: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                if required_height > combined {
                    let deficit = required_height - combined;
                    row_heights[r + span - 1] += deficit;
                }
            }
        }

        // 3단계: 높이가 0인 행은 기본값 적용
        for h in &mut row_heights {
            if *h <= 0.0 {
                *h = hwpunit_to_px(400, self.dpi);
            }
        }

        // 셀 간격 포함한 표 높이
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let raw_table_height: f64 = row_heights.iter().sum::<f64>()
            + cell_spacing * (row_count.saturating_sub(1) as f64);
        // TAC 표: common.height(표 속성 높이)를 상한으로 사용
        // 한컴은 TAC 표의 높이를 속성값으로 유지 (셀 콘텐츠 넘침은 클리핑)
        // 비-TAC 표: 셀 콘텐츠 기반 확장 유지 (행 분할 필요)
        let common_h = hwpunit_to_px(table.common.height as i32, self.dpi);
        // [Task #672] TAC 표 비례 축소 임계값 강화 — 작은 차이 (≤2%) 는 면제.
        //
        // 본질: 셀 콘텐츠 측정값과 common.height 의 미세한 불일치 (측정 오차
        // 또는 line_height 보정 부산물) 시 비례 축소가 셀 콘텐츠 클립을 발생.
        // 한컴 뷰어는 작은 차이를 비례 축소 안 함 (계획서.hwp 1.32% 차이 — 3 줄
        // 정상 표시). 2% 이상 차이는 사용자 의도 영역 (의도적 압축) 으로 간주
        // 하여 기존 동작 유지.
        //
        // 발동 영역 sweep 진단 (187 fixture): ≤2% 7 건 면제, ≥5% 11 건 그대로.
        const TAC_SHRINK_THRESHOLD_RATIO: f64 = 0.02;
        let shrink_threshold = (common_h * TAC_SHRINK_THRESHOLD_RATIO).max(1.0);
        let table_height = if table.common.treat_as_char && common_h > 0.0 && raw_table_height > common_h + shrink_threshold {
            let scale = common_h / raw_table_height;
            for h in &mut row_heights {
                *h *= scale;
            }
            common_h
        } else {
            raw_table_height
        };

        // 누적 행 높이 계산 (이진 탐색용)
        let mut cumulative_heights = vec![0.0f64; row_count + 1];
        for (i, &h) in row_heights.iter().enumerate() {
            let cs_i = if i > 0 { cell_spacing } else { 0.0 };
            cumulative_heights[i + 1] = cumulative_heights[i] + h + cs_i;
        }

        // 캡션 높이 계산 (Left/Right 캡션은 표 높이에 영향 없음)
        let is_lr_caption = table.caption.as_ref().map_or(false, |c| {
            use crate::model::shape::CaptionDirection;
            matches!(c.direction, CaptionDirection::Left | CaptionDirection::Right)
        });
        let caption_height = if is_lr_caption {
            0.0
        } else {
            self.measure_caption(&table.caption)
        };
        let caption_spacing = if is_lr_caption {
            0.0
        } else {
            table.caption.as_ref()
                .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
                .unwrap_or(0.0)
        };

        // 총 높이 = 표 높이 + 캡션 높이 + 캡션-표 간격
        let total_height = table_height + caption_height
            + if caption_height > 0.0 { caption_spacing } else { 0.0 };

        // 셀 단위 분할용 상세 측정 (모든 셀, row_span > 1 포함)
        let mut measured_cells = {
            table.cells.iter()
                .filter(|cell| (cell.row as usize) < row_count)
                .map(|cell| {
                    let pad_top = if cell.padding.top != 0 {
                        hwpunit_to_px(cell.padding.top as i32, self.dpi)
                    } else {
                        hwpunit_to_px(table.padding.top as i32, self.dpi)
                    };
                    let pad_bottom = if cell.padding.bottom != 0 {
                        hwpunit_to_px(cell.padding.bottom as i32, self.dpi)
                    } else {
                        hwpunit_to_px(table.padding.bottom as i32, self.dpi)
                    };

                    let mut line_heights = Vec::new();
                    let mut para_line_counts = Vec::new();
                    let para_count = cell.paragraphs.len();

                    for (pi, p) in cell.paragraphs.iter().enumerate() {
                        let comp = compose_paragraph(p);
                        let para_style = styles.para_styles.get(p.para_shape_id as usize);
                        let is_last_para = pi + 1 == para_count;
                        // compute_cell_line_ranges와 동일 규칙:
                        // 첫 문단은 spacing_before 없음, 마지막 문단은 spacing_after 없음
                        let spacing_before = if pi > 0 {
                            para_style.map(|s| s.spacing_before).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        let spacing_after = if !is_last_para {
                            para_style.map(|s| s.spacing_after).unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        // LINE_SEG의 line_height에 이미 중첩 표 높이가 반영되어 있으므로
                        // 별도 추가 줄로 넣으면 이중 계산됨
                        if comp.lines.is_empty() {
                            line_heights.push(spacing_before + hwpunit_to_px(400, self.dpi) + spacing_after);
                            para_line_counts.push(1);
                        } else {
                            let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                            let cell_ls_type = para_style.map(|s| s.line_spacing_type)
                                .unwrap_or(crate::model::style::LineSpacingType::Percent);
                            let line_count = comp.lines.len();
                            for (li, line) in comp.lines.iter().enumerate() {
                                let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                                let max_fs = line.runs.iter()
                                    .map(|r| styles.char_styles.get(r.char_style_id as usize)
                                        .map(|cs| cs.font_size).unwrap_or(0.0))
                                    .fold(0.0f64, f64::max);
                                let h = crate::renderer::corrected_line_height(
                                    raw_lh, max_fs, cell_ls_type, cell_ls_val);
                                let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                                // 셀의 마지막 줄(마지막 문단의 마지막 줄)은 ls 제외
                                let is_cell_last_line = is_last_para && li + 1 == line_count;
                                let mut line_h = if !is_cell_last_line { h + ls } else { h };
                                if li == 0 {
                                    line_h += spacing_before;
                                }
                                if li == line_count - 1 {
                                    line_h += spacing_after;
                                }
                                line_heights.push(line_h);
                            }
                            para_line_counts.push(line_count);
                        }
                    }

                    let line_sum: f64 = line_heights.iter().sum();

                    // 셀에 중첩 표가 있으면 LINE_SEG가 실제 높이를 반영하지 못함
                    let has_nested_table = cell.paragraphs.iter()
                        .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));

                    MeasuredCell {
                        row: cell.row as usize,
                        col: cell.col as usize,
                        row_span: cell.row_span as usize,
                        padding_top: pad_top,
                        padding_bottom: pad_bottom,
                        line_heights,
                        total_content_height: line_sum,
                        para_line_counts,
                        has_nested_table,
                    }
                })
                .collect::<Vec<_>>()
        };

        // 중첩 표 셀: 실제 중첩 표 높이를 재귀 측정하여 total_content_height 보정
        for mc in &mut measured_cells {
            if mc.has_nested_table {
                let cell = &table.cells.iter()
                    .find(|c| c.row as usize == mc.row && c.col as usize == mc.col)
                    .unwrap();
                let nested_h: f64 = cell.paragraphs.iter()
                    .flat_map(|p| p.controls.iter())
                    .filter_map(|c| if let Control::Table(t) = c { Some(t.as_ref()) } else { None })
                    .map(|t| self.measure_table_impl(t, 0, 0, styles, depth + 1).total_height)
                    .sum();
                mc.total_content_height = nested_h.max(mc.total_content_height);
            }
        }

        let (row_block_start, row_block_end) = compute_row_blocks(table, row_heights.len());
        MeasuredTable {
            para_index,
            control_index,
            total_height,
            row_heights,
            caption_height,
            cell_spacing,
            cumulative_heights,
            repeat_header: table.repeat_header,
            has_header_cells: table.cells.iter()
                .filter(|c| c.row == 0)
                .any(|c| c.is_header),
            cells: measured_cells,
            page_break: table.page_break,
            row_block_start,
            row_block_end,
        }
    }

    /// 구역의 모든 콘텐츠 높이를 증분 측정한다.
    /// dirty=false인 표는 prev_measured에서 재사용하고, dirty=true인 표만 재측정한다.
    pub fn measure_section_incremental(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        prev_measured: &MeasuredSection,
    ) -> MeasuredSection {
        let mut measured_paras = Vec::with_capacity(paragraphs.len());
        let mut measured_tables = Vec::new();

        for (para_idx, para) in paragraphs.iter().enumerate() {
            let comp = composed.get(para_idx);

            // 블록 표 컨트롤 감지 (일반 표 + treat_as_char 블록형)
            let seg_width_r = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let has_table = para.controls.iter()
                .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char
                    || (t.common.treat_as_char && !is_tac_table_inline(t, seg_width_r, &para.text, &para.controls))));
            let has_picture = para.controls.iter()
                .any(|c| matches!(c, Control::Picture(_) | Control::Equation(_)));
            let picture_height = self.measure_pictures_in_paragraph(para);

            let measured = self.measure_paragraph(para, comp, styles, para_idx, has_table, has_picture, picture_height);
            measured_paras.push(measured);

            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(table) = ctrl {
                    if !table.dirty {
                        if let Some(prev) = prev_measured.get_measured_table(para_idx, ctrl_idx) {
                            measured_tables.push(prev.clone());
                            continue;
                        }
                    }
                    let measured_table = self.measure_table(table, para_idx, ctrl_idx, styles);
                    measured_tables.push(measured_table);
                }
            }
        }

        MeasuredSection {
            paragraphs: measured_paras,
            tables: measured_tables,
        }
    }

    /// 구역의 콘텐츠 높이를 문단 수준 증분 측정한다.
    /// dirty_paras가 Some(bits)이면 dirty 문단만 재측정하고,
    /// None이면 전체 재측정한다 (measure_section_incremental 폴백).
    pub fn measure_section_selective(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        prev_measured: &MeasuredSection,
        dirty_paras: Option<&[bool]>,
    ) -> MeasuredSection {
        let dirty_bits = match dirty_paras {
            Some(bits) => bits,
            None => {
                // 전체 dirty: 기존 incremental (표 수준만 캐싱) 폴백
                return self.measure_section_incremental(paragraphs, composed, styles, prev_measured);
            }
        };

        let mut measured_paras = Vec::with_capacity(paragraphs.len());
        let mut measured_tables = Vec::new();

        for (para_idx, para) in paragraphs.iter().enumerate() {
            let is_dirty = dirty_bits.get(para_idx).copied().unwrap_or(true);

            if !is_dirty {
                // 문단 측정 캐시 재사용
                if let Some(prev_para) = prev_measured.paragraphs.get(para_idx) {
                    measured_paras.push(prev_para.clone());
                    // 표 dirty 체크는 항상 수행 (셀 편집 시 문단 non-dirty지만 표 dirty)
                    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Table(table) = ctrl {
                            if !table.dirty {
                                if let Some(prev_t) = prev_measured.get_measured_table(para_idx, ctrl_idx) {
                                    measured_tables.push(prev_t.clone());
                                    continue;
                                }
                            }
                            let mt = self.measure_table(table, para_idx, ctrl_idx, styles);
                            measured_tables.push(mt);
                        }
                    }
                    continue;
                }
            }

            // dirty 문단: 재측정
            let comp = composed.get(para_idx);
            // 블록 표 컨트롤 감지 (일반 표 + treat_as_char 블록형)
            let seg_width_r = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let has_table = para.controls.iter()
                .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char
                    || (t.common.treat_as_char && !is_tac_table_inline(t, seg_width_r, &para.text, &para.controls))));
            let has_picture = para.controls.iter()
                .any(|c| matches!(c, Control::Picture(_) | Control::Equation(_)));
            let picture_height = self.measure_pictures_in_paragraph(para);

            let measured = self.measure_paragraph(para, comp, styles, para_idx, has_table, has_picture, picture_height);
            measured_paras.push(measured);

            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(table) = ctrl {
                    if !table.dirty {
                        if let Some(prev) = prev_measured.get_measured_table(para_idx, ctrl_idx) {
                            measured_tables.push(prev.clone());
                            continue;
                        }
                    }
                    let mt = self.measure_table(table, para_idx, ctrl_idx, styles);
                    measured_tables.push(mt);
                }
            }
        }

        MeasuredSection {
            paragraphs: measured_paras,
            tables: measured_tables,
        }
    }

    /// 캡션의 높이를 측정한다.
    fn measure_caption(&self, caption: &Option<Caption>) -> f64 {
        let caption = match caption {
            Some(c) => c,
            None => return 0.0,
        };

        if caption.paragraphs.is_empty() {
            return 0.0;
        }

        let mut total_height = 0.0;
        for para in &caption.paragraphs {
            if para.line_segs.is_empty() {
                total_height += hwpunit_to_px(400, self.dpi); // 기본 줄 높이
            } else {
                for (i, seg) in para.line_segs.iter().enumerate() {
                    let line_h = hwpunit_to_px(seg.line_height, self.dpi);
                    // 마지막 줄은 line_spacing 제외
                    let spacing = if i < para.line_segs.len() - 1 {
                        hwpunit_to_px(seg.line_spacing, self.dpi)
                    } else {
                        0.0
                    };
                    total_height += line_h + spacing;
                }
            }
        }

        total_height
    }
}

impl MeasuredTable {
    /// 지정 행의 셀별 남은 콘텐츠 높이 최대값을 반환한다.
    /// 셀의 콘텐츠 높이가 행 높이(패딩 제외)를 초과하면 행 높이로 캡핑한다.
    /// (HWP가 지정한 행 높이 = 보이는 콘텐츠 높이; 중첩 표의 클리핑된 높이만 반영)
    pub fn remaining_content_for_row(&self, row: usize, content_offset: f64) -> f64 {
        let row_h = self.row_heights.get(row).copied().unwrap_or(0.0);
        // row_span > 1 셀도 포함: 해당 행이 셀의 범위 내이면 콘텐츠 잔량 계산에 포함
        self.cells.iter()
            .filter(|c| row >= c.row && row < c.row + c.row_span)
            .map(|c| {
                let padding = c.padding_top + c.padding_bottom;
                // row_span > 1 셀: 셀이 차지하는 모든 행의 높이 합을 사용
                let cell_row_h = if c.row_span > 1 {
                    let end = (c.row + c.row_span).min(self.row_heights.len());
                    let h: f64 = self.row_heights[c.row..end].iter().sum();
                    let cs_count = if end > c.row + 1 { (end - c.row - 1) as f64 } else { 0.0 };
                    h + cs_count * self.cell_spacing
                } else {
                    row_h
                };
                let max_content = (cell_row_h - padding).max(0.0);
                let line_sum: f64 = c.line_heights.iter().sum();
                // 중첩 표 셀: total_content_height가 실제 중첩 표 전체 높이 → capping 안 함
                // 일반 셀: LINE_SEG 기반이므로 max_content로 capping
                let capped = if c.has_nested_table {
                    c.total_content_height
                } else {
                    c.total_content_height.min(max_content.max(line_sum))
                };
                if content_offset <= 0.0 {
                    return capped;
                }
                // line_heights 합이 capped보다 현저히 작은 경우 (중첩 표 등으로
                // LINE_SEG가 실제 콘텐츠 높이를 반영하지 못하는 경우):
                // 연속적 비율 기반으로 remaining 계산
                let line_sum: f64 = c.line_heights.iter().sum();
                if line_sum < capped * 0.5 {
                    // [Task #362] nested table 의 잔여 계산 시 외부 셀 행 높이로 cap.
                    // total_content_height 가 nested 의 raw 누적이라 외부 행보다 클 수 있음 →
                    // 외부 행 기준 잔여로 cap 하여 후속 페이지 누적 결함 차단.
                    let effective_total = if c.has_nested_table {
                        capped.min(max_content.max(line_sum))
                    } else {
                        capped
                    };
                    return (effective_total - content_offset).max(0.0);
                }
                // 줄 단위 스냅: content_offset을 줄별로 소비하고 나머지 줄의 높이 합산
                // (layout의 compute_cell_line_ranges와 동일한 이산 계산)
                let mut offset_rem = content_offset;
                let mut visible_start = 0usize;
                for (i, &lh) in c.line_heights.iter().enumerate() {
                    if offset_rem <= 0.0 { break; }
                    if lh <= offset_rem {
                        offset_rem -= lh;
                        visible_start = i + 1;
                    } else {
                        // 줄 중간에서 offset 소진 → 이 줄부터 보임
                        offset_rem = 0.0;
                        visible_start = i;
                        break;
                    }
                }
                // visible_start 이후의 줄 높이 합산
                c.line_heights[visible_start..].iter().sum::<f64>().min(capped)
            })
            .fold(0.0f64, f64::max)
    }

    /// 지정 행의 셀별 패딩(상+하) 최대값을 반환한다.
    pub fn max_padding_for_row(&self, row: usize) -> f64 {
        self.cells.iter()
            .filter(|c| c.row == row && c.row_span == 1)
            .map(|c| c.padding_top + c.padding_bottom)
            .fold(0.0f64, f64::max)
    }

    /// 지정 행에서 오프셋 이후의 유효 행 높이를 반환한다 (콘텐츠 + 패딩).
    pub fn effective_row_height(&self, row: usize, content_offset: f64) -> f64 {
        let remaining = self.remaining_content_for_row(row, content_offset);
        let padding = self.max_padding_for_row(row);
        remaining + padding
    }

    /// 지정 행이 인트라-로우 분할 가능한지 판별한다.
    /// 행의 모든 셀이 단일 줄(≤1)이면 분할 불가 (이미지 셀).
    /// 2줄 이상의 셀이 하나라도 있으면 분할 가능 (텍스트 셀).
    pub fn is_row_splittable(&self, row: usize) -> bool {
        let cells_in_row: Vec<&MeasuredCell> = self.cells.iter()
            .filter(|c| c.row == row && c.row_span == 1)
            .collect();
        if cells_in_row.is_empty() {
            return false;
        }
        cells_in_row.iter().any(|c| c.line_heights.len() > 1)
    }

    /// 지정 행에서 첫 번째 줄의 최소 높이를 반환한다 (인트라-로우 분할 가능 여부 판단용).
    /// content_offset이 있으면 해당 오프셋 이후의 첫 줄 높이를 계산한다.
    pub fn min_first_line_height_for_row(&self, row: usize, content_offset: f64) -> f64 {
        let mut min_h = f64::MAX;
        for c in self.cells.iter().filter(|c| c.row == row && c.row_span == 1) {
            if c.line_heights.is_empty() {
                continue;
            }
            // content_offset 이후의 첫 줄 높이 찾기
            let mut cumulative = 0.0;
            for &lh in &c.line_heights {
                cumulative += lh;
                if cumulative > content_offset {
                    // 이 줄이 offset 경계를 넘음 — 이 줄이 첫 줄
                    if lh < min_h {
                        min_h = lh;
                    }
                    break;
                }
            }
        }
        if min_h == f64::MAX { 0.0 } else { min_h }
    }

    /// O(log R) 분할점: cursor_row부터 avail 높이에 들어가는 행 수 반환 (end_row, exclusive).
    /// effective_first_row_h: 첫 행의 유효 높이 (content_offset 반영).
    /// 인트라-로우 분할은 미고려.
    pub fn find_break_row(&self, avail: f64, cursor_row: usize, effective_first_row_h: f64) -> usize {
        let row_count = self.row_heights.len();
        if cursor_row >= row_count { return cursor_row; }
        let cs = self.cell_spacing;
        let delta = self.row_heights[cursor_row] - effective_first_row_h;
        let adj_cs = if cursor_row > 0 { cs } else { 0.0 };
        let target = self.cumulative_heights[cursor_row] + avail + delta + adj_cs;
        let search_start = cursor_row + 1;
        if search_start > row_count { return cursor_row; }
        let pos = self.cumulative_heights[search_start..=row_count]
            .partition_point(|&h| h <= target);
        (cursor_row + pos).min(row_count)
    }

    /// O(1) 행 범위 높이 조회 (cell_spacing 포함).
    /// start_row..end_row 범위의 높이 (첫 행 앞에는 cs 미포함).
    pub fn range_height(&self, start_row: usize, end_row: usize) -> f64 {
        if end_row <= start_row { return 0.0; }
        let diff = self.cumulative_heights[end_row] - self.cumulative_heights[start_row];
        if start_row > 0 { diff - self.cell_spacing } else { diff }
    }

    /// 주어진 행이 속한 rowspan 묶음 블록 (start, end_exclusive, height) 반환 (Task #398).
    /// 단일 행 블록(rowspan=1만 포함)이면 (row, row+1, row_heights[row]) 반환.
    /// row가 범위를 벗어나거나 row_block_* 가 비어있으면 단일 행으로 처리.
    pub fn row_block_for(&self, row: usize) -> (usize, usize, f64) {
        let rc = self.row_heights.len();
        if row >= rc {
            return (row, row, 0.0);
        }
        let start = self.row_block_start.get(row).copied().unwrap_or(row);
        let end = self.row_block_end.get(row).copied().unwrap_or(row + 1);
        // 방어적 보정: 잘못된 데이터 시 단일 행으로
        let start = start.min(row);
        let end = end.max(row + 1).min(rc);
        let h = self.range_height(start, end);
        (start, end, h)
    }

    /// 종료 행 후보가 *보호 대상* rowspan 묶음 블록 중간이면 블록 시작 행으로 후퇴.
    /// 블록 크기가 BLOCK_UNIT_MAX_ROWS (=3) 초과인 큰 rowspan 묶음은 행 단위 분할 허용 (Task #398 v2).
    /// [Task #474] RowBreak 표는 행 경계 분할이 명시 정책이라 보호 비적용.
    pub fn snap_to_block_boundary(&self, end_row: usize) -> usize {
        let rc = self.row_heights.len();
        if end_row >= rc {
            return end_row.min(rc);
        }
        // [Task #474] RowBreak 표는 보호 블록 정책 비적용 (HWP 행 경계 분할 정책 정합)
        if self.allows_row_break_split() {
            return end_row;
        }
        let block_start = self.row_block_start.get(end_row).copied().unwrap_or(end_row);
        let block_end = self.row_block_end.get(end_row).copied().unwrap_or(end_row + 1);
        if end_row == block_start {
            return end_row;
        }
        let block_size = block_end.saturating_sub(block_start);
        if block_size <= BLOCK_UNIT_MAX_ROWS {
            block_start
        } else {
            end_row
        }
    }

    /// [Task #474] 표 정책이 RowBreak 인지 확인. RowBreak 표는 행 경계 분할이
    /// 명시 정책이므로 rowspan 보호 블록 정책 비적용 대상.
    pub fn allows_row_break_split(&self) -> bool {
        matches!(self.page_break, crate::model::table::TablePageBreak::RowBreak)
    }
}

/// 블록 단위 보호 분할의 최대 rowspan. 이 값을 초과하는 큰 rowspan 묶음은
/// 행 단위 분할을 허용하여 페이지 잔여 공간을 활용한다 (Task #398 v2, HanCom-compat).
pub const BLOCK_UNIT_MAX_ROWS: usize = 3;

/// 표의 모든 셀을 검사하여 rowspan 묶음 블록 경계를 산출한다 (Task #398).
/// row_block_start[r] = r 행을 포함하는 셀들의 최소 시작 행
/// row_block_end[r]   = r 행을 포함하는 셀들의 최대 종료 행 (exclusive)
/// 겹치는 블록은 전이 폐포로 통합한다.
fn compute_row_blocks(
    table: &crate::model::table::Table,
    row_count: usize,
) -> (Vec<usize>, Vec<usize>) {
    if row_count == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut start: Vec<usize> = (0..row_count).collect();
    let mut end: Vec<usize> = (1..=row_count).collect();
    // 1단계: rowspan>1 셀로 블록 확장
    for cell in &table.cells {
        let r0 = cell.row as usize;
        let rs = (cell.row_span as usize).max(1);
        if r0 >= row_count { continue; }
        let r1 = (r0 + rs).min(row_count);
        for r in r0..r1 {
            if start[r] > r0 { start[r] = r0; }
            if end[r] < r1   { end[r] = r1; }
        }
    }
    // 2단계: 전이 폐포 (겹치는 블록 통합)
    loop {
        let mut changed = false;
        for r in 0..row_count {
            let s = start[r];
            let e = end[r];
            // 같은 블록 내 모든 행의 start 최소값, end 최대값으로 평탄화
            let mut new_s = s;
            let mut new_e = e;
            for r2 in s..e {
                if start[r2] < new_s { new_s = start[r2]; }
                if end[r2]   > new_e { new_e = end[r2];   }
            }
            if new_s != s || new_e != e {
                start[r] = new_s;
                end[r]   = new_e;
                changed = true;
            }
        }
        if !changed { break; }
    }
    // 3단계: 같은 블록 내 모든 행이 동일 (start, end) 가지도록 정규화
    let mut r = 0;
    while r < row_count {
        let s = start[r];
        let e = end[r];
        for r2 in s..e {
            start[r2] = s;
            end[r2]   = e;
        }
        r = e;
    }
    (start, end)
}

impl MeasuredSection {
    /// 문단 인덱스로 측정된 문단 높이를 조회한다.
    pub fn get_paragraph_height(&self, para_index: usize) -> Option<f64> {
        self.paragraphs.get(para_index).map(|p| p.total_height)
    }

    /// 문단 내 표의 측정된 높이를 조회한다.
    pub fn get_table_height(&self, para_index: usize, control_index: usize) -> Option<f64> {
        self.tables.iter()
            .find(|t| t.para_index == para_index && t.control_index == control_index)
            .map(|t| t.total_height)
    }

    /// 문단 내 표의 측정 정보 전체를 조회한다.
    pub fn get_measured_table(&self, para_index: usize, control_index: usize) -> Option<&MeasuredTable> {
        self.tables.iter()
            .find(|t| t.para_index == para_index && t.control_index == control_index)
    }

    /// 문단 인덱스로 측정된 문단 정보 전체를 조회한다.
    pub fn get_measured_paragraph(&self, para_index: usize) -> Option<&MeasuredParagraph> {
        self.paragraphs.get(para_index)
    }

    /// 문단이 표를 포함하는지 확인한다.
    pub fn paragraph_has_table(&self, para_index: usize) -> bool {
        self.paragraphs.get(para_index)
            .map(|p| p.has_table)
            .unwrap_or(false)
    }

    /// 문단 삽입 시 인덱스 조정 (전체 재측정 회피).
    /// insert_at 위치에 더미 측정값을 삽입하고, 이후 표의 para_index를 +1.
    pub fn shift_for_insert(&mut self, insert_at: usize) {
        // 표 para_index 조정
        for table in &mut self.tables {
            if table.para_index >= insert_at {
                table.para_index += 1;
            }
        }
        // 더미 문단 측정값 삽입 (dirty로 표시되어 재측정됨)
        let dummy = MeasuredParagraph {
            para_index: insert_at,
            total_height: 0.0,
            line_heights: vec![0.0],
            line_spacings: vec![0.0],
            spacing_before: 0.0,
            spacing_after: 0.0,
            has_table: false,
            has_picture: false,
            picture_height: 0.0,
        };
        if insert_at <= self.paragraphs.len() {
            self.paragraphs.insert(insert_at, dummy);
        }
        // para_index 재정렬
        for (i, p) in self.paragraphs.iter_mut().enumerate() {
            p.para_index = i;
        }
    }

    /// 문단 삭제 시 인덱스 조정 (전체 재측정 회피).
    /// remove_at 위치의 측정값을 제거하고, 이후 표의 para_index를 -1.
    pub fn shift_for_remove(&mut self, remove_at: usize) {
        // 삭제된 문단의 표 측정값 제거
        self.tables.retain(|t| t.para_index != remove_at);
        // 표 para_index 조정
        for table in &mut self.tables {
            if table.para_index > remove_at {
                table.para_index -= 1;
            }
        }
        // 문단 측정값 제거
        if remove_at < self.paragraphs.len() {
            self.paragraphs.remove(remove_at);
        }
        // para_index 재정렬
        for (i, p) in self.paragraphs.iter_mut().enumerate() {
            p.para_index = i;
        }
    }
}

impl HeightMeasurer {
    /// 각주 영역의 총 높이를 추정한다.
    ///
    /// 각주 영역 = 구분선 여백 + 각주 문단들 높이 + 각주 간 간격
    pub fn estimate_footnote_area_height(
        &self,
        footnotes: &[&Footnote],
        footnote_shape: Option<&FootnoteShape>,
    ) -> f64 {
        if footnotes.is_empty() {
            return 0.0;
        }

        // 기본값: FootnoteShape이 없으면 기본 여백 사용
        let separator_margin_top = footnote_shape
            .map(|s| hwpunit_to_px(s.separator_margin_top as i32, self.dpi))
            .unwrap_or(8.0); // 약 0.6mm
        let separator_margin_bottom = footnote_shape
            .map(|s| hwpunit_to_px(s.separator_margin_bottom as i32, self.dpi))
            .unwrap_or(4.0); // 약 0.3mm
        let note_spacing = footnote_shape
            .map(|s| hwpunit_to_px(s.note_spacing as i32, self.dpi))
            .unwrap_or(2.0); // 약 0.15mm
        let separator_height = 1.0; // 구분선 두께 (1px)

        // 각주 문단 높이 합산
        let mut footnote_content_height = 0.0;
        for (i, footnote) in footnotes.iter().enumerate() {
            // 각주 문단 높이 추정: LineSeg가 있으면 사용, 없으면 기본값
            let mut fn_height = 0.0;
            for para in &footnote.paragraphs {
                if para.line_segs.is_empty() {
                    fn_height += hwpunit_to_px(400, self.dpi); // 기본 약 14pt
                } else {
                    for seg in &para.line_segs {
                        fn_height += hwpunit_to_px(seg.line_height, self.dpi);
                    }
                }
            }
            // 빈 각주도 최소 높이 보장
            if fn_height <= 0.0 {
                fn_height = hwpunit_to_px(400, self.dpi);
            }
            footnote_content_height += fn_height;

            // 각주 간 간격 (마지막 각주 제외)
            if i < footnotes.len() - 1 {
                footnote_content_height += note_spacing;
            }
        }

        // 총 높이 = 구분선 위 여백 + 구분선 + 구분선 아래 여백 + 각주 내용
        separator_margin_top + separator_height + separator_margin_bottom + footnote_content_height
    }

    /// 단일 각주의 높이를 추정한다.
    pub fn estimate_single_footnote_height(&self, footnote: &Footnote) -> f64 {
        let mut fn_height = 0.0;
        for para in &footnote.paragraphs {
            if para.line_segs.is_empty() {
                fn_height += hwpunit_to_px(400, self.dpi);
            } else {
                for seg in &para.line_segs {
                    fn_height += hwpunit_to_px(seg.line_height, self.dpi);
                }
            }
        }
        if fn_height <= 0.0 {
            fn_height = hwpunit_to_px(400, self.dpi);
        }
        fn_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{Paragraph, LineSeg};
    use crate::model::table::{Table, Cell};

    #[test]
    fn test_measure_empty_section() {
        let measurer = HeightMeasurer::with_default_dpi();
        let paragraphs: Vec<Paragraph> = Vec::new();
        let composed: Vec<ComposedParagraph> = Vec::new();
        let styles = ResolvedStyleSet::default();

        let result = measurer.measure_section(&paragraphs, &composed, &styles);
        assert!(result.paragraphs.is_empty());
        assert!(result.tables.is_empty());
    }

    #[test]
    fn test_measure_single_paragraph() {
        let measurer = HeightMeasurer::with_default_dpi();
        let paragraphs = vec![Paragraph {
            line_segs: vec![LineSeg {
                line_height: 400,
                ..Default::default()
            }],
            ..Default::default()
        }];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let styles = ResolvedStyleSet::default();

        let result = measurer.measure_section(&paragraphs, &composed, &styles);
        assert_eq!(result.paragraphs.len(), 1);
        assert!(result.paragraphs[0].total_height > 0.0);
    }

    #[test]
    fn test_measure_table() {
        let measurer = HeightMeasurer::with_default_dpi();
        let table = Table {
            row_count: 2,
            col_count: 2,
            cells: vec![
                Cell { row: 0, col: 0, row_span: 1, col_span: 1, height: 500, width: 1000, ..Default::default() },
                Cell { row: 0, col: 1, row_span: 1, col_span: 1, height: 500, width: 1000, ..Default::default() },
                Cell { row: 1, col: 0, row_span: 1, col_span: 1, height: 600, width: 1000, ..Default::default() },
                Cell { row: 1, col: 1, row_span: 1, col_span: 1, height: 600, width: 1000, ..Default::default() },
            ],
            ..Default::default()
        };

        let styles = ResolvedStyleSet::default();
        let measured = measurer.measure_table(&table, 0, 0, &styles);
        assert_eq!(measured.row_heights.len(), 2);
        assert!(measured.total_height > 0.0);
    }

    #[test]
    fn test_cumulative_heights_consistency() {
        // cumulative_heights[row_count] == table_height (cell_spacing 포함)
        let measurer = HeightMeasurer::with_default_dpi();
        let table = Table {
            row_count: 3,
            col_count: 1,
            cell_spacing: 100,
            cells: vec![
                Cell { row: 0, col: 0, row_span: 1, col_span: 1, height: 1000, width: 5000, ..Default::default() },
                Cell { row: 1, col: 0, row_span: 1, col_span: 1, height: 2000, width: 5000, ..Default::default() },
                Cell { row: 2, col: 0, row_span: 1, col_span: 1, height: 1500, width: 5000, ..Default::default() },
            ],
            ..Default::default()
        };
        let styles = ResolvedStyleSet::default();
        let mt = measurer.measure_table(&table, 0, 0, &styles);

        assert_eq!(mt.cumulative_heights.len(), 4); // row_count + 1
        assert_eq!(mt.cumulative_heights[0], 0.0);

        // cumulative_heights 마지막 값은 row_heights 합 + cs * (row_count - 1)
        let expected_total: f64 = mt.row_heights.iter().sum::<f64>()
            + mt.cell_spacing * 2.0;
        assert!((mt.cumulative_heights[3] - expected_total).abs() < 0.001,
            "cumulative_heights[3]={} expected={}", mt.cumulative_heights[3], expected_total);
    }

    #[test]
    fn test_find_break_row_all_fit() {
        // 모든 행이 들어가는 경우
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0], // 0, 20, 20+30+5, 55+25+5
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        let end = mt.find_break_row(200.0, 0, 20.0); // 200px 충분
        assert_eq!(end, 3); // 전부 fit
    }

    #[test]
    fn test_find_break_row_partial() {
        // 일부만 들어가는 경우
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0, 40.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0, 130.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        // avail=60, cursor=0, first_row_h=20
        // range(0,1)=20, range(0,2)=55, range(0,3)=85 > 60
        let end = mt.find_break_row(60.0, 0, 20.0);
        assert_eq!(end, 2); // 행 0,1 fit (높이 55), 행 2 초과

        // cursor=1: range(1,2)=cumul[2]-cumul[1]-cs = 55-20-5=30
        //           range(1,3)=cumul[3]-cumul[1]-cs = 85-20-5=60
        //           range(1,4)=cumul[4]-cumul[1]-cs = 130-20-5=105 > 60
        let end2 = mt.find_break_row(60.0, 1, 30.0);
        assert_eq!(end2, 3); // 행 1,2 fit (높이 60), 행 3 초과
    }

    #[test]
    fn test_find_break_row_first_doesnt_fit() {
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![50.0, 30.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 50.0, 85.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        let end = mt.find_break_row(30.0, 0, 50.0); // 30 < 50
        assert_eq!(end, 0); // 첫 행도 안 들어감
    }

    #[test]
    fn test_range_height() {
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        // range(0,0) = 0
        assert_eq!(mt.range_height(0, 0), 0.0);
        // range(0,1) = row[0] = 20
        assert!((mt.range_height(0, 1) - 20.0).abs() < 0.001);
        // range(0,2) = row[0] + row[1] + cs = 55
        assert!((mt.range_height(0, 2) - 55.0).abs() < 0.001);
        // range(0,3) = row[0] + row[1] + cs + row[2] + cs = 85
        assert!((mt.range_height(0, 3) - 85.0).abs() < 0.001);
        // range(1,2) = row[1] = 30 (cursor>0: diff-cs = 55-20-5 = 30)
        assert!((mt.range_height(1, 2) - 30.0).abs() < 0.001);
        // range(1,3) = row[1] + row[2] + cs = 60 (cursor>0: diff-cs = 85-20-5 = 60)
        assert!((mt.range_height(1, 3) - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_find_break_row_with_content_offset() {
        // effective_first_row_h < row_heights[cursor_row]일 때 더 많은 행이 fit
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![50.0, 30.0, 25.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 50.0, 85.0, 115.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        // avail=60, effective_first=50 → end=1 (range(0,1)=50, range(0,2)=85>60)
        let end1 = mt.find_break_row(60.0, 0, 50.0);
        assert_eq!(end1, 1);

        // avail=60, effective_first=20 (content_offset로 첫 행 줄어듦)
        // delta=50-20=30, target=0+60+30+0=90, cumul[1]=50≤90✓, cumul[2]=85≤90✓, cumul[3]=115>90
        let end2 = mt.find_break_row(60.0, 0, 20.0);
        assert_eq!(end2, 2); // 더 많은 행 fit
    }

    #[test]
    fn test_find_break_row_empty_table() {
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 0.0,
            row_heights: vec![],
            caption_height: 0.0, cell_spacing: 0.0,
            cumulative_heights: vec![0.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        assert_eq!(mt.find_break_row(100.0, 0, 0.0), 0);
        assert_eq!(mt.range_height(0, 0), 0.0);
    }

    #[test]
    fn test_find_break_row_single_row() {
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 50.0,
            row_heights: vec![50.0],
            caption_height: 0.0, cell_spacing: 0.0,
            cumulative_heights: vec![0.0, 50.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        assert_eq!(mt.find_break_row(100.0, 0, 50.0), 1); // fit
        assert_eq!(mt.find_break_row(30.0, 0, 50.0), 0); // doesn't fit
    }

    // ─────────────────────────────────────────────────────────────────────
    // Task #398: rowspan 묶음 블록 테스트
    // ─────────────────────────────────────────────────────────────────────

    fn make_table_with_cells(row_count: u16, col_count: u16, cells: Vec<crate::model::table::Cell>)
        -> crate::model::table::Table
    {
        crate::model::table::Table {
            row_count, col_count, cells,
            ..Default::default()
        }
    }

    fn cell_rs(row: u16, col: u16, row_span: u16) -> crate::model::table::Cell {
        crate::model::table::Cell {
            row, col, row_span,
            col_span: 1,
            ..Default::default()
        }
    }

    #[test]
    fn test_compute_row_blocks_all_single() {
        // 모든 셀 rowspan=1 → 각 행이 자기 자신만 포함하는 블록
        let table = make_table_with_cells(3, 2, vec![
            cell_rs(0, 0, 1), cell_rs(0, 1, 1),
            cell_rs(1, 0, 1), cell_rs(1, 1, 1),
            cell_rs(2, 0, 1), cell_rs(2, 1, 1),
        ]);
        let (s, e) = compute_row_blocks(&table, 3);
        assert_eq!(s, vec![0, 1, 2]);
        assert_eq!(e, vec![1, 2, 3]);
    }

    #[test]
    fn test_compute_row_blocks_rs2_at_row0() {
        // 행 0에 rs=2 셀 → 블록 0~2
        let table = make_table_with_cells(3, 2, vec![
            cell_rs(0, 0, 1), cell_rs(0, 1, 2), // rs=2
            cell_rs(1, 0, 1),
            cell_rs(2, 0, 1), cell_rs(2, 1, 1),
        ]);
        let (s, e) = compute_row_blocks(&table, 3);
        assert_eq!(s, vec![0, 0, 2]);
        assert_eq!(e, vec![2, 2, 3]);
    }

    #[test]
    fn test_compute_row_blocks_overlapping() {
        // 셀 A: rows 0~2, 셀 B: rows 1~3 → 통합 블록 0~3
        let table = make_table_with_cells(4, 3, vec![
            cell_rs(0, 0, 3), // rows 0,1,2
            cell_rs(1, 1, 3), // rows 1,2,3
            cell_rs(0, 2, 1),
            cell_rs(3, 0, 1),
        ]);
        let (s, e) = compute_row_blocks(&table, 4);
        assert_eq!(s, vec![0, 0, 0, 0]);
        assert_eq!(e, vec![4, 4, 4, 4]);
    }

    #[test]
    fn test_compute_row_blocks_disjoint() {
        // 비인접 rowspan은 별개 블록
        let table = make_table_with_cells(5, 1, vec![
            cell_rs(0, 0, 2), // rows 0~1
            cell_rs(2, 0, 1),
            cell_rs(3, 0, 2), // rows 3~4
        ]);
        let (s, e) = compute_row_blocks(&table, 5);
        assert_eq!(s, vec![0, 0, 2, 3, 3]);
        assert_eq!(e, vec![2, 2, 3, 5, 5]);
    }

    #[test]
    fn test_row_block_for_basic() {
        // 행 0+1을 묶는 rs=2 셀
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![0, 0, 2], row_block_end: vec![2, 2, 3],
        };
        // 행 0: 블록 (0, 2, h=20+30+5=55)
        let (s, e, h) = mt.row_block_for(0);
        assert_eq!((s, e), (0, 2));
        assert!((h - 55.0).abs() < 0.001);
        // 행 1: 같은 블록 (0, 2)
        let (s, e, h) = mt.row_block_for(1);
        assert_eq!((s, e), (0, 2));
        assert!((h - 55.0).abs() < 0.001);
        // 행 2: 단일 블록 (2, 3, h=25)
        let (s, e, h) = mt.row_block_for(2);
        assert_eq!((s, e), (2, 3));
        assert!((h - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_row_block_for_empty_metadata() {
        // row_block_* 비어있으면 단일 행으로 처리
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 50.0,
            row_heights: vec![20.0, 30.0],
            caption_height: 0.0, cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        let (s, e, h) = mt.row_block_for(0);
        assert_eq!((s, e), (0, 1));
        assert!((h - 20.0).abs() < 0.001);
        let (s, e, h) = mt.row_block_for(1);
        assert_eq!((s, e), (1, 2));
        assert!((h - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_snap_to_block_boundary() {
        // 블록 0~2, 단일 행 2, 블록 3~4 (행 3+4)
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![10.0, 10.0, 10.0, 10.0, 10.0],
            caption_height: 0.0, cell_spacing: 0.0,
            cumulative_heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![], page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![0, 0, 2, 3, 3],
            row_block_end:   vec![2, 2, 3, 5, 5],
        };
        // end_row=0: 블록 시작 → 0
        assert_eq!(mt.snap_to_block_boundary(0), 0);
        // end_row=1: 블록 0~2 중간 → 0으로 후퇴
        assert_eq!(mt.snap_to_block_boundary(1), 0);
        // end_row=2: 블록 시작 (단일 행 2) → 2
        assert_eq!(mt.snap_to_block_boundary(2), 2);
        // end_row=3: 블록 시작 → 3
        assert_eq!(mt.snap_to_block_boundary(3), 3);
        // end_row=4: 블록 3~5 중간 → 3으로 후퇴
        assert_eq!(mt.snap_to_block_boundary(4), 3);
        // end_row=5: 행 범위 끝 → 5 (snap 없음)
        assert_eq!(mt.snap_to_block_boundary(5), 5);
    }

    #[test]
    fn test_snap_to_block_boundary_row_break_skipped() {
        // [Task #474] RowBreak 표는 보호 블록 정책 비적용 — end_row 그대로 반환
        let mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 100.0,
            row_heights: vec![10.0, 10.0, 10.0, 10.0, 10.0],
            caption_height: 0.0, cell_spacing: 0.0,
            cumulative_heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0],
            repeat_header: false, has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::RowBreak,
            row_block_start: vec![0, 0, 2, 3, 3],
            row_block_end:   vec![2, 2, 3, 5, 5],
        };
        // None 정책에서는 end_row=1 → 0 으로 후퇴, RowBreak 에서는 1 그대로
        assert_eq!(mt.snap_to_block_boundary(1), 1);
        // None 정책에서는 end_row=4 → 3 으로 후퇴, RowBreak 에서는 4 그대로
        assert_eq!(mt.snap_to_block_boundary(4), 4);
    }

    #[test]
    fn test_allows_row_break_split() {
        // [Task #474] page_break 정책 별 RowBreak 인지 확인
        let mut mt = MeasuredTable {
            para_index: 0, control_index: 0, total_height: 0.0,
            row_heights: vec![], caption_height: 0.0, cell_spacing: 0.0,
            cumulative_heights: vec![0.0], repeat_header: false,
            has_header_cells: false, cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![], row_block_end: vec![],
        };
        assert!(!mt.allows_row_break_split());
        mt.page_break = crate::model::table::TablePageBreak::CellBreak;
        assert!(!mt.allows_row_break_split());
        mt.page_break = crate::model::table::TablePageBreak::RowBreak;
        assert!(mt.allows_row_break_split());
    }
}
