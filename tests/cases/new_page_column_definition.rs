#![cfg(not(target_arch = "wasm32"))]
//! 새 쪽에서 시작하는 단 정의는 단 수와 종류가 앞 정의와 같아도 그 쪽부터 적용된다.
//!
//! 한컴 다단 설정의 '새 쪽으로'는 쪽 나누기 문단 맨 앞에 새 단 정의를 둔다. 단 방향이나
//! 구분선, 단 너비만 바꾼 정의도 그 쪽의 단 배치를 바꿔야 한다.
use std::io::{Cursor, Read, Write};

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

/// 너비가 같은 2단(간격 1134) 문서를 HWPX로 저장하고, 쪽 나누기 문단 맨 앞에 `col_pr`
/// 단 정의를 넣어 다시 연다. 둘째 쪽은 '둘째 쪽' 뒤 단 나누기로 두 단에 모두 글이 있다.
fn reopen_with_new_page_columns(col_pr: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.set_column_def_native(0, 2, 0, true, 1134).unwrap();
    doc.insert_text(0, 0, 0, "첫 쪽").unwrap();
    doc.insert_page_break_native(0, 0, 3).unwrap();
    doc.insert_text(0, 1, 0, "ANCHOR둘째 쪽").unwrap();
    doc.insert_column_break_native(0, 1, 10).unwrap();
    doc.insert_text(0, 2, 0, "둘째 단").unwrap();

    let mut input = zip::ZipArchive::new(Cursor::new(doc.export_hwpx_native().unwrap())).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches("ANCHOR").count(), 1);
            let control = format!("</hp:t><hp:ctrl>{col_pr}</hp:ctrl><hp:t>");
            bytes = xml.replace("ANCHOR", &control).into_bytes();
        }
        output
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let reopened = HwpDocument::from_bytes(&output.finish().unwrap().into_inner()).unwrap();
    assert_eq!(reopened.page_count(), 2);
    reopened
}

/// 쪽 렌더 트리의 모든 노드에 `pick`을 적용해 고른 값.
fn pick<T>(doc: &HwpDocument, page: u32, pick: impl Fn(&RenderNode) -> Option<T>) -> Vec<T> {
    fn walk<T>(node: &RenderNode, pick: &impl Fn(&RenderNode) -> Option<T>, out: &mut Vec<T>) {
        out.extend(pick(node));
        node.children
            .iter()
            .for_each(|child| walk(child, pick, out));
    }
    let mut out = Vec::new();
    walk(
        &doc.build_page_render_tree(page).unwrap().root,
        &pick,
        &mut out,
    );
    out
}

/// 문단의 글이 그 쪽에서 시작하는 x.
fn text_x(doc: &HwpDocument, page: u32, para: usize) -> f64 {
    pick(doc, page, |node| match &node.node_type {
        RenderNodeType::TextRun(run)
            if run.para_index == Some(para) && !run.text.trim().is_empty() =>
        {
            Some(node.bbox.x)
        }
        _ => None,
    })
    .into_iter()
    .reduce(f64::min)
    .unwrap_or_else(|| panic!("{page}쪽에 문단 {para}의 글이 없다"))
}

/// 단 번호 차례의 단 너비.
fn column_widths(doc: &HwpDocument, page: u32) -> Vec<f64> {
    let mut columns = pick(doc, page, |node| match node.node_type {
        RenderNodeType::Column(index) => Some((index, node.bbox.width)),
        _ => None,
    });
    columns.sort_by_key(|&(index, _)| index);
    columns.into_iter().map(|(_, width)| width).collect()
}

/// 세로 선(단 구분선)의 x.
fn vertical_lines(doc: &HwpDocument, page: u32) -> Vec<f64> {
    pick(doc, page, |node| match &node.node_type {
        RenderNodeType::Line(line) if line.x1 == line.x2 && line.y1 != line.y2 => Some(line.x1),
        _ => None,
    })
}

#[test]
fn right_to_left_columns_start_on_the_right_of_the_new_page() {
    let doc = reopen_with_new_page_columns(
        r#"<hp:colPr id="" type="NEWSPAPER" layout="RIGHT" colCount="2" sameSz="1" sameGap="1134"/>"#,
    );
    let (first, second) = (text_x(&doc, 1, 1), text_x(&doc, 1, 2));
    assert!(
        first > second,
        "오른쪽부터 채우는 단이면 둘째 쪽의 첫 단({first}px)이 둘째 단({second}px)보다 오른쪽이다"
    );
}

#[test]
fn column_line_is_drawn_on_the_new_page() {
    let doc = reopen_with_new_page_columns(
        r##"<hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="1" sameGap="1134"><hp:colLine type="SOLID" width="0.12 mm" color="#000000"/></hp:colPr>"##,
    );
    assert!(
        vertical_lines(&doc, 0).is_empty(),
        "첫 쪽에는 구분선이 없다"
    );
    let lines = vertical_lines(&doc, 1);
    let (first, second) = (text_x(&doc, 1, 1), text_x(&doc, 1, 2));
    assert!(
        lines.iter().any(|&x| first < x && x < second),
        "둘째 쪽의 두 단({first}px, {second}px) 사이에 구분선이 있어야 한다: {lines:?}"
    );
}

#[test]
fn column_widths_change_on_the_new_page() {
    let doc = reopen_with_new_page_columns(
        r#"<hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="0" sameGap="0"><hp:colSz width="8000" gap="768"/><hp:colSz width="24000" gap="0"/></hp:colPr>"#,
    );
    let widths = column_widths(&doc, 1);
    assert_eq!(widths.len(), 2, "둘째 쪽의 두 단: {widths:?}");
    let ratio = widths[0] / widths[1];
    assert!(
        (ratio - 8000.0 / 24000.0).abs() < 0.01,
        "둘째 쪽의 단 너비 비가 새 단 정의(8000:24000)와 같아야 한다: {widths:?}"
    );
}
