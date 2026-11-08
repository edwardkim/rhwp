//! 고정형: #4138 독립 Print의 온도 단위 이어받기 x 원점을 실제 192DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-06T22:13:05+09:00.
//! 2026-11-06 raw(192DPI, PDFium, 2px): 90쪽 Native 99.99501% / fresh WASM 100.00000%.
//! 전197쪽 최저 91.17674% / 91.29951%; source fa0436f495, 양쪽 각7쪽의 98% 미달은 남아 있다.
//! Native API 논리 좌표 96DPI를 실제 비교 PNG와 같은 192DPI로 환산한다.
//! 줄 소속·원문 보존은 별도 관계 검사로 유지한다.
//! 근거: mydocs/pr/assets/issue4138/numeric-closing-20261106/README.md.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.display_or_text().to_owned();
    }
    node.children.iter().map(text).collect()
}

fn continuation(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextLine(meta)
        if meta.para_index == Some(1201))
        && text(node).trim_start().starts_with("0℃")
    {
        return Some(node);
    }
    node.children.iter().find_map(continuation)
}

#[test]
fn print_temperature_continuation_origin_matches_the_independent_pdf() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp",
    );
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(89).unwrap();
    let line = continuation(&page.root).expect("독립 Print처럼 0℃가 이어받은 줄의 처음을 소유함");
    let physical_x = line.bbox.x * (192.0 / 96.0);
    let expected_x = 111.179_740_9 * (192.0 / 72.0);
    assert!(
        (physical_x - expected_x).abs() <= 2.0,
        "독립 PDF의 이어받기 원점에서 ±2px: actual={physical_x}, expected={expected_x}"
    );
}
