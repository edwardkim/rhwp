//! 저장 표 선언 높이를 지키며, 앞 행이 커진 만큼을 마지막 행의 빈 공간에서 거둔다.
//!
//! `2025 행정업무운영 편람(최종)` 구역 10 pi=85(질의 41, 6×5 RowBreak, 선언 29772HU = 397.0px).
//! 정본 `pdf/…-hwp-kopub-2024.pdf`·`-hwpx-kopub-2024.pdf` 301쪽 괘선 실측:
//!
//! ```text
//!   행      0     1     2     3      4      5      합
//!   정본   12.3  33.3  20.2  13.3  308.1   9.6   396.9  (≒ 선언 397.0)
//!   선언   12.5  31.3  20.2  13.3   94.6  21.5           (셀 cellSz — 최솟값)
//!   종전   12.5  33.3  20.2  13.3  308.0  21.5   408.8  → 쪽을 넘어 마지막 행만 302쪽으로 분할
//! ```
//!
//! 마지막 행의 저장 내용 하한은 8.9px(빈 줄 1.3 + 위·아래 여백 7.5)이라 11.8px 를 거둘 수 있다.
//! 종전 규칙(#5906)은 같은 회수를 하되 «선언의 2% 이내» 라는 비율 상한으로 막혀 있었다.
//! 이 검사는 표가 한 쪽에 통째로 놓이고 그 높이가 선언과 같은지 본다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLES: [&str; 2] = [
    "samples/2025 행정업무운영 편람(최종).hwp",
    "samples/2025 행정업무운영 편람(최종).hwpx",
];
/// 질의 41 답변 첫 글.
const Q41_ANSWER: &str = "“업무관리시스템”은 업무처리의 전 과정에서 생산된";
/// pi=85 표 선언 높이(HWPUNIT) — 덤프 `[common] size=39686×29772`.
const DECLARED_HEIGHT_HU: f64 = 29772.0;

fn texts(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        texts(child, out);
    }
}

/// 질의 41 답변 글을 담은 표 노드들 — (쪽, 높이).
fn q41_fragments(doc: &HwpDocument) -> Vec<(u32, f64)> {
    let needle: String = Q41_ANSWER.split_whitespace().collect();
    let mut found = Vec::new();
    let mut first_page = None;
    for page in 0..doc.page_count() {
        if first_page.is_some_and(|first| page > first + 1) {
            break;
        }
        let root = doc.build_page_render_tree(page).expect("render tree").root;
        let mut stack = vec![&root];
        while let Some(node) = stack.pop() {
            if let RenderNodeType::Table(table) = &node.node_type {
                let mut text = String::new();
                texts(node, &mut text);
                let text: String = text.split_whitespace().collect();
                if table.para_index == Some(85) && text.contains(&needle) {
                    first_page.get_or_insert(page);
                    found.push((page, node.bbox.height));
                } else if first_page == Some(page.saturating_sub(1)) && table.para_index == Some(85)
                {
                    // 이어진 조각(글 없는 마지막 행만 넘어간 경우)도 센다.
                    found.push((page, node.bbox.height));
                }
                continue;
            }
            stack.extend(node.children.iter());
        }
    }
    found
}

#[test]
fn last_row_gives_back_the_growth_of_earlier_rows() {
    let declared_px = DECLARED_HEIGHT_HU * 96.0 / 7200.0;
    for sample in SAMPLES {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
        let doc =
            HwpDocument::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
        let fragments = q41_fragments(&doc);
        assert_eq!(
            fragments.len(),
            1,
            "{sample}: 질의 41 표는 정본처럼 한 쪽에 통째로 놓여야 한다: {fragments:?}"
        );
        let (_, height) = fragments[0];
        assert!(
            (height - declared_px).abs() < 0.6,
            "{sample}: 질의 41 표 높이 {height:.2} 는 선언 {declared_px:.2} 와 같아야 한다"
        );
    }
}
