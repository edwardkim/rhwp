//! Issue #2727: 수식 EQEDIT attribute(UINT32) 왕복 보존 회귀 가드.
//!
//! 정정 전 동작:
//! - `src/parser/control.rs::parse_equation_control` 이 EQEDIT 선두 UINT32 를
//!   `let _attr = ...` 로 버렸다.
//! - `src/serializer/control.rs::serialize_equation_control` 이 같은 자리에 상수 `0` 을 썼다.
//! - `src/parser/hwpx/section.rs::parse_equation` 이 `lineMode` 를 읽지 않았고,
//!   `src/serializer/hwpx/section.rs::render_equation` 이 `lineMode` 를 방출하지 않았다.
//!
//! 그 결과 수식의 "차지하는 범위"(HWP5 표 105 attribute bit0 = HWPX `lineMode`)가
//! HWP5→HWP5 / HWP5→HWPX / HWPX→HWP5 / HWPX→HWPX 네 경로 전부에서 CHAR(0) 로 초기화됐다.
//!
//! 본 파일은 한컴이 만든 실제 저장본(`samples/수식-문자처럼취급-아님.hwp`)을 입력으로
//! 삼아 (1) LINE 설정이 왕복에서 살아남는지, (2) 원본 CHAR 문서가 그대로 CHAR 로
//! 남는지(정답지 무변경)를 동시에 단언한다.

use rhwp::model::control::{Control, Equation, EQUATION_LINE_MODE_BIT};
use rhwp::model::document::Document;
use std::fs;
use std::path::Path;

const HANCOM_EQUATION_HWP: &str = "samples/수식-문자처럼취급-아님.hwp";

fn load_hwp(rel: &str) -> Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::parse_document(&bytes).unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e))
}

fn first_equation(doc: &Document) -> &Equation {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| match c {
            Control::Equation(eq) => Some(eq.as_ref()),
            _ => None,
        })
        .expect("문서에 수식 컨트롤이 있어야 한다")
}

fn set_first_equation_line_mode(doc: &mut Document) {
    let mut done = false;
    for section in &mut doc.sections {
        // 원본 구역 스트림이 남아 있으면 직렬화기가 원본 바이트를 그대로 되돌려주므로
        // IR 편집이 저장에 반영되지 않는다. 실제 편집 API(set_equation_properties_native 등)
        // 와 동일하게 패스스루를 무효화한다.
        section.raw_stream = None;
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                if let Control::Equation(eq) = ctrl {
                    eq.attr |= EQUATION_LINE_MODE_BIT;
                    done = true;
                }
            }
        }
    }
    assert!(done, "수식 컨트롤을 찾지 못했다");
}

/// HWP5 → HWP5: 수식 범위 LINE(attribute bit0)이 저장·재적재 후에도 남아야 한다.
#[test]
fn issue_2727_hwp5_line_mode_survives_roundtrip() {
    let mut doc = load_hwp(HANCOM_EQUATION_HWP);
    set_first_equation_line_mode(&mut doc);

    let bytes = rhwp::serialize_document(&doc).expect("HWP5 직렬화");
    let re_doc = rhwp::parse_document(&bytes).expect("HWP5 재파싱");

    let eq = first_equation(&re_doc);
    assert_eq!(
        eq.attr & EQUATION_LINE_MODE_BIT,
        EQUATION_LINE_MODE_BIT,
        "EQEDIT attribute bit0(수식 범위 LINE)이 HWP5 왕복에서 보존돼야 한다. attr=0x{:08X}",
        eq.attr
    );
}

/// HWP5 → HWPX → IR: `lineMode="LINE"` 로 방출되고 다시 bit0 으로 읽혀야 한다.
#[test]
fn issue_2727_hwpx_line_mode_survives_roundtrip() {
    let mut doc = load_hwp(HANCOM_EQUATION_HWP);
    set_first_equation_line_mode(&mut doc);

    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("HWPX 직렬화");
    let re_doc = rhwp::parser::hwpx::parse_hwpx(&bytes).expect("HWPX 재파싱");

    let eq = first_equation(&re_doc);
    assert_eq!(
        eq.attr & EQUATION_LINE_MODE_BIT,
        EQUATION_LINE_MODE_BIT,
        "HWPX lineMode=\"LINE\" 이 왕복에서 보존돼야 한다. attr=0x{:08X}",
        eq.attr
    );
}

/// 한컴 원본(수식 범위 CHAR)은 정정 후에도 CHAR 그대로여야 한다 — 말뭉치 무변경 보장.
#[test]
fn issue_2727_hancom_char_equation_stays_char() {
    let doc = load_hwp(HANCOM_EQUATION_HWP);
    assert_eq!(
        first_equation(&doc).attr,
        0,
        "한컴 원본 EQEDIT attribute 는 0(CHAR)이어야 한다"
    );

    let hwp_bytes = rhwp::serialize_document(&doc).expect("HWP5 직렬화");
    let hwp_doc = rhwp::parse_document(&hwp_bytes).expect("HWP5 재파싱");
    assert_eq!(
        first_equation(&hwp_doc).attr,
        0,
        "CHAR 수식은 HWP5 왕복 후에도 attribute 0 이어야 한다"
    );

    let hwpx_bytes = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("HWPX 직렬화");
    let hwpx_doc = rhwp::parser::hwpx::parse_hwpx(&hwpx_bytes).expect("HWPX 재파싱");
    assert_eq!(
        first_equation(&hwpx_doc).attr,
        0,
        "CHAR 수식은 HWPX 왕복 후에도 attribute 0 이어야 한다"
    );
}

/// 한컴 원본 HWPX(`lineMode="CHAR"`)를 그대로 읽으면 bit0 이 서지 않아야 한다.
#[test]
fn issue_2727_hancom_hwpx_char_parses_as_zero_bit() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/수식-문자처럼취급-아님.hwpx");
    let bytes = fs::read(&path).expect("한컴 원본 HWPX 읽기");
    let doc = rhwp::parser::hwpx::parse_hwpx(&bytes).expect("HWPX 파싱");
    assert_eq!(
        first_equation(&doc).attr & EQUATION_LINE_MODE_BIT,
        0,
        "lineMode=\"CHAR\" 는 bit0 clear 여야 한다"
    );
}
