//! 개체 뒤 마지막 글자 조각도 반올림하지 않은 폭으로 다음 런을 놓는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const PNG: &[u8] = include_bytes!("../../assets/logo/logo-16.png");

#[test]
fn run_after_an_inline_object_tail_starts_at_the_caret() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가🦦나다라").unwrap();
    let inserted: Value = serde_json::from_str(
        &core
            .insert_picture_native(0, 0, 1, &[], PNG, 1300, 1400, 16, 16, "png", "", None, None)
            .unwrap(),
    )
    .unwrap();
    let control = inserted["controlIdx"].as_u64().unwrap() as usize;
    core.set_picture_properties_native(0, 0, control, r#"{"treatAsChar":true}"#)
        .unwrap();
    // 굵게 런이 개체를 감싸 "가" | 개체 | "🦦" 조각으로 나뉘고, 이모지 폭은 정수 px가 아니다.
    core.apply_char_format_native(0, 0, 0, 2, r#"{"bold":true}"#)
        .unwrap();

    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    let runs = layout["runs"].as_array().unwrap();
    let emoji = runs.iter().find(|run| run["text"] == "🦦").unwrap();
    let next = runs.iter().find(|run| run["text"] == "나다라").unwrap();
    let emoji_end = emoji["x"].as_f64().unwrap() + emoji["charX"][1].as_f64().unwrap();
    let caret: Value =
        serde_json::from_str(&core.get_cursor_rect_native(0, 0, 3).unwrap()).unwrap();

    assert!(
        (next["x"].as_f64().unwrap() - emoji_end).abs() <= 0.1,
        "다음 런은 이모지 글자 경계에서 시작한다: {next} / {emoji}"
    );
    assert!(
        (next["x"].as_f64().unwrap() - caret["x"].as_f64().unwrap()).abs() <= 0.1,
        "다음 런은 캐럿 자리에서 시작한다: {next} / {caret}"
    );
}
