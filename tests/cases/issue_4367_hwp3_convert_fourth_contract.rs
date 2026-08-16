//! [Issue #4367] hwp3-sample16 변환본을 한컴이 열지 못하던 네 번째 저장 계약.
//!
//! COM 문단-이등분 실측(한글 2022, 문서당 프로세스 격리)으로 두 발동체를
//! 확정했다 — 정답지는 한컴 자체 변환본(samples/hwp3-sample16-hwp5.hwp)과의
//! 레코드 바이트 대조다.
//!
//! 1. **글상자 사각형의 storage 필드 공란** (문단 5, 개방 거부) — SHAPE_COMPONENT
//!    storage flip `0x0108_0000`(글상자 0x0100_0000 + 0x0008_0000)과 회전중심
//!    (w/2, h/2)이 결정타였고(이 둘을 채우는 순간 개방), SC_RECT 꼭짓점
//!    `(0,0)(w,0)(w,h)(0,h)`·글상자 LIST_HEADER 최대 폭도 한컴 계약대로 채운다
//!    (#3676 계약 ②·③ 의 사각형 판 — 그림·local_file_version 만 덮여 있었다).
//! 2. **수식 EQEDIT** (문단 155, 크래시/RPC 붕괴) — 크기 0(개체 헤더 42..46 을
//!    파서가 안 읽음)·font_size=0·baseline 범위 밖(465)·수식 글꼴 공란이면
//!    한글 2022 가 죽는다. 한컴: 1200 / 67(%) / "HYhwpEQ".
//!
//! CI 에는 한컴이 없으므로 어댑터 IR 에서 계약을 검사한다(#3676 과 동형).

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
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
fn textbox_rect_storage_fields_are_materialized() {
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
fn equation_eqedit_contract_is_normalized() {
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

/// 다섯 번째 계약 (hwp3-sample11, 같은 COM 이등분 기법) — 다각형 꼭짓점.
///
/// HWP3 파서가 점 배열을 읽고도 `PolygonShape::default()` 로 버려 SC_POLYGON 이
/// 점 0개(8B)로 저장됐고, 한글 2022 는 빈 다각형이 든 문서를 통째로 거부했다
/// (p1809 Polygon 이 발동체 — N=1809 열림/1810 거부). 점을 실으면 전문서가
/// 열린다(OPEN_OK 207,570자).
#[test]
fn hwp3_polygon_points_are_loaded() {
    use rhwp::model::shape::ShapeObject;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample11.hwp");
    let raw = std::fs::read(&path).expect("read sample11");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    fn walk(paragraphs: &[rhwp::model::paragraph::Paragraph], seen: &mut usize, empty: &mut usize) {
        for p in paragraphs {
            for c in &p.controls {
                if let Control::Shape(s) = c {
                    match s.as_ref() {
                        ShapeObject::Polygon(poly) => {
                            *seen += 1;
                            if poly.points.is_empty() {
                                *empty += 1;
                            }
                        }
                        ShapeObject::Group(g) => {
                            for child in &g.children {
                                if let ShapeObject::Polygon(poly) = child {
                                    *seen += 1;
                                    if poly.points.is_empty() {
                                        *empty += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let (mut seen, mut empty) = (0usize, 0usize);
    for sec in &doc.sections {
        walk(&sec.paragraphs, &mut seen, &mut empty);
    }
    assert!(seen > 0, "샘플 전제: 다각형이 있어야 한다");
    assert_eq!(
        empty, 0,
        "점 0개 SC_POLYGON 은 한글 2022 가 문서 전체를 거부한다"
    );
}
