//! 콘텐츠 높이 측정 모듈
//!
//! 페이지네이션 전에 각 콘텐츠의 실제 렌더링 높이를 측정한다.
//! LayoutEngine과 동일한 계산 로직을 사용하여 정확한 높이를 산출한다.

use super::composer::{compose_paragraph, ComposedParagraph};
use super::style_resolver::ResolvedStyleSet;
use super::{hwpunit_to_px, DEFAULT_DPI};
use crate::model::control::Control;
use crate::model::footnote::{Footnote, FootnoteShape};
use crate::model::paragraph::Paragraph;
use crate::model::shape::{Caption, CommonObjAttr, TextWrap, VertRelTo};
use crate::model::table::{Table, TablePageBreak};

/// treat_as_char 표가 인라인(텍스트와 나란히)인지 판별
///
/// 인라인 조건:
/// 1. 텍스트가 있으면 → 표 너비가 줄 너비의 90% 미만
/// 2. 텍스트가 없어도 → 같은 문단에 TAC 표가 2개 이상이고 합산 너비가 줄 너비 이내
pub fn is_tac_table_inline(
    table: &Table,
    seg_width: i32,
    text: &str,
    controls: &[Control],
) -> bool {
    let table_width: u32 = table.get_column_widths().iter().sum();

    if !text.is_empty() {
        return (table_width as i32) < (seg_width as f64 * 0.9) as i32;
    }

    // 텍스트 없는 문단: 다중 TAC 표의 합산 너비가 줄 너비 이내이면 인라인
    let tac_tables: Vec<&Table> = controls
        .iter()
        .filter_map(|c| match c {
            Control::Table(t) if t.common.treat_as_char => Some(t.as_ref()),
            _ => None,
        })
        .collect();

    if tac_tables.len() >= 2 {
        let total_width: u32 = tac_tables
            .iter()
            .map(|t| t.get_column_widths().iter().sum::<u32>())
            .sum();
        return (total_width as i32) <= seg_width;
    }

    false
}

/// treat_as_char 표가 문단 문맥에서 인라인인지 판별
///
/// 앵커 양쪽에 실제 텍스트(Letter/Number 가시 글자)가 있으면 텍스트 순서를
/// 보존하기 위해 인라인으로 판정하고, 그 외에는 [`is_tac_table_inline`] 규칙을
/// 따른다. HWP TAC 필러(U+F081C 등 PUA)·공백·오브젝트마커만 있는 문단
/// (예: 복학원서.hwp pi=16)은 실제 텍스트가 아니므로 여기서 제외된다.
pub fn is_tac_table_inline_in_para(table: &Table, seg_width: i32, para: &Paragraph) -> bool {
    let chars: Vec<char> = para.text.chars().collect();
    let control_positions = para.control_text_positions();
    let has_middle_anchor = para
        .controls
        .iter()
        .enumerate()
        .any(|(control_index, control)| {
            matches!(control, Control::Table(candidate) if std::ptr::eq(candidate.as_ref(), table))
                && control_positions
                    .get(control_index)
                    .is_some_and(|&position| {
                        chars
                            .get(..position)
                            .is_some_and(|before| before.iter().any(|ch| ch.is_alphanumeric()))
                            && chars
                                .get(position..)
                                .is_some_and(|after| after.iter().any(|ch| ch.is_alphanumeric()))
                    })
        });
    if has_middle_anchor {
        return true;
    }

    is_tac_table_inline(table, seg_width, &para.text, &para.controls)
}

