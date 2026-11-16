//! 저장 회귀: 값이 든 누름틀 뒤에 같은 문단의 컨트롤이 이어져도, HWP5 로 다시 저장한 문서에서
//! 누름틀은 값만 감싸고 뒤 컨트롤은 누름틀 밖에 남는다.
//! 주석 갱신: 2026-11-15 (KST), 새 파일.
//!
//! # 무엇이 깨져 있었나
//!
//! [#9082] 문단 글자의 끝에서 끝나는 누름틀(`end_char_idx == 글자 수`)의 끝 표식(FIELD_END)을
//! HWP5 직렬화기는 "`control_idx + inner_slot_count` 번째 컨트롤 뒤"에 쓰도록 예약한다. 값이
//! 있으면 그 누름틀 시작은 값의 첫 글자 앞에서 이미 나가므로, 글자 뒤 컨트롤을 쓰는 단계는 그
//! 번호를 지난 채 시작한다. 예약된 끝 표식은 꺼내지지 않고 남아 문단 맨 끝, 곧 뒤따르던 컨트롤들
//! 뒤에 쓰였다. 저장본에서는 누름틀이 표·책갈피를 감싼다(`inner_slot_count` 0 → 뒤 컨트롤 수).
//!
//! 원본 스트림을 그대로 내보내는 저장(아무것도 고치지 않은 구역)에서는 드러나지 않는다. 구역의
//! 어느 문단이든 고쳐 레코드를 다시 만들 때 일어난다.
//!
//! # 기대값의 출처
//!
//! - `samples/issue9082/textbox_filled_field_tac_tables.hwp`: 한/글 2022(12.0.0.535)에서 글상자 안
//!   한 문단에 누름틀과 글자처럼 취급 표 2개를 넣고, 누름틀에 「값」을 입력해 저장했다. 원시
//!   PARA_TEXT 는 `누름틀 시작 0 · 값 8 · 누름틀 끝 9 · 표1 17 · 표2 25` 다. 한/글은 값을 채워도
//!   끝 표식을 값 바로 뒤, 표 앞에 쓴다.
//! - `mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp`: 본문 문단 하나가
//!   `누름틀 시작 0 · 글자 11자 · 누름틀 끝 19 · 책갈피 27` 이다. 글상자 밖, 표가 아닌 컨트롤에서
//!   같은 구성을 가진 기존 문서다.
//!
//! # 검사하는 것
//!
//! fixture 계약(값이 든 누름틀이 글자 끝에서 닫히고 그 뒤에 컨트롤이 있음)과, 레코드를 다시 만든
//! 저장본을 재파싱했을 때 누름틀 범위가 원본과 같음. 바이트 위치는 고정하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::ir_field_sweep::sweep_hwp5_rebuild_roundtrip;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;

const TEXTBOX: &str = "samples/issue9082/textbox_filled_field_tac_tables.hwp";
const BOOKMARK: &str = "mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp";

fn read(sample: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    std::fs::read(&path).unwrap_or_else(|e| panic!("재현물 읽기 {sample}: {e}"))
}

/// 편집 뒤 저장과 같은 경로: 원본 스트림을 버리고 모델에서 레코드를 다시 만든 저장본을 재파싱한다.
fn rebuilt(bytes: &[u8]) -> Document {
    let mut doc = rhwp::parse_document(bytes).expect("문서 읽기");
    doc.doc_info.raw_stream_dirty = true;
    for section in &mut doc.sections {
        section.raw_stream = None;
    }
    let saved = rhwp::serializer::serialize_document(&doc).expect("HWP5 저장");
    rhwp::parse_document(&saved).expect("저장본 재파싱")
}

/// 문서의 글상자 안 첫 문단(누름틀과 표가 든 문단).
fn textbox_paragraph(doc: &Document) -> &Paragraph {
    doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Shape(shape) => shape.drawing()?.text_box.as_ref()?.paragraphs.first(),
            _ => None,
        })
        .expect("글상자가 없다 — 시험 설정 오류")
}

