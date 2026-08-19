//! [Issue #5563] 문단 축을 넘어서는 `textpos` 를 실은 `hp:lineseg` 는 저장하지 않는다.
//!
//! 원본이 들고 있던 낡은 줄나눔 캐시가 HWPX 산출에 그대로 옮겨 실리면(07990: 길이
//! 15 문단에 `textpos=119` 등 5개 문단) 한글 2022 가 "다음 줄은 119번째 글자에서
//! 시작"을 14글자 문단에서 해소하려다 **파일 개방이 끝나지 않는다**(COM `Open()`
//! 3,663초 미반환). rhwp 는 자기가 쓴 파일을 그대로 다시 읽으므로 `--verify` 로는
//! 보이지 않는다.
//!
//! 판정은 HWP5 저장기의 #4677 계약과 같다 — 경계 `text_start == char_count` 는
//! 한컴 자신이 쓰는 정상값이라 `>` 이고, 범위 밖이 나오면 그 앞까지만 남긴다.

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::serializer::hwpx::context::SerializeContext;
use rhwp::serializer::hwpx::section::write_section;
use std::fs;
use std::path::Path;

fn seg(text_start: u32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos: 0,
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 850,
        line_spacing: 600,
        column_start: 0,
        segment_width: 42520,
        tag: 393_216,
    }
}

/// 텍스트 3글자(끝 마커 포함 `char_count=4`) 문단에 주어진 줄들을 실어 section XML 을 얻는다.
fn section_xml(line_segs: Vec<LineSeg>) -> String {
    let mut para = Paragraph::default();
    para.text = "가나다".to_string();
    para.char_offsets = vec![0, 1, 2];
    para.char_count = 4; // 글자 3 + 끝 마커 1
    para.line_segs = line_segs;

    let mut section = Section::default();
    section.paragraphs.push(para);
    let mut doc = Document::default();
    doc.sections.push(section.clone());
    let mut ctx = SerializeContext::collect_from_document(&doc);
    let bytes = write_section(&section, &doc, 0, &mut ctx).expect("section 직렬화");
    String::from_utf8(bytes).expect("UTF-8 section XML")
}

/// 계약 1 — 범위 밖 줄부터 잘라 내고 그 앞 접두부만 남는다.
#[test]
fn line_segs_beyond_paragraph_axis_are_dropped() {
    let xml = section_xml(vec![seg(0), seg(2), seg(99)]);

    assert!(xml.contains(r#"textpos="0""#), "첫 줄은 남아야 함: {xml}");
    assert!(xml.contains(r#"textpos="2""#), "범위 안 줄은 남아야 함");
    assert!(
        !xml.contains(r#"textpos="99""#),
        "문단 축(4)을 넘는 줄은 저장되면 안 됨"
    );
}

/// 계약 2 — 경계값 `text_start == char_count` 는 한컴 자신이 쓰는 정상값이라 남는다.
#[test]
fn line_seg_at_exact_axis_end_is_kept() {
    let xml = section_xml(vec![seg(0), seg(4)]);

    assert!(xml.contains(r#"textpos="4""#), "끝을 가리키는 줄은 정상값");
}

/// 계약 3 — 첫 줄부터 범위 밖이면 `linesegarray` 를 통째로 생략한다(한글이 스스로 조판).
#[test]
fn all_out_of_axis_line_segs_drop_the_whole_array() {
    let xml = section_xml(vec![seg(7), seg(12)]);

    assert!(
        !xml.contains("<hp:linesegarray"),
        "전부 범위 밖이면 요소를 내지 않는다: {xml}"
    );
}

/// 계약 4 — 저장할 수 없는 줄은 IR 왕복 비교에서도 차이로 잡히지 않는다.
///
/// 이 규칙이 없으면 `export-hwpx --verify` 가 자기 계약대로 버린 줄을 "재파싱 후 IR
/// 차이" 로 신고해 exit 3 이 된다.
#[test]
fn out_of_axis_line_segs_are_not_reported_as_ir_difference() {
    fn doc_with(line_segs: Vec<LineSeg>) -> Document {
        let mut para = Paragraph::default();
        para.text = "가나다".to_string();
        para.char_offsets = vec![0, 1, 2];
        para.char_count = 4;
        para.line_segs = line_segs;
        let mut section = Section::default();
        section.paragraphs.push(para);
        let mut doc = Document::default();
        doc.sections.push(section);
        doc
    }

    let source = doc_with(vec![seg(0), seg(2), seg(99)]);
    let reparsed = doc_with(vec![seg(0), seg(2)]);

    let diffs = rhwp::serializer::hwpx::roundtrip::diff_linesegs(&source, &reparsed);
    assert!(
        diffs.is_empty(),
        "저장 못 하는 줄은 비교 대상이 아니어야 함: {diffs:?}"
    );
}

/// DocumentCore는 빈 누름틀의 안내문을 IR 본문에서 비울 수 있으나, HWPX 저장 때
/// Field 슬롯과 안내문을 다시 방출한다. 이 경우 직렬화된 축은 `char_count`보다
/// 길 수 있으므로, 그 뒤의 유효 lineseg를 잘라 셀 세로 정렬을 바꾸면 안 된다.
#[test]
fn field_slots_extend_the_serialized_axis() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1893_clickhere_field_roundtrip.hwpx");
    let data = fs::read(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    let diff = roundtrip_geom(&data, Via::Hwpx).expect("HWPX 라운드트립");

    assert_eq!(diff.page_count_a, 1);
    assert_eq!(diff.page_count_a, diff.page_count_b);
    assert!(
        diff.max_disp <= 1.0 && diff.pages.iter().all(|page| !page.structure_mismatch),
        "필드 슬롯 뒤 lineseg가 보존되어야 함: {diff:?}"
    );
}
