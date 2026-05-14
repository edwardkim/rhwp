//! Issue #826: HWP3 PUA U+F080F / U+F0827 글리프 부재 — 머리말/꼬리말 패턴 시각 차이.
//!
//! 본질: HWP3 char `0x301C` / `0x303D` 가 한컴 PUA `U+F080F` / `U+F0827` 로 매핑됨.
//! 한컴 함초롬 폰트는 해당 PUA glyph 보유, rhwp-studio 번들 폰트 (오픈 라이선스) 부재.
//!
//! 정정: render-time PUA substitution — `map_pua_bullet_char` (paragraph_layout.rs)
//! 의 0xF00D0~0xF09FF 범위 match 에 두 케이스 추가. IR 보존 (parser/HWP5 cross-ref
//! 정합 유지), 측정/렌더링 양쪽 자동 적용.
//!
//! PR #753 (Task #741) 본문 후속 항목 직접 수행 — "PUA char (U+F080F, U+F0827)
//! 폰트 fallback (별도 task)".

use rhwp::renderer::layout::map_pua_bullet_char;

#[test]
fn issue_826_pua_f080f_substitution() {
    // U+F080F (한컴 PUA, 굵은 가로선) → U+2501 ━ BOX DRAWINGS HEAVY HORIZONTAL.
    let input = '\u{F080F}';
    let expected = '\u{2501}';
    let result = map_pua_bullet_char(input);
    assert_eq!(
        result, expected,
        "U+F080F 는 U+2501 (━) 로 substitution 되어야 함. got: U+{:04X}",
        result as u32,
    );
}

#[test]
fn issue_826_pua_f0827_substitution() {
    // U+F0827 (한컴 PUA, 미상 — johab.rs:67) → U+25A0 ■ BLACK SQUARE (잠정).
    let input = '\u{F0827}';
    let expected = '\u{25A0}';
    let result = map_pua_bullet_char(input);
    assert_eq!(
        result, expected,
        "U+F0827 는 U+25A0 (■) 로 substitution 되어야 함 (잠정). got: U+{:04X}",
        result as u32,
    );
}

#[test]
fn issue_826_non_pua_passthrough() {
    // 회귀 가드: 일반 문자는 변환 없음.
    assert_eq!(map_pua_bullet_char('A'), 'A');
    assert_eq!(map_pua_bullet_char('가'), '가');
    assert_eq!(map_pua_bullet_char('━'), '━');
}

#[test]
fn issue_826_other_pua_existing_unchanged() {
    // 회귀 가드: 기존 매핑된 PUA 는 정합 유지.
    assert_eq!(map_pua_bullet_char('\u{F02B1}'), '\u{2460}', "①");
    assert_eq!(map_pua_bullet_char('\u{F0854}'), '\u{300A}', "《");
}
