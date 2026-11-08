//! #8160: sample16 전용 회귀. 이전 등록: tests/cases/issue_4367_hwp3_convert_fourth_contract.rs
use rhwp::model::{control::Control, shape::ShapeObject};
use rhwp::parser::FileFormat;
const SAMPLE: &str = "samples/hwp3-sample16.hwp";

fn convert_adapted() -> rhwp::model::document::Document {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    doc
}

/// 계약 1 — 글상자 사각형의 storage 필드가 한컴 저장본 계약대로 채워진다.
#[test]
fn issue_8160_textbox_rect_storage_fields_are_materialized() {
    let doc = convert_adapted();
    let p5 = &doc.sections[0].paragraphs[5];
    let Some(Control::Shape(shape)) = p5.controls.first() else {
        panic!("샘플 전제: 문단 5 는 글상자 사각형");
    };
    let ShapeObject::Rectangle(rect) = shape.as_ref() else {
        panic!("샘플 전제: Rectangle");
    };
    assert_eq!(
        rect.drawing.shape_attr.flip & 0x0108_0000,
        0x0108_0000,
        "글상자 storage flip 비트 — 이것이 개방 거부의 결정타였다 (COM 실측)"
    );
    assert!(
        rect.drawing.shape_attr.rotation_center.x > 0
            && rect.drawing.shape_attr.rotation_center.y > 0,
        "회전중심 (w/2, h/2)"
    );
    assert!(
        rect.x_coords
            .iter()
            .chain(rect.y_coords.iter())
            .any(|&v| v != 0),
        "SC_RECT 꼭짓점 — 한컴 저장본은 (0,0)(w,0)(w,h)(0,h)"
    );
    let tb = rect.drawing.text_box.as_ref().expect("글상자");
    assert!(tb.max_width > 0, "LIST_HEADER 최대 폭");
}

/// 계약 2 — 수식: 크기·font_size·글꼴·baseline 이 한컴 계약 범위다.
#[test]
fn issue_8160_equation_eqedit_contract_is_normalized() {
    let doc = convert_adapted();
    let mut checked = 0usize;
    for p in &doc.sections[0].paragraphs {
        for c in &p.controls {
            if let Control::Equation(eq) = c {
                checked += 1;
                assert!(
                    eq.common.width > 0 && eq.common.height > 0,
                    "수식 크기 0 금지 (개체 헤더 42..46)"
                );
                assert!(eq.font_size > 0, "font_size=0 이면 한글 2022 크래시");
                assert!(!eq.font_name.is_empty(), "수식 글꼴 공란 금지");
                assert!(
                    (0..=100).contains(&eq.baseline),
                    "baseline 은 % 축 (한컴 67)"
                );
            }
        }
    }
    assert!(checked >= 2, "샘플 전제: 수식 2개");
}
