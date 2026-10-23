//! 캡션 자동 번호는 자기 자리표(공백 한 글자) 자리에 그린다.
//!
//! 종전에는 캡션 글에서 공백 두 개를 찾아 그 사이에 번호를 끼웠다. `<그림 [번호]>` 는
//! 자리표가 그대로 남아 `<그림 1 >` 로 그려졌고, `<표 Ⅱ-[번호]>` 처럼 번호 앞뒤에 공백이
//! 없으면 번호가 아예 빠졌다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 표 캡션이 `<그림 [자동 번호]> 의정활동 …` 인 한컴 저장본.
const SAMPLE: &[u8] = include_bytes!("../../samples/hwpx/ta-pic-001-r.hwpx");

fn page_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        page_text(child, out);
    }
}

fn caption_line(section_edit: impl Fn(&str) -> String) -> String {
    let mut input = zip::ZipArchive::new(Cursor::new(SAMPLE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let name = entry.name().expect("name").into_owned();
        if name != "Contents/section0.xml" {
            output.raw_copy_file(entry).unwrap();
            continue;
        }
        let mut xml = String::new();
        entry.read_to_string(&mut xml).unwrap();
        output
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        output.write_all(section_edit(&xml).as_bytes()).unwrap();
    }
    let doc = DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap();
    let mut text = String::new();
    page_text(&doc.build_page_render_tree(0).unwrap().root, &mut text);
    let start = text.find('<').expect("caption text");
    let end = start + text[start..].find('>').expect("caption close");
    text[start..=end].to_string()
}

#[test]
fn caption_number_replaces_its_placeholder() {
    assert_eq!(caption_line(|xml| xml.to_string()), "<그림 1>");
}

#[test]
fn caption_number_without_surrounding_spaces_is_drawn() {
    let tight = |xml: &str| xml.replace("&lt;그림 </hp:t>", "&lt;표 Ⅱ-</hp:t>");
    assert_eq!(caption_line(tight), "<표 Ⅱ-1>");
}
