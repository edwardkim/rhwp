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
//! 1. 조각 상자 상단 = 본문 상단 + 표 `outer_margin_top`
//! 2. 마지막이 아니고 쪽 상단에서 시작하는 조각의 상자 하단
//!    = 본문 하단 − `outer_margin_bottom` − 100HU
//! 3. 칸 내용은 그 상자 안에서 칸 `valign`(이 문서는 Center)으로 배치
//!
//! 위 표의 정본 좌표는 PDF 원좌표다. 한/글 PDF 는 A4 쪽을 595×841pt 로 내 내용이 0.99895 배
//! 축소되므로, 그 척도를 걷으면 2쪽 상자 하단은 1043.6 이다. 본문 하단 1046.93 −
//! 바깥 여백 1.88 − 100HU(1.33) = 1043.72 와 맞는다. 100HU 는 여백·테두리·쪽 기하
//! 돌연변이 7종에서 흔들리지 않은 상수다(#7095).
//!
//! 1·2 는 이 시험이 잠근다. 2 를 페이지네이터 예산 없이 렌더러에만 넣으면 내용이 칸 밖으로
//! 밀린다(PR #7098 실측 overfill overflow_cell +4줄). 그래서 같은 술어로 예산에서도 뺀다.
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

    // 비끝 조각 상자 하단 = 본문 하단 1046.93 − 바깥 아래 여백 1.88 − 100HU(1.33) = 1043.72.
    // 정본 1042.54 는 PDF 쪽 척도(0.99895)가 걸린 값이고 걷으면 1043.6 이다.
    // 수정 전에는 내용 컷에서 끝나 2쪽 1025.9 · 3쪽 1006.7 이었다.
    for (label, frag) in [("2쪽", outer), ("3쪽", outer_p3)] {
        let bottom = frag.y + frag.height;
        assert!(
            (bottom - 1043.72).abs() < 1.0,
            "#7095: {label} 비끝 조각 상자 하단은 본문 하단 − 바깥여백 − 100HU(1043.72)여야 한다: {bottom:.2}"
        );
    }
}

#[test]
fn issue_7095_last_fragment_and_downstream_contracts_hold() {
    let core = load();
    // 마지막 조각(10쪽)은 쪽 상자로 늘리지 않는다 — 정본도 내용에 맞춰 줄어든다(정본 아래 920.27).
    // 비끝 조각 상자 아래(1043.72)보다 확실히 위에서 끝나야 한다.
    let last_frag = fragment_box(&page_nodes(&core, 9));
    let last_bottom = last_frag.y + last_frag.height;
    assert!(
        (last_frag.y - 47.2).abs() < 0.5 && last_bottom < 1000.0,
        "#7095: 마지막 조각도 같은 상단(47.2)이고 내용에 맞춰 줄어든다: y={:.2} bottom={:.1}",
        last_frag.y,
        last_bottom
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

fn load_sample(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

/// `para_index` 가 같은 표 노드의 상자들.
fn tables_of_para(nodes: &[RenderNode], para: usize, rows: u16, cols: u16) -> Vec<BoundingBox> {
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table(t)
                if t.para_index == Some(para) && t.row_count == rows && t.col_count == cols =>
            {
                Some(n.bbox)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn issue_7095_first_fragment_starting_at_page_top_is_pinned_too() {
    // 30269 10쪽은 표(pi136)의 **첫** 조각인데 쪽 상단에서 시작한다. 정본 상자 아래는
    // 본문 아래 1028.01 − 바깥 아래 여백 283HU(3.77) − 100HU(1.33) = 1022.91 이다.
    // 수정 전에는 내용 행 높이(마지막 줄 뒤 줄간격 포함)로 끝나 1028.3, 이어짐 조건으로만
    // 고정하면 1032.1 로 정본 상자를 9px 넘었다.
    let core = load_sample("samples/issue6023/30269_reform_recommendation.hwp");
    let boxes = tables_of_para(&page_nodes(&core, 9), 136, 1, 1);
    let frag = boxes.first().expect("30269 10쪽 조각 표");
    let bottom = frag.y + frag.height;
    assert!(
        (frag.y - 98.27).abs() < 1.0 && (bottom - 1022.91).abs() < 1.0,
        "#7095: 30269 10쪽 조각 상자는 위 98.27 · 아래 1022.91 이어야 한다: y={:.2} bottom={:.2}",
        frag.y,
        bottom
    );
    assert_eq!(
        core.page_count(),
        22,
        "#7095: 30269 쪽수는 정본과 같은 22 여야 한다"
    );
}
