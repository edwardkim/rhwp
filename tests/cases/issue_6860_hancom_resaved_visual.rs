//! 한컴 재저장 비교본의 쪽 경계와 문단 간 저장 reset을 검증한다.
//! 기존 절단 fixture는 원 저장 좌표의 결함 재현용으로 별도 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6860/3067979-road-lighting-photometric-hancom2020.hwpx";
const FIRST_LINE: &str = "Tilt=<filename> 또는 INCLUDE 또는 NONE";

fn sample() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("한컴 재저장 정식 fixture")
}

fn text(node: &RenderNode, output: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        output.push_str(&run.text);
    }
    for child in &node.children {
        text(child, output);
    }
}

fn page_text(doc: &HwpDocument, page: u32) -> String {
    let mut output = String::new();
    text(
        &doc.build_page_render_tree(page).expect("render tree").root,
        &mut output,
    );
    output
}

#[test]
fn saved_cross_paragraph_reset_keeps_the_first_line_on_page_one() {
    let doc = HwpDocument::from_bytes(&sample()).expect("문서 로드");
    assert_eq!(doc.page_count(), 2, "동일 재저장본의 한컴 PDF는 두 쪽이다");
    assert!(
        page_text(&doc, 0).contains(FIRST_LINE),
        "첫 줄은 한컴 PDF처럼 1쪽에 남아야 한다"
    );
    assert!(
        !page_text(&doc, 1).contains(FIRST_LINE),
        "이어지는 쪽에 첫 줄을 중복하지 않는다"
    );
}

#[test]
fn nonzero_paragraph_starts_do_not_claim_a_zero_reset_boundary() {
    let mut archive = zip::ZipArchive::new(Cursor::new(sample())).expect("fixture ZIP");
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("ZIP entry");
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            writer
                .add_directory(entry.name(), options)
                .expect("directory");
            continue;
        }
        writer.start_file(entry.name(), options).expect("entry");
        if entry.name() == "Contents/section0.xml" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).expect("section XML");
            assert!(xml.contains("vertpos=\"0\""));
            // 높이/문자/표 구조는 보존하고 0 reset의 증거만 제거한다.
            writer
                .write_all(xml.replace("vertpos=\"0\"", "vertpos=\"1\"").as_bytes())
                .expect("modified section");
        } else {
            std::io::copy(&mut entry, &mut writer).expect("copy ZIP entry");
        }
    }
    let bytes = writer.finish().expect("ZIP finish").into_inner();
    let doc = HwpDocument::from_bytes(&bytes).expect("대조군 로드");
    assert!(
        !page_text(&doc, 0).contains(FIRST_LINE),
        "저장 reset 없는 행은 기존 고아 줄 기준을 유지한다"
    );
    assert!(
        page_text(&doc, 1).contains(FIRST_LINE),
        "대조군의 첫 줄은 소실하지 않고 이월한다"
    );
}
