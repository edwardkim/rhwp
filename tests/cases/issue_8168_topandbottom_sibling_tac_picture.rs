//! [#8168] 같은 문단의 위아래(TopAndBottom) 개체가 글자처럼 취급 그림을 줄 밖으로 끌어내리지 않는다.
//!
//! 결함: 문단에 글자처럼 취급 그림과 떠 있는 위아래 개체가 함께 있으면, 그림 y 에 그 개체의
//! 높이(+바깥 여백)를 무조건 더했다(#1151 보정). 그 보정의 전제는 개체가 이 문단의 흐름
//! 시작(문단 기준, 세로 오프셋 0)에 놓여 그림 줄 위를 차지하는 것이다. 종이 기준으로 줄 아래에
//! 놓인 글상자에도 적용돼, 줄의 글자는 제자리인데 그림만 글상자 자리로 내려가 겹쳤다.
//!
//! 독립 기준: 같은 편집을 저장한 HWPX 를 한/글 2024(13.0.0.564) Print 로 출력하면 「앞 [그림]
//! 글 뒤 글」이 한 줄이고, 글상자는 그 줄 아래 지정 위치에 있다. 반례: #1151 의 한/글 산출물
//! `samples/tac-verify/scenario-a-after.hwp`(문단 기준 오프셋 0 의 위아래 표 + 글자처럼 그림)는
//! 그림을 표 아래에 둔다 — 그 동작은 그대로다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn find(node: &RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<BoundingBox> {
    if pred(node) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

fn line_with(node: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if matches!(
        node.node_type,
        RenderNodeType::TextBox | RenderNodeType::Table(_)
    ) {
        return None;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(needle) {
        return Some(node.bbox);
    }
    node.children
        .iter()
        .find_map(|child| line_with(child, needle))
}

fn json_usize(json: &str, key: &str) -> usize {
    let at = json.find(&format!("\"{key}\":")).expect(key) + key.len() + 3;
    json[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .expect(key)
}

/// 이슈의 재현과 같은 편집: 「앞 글 뒤 글」 둘째 글자 자리에 글자처럼 그림, 같은 문단에
/// 종이 기준 세로 15000HU 의 위아래 글상자.
fn edited_with_paper_float() -> DocumentCore {
    let jpg = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("rhwp-studio/public/samples/images/moogung.jpg"),
    )
    .expect("jpg");
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, "앞 글 뒤 글")
        .expect("text");
    core.split_paragraph_native(0, 0, 7, None).expect("split");
    core.insert_text_native(0, 1, 0, "다음 문단")
        .expect("text 2");
    let pic = core
        .insert_picture_native(
            0,
            0,
            2,
            &[],
            &jpg,
            8504,
            4447,
            1000,
            523,
            "jpg",
            "그림",
            None,
            None,
        )
        .expect("picture");
    core.set_picture_properties_native(
        0,
        json_usize(&pic, "paraIdx"),
        json_usize(&pic, "controlIdx"),
        r#"{"treatAsChar":true}"#,
    )
    .expect("tac");
    let margin_left = core.document().sections[0].section_def.page_def.margin_left;
    core.create_shape_control_native(
        0,
        0,
        7,
        17008,
        5669,
        margin_left,
        15000,
        false,
        "TopAndBottom",
        "textbox",
        false,
        false,
        &[],
    )
    .expect("float textbox");
    core
}

#[test]
fn paper_relative_topandbottom_float_keeps_tac_picture_in_its_line() {
    let core = edited_with_paper_float();
    let tree = core.build_page_render_tree(0).expect("page 1");
    let image = find(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::Image(_))
    })
    .expect("그림");
    let line = line_with(&tree.root, "앞").expect("그림 host 줄");
    let float_h = 5669.0 * 96.0 / 7200.0;
    let float = find(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::Rectangle(_)) && (n.bbox.height - float_h).abs() < 1.0
    })
    .expect("위아래 글상자");

    assert!(
        image.y >= line.y - 0.5 && image.y + image.height <= line.y + line.height + 0.5,
        "그림({:.1}–{:.1})이 host 줄({:.1}–{:.1}) 안에 남는다",
        image.y,
        image.y + image.height,
        line.y,
        line.y + line.height
    );
    assert!(
        image.y + image.height <= float.y + 0.5,
        "그림 아래({:.1})가 글상자 위({:.1})보다 위 — 겹치지 않는다",
        image.y + image.height,
        float.y
    );
}

/// 반례(#1151): 문단 기준·오프셋 0 의 위아래 표는 같은 문단의 글자처럼 그림을 표 아래로 보낸다.
#[test]
fn para_relative_topandbottom_table_at_para_start_still_puts_picture_below() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/tac-verify/scenario-a-after.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
    let tree = core.build_page_render_tree(0).expect("page 1");
    let image = find(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::Image(_))
    })
    .expect("그림");
    let table = find(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::Table(_))
    })
    .expect("표");
    assert!(
        image.y >= table.y + table.height - 0.5,
        "그림 위({:.1})가 표 아래({:.1}) 이후",
        image.y,
        table.y + table.height
    );
}
