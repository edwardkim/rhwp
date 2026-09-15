//! [#4680] 합성 쪽나눔이 HWP5 저장본에 실려 한글이 쪽을 불린다.
//!
//! ## 증상
//!
//! 코퍼스 `1480000-201400060`(40쪽)을 `convert` 하면 한글이 **50쪽**으로, `1170000-200500003`
//! (264쪽)은 **327쪽**으로 연다. 글자 수는 같다 — 늘어난 것은 쪽뿐이다.
//!
//! ## 근인
//!
//! HWP3 파서는 저장 당시의 **자연 쪽 경계**(pgy 되돌아감·줄 `break_flag`)를
//! `ColumnBreakType::Page` 로 승격해 조판에 쓴다. 그건 사용자가 넣은 쪽나눔이 아니라서
//! 파서가 `page_break_synthesized` 로 표시해 두고, 주석도 "합성 표시를 남겨 저장 포맷
//! 방출에서 제외한다 — 명시 flags bit1 나눔만 실제 pageBreak 로 저장"이라고 적는다.
//!
//! HWPX 저장기는 그 계약을 지킨다(`serializer/hwpx/section.rs`). **HWP5 저장기만 그 표시를
//! 안 봤다** — `PARA_HEADER` 의 나누기 바이트에 0x04(쪽 나누기)를 그대로 썼고, 한글은 그걸
//! 강제 쪽나눔으로 읽어 다시 조판할 때마다 쪽을 하나씩 더 만든다.
//!
//! ## 기대값의 출처
//!
//! 한/글이 같은 HWP3 원본을 저장한 HWP5 변환본이다. `1480000-201400060` 의 문단 헤더를
//! 2,398개 맞대면 `nchars`·`control mask`·`range tag`·`line seg` 개수는 **전부 일치**하고
//! 나누기 바이트만 21문단에서 갈린다(한/글 0, 우리 4).
//!
//! ## 실측 (한글 2024 COM)
//!
//! ```text
//!   1480000-201400060   원본 40쪽    수정 전 50쪽  ->  수정 후 45쪽
//!   1170000-200500003   원본 264쪽   수정 전 327쪽 ->  수정 후 274쪽
//! ```
//!
//! 남은 차이는 이 축이 아니다 — HWP3 38건 전수 재스윕에서 회귀는 없었다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{ColumnBreakType, Paragraph};
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;

/// 글자 한 줄짜리 문단 하나를 담은 최소 문서.
fn doc_with(column_type: ColumnBreakType, synthesized: bool, raw_break: u8) -> Document {
    let mut para = Paragraph::default();
    para.text = "본문".to_string();
    para.char_offsets = vec![0, 1];
    para.column_type = column_type;
    para.page_break_synthesized = synthesized;
    para.raw_break_type = raw_break;
    let mut section = Section::default();
    section.paragraphs.push(para);
    let mut doc = Document::default();
    doc.sections.push(section);
    doc
}

/// 저장 바이트의 `PARA_HEADER` 나누기 바이트(offset 11)를 모은다.
fn saved_break_bytes(doc: &Document) -> Vec<u8> {
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(doc).expect("HWP5 직렬화 실패");
    let mut cfb = CfbReader::open(&bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let section = cfb
        .read_body_text_section(0, compressed, false)
        .expect("Section0 읽기");
    Record::read_all(&section)
        .expect("Section0 record 파싱")
        .into_iter()
        .filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER)
        .filter_map(|r| r.data.get(11).copied())
        .collect()
}

/// 합성 쪽나눔은 저장하지 않는다.
#[test]
fn synthesized_page_break_is_not_written() {
    let doc = doc_with(ColumnBreakType::Page, true, 0);
    assert!(
        saved_break_bytes(&doc).iter().all(|b| *b == 0x00),
        "합성 쪽나눔이 저장본에 실렸다 — 한글이 강제 쪽나눔으로 읽어 쪽이 불어난다: {:?}",
        saved_break_bytes(&doc)
    );
}

/// 반례 — 사용자가 넣은 쪽나눔은 그대로 저장한다.
#[test]
fn explicit_page_break_is_still_written() {
    let doc = doc_with(ColumnBreakType::Page, false, 0);
    assert!(
        saved_break_bytes(&doc).contains(&0x04),
        "명시 쪽나눔이 사라졌다: {:?}",
        saved_break_bytes(&doc)
    );
}

/// 반례 — 원본 바이트가 있으면(HWP5·HWPX 왕복) 그 값이 이긴다. 합성 표시와 무관하다.
#[test]
fn raw_break_type_wins_over_the_synthesized_mark() {
    let doc = doc_with(ColumnBreakType::Page, true, 0x04);
    assert!(
        saved_break_bytes(&doc).contains(&0x04),
        "원본 나누기 바이트가 보존되지 않았다: {:?}",
        saved_break_bytes(&doc)
    );
}

/// 반례 — 단 나눔 등 다른 갈래는 합성 표시의 영향을 받지 않는다.
#[test]
fn other_break_kinds_are_untouched() {
    let doc = doc_with(ColumnBreakType::Column, true, 0);
    assert!(
        saved_break_bytes(&doc).contains(&0x08),
        "단 나눔이 사라졌다: {:?}",
        saved_break_bytes(&doc)
    );
}
