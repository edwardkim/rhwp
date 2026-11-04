//! 관계형·고정형: 위 첨자의 문단 소속과 지정 문구의 출현 횟수를 검사한다.
//! 주석 갱신: 2026-11-01T03:26:13+09:00; 직전 파일 커밋: 2026-10-31T23:20:55+09:00 (d596bdff8a9d05a66d3460c10df2be180aa9a043).
//! 실제 입력 samples/21_언어_기출_편집가능본-2010.pdf 14쪽의 편집 가져오기 문단을 검사하며 페이지 래스터의 시각 일치율을 판정하지 않는다.
//! 근거: 원본 PDF의 14쪽과 mydocs/pr/assets/issue_8381/의 원본별 시각 증적.
//! [#8381] 한/글 위 첨자는 기준선을 정확히 0.5em 올린다. PDF 가져오기의 줄 묶기가 이 경계를
//! 부동소수 그대로 비교해, 같은 높이의 첨자가 줄마다 다르게 갈렸다.
//!
//! 독립 기준: 원본 `samples/21_언어_기출_편집가능본-2010.pdf` 14쪽. `pdftotext -bbox` 로 본
//! «+» 글자 상자(y 435.5~453.9)는 «NAD는 줄어든다.» 줄 상자(433.5~462.2) 안에 있다. 같은 쪽
//! 26번 ② «NAD⁺와» 의 «+» 도 같은 크기(7.08pt, 본문 11.04pt)·같은 높이로 올라가 있다.
//!
//! 수정 전에는 25번 ③ 둘째 줄의 «+» 만 기준선 차이가 0.5em 보다 0.00003pt 커서 독립 줄이
//! 되었다. 편집 본문은 «NAD 는 줄어든다.» 와 «+» 문단으로 갈리고, 그 문단이 줄 하나를 더
//! 차지했다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::parser::pdf::{PdfImportLayout, PdfImportOptions};

fn import_2010(layout: PdfImportLayout) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본-2010.pdf");
    let options = PdfImportOptions {
        layout,
        ..PdfImportOptions::default()
    };
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &options).unwrap()
}

fn body_texts(core: &DocumentCore) -> Vec<String> {
    core.document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .map(|paragraph| paragraph.text.clone())
        .collect()
}

#[test]
fn raised_superscript_stays_in_its_editable_row() {
    let texts = body_texts(&import_2010(PdfImportLayout::Editable));
    assert!(
        !texts.iter().any(|text| text.trim() == "+"),
        "위 첨자 «+» 가 독립 문단이 되지 않는다"
    );
    let row = |needle: &str| texts.iter().filter(|text| text.contains(needle)).count();
    assert_eq!(
        row("NAD+는 줄어든다."),
        1,
        "25번 ③ 둘째 줄의 첨자가 «NAD» 와 «는» 사이에 남는다"
    );
    assert_eq!(
        row("NAD 는 줄어든다."),
        0,
        "첨자 자리가 빈칸으로 남지 않는다"
    );
    // 대조군: 같은 높이의 첨자가 수정 전에도 줄에 남던 26번 ②.
    assert_eq!(row("NAD+와"), 1, "26번 ② 의 첨자는 그대로 줄에 남는다");
}
