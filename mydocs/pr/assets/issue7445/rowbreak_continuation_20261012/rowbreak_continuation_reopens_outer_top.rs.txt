//! 이어지는 RowBreak 표 조각은 쪽 위에서 표의 바깥 위 여백을 다시 연다(#7685).
//!
//! # 규칙과 독립 기대값
//!
//! 한/글 정본에서 이어지는 조각의 표 상자 위끝은 「본문 위 + 선언 `outMargin.top`」 이다
//! (#6976 원장: 조각 표 208건 중 180건 정확 일치). rhwp 는 일부 좁은 형상에서만 이 여백을
//! 열어, 나머지 조각은 본문 위에 붙고 그만큼 쪽마다 내용을 더 받았다.
//!
//! - `samples/task2287/1342000_edu_curriculum_map.hwp` 124쪽(가로 쪽, `outMargin.top` 141HU):
//!   정본 `pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf` 표 위 괘선 77.44px =
//!   본문 위 75.6 + 1.88. 종전 rhwp 75.6.
//! - 쪽수: 이 여백이 빠지면 조각마다 행을 더 받아 1342000 이 정본 415쪽보다 2쪽 적었다.
//!
//! # 반례(칸 중간에서 이어지는 조각)
//!
//! 행 경계가 아니라 칸 내용 중간에서 이어지는 조각에는 이 여백을 열지 않는다.
//! `table_giant_cell_overfill.hwp` 2·6쪽과 `issue1949` 55쪽의 정본 글줄은 1쪽과 같은
//! 오프셋(−3.5px · −2.2px)에 놓여, 여백을 열면 3.8px 아래로 어긋난다.
//!
//! # 반례(이중 계상)
//!
//! 조판 예산의 기존 좁은 형상(끝 조각 등)이 이미 여백을 열면 다시 더하지 않는다.
//! 두 번 더하면 `rowbreak-problem-pages.hwpx` 14쪽 pi16 이 한 유닛 일러 정본 18쪽 → 19쪽,
//! `issue7336` 정본 7쪽 → 8쪽이 된다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("재현체 {}: {e}", full.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn first_table_top(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(first_table_top)
}

#[test]
fn landscape_continuation_starts_below_outer_top_margin() {
    let core = load("samples/task2287/1342000_edu_curriculum_map.hwp");
    let tree = core.build_page_render_tree(123).expect("124쪽 렌더 트리");
    let top = first_table_top(&tree.root).expect("124쪽 표");
    let expected = 75.6 + 141.0 * 96.0 / 7200.0;
    assert!(
        (top - expected).abs() < 0.5,
        "124쪽 이어지는 조각 위끝 {top:.2} ≠ 본문 위 + 바깥 위 여백 {expected:.2} (정본 77.44)"
    );
}

#[test]
fn continuation_outer_top_matches_hangul_page_counts() {
    for (path, hangul_pages) in [
        // pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf
        ("samples/task2287/1342000_edu_curriculum_map.hwp", 415),
        // 이중 계상 반례 — pdf/rowbreak-problem-pages-hwpx-2020.pdf
        ("samples/rowbreak-problem-pages.hwpx", 18),
        // 이중 계상 반례 — pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf
        (
            "samples/issue7336/stored_frame_page_larger_rowbreak.hwpx",
            7,
        ),
    ] {
        assert_eq!(load(path).page_count(), hangul_pages, "{path}");
    }
}
