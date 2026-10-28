//! [#6171 ③] 한컴 legacy face 의 폴백 사슬이 `Malgun Gothic` 보다 **한컴 대체 face** 를
//! 먼저 잡아야 한다.
//!
//! `Malgun Gothic` 만 `『`(U+300E)·`』`(U+300F)를 **반각**으로 두고 잉크를 em 왼쪽 절반에
//! 그린다. 글꼴 파일 실측(Windows 기본 설치본, em 단위):
//!
//! ```text
//! face             advance   『 잉크
//! Malgun Gothic     0.517    0.152..0.463   ← 반각 + 왼쪽 절반
//! HY중고딕          1.000    0.625..0.938
//! HY견고딕          1.000    0.602..0.938
//! Batang / Dotum    1.000    0.63 ..0.94
//! ```
//!
//! 조판은 이미 **전각 advance** 로 다음 런의 x 를 고정해 두므로, 반각 face 가 잡히면
//! 잉크가 왼쪽으로 몰려 `『`→`별` 공백이 4.25px → 13.50px 로 벌어진다(3146683 1쪽).
//!
//! `svg.rs` 의 `font_local_aliases` 는 `@font-face` src 에 한컴 대체 face(`HCR Dotum`)를
//! 이미 먼저 두는데, `font-family` 사슬을 만드는 `installed_render_font_aliases` 에는
//! 그것이 없어 **두 경로가 서로 다른 답을 냈다.** 이 시험이 그 대칭을 고정한다.
//!
//! 원 face 가 설치된 호스트의 결과는 이 순서와 무관하다 — 앞 후보가 먼저 잡힌다.

use rhwp::renderer::{render_font_family_chain, render_font_family_chain_for_weight};

/// 사슬에서 `face` 가 `Malgun Gothic` 보다 앞에 있어야 한다.
///
/// `Malgun Gothic` 이 사슬에 아예 없으면(명조 계열의 generic 은 `Batang` 으로 시작한다)
/// 반각 face 가 잡힐 길이 없으므로 통과다 — 계약은 "반각 face 가 한컴 대체 face 보다
/// **앞서지 않는다**" 이다.
fn assert_precedes_malgun(chain: &str, face: &str) {
    let families: Vec<&str> = chain
        .split(',')
        .map(|entry| entry.trim().trim_matches('\''))
        .collect();
    let target = families
        .iter()
        .position(|entry| *entry == face)
        .unwrap_or_else(|| panic!("사슬에 `{face}` 가 없다: {chain}"));
    if let Some(malgun) = families.iter().position(|entry| *entry == "Malgun Gothic") {
        assert!(
            target < malgun,
            "`{face}` 가 `Malgun Gothic`(반각 『) 보다 뒤에 있다 — {chain}"
        );
    }
}

#[test]
fn hancom_legacy_gothic_prefers_hancom_substitute_over_malgun() {
    for family in ["한양중고딕", "HY중고딕", "한양견고딕"] {
        let chain = render_font_family_chain(family);
        assert_precedes_malgun(&chain, "HCR Dotum");
        assert_precedes_malgun(&chain, "함초롬돋움");
    }
}

#[test]
fn hancom_legacy_myeongjo_prefers_hancom_substitute_over_malgun() {
    let chain = render_font_family_chain("한양견명조");
    // 명조 계열의 generic 앞머리는 `Batang` 이지만 `Malgun Gothic` 도 사슬에 들어온다.
    assert_precedes_malgun(&chain, "HCR Batang");
    assert_precedes_malgun(&chain, "함초롬바탕");
}

#[test]
fn bold_chain_keeps_the_same_order() {
    // bold 경로는 ExtraLight 를 걸러내는 별도 함수라 삽입이 빠질 수 있다.
    let chain = render_font_family_chain_for_weight("한양중고딕", true);
    assert_precedes_malgun(&chain, "HCR Dotum");
}
