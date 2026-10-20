//! 글자처럼 취급하는 양식 개체도 수식·그림처럼 캐럿 한 칸을 차지한다.
//!
//! 양식 개체가 칸을 갖지 못하면 개체 뒤 캐럿이 다음 글자 뒤로 건너뛰고, 개체 앞에 친 글이
//! 개체 뒤로 들어간다.

use rhwp::model::control::{Control, FormObject, FormType};
use rhwp::wasm_api::HwpDocument;

/// "가[양식 개체]나" 문단을 저장했다가 다시 연다.
fn fixture() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "가나").unwrap();
    // 양식 개체 삽입 API 가 없으므로 같은 확장 문자 자리를 쓰는 수식을 넣고 바꾼다.
    doc.insert_equation_native(0, 0, 1, "x", 1000, 0).unwrap();
    let para = &mut doc.document_mut().sections[0].paragraphs[0];
    let ci = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Equation(_)))
        .unwrap();
    let mut form = FormObject {
        form_type: FormType::PushButton,
        name: "단추".into(),
        caption: "확인".into(),
        width: 7087,
        height: 1984,
        enabled: true,
        ..Default::default()
    };
    form.common.width = 7087;
    form.common.height = 1984;
    form.common.treat_as_char = true;
    para.controls[ci] = Control::Form(Box::new(form));
    HwpDocument::from_bytes(&doc.export_hwp().unwrap()).unwrap()
}

fn form_logical_pos(doc: &HwpDocument) -> usize {
    let para = &doc.document().sections[0].paragraphs[0];
    let ci = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Form(_)))
        .expect("양식 개체가 남아 있어야 한다");
    para.logical_control_positions()[ci]
}

fn caret_x_y(doc: &HwpDocument, offset: usize) -> (f64, f64) {
    let rect: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect_native(0, 0, offset).unwrap()).unwrap();
    (rect["x"].as_f64().unwrap(), rect["y"].as_f64().unwrap())
}

#[test]
fn form_object_has_caret_slot() {
    let doc = fixture();
    assert_eq!(form_logical_pos(&doc), 1);
    // 가(0) [양식](1) 나(2) 끝(3): 네 캐럿이 한 줄에서 차례로 오른쪽에 놓인다.
    let carets: Vec<_> = (0..=3).map(|offset| caret_x_y(&doc, offset)).collect();
    for pair in carets.windows(2) {
        assert_eq!(
            pair[0].1, pair[1].1,
            "캐럿이 한 줄에 있어야 한다: {carets:?}"
        );
        assert!(
            pair[0].0 < pair[1].0,
            "캐럿이 차례로 오른쪽이어야 한다: {carets:?}"
        );
    }
    // 양식 개체 앞과 뒤 캐럿 사이는 개체 너비(7087HU ≈ 94.5px)만큼 벌어진다.
    assert!(carets[2].0 - carets[1].0 > 90.0, "{carets:?}");
}

#[test]
fn typing_before_form_object_stays_before_it() {
    let mut doc = fixture();
    doc.insert_text_native(0, 0, 1, "다").unwrap();
    assert_eq!(doc.document().sections[0].paragraphs[0].text, "가다나");
    assert_eq!(
        form_logical_pos(&doc),
        2,
        "새 글자는 양식 개체 앞에 들어가야 한다"
    );
}

#[test]
fn enter_after_form_object_keeps_it_in_first_paragraph() {
    let mut doc = fixture();
    // 양식 개체 뒤(캐럿 2)에서 문단을 나눈다.
    doc.split_paragraph_native(0, 0, 2, None).unwrap();
    let paras = &doc.document().sections[0].paragraphs;
    let has_form = |pi: usize| {
        paras[pi]
            .controls
            .iter()
            .any(|c| matches!(c, Control::Form(_)))
    };
    assert_eq!(
        (paras[0].text.as_str(), paras[1].text.as_str()),
        ("가", "나")
    );
    assert!(
        has_form(0) && !has_form(1),
        "양식 개체는 앞 문단에 남아야 한다"
    );
}
