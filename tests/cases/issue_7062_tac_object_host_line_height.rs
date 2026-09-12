//! [#7062] 저장 LINE_SEG 없이 TAC(글자처럼 취급) 개체만 앵커한 문단의 호스트 줄이
//! 400HU(5.3px) 로 붕괴해, 뒤 내용이 개체 한가운데 겹쳐 그려진다.
//!
//! `156060125_(금융위)보도자료…hwp` 2쪽: 29997HU(400.0px) 도해 그림이 빈 앵커 문단에
//! 글자처럼 붙어 있다. 렌더 경로(`paragraph_layout`)만 `#2287` TAC 줄 메트릭 합성을
//! 쓰지 않아 호스트 줄이 400HU 고정 advance(5.3px)로 떨어졌고, 뒤 표가 553.4px —
//! 그림(548.1 .. 948.1) 안쪽 — 에서 시작했다.
//!
//! 정답지: 한컴 engine 2020 출력을 같은 96dpi 래스터로 겹쳐 잰다(job
//! `94f14dc5-422a-462a-b4e6-810ef36ff98d`, 10쪽, `Hancom PDF 1.3.0.550`). 2쪽 잉크 기준
//! 정본은 **도해 잉크 → `※` 상자 360px** 이고, 이 수정만으로는 350px 이다 —
//! 남는 leading 10px 은 `#7079` 가 채운다. 수정 전에는 뒤 표가 그림 한가운데였다.
//!
//! `#6928` 코멘트(2026-09-09)의 "그림 상단 대비 +416.8" 은 PDF glyph box 좌표와 rhwp 줄
//! 상단을 섞어 잰 값이라 이 시험의 기준으로 쓰지 않는다 — 그 프레임에는 이 칸 위쪽에서
//! 이미 생기는 −20px 오프셋이 함께 들어 있다.
//!
//! 반례(같은 문서 통제군): 1쪽 제목 표와 본문 줄, 그리고 문서 전체 쪽수(10)는
//! 이 변경으로 움직이지 않는다 — TAC 개체가 없는 빈 문단은 종전 경로 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

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

/// 2쪽 도해 그림(400.0px) 노드의 bbox.
fn diagram_bbox(nodes: &[RenderNode]) -> rhwp::renderer::render_tree::BoundingBox {
    nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Image(_) if (n.bbox.height - 400.0).abs() < 1.0 => Some(n.bbox),
            _ => None,
        })
        .expect("2쪽 400px 도해 그림")
}

#[test]
fn issue_7062_tac_diagram_host_line_carries_object_height() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = diagram_bbox(&nodes);

    // 그림과 같은 y 에서 시작하는 호스트 줄 — 개체 높이를 알아야 한다.
    // 결함 시 5.3px(400HU 고정 advance).
    let host = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextLine(_) if (n.bbox.y - image.y).abs() < 0.5 => Some(n.bbox),
            _ => None,
        })
        .max_by(|a, b| a.height.total_cmp(&b.height))
        .expect("도해 그림의 호스트 줄");
    assert!(
        (host.height - image.height).abs() < 1.0,
        "#7062: TAC 개체 호스트 줄은 개체 높이(400.0px)를 담아야 한다 — 결함 시 5.3px: h={:.1}",
        host.height
    );
}

#[test]
fn issue_7062_paragraph_after_tac_diagram_clears_the_object() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = diagram_bbox(&nodes);

    let after = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.contains("즉시 추진 가능한 시급한 과제부터") => {
                Some(n.bbox)
            }
            _ => None,
        })
        .expect("'※ 즉시 추진 …' 런");

    // 겹침 자체가 결함이다 — 결함 시 558.7 (그림 548.1..948.1 한가운데).
    assert!(
        after.y >= image.y + image.height - 1.0,
        "#7062: 그림 뒤 문단은 그림 아래(≥{:.1})에서 시작해야 한다 — 결함 시 558.7: y={:.1}",
        image.y + image.height,
        after.y
    );

    // 정본 상대값: 도해 잉크 → `※` 상자 360px. 이 수정만으로는 leading 10px 이 빠진
    // 350px 이고, 결함(겹침)과는 자릿수가 다르다. leading 은 `#7079` 가 맡는다.
    let table = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if n.bbox.y > image.y + 100.0 => Some(n.bbox),
            _ => None,
        })
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("그림 뒤 표");
    let advance = table.y - image.y;
    assert!(
        (390.0..=415.0).contains(&advance),
        "#7062: 그림 상단 → 뒤 표는 개체 높이(400) 근처여야 한다 — 결함 시 5.3: {advance:.1}"
    );
}

#[test]
fn issue_7062_control_group_page1_and_page_count_unchanged() {
    let core = load();
    // 정답지와 같은 10쪽 — TAC 줄 높이 보정이 쪽 경계를 흔들지 않는다.
    assert_eq!(core.page_count(), 10, "#7062 통제군: 쪽수는 10 이어야 한다");

    // 1쪽: TAC 개체가 없는 문단들 — 저장 줄 좌표 그대로여야 한다.
    // 수정 전 바이너리(devel f537df5ea)에서 잰 같은 좌표다.
    let nodes = page_nodes(&core, 0);
    let title = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.contains("보 도 자 료") => Some(n.bbox),
            _ => None,
        })
        .expect("1쪽 제목 런");
    assert!(
        (title.y - 137.9).abs() < 0.5,
        "#7062 통제군: 1쪽 '보 도 자 료' 는 137.9px 그대로여야 한다: y={:.1}",
        title.y
    );
    let contact = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.contains("금융제도팀장") => Some(n.bbox),
            _ => None,
        })
        .expect("1쪽 책임자 런");
    assert!(
        (contact.y - 265.7).abs() < 0.5,
        "#7062 통제군: 1쪽 표 안 줄도 265.7px 불변이어야 한다: y={:.1}",
        contact.y
    );
}
