//! [#8355] 한/글은 고정 줄간격 문단의 마지막 줄도 고정 피치를 차지한다 — 한/글 저장본의 고정
//! 줄간격 문단은 마지막 줄까지 `줄 높이 + 줄간격 = 고정값 / 2` 다(10k 코퍼스 표본 전부). PDF 편집
//! 우선 가져오기는 마지막 줄 줄간격을 0 으로 두고 문단 끝을 줄 바닥으로 재어, 한/글이 문단마다
//! 그 몫만큼 더 내려 같은 저장본을 18쪽(rhwp 16쪽)으로 그렸다.
#![cfg(feature = "pdf-import")]

use rhwp::model::style::LineSpacingType;
use rhwp::parser::pdf::{PdfImportLayout, PdfImportOptions};
use rhwp::DocumentCore;

fn import_editable() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본-2010.pdf");
    let options = PdfImportOptions {
        layout: PdfImportLayout::Editable,
        ..PdfImportOptions::default()
    };
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &options).unwrap()
}

#[test]
fn fixed_line_paragraphs_keep_the_pitch_on_their_last_row() {
    let core = import_editable();
    let doc = core.document();
    let mut checked = 0;
    for paragraph in doc.sections.iter().flat_map(|s| &s.paragraphs) {
        let shape = &doc.doc_info.para_shapes[paragraph.para_shape_id as usize];
        if shape.line_spacing_type != LineSpacingType::Fixed || paragraph.line_segs.len() < 2 {
            continue;
        }
        let pitch = shape.line_spacing / 2;
        let last = paragraph.line_segs.last().unwrap();
        if pitch <= last.line_height {
            continue;
        }
        assert_eq!(
            last.line_height + last.line_spacing,
            pitch,
            "고정 줄간격 문단의 마지막 줄 «{}»",
            paragraph.text.chars().take(16).collect::<String>()
        );
        checked += 1;
    }
    assert!(checked > 50, "검사한 고정 줄간격 본문 문단 {checked}개");
}

#[test]
fn fixed_line_value_follows_the_rows_of_the_whole_paragraph() {
    // 한/글은 고정값 하나로 문단 전체를 짠다. 고정값은 같은 단 안 줄 피치들과 몇 HWPUNIT 안에서
    // 같아야 한다 — 첫 두 줄(또는 이어 붙인 앞 조각)의 피치만 따르면 줄마다 차이가 쌓인다.
    let core = import_editable();
    let doc = core.document();
    let mut checked = 0;
    for paragraph in doc.sections.iter().flat_map(|s| &s.paragraphs) {
        let shape = &doc.doc_info.para_shapes[paragraph.para_shape_id as usize];
        if shape.line_spacing_type != LineSpacingType::Fixed || paragraph.line_segs.len() < 8 {
            continue;
        }
        let pitch = shape.line_spacing / 2;
        let rows: Vec<i32> = paragraph
            .line_segs
            .windows(2)
            .map(|pair| pair[1].vertical_pos - pair[0].vertical_pos)
            .filter(|step| *step > 0 && (step - pitch).abs() * 20 <= pitch)
            .collect();
        let mean = rows.iter().sum::<i32>() as f64 / rows.len() as f64;
        assert!(
            (mean - f64::from(pitch)).abs() <= 2.0,
            "«{}» 고정값 피치 {pitch} / 줄 피치 평균 {mean:.1}",
            paragraph.text.chars().take(16).collect::<String>()
        );
        checked += 1;
    }
    assert!(checked > 20, "검사한 긴 고정 줄간격 문단 {checked}개");
}
