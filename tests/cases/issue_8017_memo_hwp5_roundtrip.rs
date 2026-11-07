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

/// A memo document must not grow by one empty paragraph per HWP save. Hancom puts
/// MEMO_LIST right after the real last body paragraph.
#[test]
fn memo_tail_resave_keeps_the_paragraph_count() {
    use rhwp::parser::body_text::parse_body_text_section;
    use rhwp::parser::record::Record;
    use rhwp::parser::tags;
    use rhwp::serializer::body_text::serialize_section;

    let doc = parse_document(&sample("issue5866/memo_field_hwp5.hwp"))
        .expect("parse the Hancom memo sample");
    let mut section = doc
        .sections
        .into_iter()
        .last()
        .expect("the sample has a section");
    let original = section.paragraphs.len();
    for _ in 0..2 {
        section.raw_stream = None;
        section.raw_provenance = None;
        let bytes = serialize_section(&section);
        let records = Record::read_all(&bytes).unwrap();
        let last_body = records
            .iter()
            .rposition(|r| r.tag_id == tags::HWPTAG_PARA_HEADER && r.level == 0)
            .unwrap();
        let char_count = u32::from_le_bytes(records[last_body].data[0..4].try_into().unwrap());
        assert_ne!(
            char_count & 0x8000_0000,
            0,
            "the real last paragraph keeps its last bit"
        );
        assert!(
            records[last_body + 1..]
                .iter()
                .any(|r| r.tag_id == tags::HWPTAG_MEMO_LIST && r.level == 1),
            "the memo list follows the last body paragraph"
        );
        section = parse_body_text_section(&bytes).unwrap();
        assert_eq!(
            section.paragraphs.len(),
            original,
            "no root paragraph is added"
        );
        let mut found = Vec::new();
        collect_memos(&section.paragraphs, &mut found);
        assert_eq!(
            found
                .iter()
                .filter(|field| !field.memo_paragraphs.is_empty())
                .count(),
            1,
            "the memo body is read back"
        );
    }
}
