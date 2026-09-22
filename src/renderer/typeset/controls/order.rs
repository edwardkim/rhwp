//! 소유 문단의 컨트롤 배치 순서와 첫/마지막 표 조회.
//! 문서 배열을 바꾸지 않고 기존 정렬 인덱스만 반환한다. 페이지·배치 상태는 읽거나 쓰지 않는다.

use super::super::paragraph::metrics::FormattedParagraph;
use super::super::{is_para_topbottom_float, para_has_non_whitespace_text, signed_hwpunit};
use super::tac_flow::TacFlowQuery;
use crate::model::{control::Control, paragraph::Paragraph};

pub(in crate::renderer::typeset) struct ControlPlacementOrder {
    pub ctrl_order: Vec<usize>,
    pub first_placed_table: Option<usize>,
    pub last_placed_table: Option<usize>,
}

/// 세로 오프셋 키가 꺼져도 기존 TAC/비-TAC 보조 키는 유지한다.
/// 같은 키의 선언 순서는 sort_by_key의 안정 정렬로 보존한다.
pub(in crate::renderer::typeset) fn for_paragraph(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    flow: TacFlowQuery<'_>,
) -> ControlPlacementOrder {
    // 각 컨트롤에 대해 format → fits → place/split
    // [참고2 순서 역전 fix] 빈 host 문단의 para-relative float 표(비-TAC,
    // wrap=위아래, vert=문단)는 흐름과 무관하게 vertical_offset 위치에 배치되는
    // out-of-flow 개체다. 빈 host 에서는 para.controls 배열 순서가 시각적 위·아래
    // 순서와 다를 수 있어 vertical_offset 오름차순 안정정렬을 유지한다(#986/#1088).
    //
    // [Issue #1510] 실제 비공백 텍스트가 있는 host 문단은 한컴이 문서/control 순서와
    // 선언된 절대 위치를 함께 보존한다. 여기서 vertical_offset 순으로 재정렬하면
    // 제목 텍스트와 co-anchored float 표의 순서가 뒤집히므로 정렬 대상에서 제외한다.
    // 공백-only host 는 기존 TopAndBottom empty/float 흐름을 유지한다(#157).
    // [Issue #1639] 빈 host 라도 para-relative float 표 중 음수 vertical_offset 이
    // 하나라도 있으면, 아래 vertical_offset 오름차순 정렬이 음수 표를 양수/0 형제
    // 앞으로 끌어와 문서/배열 순서를 역전시킨다(설명 표가 본문 표 뒤로 밀리는 실문서
    // 회귀). 한컴은 음수가 섞이면 표를 문서/앵커 순서대로 배치하므로, 음수 혼재
    // 빈 host 는 재정렬을 끄고 배열 순서를 보존한다. 양수 전용 빈 host 의
    // vertical_offset 재정렬(#986/#1088)은 그대로 유지한다.
    // 경계: `signed_hwpunit < 0` 인 음수만 트리거하며, offset == 0 은 음수가 아니므로
    // 양수와 함께 정렬을 유지한다(0/양수=정렬 ON, 음수 혼재=정렬 OFF).
    let has_negative_para_float = para.controls.iter().any(|ctrl| {
        matches!(
            ctrl,
            Control::Table(t)
                if is_para_topbottom_float(&t.common)
                    && signed_hwpunit(t.common.vertical_offset) < 0
        )
    });
    // [#2287 후속/1.hwpx p58] 문단 내부 저장 vpos 리셋(ls[k] vpos<=0, 직전
    // vpos>5000)이 있는 host 는 컨트롤이 서로 다른 쪽의 저장 줄에 앉는
    // 구조 — v_off 오름차순 정렬(#986/#1088)이 저장 줄 순서를 뒤집으면
    // (1.hwpx pi=322: TAC(v_off 0)가 자리차지(v_off 1768) 앞으로) 리셋
    // 경계 배치가 무너져 두 표가 같은 쪽에 겹친다. #1639 음수-혼재와
    // 동일하게 정렬을 끄고 배열(저장) 순서를 보존한다.
    let has_mid_para_vpos_reset = para.line_segs.windows(2).any(|w| {
        (w[0].tag | w[1].tag) & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && w[1].vertical_pos <= 0
            && w[0].vertical_pos > 5000
    });
    // [#5807] 자리차지(양수 v_off) 표와 TAC 표가 한 host 에 co-anchored 되면
    // 아래 정렬 키가 TAC 에 0 을 주어 TAC 가 float **앞**으로 온다. 한글의 실제
    // 배치는 선언 위치의 겹침 여부로 갈린다:
    // - float v_off 가 TAC 호스트 줄 높이보다 **작으면**(겹침) float 가 그 자리를
    //   차지하고 TAC 줄이 아래로 밀린다 — float 먼저 (1880690: v_off 937 <
    //   TAC 줄 28024, 뒤집히면 2쪽 354.6px 넘침. 저장 배열 순서·TAC 저장 줄
    //   vpos 12924 도 float 먼저).
    // - float v_off 가 TAC 줄 높이 **이상이면**(비겹침) TAC 는 문단 상단에
    //   남는다 — TAC 먼저 (rowbreak-problem-pages s1 p28: v_off 9188 ≥ TAC 줄
    //   8041, 기존 정렬이 이미 한글 18쪽과 일치 — #1488 핀).
    // 겹침 케이스만 #1639/#2287 과 같이 정렬을 끄고 배열(저장) 순서를 보존한다.
    // v_off 0/음수 float 와의 혼재(tiebreak 로 float 앞세움)는 종전 유지.
    let tac_host_line_height_hu = para
        .controls
        .iter()
        .filter_map(|c| match c {
            Control::Table(t) if flow.is_effective_tac_table(para, t, fmt) => {
                // 소속 줄 매칭이 실패하는 단일 줄 host 는 ls[0] 이 곧 TAC 줄이다
                // (1880690: ls[0] lh=28024 = 표높이 27744+바깥여백).
                let li = flow.tac_table_line_index(para, t, fmt).unwrap_or(0);
                para.line_segs.get(li).map(|ls| ls.line_height)
            }
            _ => None,
        })
        .max()
        .unwrap_or(0);
    // [#6879] `v_off` 의 기준점은 문단 상단이 아니라 **앵커 줄**(그 개체의 제어
    // 문자가 실린 저장 줄)이다. 문단 상단 기준으로 견주면 TAC 줄 **뒤**에 앵커된
    // float 이 "겹침"으로 오판되어 TAC 앞으로 나가고, TAC 라벨이 흐름 끝까지
    // 밀린다 (156767332 7쪽 pi=73: TAC 줄0 lh 3580 · float 앵커 줄1 vpos 4060 ·
    // v_off 2512 → 문단 상단 기준 2512 < 3580 "겹침"이지만 앵커 기준 6572 ≥ 3580
    // 으로 비겹침이고, 한글도 TAC 라벨을 쪽 상단에 둔다).
    //
    // `stored_float_anchor_offset_hu` 는 앵커가 첫 줄이거나 저장 줄이 개체 아래로
    // 가는 형상이면 0 을 돌려주므로, `#5807` 의 두 핀은 값이 그대로다
    // (1880690: 앵커 줄0 → 937 < 28024 겹침 유지 / s1 p28: 9188 ≥ 8041 비겹침 유지).
    let has_tac_overlapped_by_positive_float = tac_host_line_height_hu > 0
        && para.controls.iter().enumerate().any(|(ctrl_index, c)| {
            matches!(c, Control::Table(t)
            if is_para_topbottom_float(&t.common)
                && {
                    let v_off = signed_hwpunit(t.common.vertical_offset);
                    let anchor_top = crate::renderer::layout::stored_float_anchor_offset_hu(
                        para, t, ctrl_index,
                    );
                    v_off > 0 && anchor_top.saturating_add(v_off) < tac_host_line_height_hu
                })
        });
    let should_sort_para_float_tables = !para_has_non_whitespace_text(para)
        && !has_negative_para_float
        && !has_mid_para_vpos_reset
        && !has_tac_overlapped_by_positive_float;
    let float_table_voffset = |ctrl: &Control| -> i32 {
        match ctrl {
            Control::Table(t)
                if should_sort_para_float_tables && is_para_topbottom_float(&t.common) =>
            {
                t.common.vertical_offset as i32
            }
            _ => 0,
        }
    };
    let table_flow_tiebreak = |ctrl: &Control| -> u8 {
        match ctrl {
            Control::Table(t) if !flow.is_effective_tac_table(para, t, fmt) => 0,
            Control::Table(t) if flow.is_effective_tac_table(para, t, fmt) => 1,
            _ => 1,
        }
    };
    let mut ctrl_order: Vec<usize> = (0..para.controls.len()).collect();
    ctrl_order.sort_by_key(|&i| {
        (
            float_table_voffset(&para.controls[i]),
            table_flow_tiebreak(&para.controls[i]),
        )
    });
    // is_first_table/is_last_table 는 배열순서가 아닌 "놓이는 순서(ctrl_order)"
    // 기준으로 잡아, pre/post 텍스트와 spacing 이 실제 배치 첫/마지막 표에 붙도록 한다.
    let first_placed_table = ctrl_order
        .iter()
        .copied()
        .find(|&i| matches!(para.controls[i], Control::Table(_)));
    let last_placed_table = ctrl_order
        .iter()
        .copied()
        .rev()
        .find(|&i| matches!(para.controls[i], Control::Table(_)));

    ControlPlacementOrder {
        ctrl_order,
        first_placed_table,
        last_placed_table,
    }
}
