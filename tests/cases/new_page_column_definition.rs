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
        if entry.name().expect("HWPX ZIP 엔트리 이름") == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches("ANCHOR").count(), 1);
            let control = format!("</hp:t><hp:ctrl>{col_pr}</hp:ctrl><hp:t>");
            bytes = xml.replace("ANCHOR", &control).into_bytes();
        }
        output
            .start_file(
                entry.name().expect("HWPX ZIP 엔트리 이름"),
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
    for (page, right_to_left) in [(0, false), (1, true)] {
        let info: serde_json::Value =
            serde_json::from_str(&doc.get_page_info_native(page).unwrap()).unwrap();
        let columns = info["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2);
        let first_x = columns[0]["x"].as_f64().unwrap();
        let second_x = columns[1]["x"].as_f64().unwrap();
        assert_eq!(
            first_x > second_x,
            right_to_left,
            "{page}쪽 편집용 단 순서가 원본 방향과 달라짐: {columns:?}"
        );
    }
}

#[test]
fn mirror_columns_use_the_new_page_parity() {
    let doc = reopen_with_new_page_columns(
        r#"<hp:colPr id="" type="NEWSPAPER" layout="MIRROR" colCount="2" sameSz="1" sameGap="1134"/>"#,
    );
    assert!(
        text_x(&doc, 1, 1) > text_x(&doc, 1, 2),
        "맞쪽 방향의 둘째 물리 쪽은 오른쪽 단부터 채워야 한다"
    );
    let info: serde_json::Value =
        serde_json::from_str(&doc.get_page_info_native(1).unwrap()).unwrap();
    assert!(info["columns"][0]["x"].as_f64().unwrap() > info["columns"][1]["x"].as_f64().unwrap());
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
        (ratio - 1.0 / 3.0).abs() < 0.01,
        "둘째 쪽의 단 너비 비가 새 단 정의(1:3)와 같아야 한다: {widths:?}"
    );

    // 눈금자·편집기는 render tree 대신 getPageInfo의 단 정보를 소비한다.
    // 새 쪽의 첫 영역에서는 두 소비자 모두 같은 새 단 정의를 읽어야 한다.
    for (page, expected_ratio) in [(0, 1.0), (1, 1.0 / 3.0)] {
        let info: serde_json::Value =
            serde_json::from_str(&doc.get_page_info_native(page).unwrap()).unwrap();
        let columns = info["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2, "{page}쪽의 편집용 단 정보");
        let ratio = columns[0]["width"].as_f64().unwrap() / columns[1]["width"].as_f64().unwrap();
        assert!(
            (ratio - expected_ratio).abs() < 0.01,
            "{page}쪽 getPageInfo 단 너비 비가 원본 정의와 달라짐: {columns:?}"
        );
    }
}

#[test]
fn hancom_saved_unequal_columns_fill_the_body_in_both_formats() {
    for format in ["hwp", "hwpx"] {
        let path = format!(
            "{}/tests/fixtures/semanticist21_20261015/pr7673-width-hancom.{format}",
            env!("CARGO_MANIFEST_DIR")
        );
        let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(doc.page_count(), 2, "한컴 재저장본의 쪽수");
        let mut columns = pick(&doc, 1, |node| match node.node_type {
            RenderNodeType::Column(index) => Some((index, node.bbox)),
            _ => None,
        });
        columns.sort_by_key(|(index, _)| *index);
        assert_eq!(columns.len(), 2);
        let info: serde_json::Value =
            serde_json::from_str(&doc.get_page_info_native(1).unwrap()).unwrap();
        let body_right = info["bodyRight"].as_f64().unwrap();
        let right = columns[1].1.x + columns[1].1.width;
        // 한컴이 저장한 width 8000, gap 768, width 24000은 합계 32768인 비례값이다.
        // 두 단과 간격은 본문 폭을 채운다. 0.1은 JSON API의 소수 첫째 자리 반올림 오차다.
        assert!(
            (right - body_right).abs() < 0.1,
            "{format} 단 영역이 본문 가로 범위를 채우지 못함: 끝={right}, 본문 끝={body_right}"
        );
    }
}
