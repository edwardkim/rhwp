//! [Issue #6110] 표 칸 안 빈 문단에 앵커된 자리차지 그림이 그림 높이만큼(+98px)
//! 아래로 밀려 칸 밖으로 나간다 (39819 보도자료 머리 표 로고).
//!
//! 근인: 자리차지 그림은 글줄을 **자기 높이만큼** 아래로 밀고, 한글은 그 밀린
//! 줄의 vpos 를 저장한다. 이 칸은 문단이 하나뿐인 빈 문단인데 저장 vpos(7382HU)가
//! 그림 높이(7382HU)와 정확히 같다 — 즉 그 vpos 는 **이 그림 자신이 만든 변위**다.
//! #5731 이 그 값을 흐름 오프셋으로 써서(앞선 캡션·그림이 자리를 차지한 다문단 셀
//! 형상을 겨냥한 계약) 그림을 제 높이만큼 칸 밖으로 내렸다. #2226 이 앵커를 칸
//! 상단으로 잡아 뒀는데도 하류에서 되돌아간 것이다.
//!
//! 수정: 앞 내용이 없는 빈 문단에서 저장 vpos 가 이 그림의 높이와 같으면 흐름
//! 오프셋으로 쓰지 않는다.
//!
//! 픽스처는 원본 12MB 의 구역0 문단 0..3 슬라이스(그림 바이너리 제외, 15KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6110/39819_press_release_header_slice.hwp";

#[test]
fn issue_6110_cell_float_stays_inside_its_cell() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let cell = find_header_logo_cell(&page.root).expect("머리 표 첫 칸(5행 병합)");
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let logo = find_float(&page.root).expect("로고 개체");
    let logo_bottom = logo.bbox.y + logo.bbox.height;

    assert!(
        logo.bbox.y >= cell.bbox.y - 1.0 && logo_bottom <= cell_bottom + 1.0,
        "로고가 칸(y {:.1}~{cell_bottom:.1}) 밖으로 나갔다: y={:.1}~{logo_bottom:.1}",
        cell.bbox.y,
        logo.bbox.y
    );
}

/// 이 쪽에서 그림/자리표시자를 품은 첫 표 칸.
fn find_header_logo_cell(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) && node.bbox.y < 200.0 {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            if cell.row == 0 && cell.col == 0 && cell.row_span > 1 {
                return Some(node);
            }
        }
    }
    node.children.iter().find_map(find_header_logo_cell)
}

/// 그림 바이너리를 뺀 픽스처에서는 Placeholder 로 나온다.
fn find_float(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(
        node.node_type,
        RenderNodeType::Image(_) | RenderNodeType::Placeholder(_)
    ) && node.bbox.y < 400.0
    {
        return Some(node);
    }
    node.children.iter().find_map(find_float)
}
