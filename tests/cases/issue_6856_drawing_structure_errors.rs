//! Deliberately incomplete parser inputs, not valid Hancom samples.
use rhwp::model::shape::{GroupShape, RectangleShape, ShapeObject, TextBox};
use rhwp::model::{control::Control, document::Section, paragraph::Paragraph};
use rhwp::parser::{
    body_text::parse_body_text_section, hwpx::section::parse_hwpx_section, record::Record, tags,
};
use rhwp::parser::{body_text::BodyTextError, hwpx::HwpxError, ParseError};
use rhwp::serializer::{body_text::serialize_section, record_writer::write_records};
use std::io::{Cursor, Read, Write};

fn damaged_hwp_section(nested: bool, missing_paragraph: bool) -> Vec<u8> {
    let mut rect = RectangleShape::default();
    rect.drawing.text_box = Some(TextBox {
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    });
    let shape = if nested {
        ShapeObject::Group(GroupShape {
            children: vec![ShapeObject::Rectangle(rect)],
            ..Default::default()
        })
    } else {
        ShapeObject::Rectangle(rect)
    };
    let section = Section {
        paragraphs: vec![Paragraph {
            controls: vec![Control::Shape(Box::new(shape))],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut records = Record::read_all(&serialize_section(&section)).unwrap();
    let list = records
        .iter()
        .position(|r| r.tag_id == tags::HWPTAG_LIST_HEADER)
        .unwrap();
    if missing_paragraph {
        records.truncate(list + 1);
    } else {
        records[list].data.clear();
    }
    write_records(&records)
}

#[test]
fn hwp_owned_missing_structure_is_an_error_even_in_groups() {
    for nested in [false, true] {
        for missing in [false, true] {
            assert!(
                matches!(
                    parse_body_text_section(&damaged_hwp_section(nested, missing)),
                    Err(BodyTextError::DrawingTextStructure(_))
                ),
                "nested={nested} missing={missing}"
            );
        }
    }
}

fn hwp_container(section: &[u8]) -> Vec<u8> {
    let template = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/hwpx/hancom-hwp/blank_hwpx.hwp"),
    )
    .unwrap();
    let mut cfb = cfb::CompoundFile::open(Cursor::new(template)).unwrap();
    let mut header = Vec::new();
    cfb.open_stream("/FileHeader")
        .unwrap()
        .read_to_end(&mut header)
        .unwrap();
    let payload = if header[36] & 1 != 0 {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(section).unwrap();
        encoder.finish().unwrap()
    } else {
        section.to_vec()
    };
    cfb.create_stream("/BodyText/Section0")
        .unwrap()
        .write_all(&payload)
        .unwrap();
    cfb.into_inner().into_inner()
}

// Same unused FAT-tail corruption strategy as the existing parser budget tests:
// standard CFB rejects duplicate pointees, lenient CFB can still read real chains.
fn force_lenient(mut bytes: Vec<u8>) -> Vec<u8> {
    let sector_size = 1usize << u16::from_le_bytes(bytes[30..32].try_into().unwrap());
    let physical = (bytes.len() - 512) / sector_size;
    let offset = |index: usize| {
        let owner = index / (sector_size / 4);
        let header_offset = 76 + owner * 4;
        assert!(header_offset + 4 <= 512);
        let sector = u32::from_le_bytes(bytes[header_offset..header_offset + 4].try_into().unwrap())
            as usize;
        512 + sector * sector_size + (index % (sector_size / 4)) * 4
    };
    let live = (0..physical)
        .map(|i| u32::from_le_bytes(bytes[offset(i)..offset(i) + 4].try_into().unwrap()))
        .find(|value| (*value as usize) < physical)
        .unwrap();
    let padding = offset(physical);
    bytes[padding..padding + 4].copy_from_slice(&live.to_le_bytes());
    assert!(rhwp::parser::cfb_reader::CfbReader::open(&bytes).is_err());
    assert!(rhwp::parser::cfb_reader::LenientCfbReader::open(&bytes).is_ok());
    bytes
}

#[test]
fn hwp_lenient_container_still_propagates_owned_structure_errors() {
    let normal = force_lenient(hwp_container(&serialize_section(&Section::default())));
    assert!(rhwp::parser::parse_hwp(&normal).is_ok());
    let damaged = force_lenient(hwp_container(&damaged_hwp_section(true, false)));
    assert!(matches!(
        rhwp::parser::parse_hwp(&damaged),
        Err(ParseError::BodyTextError(
            BodyTextError::DrawingTextStructure(_)
        ))
    ));
}

#[test]
fn hwpx_nested_and_mismatched_owned_areas_propagate_structure_errors() {
    for suffix in ["<hp:p><hp:run><hp:rect><hp:drawText>", "</hp:wrong>"] {
        let xml = format!("{}{suffix}", truncated_hwpx());
        assert!(matches!(
            parse_hwpx_section(&xml),
            Err(HwpxError::DrawingTextStructure(_))
        ));
    }
}

#[test]
fn hwp_document_open_must_not_replace_damaged_owned_area_with_empty_section() {
    for nested in [false, true] {
        let normal = hwp_container(&serialize_section(&Section::default()));
        assert!(
            rhwp::parser::parse_hwp(&normal).is_ok(),
            "container control"
        );
        let bytes = hwp_container(&damaged_hwp_section(nested, true));
        assert!(
            matches!(
                rhwp::parser::parse_hwp(&bytes),
                Err(ParseError::BodyTextError(
                    BodyTextError::DrawingTextStructure(_)
                ))
            ),
            "owned error swallowed"
        );
        assert!(matches!(
            rhwp::parser::parse_document(&bytes),
            Err(ParseError::BodyTextError(
                BodyTextError::DrawingTextStructure(_)
            ))
        ));
    }
}

fn truncated_hwpx() -> &'static str {
    r#"<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"><hp:p><hp:run><hp:rect><hp:drawText><hp:subList>"#
}

#[test]
fn hwpx_unclosed_owned_area_is_an_error() {
    assert!(matches!(
        parse_hwpx_section(truncated_hwpx()),
        Err(HwpxError::DrawingTextStructure(_))
    ));
}

#[test]
fn hwpx_depth_limit_is_not_reclassified_as_xml_structure_damage() {
    let nested = "<hp:p><hp:run><hp:rect><hp:drawText><hp:subList>".repeat(32);
    let xml = format!("{}{nested}", truncated_hwpx());
    assert!(matches!(
        parse_hwpx_section(&xml),
        Err(HwpxError::XmlError(_))
    ));
}

#[test]
fn hwpx_master_page_owned_structure_error_reaches_document_caller() {
    check_hwpx_package(
        "samples/2025 행정업무운영 편람(최종).hwpx",
        "Contents/masterpage0.xml",
        &truncated_hwpx().replace("hs:sec", "hs:masterPage"),
    );
}

#[test]
fn hwpx_document_open_must_not_replace_damaged_owned_area_with_empty_section() {
    check_hwpx_package(
        "samples/hwpx/156160455-social-pig-farm-income.hwpx",
        "Contents/section0.xml",
        truncated_hwpx(),
    );
}

fn check_hwpx_package(path: &str, entry: &str, replacement: &str) {
    let source =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    assert!(
        rhwp::parser::parse_document(&source).is_ok(),
        "original package control"
    );
    let mut reader = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut changed = false;
    for i in 0..reader.len() {
        let mut file = reader.by_index(i).unwrap();
        let name = file.name().to_owned();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if name == entry {
            data = replacement.as_bytes().to_vec();
            changed = true;
        }
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&data).unwrap();
    }
    assert!(changed);
    let bytes = writer.finish().unwrap().into_inner();
    assert!(
        matches!(
            rhwp::parser::parse_document(&bytes),
            Err(ParseError::HwpxError(HwpxError::DrawingTextStructure(_)))
        ),
        "owned error swallowed"
    );
}
