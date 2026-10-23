//! 문서 구조 구성 (Document Composition)
//!
//! 문단의 텍스트를 줄 단위로 분할하고, 각 줄 내에서
//! CharShapeRef 경계에 따라 다중 TextRun으로 분할한다.
//! 인라인 컨트롤(표/도형) 삽입 위치를 식별한다.

use super::layout::{
    control_line_seg_index, estimate_text_width, estimate_text_width_unrounded,
    hancom_regenerated_space_width, map_pua_bullet_char, resolved_to_text_style,
};
use super::style_resolver::{detect_lang_category, ResolvedStyleSet};
use super::{hwpunit_to_px, px_to_hwpunit, TextStyle};
use crate::model::control::Control;
use crate::model::document::Section;
pub use crate::model::paragraph::SpaceMetric;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::model::shape::{Caption, CaptionDirection, ShapeObject};
use crate::renderer::layout_frame::LayoutFrame;
pub use crate::renderer::layout_frame::ParagraphBox;

mod frame_geometry;
mod nested_table_frames;
pub(crate) use nested_table_frames::{
    recompose_cell_lines_with_nested_tables, table_cells_have_no_stored_lines,
};
mod inline_frame_admission;
mod inline_picture_metrics;
mod resolved_fields;
mod stored_marker_frames;
mod stored_tac_host_frame;
mod stored_word_tail;
pub(crate) use inline_picture_metrics::{
    inline_picture_ascent_ratio, inline_picture_occupied_size_hu, inline_picture_size_hu,
};
pub(crate) use stored_tac_host_frame::stored_tac_host_frame;
pub(crate) use stored_word_tail::{project_stored_soft_wraps, projects_current_marker_geometry};
mod punctuation_slots;

/// 글자겹침(CharOverlap) 렌더링 정보
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CharOverlapInfo {
    /// 테두리 타입 (0=없음, 1=원, 2=반전원, 3=사각형, 4=반전사각형)
    pub border_type: u8,
    /// 내부 글자 크기 (%, 기본 100)
    pub inner_char_size: i8,
}

/// 구성된 텍스트 런 (줄 내 동일 스타일 + 동일 언어 구간)
#[derive(Debug, Clone, Default)]
pub struct ComposedTextRun {
    /// 텍스트 조각
    pub text: String,
    /// 글자 스타일 ID (ResolvedStyleSet.char_styles 인덱스)
    pub char_style_id: u32,
    /// 언어 카테고리 (0=한국어, 1=영어, 2=한자, 3=일본어, 4=기타, 5=기호, 6=사용자)
    pub lang_index: usize,
    /// 중립 공백의 반각 전진을 소유한 원래 언어. 표시·자간 슬롯과 구분한다.
    pub neutral_space_origin: Option<usize>,
    /// 글자겹침 정보 (CharOverlap 컨트롤에서 생성된 런인 경우)
    pub char_overlap: Option<CharOverlapInfo>,
    /// 각주/미주 마커 (Some이면 위첨자로 렌더링, 텍스트 흐름에 포함)
    pub footnote_marker: Option<u16>,
    /// PUA 옛한글 변환 후 표시 텍스트 (Some 이면 렌더러는 본 필드 사용).
    /// `text` 는 IR 와 동일하게 PUA char 1글자로 보존하여 char_offsets /
    /// char_start / line_chars 등 인덱싱 불변성을 유지한다 (Task #528).
    pub display_text: Option<String>,
    /// Logical paragraph grapheme membership, not a font/style heuristic.
    pub supplemental_metrics_blocked: bool,
    /// Text inserted from a control payload, not a scalar span of Paragraph.text.
    pub inserted_control_text: bool,
    /// 공백 폭의 출처는 run 분할 뒤에도 줄 채움 결과를 따른다.
    pub space_metric: SpaceMetric,
    /// 새 줄 구성의 글자 메트릭 출처. 저장 줄의 원래 전진과 구분한다.
    pub regenerated_font_metrics: bool,
}

impl ComposedTextRun {
    /// 원문 scalar가 없는 control 표제는 뒤 본문의 문자 좌표를 전진시키지 않는다.
    pub(crate) fn source_scalar_count(&self) -> usize {
        if self.inserted_control_text && self.char_overlap.is_none() {
            0
        } else {
            self.text.chars().count()
        }
    }

