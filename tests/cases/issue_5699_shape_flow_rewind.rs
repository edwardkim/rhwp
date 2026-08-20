//! [Issue #5699 H2] TopAndBottom(위·아래 어울림) 비-tac 그림/도형 Shape 아이템이
//! 단 흐름을 후퇴시켜, 후속 문단이 방금 페인트한 표·본문을 관통하던 결함의 회귀 가드.
//!
//! 근인: host 문단(텍스트 + RowBreak 표 + TopAndBottom 그림)의 Shape 아이템이
//! 그림 배치 y(문단 시작 기준)를 흐름 반환값으로 되돌려, 저장 앵커 줄까지 전진한
//! 흐름(베트남노동시장1125 p75: 841.7px)을 문단 시작(307.6px)으로 되감았다. 후속
//! 문단들이 표 페인트 대역(307..549) 안에서 흘러 겹침. Square 어울림은 텍스트가
//! 개체 옆으로 흐르도록 앵커로 되돌리는 것이 정당하지만 TopAndBottom 은 텍스트가
//! 개체 아래에서만 이어지므로 후퇴가 불법이다.
//!
//! 계약: 동봉 재현 문서(서울시 가로수 조례 별표 3 — TopAndBottom 그림 서식)에서
//! **서로 다른 문단의 본문 줄이 같은 y 대역(90% 이상)에 겹쳐 그려지지 않는다**.
//! 수정 전 실측: p3 pi2~pi3 줄이 y581 동일 대역 완전 겹침(r=1.00).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue5699/16758113_pruning_forms.hwp";

type Line = (usize, f64, f64, f64, f64);

fn collect_body_lines(node: &RenderNode, out: &mut Vec<Line>, in_cell: bool) {
    let cell_like = matches!(
        node.node_type,
        RenderNodeType::TableCell(_) | RenderNodeType::Header | RenderNodeType::Footer
    ) || in_cell;
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if !cell_like {
            if let Some(pi) = tl.para_index {
                let b = &node.bbox;
                if b.height > 0.0 && b.height < 150.0 {
                    out.push((pi, b.y, b.y + b.height, b.x, b.x + b.width));
                }
            }
        }
    }
    for child in &node.children {
        collect_body_lines(child, out, cell_like);
    }
}

#[test]
fn issue_5699_topbottom_shape_does_not_rewind_flow_into_overlap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    // 수정 전 실측 결함: p3 에서 pi2 의 줄과 pi3 의 줄이 같은 y 대역(r=1.00)에
    // 완전 겹침(r38 검출기 v4). 그 쌍에 한정해 잠근다 — 페이지 전수 스윕 계약은
    // 부속 플로우(나란한 캡션 등) 기존 잔재까지 걸어 위양성이 난다.
    let mut checked = false;
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render tree p{page}: {e}"));
        let mut lines = Vec::new();
        collect_body_lines(&tree.root, &mut lines, false);
        let pi2: Vec<&Line> = lines.iter().filter(|l| l.0 == 2).collect();
        let pi3: Vec<&Line> = lines.iter().filter(|l| l.0 == 3).collect();
        if pi2.is_empty() || pi3.is_empty() {
            continue;
        }
        checked = true;
        for a in &pi2 {
            for b in &pi3 {
                let dy = a.2.min(b.2) - a.1.max(b.1);
                let dx = a.4.min(b.4) - a.3.max(b.3);
                let small = (a.2 - a.1).min(b.2 - b.1);
                assert!(
                    !(small > 0.0 && dx > 4.0 && dy >= small * 0.9),
                    "p{} 에서 pi2 줄({:.0}..{:.0})과 pi3 줄({:.0}..{:.0})이 같은 y 대역에 겹침 (#5699 H2 회귀)",
                    page + 1, a.1, a.2, b.1, b.2
                );
            }
        }
    }
    assert!(checked, "pi2/pi3 줄을 가진 쪽이 없다 — 샘플/계약 확인 필요");
}
