//! 글 사이 `<img>`(data: URI)가 든 HTML 을 붙이면 그 자리에 글자처럼 취급하는 그림이 들어간다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

/// 1×1 PNG
const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";

fn img() -> String {
    format!(r#"<img src="data:image/png;base64,{PNG}" width="20" height="20">"#)
}

fn document_with(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank document");
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).expect("text");
    }
    core
}

fn paragraphs(core: &DocumentCore) -> &[Paragraph] {
    &core.document().sections[0].paragraphs
}

/// 문단 글과, 글자처럼 취급하는 그림마다 그 앞 글자 수
fn text_and_pictures(paragraph: &Paragraph) -> (&str, Vec<usize>) {
    let pictures = paragraph
        .controls
        .iter()
        .zip(paragraph.control_text_positions())
        .filter(|(control, _)| matches!(control, Control::Picture(p) if p.common.treat_as_char))
        .map(|(_, position)| position)
        .collect();
    (paragraph.text.as_str(), pictures)
}

#[test]
fn paragraph_image_between_text_is_pasted_inline_and_saved() {
    let mut core = document_with("");
    core.paste_html_native(0, 0, 0, &format!("<p>앞{}뒤</p>", img()))
        .expect("HTML paste");

    assert_eq!(paragraphs(&core).len(), 1);
    assert_eq!(text_and_pictures(&paragraphs(&core)[0]), ("앞뒤", vec![1]));

    for (format, bytes) in [
        ("HWP", core.export_hwp_native()),
        ("HWPX", core.export_hwpx_native()),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes.expect("export")).expect(format);
        assert_eq!(
            text_and_pictures(&paragraphs(&reopened)[0]),
            ("앞뒤", vec![1]),
            "{format} 저장 뒤에도 그림이 글 사이에 남아야 한다"
        );
    }
}

#[test]
fn paragraph_images_merge_into_the_caret_paragraph_with_their_styles() {
    let mut core = document_with("가나");
    let result = core
        .paste_html_native(
            0,
            0,
            1,
            &format!("<p>{0}앞<b>뒤</b>{0}</p><p>다음</p>", img()),
        )
        .expect("HTML paste");

    assert_eq!(result, r#"{"ok":true,"paraIdx":1,"charOffset":2}"#);
    let paragraphs = paragraphs(&core);
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(text_and_pictures(&paragraphs[0]), ("가앞뒤", vec![1, 3]));
    assert_eq!(text_and_pictures(&paragraphs[1]), ("다음나", vec![]));

    let bold = |char_offset| {
        let id = paragraphs[0]
            .char_shape_id_at(char_offset)
            .expect("글자 모양");
        core.document().doc_info.char_shapes[id as usize].bold
    };
    assert!(!bold(1), "그림 뒤 '앞'은 굵지 않다");
    assert!(bold(2), "<b> 구간은 그림 자리만큼 밀린 '뒤'에서 시작한다");
}

#[test]
fn list_item_image_keeps_its_place_after_the_bullet() {
    let mut core = document_with("");
    core.paste_html_native(0, 0, 0, &format!("<ul><li>앞{}뒤</li></ul>", img()))
        .expect("HTML paste");

    assert_eq!(
        text_and_pictures(&paragraphs(&core)[0]),
        ("• 앞뒤", vec![3])
    );
}
