//! 저장 계약 검사(조판·렌더링 비대상) — HWPX→HWP 저장의 개체 공통 attr bit 28 과 변경 추적 설정.
//!
//! 측정일 2026-10-30, 한글 2024(13.0.0.564) COM 으로 한컴 HWPX 를 열어 HWP 로 저장한 레코드를
//! 정본으로 삼았다. 이 비트와 TRACKCHANGE 값은 rhwp 조판이 읽지 않으므로 Visual Sweep 대상이
//! 아니다(시각 수치 없음).
//!
//! - bit 28 은 번호 범주(bit 26-27)가 아니다. `<hh:trackchageConfig flags>` 에 0x08 이 있으면
//!   모든 개체(표·그림·도형·수식)에 켜지고, 없으면 바탕쪽·머리말·꼬리말 안의 개체만 켜진다.
//!   한컴 HWPX 176문서 중 flags 에 0x08 이 있는(요소 없음 = 56 포함) 175문서의 개체 26,644개가
//!   전부 1, flags=0 인 1문서(issue1510)의 본문 표 3개가 전부 0.
//! - exam_social.hwpx 의 flags 만 0 으로 바꾼 변형: 바탕쪽·머리말 개체 10개(표 5·도형 5)만 1,
//!   본문 개체 71개(표 62·도형 9)는 0. 원래 flags=56 이면 81개 전부 1.
//! - eq-002.hwpx: flags=56 이면 수식 16개 전부 1, flags=0 이면 전부 0.
//! - 한글은 HWPX 의 flags 를 HWP `HWPTAG_TRACKCHANGE` 첫 `u32` 에 그대로 옮긴다
//!   (0·1·2·4·8·16·24·32·40·48·56·64 변형 전부).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::tags;

const EXAM_SOCIAL: &str = "samples/hwpx/exam_social.hwpx";
const EQ_002: &str = "samples/hwpx/eq-002.hwpx";
const ISSUE1510_FLAGS_ZERO: &str = "samples/issue1510_coanchored_float_tables.hwpx";

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// HWPX 의 `<hh:trackchageConfig flags>` 만 바꾼 사본 — 한글 변형 실측과 같은 조작.
fn with_track_change_flags(hwpx: &[u8], flags: u32) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(hwpx)).expect("open hwpx");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut replaced = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("hwpx entry");
        let name = entry.name().expect("entry name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/header.xml" {
            let xml = String::from_utf8(data).expect("header utf8");
            let start = xml
                .find("<hh:trackchageConfig flags=\"")
                .expect("trackchageConfig")
                + "<hh:trackchageConfig flags=\"".len();
            let end = start + xml[start..].find('"').expect("flags end");
            data = format!("{}{}{}", &xml[..start], flags, &xml[end..]).into_bytes();
            replaced += 1;
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        out.start_file(name, options).expect("start entry");
        out.write_all(&data).expect("write entry");
    }
    assert_eq!(replaced, 1);
    out.finish().expect("finish hwpx").into_inner()
}

fn export_hwp(hwpx: &[u8]) -> Document {
    let mut core = DocumentCore::from_bytes(hwpx).expect("parse hwpx");
    let hwp = core.export_hwp_with_adapter().expect("export HWP");
    rhwp::parse_document(&hwp).expect("reparse exported HWP")
}

fn track_change_record(doc: &Document) -> Option<u32> {
    doc.doc_info
        .extra_records
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_TRACKCHANGE)
        .map(|r| u32::from_le_bytes(r.data[..4].try_into().unwrap()))
}

