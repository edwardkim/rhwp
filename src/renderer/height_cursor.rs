//! [Task #1027 Stage C] 공유 측정 커서 (페이지네이터 ↔ 렌더러 y-advance 정합).
//!
//! 렌더러(`layout.rs build_single_column`)의 컬럼 단위 inter-item VPOS_CORR
//! 상태머신을 캡슐화한다. 한 컬럼을 흐르는 동안의 vpos 기준점(page_base/
//! lazy_base)과 직전 항목 추적 상태를 보유하며, 항목 사이의 vpos 보정(Stage A
//! `vpos_corrected_end_y` + Stage B `para_has_overlay_shape` 결합)을 적용한다.
//!
//! Stage C: 렌더러가 이 커서에 위임(무동작). Stage D 에서 페이지네이터(typeset)
//! 가 동일 커서로 height-only 패스를 수행하여 두 측정 공간을 일치시킨다.
//!
//! 보유 상태(렌더러 build_single_column 로컬과 1:1):
//! - `vpos_page_base` / `vpos_lazy_base`: vpos→y 변환 기준점 (#412).
//! - `prev_layout_para`: 직전에 배치한 문단 인덱스.
//! - `prev_item_was_partial_table`: 직전 항목이 분할 표였는지 (#991).
//!
//! 기하 상수: `dpi`, `col_area_y/height`, `col_anchor_y`.

use super::layout::{para_has_overlay_shape, vpos_corrected_end_y};
use super::style_resolver::ResolvedStyleSet;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{TextWrap, VertRelTo};
use crate::renderer::hwpunit_to_px;

fn para_has_visible_text(para: &Paragraph) -> bool {
    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
}

fn para_is_treat_as_char_equation_only(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, Control::Equation(eq) if eq.common.treat_as_char))
}

fn para_is_treat_as_char_picture_only(para: &Paragraph) -> bool {
    para.text.trim().is_empty()
        && para.controls.iter().any(|ctrl| match ctrl {
            Control::Picture(pic) => pic.common.treat_as_char,
            Control::Shape(shape) => shape.common().treat_as_char,
            _ => false,
        })
}

pub(crate) struct HeightCursor {
    /// DPI (px/inch).
    pub dpi: f64,
    /// 단 영역 top y (px). lazy_path anchor.
    pub col_area_y: f64,
    /// 단 영역 높이 (px). 본문내 클램프 상한 산출.
    pub col_area_height: f64,
    /// body_wide_reserved 푸시 적용 후 첫 항목 y (px). page_path anchor (#412).
    pub col_anchor_y: f64,
    /// 페이지 기준 vpos. 첫 PageItem 이 명확한 vpos 를 가질 때 (#412).
    pub vpos_page_base: Option<i32>,
    /// 지연 기준 vpos. 첫 PageItem 이 신뢰 불가할 때 sequential y 에서 역산 (#412).
    pub vpos_lazy_base: Option<i32>,
    /// 직전 배치 문단 인덱스.
    pub prev_layout_para: Option<usize>,
    /// 직전 항목이 분할 표(PartialTable)였는지 (#991).
    pub prev_item_was_partial_table: bool,
    /// HWP3-origin 흐름에서는 vpos 보정에서 spacing_before 사전 차감을 생략한다(#1116).
    pub skip_spacing_before_prededuct: bool,
    /// 미주 흐름에서는 LINE_SEG vpos 가 같은 단 안에서 크게 되감길 수 있다.
    pub allow_vpos_rewind: bool,
    /// 미주 단은 하단부에서 뒤로 크게 되감을 수 있다.
    pub allow_start_height_backtrack: bool,
    /// 미주 흐름의 큰 forward vpos 점프는 단/쪽 재배치 흔적일 수 있어 순차 배치를 유지한다.
    pub suppress_large_forward_jump: bool,
    /// [Task #1246] 현재 섹션 미주의 between-notes 마진(HU, 0=미적용). 새 미주 제목이 forward
    /// 흐름에서 이 마진보다 작은 간격을 가지면(다줄 풀이 끝 trailing 누락=문22) 끌어올린다.
    /// 생성자는 0 으로 두고 호출자(build_single_column)가 미주 흐름 컬럼에서만 설정한다.
    pub endnote_between_notes_hu: i32,
    /// 렌더러가 기록한 직전 항목의 실제 콘텐츠 하단(px). trailing 줄간격을 실제
    /// 콘텐츠 하단으로 오인하는 compact 미주 경계에서 공통 gap 기준으로 사용한다.
    pub prev_item_content_bottom_y: Option<f64>,
}

impl HeightCursor {
    /// 컬럼 진입 시 생성. `vpos_page_base` 초기값은 호출자가 첫 PageItem 에서 산출.
    pub(crate) fn new(
        dpi: f64,
        col_area_y: f64,
        col_area_height: f64,
        col_anchor_y: f64,
        vpos_page_base: Option<i32>,
        skip_spacing_before_prededuct: bool,
        allow_vpos_rewind: bool,
        allow_start_height_backtrack: bool,
        suppress_large_forward_jump: bool,
    ) -> Self {
        HeightCursor {
            dpi,
            col_area_y,
            col_area_height,
            col_anchor_y,
            vpos_page_base,
            vpos_lazy_base: None,
            prev_layout_para: None,
            prev_item_was_partial_table: false,
            skip_spacing_before_prededuct,
            allow_vpos_rewind,
            allow_start_height_backtrack,
            suppress_large_forward_jump,
            endnote_between_notes_hu: 0,
            prev_item_content_bottom_y: None,
        }
    }

