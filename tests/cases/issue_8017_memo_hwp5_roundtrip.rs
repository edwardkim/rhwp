//! [#8017] HWP5 memo lists survive a save that rebuilds the section from the model.

use rhwp::model::control::{Control, Field};
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;

fn sample(rel: &str) -> Vec<u8> {
    let path = format!("{}/samples/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn collect_memos<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<&'a Field>) {
    for para in paragraphs {
        for ctrl in &para.controls {
            match ctrl {
                Control::Field(field) if field.command.starts_with("MEMO/") => out.push(field),
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect_memos(&cell.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

fn memos(doc: &Document) -> Vec<&Field> {
    let mut out = Vec::new();
    for section in &doc.sections {
        collect_memos(&section.paragraphs, &mut out);
    }
    out
}

/// Save the way an edited document is saved: every section from the model.
fn rebuild(doc: &Document) -> Document {
    let mut edited = doc.clone();
    edited.doc_info.raw_stream_dirty = true;
    for section in &mut edited.sections {
        section.raw_stream = None;
    }
    let bytes = serialize_document(&edited).expect("write HWP");
    parse_document(&bytes).expect("read the written HWP")
}

/// Hancom memo body paragraphs carry their own PARA_HEADER tail (instance id and,
/// from 5.0.3.2, the change-tracking word). The memo list writer replaced every
/// tail shorter than 12 bytes with zeros, so a Hancom 10-byte tail grew and lost
/// its instance id.
#[test]
fn memo_body_paragraph_header_tails_survive_a_rebuilt_save() {
    for rel in ["basic/NewYear_s_Day.hwp", "basic/english.hwp"] {
        let original = parse_document(&sample(rel)).expect("parse the sample");
        let before: Vec<Vec<Vec<u8>>> = memos(&original)
            .iter()
            .map(|field| {
                field
                    .memo_paragraphs
                    .iter()
                    .map(|para| para.raw_header_extra.clone())
                    .collect()
            })
            .collect();
        assert!(
            before.iter().any(|tails| !tails.is_empty()),
            "{rel} has memo bodies"
        );

        let reread = rebuild(&original);
        let after: Vec<Vec<Vec<u8>>> = memos(&reread)
            .iter()
            .map(|field| {
                field
                    .memo_paragraphs
                    .iter()
                    .map(|para| para.raw_header_extra.clone())
                    .collect()
            })
            .collect();
        assert_eq!(after, before, "{rel}: memo paragraph header tails");
    }
}

/// A memo body without a header tail (a new memo) is written with the zero tail.
#[test]
fn a_new_memo_body_gets_the_zero_header_tail() {
    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "anchor").unwrap();
    core.insert_memo_at(0, 0, 0, 6, "tester", "one\ntwo")
        .unwrap();
    let bytes = core.export_hwp_native().expect("write HWP");
    let reread = parse_document(&bytes).expect("read the written HWP");
    let found = memos(&reread);
    assert_eq!(found.len(), 1);
    let tails: Vec<&[u8]> = found[0]
        .memo_paragraphs
        .iter()
        .map(|para| para.raw_header_extra.as_slice())
        .collect();
    assert_eq!(tails.len(), 2);
    for tail in tails {
        assert_eq!(tail.len(), 12);
        assert!(tail[6..].iter().all(|byte| *byte == 0));
    }
}
