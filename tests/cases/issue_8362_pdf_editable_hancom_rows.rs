//! [#8362] PDF 편집 우선 가져오기 저장본을 한/글 2024 로 인쇄하면 시험지 13쪽이 82%로 어긋났다.
//! 원인은 한/글의 두 조판 규칙이다(output/pr-review/rebase-f125/cmpP13/hx·hx2 의 한/글 실험).
//! - 고정 줄간격 문단의 마지막 줄은 저장 꼬리를 무시하고 고정 피치로 다시 짠다. 음수 문단 아래
//!   간격은 반영한다. rhwp 는 저장 줄 위치를 따르고 음수 간격을 무시한다.
//! - 줄 맞춤은 줄 마지막 글자의 자간을 폭에 넣지 않는다. 단 끝에 붙은 원본 줄은 마지막 글자의
//!   외곽이 줄보다 넓으면 그 낱말을 다음 줄로 보낸다(24번 제목 «것|은?»).
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
fn next_paragraph_after_a_fixed_line_paragraph_stacks_where_hancom_puts_it() {
    // rhwp 는 다음 문단을 저장 줄 위치에 그린다. 한/글은 같은 단에서 `앞 문단 마지막 줄 + 고정
    // 피치 + 아래 간격 + 다음 문단 앞 간격`(음수 간격 포함)에 쌓는다. 앞 문단 꼬리를 원본 꼬리
    // 구간으로 줄였으면 그 몫이 문단 간격에 들어 있어야 두 자리가 같다(13쪽 24번 제목 뒤 571).
    let core = import_editable();
    let doc = core.document();
    let (mut checked, mut shortened) = (0, 0);
    for pair in doc.sections[0].paragraphs.windows(2) {
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
        // 단을 넘어 이어지는 문단(저장 줄 되감기)의 마지막 줄은 한/글 실험 범위 밖이다 — 2010판
        // 7쪽 «물론 르포르도» 뒤는 원본과 같은 자리에 오지만 이 식보다 57 아래다.
        if previous
            .line_segs
            .windows(2)
            .any(|rows| rows[1].vertical_pos < rows[0].vertical_pos)
        {
            continue;
        }
        let before = &doc.doc_info.para_shapes[previous.para_shape_id as usize];
        let after = &doc.doc_info.para_shapes[next.para_shape_id as usize];
        let pitch = before.line_spacing / 2;
        if before.line_spacing_type != LineSpacingType::Fixed || pitch <= last.line_height {
            continue;
        }
        // Paragraph measures are stored as twice HWPUNIT.
        let hancom =
            last.vertical_pos + pitch + before.spacing_after / 2 + after.spacing_before / 2;
        assert!(
            (first.vertical_pos - hancom).abs() <= 2,
            "«{}» 저장 위치 {} / 한/글 쌓기 {hancom}",
            next.text.chars().take(16).collect::<String>(),
            first.vertical_pos
        );
        checked += 1;
        if last.line_height + last.line_spacing < pitch {
            shortened += 1;
        }
    }
    assert!(checked > 100, "검사한 고정 줄간격 문단 경계 {checked}개");
    assert!(shortened > 0, "꼬리를 원본 구간으로 줄인 문단 경계가 없다");
}

#[test]
fn column_wide_title_ends_its_row_with_spacing_not_outline() {
    // 24번 제목은 원본에서 한 줄이고 단 오른쪽 끝(406.27pt)에 0.03pt 차이로 붙어 있다. 한/글은
    // 줄 마지막 글자의 자간을 줄 폭에 넣지 않아, 마지막 «?» 자간을 줄여도 끊고(hx/v18) 장평을
    // 약 12 HWPUNIT 줄이면 한 줄에 넣었다(hx/v19). 원본 끝 위치는 그 글자의 양수 자간이 채우고,
    // 외곽은 적어도 그 문턱만큼 원본 끝 안쪽에서 끝나야 한다.
    const HANCOM_FIT_MARGIN: i32 = 12;
    let core = import_editable();
    let doc = core.document();
    let title = doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .find(|p| p.text.starts_with("24.") && p.text.contains("않은 것은?"))
        .expect("24번 제목 문단");
    let next_row = title
        .line_segs
        .get(1)
        .map_or(u32::MAX, |seg| seg.text_start);
    let row: Vec<(u32, char)> = title
        .text
        .chars()
        .enumerate()
        .map(|(i, c)| (title.char_offsets[i], c))
        .filter(|&(pos, _)| pos < next_row)
        .collect();
    let first: String = row.iter().map(|&(_, c)| c).collect();
    assert!(
        first.trim_end().ends_with("않은 것은?"),
        "24번 제목은 원본처럼 첫 줄에서 끝난다: «{first}»"
    );
    let &(pos, last) = row.iter().rev().find(|(_, c)| !c.is_whitespace()).unwrap();
    assert_eq!(last, '?');
    let id = title
        .char_shapes
        .iter()
        .rev()
        .find(|r| r.start_pos <= pos)
        .unwrap()
        .char_shape_id;
    let shape = &doc.doc_info.char_shapes[id as usize];
    // 자간은 글자 크기의 백분율이다(영문 자리: «?»).
    let spacing = i32::from(shape.spacings[1]) * shape.base_size / 100;
    assert!(
        spacing >= HANCOM_FIT_MARGIN,
        "24번 제목 마지막 «?» 자간 {spacing} HWPUNIT < 한/글 줄 맞춤 문턱 {HANCOM_FIT_MARGIN}"
    );
}
