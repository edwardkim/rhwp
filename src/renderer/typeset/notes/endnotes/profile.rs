//! 기존 미주 간격·구분선·SSOT 정책 관측. 정책과 상수는 변경하지 않는다.

use crate::renderer::typeset::{border_width_to_px, hwpunit_to_px, FootnoteShape};

/// [Task #1363] 미주 높이 모델 SSOT 마이그레이션 단계 플래그(`RHWP_EN_SSOT`).
///
/// 미주 para 누적(`acc`)을 layout 순차 렌더 높이(`line_advances_sum`)로 점진 이전하는
/// 동안, divergence 항목을 단계별로 게이트하기 위한 A/B 스위치. 기본은 B(A + TAC 그림 미주 순차
/// 적층)이며, `legacy`/`off`로 기존 saved-vpos delta 경로를 비교·롤백할 수 있다.
/// 상세: `mydocs/working/archives/task_m100_1363_stage2.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::renderer::typeset) enum EnSsotLevel {
    /// 전 divergence 원복 — 현행 `metric_advance_px.max(min_h)` (saved-vpos delta). 롤백용.
    Legacy,
    /// Stage 3: Divergence A(내부 vpos rewind)를 SSOT(line_advances_sum)로 이전.
    A,
    /// **기본값(Stage 4 승격)**: A + Divergence C(TAC 그림 미주 순차 적층 — 겹침 가정 제거).
    B,
    /// 예약 tier — 현재 B 와 동일. 잔여 Divergence B(trailing-ls)·전면 SSOT 는 안전 정합
    /// 불가(Stage 5 실증: overflow 무영향이나 2022 overflow/2024·2023 질문흐름 회귀)로 보류.
    On,
    /// [v2 후보 A] 미주 다단 누적을 **렌더러 HeightCursor 시뮬레이션**으로 대체(실험).
    /// compute_en_metrics 근사 대신 build_single_column 동일 경로로 단 bottom y 를 스냅.
    A2,
    /// [v3 후보 A 정확화] A2 시뮬의 per-para 휴리스틱 높이 추정을 **scratch
    /// LayoutEngine::layout_partial_paragraph 실측**(렌더 권위)으로 대체. saved-vpos delta /
    /// total_height 근사 대신 실제 렌더 advance 를 사용 → A2 의 7건 재튜닝 회귀 해소가 목표.
    A3,
}

pub(in crate::renderer::typeset) fn en_ssot_level() -> EnSsotLevel {
    // [Task #1363] 승격 이력:
    //   Stage 3 — A(rewind→line_advances_sum): 전 골든 무회귀로 기본 승격.
    //   Stage 4 — B(+TAC 그림 순차 적층, Divergence C): sep20/20 p22 overflow 50.1→0,
    //             cargo test 2126 pass·sweep flagged 불변으로 기본 승격. 미설정 시 B.
    // `legacy`/`off` 로 전 divergence 원복(긴급 롤백·비교), `A` 는 C 제외 단계, `on` 은 예약(현 B 동일).
    match std::env::var("RHWP_EN_SSOT").ok().as_deref() {
        Some("legacy") | Some("Legacy") | Some("off") => EnSsotLevel::Legacy,
        Some("A") => EnSsotLevel::A,
        Some("on") | Some("On") | Some("ON") => EnSsotLevel::On,
        Some("A2") | Some("a2") => EnSsotLevel::A2,
        Some("A3") | Some("a3") => EnSsotLevel::A3,
        _ => EnSsotLevel::B,
    }
}

/// [Task #1363] 미주 para 단위 SSOT divergence 정량 측정 디버그(`RHWP_EN_SSOT_DEBUG=1`).
/// `scripts/task1363_ssot_diff.py` 가 stderr 의 `EN_SSOT` 라인을 수집한다.
pub(in crate::renderer::typeset) fn en_ssot_debug() -> bool {
    std::env::var("RHWP_EN_SSOT_DEBUG").is_ok()
}
pub(in crate::renderer::typeset) fn endnote_separator_below_margin(shape: &FootnoteShape) -> i16 {
    shape.separator_below_margin_hu()
}

