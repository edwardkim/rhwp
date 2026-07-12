//! Issue #2020: 첨부 문서 렌더링 차이 회귀 게이트.
//!
//! 첨부/참조 문서는 하나의 이슈 범위에서 다룬다. 이 테스트는 자동 판정 가능한
//! 페이지 수와 FSC HWP/HWPX 흐름 동기화를 먼저 고정한다.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn has_table(root: &RenderNode, para_index: usize, control_index: usize) -> bool {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(para_index) && table.control_index == Some(control_index) {
                return true;
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    false
}

fn find_table_bbox(
    root: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(para_index) && table.control_index == Some(control_index) {
                return Some(node.bbox);
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    None
}

fn find_text_bbox(root: &RenderNode, needle: &str) -> Option<BoundingBox> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(needle) {
                return Some(node.bbox);
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    None
}

fn find_first_ellipse_bbox(root: &RenderNode) -> Option<BoundingBox> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if matches!(node.node_type, RenderNodeType::Ellipse(_)) {
            return Some(node.bbox);
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    None
}

fn find_line_bbox_near(
    root: &RenderNode,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Option<BoundingBox> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if matches!(node.node_type, RenderNodeType::Line(_))
            && (x_range.0..=x_range.1).contains(&node.bbox.x)
            && (y_range.0..=y_range.1).contains(&node.bbox.y)
        {
            return Some(node.bbox);
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    None
}

fn parse_svg_attr(attrs: &str, key: &str) -> Option<f64> {
    let p = attrs.find(&format!("{key}=\""))?;
    let s = p + key.len() + 2;
    let e = attrs[s..].find('"')? + s;
    attrs[s..e].parse::<f64>().ok()
}

fn svg_line_with_text(svg: &str, needle: &str) -> Option<(String, Vec<(f64, String)>)> {
    let mut by_y: BTreeMap<i32, Vec<(f64, String)>> = BTreeMap::new();
    let mut i = 0;
    while i < svg.len() {
        let Some(rel) = svg[i..].find("<text ") else {
            break;
        };
        let abs = i + rel;
        let after = &svg[abs + 6..];
        let Some(close) = after.find('>') else {
            i = abs + 6;
            continue;
        };
        let attrs = &after[..close];
        let content_start = abs + 6 + close + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else {
            i = abs + 6;
            continue;
        };
        let content = &svg[content_start..content_start + end_rel];
        if let (Some(x), Some(y)) = (parse_svg_attr(attrs, "x"), parse_svg_attr(attrs, "y")) {
            let y_key = (y * 10.0).round() as i32;
            by_y.entry(y_key)
                .or_default()
                .push((x, content.to_string()));
        }
        i = content_start + end_rel + 7;
    }

    for (_y, mut chars) in by_y {
        chars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let full: String = chars.iter().map(|(_, s)| s.as_str()).collect();
        if full.contains(needle) {
            return Some((full, chars));
        }
    }
    None
}

#[test]
fn issue_2020_reference_documents_keep_expected_page_counts() {
    assert_eq!(
        load_doc("samples/issue2020/passport_application_lawgo.hwp").page_count(),
        2
    );
    assert_eq!(
        load_doc("samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp")
            .page_count(),
        5
    );
    assert_eq!(
        load_doc("samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwpx")
            .page_count(),
        5
    );
    assert_eq!(load_doc("samples/복학원서.hwp").page_count(), 1);
    assert_eq!(
        load_doc("samples/2022년 국립국어원 업무계획.hwp").page_count(),
        35
    );
}

#[test]
fn issue_2020_passport_corner_quote_does_not_leave_extra_gap() {
    let doc = load_doc("samples/issue2020/passport_application_lawgo.hwp");
    let svg = doc
        .render_page_svg_native(0)
        .expect("render passport application page 1 SVG");
    let (line_text, chars) = svg_line_with_text(&svg, "2.「여권법」제9조")
        .expect("여권신청서 1쪽 동의 문구 줄을 찾아야 함");

    assert!(
        !line_text.contains("「 "),
        "원문에 없는 낫표 뒤 공백이 SVG 텍스트에 생기면 안 됨: {line_text}"
    );

    let open_idx = chars
        .iter()
        .position(|(_, text)| text == "「")
        .expect("opening corner quote");
    let yeo_idx = chars[open_idx + 1..]
        .iter()
        .position(|(_, text)| text == "여")
        .map(|idx| idx + open_idx + 1)
        .expect("Hangul after opening corner quote");
    let close_idx = chars
        .iter()
        .position(|(_, text)| text == "」")
        .expect("closing corner quote");
    let je_idx = chars[close_idx + 1..]
        .iter()
        .position(|(_, text)| text == "제")
        .map(|idx| idx + close_idx + 1)
        .expect("Hangul after closing corner quote");

    let open_gap = chars[yeo_idx].0 - chars[open_idx].0;
    let close_gap = chars[je_idx].0 - chars[close_idx].0;
    assert!(
        open_gap <= 8.5 && close_gap <= 8.5,
        "낫표 advance 는 반각 수준이어야 함: open_gap={open_gap:.2}, close_gap={close_gap:.2}, line={line_text}"
    );
}

#[test]
fn issue_2020_fsc_hwp_keeps_tail_table_on_page_two() {
    let doc = load_doc("samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp");
    let tree = doc
        .build_page_render_tree(1)
        .expect("render FSC HWP page 2");

    assert!(
        has_table(&tree.root, 24, 0),
        "FSC HWP pi=24 14x15 표는 HWPX/한컴 기준처럼 2쪽 하단에 남아야 한다"
    );
}

#[test]
fn issue_2020_bokhak_receipt_seal_line_and_stamp_align() {
    let doc = load_doc("samples/복학원서.hwp");
    let tree = doc
        .build_page_render_tree(0)
        .expect("render bokhak receipt page");
    let svg = doc
        .render_page_svg_native(0)
        .expect("render bokhak receipt SVG");

    assert!(
        svg_line_with_text(&svg, "(인)").is_some(),
        "복학원서 접수증 위 날인선에는 `(인)` 표시가 렌더링되어야 함"
    );
    assert!(
        !svg.contains('\u{F081C}'),
        "TAC filler 원문 U+F081C가 SVG에 그대로 출력되면 안 됨"
    );

    let seal_line = find_text_bbox(&tree.root, "(인)").expect("합성 날인선 TextRun");
    let receipt_table = find_table_bbox(&tree.root, 16, 0).expect("pi=16 receipt table");
    assert!(
        seal_line.y < receipt_table.y && seal_line.width > 600.0,
        "날인선은 접수증 표 위 본문 폭에 가깝게 놓여야 함: line={seal_line:?}, table={receipt_table:?}"
    );
    assert!(
        (790.0..=800.0).contains(&receipt_table.y),
        "접수증 TAC 표는 filler line 다음 line-seg 위치에 배치되어야 함: y={:.1}",
        receipt_table.y
    );

    let stamp_text = find_text_bbox(&tree.root, "㊞").expect("receipt stamp text");
    let stamp_circle = find_first_ellipse_bbox(&tree.root).expect("receipt stamp circle");
    let text_cx = stamp_text.x + stamp_text.width / 2.0;
    let text_cy = stamp_text.y + stamp_text.height / 2.0;
    let circle_cx = stamp_circle.x + stamp_circle.width / 2.0;
    let circle_cy = stamp_circle.y + stamp_circle.height / 2.0;
    assert!(
        (609.0..=616.0).contains(&stamp_circle.x)
            && (948.0..=954.0).contains(&stamp_circle.y)
            && (87.0..=92.0).contains(&stamp_circle.width)
            && (82.0..=88.0).contains(&stamp_circle.height),
        "빨간 도장 원은 한컴 PDF 기준 위치/크기를 따라야 함: circle={stamp_circle:?}"
    );
    assert!(
        (15.0..=28.0).contains(&(circle_cx - text_cx)) && (text_cy - circle_cy).abs() <= 8.0,
        "날짜 옆 `㊞`은 빨간 도장 원 중심이 아니라 한컴처럼 원 내부 왼쪽에 놓여야 함: text=({text_cx:.1},{text_cy:.1}) circle=({circle_cx:.1},{circle_cy:.1})"
    );

    let marker = find_line_bbox_near(&tree.root, (695.0, 713.0), (1028.0, 1035.0))
        .expect("표 뒤 U+F081C 선문자 marker");
    assert!(
        (8.0..=16.0).contains(&marker.width) && marker.height <= 1.2,
        "도장 오른쪽 아래 U+F081C 선문자는 짧은 검은 가로선으로 렌더되어야 함: marker={marker:?}"
    );
}
