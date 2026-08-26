//! Issue #5860: HWP3 사적 문자 매핑 누락으로 본문 글자가 조용히 사라진다.
//!
//! `decode_johab` 이 매핑을 못 찾아 `'?'` 를 돌려주면 HWP3 파서가 그 문자를 그냥
//! 건너뛴다(`src/parser/hwp3/mod.rs` 의 `s == '?' && ch >= 0x0080` 분기). 한글은 같은
//! 자리에서 글자를 정상으로 내므로, 매핑이 없는 코드는 곧 **읽기 단계의 본문 소실**이다.
//!
//! s36 load/save 전수(한글 2022 오라클, 10k 코퍼스)에서 이 경로로 사라지던 코드를
//! 계측으로 모으고 오라클 추출 텍스트와 위치 정렬해 짝지었다. HWP3 38문서 중 15문서에서
//! 479자가 사라지고 있었고(07270 은 40쪽→52쪽, 04442 는 1쪽 서식이 2쪽), 아래 표가 그 전량이다.
//!
//! **범위가 아니라 실측 집합이다.** 같은 `0x20xx` 대에 항등(`0x2013`·`0x2103`·`0x2113`·
//! `0x2219`·`0x22C5`·`0x2010`)과 비항등(`0x2024`→`・`, `0x2058`→`△`, `0x205A`→`○`)이
//! 섞여 있어 구간을 통째로 통과시키면 다른 글자가 된다.

use rhwp::parser::hwp3::johab::decode_johab;

/// 소실되던 코드 전량 — 값마다 근거 문서(횟수)는 `src/parser/hwp3/johab.rs` 주석에 있다.
#[test]
fn measured_hwp3_private_codes_decode_to_the_hangul_glyph() {
    let measured: &[(u16, char)] = &[
        (0x0083, '\u{2018}'),
        (0x0084, '\u{2019}'),
        (0x0480, 'ː'),
        (0x1F2E, 'の'),
        (0x2010, '‐'),
        (0x2013, '–'),
        (0x2024, '・'),
        (0x2058, '△'),
        (0x205A, '○'),
        (0x2103, '℃'),
        (0x2113, 'ℓ'),
        (0x2190, '←'),
        (0x2192, '→'),
        (0x2193, '↓'),
        (0x2219, '∙'),
        (0x22C5, '⋅'),
        (0x25F5, '\u{F0099}'),
        (0x2BCE, '\u{F012B}'),
        (0x2F08, '▪'),
        (0x2F11, '◦'),
        (0x2F14, '◦'),
        (0x3048, '\u{F0832}'),
        (0x3067, '\u{2018}'),
        (0x3068, '\u{2019}'),
        (0x309B, '｢'),
        (0x309D, '｣'),
        (0x3157, '‧'),
        (0x32B0, '\u{FF70}'),
    ];
    for &(code, expected) in measured {
        assert_eq!(
            decode_johab(code),
            expected,
            "0x{code:04X} 가 다시 버려진다"
        );
    }
}

/// 관인·서명란 도장 기호(`0x2BCE`)는 한컴 사설 영역 코드포인트로 보존한다 —
/// 렌더러의 공통 한컴 PUA 표가 표시 문자열(`(인)`)을 담당하는 기존 계약을 그대로 탄다.
/// 7문서(00460·00465·04442·04640·04845·05428·05755)에서 8회 사라지던 값이다.
#[test]
fn seal_symbol_keeps_the_hancom_pua_codepoint() {
    assert_eq!(decode_johab(0x2BCE), '\u{F012B}');
    assert_eq!(decode_johab(0x25F5), '\u{F0099}');
    assert_eq!(decode_johab(0x3048), '\u{F0832}');
}

/// 기존 계약이 깨지지 않았는지 — 하드코딩이 완성형 좌표 규칙보다 먼저다.
/// 새 값은 전부 `0x3401`(기호 규칙 BASE) 미만이라 규칙의 정의역과 겹치지 않는다.
#[test]
fn existing_rule_derived_mappings_are_unchanged() {
    assert_eq!(decode_johab(0x3446), '→');
    assert_eq!(decode_johab(0x3441), '■');
    assert_eq!(decode_johab(0x343B), '○');
    assert_eq!(decode_johab(0x3438), '※');
    assert_eq!(decode_johab(0x4F5D), '債');
    assert_eq!(decode_johab(0x37C1), '\u{F03F0}');
    assert_eq!(decode_johab(0x3366), '\u{F03C5}');
    assert_eq!(decode_johab(0x203B), '※');
}

/// 실측 근거가 없는 값은 여전히 매핑하지 않는다 — 근거 없는 구간 통과 금지 계약.
#[test]
fn unmeasured_private_codes_stay_unmapped() {
    for ch in [0x0085_u16, 0x2059, 0x2F09, 0x2BCD, 0x3049] {
        assert_eq!(
            decode_johab(ch),
            '?',
            "0x{ch:04X} 는 근거 없이 매핑하면 안 된다"
        );
    }
}