pub(in crate::renderer::typeset) fn endnote_between_notes_margin(shape: &FootnoteShape) -> u16 {
    shape.between_notes_margin_hu()
}

// 3-09월_교육_통합_2022.hwp의 기본 "미주 사이 7mm"는 원본 LINE_SEG
// 흐름에 이미 상당 부분 녹아 있어 추가 pagination 높이로 더하지 않는다.
// 별도 저장한 "미주사이20" 기준 파일에서는 7mm를 넘는 초과분만 다음
// 미주 묶음 vpos에 반영할 때 한컴오피스의 24쪽 분기와 맞는다.
pub(in crate::renderer::typeset) const ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU: i32 = 1984;
pub(in crate::renderer::typeset) const ENDNOTE_COMPACT_SEPARATOR_BELOW_MAX_HU: i16 = 1000;

#[derive(Clone, Copy, Debug)]
pub(in crate::renderer::typeset) struct EndnoteFlowProfile {
    pub(in crate::renderer::typeset) separator_above_hu: i32,
    pub(in crate::renderer::typeset) separator_below_hu: i32,
    pub(in crate::renderer::typeset) between_notes_hu: i32,
    pub(in crate::renderer::typeset) visible_separator: bool,
    pub(in crate::renderer::typeset) absorbed_between_notes_gap: bool,
    pub(in crate::renderer::typeset) compact_separator_below: bool,
    pub(in crate::renderer::typeset) separator_line_width: u8,
}

impl EndnoteFlowProfile {
    pub(in crate::renderer::typeset) fn from_shape(shape: &FootnoteShape) -> Self {
        let separator_above_hu = shape.separator_above_margin_hu() as i32;
        let separator_below_hu = endnote_separator_below_margin(shape) as i32;
        let between_notes_hu = endnote_between_notes_margin(shape) as i32;
        let visible_separator = endnote_has_visible_separator(shape);
        let absorbed_between_notes_gap = endnote_has_absorbed_between_notes_gap(shape);
        let compact_separator_below =
            separator_below_hu <= ENDNOTE_COMPACT_SEPARATOR_BELOW_MAX_HU as i32;

        Self {
            separator_above_hu,
            separator_below_hu,
            between_notes_hu,
            visible_separator,
            absorbed_between_notes_gap,
            compact_separator_below,
            separator_line_width: shape.separator_line_width,
        }
    }

    pub(in crate::renderer::typeset) fn zero_spacing(self) -> bool {
        self.separator_above_hu == 0 && self.between_notes_hu == 0 && self.separator_below_hu == 0
    }

    pub(in crate::renderer::typeset) fn default_or_compact_between_notes(self) -> bool {
        self.between_notes_hu <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
            || self.absorbed_between_notes_gap
    }

    pub(in crate::renderer::typeset) fn default_between_notes(self) -> bool {
        self.between_notes_hu <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
    }

    pub(in crate::renderer::typeset) fn nonzero_default_between_notes(self) -> bool {
        self.between_notes_hu > 0 && self.default_between_notes()
    }

    pub(in crate::renderer::typeset) fn visible_nonzero_default_between_notes(self) -> bool {
        self.visible_separator && self.nonzero_default_between_notes()
    }

    pub(in crate::renderer::typeset) fn visible_non_default_between_notes(self) -> bool {
        self.visible_separator && !self.default_between_notes()
    }

    pub(in crate::renderer::typeset) fn visible_non_default_compact_between_notes(self) -> bool {
        self.visible_non_default_between_notes() && self.default_or_compact_between_notes()
    }

    pub(in crate::renderer::typeset) fn large_between_notes(self) -> bool {
        self.between_notes_hu > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
            && !self.absorbed_between_notes_gap
    }

    pub(in crate::renderer::typeset) fn visible_large_between_notes(self) -> bool {
        self.visible_separator && self.large_between_notes()
    }

    pub(in crate::renderer::typeset) fn no_separator_large_between_notes(self) -> bool {
        !self.visible_separator && self.large_between_notes()
    }

