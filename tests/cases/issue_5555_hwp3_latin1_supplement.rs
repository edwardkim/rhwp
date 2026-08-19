//! [Issue #5555] HWP3 라틴 확장(Latin-1 Supplement)은 유니코드 값 그대로 디코딩된다.
//!
//! HWP3 은 0x00A0..=0x00FF 구간 문자를 hchar 에 유니코드 값 그대로 담는다.
//! 매핑이 없으면 '?' 가 되어 파서가 조용히 버리고 "für"→"fr" 처럼 글자가
//! 삭제된다. 07615 원시 실측 8문자(ü·ä·ö·ß·Ö·Ü·Ä·é)는 한글 SaveAs 정답지의
//! 분포와 정합한다. 사적 따옴표(0x0081/0x0082)는 구간 밖이라 종전 하드코딩이
//! 그대로 담당한다.

use rhwp::parser::hwp3::johab::decode_johab;

/// 계약 1 — 실측 8문자가 유니코드 항등으로 통과한다.
#[test]
fn latin1_supplement_passes_through() {
    assert_eq!(decode_johab(0x00FC), 'ü');
    assert_eq!(decode_johab(0x00E4), 'ä');
    assert_eq!(decode_johab(0x00F6), 'ö');
    assert_eq!(decode_johab(0x00DF), 'ß');
    assert_eq!(decode_johab(0x00D6), 'Ö');
    assert_eq!(decode_johab(0x00DC), 'Ü');
    assert_eq!(decode_johab(0x00C4), 'Ä');
    assert_eq!(decode_johab(0x00E9), 'é');
}

/// 계약 2 — 구간 밖 사적 따옴표 매핑은 종전대로 유지된다.
#[test]
fn private_quote_mappings_are_unchanged() {
    assert_eq!(decode_johab(0x0081), '\u{201C}');
    assert_eq!(decode_johab(0x0082), '\u{201D}');
}
