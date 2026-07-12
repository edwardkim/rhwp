//! Issue #1858 발현 2: vert=쪽 valign=Bottom 하단앵커 표의 하단 밀착 회귀 가드.
//!
//! 한컴은 하단앵커 표의 **렌더된(실측) 하단**을 anchor 하단(body 하단)에 밀착시킨다.
//! 종전 rhwp 는 top 을 선언높이(common.height) 기준으로 잡아, 선언높이가 실측보다
//! 큰(stale) 결재/발신명의 문서에서 블록 전체가 위로 떴다 — opengov 코퍼스 18건 중
//! 13건이 동일 상수 −30.5pt 오프셋(36389312 pi=6: 선언 357.2px vs 실측 316.7px →
//! 40.5px 상향), 수정 후 전부 −3pt 이내.
//!
//! fixture: `samples/hwpx/opengov/36389312_...hwpx` (pi=5/pi=6 하단앵커 표 2개).
//! 오라클: `pdf/36389312_..._-2024.pdf` (Producer=Hancom PDF, Hwp 2024) — 하단 블록
//! 텍스트 라인 y 가 rhwp 와 ±3pt 정합.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str =
    "samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx";

fn page_tree() -> rhwp::renderer::render_tree::PageRenderTree {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    doc.build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render p1: {e}"))
}

fn find_body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(find_body_bottom)
}

fn find_table<'a>(node: &'a RenderNode, pi: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.para_index == Some(pi) && t.control_index == Some(0) {
            return Some(node);
        }
    }
    node.children.iter().find_map(|c| find_table(c, pi))
}

/// pi=5(선언==실측)·pi=6(선언>실측) 하단앵커 표 둘 다 렌더 하단이 body 하단에 밀착.
#[test]
fn bottom_anchored_tables_render_flush_with_body_bottom() {
    let tree = page_tree();
    let body_bottom = find_body_bottom(&tree.root).expect("Body 노드");
    for pi in [5usize, 6usize] {
        let table = find_table(&tree.root, pi).unwrap_or_else(|| panic!("pi={pi} 표"));
        let bottom = table.bbox.y + table.bbox.height;
        // [Task #2221] 측정·렌더 pad 회계 일관화로 pi=6 드리프트 3.7px→0 —
        // 허용오차 6px→2px 강화. 수정 전 오프셋은 40.5px 로 확실히 구분된다.
        assert!(
            (bottom - body_bottom).abs() <= 2.0,
            "#1858: pi={pi} 하단앵커 표 하단({bottom:.1})이 body 하단({body_bottom:.1})에 \
             밀착하지 않음 — 선언높이 기준 top 배치(블록 상향 부유) 회귀",
        );
    }
}
