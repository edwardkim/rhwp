//! Empty HF paragraphs must obey the emitted HWP5 version, not a 5.0.3.0 sample.
//! Hancom PARA_HEADER: trailing merged-track UINT16 since 5.0.3.2.
//! https://tech.hancom.com/python-hwp-parsing-2/

use rhwp::model::control::Control;
use rhwp::model::document::{Document, HwpVersion, Section};
use rhwp::model::header_footer::{Footer, Header};
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;
use rhwp::serializer::serialize_hwp;
use rhwp::wasm_api::HwpDocument;

fn document(version: [u8; 4]) -> Document {
    let [major, minor, build, revision] = version;
    let mut doc = Document::default();
    doc.header.version = HwpVersion {
        major,
        minor,
        build,
        revision,
    };
    doc.sections.push(Section {
        paragraphs: vec![Paragraph {
            controls: vec![
                Control::Header(Box::new(Header {
                    paragraphs: vec![Paragraph::default()],
                    ..Default::default()
                })),
                Control::Footer(Box::new(Footer {
                    paragraphs: vec![Paragraph::default()],
                    ..Default::default()
                })),
            ],
            ..Default::default()
        }],
        ..Default::default()
    });
    doc
}

fn body_records(bytes: &[u8]) -> Vec<Record> {
    let mut cfb = CfbReader::open(bytes).expect("CFB");
    let header = cfb.read_stream_raw("/FileHeader").expect("FileHeader");
    let compressed = header[36] & 1 != 0;
    let body = cfb
        .read_body_text_section(0, compressed, false)
        .expect("Section0");
    Record::read_all(&body).expect("records")
}

#[test]
fn empty_header_footer_counts_the_implicit_paragraph_end() {
    let records = body_records(&serialize_hwp(&document([5, 1, 0, 1])).unwrap());
    let empty: Vec<_> = records
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER && r.level == 2)
        .collect();
    assert_eq!(empty.len(), 2, "both header and footer must be present");
    for p in empty {
        assert_eq!(
            u32::from_le_bytes(p.data[..4].try_into().unwrap()),
            0x80000001
        );
    }
    assert!(!records.iter().any(|r| r.level == 3
        && matches!(
            r.tag_id,
            tags::HWPTAG_PARA_TEXT | tags::HWPTAG_PARA_LINE_SEG
        )));
    let lists: Vec<_> = records
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_LIST_HEADER)
        .collect();
    assert_eq!(lists.len(), 2);
    assert!(lists.iter().all(|r| r.data[..2] == [1, 0]));
}

#[test]
fn paragraph_header_length_changes_at_5032() {
    for (version, expected) in [
        ([5, 0, 3, 0], 22),
        ([5, 0, 3, 1], 22),
        ([5, 0, 3, 2], 24),
        ([5, 1, 0, 1], 24),
    ] {
        let records = body_records(&serialize_hwp(&document(version)).unwrap());
        let headers: Vec<_> = records
            .iter()
            .filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER)
            .collect();
        assert_eq!(headers.len(), 3);
        for h in headers {
            assert_eq!(
                h.data.len(),
                expected,
                "version={version:?}, level={}",
                h.level
            );
        }
    }
}

#[test]
fn preserved_file_header_version_controls_new_paragraphs() {
    for (model, emitted, expected) in [
        ([5, 0, 3, 0], [5, 1, 0, 1], 24),
        ([5, 1, 0, 1], [5, 0, 3, 0], 22),
    ] {
        let mut doc = document(model);
        let mut raw = rhwp::serializer::header::serialize_file_header(&doc.header);
        raw[32..36].copy_from_slice(&[emitted[3], emitted[2], emitted[1], emitted[0]]);
        doc.header.raw_data = Some(raw);
        let records = body_records(&serialize_hwp(&doc).unwrap());
        assert!(
            records
                .iter()
                .filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER)
                .all(|r| r.data.len() == expected),
            "emitted={emitted:?}"
        );
    }
}

#[test]
fn existing_instance_and_change_tracking_bytes_survive() {
    let mut doc = document([5, 1, 0, 1]);
    let tail = [0x12, 0x34, 0x56, 0x78, 0x01, 0x00];
    let mut extra = vec![0; 6];
    extra.extend_from_slice(&tail);
    doc.sections[0].paragraphs[0].raw_header_extra = extra;
    let records = body_records(&serialize_hwp(&doc).unwrap());
    assert_eq!(&records[0].data[18..], &tail);
}

#[test]
fn newly_created_header_and_footer_save_without_stored_lines() {
    let mut core = HwpDocument::create_empty();
    core.create_blank_document_native().unwrap();
    for is_header in [true, false] {
        for apply in 0..3 {
            core.create_header_footer_native(0, is_header, apply)
                .unwrap();
        }
    }
    let mut checked = 0;
    for ctrl in &core.document().sections[0].paragraphs[0].controls {
        let paras = match ctrl {
            Control::Header(h) => &h.paragraphs,
            Control::Footer(f) => &f.paragraphs,
            _ => continue,
        };
        checked += 1;
        assert_eq!(paras.len(), 1);
        assert!(paras[0].line_segs.is_empty());
        assert!(!paras[0].has_para_text);
    }
    assert_eq!(checked, 6);
    let records = body_records(&serialize_hwp(core.document()).unwrap());
    let headers: Vec<_> = records
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_PARA_HEADER && r.level == 2)
        .collect();
    assert_eq!(headers.len(), 6);
    for header in headers {
        assert_eq!(
            u32::from_le_bytes(header.data[..4].try_into().unwrap()),
            0x80000001
        );
    }
    assert!(!records
        .iter()
        .any(|r| r.level == 3 && r.tag_id == tags::HWPTAG_PARA_LINE_SEG));
}