    /// 항목 배치 직전, vpos 기반 y_offset 보정을 적용한다.
    ///
    /// 렌더러 `build_single_column` 의 inter-item VPOS_CORR 블록과 동작 동일.
    /// 보정이 적용되면 보정된 y, 아니면 입력 `y_offset` 을 그대로 반환한다.
    /// `vpos_lazy_base` 는 지연 산출 시 갱신된다.
    ///
    /// 호출자는 `!shape_jumped && !prev_tac_seg_applied` 가드 안에서 호출한다.
    pub(crate) fn vpos_adjust(
        &mut self,
        y_offset: f64,
        item_para: usize,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let Some(prev_pi) = self.prev_layout_para else {
            return y_offset;
        };
        if item_para == prev_pi {
            return y_offset;
        }
        // 글앞으로/글뒤로/위아래 Shape·Picture 가 있는 문단: vpos 에 개체 높이 포함 → bypass
        // (#409, #1027 Stage B). 분할 표 직후 첫 문단도 sequential 신뢰 (#991).
        let prev_has_overlay_shape = paragraphs
            .get(prev_pi)
            .map(para_has_overlay_shape)
            .unwrap_or(false);
        if prev_has_overlay_shape || self.prev_item_was_partial_table {
            return y_offset;
        }
        let Some(prev_para) = paragraphs.get(prev_pi) else {
            return y_offset;
        };
        // Task #332 Stage 5: width 검증을 가드 조건으로 약화, 마지막 유효 segment 사용.
        let prev_seg = prev_para
            .line_segs
            .iter()
            .rev()
            .find(|ls| ls.segment_width > 0)
            .or_else(|| prev_para.line_segs.last());
        let Some(seg) = prev_seg else {
            return y_offset;
        };
        if seg.vertical_pos == 0 && prev_pi > 0 {
            return y_offset;
        }
        let prev_vpos_end = seg.vertical_pos + seg.line_height + seg.line_spacing;
        let curr_first_vpos = paragraphs
            .get(item_para)
            .and_then(|p| p.line_segs.first())
            .map(|ls| ls.vertical_pos);
        // [Task #412] page_base / lazy_base 경로 분리.
        let (base, is_page_path) = if let Some(b) = self.vpos_page_base {
            (b, true)
        } else if let Some(b) = self.vpos_lazy_base {
            (b, false)
        } else {
            // [Task #1022 v2] trailing-ls 보정의 조건부 복원 (upstream stream/devel 정합).
            // 컬럼이 vpos 0 에서 시작해 sequential 이 IR 을 정확히 추적(drift 0)하면
            // +trailing_ls 는 over-correction(lazy_base 음수 → 표 overflow, exam_kor p5).
            // 그러나 컬럼이 vpos 0 이 아닌 곳에서 시작(상단 박스/도형 뒤 본문, footnote-01 p1)
            // 하면 trailing_ls bridge 가 필요. 게이트: 보정 lazy_base ≥ 0 이면 보정 적용.
            // [Task #1049] 직전이 실텍스트 본문 문단이고 vpos 가 연속
            // (curr_first_vpos == prev_vpos_end)이면, 그 문단의 trailing 줄간격이 이미
            // 연속 vpos 흐름·sequential y 에 포함되어 있으므로 trailing-ls bridge 를 끈다
            // (인라인 TAC 리셋 직후 +trailing_ls 가 12.8px 과대 전진을 일으키는 회귀 차단).
            // - curr_first_vpos 가 prev_vpos_end 초과(gap: top-box 후 본문·footnote-01 p1)
            //   또는 미상이면 종전대로 bridge 적용(#1022 v2).
            // - 직전이 빈 문단이면 렌더러의 빈줄 높이 억제로 trailing_ls 가 sequential y 에
            //   반영되지 않을 수 있어 bridge 유지(복학원서 page1: 빈 문단 뒤 폼 표).
            let prev_has_text = prev_para
                .text
                .chars()
                .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
            let vpos_continuous = matches!(curr_first_vpos, Some(v) if v <= prev_vpos_end);
            let trailing_ls_hu = if vpos_continuous && prev_has_text {
                0
            } else {
                paragraphs
                    .get(prev_pi)
                    .and_then(|p| p.line_segs.last())
                    .map(|s| s.line_spacing.max(0))
                    .unwrap_or(0)
            };
            let y_delta_hu = ((y_offset - self.col_area_y) / self.dpi * 7200.0).round() as i32;
            let lazy_base_corrected = prev_vpos_end - (y_delta_hu + trailing_ls_hu);
            let lazy_base = if lazy_base_corrected >= 0 {
                lazy_base_corrected
            } else {
                prev_vpos_end - y_delta_hu
            };
            if lazy_base < 0 {
                // 역산 무효(자리차지 표 등): 이전 개체 높이가 sequential y 에 이미
                // 반영된 상태다. 여기서 vpos 보정을 적용하면 단 상단으로 되감겨
                // 본문 표와 뒤따르는 미주가 겹친다.
                if std::env::var("RHWP_VPOS_DEBUG").is_ok() {
                    eprintln!(
                        "VPOS_CORR_SKIP: pi={} prev_pi={} y_in={:.2} prev_vpos_end={} lazy_base_corrected={} lazy_base={}",
                        item_para, prev_pi, y_offset, prev_vpos_end, lazy_base_corrected, lazy_base,
                    );
                }
                let compact_endnote_question_title = self.suppress_large_forward_jump
                    && paragraphs
                        .get(item_para)
                        .map(|p| p.text.trim_start().starts_with('문'))
                        .unwrap_or(false)
                    && seg.line_spacing > 1000;
                if compact_endnote_question_title
                    && y_offset > self.col_area_y + self.col_area_height * 0.85
                {
                    let prev_line_spacing_px = (seg.line_spacing.max(0) as f64) / 7200.0 * self.dpi;
                    let prev_content_bottom_y = y_offset - prev_line_spacing_px;
                    let capped_y = prev_content_bottom_y + 10.0;
                    if capped_y < y_offset
                        && y_offset - capped_y <= 24.0
                        && capped_y >= self.col_area_y
                    {
                        return capped_y;
                    }
                }
                return y_offset;
            } else {
                self.vpos_lazy_base = Some(lazy_base);
                (lazy_base, false)
            }
        };
        // [Task #874 #8] stale table-host(TopAndBottom+vert=Para) 판정.
        let curr_has_topbottom_para_table = paragraphs
            .get(item_para)
            .map(|p| {
                p.controls.iter().any(|c| {
                    matches!(c, Control::Table(t)
                        if !t.common.treat_as_char
                        && matches!(t.common.text_wrap, TextWrap::TopAndBottom)
                        && matches!(t.common.vert_rel_to, VertRelTo::Para))
                })
            })
            .unwrap_or(false);
        // [Task #412] 현재 paragraph first vpos 우선(spacing_after 인코딩), reset 시 fallback.
        //
        // 단, 현재 문단이 para-relative TopAndBottom 표의 host 이면 first_vpos 가 표
        // 예약 높이를 포함할 수 있다. 그 값을 inter-item 목표 y 로 쓰면 표 높이만큼
        // 빈 공간을 만든 뒤 표를 다시 배치하게 된다(PR #1088 hwp-multi-001 pi=14).
        // 이 경우에는 직전 문단의 line-seg 끝만 신뢰하고, 표 위치/높이는 Table
        // PageItem 렌더 단계에서 반영한다.
        let vpos_rewind = matches!(curr_first_vpos, Some(v) if v < seg.vertical_pos);
        let curr_tac_picture_only = paragraphs
            .get(item_para)
            .map(para_is_treat_as_char_picture_only)
            .unwrap_or(false);
        let compact_endnote_tac_picture_rewind =
            self.suppress_large_forward_jump && vpos_rewind && curr_tac_picture_only;
        let compact_endnote_bottom_rewind = self.suppress_large_forward_jump
            && vpos_rewind
            && y_offset > self.col_area_y + self.col_area_height * 0.75;
        let vpos_end = match curr_first_vpos {
            Some(v)
                if (self.allow_vpos_rewind
                    || compact_endnote_bottom_rewind
                    || compact_endnote_tac_picture_rewind)
                    && vpos_rewind =>
            {
                v
            }
            Some(v) if v > seg.vertical_pos && !curr_has_topbottom_para_table => v,
            _ => prev_vpos_end,
        };
        // [Task #643] sb_N 사전 차감 대상 (vpos_corrected_end_y 내부에서 차감).
        let curr_sb = paragraphs
            .get(item_para)
            .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
            .map(|ps| ps.spacing_before)
            .unwrap_or(0.0);
        // [Task #1027 Stage A] 공유 클램프 함수.
        let allow_large_backward = (self.allow_vpos_rewind && vpos_rewind)
            || (self.allow_start_height_backtrack
                && y_offset > self.col_area_y + self.col_area_height * 0.75)
            || compact_endnote_bottom_rewind
            || compact_endnote_tac_picture_rewind;
        let (end_y, applied) = vpos_corrected_end_y(
            is_page_path,
            self.col_anchor_y,
            self.col_area_y,
            self.col_area_height,
            vpos_end,
            base,
            curr_sb,
            y_offset,
            curr_has_topbottom_para_table,
            self.skip_spacing_before_prededuct,
            allow_large_backward,
            self.dpi,
        );
        let prev_line_spacing_px = (seg.line_spacing.max(0) as f64) / 7200.0 * self.dpi;
        let prev_content_bottom_y = y_offset - prev_line_spacing_px;
        let measured_prev_content_bottom_y =
            self.prev_item_content_bottom_y.filter(|y| y.is_finite());
        let follows_tall_inline_item = self.suppress_large_forward_jump && seg.line_height > 1500;
        let compact_endnote_question_title = self.suppress_large_forward_jump
            && paragraphs
                .get(item_para)
                .map(|p| p.text.trim_start().starts_with('문'))
                .unwrap_or(false)
            && seg.line_spacing > 1000;
        let bottom_new_note_gap_cap = if self.suppress_large_forward_jump
            && end_y <= self.col_area_y + self.col_area_height
            && (y_offset > self.col_area_y + self.col_area_height * 0.75
                || compact_endnote_question_title)
        {
            let preserved_gap_y =
                if compact_endnote_question_title && follows_tall_inline_item && !is_page_path {
                    // compact 미주의 display 수식 뒤 새 문항 제목은 저장 trailing
                    // line_spacing 전체 뒤가 아니라 실제 보이는 수식 하단 뒤의 "미주 사이"
                    // 공통 간격에 맞춘다.
                    // 렌더러가 실제 콘텐츠 하단을 제공하면 그 값을 공통 기준으로 삼고,
                    // height-only 경로처럼 값이 없으면 기존 LINE_SEG 추정값으로 폴백한다.
                    let content_bottom_y = measured_prev_content_bottom_y
                        .map(|y| y.max(prev_content_bottom_y))
                        .unwrap_or(prev_content_bottom_y);
                    let gap_px = if self.endnote_between_notes_hu > 0 {
                        (self.endnote_between_notes_hu as f64) / 7200.0 * self.dpi
                    } else {
                        10.0
                    };
                    content_bottom_y + gap_px
                } else if y_offset > self.col_area_y + self.col_area_height * 0.75
                    || prev_para.text.trim().is_empty()
                {
                    // Empty paragraphs before the next compact endnote title already carry the
                    // visual spacer. Adding the mid-column buffer again pushes later notes down.
                    y_offset + prev_line_spacing_px
                } else {
                    y_offset + prev_line_spacing_px + 40.0
                };
            if compact_endnote_question_title && follows_tall_inline_item && !is_page_path {
                Some(preserved_gap_y)
            } else {
                Some(preserved_gap_y.min(end_y))
            }
        } else {
            None
        };
        let compact_endnote_new_note_jump = self.suppress_large_forward_jump
            && compact_endnote_question_title
            && (seg.line_height > 1500 || bottom_new_note_gap_cap.is_some())
            && end_y > y_offset + 32.0
            && end_y < y_offset + 120.0;
        let compact_endnote_stale_note_gap = self.suppress_large_forward_jump
            && !is_page_path
            && compact_endnote_question_title
            && !follows_tall_inline_item
            && y_offset <= self.col_area_y + self.col_area_height * 0.75
            && end_y > y_offset + 120.0
            && prev_line_spacing_px > 0.0;
        let stale_note_gap_y =
            compact_endnote_stale_note_gap.then_some(y_offset + prev_line_spacing_px);
        let compact_endnote_tac_picture_gap = self.suppress_large_forward_jump
            && !is_page_path
            && end_y > y_offset
            && end_y <= y_offset + 12.0
            && (paragraphs
                .get(prev_pi)
                .map(para_is_treat_as_char_picture_only)
                .unwrap_or(false)
                || paragraphs
                    .get(item_para)
                    .map(para_is_treat_as_char_picture_only)
                    .unwrap_or(false));
        let follows_endnote_title = self.suppress_large_forward_jump
            && paragraphs
                .get(prev_pi)
                .map(|p| p.text.trim_start().starts_with('문'))
                .unwrap_or(false);
        let current_is_endnote_title = self.suppress_large_forward_jump
            && paragraphs
                .get(item_para)
                .map(|p| p.text.trim_start().starts_with('문'))
                .unwrap_or(false);
        // page-path compact 미주 하단의 새 문항 제목은 저장 vpos가 이미
        // 제목/다음 본문을 분리하는 경우가 있다. 기존 95% 꼬리 조건은
        // 2022-09 p17 문29처럼 하단 1줄 차이에서 제목만 아래로 눌러
        // 다음 본문과 겹치게 만들었으므로 page-path 제목만 90%부터 허용한다.
        let title_bottom_threshold = if is_page_path { 0.90 } else { 0.95 };
        let compact_endnote_title_bottom_backtrack = current_is_endnote_title
            && !vpos_rewind
            && !prev_para.text.trim().is_empty()
            && end_y < y_offset - 8.0
            && y_offset > self.col_area_y + self.col_area_height * title_bottom_threshold
            && end_y <= self.col_area_y + self.col_area_height
            && y_offset - end_y <= 32.0;
        let compact_endnote_page_tail_backtrack = self.suppress_large_forward_jump
            && is_page_path
            && !vpos_rewind
            && !follows_tall_inline_item
            && end_y < y_offset - 8.0
            && y_offset > self.col_area_y + self.col_area_height * 0.95
            && end_y <= self.col_area_y + self.col_area_height
            && y_offset - end_y <= 32.0;
        let current_has_visible_text = paragraphs
            .get(item_para)
            .map(para_has_visible_text)
            .unwrap_or(false);
        let compact_endnote_title_body_stale_forward = self.suppress_large_forward_jump
            && !is_page_path
            && follows_endnote_title
            && !current_is_endnote_title
            && !vpos_rewind
            && current_has_visible_text
            && end_y > y_offset + 32.0
            && end_y < y_offset + 120.0
            && y_offset > self.col_area_y + 1.0;
        let title_body_gap_y = compact_endnote_title_body_stale_forward
            .then_some(y_offset + prev_line_spacing_px.max(10.0).min(18.0));
        let compact_endnote_text_after_tall_tail_backtrack = self.suppress_large_forward_jump
            && is_page_path
            && !vpos_rewind
            && follows_tall_inline_item
            && current_has_visible_text
            && !current_is_endnote_title
            && end_y < y_offset - 8.0
            && y_offset > self.col_area_y + self.col_area_height * 0.90
            && end_y <= self.col_area_y + self.col_area_height
            && y_offset - end_y <= 32.0;
        let compact_endnote_deep_backtrack = self.suppress_large_forward_jump
            && !is_page_path
            && !vpos_rewind
            && !follows_endnote_title
            && !follows_tall_inline_item
            && !(compact_endnote_question_title && prev_para.text.trim().is_empty())
            && end_y < y_offset - 8.0
            && end_y >= prev_content_bottom_y
            && end_y <= self.col_area_y + self.col_area_height
            && y_offset > self.col_area_y + self.col_area_height * 0.90
            && y_offset - end_y <= 80.0;
        let compact_endnote_single_line_tail_backtrack = self.suppress_large_forward_jump
            && !is_page_path
            && !vpos_rewind
            && follows_endnote_title
            && end_y < y_offset - 8.0
            && y_offset > self.col_area_y + self.col_area_height
            && end_y <= self.col_area_y + self.col_area_height
            && end_y >= prev_content_bottom_y
            && y_offset - end_y <= 32.0;
        let current_line_advance_px = paragraphs
            .get(item_para)
            .and_then(|p| p.line_segs.first())
            .map(|s| hwpunit_to_px((s.line_height + s.line_spacing).max(0), self.dpi))
            .unwrap_or(0.0);
        let current_line_height_px = paragraphs
            .get(item_para)
            .and_then(|p| p.line_segs.first())
            .map(|s| hwpunit_to_px(s.line_height.max(0), self.dpi))
            .unwrap_or(current_line_advance_px);
        let equation_tail_prev_overlap_tolerance = if is_page_path { 4.0 } else { 0.0 };
        let col_bottom = self.col_area_y + self.col_area_height;
        let compact_endnote_question_title_bottom_fit = self.suppress_large_forward_jump
            && current_is_endnote_title
            && !vpos_rewind
            && current_line_height_px > 0.0
            && y_offset + current_line_height_px > col_bottom + 0.5
            && y_offset <= col_bottom + 80.0
            && prev_content_bottom_y < col_bottom;
        let compact_endnote_equation_tail_fit = self.suppress_large_forward_jump
            && !vpos_rewind
            && paragraphs
                .get(item_para)
                .map(para_is_treat_as_char_equation_only)
                .unwrap_or(false)
            && current_line_advance_px > 0.0
            && y_offset > self.col_area_y + self.col_area_height * 0.95
            && end_y <= y_offset + 0.5
            && end_y + current_line_advance_px > self.col_area_y + self.col_area_height + 0.5
            && end_y + equation_tail_prev_overlap_tolerance >= prev_content_bottom_y
            && end_y - current_line_advance_px <= y_offset;
        let compact_endnote_title_tail_backtrack = self.suppress_large_forward_jump
            && !is_page_path
            && !vpos_rewind
            && follows_endnote_title
            && paragraphs
                .get(item_para)
                .map(|p| p.line_segs.len() >= 3)
                .unwrap_or(false)
            && end_y < y_offset - 8.0
            && y_offset > self.col_area_y + self.col_area_height * 0.90
            && y_offset - end_y <= 80.0;
        // Compact endnote LINE_SEG sometimes encodes a saved visual gap inside
        // the previous line spacing. In the active mid-column flow it is safe
        // to honor that backward target when it stays below the previous
        // visible content bottom; near the column tail, the configured endnote
        // note-gap must remain authoritative.
        let compact_endnote_safe_vpos_backtrack = self.suppress_large_forward_jump
            && !vpos_rewind
            && end_y < y_offset - 8.0
            && end_y >= prev_content_bottom_y
            && end_y <= self.col_area_y + self.col_area_height
            && y_offset <= self.col_area_y + self.col_area_height * 0.75;
        let stale_forward = self.suppress_large_forward_jump && end_y > y_offset + 100.0;
        if compact_endnote_stale_note_gap
            || compact_endnote_title_body_stale_forward
            || (applied && (compact_endnote_new_note_jump || compact_endnote_tac_picture_gap))
        {
            // Compact endnote flow encodes visual gaps in absolute vpos.
            // Suppressed gaps must also move the vpos base, otherwise the next
            // line restores the skipped gap.
            let rendered_y = if let Some(y) = title_body_gap_y {
                y
            } else if compact_endnote_new_note_jump {
                bottom_new_note_gap_cap.unwrap_or(y_offset)
            } else if let Some(y) = stale_note_gap_y {
                y
            } else {
                y_offset
            };
            let base_delta_hu = ((end_y - rendered_y) / self.dpi * 7200.0).round() as i32;
            if base_delta_hu != 0 {
                if is_page_path {
                    self.vpos_page_base = Some(base + base_delta_hu);
                } else {
                    self.vpos_lazy_base = Some(base + base_delta_hu);
                }
            }
        }
        let result = if compact_endnote_title_bottom_backtrack {
            end_y
        } else if compact_endnote_page_tail_backtrack {
            // page-path 하단 tail은 frame 안에 남기기 위해 저장 vpos를 따르되,
            // 이전 텍스트 line의 실제 하단을 깊게 침범하면 문20처럼 본문/수식이
            // 겹친다. 이전 line 하단보다 위로 올라가지 않게 한다.
            end_y.max(prev_content_bottom_y).min(y_offset)
        } else if compact_endnote_text_after_tall_tail_backtrack {
            end_y.max(prev_content_bottom_y).min(y_offset)
        } else if compact_endnote_question_title_bottom_fit {
            // 큰 미주 사이 문서에서는 새 문항 제목 1줄만 단 하단에 남기고
            // 본문은 다음 단으로 넘기는 저장본이 있다. 이때 stale-forward vpos는
            // 버리되, 순차 y가 frame을 조금 넘으면 제목 visual line-height만큼
            // 하단 안쪽으로 당겨 한컴/PDF처럼 제목 tail을 보존한다. 반환값은
            // paragraph top이므로 layout에서 다시 더해지는 spacing_before를 뺀다.
            (col_bottom - current_line_height_px - 7.0 - curr_sb)
                .max(prev_content_bottom_y - curr_sb)
                .max(self.col_area_y)
                .min(y_offset)
        } else if compact_endnote_equation_tail_fit {
            let prev_floor = prev_content_bottom_y - equation_tail_prev_overlap_tolerance;
            // page-path compact 미주 하단의 수식-only tail은 저장 vpos가 직전
            // 수식 line 하단보다 몇 px 위를 가리킬 수 있다. 이때 frame-fit을
            // 우선하되 이전 line과 과도하게 겹치지 않도록 작은 허용폭만 둔다.
            (col_bottom - current_line_advance_px - 2.0)
                .max(prev_floor)
                .max(self.col_area_y)
                .min(y_offset)
        } else if compact_endnote_single_line_tail_backtrack {
            end_y
        } else if compact_endnote_title_tail_backtrack {
            y_offset - (y_offset - end_y).min(16.0)
        } else if let Some(y) = title_body_gap_y {
            y
        } else if (applied || compact_endnote_deep_backtrack || compact_endnote_safe_vpos_backtrack)
            && !stale_forward
            && !compact_endnote_new_note_jump
            && !compact_endnote_tac_picture_gap
        {
            end_y
        } else if compact_endnote_new_note_jump {
            bottom_new_note_gap_cap.unwrap_or(y_offset)
        } else if let Some(y) = stale_note_gap_y {
            y
        } else {
            y_offset
        };
        if std::env::var("RHWP_VPOS_DEBUG").is_ok() {
            let path = if is_page_path { "page" } else { "lazy" };
            eprintln!(
                "VPOS_CORR: path={} pi={} prev_pi={} prev_vpos={} prev_lh={} prev_ls={} vpos_end={} base={} col_y={:.2} y_in={:.2} end_y={:.2} result={:.2} stale_forward={} current_title={} title_bottom={} page_tail={} equation_tail={} single_tail={} compact_new_note={} compact_stale_note_gap={} compact_tac_pic_gap={} compact_bottom_rewind={} compact_deep_backtrack={} compact_safe_backtrack={} applied={}",
                path, item_para, prev_pi, seg.vertical_pos, seg.line_height, seg.line_spacing,
                vpos_end, base, self.col_area_y, y_offset, end_y, result, stale_forward,
                current_is_endnote_title, compact_endnote_title_bottom_backtrack,
                compact_endnote_page_tail_backtrack, compact_endnote_equation_tail_fit,
                compact_endnote_single_line_tail_backtrack,
                compact_endnote_new_note_jump, compact_endnote_stale_note_gap,
                compact_endnote_tac_picture_gap, compact_endnote_bottom_rewind,
                compact_endnote_deep_backtrack, compact_endnote_safe_vpos_backtrack,
                (applied || compact_endnote_deep_backtrack || compact_endnote_safe_vpos_backtrack) && !stale_forward && !compact_endnote_new_note_jump && !compact_endnote_tac_picture_gap,
            );
        }
        let prev_is_multiline = prev_para.line_segs.len() > 1;
        let stored_gap_px = result - y_offset;
        // [Task #1256/#1261] 단일 줄 prev(빈 separator)로 끝나는 미주 제목 경계: y_offset 은
        // typeset 이 주입한 between-notes trailing 을 이미 포함한다. applied/
        // safe_vpos_backtrack 이 그보다 위로 당기는 경우뿐 아니라, page-path vpos 가 소폭
        // 아래로 미는 경우도 y_offset 을 유지해야 한다. 그렇지 않으면 `미주 사이`가 두 번
        // 적용되어 다음 제목들이 아래로 누적 밀린다(미주사이20 p10 문10→문12 overflow).
        // 기준을 y_offset 으로 유지하고, 차이만큼 활성 vpos base 를 이동해 후속 미주 항목이
        // 동일 기준을 따르게 한다. 다줄 prev(문22)는 y_offset 이 between-notes 를 못 가지는
        // 별개 경로라 아래 #1246 rescue(+prev_ls)가 담당.
        let injected_between_notes =
            self.endnote_between_notes_hu > 0 && seg.line_spacing >= self.endnote_between_notes_hu;
        if injected_between_notes
            && compact_endnote_question_title
            && !compact_endnote_title_bottom_backtrack
            && !vpos_rewind
            && !prev_is_multiline
            && (stored_gap_px < -0.5
                || (stored_gap_px > 0.5 && self.endnote_between_notes_hu > 3000))
        {
            let delta_hu = ((result - y_offset) / self.dpi * 7200.0).round() as i32;
            if delta_hu != 0 {
                if is_page_path {
                    self.vpos_page_base = Some(base + delta_hu);
                } else {
                    self.vpos_lazy_base = Some(base + delta_hu);
                }
            }
            return y_offset;
        }
        // [Task #1246] 미주 사이 min-gap: 새 미주 제목이 forward 흐름인데 between-notes 간격을
        // 확보하지 못하면(다줄 풀이로 끝나는 미주 → 직전 다줄 문단 마지막 줄 trailing 이 render
        // 에서 누락 → gap≈0, 문22) 직전 줄간격(주입된 between_notes)만큼 끌어올린다.
        // - 다줄 prev 한정: 단일줄 prev 는 위 #1256 분기가 y_offset(주입 7mm 포함) 유지로 처리.
        // - 이미 충분한 간격(result >= y_offset + prev_ls, 3-09월 문15 등)은 무변경(max 의미).
        // - backtrack/rewind 류(result < y_offset)는 위 분기가 의도 선택한 값이므로 제외.
        // #1238 render-클램프가 침범하던 #1209 safe-vpos-backtrack 과 양립(여기선 forward 만).
        // 핵심: stored vpos 가 gap 을 거의 안 주는 경우(≈0)만 보정한다. 다줄 풀이로 끝나는 미주의
        // 마지막 줄 trailing 이 render 에서 누락되어 gap≈0 이 된 케이스(문22)가 대상.
        // stored vpos 가 의도적으로 작은 gap(예: 문13 ~12px)을 인코딩한 경우는 존중(over-lift 방지).
        if self.endnote_between_notes_hu > 0
            && compact_endnote_question_title
            && !vpos_rewind
            && prev_is_multiline
            && (-0.5..4.0).contains(&stored_gap_px)
        {
            return y_offset + prev_line_spacing_px;
        }
        result
    }

