//! [#7095] 쪽을 넘기는 1×1 `RowBreak` 표의 조각 상자가 표의 `outer_margin_top` 을 열지 않아
//! 그 표에 담긴 쪽 전체가 1.9px 위에 있었다.
//!
//! 한컴 engine 2020 정본(`pdf/tac_object_host_line_height-2020.pdf`, job
//! `94f14dc5-422a-462a-b4e6-810ef36ff98d`)을 `pdftocairo -svg` 벡터 좌표로 재면
//! (96dpi 환산, 경로 자신의 `transform` 반영):
//!
//! | 축 | 정본 | 수정 전 |
//! | --- | ---: | ---: |
//! | 2쪽 바깥 표 상자 | 47.15 .. 1042.54 | 45.60 .. 1025.70 |
//! | 3쪽 바깥 표 상자 | 47.15 .. 1042.54 (같다) | 45.60 .. 1006 |
//! | 10쪽(마지막 조각) | 47.15 .. 920.27 (줄어든다) | — |
//! | 2쪽 칸 첫 내용(제목 상자) | 57.06 | 47.2 |
//! | 2쪽 칸 마지막 내용(`※` 상자 하단) | 1032.63 | — |
//!
//! 2쪽 위 여백 9.91 / 아래 여백 9.91 로 **정확히 대칭**이고, 칸 padding 141HU(1.88px)를 뺀
//! 여유 16.0 을 반씩 나눈 예측(57.03 / 1032.66)이 실측과 **0.03px** 로 맞는다. 곧
//!
//! 1. 조각 상자 상단 = 본문 상단 + 표 `outer_margin_top`  ← **이 커밋이 닫는 축**
//! 2. 마지막이 아니고 쪽 상단에서 시작하는 조각의 상자 하단 = 본문 하단 − `outer_margin_bottom`
//! 3. 칸 내용은 그 상자 안에서 칸 `valign`(이 문서는 Center)으로 배치
//!
//! 2·3 은 아직 넣지 않았다. 구현해 보면 정본과 0.03px 로 맞지만, 조각의 실제 내용 높이를
//! 컷 유닛 합으로 대신하면 문서에 따라 과소평가되어 내용이 칸 밖으로 밀린다
//! (`task1718/table_giant_cell_overfill.hwp`: overflow_cell +4줄, textOverlap 0→1).
//! 남은 두 축은 `#7095` 에 그대로 열어 둔다.
//!
//! 반례: 표가 쪽 **중간**에서 시작하는 조각(156645214 19쪽, 표 시작 y≈217)은 늘리지 않는다 —
//! 늘리면 내용이 정본보다 8px 아래로 밀린다(정본 대비 −3 → +5). 마지막 조각도 늘리지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7062/tac_object_host_line_height.hwp";

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn page_nodes(core: &DocumentCore, page_index: u32) -> Vec<RenderNode> {
    let page = core
        .build_page_render_tree(page_index)
        .expect("render tree");
    let mut refs = Vec::new();
    walk(&page.root, &mut refs);
    refs.into_iter().cloned().collect()
}

/// 본문을 담은 바깥 1×1 조각 표(폭 676px).
fn fragment_box(nodes: &[RenderNode]) -> BoundingBox {
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if (n.bbox.width - 676.0).abs() < 2.0 => Some(n.bbox),
            _ => None,
        })
        .max_by(|a, b| a.height.total_cmp(&b.height))
        .expect("바깥 조각 표")
}

#[test]
fn issue_7095_fragment_box_opens_the_table_outer_top_margin() {
    let core = load();
    let outer = fragment_box(&page_nodes(&core, 1));

    // 상단 = 본문 상단 45.35 + outer_margin_top 141HU(1.88) = 47.2 (정본 47.15).
    // 수정 전에는 본문 상단(45.6)에 그대로 붙었다.
    assert!(
        (outer.y - 47.2).abs() < 0.5,
        "#7095: 조각 상자 상단은 본문 상단 + 바깥여백(47.2)이어야 한다 — 수정 전 45.6: {:.2}",
        outer.y
    );

    // 3쪽 조각도 같은 상단이다(같은 표의 다른 조각).
    let outer_p3 = fragment_box(&page_nodes(&core, 2));
    assert!(
        (outer_p3.y - 47.2).abs() < 0.5,
        "#7095: 3쪽 조각 상단도 47.2 여야 한다: {:.2}",
        outer_p3.y
    );

    // 상자 높이는 아직 내용에 맞춘다(축 2 미구현) — 쪽 크기(≈998)로 늘리지 않는다.
    assert!(
        outer.height < 995.0,
        "#7095: 이 커밋은 상자 높이를 바꾸지 않는다: h={:.1}",
        outer.height
    );
}

#[test]
fn issue_7095_last_fragment_and_downstream_contracts_hold() {
    let core = load();
    // 마지막 조각(10쪽)은 늘리지 않는다 — 정본도 내용에 맞춰 줄어든다.
    let last_frag = fragment_box(&page_nodes(&core, 9));
    assert!(
        (last_frag.y - 47.2).abs() < 0.5 && last_frag.height < 400.0,
        "#7095: 마지막 조각도 같은 상단(47.2)이고 내용에 맞춰 줄어든다: y={:.2} h={:.1}",
        last_frag.y,
        last_frag.height
    );

    // #7079 계약: 도해 그림 → 뒤 표 간격 410.0(저장 사다리)은 이 변경과 무관하다.
    let nodes = page_nodes(&core, 1);
    let image = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Image(_) if (n.bbox.height - 400.0).abs() < 1.0 => Some(n.bbox),
            _ => None,
        })
        .expect("도해 그림");
    let table = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if n.bbox.y > image.y + 100.0 => Some(n.bbox),
            _ => None,
        })
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("그림 뒤 표");
    assert!(
        (table.y - image.y - 410.0).abs() < 1.0,
        "#7095: 그림 → 뒤 표 410.0(#7079)은 유지되어야 한다: {:.1}",
        table.y - image.y
    );

    assert_eq!(core.page_count(), 10, "#7095: 쪽수는 10 이어야 한다");
}
