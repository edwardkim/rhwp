//! Issue #1835: TAC 표에서 셀 내용이 저장 높이(common.height)를 크게(>1.5×) 초과하면
//! 비례 축소 대신 내용 높이로 확장한다 (한글 2022 편집기 오라클 정합).
//!
//! fixture: `samples/issue1835_tac_stale_height.hwp` — `samples/hwp_table_test.hwp` 의
//! 4×3 TAC 표 common.height 를 1/1.8(7126→3950HU)로 훼손해 합성(외부 도구 생성/템플릿
//! 값 채움으로 stale 한 문서 재현). 오라클: `pdf/issue1835_tac_stale_height-2022.pdf`
//! (Producer=Hancom PDF) — 표를 내용 높이(~85pt 스팬)로 확장, 후속 문단도 아래로 흐름.
//! 수정 전 rhwp 는 행을 0.55×로 눌러(52.7px) 셀 텍스트가 겹쳤다.
//!
//! 경미한 초과(2%~150%)의 비례 축소(#672, 계획서.hwp 1.32% 면제/의도적 압축 유지)는
//! 불변 — `TAC_SHRINK_MAX_OVERFLOW_RATIO=1.5` 상한만 추가.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1835_tac_stale_height.hwp";

fn load_tree() -> rhwp::renderer::render_tree::PageRenderTree {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));
    doc.build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page 1: {e}"))
}

fn find_table<'a>(node: &'a RenderNode, pi: usize, ci: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.para_index == Some(pi) && t.control_index == Some(ci) {
            return Some(node);
        }
    }
    node.children.iter().find_map(|c| find_table(c, pi, ci))
}

/// needle 포함 TextRun 의 y 들 수집.
fn collect_text_runs_y(node: &RenderNode, needle: &str, out: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            out.push(node.bbox.y);
        }
    }
    for c in &node.children {
        collect_text_runs_y(c, needle, out);
    }
}

/// stale common.height(3950HU=52.7px)로 눌리지 않고 내용 높이(~95px)로 확장.
#[test]
fn tac_stale_height_table_expands_to_content() {
    let tree = load_tree();
    let table = find_table(&tree.root, 3, 0).expect("pi=3 ci=0 4×3 TAC 표");
    assert!(
        table.bbox.height > 85.0,
        "#1835: 표가 stale 높이로 눌림 — height={:.1}px (내용 높이 ~95px 기대)",
        table.bbox.height
    );
}

/// 후속 문단('셀 편집' 불릿)이 확장된 표 하단 아래에서 시작한다 — 한컴 오라클
/// (후속 문단 277.1pt, 표 헤더와 스팬 85.4pt)과 동일 축. 수정 전에는 표가 52.7px
/// 로 눌려 후속 문단이 ~63px 일찍(표 내용과 겹치는 위치에서) 시작했다.
#[test]
fn tac_stale_height_following_paragraph_flows_below_table() {
    let tree = load_tree();
    let table = find_table(&tree.root, 3, 0).expect("pi=3 ci=0 4×3 TAC 표");
    let table_bottom = table.bbox.y + table.bbox.height;
    let mut ys = Vec::new();
    collect_text_runs_y(&tree.root, "셀 편집", &mut ys);
    let next_y = ys
        .iter()
        .copied()
        .filter(|&y| y > table.bbox.y)
        .fold(f64::INFINITY, f64::min);
    assert!(next_y.is_finite(), "#1835: 후속 '셀 편집' 문단을 찾지 못함");
    assert!(
        next_y >= table_bottom - 1.0,
        "#1835: 후속 문단이 확장된 표 하단(y={table_bottom:.1}) 위(y={next_y:.1})에서 시작 — 표 확장 미반영"
    );
}
