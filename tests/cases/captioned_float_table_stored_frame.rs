//! 위 캡션을 단 문단 기준 자리차지 표(빈 호스트, 음수 세로 오프셋)의 저장 프레임.
//!
//! 위 캡션 흐름 표의 음수 오프셋은 이미 받은 흐름 원점 앞으로 상자를 올리지 못해 출력에서
//! 0 이다. 저장 빈 호스트 프레임 판정이 같은 흐름 오프셋을 써야 저장 원점(앞 글줄 전진
//! 끝에 바깥 위 여백을 더한 자리)을 받는다. 종전에는 원래 오프셋으로 프레임을 기각해 흐름 배치가 바깥
//! 위 여백만큼 표를 위에 그렸다.
//!
//! 입력 `samples/caption-float-frame/caption-negative-offset.hwp` 는 1342000-202300055
//! 41쪽(표 11)을 발췌해 한/글 2020 으로 저장한 원본이다. 기대값은 같은 저장본의 한/글
//! 2020 Print PDF(`pdf/caption-float-frame/caption-negative-offset-2020.pdf`)와 대조했다(1쪽
//! Native 97.55·fresh WASM 97.74). 98% 미만이라 px 를 고정하지 않고 원본 단위의 관계를
//! 검사한다: 캡션 줄은 호스트 저장 줄 원점(vpos 10400 HU) + 바깥 위 여백(283 HU)에서
//! 시작하고(한/글 Print 캡션 위 274.4px), 표 본체는 캡션 아래에 놓인다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn first_page() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/caption-float-frame/caption-negative-offset.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 2, "한/글 Print 도 2쪽");
    core.build_page_render_tree(0).unwrap().root
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

#[test]
fn captioned_negative_offset_table_takes_the_stored_outer_frame() {
    let root = first_page();
    let mut nodes = Vec::new();
    walk(&root, &mut nodes);
    let body: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .collect();
    assert_eq!(body.len(), 1);
    let body = body[0].bbox;
    let tables: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(3) && t.control_index == Some(0)
                    && t.row_count == 34 && t.col_count == 7)
        })
        .collect();
    assert_eq!(tables.len(), 1, "표 11 은 1쪽에 한 번");
    let table = tables[0].bbox;
    let caption: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::TextLine(_))
                && text(node).replace(' ', "").starts_with("[표11]")
        })
        .collect();
    assert_eq!(caption.len(), 1, "캡션 한 줄");
    let caption = caption[0].bbox;
    // 호스트 문단 3 의 저장 줄 vpos 10400 HU, 표 바깥 위 여백 283 HU(원본 값).
    let frame_top = body.y + (10400.0 + 283.0) * 96.0 / 7200.0;
    assert!(
        (caption.y - frame_top).abs() <= 0.5,
        "캡션 줄 위 {:.3} 는 저장 원점 + 바깥 위 여백 {:.3} 에서 시작해야 한다",
        caption.y,
        frame_top
    );
    assert!(
        caption.y + caption.height <= table.y + 0.5,
        "캡션은 표 위에, 표와 겹치지 않는다"
    );
}
