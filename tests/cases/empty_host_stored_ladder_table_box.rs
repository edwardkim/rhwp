//! 글자 없는 host 의 문단 기준 자리차지 표는 host 의 문단 앞·뒤 간격과 줄 상자를
//! 흐름에 싣지 않는다 — 한/글 저장 사다리가 그 상자를 증언한다.
//!
//! `2025 행정업무운영 편람(최종)` 구역 10 pi=73(질의 36, 문단 앞 간격 20px·뒤 13.3px,
//! 6×5 RowBreak 표 선언 291.3px, 아래 바깥여백 7.5px) 다음 pi=74(질의 37, 선언 319.8px).
//!
//! ```text
//!   저장 사다리   pi=74.vpos − pi=73.vpos = 278.8 = 291.3 + 7.5 − 20.0
//!   한/글 정본    299쪽에 질의 36·37 두 표가 함께 있다
//!                 (`pdf/…-hwp-kopub-2024.pdf`, 표 안 괘선 326.5·625.4)
//! ```
//!
//! 종전 조판은 pi=73 에 표 + 앞·뒤 간격 + 줄 간격(334.6px)을 예약해 pi=74 가 쪽에
//! 들어가지 않는다고 보고 다음 쪽으로 넘겼다(렌더는 저장 자리 그대로라 300쪽 위가 빈다).
//! 이 검사는 두 표가 같은 쪽에 있고, 둘 사이가 앞 표의 아래 바깥여백뿐인지 본다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLES: [&str; 2] = [
    "samples/2025 행정업무운영 편람(최종).hwp",
    "samples/2025 행정업무운영 편람(최종).hwpx",
];
const Q36: &str = "현재 사용중인 업무관리시스템(온나라 문서 시스템)의 경우에도";
const Q37: &str = "원칙적으로 공무원이 아닌 자는 전자문서시스템의 계정을";
/// pi=73 표 아래 바깥여백 566HU.
const OUTER_BOTTOM_PX: f64 = 566.0 * 96.0 / 7200.0;

fn texts(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        texts(child, out);
    }
}

/// (쪽, 위, 아래) — `from` 쪽부터 찾은, 그 글을 담은 맨 바깥 표.
fn table_with(doc: &HwpDocument, needle: &str, from: u32) -> Option<(u32, f64, f64)> {
    let needle: String = needle.split_whitespace().collect();
    for page in from..doc.page_count() {
        let root = doc.build_page_render_tree(page).ok()?.root;
        let mut stack = vec![&root];
        while let Some(node) = stack.pop() {
            if matches!(node.node_type, RenderNodeType::Table(_)) {
                let mut text = String::new();
                texts(node, &mut text);
                if text
                    .split_whitespace()
                    .collect::<String>()
                    .contains(&needle)
                {
                    return Some((page, node.bbox.y, node.bbox.y + node.bbox.height));
                }
                continue;
            }
            stack.extend(node.children.iter());
        }
    }
    None
}

#[test]
fn empty_host_ladder_table_reserves_only_its_box() {
    for sample in SAMPLES {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
        let doc =
            HwpDocument::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
        let (p36, _, bottom36) = table_with(&doc, Q36, 0).expect("질의 36 표");
        let (p37, top37, _) = table_with(&doc, Q37, p36).expect("질의 37 표");
        assert_eq!(
            p36, p37,
            "{sample}: 질의 36·37 표는 정본처럼 같은 쪽(299쪽)에 있어야 한다"
        );
        assert!(
            (top37 - bottom36 - OUTER_BOTTOM_PX).abs() < 0.6,
            "{sample}: 두 표 사이는 앞 표의 아래 바깥여백 {OUTER_BOTTOM_PX:.2}px 이어야 한다 \
             (앞 표 아래 {bottom36:.2}, 뒤 표 위 {top37:.2})"
        );
    }
}
