//! [#9032] 고정형 계약 — HWP 3.0 날짜 코드(특수 문자 8)는 문단 글자 수에서 48 hchar 를 차지한다.
//!
//! 주석 갱신: 2026-11-13 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 문서 `samples/issue9032/tax_relief_form_date_code.hwp`
//!   (1999년 국세청 조세감면 건의서 서식 41868, rhwp-issue-docs#1091). 첫 문단 머리말 안
//!   문단의 글자 수 49 = 날짜 코드 96바이트(48 hchar) + 문단 끝 1.
//! - 기준: `pdf/issue9032/tax_relief_form_date_code-print-2022.pdf`
//!   (Hwp 2022 12.0.0.4547 Print, Hancom PDF 1.3.0.547) 4쪽.
//! - 수정 전 rhwp(devel `31a4225757`)는 날짜 코드를 4 hchar 로 세어 문단 끝 뒤 바이트를
//!   글자로 읽었고, 본문이 어긋나 `failed to fill whole buffer` 로 문서 전체를 거부했다.
//!
//! 검사: 문서 열기 성공, 머리말 문단 리스트가 날짜 코드 문단 하나로 끝나고, 본문에 한컴
//! Print 의 서식 제목·작성요령이 순서대로 있으며, 전체 쪽수 = 한컴 Print 4쪽.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::DocumentCore;
use std::path::PathBuf;

const SOURCE: &str = "samples/issue9032/tax_relief_form_date_code.hwp";
/// 한컴 2022 Print 의 전체 쪽수.
const PRINT_PAGES: u32 = 4;

fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 본문·표 셀 문단 글을 문서 순서대로 모은다(머리말·꼬리말 제외).
fn body_text(paragraphs: &[Paragraph], out: &mut String) {
    for paragraph in paragraphs {
        out.push_str(&ink(&paragraph.text));
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    body_text(&cell.paragraphs, out);
                }
            }
        }
    }
}

#[test]
fn date_code_counts_its_whole_structure_in_the_paragraph_char_count() {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
    let document = rhwp::parse_document(&bytes).expect("날짜 코드가 있는 HWP 3.0 을 연다");
    let paragraphs = &document.sections[0].paragraphs;

    let header = paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Header(header) => Some(header),
            _ => None,
        })
        .expect("첫 문단의 머리말");
    assert_eq!(
        header.paragraphs.len(),
        1,
        "머리말 리스트는 날짜 코드 문단 하나로 끝난다"
    );

    let mut text = String::new();
    body_text(paragraphs, &mut text);
    let title = text.find("[별지제1호서식]").expect("서식 번호");
    let heading = text.find("조세감면건의서작성요령").expect("뒷면 제목");
    let last = text
        .find("유를해당항목에기재합니다")
        .expect("작성요령 마지막 항목");
    assert!(
        title < heading && heading < last,
        "한컴 Print 와 같은 순서"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), PRINT_PAGES, "한컴 Print 의 전체 쪽수");
}