    pub(crate) fn text_style(&self, styles: &ResolvedStyleSet) -> TextStyle {
        let mut style = resolved_to_text_style(styles, self.char_style_id, self.lang_index);
        if let Some(origin) = self.neutral_space_origin {
            punctuation_slots::apply_space_origin(&mut style, styles, self.char_style_id, origin);
        }
        if mapped_symbol_fonts::owns_shadowed_circle_font(self) {
            // 표시 기호 한 글자만 실제 프로그램을 선택한다. 본문과 IR의 글꼴은 보존한다.
            style.font_family = format!(
                "{},{}",
                mapped_symbol_fonts::SHADOWED_CIRCLE_FONT,
                style.font_family
            );
        }
        if self.supplemental_metrics_blocked {
            style.supplemental_metrics = None;
        }
        conjoining_jamo::apply_font(self, &mut style);
        self.space_metric.apply_to_style(&mut style);
        style.regenerated_font_metrics = self.regenerated_font_metrics;
        style
    }

    /// 줄 상자를 정하는 글자 크기(px). 글자 모양의 **선언 크기**다.
    ///
    /// [#7398] 언어별 상대 크기(`relSz`)는 글리프 폭과 그리는 크기에만 곱한다
    /// ([`Self::text_style`]). 한/글은 줄 상자를 기준 크기로 둔다 — `exam_eng.hwp`
    /// 의 라틴 106% 문단도 저장 `LINE_SEG` 가 `lh = th = 1150`(기준 크기)이다.
    /// 조판의 `composed_line_max_font_size` 와 같은 값을 소비해야 측정과 배치의
    /// 줄 진행이 갈라지지 않는다.
    pub(crate) fn line_box_font_size(&self, styles: &ResolvedStyleSet) -> f64 {
        styles
            .char_styles
            .get(self.char_style_id as usize)
            .map(|style| style.font_size)
            .filter(|size| *size > 0.0)
            .unwrap_or(12.0)
    }
}

mod auto_number_projection;
mod conjoining_jamo;
mod equation_line_metrics;
mod mapped_symbol_fonts;
mod stored_row_fragments;
mod text_metric_origin;
pub(crate) use stored_row_fragments::crosses_physical_frame as stored_rows_cross_physical_frame;
pub(crate) mod stored_row_projection;
pub(crate) mod supplemental_clusters;

/// 구성된 줄 (LineSeg 기반)
#[derive(Debug, Clone)]
pub struct ComposedLine {
    /// 스타일별 텍스트 런 목록
    pub runs: Vec<ComposedTextRun>,
    /// 원본 LineSeg (높이, 베이스라인 등)
    pub line_height: i32,
    /// 베이스라인 거리
    pub baseline_distance: i32,
    /// 세그먼트 폭
    pub segment_width: i32,
    /// 컬럼 시작 위치
    pub column_start: i32,
    /// 줄간격 (LineSeg.line_spacing)
    pub line_spacing: i32,
    /// 강제 줄 바꿈(\n, Shift+Enter)으로 끝나는 줄인지 여부
    pub has_line_break: bool,
    /// 이 줄의 첫 문자가 para.text 내에서 갖는 절대 char 인덱스
    pub char_start: usize,
}

/// 인라인 컨트롤 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InlineControlType {
    /// 표
    Table,
    /// 도형/그림
    Shape,
    /// 기타 (구역정의, 단정의 등)
    Other,
}

/// 인라인 컨트롤 위치 정보
#[derive(Debug, Clone)]
pub struct InlineControl {
    /// 삽입될 줄 인덱스
    pub line_index: usize,
    /// Paragraph.controls 내 인덱스
    pub control_index: usize,
    /// 컨트롤 종류
    pub control_type: InlineControlType,
}