/// 누름틀 범위마다 (여는 컨트롤 순번, 글자 시작, 글자 끝, 안에 든 컨트롤 수).
fn field_spans(para: &Paragraph) -> Vec<(usize, usize, usize, usize)> {
    para.field_ranges
        .iter()
        .map(|range| {
            (
                range.control_idx,
                range.start_char_idx,
                range.end_char_idx,
                range.inner_slot_count,
            )
        })
        .collect()
}

fn control_kinds(para: &Paragraph) -> Vec<&'static str> {
    para.controls
        .iter()
        .map(|control| match control {
            Control::Field(_) => "누름틀",
            Control::Table(_) => "표",
            Control::Bookmark(_) => "책갈피",
            _ => "기타",
        })
        .collect()
}

/// 값이 든 누름틀이 글자 끝에서 닫히고, 그 뒤에 누름틀 밖 컨트롤이 있는 문단인가.
fn closes_a_filled_field_before_trailing_controls(para: &Paragraph) -> bool {
    let text_len = para.text.chars().count();
    para.field_ranges.iter().any(|range| {
        range.start_char_idx < range.end_char_idx
            && range.end_char_idx == text_len
            && range.control_idx + range.inner_slot_count + 1 < para.controls.len()
    })
}

/// fixture 계약 — 시험 설정 오류를 먼저 걸러낸다.
#[test]
fn textbox_fixture_closes_the_filled_field_before_the_tables() {
    let doc = rhwp::parse_document(&read(TEXTBOX)).expect("문서 읽기");
    let para = textbox_paragraph(&doc);
    assert_eq!(para.text, "값");
    assert_eq!(control_kinds(para), ["누름틀", "표", "표"]);
    assert_eq!(field_spans(para), [(0, 0, 1, 0)]);
    assert!(closes_a_filled_field_before_trailing_controls(para));
}

/// 결함 검출 — 수정 전에는 저장본의 누름틀이 표 2개를 감싼다(`(0, 0, 1, 2)`).
#[test]
fn rebuilt_hwp5_keeps_the_tables_outside_the_filled_field() {
    let bytes = read(TEXTBOX);
    let original = rhwp::parse_document(&bytes).expect("문서 읽기");
    let saved = rebuilt(&bytes);
    let before = textbox_paragraph(&original);
    let after = textbox_paragraph(&saved);
    assert_eq!(after.text, before.text);
    assert_eq!(control_kinds(after), control_kinds(before));
    assert_eq!(
        field_spans(after),
        field_spans(before),
        "저장본의 누름틀 범위가 원본과 다르다 — 누름틀 끝이 뒤 컨트롤 뒤에 쓰였다(#9082)"
    );
}

/// 글상자 밖 본문 문단, 표가 아닌 컨트롤(책갈피)에서도 같다.
#[test]
fn rebuilt_hwp5_keeps_the_bookmark_outside_the_filled_field() {
    let bytes = read(BOOKMARK);
    let original = rhwp::parse_document(&bytes).expect("문서 읽기");
    let target = original
        .sections
        .iter()
        .flat_map(|section| section.paragraphs.iter())
        .find(|para| {
            closes_a_filled_field_before_trailing_controls(para)
                && control_kinds(para).contains(&"책갈피")
        })
        .expect("값이 든 누름틀 뒤에 책갈피가 있는 본문 문단이 없다 — 시험 설정 오류");
    assert!(field_spans(target).iter().all(|span| span.3 == 0));

    let report = sweep_hwp5_rebuild_roundtrip(&bytes).expect("레코드 재생성 왕복");
    let moved: Vec<_> = report
        .counts()
        .iter()
        .filter(|(path, _)| path.contains("field_ranges[]"))
        .collect();
    assert!(
        moved.is_empty(),
        "저장본에서 누름틀 범위가 달라진 경로가 있다(#9082): {moved:?}"
    );
}
