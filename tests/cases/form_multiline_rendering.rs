//! 저장된 개행과 입력 상자의 Print 표시 문자열을 구분한다.
//! 독립 한컴 Print는 같은 FIRST LF SECOND 입력에서 MultiLine=0/1 모두 FIRST만 표시한다.
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
fn edit_print_displays_first_stored_line_for_both_multiline_values() {
    let multiline = fixture(true, "FIRST\nSECOND");
    let singleline = fixture(false, "FIRST\nSECOND");
    let outputs = [multiline, singleline].map(|doc| doc.render_page_svg_native(0).unwrap());
    // 동일성만 검사하면 두 출력에 SECOND를 잘못 이어 그린 이전 코드도 통과한다.
    // 원문·속성 보존은 fixture에서, 실제 표시 문자열은 출력에서 독립적으로 확인한다.
    for svg in &outputs {
        assert!(
            svg.contains(">FIRST</text>"),
            "Print에 첫 줄이 표시되어야 한다"
        );
        assert!(
            !svg.contains("SECOND"),
            "Print에 저장된 다음 줄을 이어 그리지 않는다"
        );
    }
    assert_eq!(outputs[0], outputs[1], "이 입력의 독립 Print 두 장은 같다");
}

#[test]
fn multiline_edit_without_line_break_keeps_single_line_output() {
    assert_eq!(
        fixture(true, "SINGLE").render_page_svg_native(0).unwrap(),
        fixture(false, "SINGLE").render_page_svg_native(0).unwrap(),
    );
}
