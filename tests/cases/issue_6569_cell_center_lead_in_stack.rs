//! [Issue #6569] 세로 가운데 정렬 칸의 첫 문단 위 여백을 **정렬 공간에서 두 번 빼던**
//! 결함의 가드.
//!
//! `#6630` 이 "첫 문단 위 여백(`first_para_lead`)을 정렬 공간에서 뺀다" 는 계약을
//! 세웠는데, 그 계약은 **재조판 스택이 그 여백을 아직 안 품었을 때만** 옳다. 정렬은
//! 결국 *그려지는* 범위를 가운데에 두는 일이기 때문이다.
//!
//! 판별자는 저장 사다리가 준다 — `stored_flow_extent` 는 셀 내용 상단부터 마지막 줄
//! 바닥까지라 **첫 줄의 `vpos` 를 이미 포함**한다.
//!
//! ```text
//! 156678235 1쪽 제목 칸   ext 78.13 == content 78.13          → 스택이 품었다 → 빼면 안 됨
//! #6630 exam_eng 머리 칸  ext 45.37 vs content 37.80 (차=lead) → 스택이 뺐다   → 빼야 함
//! ```
//!
//! 한/글 2024 실측(제목 표): 표 상단 170.57pt, 제목 1행 글자 상단 188.25pt.
//! 종전 rhwp 186.03(−2.22pt = lead 6.67px 의 절반), 교정 후 188.53(+0.28).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6542/156678235_mid_para_vpos_rewind.hwp";

fn has_text(node: &RenderNode, needle: &str) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        || node.children.iter().any(|child| has_text(child, needle))
}

fn table_with_text<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table { .. }) && has_text(node, needle) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_with_text(child, needle))
}

/// 제목의 두 글줄은 같은 셀 안에 남고, 저장된 첫 문단 여백이 상단에 보존된다.
/// 첫 문단 여백을 정렬 공간에서 다시 빼면 상·하단 여백이 거의 같아진다.
#[test]
fn centered_cell_does_not_subtract_lead_the_stack_already_holds() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");

    let table = table_with_text(&page.root, "사후소득").expect("제목 표는 1쪽에 있어야 한다");
    let cell = table
        .children
        .iter()
        .find(|child| {
            matches!(child.node_type, RenderNodeType::TableCell(_)) && has_text(child, "사후소득")
        })
        .expect("제목은 제목 표의 셀에 속해야 한다");
    let lines: Vec<_> = cell
        .children
        .iter()
        .filter(|child| matches!(child.node_type, RenderNodeType::TextLine(_)))
        .collect();
    assert_eq!(lines.len(), 2, "제목 셀의 두 글줄이 보존돼야 한다");
    assert!(has_text(lines[0], "사후소득"));
    assert!(has_text(lines[1], "노후생활"));
    let top_gap = lines[0].bbox.y - cell.bbox.y;
    let bottom_gap = cell.bbox.y + cell.bbox.height - lines[1].bbox.y - lines[1].bbox.height;
    assert!(
        top_gap > bottom_gap * 1.1,
        "첫 문단 위 여백이 셀 정렬에서 두 번 빠졌다 — 상단 {top_gap:.2}, 하단 {bottom_gap:.2}"
    );
}

/// `#6630` 의 반대 갈래는 그대로 유지된다 — 스택이 `lead` 를 안 품은 칸에서는 계속 뺀다.
/// `exam_eng` 2쪽 바탕쪽 머리 표의 제목 그림이 그 계약이다.
#[test]
fn issue_6630_contract_still_holds_when_stack_excludes_lead() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_eng.hwp");
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_eng");
    let svg = doc.render_page_svg(1).expect("exam_eng 2쪽 SVG");

    // 148.7px 폭 제목 그림의 y — 한/글 149.2(셀 상단 132.3 + 16.9). 종전 결함은 145.5.
    let mut ys = Vec::new();
    for open in ["<image ", "<svg "] {
        for t in svg.split(open).skip(1) {
            let t = &t[..t.find('>').expect("태그 닫힘")];
            let num = |n: &str| -> Option<f64> {
                t.split(&format!("{n}=\""))
                    .nth(1)
                    .and_then(|r| r.split('"').next())
                    .and_then(|v| v.parse().ok())
            };
            if num("width").is_some_and(|w| (w - 148.7).abs() < 0.6) {
                if let Some(y) = num("y") {
                    ys.push(y);
                }
            }
        }
    }
    let Some(y) = ys.into_iter().reduce(f64::min) else {
        return;
    };
    assert!(
        (y - 149.2).abs() < 2.0,
        "#6630 계약(스택이 lead 를 안 품은 칸)이 깨졌다 — 제목 그림 y={y:.2} (한/글 149.2)"
    );
}
