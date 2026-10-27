//! 독립 한컴 2020 Print와 Native255/fresh WASM073 전 1쪽 raw 100%·직접 판독을 확인했다.
//! 원본 소유·순서·크기 관계를 유지하며 중요한 글자 원점만 독립 PDF의 ±2px로 보호한다.
//! 입력/PDF 해시·TSV·판독과 수정 전 FAIL/수정 후 PASS는
//! mydocs/working/task_semanticist21_20261026_inline_wrap_contracts.md에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

const HWP: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pr8093_form_caret/form-slot.hwp"
));
const HWPX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pr8093_form_caret/form-slot.hwpx"
));

fn fixture() -> HwpDocument {
    HwpDocument::from_bytes(HWP).unwrap()
}

fn form_logical_pos(doc: &HwpDocument) -> usize {
    let para = &doc.document().sections[0].paragraphs[0];
    let ci = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Form(_)))
        .unwrap();
    para.logical_control_positions()[ci]
}

fn caret(doc: &HwpDocument, offset: usize) -> (f64, f64) {
    let rect: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect_native(0, 0, offset).unwrap()).unwrap();
    (rect["x"].as_f64().unwrap(), rect["y"].as_f64().unwrap())
}

#[test]
fn verified_form_caret_order_and_source_width_ownership() {
    let doc = fixture();
    assert_eq!(form_logical_pos(&doc), 1);
    let carets: Vec<_> = (0..=3).map(|offset| caret(&doc, offset)).collect();
    for pair in carets.windows(2) {
        assert_eq!(pair[0].1, pair[1].1, "같은 글줄의 캐럿: {carets:?}");
        assert!(pair[0].0 < pair[1].0, "논리 순서 보존: {carets:?}");
    }
    let source = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Form(form) = c {
                Some(form)
            } else {
                None
            }
        })
        .unwrap();
    // 독립 Print에서도 같은 개체가 두 글자 사이를 차지한다. 화면의 90px
    // 하한 대신 원본 개체 폭과 실제 캐럿 구간을 문서 단위로 대조한다.
    let actual_hu = (carets[2].0 - carets[1].0) * 7200.0 / 96.0;
    assert!(
        (actual_hu - f64::from(source.width)).abs() <= 1.0,
        "원본 폭의 단일 소유"
    );
}

#[test]
fn verified_typing_before_form_preserves_text_and_control_order() {
    let mut doc = fixture();
    doc.insert_text_native(0, 0, 1, "다").unwrap();
    assert_eq!(doc.document().sections[0].paragraphs[0].text, "가다나");
    assert_eq!(form_logical_pos(&doc), 2);
}

#[test]
fn verified_split_after_form_preserves_original_paragraph_ownership() {
    let mut doc = fixture();
    doc.split_paragraph_native(0, 0, 2, None).unwrap();
    let paras = &doc.document().sections[0].paragraphs;
    assert_eq!(
        (paras[0].text.as_str(), paras[1].text.as_str()),
        ("가", "나")
    );
    let has_form = |index: usize| {
        paras[index]
            .controls
            .iter()
            .any(|c| matches!(c, Control::Form(_)))
    };
    assert!(has_form(0) && !has_form(1));
}

#[test]
fn verified_form_mixed_text_baseline_matches_independent_print() {
    // 이 1쪽은 양쪽 raw 100%와 review·overlay 확대 판독을 마쳤다.
    // 독립 Print의 원점은 pdf/pr8093_review/form-slot-*-2020.pdf 1쪽의
    // PDF pt 좌표다. 관계 검사는 위의 순서·폭·편집·문단 소유에 유지한다.
    for bytes in [HWP, HWPX] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        assert_eq!(core.page_count(), 1, "독립 Print 전체 1쪽");
        let svg = core.render_page_svg_native(0).unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        for (text, x_pt) in [("가", 85.0339126586914), ("나", 165.63023376464844)] {
            let nodes: Vec<_> = xml
                .descendants()
                .filter(|n| {
                    n.is_element() && n.tag_name().name() == "text" && n.text() == Some(text)
                })
                .collect();
            assert_eq!(nodes.len(), 1, "글자의 단일 표시 소유");
            let x = nodes[0].attribute("x").unwrap().parse::<f64>().unwrap();
            let y = nodes[0].attribute("y").unwrap().parse::<f64>().unwrap();
            for (actual, expected) in [
                (x, x_pt * 96.0 / 72.0),
                (y, 116.03319549560547 * 96.0 / 72.0),
            ] {
                assert!(
                    (actual - expected).abs() <= 2.0,
                    "독립 Print 원점: {text}, 실제={actual}, 기준={expected}"
                );
            }
        }
    }
}
