//! 문단 진입의 저장 꼬리·편집 그림 공간 요구 판단.
//! 가용 높이는 기존 단락 평가 위치에서만 조회한다. 진단을 포함할 수 있어 미리 계산하지 않는다.

use super::super::{is_synthetic_line_seg, signed_hwpunit};
use super::metrics::FormattedParagraph;
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::hwpunit_to_px;

/// 저장 꼬리가 현재 쪽을 채웠는지 판단한다. 높이 적용은 호출자의 별도 command다.
pub(super) fn stored_tail_fills_page(
    para_idx: usize,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    hwp5_stored_pagination_layout: bool,
    current_height: f64,
    dpi: f64,
    available_height: impl Fn() -> f64,
) -> bool {
    // [#6793] 앞 문단의 **저장 꼬리가 쪽을 채우고** 이 문단의 첫 저장 줄이
    // `vpos == 0` 이면, 한글은 이 문단을 **다음 쪽 상단**에 둔 것이다. 쪽을 닫는다.
    //
    // 1611000-201000141 표지 실측:
    //
    // ```text
    //   pi=0  ls[0] vpos=0     lh=61600 th=1000     ← 앵커 줄
    //         ls[1] vpos=1600  lh=61600 gap=36960   ← 표 줄 + 꼬리
    //         저장 꼬리 끝 = (1600 + 61600 + 36960)/75 = 1335.5px  > 예산 876.9
    //   pi=1  ls[0] vpos=0                          ← 새 쪽 상단
    // ```
    //
    // 종전에는 흐름 계상이 842.7 에서 멈춰 `pi=1`(`< 차 례 >`)이 1쪽 꼬리에
    // 붙었고, 렌더가 그 꼬리 간격을 더해 용지 밖 363.9px 로 내보냈다.
    // 보이게만 고치면 **쪽 귀속이 여전히 틀리다** — 한/글은 2쪽 첫 줄이다.
    //
    // ⭐ 판정은 저장 사다리 둘이 함께 준다 — 크기 문턱이 없다.
    //   ① 이 문단의 첫 **비합성** 저장 줄이 `vpos == 0`.
    //   ② 앞 문단의 마지막 비합성 저장 줄이 `vpos + lh + gap` 으로 **이 쪽
    //      예산을 넘는다** — 그 꼬리가 쪽-끝 채움이라는 뜻이다.
    //
    // ⚠ ② 가 없으면 안 된다. `vpos == 0` 은 새 쪽 상단인 동시에 **"앵커 없음"
    // 센티널**이기도 하다(`#6753` 이 남긴 함정 — 조각 시작 `vpos == 0` 을 무조건
    // 쪽 경계로 읽은 선행 시도가 242쪽을 243쪽으로 늘려 기각됐다).
    // 저장 사다리가 권위인 **네이티브 HWP5** 조판에 한정한다.
    if hwp5_stored_pagination_layout
        && para_idx > 0
        && current_height > 0.5
        && para
            .line_segs
            .iter()
            .find(|seg| !is_synthetic_line_seg(seg))
            .is_some_and(|seg| seg.vertical_pos == 0)
    {
        // 이 문단의 첫 저장 줄 높이 — 아래 검사의 허용오차다.
        let this_line_px = para
            .line_segs
            .iter()
            .find(|seg| !is_synthetic_line_seg(seg))
            .map(|seg| hwpunit_to_px(seg.line_height, dpi))
            .unwrap_or(0.0);
        let prev_tail_fills_the_page = paragraphs
            .get(para_idx - 1)
            .and_then(|prev| {
                prev.line_segs
                    .iter()
                    .rev()
                    .find(|s| !is_synthetic_line_seg(s))
            })
            .is_some_and(|seg| {
                let without_gap =
                    hwpunit_to_px(seg.vertical_pos.saturating_add(seg.line_height), dpi);
                let with_gap = without_gap + hwpunit_to_px(seg.line_spacing.max(0), dpi);
                // ⚠ **흐름이 사다리가 앞 문단을 남겨 둔 자리에 있어야 한다.**
                // 어긋나 있으면 이미 쪽 경계가 지나간 것이라, 여기서 또 닫으면
                // 쪽이 하나 늘어난다 (`#5941` 1490000-201600081 `pi=1201`:
                // 사다리 867.3 대 흐름 28.4 — 이미 다음 쪽이다. 304 → 305).
                let flow_matches_ladder =
                    (current_height - without_gap).abs() <= this_line_px + 0.5;
                // 꼬리 **없이는** 쪽 안에 들어가는데 꼬리를 실으면 넘는다 —
                // 그 꼬리가 흐름 간격이 아니라 쪽-끝 채움이라는 뜻이다.
                flow_matches_ladder
                    && without_gap <= available_height() + 0.5
                    && with_gap > available_height() - 0.5
            });
        return prev_tail_fills_the_page;
    }

    false
}

/// 편집 세션의 문단 기준 자리차지 그림이 현재 쪽에 들어가지 않는지 판단한다.
pub(super) fn edited_picture_requires_transition(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    session_edited: bool,
    has_items: bool,
    current_height: f64,
    dpi: f64,
    available_height: impl Fn() -> f64,
) -> bool {
    // [편집 세션] vert=Para 자리차지 그림의 하단 요구를 문단 fit 에 반영한다.
    // 그림은 문단 y + vertical_offset + 높이까지 차지하는데, 줄 기반 fit 은
    // 이를 모른 채 문단을 쪽 말미에 배정하고, 렌더의 쪽-안 클램프(#2032)가
    // 그림을 끌어올려 앞 표에 겹친다(셀 Enter 재현: 그림이 커진 표 하단
    // 위에 얹힘). 한글은 문단 블록째 다음 쪽으로 보낸다.
    if session_edited && has_items {
        let para_float_bottom_req = para
            .controls
            .iter()
            .filter_map(|c| match c {
                Control::Picture(p)
                    if !p.common.treat_as_char
                        && matches!(
                            p.common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        )
                        && matches!(p.common.vert_rel_to, crate::model::shape::VertRelTo::Para) =>
                {
                    Some(hwpunit_to_px(
                        signed_hwpunit(p.common.vertical_offset)
                            .saturating_add(p.common.height.min(i32::MAX as u32) as i32),
                        dpi,
                    ))
                }
                _ => None,
            })
            .fold(0.0_f64, f64::max);
        if para_float_bottom_req > 0.0
            && current_height + para_float_bottom_req.max(fmt.total_height) > available_height()
        {
            return true;
        }
    }

    false
}
