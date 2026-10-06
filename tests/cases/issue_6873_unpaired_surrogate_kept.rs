//! [#6873] 짝을 잃은 UTF-16 서로게이트가 파서에서 **통째로 사라진다**.
//!
//! `PARA_TEXT` 는 UTF-16 코드 단위 배열이라 사용자 정의 기호를 **서로게이트 반쪽**으로
//! 싣는 문서가 있다. Rust `char` 도 XML 도 그 값을 담지 못한다 — 종전 파서는
//! `char::from_u32` 가 `None` 을 주는 자리를 그냥 건너뛰어, HWPX 저장본에서 그만큼 글자가
//! 없어졌다.
//!
//! ```text
//!   19211507   … 'C7A5'(장) '0020' 'DB80' '000D'      상위 반쪽 1개, 문단 끝
//!   18096141   'DFDA' '0020'×3 '신 청 인' …           하위 반쪽 4개, 항목 머리글
//! ```
//!
//! 정본은 **한글 자신의 h2x 산출**이다(engine 2020, `samples/issue6873/hwpx/`). 한글도
//! XML 에 반쪽을 담을 수 없어 그 자리를 `□`(U+25A1) 로 적는다.
//!
//! ```text
//!   19211507  <hp:t>충    주    시    장 </hp:t><hp:t>□</hp:t>
//!   18096141  <hp:t> □   신 청 인</hp:t>  (형제 항목 3개도 같은 모양)
//! ```
//!
//! ⚠ 인쇄와 저장의 답이 다르다 — 같은 engine 2020 이 **PDF 로는** 그 기호를 글리프로 찍고
//! (ToUnicode 가 U+FFFF 로 빠져 텍스트로는 못 꺼낸다) **HWPX 로는** `□` 로 적는다. 이
//! 이슈는 h2x 축이므로 HWPX 가 판정을 준다.
//!
//! 코퍼스 `.hwp` 6,582건 전수에서 서로게이트를 담은 문서 453건 중 **짝 없는 반쪽은 이 두
//! 문서(5글자)뿐**이다. 나머지 451건은 쌍이라 종전 경로가 이미 옳았다 — 음성 대조로
//! `samples/issue5793` 를 함께 잰다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;

/// 한글이 반쪽 서로게이트 자리에 적는 글자.
const SQUARE: char = '\u{25A1}';

const CHUNGJU: &str = "samples/issue6873/19211507-chungju-paid-restroom-certificate.hwp";
const GYEONGJU: &str = "samples/issue6873/18096141-gyeongju-gas-subsidy-plan.hwp";
/// 음성 대조 — 서로게이트 **쌍** 34개를 담은 기존 fixture.
const PAIRED: &str = "samples/issue5793/pua_f0827_double_rule.hwp";

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 표 셀·각주까지 내려가며 본문 문단 글자를 모은다.
fn collect(paragraphs: &[Paragraph], out: &mut Vec<String>) {
    for para in paragraphs {
        out.push(para.text.clone());
        for control in &para.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    collect(&cell.paragraphs, out);
                }
            }
        }
    }
}

fn paragraph_texts(rel: &str) -> Vec<String> {
    let bytes = std::fs::read(fixture(rel)).expect("정식 원본");
    let doc = HwpDocument::from_bytes(&bytes).expect("문서 로드");
    let mut out = Vec::new();
    for section in &doc.document().sections {
        collect(&section.paragraphs, &mut out);
    }
    out
}

fn square_count(texts: &[String]) -> usize {
    texts.iter().map(|t| t.matches(SQUARE).count()).sum()
}

#[test]
fn issue_6873_lone_high_surrogate_becomes_the_oracle_square() {
    let texts = paragraph_texts(CHUNGJU);

    // 정본 `<hp:t>충    주    시    장 </hp:t><hp:t>□</hp:t>` 와 같은 문단이다.
    let hit = texts
        .iter()
        .find(|t| t.starts_with('충') && t.contains('장'))
        .unwrap_or_else(|| panic!("`충…장` 문단이 없다: {texts:?}"));
    assert_eq!(
        hit.as_str(),
        "충    주    시    장 \u{25A1}",
        "짝 없는 상위 서로게이트가 정본과 같은 글자로 남아야 한다"
    );

    // 이 문서의 반쪽은 한 개다 — `□` 를 다른 자리에 만들어 내지 않는다.
    assert_eq!(square_count(&texts), 1, "정본 HWPX 도 `□` 가 1개다");
}

#[test]
fn issue_6873_lone_low_surrogate_becomes_the_oracle_square() {
    let texts = paragraph_texts(GYEONGJU);

    let hit = texts
        .iter()
        .find(|t| t.contains("신 청 인"))
        .unwrap_or_else(|| panic!("`신 청 인` 문단이 없다: {texts:?}"));
    assert_eq!(
        hit.as_str(),
        " \u{25A1}   신 청 인",
        "짝 없는 하위 서로게이트도 같은 글자로 받는다"
    );

    // 형제 항목 세 개가 같은 문서 안의 통제군이다 — 정본도 넷 다 `□` 로 적는다.
    for sibling in ["사 업 명", "보조사업의 목적", "사업량 및 사업비"] {
        let text = texts
            .iter()
            .find(|t| t.contains(sibling))
            .unwrap_or_else(|| panic!("`{sibling}` 문단이 없다"));
        assert!(
            text.starts_with(" \u{25A1}   "),
            "형제 항목 `{sibling}` 도 같은 머리글이어야 한다: {text:?}"
        );
    }
    assert_eq!(square_count(&texts), 4, "정본 HWPX 도 `□` 가 4개다");
}

#[test]
fn issue_6873_paired_surrogates_stay_on_the_plane_15_path() {
    let texts = paragraph_texts(PAIRED);

    // 쌍 갈래는 종전 코드가 이미 옳게 처리하던 자리다 — 그대로 평면 15 로 남는다.
    let plane15 = texts
        .iter()
        .flat_map(|t| t.chars())
        .filter(|c| ('\u{F0000}'..='\u{FFFFD}').contains(c))
        .count();
    assert!(
        plane15 >= 30,
        "서로게이트 쌍 34개가 평면 15 글자로 남아야 하는데 {plane15}개다"
    );

    // 반쪽 갈래가 쌍을 가로채면 여기서 `□` 가 생긴다.
    assert_eq!(
        square_count(&texts),
        0,
        "쌍 문서는 `□` 를 하나도 만들지 않는다"
    );
}
