//! 번호 넣기 바로 앞에서 문단을 나눠도 번호 자리표 공백이 한 번만 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::io::{Read, Write};

/// 공개 번호 삽입 API의 HWPX에 번호 뒤 글을 이어 붙인다.
fn numbered() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞").unwrap();
    core.insert_auto_number_at_cursor(0, 0, 17, "page").unwrap();
    let source = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(std::io::Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut entry = input.by_index(index).unwrap();
        let name = entry.name().unwrap().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            let marker = "</hp:autoNum></hp:ctrl></hp:run>";
            assert!(xml.contains(marker));
            bytes = xml
                .replacen(
                    marker,
                    "</hp:autoNum></hp:ctrl><hp:t>뒤끝</hp:t></hp:run>",
                    1,
                )
                .into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        output.start_file(name, options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap()
}

#[test]
fn splitting_before_auto_number_keeps_one_placeholder_slot() {
    let mut core = numbered();
    assert_eq!(core.document().sections[0].paragraphs[0].text, "앞 뒤끝");
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    let tail = &core.document().sections[0].paragraphs[1];
    assert_eq!(tail.text, " 뒤끝");
    // 자리표 공백이 번호의 8유닛을 담는다. 앞에 번호 갭을 따로 두지 않는다.
    assert_eq!(tail.char_offsets, vec![0, 8, 9]);
    assert_eq!(tail.char_count, 11);
    assert_eq!(tail.control_text_positions(), vec![0]);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let tail = &reopened.document().sections[0].paragraphs[1];
        assert_eq!(tail.text, " 뒤끝");
        assert_eq!(tail.control_text_positions(), vec![0]);
    }
}
