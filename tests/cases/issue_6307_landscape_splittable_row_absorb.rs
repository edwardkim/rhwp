//! [Issue #6307] 가로 용지 RowBreak 연속 조각이 행내 분할 가능한 다줄 행을
//! 통짜로 얹어 본문 하한(737.0px)을 25.7px 넘는다.
//!
//! `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` 11쪽(0-기준 10): 잔여 33.4px 에
//! `UnderlineColor` 행(38.2px, 2줄 셀)이 안 들어가는 자리에서 —
//!
//! | | rhwp(수정 전) | rhwp(수정 후) | 한글 2022 PDF |
//! |---|---|---|---|
//! | 경계 행 처리 | 38.2px 통짜 흡수 + 다음 행(20.9px)까지 | 첫 줄만 24.9px 조각 | 첫 줄만 ~27.6px 조각 |
//! | 표 조각 하한 | **763.2px (+25.7px 초과)** | 729.0px | 732.6px |
//! | 11쪽 마지막 행 | `ShadowOffsetX` | `UnderlineColor` 첫 줄 | `UnderlineColor` 첫 줄 |
//!
//! 근인은 #1672 landscape 경계 행 흡수(연속 조각 + 머리행 반복 + tolerance
//! 36/260px)다. #5828 이 같은 높이 행의 연속 흡수만 막아서, 이종 높이(38.2 →
//! 20.9px) 2행 연속 흡수는 통과했다. 행 높이 자체는 한글과 일치(조각 누적
//! rhwp 646.9px vs 한글 647.1px — 기준 PDF 괘선 실측)하므로 경계 결정만 문제다.
//!
//! 고침 2겹: ① 흡수 분기(whole-row·short-row)는 `is_row_splittable` 행에 적용하지
//! 않는다 — 한컴은 이런 행을 본문 하한에서 가른다. ② 그렇게 분할로 돌린 행의
//! 첫 줄 컷(painted 24.8px)을 25px 고아 가드가 0.2px 차로 기각해 행 통째 이월로
//! 퇴행하지 않도록, 남는 밴드(≥25px)가 실측인 경우 컷을 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpctl_ParameterSetID_Item_v1.2.hwp";
/// A4 가로(높이 210mm), 아래 여백 15mm, 머리말·꼬리말 0 — 본문 하한 737.0px.
const BODY_BOTTOM_PX: f64 = 737.0;

#[test]
fn issue_6307_landscape_boundary_row_splits_like_hancom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(10)
        .expect("page 11 render tree");

    let mut table_bottom: f64 = 0.0;
    let mut underline_color: Option<(f64, f64)> = None;
    let mut shadow_offset_x = false;
    walk(
        &page.root,
        &mut table_bottom,
        &mut underline_color,
        &mut shadow_offset_x,
    );

    // 흡수가 얹은 마지막 행(ShadowOffsetX)은 12쪽 몫이다.
    assert!(
        !shadow_offset_x,
        "11쪽에 ShadowOffsetX 행이 남아 있다 — landscape 경계 흡수가 다시 발화했다"
    );
    // 경계 행(UnderlineColor)은 한글처럼 첫 줄 조각으로 11쪽에 남아야 한다.
    // 통째 이월(셀 부재)도, 통째 흡수(38.2px 전체)도 아니다.
    let (y, height) = underline_color
        .expect("11쪽 하단에 UnderlineColor 경계 행의 첫 줄 조각이 있어야 한다 (한글 2022 동일)");
    assert!(
        height < 30.0,
        "UnderlineColor 행이 분할되지 않고 통째(38.2px)로 남았다: y={y:.1} h={height:.1}"
    );
    // 표 조각은 본문 하한 안에서 끝난다 (수정 전 763.2px — 바탕쪽 로고 밴드 침범).
    assert!(
        table_bottom <= BODY_BOTTOM_PX + 0.5,
        "표 조각이 본문 하한을 넘는다: bottom={table_bottom:.1} (한도 {BODY_BOTTOM_PX})"
    );
}

fn walk(
    node: &RenderNode,
    table_bottom: &mut f64,
    underline_color: &mut Option<(f64, f64)>,
    shadow_offset_x: &mut bool,
) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            *table_bottom = table_bottom.max(node.bbox.y + node.bbox.height);
        }
        RenderNodeType::TableCell(_) => {
            let text = collect_text(node);
            if text == "UnderlineColor" {
                *underline_color = Some((node.bbox.y, node.bbox.height));
            }
            if text == "ShadowOffsetX" {
                *shadow_offset_x = true;
            }
        }
        _ => {}
    }
    for child in &node.children {
        walk(child, table_bottom, underline_color, shadow_offset_x);
    }
}

fn collect_text(node: &RenderNode) -> String {
    let mut acc = String::new();
    fn rec(node: &RenderNode, acc: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            acc.push_str(run.text.trim());
        }
        for child in &node.children {
            rec(child, acc);
        }
    }
    rec(node, &mut acc);
    acc
}
