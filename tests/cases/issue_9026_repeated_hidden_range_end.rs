//! [#9026] 고정형 계약 — 같은 문단에서 반복 저장된 숨김 범위 끝 표지가 있는 HWPX 를 연다.
//!
//! 주석 갱신: 2026-11-13 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 보도자료 `samples/issue9026/press_release_repeated_hidden_end.hwpx`
//!   (질병관리청 156594737, rhwp-issue-docs#1088). 한 문단에서 숨김 삭제 범위
//!   `deleteBegin Id="7" TcId="3"` 뒤 `deleteEnd Id="7"` 가 run 분할 지점에 두 번 저장되어 있다.
//! - 기준: `pdf/issue9026/press_release_repeated_hidden_end-print-2022.pdf`
//!   (Hwp 2022 12.0.0.4547 Print, Hancom PDF 1.3.0.547) 5쪽. 삭제 범위는 숨겨지고 문단은
//!   「박현영 국립보건연구원장은 …」으로 시작한다.
//! - 수정 전 rhwp(devel `31a4225757`)는 `tracking structure: hidden range end without begin`
//!   으로 문서 전체를 거부했다.
//!
//! 검사: 문서 열기 성공, 전체 쪽수 = 한컴 Print 5쪽, 숨김 삭제 범위 내용(「미래의료연구부장」)
//! 이 본문에 없고, 삭제 범위 뒤 문단 내용이 한컴과 같은 시작으로 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::paragraph::Paragraph;
use rhwp::DocumentCore;
use std::path::PathBuf;

const SOURCE: &str = "samples/issue9026/press_release_repeated_hidden_end.hwpx";
/// 한컴 2022 Print 의 전체 쪽수.
const PRINT_PAGES: u32 = 5;
/// 숨김 삭제 범위(Id 7) 안에만 있는 원문 조각.
const HIDDEN: &str = "미래의료연구부장";
/// 삭제 범위 뒤에 남는 문단 머리(한컴 Print 의 문단 시작).
const KEPT_HEAD: &str = "박현영국립보건연구원장은";

fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn texts(paragraphs: &[Paragraph]) -> Vec<String> {
    paragraphs.iter().map(|p| ink(&p.text)).collect()
}

#[test]
fn repeated_end_of_a_closed_hidden_range_still_opens_the_document() {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
    let document = rhwp::parse_document(&bytes).expect("반복 끝 표지가 있어도 문서를 연다");
    let paragraphs = texts(&document.sections[0].paragraphs);

    assert!(
        paragraphs.iter().all(|text| !text.contains(HIDDEN)),
        "숨김 삭제 범위 내용은 본문에 남지 않는다"
    );
    assert!(
        paragraphs.iter().any(|text| text.starts_with(KEPT_HEAD)),
        "삭제 범위 뒤 내용은 한컴 Print 와 같은 문단 머리로 남는다"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), PRINT_PAGES, "한컴 Print 의 전체 쪽수");
}
