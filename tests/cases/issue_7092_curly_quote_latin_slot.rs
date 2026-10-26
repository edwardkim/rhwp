//! [#7092] 굽은 따옴표 `‘ ’ “ ”` 는 글자모양의 **영문(1) 슬롯** 글꼴로 그린다.
//!
//! ## 무엇이 문제였나
//!
//! `style_resolver::detect_lang_category` 는 U+2000 블록을 기본값 0(한글)으로 떨어뜨려
//! 굽은 따옴표를 한글 슬롯 글꼴로 그렸다. 한컴 정본은 영문 슬롯 글꼴로 그린다.
//!
//! ## 정본 근거 — 슬롯 face 가 서로 다른 출현만 대조
//!
//! 저장소 `samples/`↔`pdf/` 정본 233문서에서 한글·영문·기호 슬롯 face 가 서로 다른 따옴표
//! 출현만 골라, 정본 PDF 의 그 글자 글꼴 이름과 각 슬롯 face 를 문서별 이름 대응으로 맞댔다.
//!
//! ```text
//!   영문 슬롯만 일치하는 문서   ‘ 46 · ’ 39 · “ 29 · ” 27
//!   한글 슬롯만 일치하는 문서   0
//!   기호 슬롯만 일치하는 문서   1
//! ```
//!
//! 이 시험의 앵커 `samples/basic/treatise sample.hwp` 3쪽은 한글=한컴바탕 · 영문=Times New
//! Roman · 기호=한컴바탕 이고, 정본 `pdf/basic/treatise sample-2022.pdf` 는 `‘닭이 …’` 의
//! 두 따옴표를 `TimesNewRomanPSMT` 로 그린다(앞뒤 한글은 `Haansoft Batang`).
//!
//! ## 반례
//!
//! - 같은 블록의 `․`(U+2024)는 정본이 영문 슬롯이 아니다 — 같은 문서 4쪽에서
//!   `Haansoft Batang` 이다(한글·기호 슬롯이 같은 문서). 슬롯이 다른 sample16 2022의
//!   독립 Print로 U+2024의 기호(5) 소속을 별도로 확인했다. 영문으로 옮기지 않는다.
//! - 곧은 따옴표(ASCII `'` `"`)는 언어 중립 문자로 남는다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::style_resolver::detect_lang_category;

/// 정본 `pdf/basic/treatise sample-2022.pdf`.
const SAMPLE_TREATISE: &str = "samples/basic/treatise sample.hwp";

/// 한 쪽에서 `targets` 글자를 담은 run 의 (글자, 첫 글꼴 이름) 을 모은다.
fn faces_on_page(sample: &str, page: u32, targets: &[char]) -> Vec<(char, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let raw = core
        .get_page_text_layout_native(page)
        .expect("text-layout JSON");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON 해석");
    let mut out = Vec::new();
    for run in layout["runs"].as_array().into_iter().flatten() {
        let family = run["fontFamily"].as_str().unwrap_or_default();
        let primary = family
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_matches(['\'', '"'])
            .to_string();
        for c in run["text"].as_str().unwrap_or_default().chars() {
            if targets.contains(&c) {
                out.push((c, primary.clone()));
            }
        }
    }
    out
}

#[test]
fn curly_quotes_are_classified_into_the_latin_slot() {
    for quote in ['\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}'] {
        assert_eq!(detect_lang_category(quote), 1, "{quote}");
    }
    // 한글/기호 face가 같은 treatise만으로 슬롯 0과 5를 구별할 수 없다.
    // 두 face가 다른 월간 수출입 원본의 한컴 Print와 원본 글자모양이 기호 슬롯을 확인한다.
    assert_eq!(detect_lang_category('\u{2024}'), 5, "점 리더는 기호 슬롯");
    // [#8136] 슬롯마다 다른 설치 글꼴을 준 합성 입력(한/글 2020·2024 Print)에서 … – — ※ • 는
    // 앞뒤가 한글이어도 기호 슬롯 글꼴로 그려진다. 종전의 「근거 없음」을 그 측정으로 대신한다.
    for symbol in ['\u{2026}', '\u{2013}', '\u{2014}', '\u{203B}', '\u{2022}'] {
        assert_eq!(detect_lang_category(symbol), 5, "{symbol} 는 기호 슬롯");
    }
    for other in ['\'', '"'] {
        assert_eq!(
            detect_lang_category(other),
            0,
            "{other} 는 근거 없이 옮기지 않는다"
        );
    }
}

/// 정본은 `‘닭이 … 먼저냐’` 의 두 따옴표를 영문 슬롯 face(Times New Roman)로 그린다.
/// 수정 전 rhwp 는 한글 슬롯 face(한컴바탕)였다.
#[test]
fn treatise_quotes_use_the_latin_slot_face() {
    let faces = faces_on_page(SAMPLE_TREATISE, 2, &['\u{2018}', '\u{2019}']);
    assert_eq!(
        faces.len(),
        2,
        "3쪽의 ‘ ’ 두 글자가 있어야 한다 — 검사 대상이 0건이면 통과 증거가 아니다: {faces:?}"
    );
    for (c, face) in &faces {
        assert_eq!(
            face, "Times New Roman",
            "{c} 는 정본(TimesNewRomanPSMT)처럼 영문 슬롯 face 여야 한다"
        );
    }
}

/// 반례 — 같은 블록의 `․`(U+2024)는 정본이 `Haansoft Batang` 이라 영문 슬롯으로 옮기지 않는다.
#[test]
fn treatise_one_dot_leader_keeps_its_non_latin_face() {
    let faces = faces_on_page(SAMPLE_TREATISE, 3, &['\u{2024}']);
    assert!(
        !faces.is_empty(),
        "4쪽의 ․ 가 있어야 한다 — 반례 표본 전제가 깨졌다"
    );
    for (_, face) in &faces {
        assert_eq!(face, "한컴바탕", "․ 는 영문 슬롯(Times New Roman)이 아니다");
    }
}
