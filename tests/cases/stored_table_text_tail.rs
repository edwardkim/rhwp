//! 내용에 따라 커진 TAC 표 다음 줄은 실제 표 흐름을 이어받는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::io::{Cursor, Read, Write};

const FIXTURE: &[u8] = include_bytes!("../../samples/stored-table-text-tail/native-8-0.hwpx");

fn fixture_bytes(profile: &str, count: usize, gap: i32, with_shape: bool) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(FIXTURE)).expect("fixture ZIP");
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).expect("fixture entry");
        let name = entry.name().to_owned();
        // 계보 표식은 파싱에도 영향을 주므로 문서 모델을 만든 뒤 지우지 않는다.
        if profile == "pure" && name == rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH {
            continue;
        }
        if name == "Contents/section0.xml" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).expect("section XML");
            let parsed = roxmltree::Document::parse(&xml).expect("fixture XML");
            let cell = parsed
                .descendants()
                .find(|n| n.has_tag_name("subList"))
                .expect("cell");
            let paragraphs: Vec<_> = cell.children().filter(|n| n.has_tag_name("p")).collect();
            assert_eq!(paragraphs.len(), 8, "base fixture cell paragraphs");
            let removed: Vec<_> = paragraphs.iter().skip(count).map(|n| n.range()).collect();
            for range in removed.into_iter().rev() {
                xml.replace_range(range, "");
            }
            // 공개 샘플은 한컴 정상 저장본이다. 아래부터는 PDF 기준 입력이 아니라
            // 작은 선언 표가 자라는 경우의 저장 흐름 계약을 메모리에서 만든다.
            // 실제 파일의 4줄/셀 줄 메트릭을 수동 정보로 덮어쓴 사실을 숨기지 않는다.
            let parsed = roxmltree::Document::parse(&xml).expect("fixture XML");
            let ranges: Vec<_> = parsed
                .descendants()
                .filter(|n| n.has_tag_name("linesegarray"))
                .map(|n| n.range())
                .collect();
            for range in ranges.into_iter().rev() {
                xml.replace_range(range, "");
            }
            assert_eq!(xml.matches("height=\"12872\"").count(), 1);
            xml = xml.replacen("height=\"12872\"", "height=\"6000\"", 1);
            let saved_tail = format!("<hp:t>{}Footer</hp:t>", " ".repeat(70));
            let split_tail = format!(
                "<hp:t>{}</hp:t></hp:run><hp:run charPrIDRef=\"0\"><hp:t>Footer</hp:t>",
                " ".repeat(2)
            );
            assert_eq!(xml.matches(&saved_tail).count(), 1);
            xml = xml.replacen(&saved_tail, &split_tail, 1);
            let parsed = roxmltree::Document::parse(&xml).expect("fixture XML");
            let host_end = parsed
                .root_element()
                .children()
                .filter(|n| n.has_tag_name("p"))
                .nth(1)
                .expect("host")
                .range()
                .end
                - "</hp:p>".len();
            let rows = format!(
                r#"<hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="1000" textheight="1000" baseline="850" spacing="0" horzpos="0" horzsize="28800" flags="393216"/><hp:lineseg textpos="15" vertpos="1000" vertsize="6000" textheight="6000" baseline="5100" spacing="0" horzpos="0" horzsize="28800" flags="393216"/><hp:lineseg textpos="25" vertpos="{}" vertsize="1000" textheight="1000" baseline="850" spacing="0" horzpos="0" horzsize="28800" flags="393216"/></hp:linesegarray>"#,
                7000 + gap
            );
            xml.insert_str(host_end, &rows);
            let parsed = roxmltree::Document::parse(&xml).expect("fixture XML");
            let first_end = parsed
                .root_element()
                .children()
                .find(|n| n.has_tag_name("p"))
                .expect("section paragraph")
                .range()
                .end
                - "</hp:p>".len();
            xml.insert_str(first_end, r#"<hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="1000" textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="42520" flags="393216"/></hp:linesegarray>"#);
            if with_shape {
                // The rectangle is another body item owned by the table's paragraph.
                // Keep its eight-unit control slot in the saved text-position axis.
                assert_eq!(xml.matches("textpos=\"25\"").count(), 1);
                xml = xml.replacen("textpos=\"25\"", "textpos=\"33\"", 1);
                xml = xml.replacen(
                    "</hp:tbl>",
                    r#"</hp:tbl><hp:rect id="101" zOrder="1" numberingType="PICTURE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" ratio="0">
<hp:offset x="0" y="0"/><hp:orgSz width="6000" height="1500"/><hp:curSz width="6000" height="1500"/>
<hp:rotationInfo angle="0" centerX="3000" centerY="750" rotateimage="1"/>
<hp:sz width="6000" widthRelTo="ABSOLUTE" height="1500" heightRelTo="ABSOLUTE" protect="0"/>
<hp:pos treatAsChar="0" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="18000"/>
<hp:outMargin left="0" right="0" top="0" bottom="0"/>
<hp:pt0 x="0" y="0"/><hp:pt1 x="6000" y="0"/><hp:pt2 x="6000" y="1500"/><hp:pt3 x="0" y="1500"/>
</hp:rect>"#,
                    1,
                );
                xml = xml.replacen(
                    "</hs:sec>",
                    r#"<hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>Following</hp:t></hp:run></hp:p></hs:sec>"#,
                    1,
                );
            }
            output
                .start_file(name, zip::write::SimpleFileOptions::default())
                .expect("section entry");
            output.write_all(xml.as_bytes()).expect("section contents");
        } else {
            output.raw_copy_file(entry).expect("copy unchanged entry");
        }
    }
    if profile == "native" {
        output
            .start_file(
                rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH,
                zip::write::SimpleFileOptions::default(),
            )
            .expect("synthetic native profile marker");
        output.write_all(b"1").expect("marker contents");
    }
    output.finish().expect("fixture ZIP finish").into_inner()
}