    pub(in crate::renderer::typeset) fn large_separator_margin(self) -> bool {
        self.separator_above_hu > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
            || self.separator_below_hu > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
    }

    pub(in crate::renderer::typeset) fn visible_zero_between_large_separator_margin(self) -> bool {
        self.visible_separator && self.between_notes_hu == 0 && self.large_separator_margin()
    }

    /// [#4318] 구분선 위/아래가 모두 7mm를 넘는 20mm급이고, 미주 사이는
    /// 기본(0이 아닌 7mm 이하). 0/0/0·미주사이 0·미주사이 20은 제외한다.
    pub(in crate::renderer::typeset) fn visible_both_large_separator_default_between(self) -> bool {
        self.visible_separator
            && self.separator_above_hu > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
            && self.separator_below_hu > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
            && self.nonzero_default_between_notes()
    }

    pub(in crate::renderer::typeset) fn visible_large_between_zero_above_compact_below(
        self,
    ) -> bool {
        self.visible_large_between_notes()
            && self.separator_above_hu == 0
            && self.compact_separator_below
    }

    pub(in crate::renderer::typeset) fn pagination_between_notes_margin(self) -> i32 {
        if self.visible_separator && self.absorbed_between_notes_gap {
            0
        } else {
            (self.between_notes_hu - ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU).max(0)
        }
    }

    pub(in crate::renderer::typeset) fn separator_height_px(self, dpi: f64) -> f64 {
        let line_height = if self.visible_separator {
            border_width_to_px(self.separator_line_width).max(0.5)
        } else {
            0.0
        };
        hwpunit_to_px(self.separator_above_hu, dpi)
            + line_height
            + hwpunit_to_px(self.separator_below_hu, dpi)
    }
}

pub(in crate::renderer::typeset) fn endnote_between_notes_pagination_margin(
    shape: &FootnoteShape,
) -> i32 {
    // 7mm 기본값은 저장 LINE_SEG 흐름에 이미 녹아 있지만, 20mm처럼 커진
    // "미주 사이" 초과분은 번호 경계마다 pagination vpos에도 온전히
    // 반영해야 한컴의 단 분기와 맞는다.
    (endnote_between_notes_margin(shape) as i32 - ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU).max(0)
}

pub(in crate::renderer::typeset) fn compact_endnote_between_notes_flow(
    shape: &FootnoteShape,
) -> bool {
    let between = endnote_between_notes_margin(shape) as i32;
    between <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU || endnote_has_absorbed_between_notes_gap(shape)
}

pub(in crate::renderer::typeset) fn endnote_has_absorbed_between_notes_gap(
    shape: &FootnoteShape,
) -> bool {
    let between = endnote_between_notes_margin(shape) as i32;
    if between <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU {
        return false;
    }

    // 한컴 기본 근방의 "미주 사이"는 구분선 아래가 작고 구분선 위가 충분하면
    // 단 전환 기준에서는 별도 20mm 블록처럼 소비되지 않고 앞쪽 여백에 흡수된다.
    let below = endnote_separator_below_margin(shape) as i32;
    let above = shape.separator_above_margin_hu() as i32;
    below <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU && above > 0 && between <= above
}

pub(in crate::renderer::typeset) fn endnote_has_compact_separator_below(
    shape: &FootnoteShape,
) -> bool {
    endnote_separator_below_margin(shape) <= ENDNOTE_COMPACT_SEPARATOR_BELOW_MAX_HU
}

pub(in crate::renderer::typeset) fn endnote_has_visible_separator(shape: &FootnoteShape) -> bool {
    shape.separator_line_type != 0 && shape.separator_line_width != 0
}

pub(in crate::renderer::typeset) fn endnote_separator_height_px(
    shape: &FootnoteShape,
    dpi: f64,
) -> f64 {
    let line_height = if endnote_has_visible_separator(shape) {
        border_width_to_px(shape.separator_line_width).max(0.5)
    } else {
        0.0
    };
    hwpunit_to_px(shape.separator_above_margin_hu() as i32, dpi)
        + line_height
        + hwpunit_to_px(endnote_separator_below_margin(shape) as i32, dpi)
}
