//! [#7203] 빈 host 자리차지(TopAndBottom) 표의 **저장 앵커 원점**을 한 곳에서 정한다.
//!
//! 종전에는 같은 이름의 게이트가 `typeset.rs` 와 `layout.rs` 에 각각 있었고 산식이 갈렸다.
//!
//! ```text
//!   layout.rs   need = 높이 + 아래여백 − 위여백 ,  윗변 = vpos − 위여백
//!   typeset.rs  need = 높이 + 위여백 + 아래여백 ,  윗변 = vpos
//! ```
//!
//! 대칭 여백(283/283)이면 need 가 `2 × 위여백` 만큼, 윗변이 `위여백` 만큼 갈린다. 그래서
//! 같은 표를 두고 **조판은 저장 앵커를 거부하고 렌더는 수용**하는 구간이 생기고, 둘 다
//! 수용해도 원점이 3.77px 어긋난다. `hwpctl_API_v2.4.hwp` 의 어긋난 자리차지 표는
//! 예외 없이 이 구간에 있었다(사다리 간격이 `높이 + 66 HU`).
//!
//! ## 정본이 정하는 원점
//!
//! `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 의 가로 괘선과 원본 저장 사다리를 대조하면
//!
//! ```text
//!   정본 윗변 = 본문 상단 + (앵커 vpos − 위 바깥여백)      잔차 +0.26 .. +0.74px (9건)
//! ```
//!
//! 즉 저장 `vpos` 는 표 상자의 상단이 아니라 **위 바깥여백 뒤**를 가리킨다. 그러면 앵커
//! 아래로 필요한 공간은 `높이 + 아래여백 − 위여백` 이다. 이 모듈이 그 한 규칙을 갖고,
//! 조판·렌더가 같은 값을 소비한다.

use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::table::Table;
use crate::renderer::hwpunit_to_px;

