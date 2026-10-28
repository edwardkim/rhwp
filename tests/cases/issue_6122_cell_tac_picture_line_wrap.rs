//! [Issue #6122] 쪽 분할 거대 셀 안 같은 문단의 TAC 그림 2개를 줄바꿈 없이 나란히
//! 놓아 둘째가 칸·용지 밖으로 나간다 (2181727 6쪽 [그림 7]).
//!
//! 근인: `table_partial` 의 "단독 이미지" 폴백(텍스트를 가진 문단 경로)이 저장
//! lineseg 를 보지 않고 x 만 누적했다. 이 문단의 저장 lineseg 는 2줄이고 각 줄
//! 높이(15693·10010 HU)가 각 그림 높이와 정확히 같다 — 한글은 둘째 그림을 다음
//! 줄로 내린다. #4370·#6101 과 같은 "인라인 개체 폭 초과 미개행" 계열의 그림 판.
//!
//! 수정 후 계약 세 가지를 고정한다.
//!   1) 두 그림이 세로로 쌓이고 둘 다 칸 안에 있다
//!   2) 개행한 줄의 x 는 문단 정렬(가운데)을 그 줄의 폭 기준으로 따른다
//!   3) 캡션 "[그림 7] …" 이 그림 아래로 흐른다(겹치지 않는다)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6122/2181727_press_guard_test_method.hwp";
/// 결함 형상이 나타나는 쪽(0-based) — [그림 7] 이 있는 6쪽.
const PAGE: u32 = 5;
const CAPTION_HEAD: &str = "[그림";

#[test]
fn issue_6122_cell_tac_pictures_stack_inside_the_cell() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 6 render tree");

    let cell = find_tall_cell(&page.root).expect("쪽 분할된 거대 셀");
    let cell_right = cell.bbox.x + cell.bbox.width;
    let mut images: Vec<&RenderNode> = Vec::new();
    collect_images(cell, &mut images);
    assert_eq!(
        images.len(),
        2,
        "[그림 7] 두 조각이 이 칸에 그려져야 한다 (그려진 수: {})",
        images.len()
    );

    let (upper, lower) = if images[0].bbox.y <= images[1].bbox.y {
        (images[0], images[1])
    } else {
        (images[1], images[0])
    };

    // 1) 세로로 쌓인다 — 결함 시 두 그림의 y 가 같다.
    let upper_bottom = upper.bbox.y + upper.bbox.height;
    assert!(
        lower.bbox.y >= upper_bottom - 1.0,
        "둘째 그림이 첫 그림 아래로 내려가야 한다: 첫 아래끝={upper_bottom:.1}, 둘째 위끝={:.1}",
        lower.bbox.y
    );

    // 2) 둘 다 칸 안 — 결함 시 둘째가 우단 1097.9(용지 794) 까지 나갔다.
    for img in [upper, lower] {
        let right = img.bbox.x + img.bbox.width;
        assert!(
            img.bbox.x >= cell.bbox.x - 1.0 && right <= cell_right + 1.0,
            "그림이 칸(x {:.1}~{cell_right:.1}) 밖으로 나갔다: x={:.1}~{right:.1}",
            cell.bbox.x,
            img.bbox.x
        );
    }

    // 가운데 정렬: 두 그림 모두 칸 좌우 여백이 대등해야 한다(칸 왼쪽 끝에 붙지 않음).
    let upper_left_gap = upper.bbox.x - cell.bbox.x;
    assert!(
        upper_left_gap > 8.0,
        "가운데 정렬 그림이 칸 왼쪽 끝에 붙었다: 좌여백={upper_left_gap:.1}"
    );

    // 3) 캡션이 그림 아래 — 결함 시 첫 그림 위에 겹쳐 그려졌다.
    let caption_y = first_text_line_y(cell, CAPTION_HEAD).expect("캡션 줄");
    let lower_bottom = lower.bbox.y + lower.bbox.height;
    assert!(
        caption_y >= lower_bottom - 1.0,
        "캡션이 그림 아래로 흘러야 한다: 캡션 y={caption_y:.1}, 그림 아래끝={lower_bottom:.1}"
    );
}

/// 이 쪽에서 그림을 품은 가장 높은 표 칸.
fn find_tall_cell(node: &RenderNode) -> Option<&RenderNode> {
    let mut best: Option<&RenderNode> = None;
    fn walk<'a>(node: &'a RenderNode, best: &mut Option<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            let mut images = Vec::new();
            collect_images(node, &mut images);
            if !images.is_empty()
                && best.is_none_or(|current| node.bbox.height > current.bbox.height)
            {
                *best = Some(node);
            }
        }
        for child in &node.children {
            walk(child, best);
        }
    }
    walk(node, &mut best);
    best
}

fn collect_images<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_images(child, out);
    }
}

/// 자손 TextRun 이 `head` 로 시작하는 첫 줄의 y.
fn first_text_line_y(node: &RenderNode, head: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.starts_with(head) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| first_text_line_y(child, head))
}
