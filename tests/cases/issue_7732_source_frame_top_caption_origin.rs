//! [#7732] 저장 첫 프레임 경로의 위 캡션 표 원점 — 캡션을 원점에 두 번 더하지 않는다.
//!
//! 쪽 끝에서 나뉘는 쪽 나눔 표의 첫 조각이 저장 첫 프레임(`stored_complete_extent`·첫 조각 빈 밴드)
//! 경로로 확정되고 앞선 배치 계획이 없으면, 조판이 원점을 새로 만든다. 예산(프레임 = 하단 − 원점 −
//! 캡션), 확정(캡션 + 조각 높이)과 그리기(표 = 원점 + 캡션 + 간격)는 모두 원점을 **위 캡션을 포함한
//! 개체 상자의 위끝**으로 쓴다. 그 원점에 캡션을 넣으면 위 캡션 표가 캡션 높이만큼 내려 그려진다.
//!
//! 입력: `samples/issue1937/top_caption_rowbreak_p20.hwp` — 공공 보고서
//! `samples/issue1937_rowbreak_footnote_overpagination.hwp`(50쪽)의 문단 126–145(18–21쪽)만 남겨
//! 한/글 2024 로 다시 저장한 5쪽 문서(저장 줄·표 첫 프레임은 한/글이 다시 썼다). 정본은 같은 저장본의
//! 한/글 2024 Print `pdf/issue1937/top_caption_rowbreak_p20-2024.pdf` 다. 3쪽 끝 표 pi=18 은 위 캡션
//! «< 부담금 개별법령상 납부의무 완화 규정 종합 >» 을 달고 쪽 끝에서 나뉜다. 정본은 표 위 괘선
//! 834.1px · 첫 조각 아래 괘선 1005.0px 다. 수정 전 rhwp 는 표를 캡션 높이만큼 내려 위 괘선을
//! 859.6px 에 그렸다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue1937/top_caption_rowbreak_p20.hwp";
const PAGE: u32 = 2;
const HOST_PARA: usize = 18;
const CAPTION: &str = "개별법령상";

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn host_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(HOST_PARA)) {
        return Some(node);
    }
    node.children.iter().find_map(host_table)
}

/// 표 노드 밖에서 `needle` 을 담은 글줄의 (위, 아래).
fn line_outside_tables(node: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(needle) {
        return Some((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    node.children
        .iter()
        .find_map(|child| line_outside_tables(child, needle))
}

#[test]
fn stored_first_frame_top_caption_table_is_drawn_once_below_its_caption() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse");
    assert_eq!(core.page_count(), 5, "한/글 2024 Print 정본 쪽수");
    let pages = core.dump_page_items_json(None);
    let body = &pages.as_array().unwrap()[PAGE as usize]["bodyArea"];
    let body_bottom = body["y"].as_f64().unwrap() + body["height"].as_f64().unwrap();

    let tree = core.build_page_render_tree(PAGE).expect("page 3");
    let table = host_table(&tree.root).expect("3쪽 표 pi=18 첫 조각");
    let (table_top, table_bottom) = (table.bbox.y, table.bbox.y + table.bbox.height);
    let (caption_top, caption_bottom) =
        line_outside_tables(&tree.root, CAPTION).expect("3쪽 위 캡션 «… 개별법령상 …»");

    // 캡션은 표 바로 위 한 번: 캡션 줄 아래 ≤ 표 위, 그 사이는 캡션 간격 한 칸 안이다.
    assert!(
        caption_bottom <= table_top + 0.5 && table_top - caption_bottom <= 12.0,
        "캡션 줄({caption_top:.1}–{caption_bottom:.1}) 바로 아래에 표({table_top:.1})"
    );
    // 첫 조각은 본문 안에 든다.
    assert!(
        table_bottom <= body_bottom + 0.5,
        "표 첫 조각 아래({table_bottom:.1})가 본문 아래({body_bottom:.1}) 안"
    );
    // 3쪽은 Visual Sweep 판정값(raw 실루엣) Native 99.90% · fresh WASM 99.87%로 정본과 맞고 직접
    // 대조했다 — 정본 표 위 괘선 834.1px · 첫 조각 아래 괘선 1005.0px(96dpi)을 ±2px 로 고정한다.
    assert!(
        (table_top - 834.1).abs() <= 2.0,
        "표 위 괘선 {table_top:.1} — 정본 834.1 ±2"
    );
    assert!(
        (table_bottom - 1005.0).abs() <= 2.0,
        "표 첫 조각 아래 괘선 {table_bottom:.1} — 정본 1005.0 ±2"
    );
}