    /// 이미 계산된 vpos 기준 y보다 실제 렌더 y를 아래로 밀었을 때, 후속 항목도
    /// 같은 시각 기준을 따르도록 활성 vpos base를 반대로 이동한다.
    pub(crate) fn shift_vpos_base_for_rendered_delta(&mut self, delta_px: f64) {
        if delta_px <= 0.0 {
            return;
        }
        let delta_hu = (delta_px / self.dpi * 7200.0).round() as i32;
        if delta_hu <= 0 {
            return;
        }
        if let Some(base) = self.vpos_page_base {
            self.vpos_page_base = Some(base - delta_hu);
        } else if let Some(base) = self.vpos_lazy_base {
            self.vpos_lazy_base = Some(base - delta_hu);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::LineSeg;
    use crate::renderer::style_resolver::ResolvedParaStyle;

    // DPI=96 → 75 HWPUNIT = 1px (1 inch = 7200 HU = 96px). 손계산 정합용.
    const DPI: f64 = 96.0;
    const COL_Y: f64 = 100.0;
    const COL_H: f64 = 900.0;

    fn para(para_shape_id: u16, vpos: i32, lh: i32, ls: i32, seg_w: i32) -> Paragraph {
        Paragraph {
            para_shape_id,
            line_segs: vec![LineSeg {
                vertical_pos: vpos,
                line_height: lh,
                line_spacing: ls,
                segment_width: seg_w,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn styles(spacing_before: f64) -> ResolvedStyleSet {
        ResolvedStyleSet {
            para_styles: vec![ResolvedParaStyle {
                spacing_before,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn cursor(page_base: Option<i32>) -> HeightCursor {
        HeightCursor::new(
            DPI, COL_Y, COL_H, COL_Y, page_base, false, false, false, false,
        )
    }

    fn hwp3_origin_cursor(page_base: Option<i32>) -> HeightCursor {
        HeightCursor::new(
            DPI, COL_Y, COL_H, COL_Y, page_base, true, false, false, false,
        )
    }

    fn compact_endnote_cursor(page_base: Option<i32>) -> HeightCursor {
        HeightCursor::new(
            DPI, COL_Y, COL_H, COL_Y, page_base, false, false, false, true,
        )
    }

    /// 직전 문단이 없으면 보정하지 않는다.
    #[test]
    fn no_prev_para_passthrough() {
        let mut c = cursor(Some(0));
        let ps = vec![para(0, 2000, 1000, 0, 5000)];
        assert_eq!(c.vpos_adjust(90.0, 0, &ps, &styles(0.0)), 90.0);
    }

    /// 같은 문단(item==prev)이면 보정하지 않는다.
    #[test]
    fn same_para_passthrough() {
        let mut c = cursor(Some(0));
        c.prev_layout_para = Some(1);
        let ps = vec![para(0, 1000, 1000, 0, 5000), para(0, 2000, 1000, 0, 5000)];
        assert_eq!(c.vpos_adjust(123.0, 1, &ps, &styles(0.0)), 123.0);
    }

    /// 직전 항목이 분할 표였으면(#991) sequential 신뢰 — 보정 안 함.
    #[test]
    fn partial_table_bypass() {
        let mut c = cursor(Some(0));
        c.prev_layout_para = Some(0);
        c.prev_item_was_partial_table = true;
        let ps = vec![para(0, 1000, 1000, 0, 5000), para(0, 2000, 1000, 0, 5000)];
        assert_eq!(c.vpos_adjust(90.0, 1, &ps, &styles(0.0)), 90.0);
    }

    /// 직전 문단의 마지막 seg vpos==0(reset, prev_pi>0)이면 보정 안 함.
    #[test]
    fn vpos_reset_bypass() {
        let mut c = cursor(Some(0));
        c.prev_layout_para = Some(2);
        let ps = vec![
            para(0, 0, 0, 0, 0),
            para(0, 0, 0, 0, 0),
            para(0, 0, 1000, 0, 5000), // prev seg vpos==0, prev_pi=2>0
        ];
        // item_para=1: get(1)=일반. prev=2 의 seg.vpos==0 → bypass.
        assert_eq!(c.vpos_adjust(90.0, 1, &ps, &styles(0.0)), 90.0);
    }

    /// page_path: end_y = col_anchor_y + (vpos_end - base)*scale, 백워드 허용 내 적용.
    #[test]
    fn page_path_applied() {
        let mut c = cursor(Some(0)); // base=0, page_path
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 1000, 1000, 0, 5000), // prev: vpos_end=2000
            para(0, 2000, 1000, 0, 5000), // curr first vpos=2000 > 1000 → vpos_end=2000
        ];
        // raw_end_y = 100 + (2000-0)/75 = 126.6667, sb=0
        let got = c.vpos_adjust(90.0, 1, &ps, &styles(0.0));
        assert!((got - (100.0 + 2000.0 / 75.0)).abs() < 1e-6, "got={got}");
    }

    /// page_path + sb 사전 차감(#643): end_y 에서 spacing_before(px) 만큼 당겨짐.
    #[test]
    fn page_path_sb_prededuct() {
        let mut c = cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![para(0, 1000, 1000, 0, 5000), para(0, 2000, 1000, 0, 5000)];
        // curr_sb=10px → end_y = max(126.6667 - 10, col_y=100) = 116.6667
        let got = c.vpos_adjust(90.0, 1, &ps, &styles(10.0));
        assert!(
            (got - (100.0 + 2000.0 / 75.0 - 10.0)).abs() < 1e-6,
            "got={got}"
        );
    }

    /// HWP3-origin 흐름에서는 #1116 p3 3mm 격자 정합을 위해 sb 사전 차감을 생략한다.
    #[test]
    fn hwp3_origin_page_path_keeps_spacing_before_in_vpos() {
        let mut c = hwp3_origin_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![para(0, 1000, 1000, 0, 5000), para(0, 2000, 1000, 0, 5000)];
        let got = c.vpos_adjust(90.0, 1, &ps, &styles(10.0));
        assert!((got - (100.0 + 2000.0 / 75.0)).abs() < 1e-6, "got={got}");
    }

    /// lazy_path: page_base 없음 → sequential y 에서 lazy_base 역산, 이후 적용.
    #[test]
    fn lazy_path_applied_and_base_set() {
        let mut c = cursor(None); // page_base/lazy_base 모두 None
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 1000, 1000, 0, 5000), // prev_vpos_end=2000
            para(0, 2200, 1000, 0, 5000), // curr vpos=2200>1000 → vpos_end=2200
        ];
        // y_in=120: y_delta_hu=(120-100)*75=1500, lazy_base=2000-1500=500
        // anchor=col_y=100 (lazy): raw_end_y=100+(2200-500)/75=122.6667
        let got = c.vpos_adjust(120.0, 1, &ps, &styles(0.0));
        assert_eq!(c.vpos_lazy_base, Some(500));
        assert!((got - (100.0 + 1700.0 / 75.0)).abs() < 1e-6, "got={got}");
    }

    /// 백워드 클램프: end_y 가 y_offset-8px 미만이면 보정 거부(원 y 유지).
    #[test]
    fn backward_clamp_rejected() {
        let mut c = cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 50, 1000, 0, 5000),
            para(0, 100, 1000, 0, 5000), // curr vpos=100 → end_y≈101.33
        ];
        // y_in=500: end_y=100+100/75=101.33 < 500-8=492 → 미적용
        assert_eq!(c.vpos_adjust(500.0, 1, &ps, &styles(0.0)), 500.0);
    }

    /// Compact 미주 하단에서 VPOS가 크게 되감기면 현재 문단 first_vpos를 재앵커한다.
    #[test]
    fn compact_endnote_bottom_rewind_uses_current_vpos() {
        let mut c = compact_endnote_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 55350, 900, 0, 5000), // prev_vpos_end=56250 → y=850
            para(0, 45000, 900, 0, 5000), // rewind → y=700
        ];

        let got = c.vpos_adjust(850.0, 1, &ps, &styles(0.0));
        assert!((got - 700.0).abs() < 1e-6, "got={got}");
    }

    /// 같은 compact 미주 되감김이라도 단 하단부가 아니면 기존 순차 흐름을 유지한다.
    #[test]
    fn compact_endnote_rewind_above_bottom_keeps_previous_vpos() {
        let mut c = compact_endnote_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 44100, 900, 0, 5000), // prev_vpos_end=45000 → y=700
            para(0, 37500, 900, 0, 5000), // rewind → y=600, but above bottom band
        ];

        let got = c.vpos_adjust(700.0, 1, &ps, &styles(0.0));
        assert!((got - 700.0).abs() < 1e-6, "got={got}");
    }

