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
}

impl HeightCursor {
    /// 컬럼 진입 시 생성. `vpos_page_base` 초기값은 호출자가 첫 PageItem 에서 산출.
    pub(crate) fn new(
        dpi: f64,
        col_area_y: f64,
        col_area_height: f64,
        col_anchor_y: f64,
        vpos_page_base: Option<i32>,
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
                // 역산 무효(자리차지 표 등) → base=vpos_end 로 검증 실패 유도.
                (prev_vpos_end, false)
            } else {
                self.vpos_lazy_base = Some(lazy_base);
                (lazy_base, false)
            }
        };
        // [Task #412] 현재 paragraph first vpos 우선(spacing_after 인코딩), reset 시 fallback.
        let vpos_end = match curr_first_vpos {
            Some(v) if v > seg.vertical_pos => v,
            _ => prev_vpos_end,
        };
        // [Task #643] sb_N 사전 차감 대상 (vpos_corrected_end_y 내부에서 차감).
        let curr_sb = paragraphs
            .get(item_para)
            .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
            .map(|ps| ps.spacing_before)
            .unwrap_or(0.0);
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
        // [Task #1027 Stage A] 공유 클램프 함수.
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
            self.dpi,
        );
        if std::env::var("RHWP_VPOS_DEBUG").is_ok() {
            let path = if is_page_path { "page" } else { "lazy" };
            eprintln!(
                "VPOS_CORR: path={} pi={} prev_pi={} prev_vpos={} prev_lh={} prev_ls={} vpos_end={} base={} col_y={:.2} y_in={:.2} end_y={:.2} applied={}",
                path, item_para, prev_pi, seg.vertical_pos, seg.line_height, seg.line_spacing,
                vpos_end, base, self.col_area_y, y_offset, end_y, applied,
            );
        }
        if applied {
            end_y
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
        HeightCursor::new(DPI, COL_Y, COL_H, COL_Y, page_base)
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
}
