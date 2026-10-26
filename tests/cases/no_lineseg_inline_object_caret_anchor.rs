//! 저장 줄 없이 다시 연 문단도 개체 뒤 캐럿과 글자 위치를 저장 줄 경로와 같게 둔다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const PNG: &[u8] = include_bytes!("../../assets/logo/logo-16.png");

fn runs(core: &DocumentCore) -> Vec<(String, u64, f64)> {
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["paraIdx"] == 0)
        .map(|run| {
            (
                run["text"].as_str().unwrap().to_string(),
                run["charStart"].as_u64().unwrap(),
                run["fontSize"].as_f64().unwrap(),
            )
        })
        .collect()
}

fn document(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    let made: Value = serde_json::from_str(
        &core
            .insert_picture_native(
                0,
                0,
                1.min(text.chars().count()),
                &[],
                PNG,
                1200,
                400,
                16,
                16,
                "png",
                "",
                None,
                None,
            )
            .unwrap(),
    )
    .unwrap();
    let control = made["controlIdx"].as_u64().unwrap() as usize;
    core.set_picture_properties_native(0, 0, control, r#"{"treatAsChar":true}"#)
        .unwrap();
    core.apply_char_format_native(0, 0, 0, text.chars().count(), r#"{"fontSize":1000}"#)
        .unwrap();
    core
}

#[test]
fn empty_inline_picture_paragraph_keeps_end_anchor_after_hwpx_without_line_cache() {
    let core = document("");
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    // 편집으로 계산한 줄 캐시는 HWPX에 쓰지 않으므로 다시 연 문단은 저장 줄이 없다.
    assert!(reopened.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
    let expected = runs(&core);
    assert_eq!(expected, vec![(String::new(), 1, 13.3)]);
    assert_eq!(runs(&reopened), expected);
    assert_eq!(
        reopened.get_cursor_rect_native(0, 0, 1).unwrap(),
        core.get_cursor_rect_native(0, 0, 1).unwrap()
    );
}

#[test]
fn text_after_inline_picture_keeps_logical_char_start_without_line_cache() {
    let core = document("가나");
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert!(reopened.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
    let starts = |core: &DocumentCore| {
        runs(core)
            .into_iter()
            .filter(|(text, _, _)| !text.is_empty())
            .map(|(text, start, _)| (text, start))
            .collect::<Vec<_>>()
    };
    // 그림이 논리 한 칸을 차지하므로 뒤 글자는 2에서 시작한다.
    assert_eq!(
        starts(&reopened),
        vec![("가".to_string(), 0), ("나".to_string(), 2)]
    );
}
