//! [Issue #4680] HWP3 제어 문자를 글리프로 눌러 써서 저장본이 그 문자를 잃는다.
//!
//! IR 규약은 제어 문자를 **가시 유니코드 등가물 + `control_mask` 비트**로 표현하고,
//! 직렬화기가 그 짝을 보고 HWP5 제어 코드를 되살리는 것이다. HWP3 파서는 그 비트를
//! 세우지 않고 글리프로 눌러 썼다 — 고정폭 빈칸(코드 31)은 일반 공백으로, 하이픈
//! (코드 24)은 `'-'`(U+002D)로.
//!
//! # 기대값의 출처
//!
//! `samples/` 에는 같은 문서의 HWP3 판과 **한/글이 저장한 HWPX 판**이 짝으로 있다
//! (`HWP3-password-123456.hwp` ↔ `HWP5-nopassword-123456.hwpx`). 두 파일은 포맷만
//! 다른 같은 문서이므로 본문 문자도 같아야 한다. 수정 전에는 HWP3 쪽만 그 자리가
//! `U+0020` 이었다(264쪽 문서 실측으로도 한/글은 `control_mask` 비트 31 을 174문단,
//! 비트 24 를 96문단에 세우는데 우리는 하나도 세우지 않았다).
//!
//! 묶음 빈칸(코드 30)은 어느 쪽으로도 증거가 없어 이 수정의 범위 밖이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::{parse_document, parse_document_with_password};
use std::path::Path;

const HWP3: &str = "samples/HWP3-password-123456.hwp";
const HWPX: &str = "samples/HWP5-nopassword-123456.hwpx";
const PASSWORD: &[u8] = &[49, 50, 51, 52, 53, 54];

fn read(rel: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).expect("표본 읽기")
}

fn body(doc: &rhwp::model::document::Document) -> String {
    doc.sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect()
}

fn hwp3_body() -> String {
    body(&parse_document_with_password(&read(HWP3), PASSWORD).expect("HWP3 파싱"))
}

#[test]
fn hwp3_fixed_width_space_matches_the_hangul_conversion() {
    let a = hwp3_body();
    let b = body(&parse_document(&read(HWPX)).expect("HWPX 파싱"));
    let needle = "글\u{2007}97";
    assert!(
        b.contains(needle),
        "정답지(한/글 HWPX 판)에 고정폭 빈칸이 없다 — 표본이 바뀌었는지 확인하라"
    );
    assert!(
        a.contains(needle),
        "HWP3 판이 고정폭 빈칸을 일반 공백으로 눌러 썼다. 같은 문서의 한/글 판은 \
         U+2007 을 쓴다"
    );
}

#[test]
fn hwp3_space_forms_match_the_hangul_conversion() {
    // 반례 — 보존이 과하면 안 된다. 이 문서에는 **진짜 일반 공백**을 쓴 자리도
    // 따로 있으므로 "일반 공백이 없어야 한다" 는 과한 주장이다. 두 형태의
    // 출현 수가 한/글 판과 같아야 보존이 정확한 것이다.
    let a = hwp3_body();
    let b = body(&parse_document(&read(HWPX)).expect("HWPX 파싱"));
    for (label, ch) in [
        ("고정폭 빈칸 U+2007", '\u{2007}'),
        ("일반 공백 U+0020", ' '),
    ] {
        let na = a.matches(ch).count();
        let nb = b.matches(ch).count();
        assert_eq!(
            na, nb,
            "{label} 개수가 한/글 판과 다르다 (HWP3 {na} · HWPX {nb})"
        );
    }
}