fn collect(node: &RenderNode, nodes: &mut Vec<RenderNode>) {
    nodes.push(node.clone());
    for child in &node.children {
        collect(child, nodes);
    }
}

fn render(
    profile: &str,
    count: usize,
    gap: i32,
    spacing_after_hu: Option<i32>,
    with_shape: bool,
) -> Vec<RenderNode> {
    let name = format!("{profile}-{count}-{gap}");
    let mut core = DocumentCore::from_bytes(&fixture_bytes(profile, count, gap, with_shape))
        .expect("parse synthetic document");
    assert_eq!(
        core.document().layout_profile().hwp5_origin_hwpx(),
        profile == "native"
    );
    if let Some(spacing_after_hu) = spacing_after_hu {
        let mut doc = core.document().clone();
        let host = &mut doc.sections[0].paragraphs[1];
        let mut shape = doc.doc_info.para_shapes[host.para_shape_id as usize].clone();
        // HWPX HwpUnitChar values are normalized to the HWP5 model's 2x scale.
        shape.spacing_after = spacing_after_hu * 2;
        host.para_shape_id = doc.doc_info.para_shapes.len() as u16;
        doc.doc_info.para_shapes.push(shape);
        core.set_document(doc);
    }
    assert_eq!(core.page_count(), 1, "{name}");
    if with_shape {
        let pages = core.dump_page_items_json(Some(0));
        let items = pages[0]["columns"][0]["items"]
            .as_array()
            .expect("body items");
        for kind in ["table", "shape", "partialParagraph"] {
            assert!(
                items
                    .iter()
                    .any(|item| item["paraIndex"] == 1 && item["kind"] == kind),
                "{name}: exercise the real mixed-object paragraph path: {items:?}"
            );
        }
    }
    let tree = core.build_page_render_tree(0).expect("render");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    nodes
}

