//! 구역 첫 문단에서 누름틀·책갈피 뒤 글자처럼 취급 표의 줄 배치.
//!
//! "첫문단"에 "가나·다[표]라" 문단을 Backspace로 합친다(`·`는 누름틀, 책갈피 또는 없음). 표는
//! 본문 폭을 다 차지하므로 앞 글자 줄에도, 뒤 글자 옆에도 들어가지 않는다. 한컴처럼 앞 글자,
//! 표, 뒤 글자가 세 줄에 놓이고, 저장 줄은 표 자리와 표 뒤 글자에서 시작해야 한다.
//! HWP·HWPX로 저장해 다시 열어도 같아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

/// HWPX 본문의 `X` 하나를 책갈피로 바꾼다. 파서가 만든 책갈피를 입력으로 쓴다.
fn with_bookmark(hwpx: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(hwpx)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches('X').count(), 1);
            bytes = xml
                .replace(
                    'X',
                    "</hp:t><hp:ctrl><hp:bookmark name=\"책갈피\"/></hp:ctrl><hp:t>",
                )
                .into_bytes();
        }
        output
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

/// 표 앞 '다' 바로 앞에 두는 컨트롤.
#[derive(Clone, Copy, PartialEq)]
enum Mark {
    Plain,
    Field,
    Bookmark,
}

/// HWPX로 저장해 다시 연 세 문단 문서에서 둘째 문단을 첫 문단에 합친다.
fn merged(mark: Mark) -> HwpDocument {
    let text = if mark == Mark::Bookmark {
        "가나X다라"
    } else {
        "가나다라"
    };
    let len = text.chars().count();
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "첫문단").unwrap();
    doc.split_paragraph_native(0, 0, 3, None).unwrap();
    doc.insert_text_native(0, 1, 0, text).unwrap();
    doc.split_paragraph_native(0, 1, len, None).unwrap();
    doc.insert_text_native(0, 2, 0, "끝문단").unwrap();
    doc.create_table_ex_native(0, 1, len - 1, 1, 2, true, None, None)
        .unwrap();
    if mark == Mark::Field {
        doc.insert_click_here_field_at(0, 1, 2, "안내", "", "f1", true)
            .unwrap();
    }
    let hwpx = doc.export_hwpx_native().unwrap();
    let hwpx = if mark == Mark::Bookmark {
        with_bookmark(&hwpx)
    } else {
        hwpx
    };
    let mut doc = HwpDocument::from_bytes(&hwpx).unwrap();
    doc.merge_paragraph_native(0, 1).unwrap();
    doc
}

fn table_box(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(0) && table.cell_context.is_none())
    {
        return Some(node.bbox);
    }
    node.children.iter().find_map(table_box)
}

/// 첫 문단의 `char_idx`번 글자를 그린 런의 상자.
fn char_box(node: &RenderNode, char_idx: usize) -> Option<BoundingBox> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(0) && run.cell_context.is_none() {
            if let Some(start) = run.char_start {
                if (start..start + run.text.chars().count()).contains(&char_idx) {
                    return Some(node.bbox);
                }
            }
        }
    }
    node.children
        .iter()
        .find_map(|child| char_box(child, char_idx))
}

fn assert_rows(doc: &HwpDocument, label: &str) {
    let para = &doc.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "첫문단가나다라", "{label}");
    // 표는 '라'(6번 글자) 바로 앞 8유닛 슬롯이다.
    let after_table = para.char_offsets[6];
    let starts: Vec<u32> = (0..para.line_segs.len())
        .map(|i| para.line_seg_text_start(i))
        .collect();
    assert_eq!(
        starts,
        [0, after_table - 8, after_table],
        "{label}: 표 줄은 표 자리에서, 다음 줄은 '라'에서 시작한다"
    );

    let tree = doc.build_page_render_tree(0).unwrap();
    let table = table_box(&tree.root).expect("표");
    let [ga, da, ra] = [3, 5, 6].map(|i| char_box(&tree.root, i).expect("글자 런"));
    assert!(
        (da.y - ga.y).abs() < 0.5,
        "{label}: '다'는 '가'와 같은 줄이다 — 가 {ga:?}, 다 {da:?}"
    );
    assert!(
        da.y + da.height <= table.y + 0.5,
        "{label}: 표는 앞 글자 줄 아래에 놓인다 — 다 {da:?}, 표 {table:?}"
    );
    assert!(
        ra.y >= table.y + table.height - 0.5,
        "{label}: '라'는 표 아래 줄에 놓인다 — 라 {ra:?}, 표 {table:?}"
    );
}

fn assert_rows_survive_save(doc: &HwpDocument) {
    assert_rows(doc, "합친 직후");
    let hwp = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    assert_rows(&hwp, "HWP 재열기");
    let hwpx = HwpDocument::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap();
    assert_rows(&hwpx, "HWPX 재열기");
}

#[test]
fn inline_table_gets_its_own_row_after_merge() {
    assert_rows_survive_save(&merged(Mark::Plain));
}

#[test]
fn inline_table_after_field_gets_its_own_row_after_merge() {
    assert_rows_survive_save(&merged(Mark::Field));
}

#[test]
fn inline_table_after_bookmark_gets_its_own_row_after_merge() {
    assert_rows_survive_save(&merged(Mark::Bookmark));
}
