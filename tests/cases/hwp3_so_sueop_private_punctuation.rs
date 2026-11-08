//! `samples/SO-SUEOP.hwp` 의 HWP3 사적 문장부호가 `'?'` 로 버려지던 문제.
//!
//! `decode_johab` 이 매핑을 못 찾아 `'?'` 를 돌려주면 HWP3 파서가 그 문자를 건너뛴다.
//! 각 값은 같은 문서의 한컴 변환본 `samples/SO-SUEOP.hwpx` 와 문자 멀티셋·문맥으로 대조했고,
//! 넣은 뒤 두 문서의 해당 문자 개수가 같다(“ 31·” 32·‧ 4·— 1·∣ 6·↳ 18·책괄호 3·3).

use rhwp::parser::hwp3::johab::decode_johab;

#[test]
fn so_sueop_private_punctuation_decodes() {
    let measured: &[(u16, char)] = &[
        (0x3062, '“'),
        (0x3063, '”'),
        (0x3066, '‧'),
        (0x2014, '—'),
        (0x2223, '∣'),
        (0x21B3, '↳'),
    ];
    for &(code, expected) in measured {
        assert_eq!(
            decode_johab(code),
            expected,
            "0x{code:04X} 가 다시 버려진다"
        );
    }
}

/// 책괄호 두 코드는 변환본이 한컴 PUA(F0854·F0855, 흰 겹낫표 반각)로 저장한다 —
/// 렌더러의 공통 한컴 PUA 표가 표시값을 맡는 계약(0x3366·0x37C0~)과 같다.
#[test]
fn so_sueop_book_brackets_keep_the_hancom_pua() {
    assert_eq!(decode_johab(0x30BB), '\u{F0854}');
    assert_eq!(decode_johab(0x30BD), '\u{F0855}');
}
