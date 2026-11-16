//! 관계형·고정형 — [#8948] 같은 문단 블록 표가 측정 높이로 자라 넘친 뒤의 쪽 흐름.
//! 주석 갱신: 2026-11-11 20:10 KST (새 파일)
//!
//! 입력: 공개 결재문서 36481164(한컴오피스 2020 저장) 원본. 1쪽 `pi=2` 는 글자처럼 취급 5×7 표와
//! 문단 기준 글 위아래 15×30 블록 표(세로 오프셋 9052HU, 선언 높이 45302HU)를 함께 품는다. 블록 표는
//! 현재 글자 모양으로 재면 선언보다 크다. 저장 사다리는 뒤 문단 `pi=3` 을 선언 높이 기준의 표 아래끝
//! (62676HU)에 두고, `pi=4` 의 「재무관 귀하」 상자(글자처럼 취급 2×1 표)를 1쪽에, `pi=5` 를 저장 2쪽
//! 첫머리(vpos 0)에 둔다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8947/36481164-squeeze-title-cell-2020.pdf`(2쪽) — 한/글은
//! 자란 표 뒤에 상자가 들어가지 않아 상자를 2쪽 맨 위로 넘기고, 「붙임」 문단들을 그 아래에 잇는다.
//! - 고정형: 전체 쪽수 2.
//! - 관계형: 1쪽의 모든 표·글줄은 본문 아래끝 안에 있다.
//! - 관계형: 「재무관 귀하」 상자와 「붙임」 글줄은 2쪽에 있고, 상자가 「붙임」 위에 있다.
//!
//! 반례: 넘침 없이 표 자신의 미세한 측정 차로 이월된 쪽 뒤의 저장 경계는 그대로 쪽을 나눈다 —
//! 기존 `tac_group_page_bottom_overflow::sub_pixel_host_spacing_drift_…`(3쪽). 저장 사다리가 표를
//! 돌아 흐르는 문서(36374873 야간방호일지: 뒤 문단 둘째 줄이 표의 저장 아래끝 67877HU 에서 시작)는
//! 블록 표 흐름 끝 하한을 쓰지 않는다 — 기존 `issue_4599_ladder_band_double_count` 와 코퍼스 A/B
//! (출력 불변)로 보호한다.
//!
//! 측정(2026-11-11, 측정 코드 02511ea855(#8947 + #8948), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
//! raw = `silhouette_raw_match_percent`):
//! - Native: 2/2쪽, p1 98.09 · p2 97.47, 전쪽 최저 97.47.
//! - fresh WASM: 2/2쪽, p1 98.24 · p2 97.82, 전쪽 최저 97.82.
//! - 수정 전(#8947 만, 6cb75d5a52 와 같은 트리 a4c120eac1): Native p1 88.05 · p2 15.79, fresh WASM p1 88.00 · p2 16.04.
//! - 증거: `mydocs/pr/assets/issue_8948/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8947/36481164-squeeze-title-cell.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn text_of(node: &RenderNode) -> String {
    let mut all = Vec::new();
    walk(node, &mut all);
    all.iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn find_box<'a>(
    nodes: &[&'a RenderNode],
    pred: impl Fn(&RenderNode) -> bool,
) -> Option<BoundingBox> {
    nodes.iter().find(|n| pred(n)).map(|n| n.bbox)
}

#[test]
fn grown_block_table_pushes_the_closing_box_to_the_next_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open: {e:?}"));
    assert_eq!(core.page_count(), 2, "한컴 engine 2020 Print PDF 는 2쪽");

    // 1쪽: 표와 글줄이 본문 아래끝을 넘지 않는다.
    let page1 = core.build_page_render_tree(0).expect("1쪽");
    let mut nodes1 = Vec::new();
    walk(&page1.root, &mut nodes1);
    let body1 = find_box(&nodes1, |n| {
        matches!(n.node_type, RenderNodeType::Body { .. })
    })
    .expect("1쪽 본문");
    let body_bottom = body1.y + body1.height;
    for node in &nodes1 {
        if matches!(
            node.node_type,
            RenderNodeType::Table(_) | RenderNodeType::TextLine(_)
        ) {
            assert!(
                node.bbox.y + node.bbox.height <= body_bottom + TOLERANCE_PX,
                "1쪽 항목 아래끝 {:.2} 이 본문 아래끝 {body_bottom:.2} 을 넘는다: {:?}",
                node.bbox.y + node.bbox.height,
                text_of(node).chars().take(20).collect::<String>()
            );
        }
    }
    assert!(
        !text_of(&page1.root).contains("재무관 귀하"),
        "「재무관 귀하」 상자는 1쪽에 남지 않아야 한다"
    );

    // 2쪽: 상자가 「붙임」 위에 있다.
    let page2 = core.build_page_render_tree(1).expect("2쪽");
    let mut nodes2 = Vec::new();
    walk(&page2.root, &mut nodes2);
    let closing_box = find_box(&nodes2, |n| {
        matches!(n.node_type, RenderNodeType::Table(_)) && text_of(n).contains("재무관 귀하")
    })
    .expect("2쪽 「재무관 귀하」 상자");
    let attachment = find_box(&nodes2, |n| {
        matches!(n.node_type, RenderNodeType::TextLine(_))
            && text_of(n).trim_start().starts_with("붙임")
    })
    .expect("2쪽 「붙임」 글줄");
    assert!(
        closing_box.y + closing_box.height <= attachment.y + TOLERANCE_PX,
        "상자 아래끝 {:.2} 이 「붙임」 위끝 {:.2} 보다 아래다",
        closing_box.y + closing_box.height,
        attachment.y
    );
}
