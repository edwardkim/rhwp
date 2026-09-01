//! [Issue #6535] 쪽-앵커(`vert=쪽`) 자리차지 블록의 host 문단 저장 `vpos` 는 **절대 위치
//! 산물**이다. 그것으로 본문 흐름을 동기화하면 흐름이 실제보다 위로 올라가, 1쪽에 들어가는
//! 블록이 통째로 다음 쪽에 단독 배치되고 **본문 없는 빈 쪽**이 생긴다.
//!
//! `36404612_결재문서본문.hwpx` pi=4 (`발신명의` 틀, `wrap=자리차지 vert=쪽(0)`) 실측:
//!
//! ```text
//! 흐름 cur_h   542.80px
//! 저장 vpos    49154 = 655.39px      ← 112.6px 상향 (절대배치 산물)
//! 블록 높이    351.37px
//! 배타 잔여    638.87px
//!
//! 동기화 함  → slack -16.52  → 2쪽에 단독 배치 (한/글은 1쪽)
//! 건너뜀     → slack +96.07  → 같은 쪽에 흡수 ✔
//! ```
//!
//! 같은 논리는 `#2279` 가 **같은 쪽의 다른 절대배치 표**(`page_has_page_abs_top_table`)에
//! 대해 이미 편 것이다 — 블록 자신에게 적용하지 않을 이유가 없다.
//!
//! `anchor_vpos <= 0` 인 경우는 상류에서 이미 `prev_body_bottom_vpos`(직전 본문 문단의 저장
//! 흐름 하단)로 복원한 **본문 좌표**라 이 예외 대상이 아니다 — 그 갈래는 `#2098`/`#2138` 이
//! 눈금을 맞춘 50px 마진이 따로 판정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6535/36404612_page_anchored_footer_block.hwpx";

fn find_text_y(node: &rhwp::renderer::render_tree::RenderNode, needle: &str) -> Option<f64> {
    if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_text_y(child, needle))
}

fn contains_text(node: &rhwp::renderer::render_tree::RenderNode, needle: &str) -> bool {
    find_text_y(node, needle).is_some()
}

fn find_table_bounds_with_text(
    node: &rhwp::renderer::render_tree::RenderNode,
    needle: &str,
) -> Option<(f64, f64)> {
    if matches!(
        node.node_type,
        rhwp::renderer::render_tree::RenderNodeType::Table(_)
    ) && contains_text(node, needle)
    {
        return Some((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    node.children
        .iter()
        .find_map(|child| find_table_bounds_with_text(child, needle))
}

#[test]
fn issue_6535_page_anchored_block_stays_on_its_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6535 sample");

    let pages = document.page_count();
    assert_eq!(
        pages, 1,
        "쪽-앵커 블록이 1쪽에 들어가는데도 새 쪽으로 밀렸다 — 실측 {pages}쪽 (한/글 1쪽, \
         회귀 시 2쪽이고 2쪽에는 본문이 하나도 없다)"
    );

    // 원점이 아니라 쪽 수만 보면 "블록이 사라져도" 통과한다 — 블록이 그 쪽에 실재하는지 함께
    // 고정한다.
    let tree = document.build_page_render_tree(0).expect("render p1");
    let mut tables = 0usize;
    fn walk(node: &rhwp::renderer::render_tree::RenderNode, tables: &mut usize) {
        if matches!(
            node.node_type,
            rhwp::renderer::render_tree::RenderNodeType::Table(_)
        ) {
            *tables += 1;
        }
        for child in &node.children {
            walk(child, tables);
        }
    }
    walk(&tree.root, &mut tables);
    assert!(
        tables >= 3,
        "1쪽에 표가 {tables}개뿐이다 — 발신명의 틀까지 같은 쪽에 있어야 한다"
    );

    let body_y = find_text_y(&tree.root, "2.").expect("find numbered body paragraph");
    let (table_y, table_bottom) =
        find_table_bounds_with_text(&tree.root, "연번").expect("find incident table");
    let ending_y = find_text_y(&tree.root, "끝.").expect("find ending paragraph");
    assert!(
        body_y < table_y && table_bottom <= ending_y,
        "Hancom 순서·비중첩과 달리 본문({body_y:.1})/표({table_y:.1}..{table_bottom:.1})/종결문({ending_y:.1})이 배치됐다"
    );

    // Hancom 2020 PDF(d5a4a5f8…)의 bbox 좌표를 72→96 DPI로 환산한 기준이다.
    // 순서만 맞고 본문이 한 줄 위로 가거나 종결문이 표에 걸치는 보정을 막는다.
    let tolerance_px = 2.0;
    for (name, actual, oracle) in [
        ("2. 본문", body_y, 393.5),
        ("연번 표", table_y, 456.2),
        ("끝. 종결문", ending_y, 646.9),
    ] {
        assert!(
            (actual - oracle).abs() <= tolerance_px,
            "{name} y={actual:.1}px, Hancom 2020 oracle={oracle:.1}px (허용 ±{tolerance_px:.1}px)"
        );
    }
}
