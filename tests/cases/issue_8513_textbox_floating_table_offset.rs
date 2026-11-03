//! 검사 유형: 관계형.
//!
//! 글상자 안에서 글자처럼 취급하지 않는 '글앞으로' 표는 자기 가로·세로 오프셋 자리에 놓인다.
//!
//! 결함: 글상자 내용 배치(`layout_textbox_content`)가 떠 있는 표도 글자처럼 취급한 표처럼 문단
//! 정렬로만 놓아, 가로(단 기준)·세로(문단 기준) 오프셋과 바깥 여백을 버리고 글상자 글 영역
//! 왼쪽 위에 그렸다. 제목 글자와 함께 글상자에 든 결재 칸 같은 표가 제목 위로 겹친다.
//!
//! 입력은 실문서를 쓰지 않은 합성 문서다. 공개 샘플 `issue8167/tac_textbox_inline.hwpx` 의
//! 글상자에 `issue3738/tac_sibling_shape_line_advance.hwpx` 의 표를 넣고, 표를 글앞으로·
//! 가로 단 기준 100mm·세로 문단 기준 2mm·바깥 여백 1mm 로 바꿨다(저장 줄 없음).
//!
//! 독립 기준: 같은 파일을 한컴 웹한글기안기로 열면 표 왼쪽 선이 글상자 글 영역 왼쪽에서
//! 오프셋 + 바깥 여백만큼, 표 위 선이 글상자 첫 문단 위에서 세로 오프셋 + 바깥 여백만큼
//! 떨어져 그려진다(96dpi 환산 가로 385px 차, 세로 선 y 151·226 이 rhwp 와 일치).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const HWPX: &[u8] = include_bytes!("../../samples/issue8513/textbox_floating_table_offset.hwpx");

/// 합성 문서에 적은 표 배치(HWPUNIT): 가로 오프셋 100mm, 세로 오프셋 2mm, 바깥 여백 1mm.
const HORZ_OFFSET: i32 = 28346;
const VERT_OFFSET: i32 = 567;
const OUTER_MARGIN: i32 = 283;

/// `needle` 을 담은 글자 묶음(TextRun)의 상자.
fn run_box(node: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(|c| run_box(c, needle))
}

/// 글상자 글 영역, 그 안의 표, 표 밖 "TITLE" 글자 묶음.
fn textbox_parts(node: &RenderNode) -> Option<(BoundingBox, BoundingBox, BoundingBox)> {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        let table = node
            .children
            .iter()
            .find(|c| matches!(c.node_type, RenderNodeType::Table(_)))?;
        let title = node
            .children
            .iter()
            .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
            .find_map(|c| run_box(c, "TITLE"))?;
        return Some((node.bbox, table.bbox, title));
    }
    node.children.iter().find_map(textbox_parts)
}

#[test]
fn floating_table_in_textbox_keeps_its_offsets() {
    let core = DocumentCore::from_bytes(HWPX).unwrap();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let (area, table, title) = textbox_parts(&tree.root).expect("글상자 안 표와 제목 글줄");

    let dx = hwpunit_to_px(HORZ_OFFSET + OUTER_MARGIN, 96.0);
    let dy = hwpunit_to_px(VERT_OFFSET + OUTER_MARGIN, 96.0);
    let tolerance = hwpunit_to_px(10, 96.0);
    assert!(
        (table.x - (area.x + dx)).abs() <= tolerance,
        "표 왼쪽은 글 영역 왼쪽 + 가로 오프셋 + 바깥 여백: table.x={} area.x={} dx={}",
        table.x,
        area.x,
        dx
    );
    assert!(
        (table.y - (area.y + dy)).abs() <= tolerance,
        "표 위쪽은 첫 문단 위 + 세로 오프셋 + 바깥 여백: table.y={} area.y={} dy={}",
        table.y,
        area.y,
        dy
    );
    assert!(
        table.x + table.width <= area.x + area.width,
        "표는 글상자 글 영역 안에 있다"
    );
    assert!(
        table.x > title.x + title.width,
        "표가 글상자 왼쪽의 제목 글자와 겹치지 않는다: table.x={} title=[{}, {}]",
        table.x,
        title.x,
        title.x + title.width
    );
}
