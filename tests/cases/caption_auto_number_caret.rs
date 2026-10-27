//! 캡션 자동 번호 뒤 글자의 캐럿·클릭 위치는 그 글자가 그려진 자리와 같다.
//!
//! 캡션 조판이 번호 글자를 줄 글에 끼우면서 글자 위치 대응은 그대로 두어, 번호 뒤
//! 캐럿이 한 글자 앞에 그려지고 클릭은 한 글자 뒤에 떨어졌다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// 캡션 `그림 [번호]수달`(자리 0–2 `그림 `, 3 번호 자리표, 4–5 `수달`)을 단 그림 문서.
fn captioned() -> HwpDocument {
    let png =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/logo/logo-16.png"))
            .expect("tiny png");
    let mut doc = HwpDocument::create_empty();
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &png,
        9600,
        9600,
        16,
        16,
        "png",
        "",
        None,
        None,
    )
    .expect("그림 삽입");
    let result = doc
        .set_picture_properties_native(0, 0, 0, r#"{"hasCaption":true}"#)
        .expect("캡션 넣기");
    assert!(result.contains(r#""ok":true"#), "{result}");
    doc.insert_text_in_cell_native(0, 0, 0, 0, 0, 4, "수달")
        .expect("캡션 글 입력");
    doc
}

fn caret(doc: &HwpDocument, offset: usize) -> Value {
    let json = doc
        .get_cursor_rect_in_cell_native(0, 0, 0, 0, 0, offset)
        .expect("캡션 캐럿");
    serde_json::from_str(&json).unwrap()
}

/// 그려진 캡션 글 `…수달`의 오른쪽 끝 x.
fn caption_text_end(doc: &HwpDocument) -> f64 {
    fn find(node: &RenderNode) -> Option<f64> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.ends_with("수달") {
                return Some(node.bbox.x + node.bbox.width);
            }
        }
        node.children.iter().find_map(find)
    }
    find(&doc.build_page_render_tree(0).unwrap().root).expect("캡션 글")
}

#[test]
fn caret_after_caption_number_sits_on_drawn_glyph() {
    let doc = captioned();
    let x = |offset| caret(&doc, offset)["x"].as_f64().unwrap();
    // `수달` 뒤 캐럿은 그려진 글 끝에, `수` 뒤 캐럿은 `달` 왼쪽 끝에 있어야 한다.
    let end = caption_text_end(&doc);
    let dal = x(6) - x(5);
    assert!(
        (x(6) - end).abs() < 0.5,
        "끝 캐럿 {:.2}, 글 끝 {end:.2}",
        x(6)
    );
    assert!(
        (x(5) - (end - dal)).abs() < 0.5,
        "수 뒤 캐럿 {:.2}, 달 왼쪽 {:.2}",
        x(5),
        end - dal
    );
}

#[test]
fn click_after_caption_number_lands_on_clicked_glyph() {
    let doc = captioned();
    let at = caret(&doc, 6);
    let dal = at["x"].as_f64().unwrap() - caret(&doc, 5)["x"].as_f64().unwrap();
    // 그려진 `수`의 오른쪽 끝 바로 안쪽을 누르면 `수` 뒤(5)에 캐럿이 와야 한다.
    let x = caption_text_end(&doc) - dal - 2.0;
    let y = at["y"].as_f64().unwrap() + at["height"].as_f64().unwrap() / 2.0;
    let hit: Value = serde_json::from_str(&doc.hit_test_native(0, x, y).unwrap()).unwrap();
    assert_eq!(hit["charOffset"], 5, "{hit}");
}
