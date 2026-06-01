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
        let compact_endnote_bottom_rewind = self.suppress_large_forward_jump
            && vpos_rewind
            && y_offset > self.col_area_y + self.col_area_height * 0.75;
        let vpos_end = match curr_first_vpos {
            Some(v) if (self.allow_vpos_rewind || compact_endnote_bottom_rewind) && vpos_rewind => {
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
            || compact_endnote_bottom_rewind;
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
                    // Display-style equation lines in compact endnotes already include a large
                    // trailing line_spacing in some lazy-base flows. Hancom places the next
                    // red question title shortly after the visible equation bottom instead
                    // of after that full trailing gap (3-11월_실전_통합_2022 p11 문13/문14).
                    // Page-base flows already carry the correct 7mm note gap and must keep it.
                    prev_content_bottom_y + 10.0
                } else if y_offset > self.col_area_y + self.col_area_height * 0.75
                    || prev_para.text.trim().is_empty()
                {
                    // Empty paragraphs before the next compact endnote title already carry the
                    // visual spacer. Adding the mid-column buffer again pushes later notes down.
                    y_offset + prev_line_spacing_px
                } else {
                    y_offset + prev_line_spacing_px + 40.0
                };
            Some(preserved_gap_y.min(end_y))
        } else {
            None
        };
        let compact_endnote_new_note_jump = self.suppress_large_forward_jump
            && compact_endnote_question_title
            && (seg.line_height > 1500 || bottom_new_note_gap_cap.is_some())
            && end_y > y_offset + 32.0
            && end_y < y_offset + 120.0;
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
        let compact_endnote_deep_backtrack = self.suppress_large_forward_jump
            && !is_page_path
            && !vpos_rewind
            && !follows_endnote_title
            && !follows_tall_inline_item
            && end_y < y_offset - 8.0
            && end_y >= prev_content_bottom_y
            && end_y <= self.col_area_y + self.col_area_height
            && y_offset > self.col_area_y + self.col_area_height * 0.90
            && y_offset - end_y <= 80.0;
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
        if std::env::var("RHWP_VPOS_DEBUG").is_ok() {
            let path = if is_page_path { "page" } else { "lazy" };
            let stale_forward = self.suppress_large_forward_jump && end_y > y_offset + 100.0;
            eprintln!(
                "VPOS_CORR: path={} pi={} prev_pi={} prev_vpos={} prev_lh={} prev_ls={} vpos_end={} base={} col_y={:.2} y_in={:.2} end_y={:.2} stale_forward={} compact_new_note={} compact_tac_pic_gap={} compact_bottom_rewind={} compact_deep_backtrack={} applied={}",
                path, item_para, prev_pi, seg.vertical_pos, seg.line_height, seg.line_spacing,
                vpos_end, base, self.col_area_y, y_offset, end_y, stale_forward, compact_endnote_new_note_jump, compact_endnote_tac_picture_gap, compact_endnote_bottom_rewind, compact_endnote_deep_backtrack, (applied || compact_endnote_deep_backtrack) && !stale_forward && !compact_endnote_new_note_jump && !compact_endnote_tac_picture_gap,
            );
        }
        let stale_forward = self.suppress_large_forward_jump && end_y > y_offset + 100.0;
        if applied && (compact_endnote_new_note_jump || compact_endnote_tac_picture_gap) {
            // Compact endnote flow encodes visual gaps in absolute vpos.
            // Suppressed gaps must also move the vpos base, otherwise the next
            // line restores the skipped gap.
            let rendered_y = if compact_endnote_new_note_jump {
                bottom_new_note_gap_cap.unwrap_or(y_offset)
            } else {
                y_offset
            };
            let suppressed_hu = ((end_y - rendered_y).max(0.0) / self.dpi * 7200.0).round() as i32;
            if suppressed_hu > 0 {
                if is_page_path {
                    self.vpos_page_base = Some(base + suppressed_hu);
                } else {
                    self.vpos_lazy_base = Some(base + suppressed_hu);
                }
            }
        }
        if compact_endnote_title_tail_backtrack {
            y_offset - (y_offset - end_y).min(16.0)
        } else if (applied || compact_endnote_deep_backtrack)
            && !stale_forward
            && !compact_endnote_new_note_jump
            && !compact_endnote_tac_picture_gap
        {
            end_y
        } else if compact_endnote_new_note_jump {
            bottom_new_note_gap_cap.unwrap_or(y_offset)
        } else {
            y_offset
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

    /// 직전 문단의 line spacing 안으로만 당겨지는 새 미주 제목은 하단 overflow 완화를 위해 허용한다.
    #[test]
    fn compact_endnote_deep_backtrack_allows_safe_new_note_title() {
        let mut c = compact_endnote_cursor(None);
        c.prev_layout_para = Some(0);
        let mut ps = vec![
            para(0, 70100, 900, 6000, 5000),
            para(0, 70150, 900, 0, 5000),
        ];
        ps[1].text = "문30)".to_string();

        let got = c.vpos_adjust(980.0, 1, &ps, &styles(0.0));
        assert!(got < 980.0, "got={got}");
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
}
