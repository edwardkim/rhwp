//! MultiLine의 저장 값과 표시 효과를 분리하는 미해결 렌더링 회귀.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormType};

fn fixture(multiline: bool, text: &str) -> DocumentCore {
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwpx/form-01.hwpx")).unwrap();
    let Control::Form(form) = &mut core.document_mut().sections[0].paragraphs[8].controls[0] else {
        panic!("공개 form-01의 Edit 컨트롤")
    };
    assert_eq!(form.form_type, FormType::Edit);
    form.text = text.into();
    form.properties
        .insert("MultiLine".into(), if multiline { "1" } else { "0" }.into());
    form.properties.insert("PasswordChar".into(), "".into());
    // JSON setter의 별도 escaping 결함을 거치지 않고 실제 저장된 개행을 파싱한다.
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    let Control::Form(form) = &reopened.document().sections[0].paragraphs[8].controls[0] else {
        unreachable!()
    };
    assert_eq!(form.text, text);
    assert_eq!(
        form.properties["MultiLine"],
        if multiline { "1" } else { "0" }
    );
    reopened
}

#[test]
fn multiline_edit_must_not_paint_a_stored_line_break_as_single_line() {
    let multiline = fixture(true, "FIRST\nSECOND");
    let singleline = fixture(false, "FIRST\nSECOND");
    // 이 회귀는 줄 높이·정렬 수치를 정하지 않고 현재 속성의 완전 무시만 검출한다.
    // SVG text 요소 안의 개행 하나는 두 표시 줄을 만들지 않는다.
    assert_ne!(
        multiline.render_page_svg_native(0).unwrap(),
        singleline.render_page_svg_native(0).unwrap(),
        "MultiLine=1의 저장된 두 줄이 한 줄 입력 상자와 똑같이 그려짐",
    );
}

#[test]
fn multiline_edit_without_line_break_keeps_single_line_output() {
    assert_eq!(
        fixture(true, "SINGLE").render_page_svg_native(0).unwrap(),
        fixture(false, "SINGLE").render_page_svg_native(0).unwrap(),
    );
}
