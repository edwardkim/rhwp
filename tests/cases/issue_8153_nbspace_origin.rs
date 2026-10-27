//! #8153: 묶음 빈칸의 제어·리터럴 혼용 표기를 보존한다.
//!
//! 이 검사는 합성 HWPX의 저장 표기와 편집 문자 소유를 확인한다. 한컴 출력과의
//! 시각 일치율을 주장하는 검사가 아니며, 원본 편람의 렌더링 회귀를 대체하지 않는다.

use std::io::{Cursor, Read, Write};

use rhwp::model::paragraph::Paragraph;
use rhwp::DocumentCore;

fn mixed_source() -> Vec<u8> {
    let core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
    let original = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(original)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut replaced = false;
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let name = file.name().unwrap().to_string();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name == "Contents/section0.xml" {
            let text = String::from_utf8(bytes).unwrap();
            let marker = "<hp:t></hp:t>";
            assert_eq!(text.matches(marker).count(), 1);
            let mixed = "<hp:t>가<hp:nbSpace/>\u{00a0}<hp:nbSpace/>나</hp:t>";
            bytes = text.replacen(marker, mixed, 1).into_bytes();
            replaced = true;
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(if name == "mimetype" {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            });
        output.start_file(name, options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert!(replaced);
    output.finish().unwrap().into_inner()
}

fn representation_order(hwpx: &[u8]) -> Vec<bool> {
    let mut input = zip::ZipArchive::new(Cursor::new(hwpx)).unwrap();
    let mut file = input.by_name("Contents/section0.xml").unwrap();
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    let mut found: Vec<(usize, bool)> = xml
        .match_indices("<hp:nbSpace")
        .map(|(index, _)| (index, true))
        .chain(
            xml.match_indices('\u{00a0}')
                .map(|(index, _)| (index, false)),
        )
        .collect();
    found.sort_unstable_by_key(|entry| entry.0);
    found.into_iter().map(|entry| entry.1).collect()
}

#[test]
fn mixed_origin_survives_hwpx_and_hwp_roundtrips_in_order() {
    let source = mixed_source();
    assert_eq!(representation_order(&source), [true, false, true]);
    let core = DocumentCore::from_bytes(&source).unwrap();
    let paragraph = &core.document().sections[0].paragraphs[0];
    assert_eq!(
        paragraph.nb_space_control_indices.as_deref(),
        Some(&[1, 3][..])
    );
    let hwpx = core.export_hwpx_native().unwrap();
    assert_eq!(representation_order(&hwpx), [true, false, true]);
    let hwp = core.export_hwp_with_adapter().unwrap();
    let reparsed = DocumentCore::from_bytes(&hwp).unwrap();
    let paragraph = &reparsed.document().sections[0].paragraphs[0];
    assert_eq!(
        paragraph.nb_space_control_indices.as_deref(),
        Some(&[1, 3][..])
    );
    assert_eq!(
        representation_order(&reparsed.export_hwpx_native().unwrap()),
        [true, false, true]
    );
}

#[test]
fn mixed_origin_follows_insert_delete_split_and_merge() {
    let source = mixed_source();
    let core = DocumentCore::from_bytes(&source).unwrap();
    let mut p = core.document().sections[0].paragraphs[0].clone();
    p.insert_text_at(2, "😀\u{00a0}");
    assert_eq!(p.nb_space_control_indices.as_deref(), Some(&[1, 5][..]));
    assert!(!p.nb_space_is_control_at(3));
    p.delete_text_at(1, 2);
    assert_eq!(p.nb_space_control_indices.as_deref(), Some(&[3][..]));
    let tail = p.split_at(2);
    assert_eq!(p.nb_space_control_indices.as_deref(), Some(&[][..]));
    assert_eq!(p.control_mask & (1 << 30), 0);
    assert_eq!(tail.nb_space_control_indices.as_deref(), Some(&[1][..]));
    p.merge_from(&tail);
    assert_eq!(p.nb_space_control_indices.as_deref(), Some(&[3][..]));
    assert_ne!(p.control_mask & (1 << 30), 0);
}

#[test]
fn legacy_origin_without_character_data_survives_until_edit() {
    let mut p = Paragraph {
        text: "가\u{00a0}\u{00a0}\u{00a0}나".into(),
        char_offsets: (0..5).collect(),
        char_count: 6,
        control_mask: 1 << 30,
        has_para_text: true,
        ..Paragraph::default()
    };
    assert!(p.nb_space_is_control_at(1));
    assert!(p.nb_space_is_control_at(2));
    p.insert_text_at(2, "\u{00a0}");
    assert_eq!(p.nb_space_control_indices.as_deref(), Some(&[1, 3, 4][..]));
    assert!(!p.nb_space_is_control_at(2));
}

#[test]
fn autonumber_placeholder_does_not_shift_following_space_origins() {
    use rhwp::model::control::{AutoNumber, Control};
    use rhwp::model::document::{Document, Section};
    use rhwp::serializer::hwpx::{context::SerializeContext, section::write_section};

    let paragraph = Paragraph {
        text: "가 \u{00a0}\u{00a0}\u{00a0}나".into(),
        char_offsets: vec![0, 1, 9, 10, 11, 12],
        char_count: 14,
        has_para_text: true,
        controls: vec![Control::AutoNumber(AutoNumber::default())],
        control_mask: (1 << 30) | (1 << 18),
        nb_space_control_indices: Some(vec![2, 4]),
        ..Paragraph::default()
    };
    let section = Section {
        paragraphs: vec![paragraph],
        ..Section::default()
    };
    let document = Document {
        sections: vec![section.clone()],
        ..Document::default()
    };
    let mut context = SerializeContext::collect_from_document(&document);
    let output = write_section(&section, &document, 0, &mut context).unwrap();
    let xml = std::str::from_utf8(&output).unwrap();
    let mut found: Vec<(usize, bool)> = xml
        .match_indices("<hp:nbSpace")
        .map(|(index, _)| (index, true))
        .chain(
            xml.match_indices('\u{00a0}')
                .map(|(index, _)| (index, false)),
        )
        .collect();
    found.sort_unstable_by_key(|entry| entry.0);
    assert_eq!(
        found.iter().map(|entry| entry.1).collect::<Vec<_>>(),
        [true, false, true]
    );
    assert!(xml.contains("autoNum"));
}

#[test]
fn copy_and_paste_preserve_each_space_representation() {
    let mut core = DocumentCore::from_bytes(&mixed_source()).unwrap();
    core.copy_selection_native(0, 0, 1, 0, 4).unwrap();
    core.paste_internal_native(0, 0, 5).unwrap();
    let paragraph = &core.document().sections[0].paragraphs[0];
    assert_eq!(
        paragraph.text,
        "가\u{00a0}\u{00a0}\u{00a0}나\u{00a0}\u{00a0}\u{00a0}"
    );
    assert_eq!(
        paragraph.nb_space_control_indices.as_deref(),
        Some(&[1, 3, 5, 7][..])
    );
    assert_eq!(
        representation_order(&core.export_hwpx_native().unwrap()),
        [true, false, true, true, false, true]
    );
}
