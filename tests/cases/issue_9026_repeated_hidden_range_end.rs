//! [#9026] 고정형 계약 — 같은 문단에서 반복 저장된 숨김 범위 끝 표지가 있는 HWPX 를 연다.
//!
//! 주석 갱신: 2026-11-13 10:45 KST(+09:00). 직전 파일 변경 커밋 be932a3ead
//! (2026-11-13 10:08:11 +0900, 새 파일).
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
//! - 둘째 입력: 공개 보도자료 `samples/issue9026/press_release_orphan_hidden_end.hwpx`
//!   (2026 돌봄로봇 네트워크 포럼, korea.kr 배포). 반복 끝(deleteEnd 24·25·127, insertEnd
//!   120·138·150)에 더해, 문서의 어느 XML 에도 시작 표지가 없는 숨김 삭제 끝(deleteEnd 103·107·126,
//!   TcId 139, `paraend="0"`)이 있다. 기준 `pdf/issue9026/press_release_orphan_hidden_end-hwpx-2020.pdf`
//!   (로컬 오라클 engine 2020, Hwp 2022 12.0.0.4547 Print) 8쪽은 그 끝 표지 앞 글(「다섯 번째 세션부터
//!   아홉 번째 세션까지 …」)을 그대로 출력한다. 열린 적 없는 끝은 아무것도 감싸지 않는다.
//!
//! 검사: 문서 열기 성공, 전체 쪽수 = 한컴 Print(5쪽·8쪽), 숨김 삭제 범위 내용(「미래의료연구부장」,
//! 「실제 돌봄 현장의 특성을 고려하고」)이 본문에 없고, 삭제 범위 뒤 문단 머리·삽입 글·짝 없는 끝
//! 앞 글이 한컴과 같이 남는다.
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

/// 문서 전체에 시작 표지가 없는 숨김 삭제 끝 표지(Id 103·107·126)와 반복 끝을 함께 가진 공개 보도자료.
const ORPHAN_SOURCE: &str = "samples/issue9026/press_release_orphan_hidden_end.hwpx";
/// 한컴 2022 Print 의 전체 쪽수.
const ORPHAN_PRINT_PAGES: u32 = 8;
/// 숨김 삭제 범위(Id 124) 안에만 있는 원문 조각.
const ORPHAN_HIDDEN: &str = "실제돌봄현장의특성을고려하고";
/// 같은 자리의 삽입 범위(Id 118) 글.
const ORPHAN_INSERTED: &str = "의료와복지가동시에필요한돌봄현장";
/// 짝 없는 끝(Id 103) 앞에 있는 같은 문단의 글. 한컴 Print 는 이 글을 숨기지 않는다.
const ORPHAN_KEPT_BEFORE_END: &str = "다섯번째세션부터아홉번째세션까지";

#[test]
fn end_of_a_never_opened_hidden_range_hides_nothing() {
    let bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ORPHAN_SOURCE))
        .expect(ORPHAN_SOURCE);
    let document = rhwp::parse_document(&bytes).expect("짝 없는 끝 표지가 있어도 문서를 연다");
    let paragraphs = texts(&document.sections[0].paragraphs);

    assert!(
        paragraphs.iter().all(|text| !text.contains(ORPHAN_HIDDEN)),
        "열린 숨김 삭제 범위 내용은 본문에 남지 않는다"
    );
    assert!(
        paragraphs.iter().any(|text| text.contains(ORPHAN_INSERTED)),
        "삽입 범위 글은 남는다"
    );
    assert!(
        paragraphs
            .iter()
            .any(|text| text.contains(ORPHAN_KEPT_BEFORE_END)),
        "짝 없는 끝 표지 앞 글은 숨기지 않는다(한컴 Print 와 같음)"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        ORPHAN_PRINT_PAGES,
        "한컴 Print 의 전체 쪽수"
    );
}
