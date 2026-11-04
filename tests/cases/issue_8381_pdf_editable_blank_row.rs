//! 관계형: PR #8408의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:35:10+09:00 (`6da5cebfb8b407a2a63b15f82518140850815614`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `mydocs/pr/assets/issue_8381/integration_20261103/editable-2010.hwpx`; 독립 Print `samples/21_언어_기출_편집가능본-2010.pdf`.
//! 전체 16쪽, 전쪽 최저 Native 97.80% / fresh WASM 98.08%.
//! 주장 14쪽 raw: Native 98.93% / fresh WASM 98.91%.
//! 입력 `mydocs/pr/assets/issue_8381/integration_20261103/editable-2020.hwpx`; 독립 Print `samples/21_언어_기출_편집가능본-2020.pdf`.
//! 전체 15쪽, 전쪽 최저 Native 99.19% / fresh WASM 99.18%.
//! 주장 1쪽 raw: Native 99.76% / fresh WASM 99.73%.
//! 입력 `mydocs/pr/assets/issue_8381/integration_20261103/editable-plain.hwpx`; 독립 Print `samples/21_언어_기출_편집가능본.pdf`.
//! 전체 15쪽, 전쪽 최저 Native 98.54% / fresh WASM 98.44%.
//! 주장 5쪽 raw: Native 99.67% / fresh WASM 99.68%.
//! 주장 13쪽 raw: Native 99.68% / fresh WASM 99.69%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8408_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
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