/// 구현이 합성한 줄은 저장 사다리의 증거가 아니다.
fn stored_seg(para: &Paragraph) -> Option<&LineSeg> {
    para.line_segs
        .iter()
        .find(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
}

fn stored_vpos(para: &Paragraph) -> Option<i64> {
    stored_seg(para).map(|seg| i64::from(seg.vertical_pos))
}

/// 저장 host 기준의 표 윗변 offset 과 흐름 점유 끝(HWPUNIT).
///
/// 저장 사다리가 무엇을 쟀는지에 따라 `vpos` 의 뜻이 갈린다.
///
/// - 다음 앵커가 `높이 + 양쪽 여백`만큼 **정확히** 전진하면 `vpos` 는 **바깥 여백 상자의
///   위끝**이다. 표 자신의 윗변은 거기서 `outMargin.top` 만큼 아래고, 점유 끝은
///   `vpos + 상자 높이` 그대로다(`om_top + 높이 + om_bottom` 과 같다).
/// - 그 밖의 수용된 저장 앵커는 위여백 **뒤**를 가리키므로 윗변이 `vpos − om_top` 이다.
///
/// [#7203] 앞 갈래는 종전에 offset 0 이었다 — paint 와 예약을 맞추려는 값이었지 기하가
/// 아니었다. `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 전수 대조가 두 갈래를 갈라 준다.
///
/// ```text
///   사다리 advance == 높이+위+아래  (18건)  정본 − rhwp = +3.41px  ← 위여백 한 개만큼 위였다
///   사다리 advance == 높이 + 66HU   (16건)  정본 − rhwp = +0.55px  ← 이미 맞다
/// ```
///
/// 두 코호트는 같은 문서·같은 선언 여백(283HU)이고 사다리 간격만 다르다. 점유 끝은
/// 양쪽 모두 바뀌지 않으므로 예약 높이는 그대로다.
pub(crate) fn stored_topbottom_object_span(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    table: &Table,
) -> (i64, i64) {
    if let Some(outer_box_height) = stored_topbottom_flow_advance_hu(para, next_para, table) {
        // 사방 균등일 때만 `vpos` 를 상자 위끝으로 읽는다. 저장소의 좁은 술어 셋
        // (`#6378` · `#3820 Stage 120` · `native_empty_host_physical_outer_box_paint_inset`)
        // 이 모두 같은 조건을 쓰고, 정본도 같은 말을 한다 —
        // `76076_regulatory_analysis.hwp` pi=323·324 는 (**0**/**0**/566/566) 이고
        // 한/글이 여백을 싣지 않는다(정본 괘선 400.52 로 확인).
        let uniform = table.outer_margin_top == table.outer_margin_bottom
            && table.outer_margin_top == table.outer_margin_left
            && table.outer_margin_top == table.outer_margin_right;
        let top = if uniform {
            i64::from(table.outer_margin_top)
        } else {
            0
        };
        (top, outer_box_height)
    } else {
        let top = -i64::from(table.outer_margin_top);
        (
            top,
            top + i64::from(table.common.height) + i64::from(table.outer_margin_bottom),
        )
    }
}

/// 저장 사다리가 증명한 전체 흐름 상자의 advance. 단순 테두리 원점은
/// 이 높이를 증명하지 않으므로 None을 반환한다. 조판 원점 선택과 최종 flow 소비가
/// 같은 근거를 사용하며, paint 좌표 차이를 예약 높이로 오인하지 않는다.
pub(crate) fn stored_topbottom_flow_advance_hu(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    table: &Table,
) -> Option<i64> {
    let outer_box_height = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    let stored_outer_box = stored_vpos(para)
        .zip(next_para.and_then(stored_vpos))
        .is_some_and(|(current, next)| next - current == outer_box_height);
    stored_outer_box.then_some(outer_box_height)
}

/// 다음 저장 `vpos` 사다리가 이 개체가 점유할 높이를 실제로 비우는가 [#3925].
///
/// 비우지 않는 host 의 raw `vpos` 를 물리 anchor 로 읽으면 개체가 사다리보다 훨씬 아래에
/// 놓여 쪽 소비가 부풀고 뒤 문단이 다음 쪽으로 밀린다 — `36324768_결재문서본문.hwpx`
/// pi=11 은 host `lh` 가 14.7px 인데 표는 253.1px 라 쪽 소비가 187px 늘어 2→3쪽이 됐다.
/// host 줄 높이만으로는 #3738 표적을 악화시키므로 다음 문단과의 `vpos` 간격만 증거로 쓴다.
pub(crate) fn stored_ladder_leaves_object_room(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    table: &Table,
) -> bool {
    let (_, occupied_bottom) = stored_topbottom_object_span(para, next_para, table);
    let need = occupied_bottom.max(0);
    match (stored_vpos(para), next_para.and_then(stored_vpos)) {
        (Some(current), Some(next)) => next - current >= need,
        _ => false,
    }
}

/// 저장 앵커가 가리키는 표 상자 윗변 — 단 상단 기준 px. 쓸 수 없으면 `None`.
///
/// `available_px` 는 단(칸) 높이다. 반환값은 그 단 상단으로부터의 offset 이라
/// 조판(상대 lane)과 렌더(절대 lane)가 각자의 기준점만 더하면 된다.
pub(crate) fn stored_single_topbottom_top_px(
    para: &Paragraph,
    next_para: Option<&Paragraph>,
    table: &Table,
    available_px: f64,
    dpi: f64,
) -> Option<f64> {
    // 문단 기준 세로 오프셋이 걸린 개체는 상자 윗변이 `앵커 + 오프셋` 이라 이 산식이
    // 성립하지 않는다. 오프셋을 더해 읽을 근거(정본 실측)가 없으므로 흐름 배치에 맡긴다.
    // `rowbreak-problem-pages.hwp` 구역1 의 `vertical_offset` 152·3019 HU 표가 이 갈래다 —
    // 앵커를 그대로 믿으면 본문이 쪽 밖 480px 까지 밀린다.
    if table.common.vertical_offset != 0 {
        return None;
    }
    let current = stored_vpos(para)?;
    let next = next_para.and_then(stored_vpos)?;
    // vpos 가 다음 문단에서 되감기면 이 표는 다음 물리 쪽의 첫 anchor 일 수 있다.
    // raw vpos 를 현재 쪽 좌표로 강제하면 p14 그림 8처럼 상단 그림이 하단에 떨어진다.
    if next <= current {
        return None;
    }
    if !stored_ladder_leaves_object_room(para, next_para, table) {
        return None;
    }
    let (top_offset, _) = stored_topbottom_object_span(para, next_para, table);
    let top = hwpunit_to_px((current + top_offset) as i32, dpi);
    let bottom = top + hwpunit_to_px(table.common.height as i32, dpi);
    // 본문 안에 온전히 드는 앵커만 물리 좌표로 읽는다.
    (top >= -0.5 && bottom <= available_px + 0.5).then_some(top)
}
