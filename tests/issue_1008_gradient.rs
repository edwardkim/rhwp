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

/// 격차 C: HWP3 heading decoration ("═══■ 1.추진목적 ■═══") strip 회귀 가드.
/// fixup_hwp3_heading_decoration 후 paragraph.text 가 decoration 제거된 상태인지 단언.
#[test]
fn hwp3_sample16_heading_decoration_stripped() {
    let bytes = std::fs::read("samples/hwp3-sample16.hwp").expect("read hwp3-sample16.hwp");
    let doc = parse_hwp3(&bytes).expect("parse hwp3-sample16.hwp");

    // pi=70 이 1.추진목적 heading paragraph
    let para = &doc.sections[0].paragraphs[70];
    assert!(
        !para.text.contains('═'),
        "decoration char ═ 잔존 — heading strip 회귀 (text={:?})",
        para.text
    );
    assert!(
        !para.text.contains('■'),
        "marker char ■ 잔존 — heading strip 회귀 (text={:?})",
        para.text
    );
    assert!(
        para.text.contains("추진목적"),
        "heading 본문 보존 (text={:?})",
        para.text
    );
}

/// 격차 B: HWP3 Shape border LineType 2~7 → Solid normalization 회귀 가드.
/// HWP3 raw line_style=2 (Dash per spec) 인 sample16 사업개요 박스 (pi=71) 의
/// border attr 가 1 (Solid) 로 normalize 되어야 함.
#[test]
fn hwp3_sample16_business_box_border_solid() {
    let bytes = std::fs::read("samples/hwp3-sample16.hwp").expect("read hwp3-sample16.hwp");
    let doc = parse_hwp3(&bytes).expect("parse hwp3-sample16.hwp");

    let para = &doc.sections[0].paragraphs[71];
    let shape = para
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(s) => Some(s.as_ref()),
            _ => None,
        })
        .expect("pi=71 Shape control 존재");

    let rect = match shape {
        ShapeObject::Rectangle(r) => r,
        other => panic!("expected Rectangle, got {:?}", other),
    };

    let line_type = (rect.drawing.border_line.attr as u32) & 0x3F;
    assert_eq!(
        line_type, 1,
        "HWP3 raw line_style=2 (Dash) 가 1 (Solid) 로 normalize 되어야 함 (격차 B)"
    );
}

/// 격차 D: HWP3 legacy 폰트명 ("신명조" 등) → 한컴 변환기 정합 명칭 ("HY신명조" 등)
/// 매핑 회귀 가드. font_faces[0] 의 "신명조" 가 "HY신명조" 로 매핑되어야 함.
#[test]
fn hwp3_sample16_font_name_mapped_to_hwp5_convention() {
    let bytes = std::fs::read("samples/hwp3-sample16.hwp").expect("read hwp3-sample16.hwp");
    let doc = parse_hwp3(&bytes).expect("parse hwp3-sample16.hwp");

    let group0 = doc
        .doc_info
        .font_faces
        .first()
        .expect("group 0 (한글) 존재");

    // HWP3 raw idx 1 = "신명조" 가 "HY신명조" 로 매핑된 후 alt_name 에 원본 보존
    let f1 = &group0[1];
    assert_eq!(
        f1.name, "HY신명조",
        "신명조 → HY신명조 매핑 (격차 D, 한컴 변환기 mimic)"
    );
    assert_eq!(
        f1.alt_name.as_deref(),
        Some("신명조"),
        "alt_name 에 원본 폰트명 보존"
    );

    // 다른 매핑 단언
    assert_eq!(group0[0].name, "HY고딕", "고딕 → HY고딕");
    assert_eq!(group0[2].name, "HY중고딕", "중고딕 → HY중고딕");
    assert_eq!(group0[3].name, "HY견고딕", "견고딕 → HY견고딕");
    assert_eq!(group0[4].name, "HY그래픽", "그래픽 → HY그래픽");
}
