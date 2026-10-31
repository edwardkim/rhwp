//! [#8381] PDF 편집 우선 가져오기가 원본 PDF 줄 위치와 한/글 재조판을 함께 맞춘다.
//!
//! 공백만 있는 한 줄 문단은 잉크가 없어 그 글자 상자가 점유한 내용이 아니다. 무표기본 지문
//! 상자 맨 위의 « » 줄(글자 11.04pt)은 다음 본문 줄까지 띠가 짧은데, 수정 전에는 글자 높이
//! 피치를 차지해 본문이 4.4pt 아래에 놓였다(원본 대비 5쪽 80.05·13쪽 77.86).
//!
//! 독립 기준: 원본 `samples/21_언어_기출_편집가능본.pdf` 의 `pdftotext -bbox-layout` 줄 위끝
//! (pt). 제목과 본문 첫 줄은 같은 11.04pt 글자라 위끝 차이가 곧 줄 위치 차이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::pdf::{PdfImportLayout, PdfImportOptions};

fn import_unmarked() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본.pdf");
    let options = PdfImportOptions {
        layout: PdfImportLayout::Editable,
        ..PdfImportOptions::default()
    };
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &options).unwrap()
}

fn starts(paragraph: &Paragraph, prefix: &str) -> bool {
    paragraph.text.trim_start().starts_with(prefix)
}

#[test]
fn passage_body_after_a_blank_row_keeps_its_source_offset() {
    let core = import_unmarked();
    let paragraphs = &core.document().sections[0].paragraphs;
    // (제목, 본문 첫 문단, 원본 위끝 차이 HU) — 원본 13쪽 154.87→178.51pt, 5쪽 154.87→179.11pt.
    for (title, body, source_gap) in [
        ("[25~27]", "암세포의 대사 과정은", 2364),
        ("[10~12]", "살펴보건대", 2424),
    ] {
        let at = paragraphs
            .iter()
            .position(|p| starts(p, title))
            .unwrap_or_else(|| panic!("{title} 제목 문단"));
        let blank = &paragraphs[at + 1];
        assert!(
            !blank.text.is_empty() && blank.text.chars().all(char::is_whitespace),
            "전제: {title} 제목과 본문 사이에 공백만 있는 문단이 있다"
        );
        let next = &paragraphs[at + 2];
        assert!(
            starts(next, body),
            "{title} 뒤 본문 첫 문단: {:?}",
            next.text
        );
        let gap = next.line_segs[0].vertical_pos - paragraphs[at].line_segs[0].vertical_pos;
        assert!(
            (gap - source_gap).abs() <= 2,
            "{title}: 제목→본문 첫 줄 {gap} HU, 원본 {source_gap} HU"
        );
        // 한/글은 마지막 줄을 고정 피치로 다시 짠다 — 그 피치가 저장 줄이 차지한 몫과 같아야
        // 두 엔진이 본문을 같은 자리에 둔다.
        let row = &blank.line_segs[0];
        let shape = &core.document().doc_info.para_shapes[blank.para_shape_id as usize];
        assert_eq!(
            shape.line_spacing / 2,
            row.line_height + row.line_spacing,
            "{title}: 공백 줄의 한/글 피치 = 저장 줄이 차지한 몫"
        );
        assert_eq!(
            next.line_segs[0].vertical_pos,
            row.vertical_pos + row.line_height + row.line_spacing,
            "{title}: 본문은 공백 줄 바로 뒤에서 시작한다"
        );
    }
}
