//! Issue #5140: 한컴 사용자 정의 기호(`0xA000 | X`) ↔ 평면 15 보충 PUA 사상 계약.
//!
//! HWP5 는 사용자 정의 기호를 BMP `0xA000 | X` 단일 유닛으로 담고, 한글이 HWPX 로 저장할
//! 때는 같은 글자를 평면 15 보충 PUA(`U+F0000 | X`)로 올린다. 이 사상을 하지 않으면
//! h2x 저장에서 그 글자가 Yi 음절 등으로 깨진다.
//!
//! **범위가 아니라 실측 집합이다.** `0xA000..=0xABFF` 전체를 밀면 한글이 실제로는 BMP 로
//! 두는 값(`0xA813`)까지 옮겨 글자를 망친다. 표는 10k 코퍼스와 한글 SaveAs 대조에서
//! 확인된 값만 담는다 — 새 값은 한글 실측으로만 추가한다.
//!
//! 이 계약은 원래 `src/parser/tags.rs` 의 `#[cfg(test)]` 안에 있었으나, 저장소 규약이
//! 제품 소스의 단위시험 증가를 막으므로 통합 테스트로 옮겼다. 검사 대상은 모두 공개
//! 항목이라 내용은 그대로다.

use rhwp::parser::tags::{
    hancom_symbol_to_plane15, plane15_to_hancom_symbol, HANCOM_SYMBOL_BMP_TO_PLANE15,
};

/// 실측한 네 쌍 — 사상은 하위 12비트를 평면 15 로 올리는 것이다.
#[test]
fn hancom_symbol_maps_low_12_bits_into_plane15() {
    for (bmp, cp) in [
        (0xA80Fu16, 0x0F_080Fu32),
        (0xA12B, 0x0F_012B),
        (0xA832, 0x0F_0832),
        (0xA2B1, 0x0F_02B1),
    ] {
        assert_eq!(hancom_symbol_to_plane15(bmp), Some(cp));
        assert_eq!(plane15_to_hancom_symbol(cp), Some(bmp));
        assert!(
            char::from_u32(cp).is_some(),
            "{cp:#x} 는 유효한 스칼라여야 한다"
        );
    }
}

/// 범위가 아니라 실측 집합이라는 계약. `0xA813` 은 한글이 평면 15 로 올리지 않는
/// 실측 반례(08103, 영어 단어 중간·맑은 고딕)이므로 표에 들어가면 안 된다.
#[test]
fn hancom_symbol_table_excludes_the_measured_counter_example() {
    assert_eq!(hancom_symbol_to_plane15(0xA813), None);
    assert_eq!(plane15_to_hancom_symbol(0x0F_0813), None);
}

/// 사상 대상이 아닌 글자는 통과시킨다 — 특히 `0xAC00` 이상은 실제 한글 음절이다.
#[test]
fn hancom_symbol_leaves_non_symbols_alone() {
    for u in [0x0041u16, 0xAC00, 0xD55C, 0xFFFC, 0x9FFF] {
        assert_eq!(hancom_symbol_to_plane15(u), None, "{u:#x}");
    }
    // 평면 15 밖 보충 PUA(평면 16)와 BMP 사설영역은 되돌리지 않는다.
    for cp in [0x10_0000u32, 0x00_E000, 0x01_F600] {
        assert_eq!(plane15_to_hancom_symbol(cp), None, "{cp:#x}");
    }
}

/// 글자겹침(`composeText`) 실측으로 넓힌 값들 — 한글 SaveAs 에서 107/107 전량
/// 평면 15 로 갔다(06190 · 06638 · 08403 · 08396).
#[test]
fn hancom_symbol_table_covers_the_compose_text_measurements() {
    for u in [
        0xA289u16, 0xA28A, 0xA292, 0xA29B, 0xA2BA, 0xA2C0, 0xA2C3, 0xA2CC,
    ] {
        assert_eq!(
            hancom_symbol_to_plane15(u),
            Some(0x0F_0000 | u32::from(u & 0x0FFF)),
            "{u:#x}"
        );
    }
}

/// 표는 정렬·중복 없음이어야 한다 — 값을 손으로 추가하다 흐트러지는 것을 막는다.
#[test]
fn hancom_symbol_table_is_sorted_and_unique() {
    assert!(
        HANCOM_SYMBOL_BMP_TO_PLANE15.windows(2).all(|w| w[0] < w[1]),
        "표가 정렬되지 않았거나 중복이 있다"
    );
    assert!(
        HANCOM_SYMBOL_BMP_TO_PLANE15
            .iter()
            .all(|&u| (0xA000..0xAC00).contains(&u)),
        "실측 값은 0xA000..0xABFF 안에 있어야 한다(0xAC00 부터는 한글 음절)"
    );
}