/// CTRL_HEADER 를 가진 개체의 (바탕쪽·머리말·꼬리말 안인가, bit 28) 목록.
fn object_bits(doc: &Document) -> Vec<(bool, bool)> {
    fn paragraphs(paras: &[Paragraph], frame: bool, out: &mut Vec<(bool, bool)>) {
        for para in paras {
            for ctrl in &para.controls {
                control(ctrl, frame, out);
            }
        }
    }
    fn shape(shape: &ShapeObject, frame: bool, out: &mut Vec<(bool, bool)>) {
        out.push((frame, shape.common().attr & (1 << 28) != 0));
        if let Some(drawing) = shape.drawing() {
            if let Some(text_box) = &drawing.text_box {
                paragraphs(&text_box.paragraphs, frame, out);
            }
        }
    }
    fn control(ctrl: &Control, frame: bool, out: &mut Vec<(bool, bool)>) {
        match ctrl {
            Control::Table(t) => {
                out.push((frame, t.common.attr & (1 << 28) != 0));
                for cell in &t.cells {
                    paragraphs(&cell.paragraphs, frame, out);
                }
            }
            Control::Picture(p) => out.push((frame, p.common.attr & (1 << 28) != 0)),
            Control::Shape(s) => shape(s, frame, out),
            Control::Equation(e) => out.push((frame, e.common.attr & (1 << 28) != 0)),
            Control::Header(h) => paragraphs(&h.paragraphs, true, out),
            Control::Footer(f) => paragraphs(&f.paragraphs, true, out),
            Control::Footnote(n) => paragraphs(&n.paragraphs, frame, out),
            Control::Endnote(n) => paragraphs(&n.paragraphs, frame, out),
            Control::SectionDef(sd) => {
                for mp in &sd.master_pages {
                    paragraphs(&mp.paragraphs, true, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for section in &doc.sections {
        paragraphs(&section.paragraphs, false, &mut out);
    }
    out
}

/// (바탕쪽·머리말 개체 수, 그중 bit 28 수, 본문 개체 수, 그중 bit 28 수)
fn tally(doc: &Document) -> (usize, usize, usize, usize) {
    let bits = object_bits(doc);
    let frame: Vec<_> = bits.iter().filter(|(f, _)| *f).collect();
    let body: Vec<_> = bits.iter().filter(|(f, _)| !*f).collect();
    (
        frame.len(),
        frame.iter().filter(|(_, b)| *b).count(),
        body.len(),
        body.iter().filter(|(_, b)| *b).count(),
    )
}

#[test]
fn track_change_flag_0x08_stamps_every_object() {
    let doc = export_hwp(&read(EXAM_SOCIAL));
    assert_eq!(track_change_record(&doc), Some(56));
    // 한글 2024: 표 67·도형 14, 81개 전부 bit 28.
    assert_eq!(tally(&doc), (10, 10, 71, 71));
}

#[test]
fn without_0x08_only_page_frame_objects_are_stamped() {
    let doc = export_hwp(&with_track_change_flags(&read(EXAM_SOCIAL), 0));
    assert_eq!(track_change_record(&doc), Some(0), "flags 를 그대로 옮긴다");
    // 한글 2024: 바탕쪽·머리말 표 5·도형 5 만 bit 28, 본문 표 62·도형 9 는 0.
    assert_eq!(tally(&doc), (10, 10, 71, 0));
}

#[test]
fn other_track_change_bits_do_not_stamp_body_objects() {
    for (flags, stamped) in [(16, false), (48, false), (8, true), (24, true)] {
        let doc = export_hwp(&with_track_change_flags(&read(EXAM_SOCIAL), flags));
        assert_eq!(track_change_record(&doc), Some(flags));
        let (_, _, body, body_bits) = tally(&doc);
        assert_eq!(body_bits, if stamped { body } else { 0 }, "flags={flags}");
    }
}

#[test]
fn equations_follow_the_same_rule() {
    let doc = export_hwp(&read(EQ_002));
    assert_eq!(tally(&doc), (0, 0, 16, 16));
    let doc = export_hwp(&with_track_change_flags(&read(EQ_002), 0));
    assert_eq!(tally(&doc), (0, 0, 16, 0));
}

#[test]
fn flags_zero_document_keeps_body_tables_unstamped() {
    let doc = export_hwp(&read(ISSUE1510_FLAGS_ZERO));
    assert_eq!(track_change_record(&doc), Some(0));
    // 한글 2024 저장본: 본문 표 3개 전부 bit 28=0.
    assert_eq!(tally(&doc), (0, 0, 3, 0));
}
