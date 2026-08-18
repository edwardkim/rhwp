//! HWPX → HWP5 저장에서 라틴 줄나눔(breakLatinWord)이 소실되던 문제(#5327).
//!
//! HWP5 는 breakLatinWord 를 para_shape attr1 bits5-6 에 저장한다(0=KEEP_WORD·
//! 1=HYPHENATION·2=BREAK_WORD). HWPX 파서는 이를 lexical 필드 break_latin_word 로만
//! 읽고 attr1 에는 싣지 않으므로(짝 breakNonLatinWord 는 attr1 bit7 에 인코딩하는 것과
//! 비대칭), HWP5 직렬화기가 lexical 값을 attr1 bits5-6 으로 재인코딩하지 않으면
//! HWPX→HWP5 저장에서 라틴 줄나눔 설정이 통째로 사라진다. #5298(HWP5→HWPX)의 거울.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::style::ParaShape;
use rhwp::serializer::doc_info::serialize_para_shape;

/// 직렬화된 para_shape 레코드의 attr1(선두 u32 LE) bits5-6 을 뽑는다.
fn attr1_latin_bits(ps: &ParaShape) -> u32 {
    let bytes = serialize_para_shape(ps);
    let attr1 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    (attr1 >> 5) & 0x03
}

fn ps_lexical(token: &str) -> ParaShape {
    ParaShape {
        break_latin_word: Some(token.to_string()),
        ..Default::default()
    }
}

#[test]
fn hwp5_break_latin_word_encoded_into_attr1_bits() {
    // HWPX 소스(lexical Some): attr1 bits5-6 으로 인코딩돼야 한다.
    assert_eq!(attr1_latin_bits(&ps_lexical("KEEP_WORD")), 0);
    assert_eq!(attr1_latin_bits(&ps_lexical("HYPHENATION")), 1);
    assert_eq!(attr1_latin_bits(&ps_lexical("BREAK_WORD")), 2);

    // HWP5 원본(lexical None): 원본 attr1 비트를 보존해야 한다(clobber 금지).
    let ps_none = ParaShape {
        attr1: 2 << 5, // BREAK_WORD 를 담은 원본 비트
        break_latin_word: None,
        ..Default::default()
    };
    assert_eq!(attr1_latin_bits(&ps_none), 2);
}