    /// Compact 미주 하단에서 reset 없는 VPOS 후퇴가 80px 이내면 overflow 완화를 위해 적용한다.
    #[test]
    fn compact_endnote_deep_backtrack_uses_vpos_near_column_bottom() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 71900, 900, 6000, 5000), // prev_vpos_end=78800; spacing 안에서만 backtrack 허용
            para(0, 71950, 900, 0, 5000),    // curr advances, but VPOS target is behind y
        ];

        let got = c.vpos_adjust(980.0, 1, &ps, &styles(0.0));
        assert!(
            (got - (100.0 + (71950.0 - 6800.0) / 75.0)).abs() < 1e-6,
            "got={got}"
        );
    }

    /// Page-base가 이미 확정된 흐름에서는 같은 보정이 직전 줄과 겹칠 수 있어 적용하지 않는다.
    #[test]
    fn compact_endnote_deep_backtrack_skips_page_path() {
        let mut c = compact_endnote_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![para(0, 65000, 2700, 0, 5000), para(0, 66000, 900, 0, 5000)];

        let got = c.vpos_adjust(1005.0, 1, &ps, &styles(0.0));
        assert!((got - 1005.0).abs() < 1e-6, "got={got}");
    }

    /// 수식처럼 tall inline 항목 뒤에서는 되감김이 수식 bbox와 다음 문단을 겹치게 만든다.
    #[test]
    fn compact_endnote_deep_backtrack_skips_after_tall_line() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let ps = vec![para(0, 70100, 2200, 0, 5000), para(0, 70150, 900, 0, 5000)];

        let got = c.vpos_adjust(1030.0, 1, &ps, &styles(0.0));
        assert!((got - 1030.0).abs() < 1e-6, "got={got}");
    }

    /// 새 미주 제목은 미주 사이 간격을 보존해야 하므로 deep backtrack 대상이 아니다.
    #[test]
    fn compact_endnote_deep_backtrack_skips_new_note_title() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![para(0, 70100, 900, 0, 5000), para(0, 70150, 900, 0, 5000)];
        ps[1].text = "문11)".to_string();

        let got = c.vpos_adjust(1030.0, 1, &ps, &styles(0.0));
        assert!((got - 1030.0).abs() < 1e-6, "got={got}");
    }

    /// 실텍스트 뒤에서 직전 문단의 line spacing 안으로만 당겨지는 새 미주 제목은
    /// 하단 overflow 완화를 위해 허용한다.
    #[test]
    fn compact_endnote_deep_backtrack_allows_safe_new_note_title() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 70100, 900, 6000, 5000),
            para(0, 76100, 900, 0, 5000),
        ];
        ps[0].text = "따라서".to_string();
        ps[1].text = "문30)".to_string();

        let got = c.vpos_adjust(980.0, 1, &ps, &styles(0.0));
        assert!(got < 980.0, "got={got}");
    }

    /// page-path 하단의 새 미주 제목도 저장 vpos가 32px 이내 위쪽을 가리키면
    /// 제목을 그 위치로 되돌려 다음 본문 line과 겹치지 않게 한다.
    #[test]
    fn compact_endnote_page_path_title_bottom_backtrack_allows_safe_title() {
        let mut c = compact_endnote_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 61000, 2070, 1984, 5000),
            para(0, 62000, 900, 452, 5000),
        ];
        ps[0].text = "구하는 확률은".to_string();
        ps[1].text = "문29)".to_string();

        let got = c.vpos_adjust(946.0, 1, &ps, &styles(0.0));
        let expected = 100.0 + 62000.0 / 75.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// page-path 하단 tail backtrack은 frame 안에 남아야 하지만 직전 줄의
    /// 실제 콘텐츠 하단을 깊게 침범하면 문20처럼 본문 line과 다음 수식 line이 겹친다.
    #[test]
    fn compact_endnote_page_tail_backtrack_keeps_previous_content_bottom() {
        let mut c = compact_endnote_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 65000, 900, 452, 5000),
            para(0, 66000, 900, 452, 5000),
        ];

        let got = c.vpos_adjust(1000.0, 1, &ps, &styles(0.0));
        let expected = 1000.0 - 452.0 / 75.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// page-path 하단에서 tall inline 뒤의 일반 텍스트도 저장 vpos가 안전한
    /// 위치를 가리키면 직전 콘텐츠 하단까지 당겨 뒤 수식 line의 공간을 만든다.
    #[test]
    fn compact_endnote_page_tail_text_after_tall_line_backtracks_to_previous_bottom() {
        let mut c = compact_endnote_cursor(Some(0));
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 65000, 1650, 452, 5000),
            para(0, 66000, 900, 452, 5000),
        ];
        ps[1].text = "이므로 삼차식".to_string();

        let got = c.vpos_adjust(1000.0, 1, &ps, &styles(0.0));
        let expected = 1000.0 - 452.0 / 75.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 빈 spacer 문단 뒤의 새 미주 제목은 빈 문단이 만든 간격을 다시 되감으면 안 된다.
    #[test]
    fn compact_endnote_deep_backtrack_skips_title_after_empty_spacer() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 70100, 900, 6000, 5000),
            para(0, 70150, 900, 0, 5000),
        ];
        ps[1].text = "문23)".to_string();

        let got = c.vpos_adjust(980.0, 1, &ps, &styles(0.0));

        assert!((got - 980.0).abs() < 1e-6, "got={got}");
    }

    /// 기본 미주 사이 간격을 가진 새 문제 제목이 단 중간에서 과도하게 전진하면
    /// 뒤쪽 TAC 그림/문단이 단 하단을 넘는다. 제목 자체는 유지하되 저장된 간격에
    /// 완충분만 더해 forward jump를 제한한다.
    #[test]
    fn compact_endnote_question_title_caps_large_forward_gap() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 100000, 900, 1984, 5000),
            para(0, 108025, 900, 452, 5000),
        ];
        ps[0].text = "따라서".to_string();
        ps[1].text = "문29)".to_string();

        let got = c.vpos_adjust(650.0, 1, &ps, &styles(0.0));
        let expected = 650.0 + 1984.0 / 75.0 + 40.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 단 중간의 새 미주 제목에서 저장 VPOS가 페이지 하단 근처까지 크게 튀면
    /// 그 절대 위치는 버리되 직전 문단의 미주 사이 간격만 보존한다.
    #[test]
    fn compact_endnote_question_title_preserves_spacing_on_stale_forward_jump() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 100000, 900, 1984, 5000),
            para(0, 150000, 900, 452, 5000),
        ];
        ps[0].text = "따라서".to_string();
        ps[1].text = "문22)".to_string();

        let got = c.vpos_adjust(450.0, 1, &ps, &styles(0.0));
        let expected = 450.0 + 1984.0 / 75.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 빈 문단이 새 미주 제목 앞의 시각 간격을 이미 만들었다면 추가 40px 완충은 넣지 않는다.
    #[test]
    fn compact_endnote_question_title_after_empty_spacer_keeps_stored_gap_only() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 100000, 900, 1984, 5000),
            para(0, 108025, 900, 452, 5000),
        ];
        ps[1].text = "문19)".to_string();

        let got = c.vpos_adjust(650.0, 1, &ps, &styles(0.0));
        let expected = 650.0 + 1984.0 / 75.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 큰 디스플레이 수식 줄 뒤 새 문제 제목은 trailing 줄간격 전체 뒤가 아니라
    /// 보이는 수식 바닥 직후로 붙는다.
    #[test]
    fn compact_endnote_question_title_after_tall_line_uses_content_bottom_gap() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 100000, 2690, 1984, 5000),
            para(0, 109174, 900, 452, 5000),
        ];
        ps[0].text = "따라서".to_string();
        ps[1].text = "문13)".to_string();

        let got = c.vpos_adjust(500.0, 1, &ps, &styles(0.0));
        let expected = 500.0 - 1984.0 / 75.0 + 10.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 렌더러가 실제 콘텐츠 하단을 제공하면 display 수식 뒤 제목은 그 하단 아래로 배치한다.
    #[test]
    fn compact_endnote_question_title_after_tall_line_uses_rendered_content_bottom_gap() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        c.prev_item_content_bottom_y = Some(500.0);
        let mut ps = vec![
            para(0, 100000, 2690, 1984, 5000),
            para(0, 109174, 900, 452, 5000),
        ];
        ps[0].text = "따라서".to_string();
        ps[1].text = "문8)".to_string();

        let got = c.vpos_adjust(500.0, 1, &ps, &styles(0.0));
        let expected = 510.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 설정된 미주 사이 값이 있으면 display 수식 하단 뒤에 그 공통 간격을 적용한다.
    #[test]
    fn compact_endnote_question_title_after_tall_line_uses_between_notes_gap() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        c.prev_item_content_bottom_y = Some(500.0);
        c.endnote_between_notes_hu = 5669; // 20mm
        let mut ps = vec![
            para(0, 100000, 2690, 1984, 5000),
            para(0, 109174, 900, 452, 5000),
        ];
        ps[0].text = "따라서".to_string();
        ps[1].text = "문8)".to_string();

        let got = c.vpos_adjust(500.0, 1, &ps, &styles(0.0));
        let expected = 500.0 + 5669.0 / 75.0;

        assert!(
            (got - expected).abs() < 1e-6,
            "got={got}, expected={expected}"
        );
    }

    /// 새 미주 제목 바로 다음 문단도 제목 위로 되감기면 미주 사이 간격과 제목이 깨진다.
    #[test]
    fn compact_endnote_deep_backtrack_skips_after_note_title() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![para(0, 70100, 900, 0, 5000), para(0, 70150, 900, 0, 5000)];
        ps[0].text = "문11)".to_string();

        let got = c.vpos_adjust(1030.0, 1, &ps, &styles(0.0));
        assert!((got - 1030.0).abs() < 1e-6, "got={got}");
    }

    /// 제목 직후의 다줄 꼬리 문단은 하단 overflow를 막기 위해 제한적으로만 당긴다.
    #[test]
    fn compact_endnote_limited_backtrack_after_note_title_tail() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![para(0, 70100, 900, 0, 5000), para(0, 70150, 900, 0, 5000)];
        ps[0].text = "문27)".to_string();
        ps[1].line_segs.push(LineSeg {
            vertical_pos: 71502,
            line_height: 900,
            text_height: 900,
            baseline_distance: 765,
            line_spacing: 452,
            column_start: 0,
            segment_width: 5000,
            text_start: 0,
            tag: 0,
        });
        ps[1].line_segs.push(LineSeg {
            vertical_pos: 72854,
            line_height: 900,
            text_height: 900,
            baseline_distance: 765,
            line_spacing: 452,
            column_start: 0,
            segment_width: 5000,
            text_start: 0,
            tag: 0,
        });

        let got = c.vpos_adjust(1030.0, 1, &ps, &styles(0.0));
        assert!(got < 1030.0, "got={got}");
        assert!(got >= 1014.0, "backtrack must stay capped: got={got}");
    }

    /// 되감긴 목표가 직전 줄 내용 하단보다 위이면 overflow보다 겹침 위험이 크다.
    #[test]
    fn compact_endnote_deep_backtrack_skips_if_it_crosses_previous_content() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let ps = vec![para(0, 70100, 900, 452, 5000), para(0, 70150, 900, 0, 5000)];

        let got = c.vpos_adjust(1030.0, 1, &ps, &styles(0.0));
        assert!((got - 1030.0).abs() < 1e-6, "got={got}");
    }

    /// 앞선 TAC 표 높이 때문에 lazy base 역산이 음수가 되면 되감김 보정을 건너뛴다.
    #[test]
    fn invalid_lazy_base_skips_backtrack_after_tall_object() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 18177, 900, 452, 5000), // prev_vpos_end=19529
            para(0, 19529, 900, 452, 5000),
        ];

        let got = c.vpos_adjust(361.37, 1, &ps, &styles(0.0));
        assert!((got - 361.37).abs() < 1e-6, "got={got}");
        assert_eq!(c.vpos_lazy_base, None);
    }

    /// VPOS가 하단부에서 크게 되감길 때(8px 이상)에도 보정이 적용되면
    /// 반환 y 자체가 새 기준이므로 page base는 추가로 움직이지 않는다.
    #[test]
    fn backward_correction_keeps_page_base() {
        let mut c = HeightCursor::new(DPI, COL_Y, COL_H, COL_Y, Some(0), false, true, true, false);
        c.prev_layout_para = Some(0);
        let ps = vec![
            para(0, 1000, 1000, 0, 5000), // prev end: 1100
            para(0, 200, 1000, 0, 5000),  // vpos rewind candidate
        ];

        let got = c.vpos_adjust(780.0, 1, &ps, &styles(0.0));
        let expected_end_y = 100.0 + 200.0 / 75.0;

        assert!((got - expected_end_y).abs() < 1e-6, "got={got}");
        assert_eq!(c.vpos_page_base, Some(0));
    }

    fn multiline_prev_with_injected_gap() -> Paragraph {
        // 다줄(2 seg) 미주 문단, 마지막 seg ls=1984(typeset 가 주입한 between-notes).
        let mut p = para(0, 1000, 900, 0, 5000);
        p.line_segs.push(LineSeg {
            vertical_pos: 1900,
            line_height: 900,
            line_spacing: 1984,
            segment_width: 5000,
            ..Default::default()
        });
        p
    }

    /// [Task #1246] 다줄 풀이로 끝나는 미주 다음 제목이 stored vpos gap≈0(마지막 줄 trailing
    /// 누락=문22)이면 between-notes(직전 주입 줄간격)만큼 끌어올린다.
    #[test]
    fn compact_endnote_min_gap_lifts_zero_gap_question_title() {
        let mut c = compact_endnote_cursor(Some(0));
        c.endnote_between_notes_hu = 1984;
        c.prev_layout_para = Some(0);
        let mut curr = para(0, 2800, 900, 0, 5000); // prev 내용 바닥(1900+900) → stored gap≈0
        curr.text = "문11)".to_string();
        let ps = vec![multiline_prev_with_injected_gap(), curr];
        let y_in = 100.0 + 2800.0 / 75.0; // page_path end_y 와 동일 → stored gap 0
        let got = c.vpos_adjust(y_in, 1, &ps, &styles(0.0));
        let expected = y_in + 1984.0 / 75.0;
        assert!(
            (got - expected).abs() < 1e-6,
            "got={got} expected={expected}"
        );
    }

    /// stored vpos 가 의도적 gap(>4px)을 인코딩하면 over-lift 하지 않는다(문13 류 backtrack/소gap).
    #[test]
    fn compact_endnote_min_gap_respects_existing_vpos_gap() {
        let mut c = compact_endnote_cursor(Some(0));
        c.endnote_between_notes_hu = 1984;
        c.prev_layout_para = Some(0);
        let mut curr = para(0, 3550, 900, 0, 5000); // 2800 + 750(=10px) → stored gap 10px
        curr.text = "문13)".to_string();
        let ps = vec![multiline_prev_with_injected_gap(), curr];
        let y_in = 100.0 + 2800.0 / 75.0;
        let got = c.vpos_adjust(y_in, 1, &ps, &styles(0.0));
        let expected_end = 100.0 + 3550.0 / 75.0; // 보정 없이 vpos 위치 유지
        assert!(
            (got - expected_end).abs() < 1e-6,
            "got={got} expected={expected_end}"
        );
    }

    /// 단일줄 prev 는 trailing 이 이미 sequential y 에 포함되므로 min-gap 보정 대상이 아니다.
    #[test]
    fn compact_endnote_min_gap_skips_single_line_prev() {
        let mut c = compact_endnote_cursor(Some(0));
        c.endnote_between_notes_hu = 1984;
        c.prev_layout_para = Some(0);
        let prev = para(0, 1900, 900, 1984, 5000); // 단일줄, ls=1984
        let mut curr = para(0, 2800, 900, 0, 5000);
        curr.text = "문11)".to_string();
        let ps = vec![prev, curr];
        let y_in = 100.0 + 2800.0 / 75.0;
        let got = c.vpos_adjust(y_in, 1, &ps, &styles(0.0));
        // 단일줄 prev → 보정 없음 (stored gap 0 이어도 lift 안 함)
        assert!((got - y_in).abs() < 1e-6, "got={got} expected={y_in}");
    }

    /// [Task #1256] 단일 줄 prev(빈 separator, ls=between_notes)로 끝나는 미주 제목 경계에서
    /// safe_vpos_backtrack 이 end_y(주입 7mm 미포함)로 cram 하면, y_offset(7mm 포함)을 유지하고
    /// 내린 만큼 vpos base 를 이동한다.
    #[test]
    fn compact_endnote_between_notes_singleline_prev_keeps_gap_and_shifts_base() {
        let mut c = compact_endnote_cursor(Some(0)); // page_path, base=0
        c.endnote_between_notes_hu = 1984;
        c.prev_layout_para = Some(0);
        let prev = para(0, 1900, 900, 1984, 5000); // 단일줄 빈 separator, ls=1984(주입 7mm)
        let mut curr = para(0, 2800, 900, 0, 5000);
        curr.text = "문7)".to_string();
        let ps = vec![prev, curr];
        // end_y = 100 + 2800/75 = 137.333. y_offset=160 → safe_backtrack(end_y<y_offset-8,
        // end_y>=prev_content_bottom=160-1984/75=133.55, mid-column) 발동 → 베이스라인 cram.
        let y_offset = 160.0;
        let got = c.vpos_adjust(y_offset, 1, &ps, &styles(0.0));
        assert!(
            (got - y_offset).abs() < 1e-6,
            "y_offset(7mm 포함) 유지해야 함: got={got}"
        );
        // 제목을 (160 - 137.333)px 내렸으므로 page base 가 그만큼 음수로 이동.
        let end_y = 100.0 + 2800.0 / 75.0;
        let expected_base = -(((y_offset - end_y) / DPI * 7200.0).round() as i32);
        assert_eq!(
            c.vpos_page_base,
            Some(expected_base),
            "내린 만큼 vpos base 이동해야 후속 항목 desync 방지"
        );
    }

    /// [Task #1261] 단일 줄 prev의 between-notes gap이 이미 y_offset에 있으면,
    /// page-path vpos가 제목을 소폭 아래로 밀어도 gap을 한 번 더 더하지 않는다.
    #[test]
    fn compact_endnote_between_notes_singleline_prev_ignores_small_forward_vpos() {
        let mut c = compact_endnote_cursor(Some(0));
        c.endnote_between_notes_hu = 5669;
        c.prev_layout_para = Some(0);
        let prev = para(0, 1900, 900, 5669, 5000);
        let mut curr = para(0, 3550, 900, 0, 5000); // y_offset보다 10px 아래 저장 vpos
        curr.text = "문10)".to_string();
        let ps = vec![prev, curr];
        let y_offset = 100.0 + 2800.0 / 75.0;

        let got = c.vpos_adjust(y_offset, 1, &ps, &styles(0.0));

        assert!(
            (got - y_offset).abs() < 1e-6,
            "이미 주입된 미주 사이 y_offset을 유지해야 함: got={got}, expected={y_offset}"
        );
        let end_y = 100.0 + 3550.0 / 75.0;
        let expected_base = ((end_y - y_offset) / DPI * 7200.0).round() as i32;
        assert_eq!(
            c.vpos_page_base,
            Some(expected_base),
            "올린 만큼 vpos base 이동해야 후속 제목이 다시 밀리지 않음"
        );
    }

    /// [Task #1256] 자연 trailing(ls < between_notes)인 단일 줄 prev 는 injected_between_notes
    /// 가 아니므로 위 보정 대상이 아니다(backtrack 결과 그대로, base 무이동).
    #[test]
    fn compact_endnote_between_notes_skips_natural_trailing_prev() {
        let mut c = compact_endnote_cursor(Some(0));
        c.endnote_between_notes_hu = 1984;
        c.prev_layout_para = Some(0);
        let prev = para(0, 1900, 900, 180, 5000); // 자연 trailing(180 < 1984)
        let mut curr = para(0, 2800, 900, 0, 5000);
        curr.text = "문7)".to_string();
        let ps = vec![prev, curr];
        let base_before = c.vpos_page_base;
        let got = c.vpos_adjust(160.0, 1, &ps, &styles(0.0));
        // injected_between_notes=false → #1256 분기 미발동. base 무변경.
        assert_eq!(c.vpos_page_base, base_before, "base 무이동");
        // 보정 분기를 타지 않으므로 y_offset 유지(cram 아님) 또는 backtrack — 핵심은 base 무이동.
        let _ = got;
    }
}
