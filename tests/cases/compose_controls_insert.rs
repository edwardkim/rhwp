//! 덧말 넣기·글자 겹치기(한컴 Mac 12.30 입력 메뉴)로 만든 컨트롤을 그리고 저장한다.
//!
//! 한컴 12.30 저장본 기준: 10pt 줄에 위 덧말을 넣으면 줄 높이 1500·기준선 1350·간격 900이고,
//! HWP 원문 배열은 겹칠 글자 앞에 테두리 글자를 둔다. HWPX `composeText`는 친 글 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn document() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "가나").unwrap();
    doc
}

fn page_text(core: &DocumentCore) -> String {
    let svg = core.render_page_svg_native(0).unwrap();
    let mut text = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        text.push_str(&rest[end + 1..close]);
        rest = &rest[close + 7..];
    }
    text
}

fn saved(doc: &HwpDocument) -> [(&'static str, DocumentCore); 2] {
    [
        (
            "HWP",
            DocumentCore::from_bytes(&doc.export_hwp_with_adapter_snapshot().unwrap()).unwrap(),
        ),
        (
            "HWPX",
            DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap(),
        ),
    ]
}

#[test]
fn ruby_draws_both_texts_raises_the_line_and_survives_saving() {
    let mut doc = document();
    let result = doc
        .insert_ruby_native(0, 0, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    assert!(result.contains(r#""controlIdx":"#), "{result}");
    let line = doc.document().sections[0].paragraphs[0].line_segs[0].clone();
    assert_eq!(
        (line.line_height, line.baseline_distance, line.line_spacing),
        (1500, 1350, 900)
    );
    let text = page_text(&doc);
    for glyph in ['가', '漢', '字', '한', '자', '나'] {
        assert!(text.contains(glyph), "{glyph} 없음: {text}");
    }
    for (format, core) in saved(&doc) {
        let ruby = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Ruby(ruby) => Some(ruby.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{format}: 덧말 없음"));
        assert_eq!(
            (
                ruby.main_text.as_str(),
                ruby.ruby_text.as_str(),
                ruby.pos_type,
                ruby.align
            ),
            ("漢字", "한자", 0, 3),
            "{format}"
        );
        assert_eq!(core.document().sections[0].paragraphs[0].text, "가나");
    }
}

#[test]
fn ruby_edit_moves_it_below_with_a_style_and_reads_back() {
    let mut doc = document();
    doc.insert_ruby_native(0, 0, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    let index = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Ruby(_)))
        .unwrap();
    doc.set_compose_control_native(
        0,
        0,
        index,
        r#"{"mainText":"大韓","rubyText":"대한","below":true,"styleId":1}"#,
    )
    .unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&doc.get_compose_control_native(0, 0, index).unwrap()).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"kind":"ruby","mainText":"大韓","rubyText":"대한","below":true,"styleId":1})
    );
    // 아래 덧말은 기준선을 두고 줄을 늘인다.
    let line = doc.document().sections[0].paragraphs[0].line_segs[0].clone();
    assert_eq!((line.line_height, line.baseline_distance), (1408, 850));
}

#[test]
fn char_overlap_insert_writes_hancom_wire_chars_and_plain_hwpx_text() {
    let mut doc = document();
    doc.insert_char_overlap_native(0, 0, 1, r#"{"text":"12","shape":5}"#)
        .unwrap();
    let index = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::CharOverlap(_)))
        .unwrap();
    assert!(page_text(&doc).contains('△'));
    for (format, core) in saved(&doc) {
        let Control::CharOverlap(overlap) =
            &core.document().sections[0].paragraphs[0].controls[index]
        else {
            panic!("{format}: 겹침 없음");
        };
        let expected: &[char] = if format == "HWP" {
            &['△', '1', '2']
        } else {
            &['1', '2']
        };
        assert_eq!(overlap.chars, expected, "{format}");
        assert_eq!((overlap.border_type, overlap.inner_char_size), (5, -3));
        let value: serde_json::Value =
            serde_json::from_str(&core.get_compose_control_native(0, 0, index).unwrap()).unwrap();
        assert_eq!(value["text"], "12", "{format}");
    }
    // HWP 저장본을 다시 HWPX로 내도 표지 글자는 빠진다.
    let hwp = DocumentCore::from_bytes(&doc.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&hwp.export_hwpx_native().unwrap()).unwrap();
    let Control::CharOverlap(overlap) = &hwpx.document().sections[0].paragraphs[0].controls[index]
    else {
        panic!("HWP→HWPX: 겹침 없음");
    };
    assert_eq!(overlap.chars, ['1', '2']);
}

#[test]
fn deleting_keeps_the_ruby_main_text_only_when_asked() {
    let mut doc = document();
    doc.insert_ruby_native(0, 0, 1, r#"{"mainText":"漢字","rubyText":"한자"}"#)
        .unwrap();
    doc.insert_char_overlap_native(0, 0, 2, r#"{"text":"가","shape":1}"#)
        .unwrap();
    let index = |doc: &HwpDocument, ruby: bool| {
        doc.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|control| match control {
                Control::Ruby(_) => ruby,
                Control::CharOverlap(_) => !ruby,
                _ => false,
            })
            .unwrap()
    };
    let overlap = index(&doc, false);
    doc.delete_compose_control_native(0, 0, overlap, false)
        .unwrap();
    assert_eq!(doc.document().sections[0].paragraphs[0].text, "가나");
    let ruby = index(&doc, true);
    doc.delete_compose_control_native(0, 0, ruby, true).unwrap();
    let para = &doc.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "가漢字나");
    assert!(!para
        .controls
        .iter()
        .any(|control| matches!(control, Control::Ruby(_) | Control::CharOverlap(_))));
    for (format, core) in saved(&doc) {
        assert_eq!(
            core.document().sections[0].paragraphs[0].text,
            "가漢字나",
            "{format}"
        );
    }
}
