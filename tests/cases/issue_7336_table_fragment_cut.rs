//! [#7336] 조각 컷 회계와 조각 페인트가 어긋나 본문이 통째로 사라지던 두 갈래.
//!
//! 공통 증상은 같다 — 페이지네이터가 조각에 배정한 높이와 페인터가 실제로 그리는
//! 상자 높이가 발산해, 그 차이만큼의 내용이 clip·쪽 밖으로 사라지고 다음 조각은
//! 이미 소비한 컷 **다음**부터 재개하므로 사라진 내용이 어느 쪽에도 남지 않는다.
//!
//! 1. `nested_table_fragment_cut.hwp` — 조각이 rowspan 블록 **안쪽**에서 끝나면
//!    (`end_row < block_end`) 블록-합 보정이 차액을 이 조각이 그리지 않는 행에 실었다.
//!    4쪽 실측: 예산 `consumed=603.6` vs 그린 상자 `tbl_h=81.8` — 걸침 셀이 소비한
//!    10×2 중첩 표가 칸 하단(486.2) 밖 `y=535.3` 에 놓여 통째로 clip 됐다.
//!    **클립은 페인트 단계라 `extract_page_text_native` 에는 남는다** — 그래서 이
//!    갈래는 텍스트가 아니라 조각 셀 bbox 와 중첩 표 bbox 의 포함 관계로 잠근다.
//! 2. `stored_frame_page_larger_rowbreak.hwpx` — 저장 RowBreak object frame 이
//!    선언 901.2px 로 "이 쪽을 소유한다"고 말하는데 실측 표는 3,676px(약 4쪽)였다.
//!    통째 배치로 본문 아래 2,742px 가 넘쳐 `2-3. 추진일정` 절 전체가 사라졌다.
//!
//! 두 문서 모두 `rhwp dump` 에는 내용이 남아 있어 파서가 아니라 조판·레이아웃 결함이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const NESTED_CUT_SAMPLE: &str = "samples/issue7336/nested_table_fragment_cut.hwp";
const STORED_FRAME_SAMPLE: &str = "samples/issue7336/stored_frame_page_larger_rowbreak.hwpx";

fn load_doc(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open")
}

/// 쪽 텍스트에서 공백을 지운 형태로 비교한다 — 줄바꿈·칸 경계의 공백은 이 계약의
/// 대상이 아니고, 소실 여부만 판정한다.
fn squashed_page_text(doc: &DocumentCore, page: u32) -> String {
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트: {e}", page + 1))
        .split_whitespace()
        .collect()
}

fn assert_page_has(doc: &DocumentCore, page: u32, needle: &str) {
    let text = squashed_page_text(doc, page);
    let wanted: String = needle.split_whitespace().collect();
    assert!(
        text.contains(&wanted),
        "#7336: {}쪽에 {needle:?} 가 있어야 한다\n--- {}쪽 ---\n{text}",
        page + 1,
        page + 1
    );
}

fn bottom(bbox: &BoundingBox) -> f64 {
    bbox.y + bbox.height
}

/// 클립이 걸린 조각 셀 중 **중첩 표를 품은** 첫 셀과 그 중첩 표를 찾는다.
fn find_clipping_fragment_cell_with_nested_table(
    node: &RenderNode,
) -> Option<(&RenderNode, &RenderNode)> {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.clip && cell.page_fragment {
            if let Some(nested) = node
                .children
                .iter()
                .find(|child| matches!(child.node_type, RenderNodeType::Table(_)))
            {
                return Some((node, nested));
            }
        }
    }
    node.children
        .iter()
        .find_map(find_clipping_fragment_cell_with_nested_table)
}

#[test]
fn nested_table_fragment_box_covers_the_cut_it_consumed() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    let page = doc.build_page_render_tree(3).expect("4쪽 render tree");
    let (cell, nested) =
        find_clipping_fragment_cell_with_nested_table(&page.root).expect("4쪽 조각 칸 + 중첩 표");
    assert!(
        bottom(&nested.bbox) <= bottom(&cell.bbox) + 1.0,
        "#7336: 조각 칸이 컷으로 소비한 중첩 표는 칸 클립 안에 있어야 한다 — \
         중첩 표 하단 {:.1}, 칸 하단 {:.1} (칸 y={:.1} h={:.1})",
        bottom(&nested.bbox),
        bottom(&cell.bbox),
        cell.bbox.y,
        cell.bbox.height,
    );
}

#[test]
fn nested_table_fragment_keeps_the_straddling_cell_content_on_page_four() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    assert_eq!(
        doc.page_count(),
        6,
        "#7336: 한/글 2024 는 이 문서를 6쪽으로 조판한다"
    );
    // 한/글 정본과 같은 4쪽(0-기반 3). 컷은 처음부터 이 쪽에 배정돼 있었고,
    // 결함은 그린 상자가 짧아 clip 으로 지워지는 것이었다.
    for needle in ["무형자산취득비", "기계장치비", "회계정산비"] {
        assert_page_has(&doc, 3, needle);
    }
    // 5쪽 이어짐 조각은 4쪽이 소비한 컷 **다음**부터 재개해야 한다.
    let fifth = squashed_page_text(&doc, 4);
    assert!(
        !fifth.contains("무형자산취득비"),
        "#7336: 5쪽이 4쪽 조각의 내용을 다시 그리면 안 된다\n--- 5쪽 ---\n{fifth}"
    );
}

#[test]
fn stored_frame_page_larger_rowbreak_table_is_split_across_pages() {
    let doc = load_doc(STORED_FRAME_SAMPLE);
    assert_eq!(
        doc.page_count(),
        7,
        "#7336: 한/글 2024 는 이 문서를 7쪽으로 조판한다 — 결함 시 4쪽(표를 통째로 얹고 쪽 밖으로 넘침)"
    );
}

#[test]
fn stored_frame_page_larger_rowbreak_keeps_the_schedule_section() {
    let doc = load_doc(STORED_FRAME_SAMPLE);
    // 정본과 같은 쪽 소속: 4쪽 '2-3. 추진일정', 5쪽 '총 합계'.
    assert_page_has(&doc, 3, "2-3. 추진일정");
    assert_page_has(&doc, 4, "총 합계");
}
