//! 문단 구성 결과와 기존 흐름 높이 조회.
//!
//! fit 높이·전체 높이·줄 전진량은 서로 다른 의미를 유지한다. 원본 IR이나
//! 페이지 상태를 변경하지 않는다. 기존 환경 변수 기반 진단 출력은 유지한다.

use crate::model::paragraph::Paragraph;

/// 문단 format() 결과: 문단의 실제 렌더링 높이 정보
#[derive(Debug, Clone)]
pub(in crate::renderer::typeset) struct FormattedParagraph {
    /// Measured remaining inline space on the composed tail line. None means
    /// that this path has no reliable width result (not an implicit overflow).
    pub(in crate::renderer::typeset) tail_line_remaining_width: Option<f64>,
    /// frame이 실제로 재조판한 줄만 보존한다. Some이면 source 줄로 되돌아가지 않는다.
    pub(in crate::renderer::typeset) computed_host_lines:
        Option<Vec<crate::renderer::float_placement::ParagraphHostLine>>,
    /// 총 높이 (spacing 포함)
    pub(in crate::renderer::typeset) total_height: f64,
    /// 줄별 콘텐츠 높이 (line_height만)
    pub(in crate::renderer::typeset) line_heights: Vec<f64>,
    /// 줄별 줄간격 (line_spacing)
    pub(in crate::renderer::typeset) line_spacings: Vec<f64>,
    /// spacing_before
    pub(in crate::renderer::typeset) spacing_before: f64,
    /// spacing_after
    pub(in crate::renderer::typeset) spacing_after: f64,
    /// trailing line_spacing을 제외한 판단용 높이
    pub(in crate::renderer::typeset) height_for_fit: f64,
    /// `total_height` 에 섞어 넣은 TAC 표 바깥 여백(세로). 저장 사다리 지문처럼
    /// **생성기가 쓴 값과 대조하는** 계산에서는 이 몫을 도로 빼야 한다.
    pub(in crate::renderer::typeset) tac_outer_margin_v_px: f64,
}

impl FormattedParagraph {
    /// 특정 줄의 advance 높이 (콘텐츠 + 줄간격)
    ///
    /// Issue #3780: 연속 페이지 재배치에서 기록된 줄 인덱스가 새 레이아웃 줄 수를
    /// 넘는 off-by-one(len 31, index 31 실측 패닉)이 들어올 수 있다 — 존재하지
    /// 않는 줄의 advance 는 0.0 으로 방어해 렌더를 지속한다.
    #[inline]
    pub(in crate::renderer::typeset) fn line_advance(&self, line_idx: usize) -> f64 {
        if line_idx >= self.line_count() {
            return 0.0;
        }
        self.line_heights[line_idx] + self.line_spacings[line_idx]
    }

    /// 두 벡터가 함께 인덱싱되는 자리의 안전 상한 (구성은 zip 이라 보통 같다).
    #[inline]
    pub(in crate::renderer::typeset) fn line_count(&self) -> usize {
        self.line_heights.len().min(self.line_spacings.len())
    }

    /// 줄 범위의 advance 합계 (Issue #3780 — 범위를 실제 줄 수로 클램프)
    pub(in crate::renderer::typeset) fn line_advances_sum(
        &self,
        range: std::ops::Range<usize>,
    ) -> f64 {
        let end = range.end.min(self.line_count());
        let start = range.start.min(end);
        (start..end)
            .map(|i| self.line_heights[i] + self.line_spacings[i])
            .sum()
    }

    /// [#6753] `flow_advance_height` 가 실제로 트림한 것 중 **`spacing_before` 몫**(px).
    ///
    /// 트림 자체는 "저장 사다리가 이미 담았고 vpos-snap 이 좌표를 복원한다"는 전제 위에
    /// 서지만, **lazy 기준 역산**은 그 복원 이전의 sequential y 를 쓴다. 그래서 역산에만
    /// 이 값을 되돌려 준다(전진량은 그대로 둔다).
    pub(in crate::renderer::typeset) fn flow_trimmed_spacing_before(
        &self,
        para: &Paragraph,
        col_count: u16,
        allow_spacing_before_only: bool,
        ladder_dirty: bool,
        lazy_base: bool,
    ) -> f64 {
        let advance = self.flow_advance_height(
            para,
            col_count,
            allow_spacing_before_only,
            ladder_dirty,
            lazy_base,
        );
        if advance + 0.5 >= self.total_height {
            return 0.0; // 트림이 발동하지 않았다.
        }
        // `flow_advance_height` 의 `sb_trim` 과 같은 판정 — sa 만 트림된 경우는 0.
        let sa_trim = self.spacing_after > 0.5;
        let sb_trim =
            allow_spacing_before_only && self.spacing_before > 0.5 && !(lazy_base && !sa_trim);
        if sb_trim {
            self.spacing_before
        } else {
            0.0
        }
    }

