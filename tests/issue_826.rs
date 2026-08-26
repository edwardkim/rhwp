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
    // [#5793] 시각 판정 완료 — 한글 2022 는 U+F0827 을 반각 이중 가로선으로 그린다
    // (1776332 제목 밑 이중 밑줄, 280.0→506.5px 실측). 종전 잠정값 ■(전각)은 띠를
    // 2배로 늘여 제목을 겹쳤다. 이웃 0xF0832 와 같은 이중선 계열.
    let input = '\u{F0827}';
    let expected = '\u{2550}';
    let result = map_pua_bullet_char(input);
    assert_eq!(
        result, expected,
        "U+F0827 는 U+2550 (═) 로 substitution 되어야 함 (#5793 시각 판정). got: U+{:04X}",
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
    // [Task #727 / PR #1020] U+F02B1~F02C4 (사각 안 1~9 등) 매핑 entry 제거
    // → raw passthrough 로 변경. fallback chain 의 함초롬바탕 family 가
    // PUA 글리프 매칭 (사각 안 ①, 한컴 권위 정합). 단언 기대값을 raw
    // passthrough 로 정정.
    assert_eq!(
        map_pua_bullet_char('\u{F02B1}'),
        '\u{F02B1}',
        "사각 안 ① (raw passthrough)"
    );
    assert_eq!(map_pua_bullet_char('\u{F0854}'), '\u{300A}', "《");
}
