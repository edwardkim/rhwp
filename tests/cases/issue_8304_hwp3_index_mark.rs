//! [Issue #8304] HWP3 찾아보기 표시는 공통 IR 의 찾아보기 표시로 읽고, HWP5·HWPX 저장이
//! 함께 보존한다.
//!
//! HWP3 특수 문자 26(스펙 §10.20 표 61)의 키워드는 `hchar array[60]` 두 개다. rhwp 는 이를
//! 바이트 단위 조합형으로 읽어 키워드가 깨졌고(`씽b로?`), `Unknown` 필드로 만들어 HWP5
//! 저장에서 사라졌다(`convert --verify` exit 3). 기대 키워드는 같은 문서를 한컴 오피스 2024
//! (13.0.0.3901)가 HWP5 로 저장한 `samples/HWP5-nopassword-123456.hwp` 의 찾아보기 표시다.
//!
//! 이 문서는 화면 배치가 아니라 IR·저장 계약만 검사한다 — 찾아보기 표시는 그려지지 않는
//! 컨트롤이며, 원문의 쪽 배치는 #7445 의 피델리티 범위다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parser::hwp3::encoding::{decode_hchar_array, decode_hwp3_string};

const HWP3: &str = "samples/HWP3-password-123456.hwp";
const HANCOM_HWP5: &str = "samples/HWP5-nopassword-123456.hwp";

fn open(path: &str, password: Option<&str>) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    match password {
        Some(p) => DocumentCore::from_bytes_with_password(&bytes, p.as_bytes()),
        None => DocumentCore::from_bytes(&bytes),
    }
    .unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

/// 본문 문단마다 (구역, 문단, 첫째 키, 둘째 키).
fn index_marks(core: &DocumentCore) -> Vec<(usize, usize, String, String)> {
    let mut out = Vec::new();
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for control in &para.controls {
                if let Control::IndexMark(im) = control {
                    out.push((si, pi, im.first_key.clone(), im.second_key.clone()));
                }
            }
        }
    }
    out
}

/// HWP3 원문 본문에는 컨트롤 자리 표지(U+FFFC)가 있고 HWP5 에는 없다 — 표지를 빼고 본다.
fn paragraph_text(core: &DocumentCore, si: usize, pi: usize) -> String {
    core.document().sections[si].paragraphs[pi]
        .text
        .chars()
        .filter(|&c| c != '\u{FFFC}')
        .collect()
}

#[test]
fn hwp3_index_marks_match_hancom_hwp5_keywords() {
    let hwp3 = open(HWP3, Some("123456"));
    let hancom = open(HANCOM_HWP5, None);
    let expected = index_marks(&hancom);
    assert_eq!(
        expected
            .iter()
            .map(|(_, _, k, _)| k.as_str())
            .collect::<Vec<_>>(),
        ["입력기", "바탕글", "팩스 머지"],
        "한컴 HWP5 의 찾아보기 키워드"
    );
    assert_eq!(index_marks(&hwp3), expected, "HWP3 에서 읽은 찾아보기 표시");
    for (si, pi, _, _) in &expected {
        assert_eq!(
            paragraph_text(&hwp3, *si, *pi),
            paragraph_text(&hancom, *si, *pi),
            "구역 {si} 문단 {pi}: 표시가 놓인 문단"
        );
    }
}

#[test]
fn hwp3_index_marks_survive_hwp5_and_hwpx_save() {
    let hwp3 = open(HWP3, Some("123456"));
    let marks = index_marks(&hwp3);
    assert_eq!(marks.len(), 3);

    let hwp5 = DocumentCore::from_bytes(&hwp3.export_hwp_with_adapter().expect("HWP5 저장"))
        .expect("HWP5 다시 열기");
    assert_eq!(index_marks(&hwp5), marks, "HWP5 저장 뒤");
    assert_eq!(hwp5.page_count(), hwp3.page_count(), "HWP5 저장 뒤 쪽수");

    let hwpx = DocumentCore::from_bytes(&hwp3.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기");
    assert_eq!(index_marks(&hwpx), marks, "HWPX 저장 뒤");
    assert_eq!(hwpx.page_count(), hwp3.page_count(), "HWPX 저장 뒤 쪽수");
}

/// `hchar array` 는 글자마다 리틀 엔디언 2바이트다 — ASCII 도 2바이트를 차지하고 0 에서
/// 끝난다. 바이트 단위 조합형으로 읽으면 다른 글자가 된다.
#[test]
fn hchar_array_reads_little_endian_internal_codes() {
    // 'A'(0x0041), '가'(조합형 0x8861), 끝(0), 'B'
    let bytes = [0x41, 0x00, 0x61, 0x88, 0x00, 0x00, 0x42, 0x00];
    assert_eq!(decode_hchar_array(&bytes), "A가");
    assert_ne!(decode_hwp3_string(&bytes), "A가");
}