fn empty_paragraph_fallback_line_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
) -> Option<(f64, f64)> {
    if !para.text.is_empty()
        || !para.controls.is_empty()
        || !para.line_segs.is_empty()
        || para.char_count == 0
    {
        return None;
    }
    let char_shape_id =
        para.char_shape_id_at(0)
            .or_else(|| para.char_shapes.first().map(|cs| cs.char_shape_id))? as usize;
    let char_style = styles.char_styles.get(char_shape_id)?;
    let font_size = char_style.font_size;
    if font_size <= 0.0 {
        return None;
    }
    let small_empty_para_max_font = hwpunit_to_px(1000, DEFAULT_DPI);
    if font_size > small_empty_para_max_font + 0.1 {
        return None;
    }
    let meaningful_empty_para_min_font = hwpunit_to_px(800, DEFAULT_DPI);
    if !char_style.bold && font_size < meaningful_empty_para_min_font - 0.1 {
        return None;
    }
    let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
    let ls_type = para_style
        .map(|s| s.line_spacing_type)
        .unwrap_or(crate::model::style::LineSpacingType::Percent);
    Some(crate::renderer::corrected_line_metrics(
        0.0, 0.0, font_size, ls_type, ls_val,
    ))
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
        range
            .into_iter()
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

pub fn fit_measured_table_to_declared_height(
    measured: &MeasuredTable,
    table: &Table,
    dpi: f64,
) -> MeasuredTable {
    let mut fitted = measured.clone();
    if fitted.row_heights.is_empty() || table.common.height == 0 {
        return fitted;
    }

    let row_count = fitted.row_heights.len();
    let cell_spacing_total = fitted.cell_spacing * row_count.saturating_sub(1) as f64;
    let target_body_height = hwpunit_to_px(table.common.height as i32, dpi);
    let target_row_sum = (target_body_height - cell_spacing_total).max(0.0);
    let current_row_sum = fitted.row_heights.iter().sum::<f64>();

    // 선언 높이 보정은 #1510처럼 측정값과 저장값이 근소하게 어긋난 fixed-size 표에만
    // 적용한다. 콘텐츠가 선언 높이보다 훨씬 큰 표를 강제로 압축하면 행/중첩 표 분할
    // 페이지가 앞당겨진다(#1073).
    let min_reasonable = current_row_sum * 0.75;
    let max_reasonable = current_row_sum * 1.35;
    if target_row_sum < min_reasonable || target_row_sum > max_reasonable {
        return fitted;
    }

    if target_row_sum > 0.0 && (current_row_sum - target_row_sum).abs() > 0.5 {
        if current_row_sum > 0.0 {
            let scale = target_row_sum / current_row_sum;
            for row_height in &mut fitted.row_heights {
                *row_height *= scale;
            }
        } else {
            let per_row = target_row_sum / row_count as f64;
            for row_height in &mut fitted.row_heights {
                *row_height = per_row;
            }
        }
    }

    fitted.cumulative_heights = vec![0.0; row_count + 1];
    for (idx, row_height) in fitted.row_heights.iter().enumerate() {
        let cell_spacing = if idx > 0 { fitted.cell_spacing } else { 0.0 };
        fitted.cumulative_heights[idx + 1] =
            fitted.cumulative_heights[idx] + row_height + cell_spacing;
    }

    let previous_body_height =
        current_row_sum + measured.cell_spacing * row_count.saturating_sub(1) as f64;
    let caption_and_spacing = (measured.total_height - previous_body_height).max(0.0);
    fitted.total_height = target_body_height + caption_and_spacing;
    fitted
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
    /// [Task #1073] 셀이 분할 가능한 단일 중첩 표(텍스트 없는 문단 + 2행 이상)를 가지면
    /// 그 표의 행 수. 아니면 0. `is_row_splittable` 가 중첩행 분할 가부 판정에 사용.
    pub nested_split_row_count: usize,
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
    is_hwp3_variant: bool,
    use_hwp3_origin_flow_spacing_before: bool,
}

impl HeightMeasurer {
    pub fn new(dpi: f64) -> Self {
        Self {
            dpi,
            is_hwp3_variant: false,
            use_hwp3_origin_flow_spacing_before: false,
        }
    }

    pub fn with_hwp3_variant(mut self, enabled: bool) -> Self {
        self.is_hwp3_variant = enabled;
        self.use_hwp3_origin_flow_spacing_before = enabled;
        self
    }

    pub fn with_hwp3_origin_flow_spacing_before(mut self, enabled: bool) -> Self {
        self.use_hwp3_origin_flow_spacing_before = enabled;
        self
    }

    pub fn with_default_dpi() -> Self {
        Self::new(DEFAULT_DPI)
    }

    /// 셀 안 비-TAC 자리차지 개체가 표 흐름에 요구하는 세로 범위.
    fn non_inline_control_flow_height(&self, common: &CommonObjAttr) -> f64 {
        if common.treat_as_char || !matches!(common.text_wrap, TextWrap::TopAndBottom) {
            return 0.0;
        }
        let object_height = hwpunit_to_px(common.height as i32, self.dpi)
            + hwpunit_to_px(common.margin.top as i32, self.dpi)
            + hwpunit_to_px(common.margin.bottom as i32, self.dpi);
        if matches!(common.vert_rel_to, VertRelTo::Para) {
            if common.flow_with_text {
                hwpunit_to_px((common.vertical_offset as i32).max(0), self.dpi) + object_height
            } else {
                0.0
            }
        } else {
            object_height
        }
    }

    fn cell_wrap_object_visual_bottom(&self, common: &CommonObjAttr) -> f64 {
        if common.treat_as_char {
            return 0.0;
        }
        if !matches!(
            common.text_wrap,
            TextWrap::Square | TextWrap::Tight | TextWrap::Through
        ) {
            return 0.0;
        }

        let object_height = hwpunit_to_px(common.height as i32, self.dpi);
        let top_offset = if matches!(common.vert_rel_to, VertRelTo::Para) {
            hwpunit_to_px((common.vertical_offset as i32).max(0), self.dpi)
        } else {
            0.0
        };
        top_offset + object_height
    }

    fn cell_wrap_objects_bottom_height(&self, paragraphs: &[Paragraph]) -> f64 {
        // [Task #2226] 같은 문단에 TopAndBottom flow 개체가 있으면 첫 seg vpos 는
        // 개체에 밀려난 줄 위치라 문단 시작이 아니다 — para_top 을 사다리(이전
        // 문단 extent, 첫 문단 0)로 잡아야 개체 오프셋(문단 기준)과 원점이 맞는다.
        // (주보 p2 로고 표: line vpos 10087 + 그림 bottom 10087 = 269px 이중 계상)
        let mut prev_extent = 0.0f64;
        paragraphs
            .iter()
            .map(|p| {
                let first_vpos = p
                    .line_segs
                    .first()
                    .map(|s| hwpunit_to_px(s.vertical_pos, self.dpi))
                    .unwrap_or(0.0);
                // 개체가 문단 시작~줄 상단 구간을 채우는 배치(줄이 개체 아래로
                // 밀림)면 first_vpos 는 문단 시작이 아니다 — 기하 판정으로 전환.
                let probe_object_bottom = p
                    .controls
                    .iter()
                    .map(|ctrl| match ctrl {
                        Control::Picture(pic) => self.cell_wrap_object_visual_bottom(&pic.common),
                        Control::Shape(shape) => {
                            self.cell_wrap_object_visual_bottom(shape.common())
                        }
                        _ => 0.0,
                    })
                    .fold(0.0f64, f64::max);
                let objects_above_line = probe_object_bottom > 0.0
                    && prev_extent + probe_object_bottom <= first_vpos + 0.5;
                let para_top = if objects_above_line {
                    prev_extent
                } else {
                    first_vpos
                };
                let para_extent = p
                    .line_segs
                    .iter()
                    .map(|s| hwpunit_to_px(s.vertical_pos + s.line_height.max(0), self.dpi))
                    .fold(prev_extent, f64::max);
                prev_extent = para_extent;
                let object_bottom = p
                    .controls
                    .iter()
                    .map(|ctrl| match ctrl {
                        Control::Picture(pic) => self.cell_wrap_object_visual_bottom(&pic.common),
                        Control::Shape(shape) => {
                            self.cell_wrap_object_visual_bottom(shape.common())
                        }
                        _ => 0.0,
                    })
                    .fold(0.0f64, f64::max);
                if object_bottom > 0.0 {
                    para_top + object_bottom
                } else {
                    0.0
                }
            })
            .fold(0.0f64, f64::max)
    }

    /// 구역의 모든 콘텐츠 높이를 측정한다.
    ///
    /// `column_width_px`: 단 너비 (px). `Some` 이면 line_segs.empty paragraph 의
    /// compose_lines fallback 결과를 단 너비 기반으로 recompose 하여 측정한다
    /// (Task #1042 Stage 6c: typeset/layout 측정 정합).
    pub fn measure_section(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        column_width_px: Option<f64>,
    ) -> MeasuredSection {
        let mut measured_paras = Vec::with_capacity(paragraphs.len());
        let mut measured_tables = Vec::new();

        for (para_idx, para) in paragraphs.iter().enumerate() {
            let comp = composed.get(para_idx);

            // 블록 표 컨트롤 감지 (일반 표 + treat_as_char 블록형)
            let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let has_table = para.controls.iter().any(|c| {
                matches!(c, Control::Table(t) if !t.common.treat_as_char
                    || (t.common.treat_as_char && !is_tac_table_inline_in_para(t, seg_width, para)))
            });

            // 그림 컨트롤 감지 및 높이 측정
            let has_picture = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Picture(_) | Control::Equation(_)));
            let picture_height = self.measure_pictures_in_paragraph(para);

            // 문단 높이 측정
            let measured = self.measure_paragraph(
                para,
                comp,
                styles,
                para_idx,
                has_table,
                has_picture,
                picture_height,
                column_width_px,
            );
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
        column_width_px: Option<f64>,
    ) -> MeasuredParagraph {
        // 문단 스타일에서 spacing 조회
        let para_style_id = composed.map(|c| c.para_style_id as usize).unwrap_or(0);
        let para_style = styles.para_styles.get(para_style_id);
        let spacing_before = crate::renderer::hwp3_variant_flow_spacing_before(
            para_style.map(|s| s.spacing_before).unwrap_or(0.0),
            self.use_hwp3_origin_flow_spacing_before,
        );
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);

        // [Task #1042 Stage 6c] line_segs.empty paragraph 의 compose_lines fallback
        // 결과를 단 너비 기반으로 recompose — paragraph_layout (Stage 6b) 와 동일.
        let recomposed: Option<ComposedParagraph> = match (composed, column_width_px) {
            (Some(c), Some(cw)) if para.line_segs.is_empty() && cw > 0.0 => {
                let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                let inner = (cw - margin_l - margin_r).max(0.0);
                if inner > 0.0 {
                    let mut cloned = c.clone();
                    crate::renderer::composer::recompose_for_cell_width(
                        &mut cloned,
                        para,
                        inner,
                        styles,
                    );
                    Some(cloned)
                } else {
                    None
                }
            }
            _ => None,
        };
        let composed = recomposed.as_ref().or(composed);

        // 줄별 높이 계산: 콘텐츠 높이(line_height)와 줄간격(line_spacing)을 분리 저장
        // line_height = 줄의 콘텐츠 영역 높이
        // line_spacing = 현재 줄 하단에서 다음 줄 상단까지의 추가 공간
        // Y advance = line_height + line_spacing (HWP LineSeg 실증 결과)
        //
        // layout_paragraph와 동일한 보정: LineSeg line_height가 해당 줄의 최대
        // 폰트 크기보다 작으면 ParaShape 줄간격 설정으로 재계산한다.
        let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
        let ls_type = para_style
            .map(|s| s.line_spacing_type)
            .unwrap_or(crate::model::style::LineSpacingType::Percent);

        let (line_heights, line_spacings): (Vec<f64>, Vec<f64>) = if let Some(comp) = composed {
            let tac_offsets_px: Vec<(usize, f64, usize)> = comp
                .tac_controls
                .iter()
                .map(|(pos, width_hu, control_index)| {
                    (*pos, hwpunit_to_px(*width_hu, self.dpi), *control_index)
                })
                .collect();
            let equation_line_available_width_px = |visual_line_idx: usize| {
                column_width_px.map(|cw| {
                    let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                    let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                    let indent = para_style.map(|s| s.indent).unwrap_or(0.0);
                    // [Task #1472] 변환본은 effective indent 불변 위해 scale 절반(2.0→1.0).
                    let eq_scale = 2.0 * if self.is_hwp3_variant { 0.5 } else { 1.0 };
                    let effective_margin_l = crate::renderer::equation_tac_flow::
                        paragraph_effective_margin_left_with_indent_scale(
                            margin_l,
                            indent,
                            visual_line_idx,
                            eq_scale,
                        );
                    (cw - effective_margin_l - margin_r).max(0.0)
                })
            };
            let mut pairs: Vec<(f64, f64)> = comp
                .lines
                .iter()
                .enumerate()
                .map(|(line_idx, line)| {
                    let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                    let max_fs = line
                        .runs
                        .iter()
                        .map(|r| {
                            styles
                                .char_styles
                                .get(r.char_style_id as usize)
                                .map(|cs| cs.font_size)
                                .unwrap_or(0.0)
                        })
                        .fold(0.0f64, f64::max);
                    // [Task #1042 Stage 6c] line_segs.empty path (raw_lh < max_fs) 의 lh/ls
                    // 분해 — HWP3/HWP5 line_segs 의 (line_height=base, line_spacing=extra)
                    // 의미와 정합. 종전 처럼 ls_val/100 전체를 line_height 에 baking 하면
                    // trailing_ls 제거 효과가 line_segs 있는 path 와 어긋남.
                    let (lh, line_spacing_px) = if max_fs > 0.0 && raw_lh < max_fs {
                        use crate::model::style::LineSpacingType;
                        let (base, extra) = match ls_type {
                            LineSpacingType::Percent => {
                                let e = (max_fs * (ls_val - 100.0) / 100.0).max(0.0);
                                (max_fs, e)
                            }
                            LineSpacingType::Fixed => (ls_val.max(max_fs), 0.0),
                            LineSpacingType::SpaceOnly => (max_fs, ls_val.max(0.0)),
                            LineSpacingType::Minimum => (ls_val.max(max_fs), 0.0),
                        };
                        (base, extra)
                    } else {
                        (raw_lh, hwpunit_to_px(line.line_spacing, self.dpi))
                    };
                    let extra_rows =
                        crate::renderer::equation_tac_flow::compute_equation_only_tac_line_flow(
                            Some(para),
                            comp,
                            &tac_offsets_px,
                            line_idx,
                            equation_line_available_width_px(0).unwrap_or(f64::INFINITY),
                            equation_line_available_width_px(1).unwrap_or(f64::INFINITY),
                        )
                        .map(|flow| flow.extra_rows)
                        .unwrap_or(0);
                    (
                        lh + extra_rows as f64 * (lh + line_spacing_px),
                        line_spacing_px,
                    )
                })
                .collect();
            if pairs.is_empty() {
                if let Some(metric) =
                    empty_paragraph_fallback_line_metrics(para, styles, para_style)
                {
                    pairs.push(metric);
                }
            }
            // [#2287] 저장 LINE_SEG 없는 빈 anchor 문단의 TAC 그림/도형 —
            // typeset format_paragraph 와 동일한 줄 메트릭 합성 (측정 정합).
            if pairs.is_empty() {
                if let Some(metrics) = crate::renderer::tac_object_stack_line_metrics(
                    para,
                    self.dpi,
                    column_width_px.map(|cw| {
                        let margin_l = para_style.map(|s| s.margin_left).unwrap_or(0.0);
                        let margin_r = para_style.map(|s| s.margin_right).unwrap_or(0.0);
                        (cw - margin_l - margin_r).max(0.0)
                    }),
                ) {
                    pairs.extend(metrics);
                }
            }
            pairs.into_iter().unzip()
        } else if !para.line_segs.is_empty() {
            // 누름틀(ClickHere) 안내문이 LINE_SEG에 포함되면 줄 수가 실제보다 많음
            // 안내문 텍스트가 차지하는 줄을 제외하여 실제 렌더링 높이를 계산
            let guide_char_count: usize = para
                .controls
                .iter()
                .filter_map(|c| {
                    if let Control::Field(f) = c {
                        f.guide_text().map(|t| t.encode_utf16().count())
                    } else {
                        None
                    }
                })
                .sum();
            if guide_char_count > 0 && para.line_segs.len() >= 2 {
                // 안내문이 차지하는 LINE_SEG 수:
                // 제어문자(필드 시작/끝 약 8 code units) + 안내문 길이까지의 text_start
                let guide_end = guide_char_count + 10; // 제어문자 + 안내문 + 여유
                let skip = para
                    .line_segs
                    .iter()
                    .position(|seg| (seg.text_start as usize) >= guide_end)
                    .unwrap_or(0);
                para.line_segs
                    .iter()
                    .skip(skip)
                    .map(|seg| {
                        (
                            hwpunit_to_px(seg.line_height, self.dpi),
                            hwpunit_to_px(seg.line_spacing, self.dpi),
                        )
                    })
                    .unzip()
            } else {
                para.line_segs
                    .iter()
                    .map(|seg| {
                        (
                            hwpunit_to_px(seg.line_height, self.dpi),
                            hwpunit_to_px(seg.line_spacing, self.dpi),
                        )
                    })
                    .unzip()
            }
        } else if let Some((lh, ls)) =
            empty_paragraph_fallback_line_metrics(para, styles, para_style)
        {
            (vec![lh], vec![ls])
        } else {
            // 빈 문단: 기본 높이
            (vec![hwpunit_to_px(400, self.dpi)], vec![0.0])
        };

        let lines_total: f64 = {
            let sum: f64 = line_heights
                .iter()
                .zip(line_spacings.iter())
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
                            - first.vertical_pos,
                        self.dpi,
                    );
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
        let clickhere_adjustment: f64 = para
            .controls
            .iter()
            .filter_map(|c| {
                if let Control::Field(f) = c {
                    if let Some(guide) = f.guide_text() {
                        let guide_u16_len = guide.encode_utf16().count();
                        if guide_u16_len > 0 && para.line_segs.len() >= 2 {
                            // 안내문이 차지하는 LINE_SEG 수 계산
                            let guide_end = guide_u16_len + 10; // 제어문자 여유
                            let guide_segs = para
                                .line_segs
                                .iter()
                                .position(|seg| (seg.text_start as usize) >= guide_end)
                                .unwrap_or(0);
                            if guide_segs > 0 {
                                let adj: f64 = para.line_segs[..guide_segs]
                                    .iter()
                                    .map(|seg| {
                                        hwpunit_to_px(seg.line_height + seg.line_spacing, self.dpi)
                                    })
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
        let total_height =
            (spacing_before + lines_total + spacing_after - clickhere_adjustment).max(0.0);

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
        let mut total = 0.0;
        for para in paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Picture(pic) => {
                        total += self.non_inline_control_flow_height(&pic.common);
                    }
                    Control::Shape(shape) => {
                        total += self.non_inline_control_flow_height(shape.common());
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
        self.measure_table_impl(table, para_index, control_index, styles, 0, 1.0)
    }

    /// 재귀적 높이 제한
    const MAX_NESTED_DEPTH: usize = 10;

    /// 셀 내 중첩 표들의 총 높이를 계산한다.
    pub fn cell_controls_height(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        depth: usize,
        // [#2195] 부모 셀 전폭(px, 스트레치 기준). 0.0 = 미적용.
        parent_cell_w: f64,
    ) -> f64 {
        if depth >= Self::MAX_NESTED_DEPTH {
            return 0.0;
        }
        paragraphs
            .iter()
            .map(|p| {
                p.controls
                    .iter()
                    .filter_map(|ctrl| {
                        if let Control::Table(nested) = ctrl {
                            let nested_w = hwpunit_to_px(nested.common.width as i32, self.dpi);
                            let stretch = 1.0; // [#2195] 스트레치는 render_normalized 로 일원화
                            let mt =
                                self.measure_table_impl(nested, 0, 0, styles, depth + 1, stretch);
                            Some(mt.total_height)
                        } else {
                            None
                        }
                    })
                    .sum::<f64>()
            })
            .sum()
    }

    /// 셀 내 중첩 표가 실제로 차지하는 하단 위치를 계산한다.
    ///
    /// 중첩 표가 있는 문단의 LINE_SEG.line_height는 표의 실제 높이를 담지 못하는
    /// 문서가 있다. 이 경우 문단의 vertical_pos를 기준으로 중첩 표의 재귀 측정
    /// 높이를 더해 셀 콘텐츠의 실제 끝점을 구한다.
    fn cell_nested_controls_bottom(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        depth: usize,
        // [#2195] 부모 셀 전폭(px, 스트레치 기준). 0.0 = 미적용.
        parent_cell_w: f64,
    ) -> f64 {
        if depth >= Self::MAX_NESTED_DEPTH {
            return 0.0;
        }
        paragraphs
            .iter()
            .map(|p| {
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .filter_map(|ctrl| {
                        if let Control::Table(nested) = ctrl {
                            let nested_w = hwpunit_to_px(nested.common.width as i32, self.dpi);
                            let stretch = 1.0; // [#2195] 스트레치는 render_normalized 로 일원화
                            let mt =
                                self.measure_table_impl(nested, 0, 0, styles, depth + 1, stretch);
                            // [#2148 실험] NO_LS 중첩 표 선언 신뢰 — 성장 전용 max.
                            let declared = hwpunit_to_px(nested.common.height as i32, self.dpi);
                            // [#2169] om 가산은 additive 경로(cell_controls_height)
                            // 전담 — vpos 기반 max 경로는 저장 vpos 가 배치를 이미
                            // 반영하므로 미가산 (자기-export HWPX 왕복 이중가산 방지).
                            Some(mt.total_height.max(declared))
                        } else {
                            None
                        }
                    })
                    .sum();
                if nested_h <= 0.0 {
                    0.0
                } else {
                    let para_top = p
                        .line_segs
                        .first()
                        .map(|s| hwpunit_to_px(s.vertical_pos, self.dpi))
                        .unwrap_or(0.0);
                    para_top + nested_h
                }
            })
            .fold(0.0f64, f64::max)
    }

    /// 표의 높이를 측정한다 (depth 기반 재귀).
    fn measure_table_impl(
        &self,
        table: &Table,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
        depth: usize,
        // [#2195] 중첩 표 스트레치 배율: 부모 셀 inner / 중첩 표 저장 폭 (>=1.0).
        // 한글은 내부 표를 부모 셀 inner 폭으로 확장 배치한다 (76076 표325 r6:
        // 저장 487.6px vs 실효 ~506px, 근거설명 셀 41줄 오라클 + 휴먼명조 사다리).
        width_scale: f64,
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
        // 1x1 래퍼 표 감지: 내부 표의 높이를 직접 측정.
        // (Task #688) 셀 paragraphs 가 2개 이상이면 첫 nested 표만 unwrap 시 나머지
        // paragraph 의 nested 표가 누락되므로 paragraphs.len() == 1 가드를 둔다.
        // controls.len() == 1 가드는 두지 않는다 — table_layout 분기와 일관성을 위해
        // 정렬 마커 등 다른 control 이 동거하는 케이스에서도 첫 nested table 만 추출한다.
        if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
            let cell = &table.cells[0];
            if cell.paragraphs.len() == 1 {
                let p = &cell.paragraphs[0];
                let has_visible_text = p
                    .text
                    .chars()
                    .any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n');
                if !has_visible_text {
                    if let Some(nested) = p.controls.iter().find_map(|c| {
                        if let Control::Table(t) = c {
                            Some(t.as_ref())
                        } else {
                            None
                        }
                    }) {
                        return self.measure_table_impl(
                            nested,
                            para_index,
                            control_index,
                            styles,
                            depth + 1,
                            width_scale,
                        );
                    }
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
                // [Task #1785] 셀 패딩 — aim=false 는 layout 의 레거시 보존값 규칙
                // (Cell::effective_padding)과 통일: 단순 table.padding 폴백은 cell > table
                // 보존값 케이스에서 layout 렌더와 어긋나 표 높이가 틀어진다 (36381023
                // micro-grid 결재란 라운드트립 9.3px). aim=true 는 기존대로 cell.padding
                // 을 0 포함 그대로 존중 — layout 의 `!= 0 → 표 기본 폴백`과 다르지만,
                // #493 세로 Shift 리사이즈(셀보호2.hwp 셀[20] aim=true pad top/bottom=0)가
                // 이 의미에 의존한다 (#1809 완전 통일 시도는 해당 테스트 회귀로 원복 —
                // vertical_shift_local_height_keeps_unrelated_cells_stable).
                let eff_pad = if cell.apply_inner_margin {
                    cell.padding
                } else {
                    cell.effective_padding(&table.padding)
                };
                let pad_top = hwpunit_to_px(eff_pad.top as i32, self.dpi);
                let pad_bottom = hwpunit_to_px(eff_pad.bottom as i32, self.dpi);
                // [Task #671] 좌우 패딩 — recompose_for_cell_width 의 inner_width 계산용
                let pad_left = hwpunit_to_px(eff_pad.left as i32, self.dpi);
                let pad_right = hwpunit_to_px(eff_pad.right as i32, self.dpi);
                let cell_w_px = if cell.width < 0x80000000 {
                    hwpunit_to_px(cell.width as i32, self.dpi) * width_scale
                } else {
                    0.0
                };
                // [#2279 axis B 보류] 측정 shrink 폭은 80168 r7(한글 8줄) 회귀로 보류
                // — table_layout::cell_units_uncached 의 [#2279 axis B 보류] 참조.
                let cell_inner_width = (cell_w_px - pad_left - pad_right).max(0.0);

                // 셀 내 문단들의 실제 높이 합산
                let text_height: f64 = if cell.text_direction != 0 {
                    // 세로쓰기: line_seg.segment_width가 열의 세로 길이
                    // 셀 높이 = 최대 segment_width
                    let mut max_h: f64 = 0.0;
                    for p in &cell.paragraphs {
                        for ls in &p.line_segs {
                            let h = hwpunit_to_px(ls.segment_width, self.dpi);
                            if h > max_h {
                                max_h = h;
                            }
                        }
                    }
                    if max_h <= 0.0 {
                        hwpunit_to_px(400, self.dpi)
                    } else {
                        max_h
                    }
                } else {
                    // 가로쓰기: spacing + line_height + line_spacing 합산
                    let cell_para_count = cell.paragraphs.len();
                    cell.paragraphs
                        .iter()
                        .enumerate()
                        .map(|(pidx, p)| {
                            let mut comp = compose_paragraph(p);
                            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine
                            // 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할.
                            // 측정/렌더링 일관성 (layout 의 recompose_for_cell_width 호출과 동일).
                            crate::renderer::composer::recompose_for_cell_width(
                                &mut comp,
                                p,
                                cell_inner_width,
                                styles,
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
                                // [#2169] NO_LS 순수 빈 문단 = em 줄박스 (한글 공식).
                                let h = if crate::renderer::para_has_no_stored_line_segs(p)
                                    && p.controls.is_empty()
                                {
                                    let fs = p
                                        .char_shapes
                                        .first()
                                        .and_then(|cs| {
                                            styles.char_styles.get(cs.char_shape_id as usize)
                                        })
                                        .map(|cs| cs.font_size)
                                        .unwrap_or(0.0);
                                    if fs <= 0.0 {
                                        hwpunit_to_px(400, self.dpi)
                                    } else if is_last_para {
                                        fs
                                    } else {
                                        match para_style {
                                            Some(ps) => crate::renderer::corrected_line_height(
                                                hwpunit_to_px(400, self.dpi),
                                                fs,
                                                ps.line_spacing_type,
                                                ps.line_spacing,
                                            ),
                                            None => fs,
                                        }
                                    }
                                } else if crate::renderer::para_has_no_stored_line_segs(p)
                                    && !p.controls.is_empty()
                                    && p.controls
                                        .iter()
                                        .all(|c| matches!(c, Control::Table(_)))
                                {
                                    // [#2169] TAC 중첩 표 anchor 빈 문단 몫 = 0
                                    // (anchor 사다리: row = nested+pad 정확).
                                    // 중첩 몫은 cell_controls_height 가산이 전담.
                                    0.0
                                } else if crate::renderer::para_has_no_stored_line_segs(p)
                                    && p.controls
                                        .iter()
                                        .any(|c| matches!(c, Control::Table(_)))
                                {
                                    // [#2195] 표+타 컨트롤(누름틀 필드 등) 동반 anchor 빈
                                    // 문단은 자체 줄박스 계상 - 86712 근거설명 괘선 회계:
                                    // 호스트(15pt ls120) = 24px 가 자연 행높이 성분.
                                    let fs = p
                                        .char_shapes
                                        .first()
                                        .and_then(|cs| {
                                            styles.char_styles.get(cs.char_shape_id as usize)
                                        })
                                        .map(|cs| cs.font_size)
                                        .unwrap_or(0.0);
                                    if fs <= 0.0 {
                                        hwpunit_to_px(400, self.dpi)
                                    } else {
                                        match para_style {
                                            Some(ps) => crate::renderer::corrected_line_height(
                                                hwpunit_to_px(400, self.dpi),
                                                fs,
                                                ps.line_spacing_type,
                                                ps.line_spacing,
                                            ),
                                            None => fs,
                                        }
                                    }
                                } else {
                                    hwpunit_to_px(400, self.dpi)
                                };
                                spacing_before + h + spacing_after
                            } else {
                                let cell_ls_val =
                                    para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                                let cell_ls_type = para_style
                                    .map(|s| s.line_spacing_type)
                                    .unwrap_or(crate::model::style::LineSpacingType::Percent);
                                // [Issue #1842] 저장 LINE_SEG 부재 셀 문단은 composer 가
                                // placeholder(line_height=400) 로 합성 → corrected 가
                                // max_fs*ls% 로 팽창(단행 박스에 줄간격 오적용). 한글은 폰트
                                // em 으로 렌더 → synthetic 시 em(max_fs). hwp3 synthetic 선례 확장.
                                let synthetic_line = p.line_segs.is_empty()
                                    && !p.text.is_empty()
                                    && matches!(table.page_break, TablePageBreak::CellBreak);
                                let line_count = comp.lines.len();
                                // [#2070 stage10] 저장 LINE_SEG vpos 리셋 줄(원저작
                                // 분할 흔적)도 전량 계상 — 한글 원본 오라클(개정안{{0}}
                                // 마크 워크 28줄 = stored 재현, row 918.5px = 콘텐츠 +
                                // 조각별 패딩)이 재현을 확증. stage4의 리셋 줄 제외는
                                // 픽스처(intent 절반 버그로 재계산) 산물에 맞춘 오판.
                                let lines_total: f64 = comp
                                    .lines
                                    .iter()
                                    .enumerate()
                                    .map(|(i, line)| {
                                        let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                                        let max_fs = line
                                            .runs
                                            .iter()
                                            .map(|r| {
                                                styles
                                                    .char_styles
                                                    .get(r.char_style_id as usize)
                                                    .map(|cs| cs.font_size)
                                                    .unwrap_or(0.0)
                                            })
                                            .fold(0.0f64, f64::max);
                                        let is_cell_last_line =
                                            is_last_para && i + 1 == line_count;
                                        // [#2169] NO_LS 순수 빈 문단 — 문단 char shape fs
                                        // 폴백 (한글은 완전한 em 줄박스로 취급).
                                        let max_fs = if max_fs <= 0.0
                                            && crate::renderer::para_has_no_stored_line_segs(p)
                                            && p.controls.is_empty()
                                        {
                                            p.char_shapes
                                                .first()
                                                .and_then(|cs| {
                                                    styles
                                                        .char_styles
                                                        .get(cs.char_shape_id as usize)
                                                })
                                                .map(|cs| cs.font_size)
                                                .unwrap_or(0.0)
                                        } else {
                                            max_fs
                                        };
                                        // [#2112] 실저장 LINE_SEG(비합성, tag 0x80000000
                                        // 미설정) 보유 문단은 저장 줄높이 신뢰 — 한글은
                                        // 압축 줄높이(lh<글자크기)를 저장값대로 렌더한다.
                                        // table_layout.rs 컷 측정과 동일 원칙
                                        // (39607 +335px 팽창 소거).
                                        let h = if p
                                            .line_segs
                                            .iter()
                                            .any(|ls| ls.tag & 0x8000_0000 == 0)
                                        {
                                            raw_lh
                                        } else {
                                            crate::renderer::corrected_line_height_for_variant_synthetic(
                                                raw_lh,
                                                max_fs,
                                                cell_ls_type,
                                                cell_ls_val,
                                                // [#2070] NO_LS 단일 문단·단일 줄 셀
                                                // = em (fixed_ladder: 1줄 셀 줄간격 무시).
                                                synthetic_line
                                                    || (crate::renderer::para_has_no_stored_line_segs(p)
                                                        && is_cell_last_line),
                                            )
                                        };
                                        // [Task #874 #4 / #1086] CellBreak/TAC 표는 기존
                                        // trailing geometry 를 보존(aift.hwp pi=123, KTX TOC),
                                        // block RowBreak 표는 렌더 가시 높이처럼 셀 마지막 줄
                                        // trailing 을 제외(k-water-rfp pi=180).
                                        let is_block_rowbreak =
                                            matches!(table.page_break, TablePageBreak::RowBreak)
                                                && !table.common.treat_as_char;
                                        let include_trailing_ls =
                                            !is_cell_last_line || cell_para_count > 1;
                                        let include_trailing_ls = include_trailing_ls
                                            && (!is_cell_last_line || !is_block_rowbreak);
                                        if include_trailing_ls {
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
                let has_nested_table_in_cell = cell
                    .paragraphs
                    .iter()
                    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
                let cell_all_no_ls = cell
                    .paragraphs
                    .iter()
                    .all(crate::renderer::para_has_no_stored_line_segs);
                let content_height = if has_nested_table_in_cell && cell_all_no_ls {
                    // [#2148 실험] NO_LS 셀은 vpos 사다리가 없어 nested_bottom 의
                    // para_top(첫 lineseg vpos)=0 → 위 텍스트 문단이 소거된다
                    // (80168 pi=271 r6: 텍스트 2줄 + 중첩 99.7 → max 로 99.7 과소,
                    // 한글 160.2 는 합산 흐름). 텍스트 합 + 중첩 표 합(선언 max +
                    // outMargin — 이 additive 경로 한정, 한글 검산 160.1)으로 가산.
                    let nested_sum: f64 = cell
                        .paragraphs
                        .iter()
                        .flat_map(|p| p.controls.iter())
                        .filter_map(|ctrl| {
                            if let Control::Table(nested) = ctrl {
                                let nested_w = hwpunit_to_px(nested.common.width as i32, self.dpi);
                                // 한글 실효폭은 부모 셀 **전폭**(pad 미차감, 76076
                                // 근거설명 셀: 유효 ~504px = 부모 w 506.2, inner 492.6
                                // 으로는 L0 40자 수용 불가) 기준.
                                let stretch = 1.0; // [#2195] 스트레치는 render_normalized 로 일원화
                                let mt = self.measure_table_impl(
                                    nested,
                                    0,
                                    0,
                                    styles,
                                    depth + 1,
                                    stretch,
                                );
                                let declared = hwpunit_to_px(nested.common.height as i32, self.dpi);
                                let om = hwpunit_to_px(nested.outer_margin_top as i32, self.dpi)
                                    + hwpunit_to_px(nested.outer_margin_bottom as i32, self.dpi);
                                Some(mt.total_height.max(declared) + om)
                            } else {
                                None
                            }
                        })
                        .sum();
                    text_height + nested_sum
                } else if has_nested_table_in_cell {
                    // 마지막 문단의 마지막 LINE_SEG의 vpos + line_height
                    let last_seg_end: i32 = cell
                        .paragraphs
                        .iter()
                        .flat_map(|p| p.line_segs.last())
                        .map(|s| s.vertical_pos + s.line_height)
                        .max()
                        .unwrap_or(0);
                    let nested_bottom = self.cell_nested_controls_bottom(
                        &cell.paragraphs,
                        styles,
                        depth,
                        cell_w_px,
                    );
                    hwpunit_to_px(last_seg_end, self.dpi)
                        .max(text_height)
                        .max(nested_bottom)
                        .max(self.cell_wrap_objects_bottom_height(&cell.paragraphs))
                } else {
                    // 단, 비-인라인 이미지/도형은 LINE_SEG에 미포함이므로 별도 합산
                    let non_inline_h = self.measure_non_inline_controls_height(&cell.paragraphs);
                    let wrap_bottom = self.cell_wrap_objects_bottom_height(&cell.paragraphs);
                    // [Task #2226] 저장 LINE_SEG 흐름 extent 가 additive 합보다 작으면
                    // 저장 지오메트리 신뢰 — TopAndBottom flow 그림의 배치는 저장 vpos
                    // 사다리(줄이 그림 아래로 밀림)에 이미 반영되어 있어, 별도 합산은
                    // 이중 계상이다 (주보 p2 로고 표 셀: 그림 2개 합산 112.8px vs 저장
                    // extent 65.9px → 행 1.9× 팽창, 2행 텍스트 페이지 밖 소실).
                    // layout trust_stored_cell_flow 와 동일 원리·가드 (#2211 계보).
                    let stored_extent = if !cell.paragraphs.is_empty()
                        && cell
                            .paragraphs
                            .iter()
                            .all(|pp| !crate::renderer::para_has_no_stored_line_segs(pp))
                    {
                        cell.paragraphs
                            .iter()
                            .flat_map(|pp| pp.line_segs.iter())
                            .filter(|seg| seg.vertical_pos >= 0 && seg.line_height > 0)
                            .map(|seg| hwpunit_to_px(seg.vertical_pos + seg.line_height, self.dpi))
                            .fold(0.0f64, f64::max)
                    } else {
                        0.0
                    };
                    let additive = text_height + non_inline_h;
                    // trust 는 "저장 ladder 가 개체 밀림을 이미 반영한" 셀에만 —
                    // 증거: 텍스트-빈 문단의 첫 seg vpos > 0 (줄이 개체 아래로 밀림).
                    // 반례 캘리브: KTX TOC(개체 없음 — additive 가 한컴 쪽),
                    // #1282 쪽영역제한 ON(텍스트 문단 vpos 0, 저장 h < 그림 —
                    // 한컴이 행을 그림만큼 키움).
                    let ladder_absorbed_objects = cell.paragraphs.iter().any(|pp| {
                        pp.text.trim().is_empty()
                            && pp.line_segs.first().is_some_and(|seg| seg.vertical_pos > 0)
                            && pp.controls.iter().any(|c| {
                                matches!(c, Control::Picture(pic) if !pic.common.treat_as_char)
                                    || matches!(c, Control::Shape(sh) if !sh.common().treat_as_char)
                            })
                    });
                    let trust_stored = (depth > 0 || table.common.treat_as_char)
                        && non_inline_h > 0.0
                        && ladder_absorbed_objects
                        && stored_extent > 0.0
                        && stored_extent + 0.5 < additive
                        && wrap_bottom <= stored_extent + 0.5;
                    if trust_stored {
                        stored_extent
                    } else {
                        additive.max(wrap_bottom)
                    }
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
                // [Task #2221] layout resolve_row_heights 의 relaxed_pad 미러 —
                // 중첩(depth>0)/TAC 표에서 저장 LINE_SEG 보유 텍스트 셀의 줄 흐름은
                // pad 미가산 (#2211과 동일 규칙). layout 만 정정하고 측정을 남겨두면
                // 하단앵커 배치(측정 높이)와 렌더 행합이 어긋난다 (36389312 pi=6
                // 결재란: 측정 320.4 vs 렌더 316.7 = 3.73px 드리프트). 상위 분할/
                // 앵커 표(depth=0, 비-tac)는 기존 회계 유지 — #1748 컷 예산 캘리브
                // 비접촉. 상위 TAC 표는 렌더가 measured 행높이를 그대로 쓰므로
                // (mt 우선) 측정·렌더가 이미 일관 — tac 을 미러에 포함하면 실제
                // 지오메트리가 이동한다 (KTX/exam_kor/복학원서 golden). depth>0 만.
                let relaxed_pad_mirror = depth > 0
                    && cell.text_direction == 0
                    && !has_nested_table_in_cell
                    && !cell.paragraphs.is_empty()
                    && cell
                        .paragraphs
                        .iter()
                        .all(|p| !crate::renderer::para_has_no_stored_line_segs(p));
                let required_height = if cell_h_px > 0.0
                    && total_pad > cell_h_px * 0.5
                    && content_height <= cell_h_px
                {
                    cell_h_px
                } else if relaxed_pad_mirror {
                    let non_inline_h = self.measure_non_inline_controls_height(&cell.paragraphs);
                    let object_based =
                        non_inline_h.max(self.cell_wrap_objects_bottom_height(&cell.paragraphs));
                    let object_req = if object_based > 0.0 {
                        object_based + total_pad
                    } else {
                        0.0
                    };
                    text_height.max(object_req)
                } else {
                    content_height + total_pad
                };
                // [Task #1763] 한컴 선언 셀높이 권위 — 저장 cell.height 는 trailing ls
                // 미포함(825행 주석 원칙)인데, 다문단 셀 측정은 셀 마지막 줄 trailing ls
                // 를 포함(#874/#1086 보존 조건)해 required 가 선언높이를 초과 확장할 수
                // 있다(2501937 row0: 콘텐츠 10016HU + trailing 600HU + pad → 149.1px >
                // 선언 142.2px, 한글은 선언 유지). 초과분이 전적으로 trailing ls 때문이면
                // (trailing 제외 콘텐츠+pad 가 선언 안) 선언높이로 clamp — 콘텐츠가 진짜
                // 초과하는 기존 보존 케이스(aift/KTX)는 조건 미충족으로 불변.
                // RowBreak(행 단위 쪽나눔) 표는 TAC 여부와 무관하게 clamp 제외 —
                // 분할 배치가 trailing 포함 측정에 정합 (rowbreak-problem-pages p11~13).
                let cell_last_trailing_ls = if cell.text_direction == 0
                    && !has_nested_table_in_cell
                    && cell.paragraphs.len() > 1
                    && !matches!(table.page_break, TablePageBreak::RowBreak)
                {
                    cell.paragraphs
                        .last()
                        .map(|p| {
                            let mut comp = compose_paragraph(p);
                            crate::renderer::composer::recompose_for_cell_width(
                                &mut comp,
                                p,
                                cell_inner_width,
                                styles,
                            );
                            comp.lines
                                .last()
                                .map(|l| hwpunit_to_px(l.line_spacing, self.dpi))
                                .unwrap_or(0.0)
                        })
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                let required_height = if cell_h_px > 0.0
                    && required_height > cell_h_px
                    && cell_last_trailing_ls > 0.0
                    && content_height - cell_last_trailing_ls + total_pad <= cell_h_px
                {
                    cell_h_px
                } else {
                    required_height
                };
                // [#2146] 저장 LINE_SEG 이 전혀 없고 모든 문단이 1줄(폭 여유 포함)인
                // 라벨 셀(사선 헤더 등)은 재합성 초과가 순수 줄높이 인플레이션 —
                // 선언 셀높이 신뢰. (21761835 r0: 선언 3928HU=52.4px = 한글 실측,
                // 재합성 79.3px) 판정 기준은 composer::no_ls_short_label_cell 주석 참조.
                let required_height = if cell_h_px > 0.0
                    && cell.text_direction == 0
                    && !has_nested_table_in_cell
                    && crate::renderer::composer::no_ls_short_label_cell(
                        cell,
                        table,
                        cell_inner_width,
                        cell_h_px - pad_top - pad_bottom,
                        styles,
                    ) {
                    cell_h_px
                } else {
                    required_height
                };
                // [#2097 진단] 셀별 선언/측정/trailing 분해 — 동작 불변.
                if std::env::var("RHWP_DIAG_ROWH").is_ok() && depth == 0 {
                    let all_stored = !cell.paragraphs.is_empty()
                        && cell
                            .paragraphs
                            .iter()
                            .all(|p| !crate::renderer::para_has_no_stored_line_segs(p));
                    eprintln!(
                        "DIAG_ROWH r={} c={} decl={:.1} req={:.1} content={:.1} pad={:.1} trail={:.1} stored={} nested={}",
                        cell.row,
                        cell.col,
                        cell_h_px,
                        required_height,
                        content_height,
                        total_pad,
                        cell_last_trailing_ls,
                        all_stored,
                        has_nested_table_in_cell,
                    );
                }
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
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == r && x.1 == span)
                    {
                        if total_h > existing.2 {
                            existing.2 = total_h;
                        }
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
                    let unknown_rows: Vec<usize> =
                        (r..r + span).filter(|&i| row_heights[i] == 0.0).collect();
                    if unknown_rows.len() == 1 {
                        let remaining = (total_h - known_sum).max(0.0);
                        row_heights[unknown_rows[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress {
                    break;
                }
            }
            for &(r, span, total_h) in &constraints {
                let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                let unknown_rows: Vec<usize> =
                    (r..r + span).filter(|&i| row_heights[i] == 0.0).collect();
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
                // [#1809] aim 직접 분기 → 단일 출처(Cell::effective_padding) 통일.
                // aim=true 인데 cell padding 이 0 인 셀은 표 기본으로 폴백해야
                // 레이아웃(resolve_cell_padding)과 정합한다. 직접 분기가 남으면
                // HWPX→HWP 변환의 micro-grid 계약(aim 일괄 세트)만으로 병합 셀
                // 행높이 측정이 갈린다 (admrul_0296 행 32.37→31.60, 표 3.87px).
                let eff_pad = cell.effective_padding(&table.padding);
                let (pad_top, pad_bottom) = (
                    hwpunit_to_px(eff_pad.top as i32, self.dpi),
                    hwpunit_to_px(eff_pad.bottom as i32, self.dpi),
                );
                // [Task #671] 좌우 패딩 (recompose_for_cell_width inner_width 계산용)
                let (pad_left, pad_right) = (
                    hwpunit_to_px(eff_pad.left as i32, self.dpi),
                    hwpunit_to_px(eff_pad.right as i32, self.dpi),
                );
                let cell_w_px = if cell.width < 0x80000000 {
                    hwpunit_to_px(cell.width as i32, self.dpi) * width_scale
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
                            if h > max_h {
                                max_h = h;
                            }
                        }
                    }
                    if max_h <= 0.0 {
                        hwpunit_to_px(400, self.dpi)
                    } else {
                        max_h
                    }
                } else {
                    let cell_para_count = cell.paragraphs.len();
                    cell.paragraphs
                        .iter()
                        .enumerate()
                        .map(|(pidx, p)| {
                            let mut comp = compose_paragraph(p);
                            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine
                            // 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할.
                            crate::renderer::composer::recompose_for_cell_width(
                                &mut comp,
                                p,
                                cell_inner_width,
                                styles,
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
                                // [#2169] NO_LS 순수 빈 문단 = em 줄박스 (한글 공식).
                                let h = if crate::renderer::para_has_no_stored_line_segs(p)
                                    && p.controls.is_empty()
                                {
                                    let fs = p
                                        .char_shapes
                                        .first()
                                        .and_then(|cs| {
                                            styles.char_styles.get(cs.char_shape_id as usize)
                                        })
                                        .map(|cs| cs.font_size)
                                        .unwrap_or(0.0);
                                    if fs <= 0.0 {
                                        hwpunit_to_px(400, self.dpi)
                                    } else if is_last_para {
                                        fs
                                    } else {
                                        match para_style {
                                            Some(ps) => crate::renderer::corrected_line_height(
                                                hwpunit_to_px(400, self.dpi),
                                                fs,
                                                ps.line_spacing_type,
                                                ps.line_spacing,
                                            ),
                                            None => fs,
                                        }
                                    }
                                } else if crate::renderer::para_has_no_stored_line_segs(p)
                                    && !p.controls.is_empty()
                                    && p.controls
                                        .iter()
                                        .all(|c| matches!(c, Control::Table(_)))
                                {
                                    // [#2169] TAC 중첩 표 anchor 빈 문단 몫 = 0
                                    // (anchor 사다리: row = nested+pad 정확).
                                    // 중첩 몫은 cell_controls_height 가산이 전담.
                                    0.0
                                } else if crate::renderer::para_has_no_stored_line_segs(p)
                                    && p.controls
                                        .iter()
                                        .any(|c| matches!(c, Control::Table(_)))
                                {
                                    // [#2195] 표+타 컨트롤(누름틀 필드 등) 동반 anchor 빈
                                    // 문단은 자체 줄박스 계상 - 86712 근거설명 괘선 회계:
                                    // 호스트(15pt ls120) = 24px 가 자연 행높이 성분.
                                    let fs = p
                                        .char_shapes
                                        .first()
                                        .and_then(|cs| {
                                            styles.char_styles.get(cs.char_shape_id as usize)
                                        })
                                        .map(|cs| cs.font_size)
                                        .unwrap_or(0.0);
                                    if fs <= 0.0 {
                                        hwpunit_to_px(400, self.dpi)
                                    } else {
                                        match para_style {
                                            Some(ps) => crate::renderer::corrected_line_height(
                                                hwpunit_to_px(400, self.dpi),
                                                fs,
                                                ps.line_spacing_type,
                                                ps.line_spacing,
                                            ),
                                            None => fs,
                                        }
                                    }
                                } else {
                                    hwpunit_to_px(400, self.dpi)
                                };
                                spacing_before + h + spacing_after
                            } else {
                                let cell_ls_val =
                                    para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                                let cell_ls_type = para_style
                                    .map(|s| s.line_spacing_type)
                                    .unwrap_or(crate::model::style::LineSpacingType::Percent);
                                // [Issue #1842] 저장 LINE_SEG 부재 셀 문단은 composer 가
                                // placeholder(line_height=400) 로 합성 → corrected 가
                                // max_fs*ls% 로 팽창(단행 박스에 줄간격 오적용). 한글은 폰트
                                // em 으로 렌더 → synthetic 시 em(max_fs). hwp3 synthetic 선례 확장.
                                let synthetic_line = p.line_segs.is_empty()
                                    && !p.text.is_empty()
                                    && matches!(table.page_break, TablePageBreak::CellBreak);
                                let line_count = comp.lines.len();
                                // [#2070 stage10] 저장 LINE_SEG vpos 리셋 줄(원저작
                                // 분할 흔적)도 전량 계상 — 한글 원본 오라클(개정안{{0}}
                                // 마크 워크 28줄 = stored 재현, row 918.5px = 콘텐츠 +
                                // 조각별 패딩)이 재현을 확증. stage4의 리셋 줄 제외는
                                // 픽스처(intent 절반 버그로 재계산) 산물에 맞춘 오판.
                                let lines_total: f64 = comp
                                    .lines
                                    .iter()
                                    .enumerate()
                                    .map(|(i, line)| {
                                        let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                                        let max_fs = line
                                            .runs
                                            .iter()
                                            .map(|r| {
                                                styles
                                                    .char_styles
                                                    .get(r.char_style_id as usize)
                                                    .map(|cs| cs.font_size)
                                                    .unwrap_or(0.0)
                                            })
                                            .fold(0.0f64, f64::max);
                                        let is_cell_last_line =
                                            is_last_para && i + 1 == line_count;
                                        // [#2169] NO_LS 순수 빈 문단 — 문단 char shape fs
                                        // 폴백 (한글은 완전한 em 줄박스로 취급).
                                        let max_fs = if max_fs <= 0.0
                                            && crate::renderer::para_has_no_stored_line_segs(p)
                                            && p.controls.is_empty()
                                        {
                                            p.char_shapes
                                                .first()
                                                .and_then(|cs| {
                                                    styles
                                                        .char_styles
                                                        .get(cs.char_shape_id as usize)
                                                })
                                                .map(|cs| cs.font_size)
                                                .unwrap_or(0.0)
                                        } else {
                                            max_fs
                                        };
                                        // [#2112] 실저장 LINE_SEG(비합성, tag 0x80000000
                                        // 미설정) 보유 문단은 저장 줄높이 신뢰 — 한글은
                                        // 압축 줄높이(lh<글자크기)를 저장값대로 렌더한다.
                                        // table_layout.rs 컷 측정과 동일 원칙
                                        // (39607 +335px 팽창 소거).
                                        let h = if p
                                            .line_segs
                                            .iter()
                                            .any(|ls| ls.tag & 0x8000_0000 == 0)
                                        {
                                            raw_lh
                                        } else {
                                            crate::renderer::corrected_line_height_for_variant_synthetic(
                                                raw_lh,
                                                max_fs,
                                                cell_ls_type,
                                                cell_ls_val,
                                                // [#2070] NO_LS 단일 문단·단일 줄 셀
                                                // = em (fixed_ladder: 1줄 셀 줄간격 무시).
                                                synthetic_line
                                                    || (crate::renderer::para_has_no_stored_line_segs(p)
                                                        && is_cell_last_line),
                                            )
                                        };
                                        // [Task #874 #4 / #1086] CellBreak/TAC 표는 기존
                                        // trailing geometry 를 보존(aift.hwp pi=123, KTX TOC),
                                        // block RowBreak 표는 렌더 가시 높이처럼 셀 마지막 줄
                                        // trailing 을 제외(k-water-rfp pi=180).
                                        let is_block_rowbreak =
                                            matches!(table.page_break, TablePageBreak::RowBreak)
                                                && !table.common.treat_as_char;
                                        let include_trailing_ls =
                                            !is_cell_last_line || cell_para_count > 1;
                                        let include_trailing_ls = include_trailing_ls
                                            && (!is_cell_last_line || !is_block_rowbreak);
                                        if include_trailing_ls {
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
                let nested_bottom =
                    self.cell_nested_controls_bottom(&cell.paragraphs, styles, depth, cell_w_px);
                let wrap_bottom = self.cell_wrap_objects_bottom_height(&cell.paragraphs);
                // [Task #2221] 단일행과 동일 — 중첩/TAC 표의 저장 LINE_SEG 텍스트
                // 셀은 pad 미가산 (layout 2-b relaxed_pad 미러).
                let relaxed_pad_mirror = depth > 0
                    && cell.text_direction == 0
                    && !cell.paragraphs.is_empty()
                    && cell
                        .paragraphs
                        .iter()
                        .all(|p| !crate::renderer::para_has_no_stored_line_segs(p));
                let required_height = if relaxed_pad_mirror {
                    let object_based = non_inline_h.max(nested_bottom).max(wrap_bottom);
                    let object_req = if object_based > 0.0 {
                        object_based + pad_top + pad_bottom
                    } else {
                        0.0
                    };
                    text_height.max(object_req)
                } else {
                    let content_height = (text_height + non_inline_h)
                        .max(nested_bottom)
                        .max(wrap_bottom);
                    content_height + pad_top + pad_bottom
                };
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
        let raw_table_height: f64 =
            row_heights.iter().sum::<f64>() + cell_spacing * (row_count.saturating_sub(1) as f64);
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
        // [Issue #1835] 내용이 저장 높이를 크게(>1.5×) 초과하는 TAC 표는 비례 축소하지
        // 않는다 — 한글 2022 편집기 오라클(issue1835 fixture: common.height 를 1/1.8 로
        // 훼손한 4×3 표를 내용 높이로 확장, 후속 문단도 그만큼 아래로 흐름) 기준.
        // 외부 도구 생성/템플릿 값 채움으로 common.height 가 stale 한 문서에서 행이
        // 1/1.8 로 눌려 셀 텍스트가 겹치던 결함. 경미한 초과(2%~150%)는 종전대로
        // 속성 높이 유지(#672 한컴 정합 — 의도적 압축 존중).
        const TAC_SHRINK_MAX_OVERFLOW_RATIO: f64 = 1.5;
        let shrink_threshold = (common_h * TAC_SHRINK_THRESHOLD_RATIO).max(1.0);
        let table_height = if table.common.treat_as_char
            && common_h > 0.0
            && raw_table_height > common_h + shrink_threshold
            && raw_table_height <= common_h * TAC_SHRINK_MAX_OVERFLOW_RATIO
        {
            let scale = common_h / raw_table_height;
            for h in &mut row_heights {
                *h *= scale;
            }
            common_h
        } else if !table.common.treat_as_char
            && common_h > 0.0
            && raw_table_height > 0.0
            && common_h > raw_table_height + 0.5
            && {
                // stale-min 셀 판별: 셀 선언높이(cellSz) 합이 표 선언높이의 절반
                // 미만인 생성계 문서에서만 발동 (정상 저장 문서는 Σ셀선언 ≈ 표선언
                // 이라 무해 — 전역 발동 시 콘텐츠<선언 표가 광역 팽창, 163쪽 회귀).
                let mut per_row = vec![0.0f64; row_count];
                for cell in &table.cells {
                    if cell.row_span == 1
                        && (cell.row as usize) < row_count
                        && cell.height < 0x80000000
                    {
                        let h = hwpunit_to_px(cell.height as i32, self.dpi);
                        if h > per_row[cell.row as usize] {
                            per_row[cell.row as usize] = h;
                        }
                    }
                }
                let declared_rows_sum: f64 = per_row.iter().sum::<f64>()
                    + cell_spacing * (row_count.saturating_sub(1) as f64);
                // [#2195] stale-min(x0.5) 한정을 일반 발동으로 완화 — 한글은 콘텐츠가
                // 선언보다 작아도 표 선언높이를 유지한다 (80168 pi=419). #2070 당시 전면
                // 발동의 163쪽 폭발은 타 축 미정합 상태의 결과.
                declared_rows_sum < common_h * 0.5 || raw_table_height + 0.5 < common_h
            }
        {
            // [#2070] 비-TAC 표는 선언 표높이(size.height)가 최소 높이다 — 한글은
            // max(선언, 콘텐츠)로 배치한다 (80168 pi=354/357/362 조문 표 오라클:
            // 콘텐츠 154.2px 인 세 표를 각각 선언 211.8/192.6, 콘텐츠 212.4 로 렌더).
            // 셀 선언높이(cellSz=284HU)가 stale-min 인 생성계 문서에서 표 선언높이가
            // 권위. 부족분은 행 높이에 비례 배분한다 (1행 표는 정확).
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
            matches!(
                c.direction,
                CaptionDirection::Left | CaptionDirection::Right
            )
        });
        let caption_height = if is_lr_caption {
            0.0
        } else {
            self.measure_caption(&table.caption)
        };
        let caption_spacing = if is_lr_caption {
            0.0
        } else {
            table
                .caption
                .as_ref()
                .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
                .unwrap_or(0.0)
        };

        // 총 높이 = 표 높이 + 캡션 높이 + 캡션-표 간격
        let total_height = table_height
            + caption_height
            + if caption_height > 0.0 {
                caption_spacing
            } else {
                0.0
            };

        // 셀 단위 분할용 상세 측정 (모든 셀, row_span > 1 포함)
        let mut measured_cells = {
            table
                .cells
                .iter()
                .filter(|cell| (cell.row as usize) < row_count)
                .map(|cell| {
                    // [#1809] aim 직접 분기 → 단일 출처 통일 (위 2-c단계와 동일 근거)
                    let eff_pad = cell.effective_padding(&table.padding);
                    let pad_top = hwpunit_to_px(eff_pad.top as i32, self.dpi);
                    let pad_bottom = hwpunit_to_px(eff_pad.bottom as i32, self.dpi);

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
                            line_heights.push(
                                spacing_before + hwpunit_to_px(400, self.dpi) + spacing_after,
                            );
                            para_line_counts.push(1);
                        } else {
                            let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
                            let cell_ls_type = para_style
                                .map(|s| s.line_spacing_type)
                                .unwrap_or(crate::model::style::LineSpacingType::Percent);
                            // [Issue #1842] 부재 LINE_SEG 셀 → em(max_fs), max_fs*ls% 팽창 방지.
                            let synthetic_line = p.line_segs.is_empty()
                                && !p.text.is_empty()
                                && matches!(table.page_break, TablePageBreak::CellBreak);
                            let line_count = comp.lines.len();
                            for (li, line) in comp.lines.iter().enumerate() {
                                let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                                let max_fs = line
                                    .runs
                                    .iter()
                                    .map(|r| {
                                        styles
                                            .char_styles
                                            .get(r.char_style_id as usize)
                                            .map(|cs| cs.font_size)
                                            .unwrap_or(0.0)
                                    })
                                    .fold(0.0f64, f64::max);
                                // 셀의 마지막 줄(마지막 문단의 마지막 줄)은 ls 제외
                                let is_cell_last_line = is_last_para && li + 1 == line_count;
                                // [#2169] NO_LS 순수 빈 문단 — char shape fs 폴백.
                                let max_fs = if max_fs <= 0.0
                                    && crate::renderer::para_has_no_stored_line_segs(p)
                                    && p.controls.is_empty()
                                {
                                    p.char_shapes
                                        .first()
                                        .and_then(|cs| {
                                            styles.char_styles.get(cs.char_shape_id as usize)
                                        })
                                        .map(|cs| cs.font_size)
                                        .unwrap_or(0.0)
                                } else {
                                    max_fs
                                };
                                // [#2112] 실저장 LINE_SEG(비합성) 보유 문단은 저장 줄높이
                                // 신뢰 (위 999/1277 사이트와 동일 원칙).
                                // [#2150/#2169] NO_LS 셀 마지막 줄 = em (한글 공식).
                                let h = if p.line_segs.iter().any(|ls| ls.tag & 0x8000_0000 == 0) {
                                    raw_lh
                                } else {
                                    crate::renderer::corrected_line_height_for_variant_synthetic(
                                        raw_lh,
                                        max_fs,
                                        cell_ls_type,
                                        cell_ls_val,
                                        // [#2070] NO_LS 단일 문단·단일 줄 셀 = em.
                                        synthetic_line
                                            || (crate::renderer::para_has_no_stored_line_segs(p)
                                                && is_cell_last_line),
                                    )
                                };
                                let ls = hwpunit_to_px(line.line_spacing, self.dpi);
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
                    let has_nested_table = cell
                        .paragraphs
                        .iter()
                        .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));

                    // [Task #1073] cell_units 의 per-중첩행 분해 조건과 동일:
                    // 텍스트 없는 문단(line_segs 합성 줄 0) + 단일 중첩 표 + 2행 이상.
                    let nested_split_row_count = cell
                        .paragraphs
                        .iter()
                        .filter_map(|p| {
                            let tables: Vec<&crate::model::table::Table> = p
                                .controls
                                .iter()
                                .filter_map(|c| match c {
                                    Control::Table(t) => Some(t.as_ref()),
                                    _ => None,
                                })
                                .collect();
                            // 가시 텍스트 없는 문단의 단일 중첩 표만 분해 대상
                            if p.text.trim().is_empty()
                                && tables.len() == 1
                                && tables[0].row_count >= 2
                            {
                                Some(tables[0].row_count as usize)
                            } else {
                                None
                            }
                        })
                        .next()
                        .unwrap_or(0);

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
                        nested_split_row_count,
                    }
                })
                .collect::<Vec<_>>()
        };

        // 중첩 표 셀: 실제 중첩 표 높이를 재귀 측정하여 total_content_height 보정
        for mc in &mut measured_cells {
            if mc.has_nested_table {
                let cell = &table
                    .cells
                    .iter()
                    .find(|c| c.row as usize == mc.row && c.col as usize == mc.col)
                    .unwrap();
                let mc_cell_w = if cell.width < 0x80000000 {
                    hwpunit_to_px(cell.width as i32, self.dpi) * width_scale
                } else {
                    0.0
                };
                let nested_bottom =
                    self.cell_nested_controls_bottom(&cell.paragraphs, styles, depth, mc_cell_w);
                mc.total_content_height = nested_bottom.max(mc.total_content_height);
            }
        }
        for mc in &mut measured_cells {
            let Some(cell) = table
                .cells
                .iter()
                .find(|c| c.row as usize == mc.row && c.col as usize == mc.col)
            else {
                continue;
            };
            let wrap_bottom = self.cell_wrap_objects_bottom_height(&cell.paragraphs);
            mc.total_content_height = mc.total_content_height.max(wrap_bottom);
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
            has_header_cells: table
                .cells
                .iter()
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
        column_width_px: Option<f64>,
    ) -> MeasuredSection {
        let mut measured_paras = Vec::with_capacity(paragraphs.len());
        let mut measured_tables = Vec::new();

        for (para_idx, para) in paragraphs.iter().enumerate() {
            let comp = composed.get(para_idx);

            // 블록 표 컨트롤 감지 (일반 표 + treat_as_char 블록형)
            let seg_width_r = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let has_table = para.controls.iter().any(|c| {
                matches!(c, Control::Table(t) if !t.common.treat_as_char
                    || (t.common.treat_as_char && !is_tac_table_inline_in_para(t, seg_width_r, para)))
            });
            let has_picture = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Picture(_) | Control::Equation(_)));
            let picture_height = self.measure_pictures_in_paragraph(para);

            let measured = self.measure_paragraph(
                para,
                comp,
                styles,
                para_idx,
                has_table,
                has_picture,
                picture_height,
                column_width_px,
            );
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
        column_width_px: Option<f64>,
    ) -> MeasuredSection {
        let dirty_bits = match dirty_paras {
            Some(bits) => bits,
            None => {
                // 전체 dirty: 기존 incremental (표 수준만 캐싱) 폴백
                return self.measure_section_incremental(
                    paragraphs,
                    composed,
                    styles,
                    prev_measured,
                    column_width_px,
                );
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
                                if let Some(prev_t) =
                                    prev_measured.get_measured_table(para_idx, ctrl_idx)
                                {
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
            let has_table = para.controls.iter().any(|c| {
                matches!(c, Control::Table(t) if !t.common.treat_as_char
                    || (t.common.treat_as_char && !is_tac_table_inline_in_para(t, seg_width_r, para)))
            });
            let has_picture = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Picture(_) | Control::Equation(_)));
            let picture_height = self.measure_pictures_in_paragraph(para);

            let measured = self.measure_paragraph(
                para,
                comp,
                styles,
                para_idx,
                has_table,
                has_picture,
                picture_height,
                column_width_px,
            );
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
        self.cells
            .iter()
            .filter(|c| row >= c.row && row < c.row + c.row_span)
            .map(|c| {
                let padding = c.padding_top + c.padding_bottom;
                // row_span > 1 셀: 셀이 차지하는 모든 행의 높이 합을 사용
                let cell_row_h = if c.row_span > 1 {
                    let end = (c.row + c.row_span).min(self.row_heights.len());
                    let h: f64 = self.row_heights[c.row..end].iter().sum();
                    let cs_count = if end > c.row + 1 {
                        (end - c.row - 1) as f64
                    } else {
                        0.0
                    };
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
                    if offset_rem <= 0.0 {
                        break;
                    }
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
                c.line_heights[visible_start..]
                    .iter()
                    .sum::<f64>()
                    .min(capped)
            })
            .fold(0.0f64, f64::max)
    }

    /// 지정 행의 셀별 패딩(상+하) 최대값을 반환한다.
    pub fn max_padding_for_row(&self, row: usize) -> f64 {
        self.cells
            .iter()
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
        let cells_in_row: Vec<&MeasuredCell> = self
            .cells
            .iter()
            .filter(|c| c.row == row && c.row_span == 1)
            .collect();
        if cells_in_row.is_empty() {
            return false;
        }
        // [Task #1073] 다줄 셀 또는 per-중첩행 분해 가능한 중첩 표 셀(2행 이상)이면 분할 가능.
        cells_in_row
            .iter()
            .any(|c| c.line_heights.len() > 1 || c.nested_split_row_count > 1)
    }

    /// 지정 행에서 첫 번째 줄의 최소 높이를 반환한다 (인트라-로우 분할 가능 여부 판단용).
    /// content_offset이 있으면 해당 오프셋 이후의 첫 줄 높이를 계산한다.
    pub fn min_first_line_height_for_row(&self, row: usize, content_offset: f64) -> f64 {
        let mut min_h = f64::MAX;
        for c in self
            .cells
            .iter()
            .filter(|c| c.row == row && c.row_span == 1)
        {
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
        if min_h == f64::MAX {
            0.0
        } else {
            min_h
        }
    }

    /// O(log R) 분할점: cursor_row부터 avail 높이에 들어가는 행 수 반환 (end_row, exclusive).
    /// effective_first_row_h: 첫 행의 유효 높이 (content_offset 반영).
    /// 인트라-로우 분할은 미고려.
    pub fn find_break_row(
        &self,
        avail: f64,
        cursor_row: usize,
        effective_first_row_h: f64,
    ) -> usize {
        let row_count = self.row_heights.len();
        if cursor_row >= row_count {
            return cursor_row;
        }
        let cs = self.cell_spacing;
        let delta = self.row_heights[cursor_row] - effective_first_row_h;
        let adj_cs = if cursor_row > 0 { cs } else { 0.0 };
        let target = self.cumulative_heights[cursor_row] + avail + delta + adj_cs;
        let search_start = cursor_row + 1;
        if search_start > row_count {
            return cursor_row;
        }
        let pos =
            self.cumulative_heights[search_start..=row_count].partition_point(|&h| h <= target);
        (cursor_row + pos).min(row_count)
    }

    /// O(1) 행 범위 높이 조회 (cell_spacing 포함).
    /// start_row..end_row 범위의 높이 (첫 행 앞에는 cs 미포함).
    pub fn range_height(&self, start_row: usize, end_row: usize) -> f64 {
        if end_row <= start_row {
            return 0.0;
        }
        let diff = self.cumulative_heights[end_row] - self.cumulative_heights[start_row];
        if start_row > 0 {
            diff - self.cell_spacing
        } else {
            diff
        }
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
        let block_start = self
            .row_block_start
            .get(end_row)
            .copied()
            .unwrap_or(end_row);
        let block_end = self
            .row_block_end
            .get(end_row)
            .copied()
            .unwrap_or(end_row + 1);
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
        matches!(
            self.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
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
        if r0 >= row_count {
            continue;
        }
        let r1 = (r0 + rs).min(row_count);
        for r in r0..r1 {
            if start[r] > r0 {
                start[r] = r0;
            }
            if end[r] < r1 {
                end[r] = r1;
            }
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
                if start[r2] < new_s {
                    new_s = start[r2];
                }
                if end[r2] > new_e {
                    new_e = end[r2];
                }
            }
            if new_s != s || new_e != e {
                start[r] = new_s;
                end[r] = new_e;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // 3단계: 같은 블록 내 모든 행이 동일 (start, end) 가지도록 정규화
    let mut r = 0;
    while r < row_count {
        let s = start[r];
        let e = end[r];
        for r2 in s..e {
            start[r2] = s;
            end[r2] = e;
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
        self.tables
            .iter()
            .find(|t| t.para_index == para_index && t.control_index == control_index)
            .map(|t| t.total_height)
    }

    /// 문단 내 표의 측정 정보 전체를 조회한다.
    pub fn get_measured_table(
        &self,
        para_index: usize,
        control_index: usize,
    ) -> Option<&MeasuredTable> {
        self.tables
            .iter()
            .find(|t| t.para_index == para_index && t.control_index == control_index)
    }

    /// 문단 인덱스로 측정된 문단 정보 전체를 조회한다.
    pub fn get_measured_paragraph(&self, para_index: usize) -> Option<&MeasuredParagraph> {
        self.paragraphs.get(para_index)
    }

    /// 문단이 표를 포함하는지 확인한다.
    pub fn paragraph_has_table(&self, para_index: usize) -> bool {
        self.paragraphs
            .get(para_index)
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
            .map(|s| hwpunit_to_px(s.separator_above_margin_hu() as i32, self.dpi))
            .unwrap_or(8.0); // 약 0.6mm
        let separator_margin_bottom = footnote_shape
            .map(|s| hwpunit_to_px(s.separator_below_margin_hu() as i32, self.dpi))
            .unwrap_or(4.0); // 약 0.3mm
        let note_spacing = footnote_shape
            .map(|s| hwpunit_to_px(s.between_notes_margin_hu() as i32, self.dpi))
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
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::table::{Cell, Table};

    fn wide_tac_table(width: u32) -> Box<Table> {
        Box::new(Table {
            common: CommonObjAttr {
                treat_as_char: true,
                width,
                ..Default::default()
            },
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                row_span: 1,
                col_span: 1,
                width,
                ..Default::default()
            }],
            ..Default::default()
        })
    }

    #[test]
    fn wide_tac_table_stays_block_at_text_boundaries() {
        let leading = Paragraph {
            text: "abc".to_string(),
            char_offsets: vec![8, 9, 10],
            controls: vec![Control::Table(wide_tac_table(950))],
            ..Default::default()
        };
        let Control::Table(leading_table) = &leading.controls[0] else {
            unreachable!()
        };
        assert!(!is_tac_table_inline_in_para(leading_table, 1000, &leading));

        let trailing = Paragraph {
            text: "abc".to_string(),
            char_offsets: vec![0, 1, 2],
            controls: vec![Control::Table(wide_tac_table(950))],
            ..Default::default()
        };
        let Control::Table(trailing_table) = &trailing.controls[0] else {
            unreachable!()
        };
        assert!(!is_tac_table_inline_in_para(
            trailing_table,
            1000,
            &trailing
        ));
    }

    #[test]
    fn wide_tac_table_uses_unicode_middle_anchor_for_only_that_table() {
        let para = Paragraph {
            text: "A🎉B".to_string(),
            // A(1 UTF-16) + 🎉(2 UTF-16) + control gap(8) + B.
            char_offsets: vec![0, 1, 11],
            controls: vec![
                Control::Table(wide_tac_table(950)),
                Control::Table(wide_tac_table(950)),
            ],
            ..Default::default()
        };
        assert_eq!(para.control_text_positions(), [2, 3]);

        let Control::Table(middle_table) = &para.controls[0] else {
            unreachable!()
        };
        let Control::Table(trailing_table) = &para.controls[1] else {
            unreachable!()
        };
        assert!(is_tac_table_inline_in_para(middle_table, 1000, &para));
        assert!(!is_tac_table_inline_in_para(trailing_table, 1000, &para));
    }

    #[test]
    fn test_measure_empty_section() {
        let measurer = HeightMeasurer::with_default_dpi();
        let paragraphs: Vec<Paragraph> = Vec::new();
        let composed: Vec<ComposedParagraph> = Vec::new();
        let styles = ResolvedStyleSet::default();

        let result = measurer.measure_section(&paragraphs, &composed, &styles, None);
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

        let result = measurer.measure_section(&paragraphs, &composed, &styles, None);
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
                Cell {
                    row: 0,
                    col: 0,
                    row_span: 1,
                    col_span: 1,
                    height: 500,
                    width: 1000,
                    ..Default::default()
                },
                Cell {
                    row: 0,
                    col: 1,
                    row_span: 1,
                    col_span: 1,
                    height: 500,
                    width: 1000,
                    ..Default::default()
                },
                Cell {
                    row: 1,
                    col: 0,
                    row_span: 1,
                    col_span: 1,
                    height: 600,
                    width: 1000,
                    ..Default::default()
                },
                Cell {
                    row: 1,
                    col: 1,
                    row_span: 1,
                    col_span: 1,
                    height: 600,
                    width: 1000,
                    ..Default::default()
                },
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
                Cell {
                    row: 0,
                    col: 0,
                    row_span: 1,
                    col_span: 1,
                    height: 1000,
                    width: 5000,
                    ..Default::default()
                },
                Cell {
                    row: 1,
                    col: 0,
                    row_span: 1,
                    col_span: 1,
                    height: 2000,
                    width: 5000,
                    ..Default::default()
                },
                Cell {
                    row: 2,
                    col: 0,
                    row_span: 1,
                    col_span: 1,
                    height: 1500,
                    width: 5000,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let styles = ResolvedStyleSet::default();
        let mt = measurer.measure_table(&table, 0, 0, &styles);

        assert_eq!(mt.cumulative_heights.len(), 4); // row_count + 1
        assert_eq!(mt.cumulative_heights[0], 0.0);

        // cumulative_heights 마지막 값은 row_heights 합 + cs * (row_count - 1)
        let expected_total: f64 = mt.row_heights.iter().sum::<f64>() + mt.cell_spacing * 2.0;
        assert!(
            (mt.cumulative_heights[3] - expected_total).abs() < 0.001,
            "cumulative_heights[3]={} expected={}",
            mt.cumulative_heights[3],
            expected_total
        );
    }

    #[test]
    fn test_find_break_row_all_fit() {
        // 모든 행이 들어가는 경우
        let mt = MeasuredTable {
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0], // 0, 20, 20+30+5, 55+25+5
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
        };
        let end = mt.find_break_row(200.0, 0, 20.0); // 200px 충분
        assert_eq!(end, 3); // 전부 fit
    }

    #[test]
    fn test_find_break_row_partial() {
        // 일부만 들어가는 경우
        let mt = MeasuredTable {
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0, 40.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0, 130.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
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
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![50.0, 30.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 50.0, 85.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
        };
        let end = mt.find_break_row(30.0, 0, 50.0); // 30 < 50
        assert_eq!(end, 0); // 첫 행도 안 들어감
    }

    #[test]
    fn test_range_height() {
        let mt = MeasuredTable {
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
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
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![50.0, 30.0, 25.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 50.0, 85.0, 115.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
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
            para_index: 0,
            control_index: 0,
            total_height: 0.0,
            row_heights: vec![],
            caption_height: 0.0,
            cell_spacing: 0.0,
            cumulative_heights: vec![0.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
        };
        assert_eq!(mt.find_break_row(100.0, 0, 0.0), 0);
        assert_eq!(mt.range_height(0, 0), 0.0);
    }

    #[test]
    fn test_find_break_row_single_row() {
        let mt = MeasuredTable {
            para_index: 0,
            control_index: 0,
            total_height: 50.0,
            row_heights: vec![50.0],
            caption_height: 0.0,
            cell_spacing: 0.0,
            cumulative_heights: vec![0.0, 50.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
        };
        assert_eq!(mt.find_break_row(100.0, 0, 50.0), 1); // fit
        assert_eq!(mt.find_break_row(30.0, 0, 50.0), 0); // doesn't fit
    }

    // ─────────────────────────────────────────────────────────────────────
    // Task #398: rowspan 묶음 블록 테스트
    // ─────────────────────────────────────────────────────────────────────

    fn make_table_with_cells(
        row_count: u16,
        col_count: u16,
        cells: Vec<crate::model::table::Cell>,
    ) -> crate::model::table::Table {
        crate::model::table::Table {
            row_count,
            col_count,
            cells,
            ..Default::default()
        }
    }

    fn cell_rs(row: u16, col: u16, row_span: u16) -> crate::model::table::Cell {
        crate::model::table::Cell {
            row,
            col,
            row_span,
            col_span: 1,
            ..Default::default()
        }
    }

    #[test]
    fn test_compute_row_blocks_all_single() {
        // 모든 셀 rowspan=1 → 각 행이 자기 자신만 포함하는 블록
        let table = make_table_with_cells(
            3,
            2,
            vec![
                cell_rs(0, 0, 1),
                cell_rs(0, 1, 1),
                cell_rs(1, 0, 1),
                cell_rs(1, 1, 1),
                cell_rs(2, 0, 1),
                cell_rs(2, 1, 1),
            ],
        );
        let (s, e) = compute_row_blocks(&table, 3);
        assert_eq!(s, vec![0, 1, 2]);
        assert_eq!(e, vec![1, 2, 3]);
    }

    #[test]
    fn test_compute_row_blocks_rs2_at_row0() {
        // 행 0에 rs=2 셀 → 블록 0~2
        let table = make_table_with_cells(
            3,
            2,
            vec![
                cell_rs(0, 0, 1),
                cell_rs(0, 1, 2), // rs=2
                cell_rs(1, 0, 1),
                cell_rs(2, 0, 1),
                cell_rs(2, 1, 1),
            ],
        );
        let (s, e) = compute_row_blocks(&table, 3);
        assert_eq!(s, vec![0, 0, 2]);
        assert_eq!(e, vec![2, 2, 3]);
    }

    #[test]
    fn test_compute_row_blocks_overlapping() {
        // 셀 A: rows 0~2, 셀 B: rows 1~3 → 통합 블록 0~3
        let table = make_table_with_cells(
            4,
            3,
            vec![
                cell_rs(0, 0, 3), // rows 0,1,2
                cell_rs(1, 1, 3), // rows 1,2,3
                cell_rs(0, 2, 1),
                cell_rs(3, 0, 1),
            ],
        );
        let (s, e) = compute_row_blocks(&table, 4);
        assert_eq!(s, vec![0, 0, 0, 0]);
        assert_eq!(e, vec![4, 4, 4, 4]);
    }

    #[test]
    fn test_compute_row_blocks_disjoint() {
        // 비인접 rowspan은 별개 블록
        let table = make_table_with_cells(
            5,
            1,
            vec![
                cell_rs(0, 0, 2), // rows 0~1
                cell_rs(2, 0, 1),
                cell_rs(3, 0, 2), // rows 3~4
            ],
        );
        let (s, e) = compute_row_blocks(&table, 5);
        assert_eq!(s, vec![0, 0, 2, 3, 3]);
        assert_eq!(e, vec![2, 2, 3, 5, 5]);
    }

    #[test]
    fn test_row_block_for_basic() {
        // 행 0+1을 묶는 rs=2 셀
        let mt = MeasuredTable {
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![20.0, 30.0, 25.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0, 85.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![0, 0, 2],
            row_block_end: vec![2, 2, 3],
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
            para_index: 0,
            control_index: 0,
            total_height: 50.0,
            row_heights: vec![20.0, 30.0],
            caption_height: 0.0,
            cell_spacing: 5.0,
            cumulative_heights: vec![0.0, 20.0, 55.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
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
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![10.0, 10.0, 10.0, 10.0, 10.0],
            caption_height: 0.0,
            cell_spacing: 0.0,
            cumulative_heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![0, 0, 2, 3, 3],
            row_block_end: vec![2, 2, 3, 5, 5],
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
            para_index: 0,
            control_index: 0,
            total_height: 100.0,
            row_heights: vec![10.0, 10.0, 10.0, 10.0, 10.0],
            caption_height: 0.0,
            cell_spacing: 0.0,
            cumulative_heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::RowBreak,
            row_block_start: vec![0, 0, 2, 3, 3],
            row_block_end: vec![2, 2, 3, 5, 5],
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
            para_index: 0,
            control_index: 0,
            total_height: 0.0,
            row_heights: vec![],
            caption_height: 0.0,
            cell_spacing: 0.0,
            cumulative_heights: vec![0.0],
            repeat_header: false,
            has_header_cells: false,
            cells: vec![],
            page_break: crate::model::table::TablePageBreak::None,
            row_block_start: vec![],
            row_block_end: vec![],
        };
        assert!(!mt.allows_row_break_split());
        mt.page_break = crate::model::table::TablePageBreak::CellBreak;
        assert!(!mt.allows_row_break_split());
        mt.page_break = crate::model::table::TablePageBreak::RowBreak;
        assert!(mt.allows_row_break_split());
    }
}