    pub(in crate::renderer::typeset) fn flow_advance_height(
        &self,
        para: &Paragraph,
        col_count: u16,
        allow_spacing_before_only: bool,
        ladder_dirty: bool,
        lazy_base: bool,
    ) -> f64 {
        // [#6970] 다단에서도 **합성(reflow) lineseg 문단은 트림하지 않는다** — 아래
        // `#2279 ①` 이 단단 경로에 건 가드와 같은 이유다. 트림은 "저장 ladder 가 spacing 을
        // 이미 반영하고 vpos-snap 이 좌표를 복원한다"는 전제 위에 서는데, 합성 문단에는 그
        // ladder 가 없어 트림분이 흐름에서 그냥 소실된다.
        //
        // 저장 `LINE_SEG` 가 없는 2단 문서에서 그 소실이 쌓여 단 채움 회계가 무너진다 —
        // `synth_no_ls_square_wrap.hwp` 실측: 문단 151개에서 Σ 741.6px(문단당 6~10px)를
        // 덜 세고, 단 0 은 `usedHeight 708.9 ≤ 가용 718.1` 로 "아직 남았다"고 판단해 계속
        // 담는다. 실제로 담은 항목 합은 1000.4px 라 282px 초과이고, 넘친 내용이 다음 단으로
        // 가지 않고 그 자리에 그려진다(off-canvas 20 · overflow 16).
        //
        // 다단에서 `height_for_fit` 을 쓰는 본래 이유(#391: trailing_ls 인플레이션이 단을
        // 조기 종료시킨다)는 **저장 ladder 가 있는 문단**에 대한 것이므로 그대로 둔다.
        let has_authoritative_seg_for_multicolumn = para.line_segs.iter().any(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        });
        if col_count > 1 && has_authoritative_seg_for_multicolumn {
            return self.height_for_fit;
        }
        // [#2279 ①] spacing 트림은 **비합성(authoritative) 저장 lineseg** 문단에만.
        // 저장 ladder 가 spacing 을 이미 반영하고 vpos-snap 이 좌표를 복원하는
        // 전제의 트림이므로, 합성(reflow) lineseg 문단은 ladder 가 없어 트림하면
        // sb·ls 가 흐름에서 그냥 소실된다 (한글 fresh 는 가산 — 기계생성 결재
        // 문서 −1쪽 계열, DIAG_ADV 실측 문단당 4~33px).
        // [#2279 ①-2] dirty(합성 혼합) ladder 구간도 동일 — #2243 전방-스냅만
        // 허용되어 트림분이 복원되지 않으므로 full advance 를 쓴다
        // (36398700 pi6..9 구간 −60px 폐합, 한글 재저장 anchor 실측).
        let has_authoritative_seg = para.line_segs.iter().any(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        });
        // [#2279 ①-4] lazy-base(page_base 미확립) 사다리에서는 sb-형 트림의
        // 복원(스냅)이 성립하지 않는다 — sb-형만 차단, sa-형 트림은 유지
        // (36398700 pi31/35 −13.5px 미복원 실측 vs issue_1853 캡션 문서의
        // sa-형 트림은 정상 복원되어 전면 차단 시 +1쪽 과다 반증).
        let sa_trim = self.spacing_after > 0.5;
        let sb_trim =
            allow_spacing_before_only && self.spacing_before > 0.5 && !(lazy_base && !sa_trim);
        if std::env::var("RHWP_DIAG_LAZYBLK").is_ok()
            && allow_spacing_before_only
            && self.spacing_before > 0.5
            && lazy_base
            && !sa_trim
        {
            eprintln!(
                "DIAG_LAZYBLK sb={:.1} total={:.1} h4f={:.1}",
                self.spacing_before, self.total_height, self.height_for_fit
            );
        }
        if para.controls.is_empty()
            && has_authoritative_seg
            && !ladder_dirty
            && (sa_trim || sb_trim)
            && self.height_for_fit > 0.0
            && self.height_for_fit + 0.5 < self.total_height
        {
            return self.height_for_fit.min(self.total_height);
        }
        self.total_height
    }
}