#[test]
fn paragraph_after_spacing_does_not_move_its_own_text_tail() {
    for profile in ["native", "pure"] {
        for count in [2, 8] {
            for gap in [0, 600] {
                let name = format!("{profile}-{count}-{gap}");
                let mut positions = Vec::new();
                for spacing in [0, 600, 1200] {
                    let nodes = render(profile, count, gap, Some(spacing), false);
                    positions.push(text(&nodes, "Footer").y);
                }
                for y in &positions[1..] {
                    assert!(
                        (y - positions[0]).abs() < 0.5,
                        "{name}: paragraph after-spacing must follow its own Footer: {positions:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn paragraph_after_spacing_follows_the_union_of_text_table_and_shape() {
    for profile in ["native", "pure"] {
        for count in [2, 8] {
            let mut positions = Vec::new();
            for spacing in [0, 600, 1200] {
                let nodes = render(profile, count, 0, Some(spacing), true);
                let shapes: Vec<_> = nodes
                    .iter()
                    .filter(|node| {
                        matches!(&node.node_type, RenderNodeType::Rectangle(shape)
                        if shape.para_index == Some(1) && shape.control_index == Some(1))
                    })
                    .collect();
                assert_eq!(shapes.len(), 1, "body rectangle must actually be laid out");
                positions.push((
                    text(&nodes, "Following").y,
                    text(&nodes, "Footer").y,
                    shapes[0].bbox.y,
                ));
            }
            for (index, &(following, footer, shape)) in positions.iter().enumerate() {
                // 600 HU of paragraph after-spacing is 8 px at 96 dpi.
                assert!((following - positions[0].0 - index as f64 * 8.0).abs() < 0.5,
                    "{profile}-{count}: apply after-spacing once after all paragraph items: {positions:?}");
                assert!(
                    (footer - positions[0].1).abs() < 0.5,
                    "same-paragraph text must not move"
                );
                assert!(
                    (shape - positions[0].2).abs() < 0.5,
                    "same-paragraph shape must not move"
                );
            }
        }
    }
}

fn text(nodes: &[RenderNode], expected: &str) -> BoundingBox {
    let matches: Vec<_> = nodes
        .iter()
        .filter(
            |node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == expected),
        )
        .collect();
    assert_eq!(matches.len(), 1, "exactly one {expected}");
    matches[0].bbox
}

#[test]
fn stored_text_tail_follows_the_measured_table_and_preserves_the_line_gap() {
    for profile in ["native", "pure"] {
        for count in [2, 8] {
            let mut positions = Vec::new();
            for gap in [0, 600] {
                let name = format!("{profile}-{count}-{gap}");
                let nodes = render(profile, count, gap, None, false);
                let tables: Vec<_> = nodes
                    .iter()
                    .filter(|node| matches!(node.node_type, RenderNodeType::Table { .. }))
                    .collect();
                assert_eq!(tables.len(), 1, "{name}");
                let table = tables[0].bbox;
                let footer = text(&nodes, "Footer");
                // 15 leading spaces + the eight-unit table + two spaces = textpos 25.
                // Footer itself starts the saved tail row; no fourth line is needed.
                assert!((footer.x - table.x).abs() < 0.5, "{name}: {footer:?}");
                assert!(footer.x + footer.width <= table.x + table.width + 0.5);
                for index in 0..count {
                    // Native shaping may split one cell paragraph into several runs.
                    // Use its document ownership, not a particular run boundary.
                    let cell_runs: Vec<_> = nodes
                        .iter()
                        .filter_map(|node| match &node.node_type {
                            RenderNodeType::TextRun(run)
                                if run.cell_context.as_ref().is_some_and(|context| {
                                    context
                                        .path
                                        .last()
                                        .is_some_and(|owner| owner.cell_para_index == index)
                                }) =>
                            {
                                Some((node.bbox, run.text.as_str()))
                            }
                            _ => None,
                        })
                        .collect();
                    let content: String = cell_runs.iter().map(|(_, text)| *text).collect();
                    assert_eq!(content, format!("Cell {}", index + 1), "{name}");
                    for (bbox, _) in cell_runs {
                        assert!(bbox.y >= table.y - 0.5, "{name}");
                        assert!(
                            bbox.y + bbox.height <= table.y + table.height + 0.5,
                            "{name} cell {index} must remain visible"
                        );
                    }
                }
                assert!(
                    footer.y >= table.y + table.height - 0.5,
                    "{name}: footer {} precedes table bottom {}",
                    footer.y,
                    table.y + table.height
                );
                if count == 8 {
                    assert!(table.height > 80.5, "fixture must grow beyond 6000 HU");
                } else {
                    // The saved table line is 6000 HU high, with no outer margins.
                    // Its following line starts at 7000 + gap, the table line at 1000.
                    // The preceding 1000 HU blank line must not become a trailing gap.
                    assert!((table.height - 80.0).abs() < 0.5, "{name}");
                    let expected_gap = f64::from(gap) * 96.0 / 7200.0;
                    assert!(
                        (footer.y - table.y - table.height - expected_gap).abs() < 0.5,
                        "{name}: preserve the explicit saved table-to-text gap"
                    );
                }
                positions.push(footer.y);
            }
            // 600 HWPUNIT at 96 dpi is 8 px, independently of table growth.
            assert!((positions[1] - positions[0] - 8.0).abs() < 0.5);
        }
    }
}

/// 수동 줄 정보의 기하 검사와 별도로 한컴 정상 저장본의 PDF 좌표를 검사한다.
#[test]
fn hancom_saved_tail_preserves_pdf_baseline_after_the_table() {
    let mut core = DocumentCore::from_bytes(FIXTURE).expect("Hancom saved public fixture");
    let host = &core.document().sections[0].paragraphs[1];
    assert_eq!(host.line_segs.len(), 4, "Hancom's actual text partition");
    assert_eq!(host.line_segs[3].text_start, 83);
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).expect("render");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let (footer, baseline) = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(1)
                    && run.cell_context.is_none()
                    && run.text == "          Footer" =>
            {
                Some((node.bbox, node.bbox.y + run.baseline))
            }
            _ => None,
        })
        .expect("Footer text run");
    // Hancom PDF: Footer baseline y=265.866821pt, run x=36.24pt (10 leading spaces).
    // Half a pixel covers 600-dpi PDF quantization and font metric rounding.
    assert!(
        (baseline - 265.866821 * 96.0 / 72.0).abs() < 0.5,
        "Footer baseline {baseline} must not add the preceding blank row"
    );
    assert!((footer.x - 36.24 * 96.0 / 72.0).abs() < 0.5);
    let tables: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Table { .. }))
        .collect();
    assert_eq!(tables.len(), 1);
    let table = tables[0].bbox;
    // PDF table perimeter, separate from the surrounding paragraph border.
    assert!((table.y - 140.77).abs() < 0.5, "table top {table:?}");
    assert!(
        (table.y + table.height - 312.42).abs() < 0.5,
        "table bottom {table:?}"
    );
    assert!(footer.y > table.y + table.height);
}