/// 구성된 문단
#[derive(Debug, Clone)]
pub struct ComposedParagraph {
    /// 줄별 텍스트
    pub lines: Vec<ComposedLine>,
    /// 문단 스타일 ID
    pub para_style_id: u16,
    /// 인라인 컨트롤 위치 목록
    pub inline_controls: Vec<InlineControl>,
    /// 개요 번호/글머리표 등 문단 머리 텍스트 (렌더링 전용)
    /// 문서 좌표 char_offset에 포함되지 않으며 별도 TextRunNode로 렌더링된다.
    /// 문단 머리 속성은 마커 영역(본문과의 거리·너비 보정·자동 내어쓰기)을 정한다.
    pub numbering_text: Option<(String, crate::model::paragraph::MarkerHead)>,
    /// treat_as_char 컨트롤의 텍스트 위치와 HWPUNIT 너비 목록
    /// (para.text 내 절대 char 인덱스, 폭 HWPUNIT, para.controls 내 인덱스)
    pub tac_controls: Vec<(usize, i32, usize)>,
    /// 각주/미주 위치: (텍스트 내 char 인덱스, 번호, para.controls 내 인덱스)
    pub footnote_positions: Vec<(usize, u16, usize)>,
    /// 탭 확장 데이터 (HWP tab_extended / HWPX 인라인 탭)
    /// ext[0]=width, ext[1]=leader/fill_type, ext[2]=tab_type
    pub tab_extended: Vec<[u16; 7]>,
    /// 재조판 프레임이 발행한 줄 좌표와 공백 규칙. 원본 모델은 그대로 보존한다.
    pub(crate) frame_geometry: Option<std::sync::Arc<frame_geometry::FrameGeometry>>,
    /// Q2-C에서 qualified된 dormant shaping 결과. D1은 수명만 composition
    /// owner로 넘기며 line breaking·layout·paint는 아직 이 값을 소비하지 않는다.
    pub(crate) horizontal_shaping:
        Option<std::sync::Arc<crate::renderer::shaping_paragraph::HorizontalShapingLineOutcome>>,
}

/// Renderer-session cache for the expensive stored single-line overflow probe.
///
/// Paragraph identity is valid only until the owning layout session clears its
/// caches. The concrete cell width completes the key; source/style mutations
/// clear the session cache instead of reaching into the source model.
#[derive(Default)]
pub(crate) struct SingleLineOverflowCache {
    entries: std::cell::RefCell<std::collections::HashMap<(usize, u32), bool>>,
}

impl SingleLineOverflowCache {
    #[inline]
    fn get(&self, para: &Paragraph, width_key: u32) -> Option<bool> {
        self.entries
            .borrow()
            .get(&(para as *const Paragraph as usize, width_key))
            .copied()
    }

    #[inline]
    fn insert(&self, para: &Paragraph, width_key: u32, overflowed: bool) {
        self.entries
            .borrow_mut()
            .insert((para as *const Paragraph as usize, width_key), overflowed);
    }

    pub(crate) fn clear(&self) {
        self.entries.borrow_mut().clear();
    }
}

// 같은 composer 모듈 범위의 source를 책임 경계별로 보존한다.
include!("composer/composition.inc.rs");
include!("composer/stored_tac_display.inc.rs");
include!("composer/stored_lines_runs.inc.rs");
include!("composer/inline_control_projection.inc.rs");
include!("composer/reflow_helpers.inc.rs");
include!("composer/stored_cell_reflow.inc.rs");
include!("composer/cell_line_breaking.inc.rs");
include!("composer/pua_surface.inc.rs");

mod line_breaking;
mod picture_anchor_projection;
pub(crate) use line_breaking::{
    control_is_width_neutral_marker, frame_metrics_for_line, text_word_ranges,
};
pub mod lineseg_compare;

pub(crate) use line_breaking::{
    is_line_end_forbidden, is_line_start_forbidden, layout_paragraph_in_frame,
    layout_picture_anchor_in_frame, layout_picture_band, paragraph_flow_end,
    recalculate_section_vpos, reflow_line_segs, reflow_line_segs_after_cell_split,
    reflow_line_segs_after_cell_text_edit, reflow_line_segs_in_stored_section, restamp_indentation,
    tokenize_paragraph, BreakToken, StoredRowMissPolicy,
};

#[cfg(test)]
mod lineseg_compare_tests;
#[cfg(test)]
mod re_sample_gen;
#[cfg(test)]
mod tests;

/// 공유 프레임이 저장 수식 메트릭을 실제 부분 재조판하는 경로인지 판별한다.
pub(crate) fn uses_remeasured_equation_frame(para: &Paragraph) -> bool {
    line_breaking::remeasured_equation_start_row(para).is_some()
        && line_breaking::supports_cached_body_frame_controls(para)
        && !para.stored_text_partition_is_dirty()
}
