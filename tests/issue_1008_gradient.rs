//! Issue #1008 격차 A: HWP3 Shape 박스 배경 gradient IR 매핑 회귀 가드
//!
//! HWP3 raw stream 의 Hwp3DrawingObjectGradientAttr 는 이미 파싱되지만 (drawing.rs:149~170,
//! basic_attr.has_gradient() 시 read), drawing.rs:792~806 의 Fill IR 구축에서 종전
//! `fill_type=Solid, gradient=None` 으로 하드코딩되어 데이터가 무시되어왔음. 한컴 한글
//! 정답지는 보라/라벤더 gradient — 본 회귀 가드는 HWP3 sample16 사업개요 박스 (pi=71)
//! 의 fill.fill_type == Gradient 및 fill.gradient.is_some() 단언.

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::model::style::FillType;
use rhwp::parser::hwp3::parse_hwp3;

#[test]
fn hwp3_sample16_business_box_has_gradient() {
    let bytes = std::fs::read("samples/hwp3-sample16.hwp").expect("read hwp3-sample16.hwp");
    let doc = parse_hwp3(&bytes).expect("parse hwp3-sample16.hwp");

    // pi=71 사업개요 본문 박스 (1.추진목적) — Shape control 1 개
    let para = &doc.sections[0].paragraphs[71];
    let shape = para
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(s) => Some(s.as_ref()),
            _ => None,
        })
        .expect("pi=71 에 Shape control 존재");

    // 사각형 Shape 의 fill 단언
    let rect = match shape {
        ShapeObject::Rectangle(r) => r,
        other => panic!("expected Rectangle, got {:?}", other),
    };

    assert_eq!(
        rect.drawing.fill.fill_type,
        FillType::Gradient,
        "HWP3 Shape gradient_attr 이 IR Fill 에 매핑되어야 함 (격차 A)"
    );

    let grad = rect
        .drawing
        .fill
        .gradient
        .as_ref()
        .expect("fill.gradient is Some");
    assert!(
        !grad.colors.is_empty(),
        "gradient.colors 2-stop 이상 (start + end)"
    );
    assert_eq!(
        grad.colors.len(),
        2,
        "HWP3 raw 는 start_color + end_color 2-stop"
    );
}
