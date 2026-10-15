//! 문단 fit 예산에 필요한 읽기 전용 계산. 페이지 상태나 1회성 flag를 소비하지 않는다.

use super::super::{
    saved_bounds_overlap_current_flow, BODY_BOTTOM_SEAT_PX, SAVED_FRAME_FLOW_DRIFT_TOLERANCE_PX,
};
use super::metrics::FormattedParagraph;
use crate::model::paragraph::Paragraph;

pub(in crate::renderer::typeset) fn layout_drift_safety_px(_paragraphs: &[Paragraph]) -> f64 {
    // Fit uses the accepted line/object footprint. An unrelated margin cannot
    // reject content which fits inside the physical column; any discrepancy
    // with paint must be corrected in the shared footprint or origin instead.
    0.0
}

/// [#7824] 이 문단의 fit 에 쓸 drift 안전마진.
///
/// 저장 줄 없이 rhwp 가 로드 때 합성한 줄(`TAG_IMPLEMENTATION_PROPERTY`)만 가진 문단은
/// 마진 없이 진짜 본문 하한으로 잰다. 그런 문단의 쪽 맞춤은 이미 위 간격까지 담은 계산
/// 높이(`height_for_fit`)로 하므로, 마진은 한/글이 넣는 쪽 마지막 줄만 밀어낸다 —
/// hwp3-sample16-hwp5 21쪽 끝 한 줄 글머리(바닥 967.8px / 본문 971.3px)와 22쪽 끝
/// 다섯 줄 문단의 마지막 줄(968.4px)을 한/글은 그 쪽에 두는데 4px 마진 예산
/// 967.3px 에 걸려 다음 쪽으로 넘어갔다. 한/글 저장 줄이 있는 문단은 종전대로다.
pub(in crate::renderer::typeset) fn paragraph_layout_drift_safety_px(
    paragraphs: &[Paragraph],
    para: &Paragraph,
) -> f64 {
    let synthesized_lines_only = !para.line_segs.is_empty()
        && para.line_segs.iter().all(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        });
    if synthesized_lines_only {
        0.0
    } else {
        layout_drift_safety_px(paragraphs)
    }
}

pub(in crate::renderer::typeset) fn exclusion_probe_height(
    fmt: &FormattedParagraph,
    hwpx_stored_layout: bool,
) -> f64 {
    if hwpx_stored_layout {
        // 배제 영역은 첫 글줄의 실제 높이로 짚는다. 뒤 줄간격은 다음 흐름이
        // 소비할 공간이며, 표 위에 들어가는 잉크를 표 아래로 밀 근거가 아니다.
        fmt.line_heights
            .first()
            .copied()
            .unwrap_or(fmt.height_for_fit)
    } else {
        0.0
    }
}

/// 저장 tail의 하단이 본문 안에 있다는 source 증거가 있을 때, 현재 조판 흐름이 그
/// 하단에 닿는 데 필요한 정확한 차이만 반환한다. 임의 px allowance를 쓰지 않는다.
pub(in crate::renderer::typeset) fn saved_tail_overflow_to_fit(
    bounds: (f64, f64),
    current_height: f64,
    fit_height: f64,
    body_height: f64,
    footnote_height: f64,
) -> Option<f64> {
    let (top, bottom) = bounds;
    // [#5941 f8c784235] 누적 드리프트로 현재 흐름이 저장 tail 을 이미 지나친
    // 경우(cur > bottom)에도, 흐름이 tail 상단에서 드리프트 허용 안에 있으면
    // 그 tail 은 여전히 이 쪽의 source 증거다 — 1490000-201600081 p61: 저장
    // 853.9..868.5(body 876.9 안)인데 흐름 878.3(top+24.4)이라 overlap 이 깨져
    // 꼬리 한 줄이 단독 쪽으로 밀렸다(304→312 의 대표 기전; 부모 r37 은 고정
    // 20px 허용치로 덮던 형상). 허용 폭은 #5822 와 같은 드리프트 상수를 쓰고,
    // top == 0 은 쪽-시작 vpos 센티널이라 드리프트 갈래에서 제외한다(#6027).
    // 드리프트 갈래는 **흐름이 이미 body 를 넘긴 상태**(cur > body)에서만 —
    // 흐름이 body 안이면(잔여가 몇 px 라도) tail 의 쪽 배정은 일반 fit 의 소관이고,
    // 한글도 그때는 tail 을 다음 쪽으로 넘긴다(task1725 국제고속선기준 242쪽 핀:
    // cur 1006.1 < body 1009.1 인데 grant 를 주면 241 로 압축). 흐름이 body 를
    // 이미 넘긴 뒤(직전 문단이 넘겨 쓴 상태)의 tail 만 저장 bot 까지의 정확한
    // 차이로 구제한다 — 대표 p61: cur 878.3 > body 876.9. 각주 실가용도 함께
    // 요구해 각주 쪽의 과대 구제를 막는다.
    //
    // [#5941 잔존] `cur > body` 하나만으로는 잔존 회귀 42건이 안 열린다. 거부되는
    // 세 형상은 **다른 조건은 전부 통과**하고 이 하나에만 걸린다(실측 `body − cur`
    // = 85.4 · 15.6 · 32.6). 반면 이 조건을 넣게 만든 반례(`task1725` 국제고속선기준
    // 242쪽 핀)는 **흐름이 body 바닥에 정확히 앉아 있다**(`body − cur == 0.0`;
    // 넣을 당시 기록은 1006.1 < 1009.1 로 **값이 그 뒤 움직였다**).
    //
    // 그래서 "흐름이 body 안이면 무조건 거부" 대신 **바닥에 앉은 형상만** 거부한다.
    // 여유가 한 줄 규모 이상 남았으면 그 tail 은 아직 이 쪽의 source 증거다.
    let flow_seated_at_body_bottom =
        current_height <= body_height && body_height - current_height <= BODY_BOTTOM_SEAT_PX;
    let drift_reaches_saved_tail = top > 0.0
        && current_height > bottom
        && !flow_seated_at_body_bottom
        && current_height - top <= SAVED_FRAME_FLOW_DRIFT_TOLERANCE_PX
        && bottom <= body_height - footnote_height;
    (top >= 0.0
        && bottom <= body_height
        && (saved_bounds_overlap_current_flow(bounds, current_height) || drift_reaches_saved_tail))
        .then(|| (current_height + fit_height - bottom).max(0.0))
}
