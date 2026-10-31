//! [#8355] 한/글은 고정 줄간격 문단의 마지막 줄도 고정 피치를 차지한다 — 한/글 저장본의 고정
//! 줄간격 문단은 마지막 줄까지 `줄 높이 + 줄간격 = 고정값 / 2` 다(10k 코퍼스 표본 전부). PDF 편집
//! 우선 가져오기는 마지막 줄 줄간격을 0 으로 두고 문단 끝을 줄 바닥으로 재어, 한/글이 문단마다
//! 그 몫만큼 더 내려 같은 저장본을 18쪽(rhwp 16쪽)으로 그렸다.
//! 예외는 원본 PDF 가 다음 글상자를 고정 피치보다 가까이 둔 관측 꼬리 구간(#8220
//! `observedTailBands`)뿐이다 — 그 줄은 원본 위치를 따르며 가져오기 보고서에 기록된다.
#![cfg(feature = "pdf-import")]

use rhwp::model::style::LineSpacingType;
use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportLayout, PdfImportOptions};
use rhwp::DocumentCore;

const SOURCE: &str = "samples/21_언어_기출_편집가능본-2010.pdf";

fn options() -> PdfImportOptions {
    PdfImportOptions {
        layout: PdfImportLayout::Editable,
        ..PdfImportOptions::default()
    }
}

fn source_bytes() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).unwrap()
}

fn import_editable() -> DocumentCore {
    DocumentCore::from_pdf_bytes(&source_bytes(), &options()).unwrap()
}

#[test]
fn fixed_line_paragraphs_keep_the_pitch_on_their_last_row() {
    let result = parse_pdf_with_options(&source_bytes(), &options()).unwrap();
    // 원본 PDF 가 다음 글상자를 피치보다 가까이 둔 구간에서 받아들인 꼬리 줄간격들.
    let mut bands: Vec<i64> = result.metadata.editable_layout["sections"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|section| {
            section["observedTailBands"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .map(|band| band["acceptedTailHu"].as_i64().unwrap())
        .collect();
    let doc = &result.document;
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
        let head = paragraph.text.chars().take(16).collect::<String>();
        if last.line_height + last.line_spacing != pitch {
            // 피치를 줄인 꼬리는 원본이 관측한 구간 하나로 설명돼야 한다. 늘리지는 않는다.
            assert!(
                last.line_height + last.line_spacing < pitch,
                "고정 줄간격 문단의 마지막 줄 «{head}»이 피치 {pitch}를 넘는다"
            );
            let band = bands
                .iter()
                .position(|&tail| tail == i64::from(last.line_spacing))
                .unwrap_or_else(|| {
                    panic!("고정 줄간격 문단의 마지막 줄 «{head}»: 관측 구간 없이 피치 {pitch}보다 짧다")
                });
            bands.swap_remove(band);
            continue;
        }
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

#[test]
fn column_paragraphs_stack_like_hancom_reflow() {
    // [#8361] 한/글은 단 안 문단을 `앞 문단 끝(마지막 줄 + 꼬리) + 아래 간격 + 앞 간격` 에 차례로
    // 쌓는다. 저장 줄이 이 자리와 다르면 rhwp(저장 줄)와 한/글(재조판)이 단 끝 줄을 다른 단에 둔다.
    let core = import_editable();
    let doc = core.document();
    let mut checked = 0;
    let paragraphs = &doc.sections[0].paragraphs;
    for pair in paragraphs.windows(2) {
        let (previous, next) = (&pair[0], &pair[1]);
        if next.column_type != rhwp::model::paragraph::ColumnBreakType::None {
            continue;
        }
        let (Some(last), Some(first)) = (previous.line_segs.last(), next.line_segs.first()) else {
            continue;
        };
        // 이어 붙인 문단의 되감기나 쪽 첫머리는 쌓기 대상이 아니다.
        if first.vertical_pos <= last.vertical_pos {
            continue;
        }
        let before = &doc.doc_info.para_shapes[previous.para_shape_id as usize];
        let after = &doc.doc_info.para_shapes[next.para_shape_id as usize];
        let expected = last.vertical_pos
            + last.line_height
            + last.line_spacing.max(0)
            + before.spacing_after.max(0) / 2
            + after.spacing_before.max(0) / 2;
        assert!(
            (first.vertical_pos - expected).abs() <= 2,
            "«{}» 저장 위치 {} / 한/글 쌓기 {}",
            next.text.chars().take(16).collect::<String>(),
            first.vertical_pos,
            expected
        );
        checked += 1;
    }
    assert!(checked > 100, "검사한 단 안 문단 경계 {checked}개");
}

#[test]
fn body_bottom_sits_between_filled_and_spilled_rows() {
    // [#8361] 원본에서 다음 단으로 넘어간 줄은 본문 아래 끝에 들어가지 않아야 하고, 들어간 줄은
    // 모두 들어가야 한다 — 한/글이 같은 자리에서 단을 넘긴다.
    let core = import_editable();
    let doc = core.document();
    let page = &doc.sections[0].section_def.page_def;
    let body = page.height as i32 - page.margin_top as i32 - page.margin_bottom as i32;
    let rows = || doc.sections[0].paragraphs.iter().flat_map(|p| &p.line_segs);
    let filled = rows()
        .map(|s| s.vertical_pos + s.line_height)
        .max()
        .unwrap();
    assert!(filled <= body, "들어간 줄 바닥 {filled} ≤ 본문 {body}");
    for p in &doc.sections[0].paragraphs {
        for pair in p.line_segs.windows(2) {
            if pair[1].vertical_pos < pair[0].vertical_pos {
                let spilled = pair[0].vertical_pos
                    + pair[0].line_height
                    + pair[0].line_spacing.max(0)
                    + pair[1].line_height;
                if spilled > filled {
                    assert!(spilled > body, "넘어간 줄 바닥 {spilled} > 본문 {body}");
                }
            }
        }
    }
}
