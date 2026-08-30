//! Issue #6380: samples 실측으로 남아 있던 HWP3 사적 문자 소실.
//!
//! `decode_johab` 이 매핑을 못 찾아 `'?'` 를 돌려주면 HWP3 파서가 그 문자를 건너뛴다
//! (`src/parser/hwp3/mod.rs` 의 `s == '?' && ch >= 0x0080` 분기). #5860 이 10k 코퍼스
//! 실측으로 표를 채웠지만, 저장소 `samples/` 의 HWP3 원본만으로도 아직 버려지는 코드가
//! 남아 있었다.
//!
//! 각 값은 `samples/hwp3-sampleNN.hwp` 를 같은 문서의 한컴 변환본(`-hwpx.hwpx`·
//! `-hwp5.hwpx`)과 문자 멀티셋·문맥으로 대조해 얻었고, **코드 출현 횟수가 변환본의 해당
//! 문자 개수와 정확히 맞는 것만** 실었다.

use rhwp::parser::hwp3::johab::{decode_johab, decode_johab_araea_jamo};

/// 텍스트 다이어그램 괘선 조각 — `0x301C`(→F080F) 와 같은 묶음이고 상수 오프셋으로
/// 이어진다. sample11 코드 개수 3·1·10·6·9·17 이 변환본 PUA 개수와 그대로 맞는다.
#[test]
fn diagram_rule_fragments_decode_to_the_hancom_pua() {
    let measured: &[(u16, char)] = &[
        (0x3013, '\u{F0806}'),
        (0x3014, '\u{F0807}'),
        (0x3015, '\u{F0808}'),
        (0x3019, '\u{F080C}'),
        (0x301B, '\u{F080E}'),
        (0x301D, '\u{F0810}'),
    ];
    for &(code, expected) in measured {
        assert_eq!(
            decode_johab(code),
            expected,
            "0x{code:04X} 가 다시 버려진다"
        );
    }
}

/// 겹줄 `0x37ED` — 기호 좌표 규칙으로는 가타카나 `ネ` 가 되는 사적 graphic 코드다.
/// sample10(42건)·sample11(12건) 의 변환본이 같은 자리에 리터럴 `═` 를 동수로 갖는다.
#[test]
fn double_rule_fragment_is_a_literal_not_a_pua() {
    assert_eq!(decode_johab(0x37ED), '═');
}

/// 원문자·괄호문자 계열 (sample11). 표준 문자로 저장된 계열과 별도 글리프(PUA) 계열이
/// 같은 문서에서 함께 쓰인다 — 후자는 `hancom_pua.rs` 가 근거로 적어 둔 p23 NVRAM 라벨 줄이다.
#[test]
fn circled_and_parenthesized_series_decode() {
    let measured: &[(u16, char)] = &[
        (0x2E01, '①'),
        (0x2E07, '⑦'),
        (0x2E00, '\u{F0288}'),
        (0x2E0A, '\u{F0289}'),
        (0x2E12, '\u{F0291}'),
        (0x2C21, 'ⓐ'),
        (0x2C26, 'ⓕ'),
        (0x2C40, '㉠'),
        (0x2C42, '㉢'),
    ];
    for &(code, expected) in measured {
        assert_eq!(
            decode_johab(code),
            expected,
            "0x{code:04X} 가 다시 버려진다"
        );
    }
    // 0x2E0C(→F028B=③)는 근거 문서가 그 자리에 리터럴 ③ 을 써서 관측되지 않았다.
    assert_eq!(decode_johab(0x2E0C), '?');
}

/// 글머리표·장식 (sample·sample10).
#[test]
fn bullet_and_ornament_codes_decode() {
    assert_eq!(decode_johab(0x2022), '•');
    assert_eq!(decode_johab(0x2F17), '•');
    assert_eq!(decode_johab(0x2F06), '■');
}

/// `0xA2C1` 의 표준 매핑은 U+2299(`⊙`)지만 한글은 U+25C9(`◉`)를 쓴다.
/// `∼`→`～` 와 같은 성격의 한컴 표기 차이다 (sample11 문맥 정렬 2건).
#[test]
fn hancom_circled_dot_variant_matches_hangul() {
    assert_eq!(decode_johab(0x3481), '◉');
}

/// 초성 '채움'(인덱스 1) + 아래아 음절. 종전에는 무효 초성으로 보고 통째로 버렸다 —
/// hwp3-sample16 의 `0x87C1` 이 그 경우로, 한컴 변환본은 `석ᆞ박사급` 처럼 U+119E
/// 한 글자로 보존한다.
#[test]
fn araea_with_filler_leading_survives() {
    assert_eq!(decode_johab_araea_jamo(0x87C1), Some((None, 'ᆞ', None)));
    // 채움이 아닌 무효 초성은 종전대로 None — 없는 음절을 지어내지 않는다.
    assert_eq!(decode_johab_araea_jamo(0x83C1), None);
}

/// 실측 근거가 없는 이웃 코드는 여전히 매핑하지 않는다.
#[test]
fn unmeasured_neighbours_stay_unmapped() {
    for code in [0x2C27_u16, 0x2C43, 0x3016] {
        assert_eq!(decode_johab(code), '?', "0x{code:04X} 는 실측 근거가 없다");
    }
}
