//! #7203의 비적용 오프셋 경로와 #6111 저장 표 흐름 대조군.
//! API 원본의 렌더링24검사는 기준 코드에서도90% 미달하여 #7731에서 복원한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

/// `vertical_offset` 이 걸린 자리차지 표가 있는 문서 — 저장 앵커를 쓰면 안 되는 갈래.
const OFFSET_SAMPLE: &str = "samples/rowbreak-problem-pages.hwp";

const HWPUNIT_PER_PX: f64 = 7200.0 / 96.0;

fn page_root(sample: &str, page: u32) -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("문서 로드");
    document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"))
        .root
}

fn find<'a>(
    node: &'a RenderNode,
    want: &mut impl FnMut(&RenderNode) -> bool,
) -> Vec<&'a RenderNode> {
    let mut out = Vec::new();
    fn walk<'a>(
        node: &'a RenderNode,
        want: &mut impl FnMut(&RenderNode) -> bool,
        out: &mut Vec<&'a RenderNode>,
    ) {
        if want(node) {
            out.push(node);
        }
        for child in &node.children {
            walk(child, want, out);
        }
    }
    walk(node, want, &mut out);
    out
}

fn table_top(root: &RenderNode, para_index: usize) -> f64 {
    let tables = find(root, &mut |node| match &node.node_type {
        RenderNodeType::Table(table) => table.para_index == Some(para_index),
        _ => false,
    });
    assert!(!tables.is_empty(), "문단 {para_index} 의 표가 이 쪽에 없다");
    // 한 문단이 여러 조각을 낼 수 있다 — 가장 위 조각이 윗변이다.
    tables
        .iter()
        .map(|node| node.bbox.y)
        .fold(f64::INFINITY, f64::min)
}

/// 문단 기준 `vertical_offset` 이 걸린 자리차지 표는 저장 앵커를 쓰지 않는다.
///
/// 윗변이 `앵커 + 오프셋` 이라 `vpos − 위여백` 산식이 성립하지 않는다.
/// `rowbreak-problem-pages.hwp` 구역1 의 `vertical_offset` 152·3019 HU 표가 이 갈래다 —
/// 앵커를 그대로 믿으면 12쪽 본문이 쪽 밖 480px 까지 밀리고 글자 11쌍이 겹친다.
#[test]
fn an_offset_anchored_float_table_keeps_flow_placement() {
    const PAGE: u32 = 11;
    let root = page_root(OFFSET_SAMPLE, PAGE);
    // 쪽 번호·꼬리말은 Body 밖에 산다 — 본문 글줄만 센다.
    let body = find(&root, &mut |node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .first()
    .copied()
    .expect("Body 노드");
    let body_bottom = body.bbox.y + body.bbox.height;

    let escaped: Vec<f64> = find(body, &mut |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
    .iter()
    .map(|node| node.bbox.y + node.bbox.height)
    .filter(|bottom| *bottom > body_bottom + 2.0)
    .collect();

    assert!(
        escaped.is_empty(),
        "본문 바닥 {body_bottom:.1} 아래로 빠져나간 본문 글줄 {}개(최대 {:.1}) — 오프셋이 \
         걸린 자리차지 표까지 저장 앵커로 배치하면 이 쪽이 무너진다",
        escaped.len(),
        escaped.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );
}

/// 저장 사다리 pi186→187은2432HU = 앞 개체 높이1300 + 위/아래여백566씩이다.
/// 뒤 표의 가시 윗변은 자기 앵커에 위여백141HU를 한 번 더한 위치다.
/// 두 표의 가시 상자 간격을 원시 앵커 간격과 혼동하지 않고 독립 PDF 괘선도 대조한다.
#[test]
fn stored_anchor_after_a_taller_float_consumes_its_span_once() {
    let root = page_root("samples/issue6111/56345_regulatory_impact_analysis.hwp", 10);
    let following_top = table_top(&root, 187);
    let advance = following_top - table_top(&root, 186);
    let stored_advance = (26992.0 - 24560.0) / HWPUNIT_PER_PX;
    // 원본 dump의 뒤 표 바깥 위여백141HU는 앞 개체가 닫는2432HU 밖에 있다.
    let following_outer_top = 141.0 / HWPUNIT_PER_PX;
    let visible_advance = stored_advance + following_outer_top;
    assert!(
        (advance - visible_advance).abs() <= 0.2,
        "저장 간격과 뒤 표 위여백 {visible_advance:.2}px 대신 {advance:.2}px를 소비했다"
    );
    // 독립 정본 PDF11쪽의 가로 괘선327.721008pt를96DPI로 변환한 좌표다.
    let pdf_following_top = 436.961_344;
    assert!(
        (following_top - pdf_following_top).abs() <= 1.5,
        "뒤 표 윗변이 독립 PDF와 다르다: 실제{following_top:.2}px, 기준{pdf_following_top:.6}px"
    );
}
