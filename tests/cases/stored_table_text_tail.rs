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

/// 정상 한컴 저장 줄의 소속과 표 뒤 본문 흐름을 검사한다.
#[test]
fn hancom_saved_tail_preserves_pdf_baseline_after_the_table() {
    let mut core = DocumentCore::from_bytes(FIXTURE).expect("Hancom saved public fixture");
    let host = &core.document().sections[0].paragraphs[1];
    assert_eq!(host.line_segs.len(), 4, "한컴이 저장한 실제 줄 소속");
    assert_eq!(host.line_segs[3].text_start, 83);
    let object_line = host.line_segs[1].clone();
    let footer_line = host.line_segs[3].clone();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).expect("render");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let footers: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(1)
                    && run.cell_context.is_none()
                    && run.text == "          Footer" =>
            {
                Some((node.bbox, node.bbox.y + run.baseline))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        footers.len(),
        1,
        "뒤 문장은 원래 문단의 본문에 한 번만 표시"
    );
    let (footer, baseline) = footers[0];
    let tables: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Table { .. }))
        .collect();
    assert_eq!(tables.len(), 1);
    let table = tables[0].bbox;
    // 한컴 PDF로 확인한 저장 줄 관계를 사용한다. 용지의 절대 좌표는 고정하지 않는다.
    let stored_baseline_delta = rhwp::renderer::hwpunit_to_px(
        footer_line.vertical_pos - object_line.vertical_pos + footer_line.baseline_distance,
        96.0,
    );
    assert!(
        (baseline - table.y - stored_baseline_delta).abs() < 0.5,
        "뒤 문장은 표 앞 공백 줄을 다시 예약하지 않고 원래 저장 줄을 이어받아야 함"
    );
    assert!(footer.y > table.y + table.height);
    assert!(footer.x >= table.x && footer.x + footer.width <= table.x + table.width);
    let mut previous_bottom = table.y;
    for index in 1..=8 {
        let value = format!("Cell {index}");
        let cell_runs: Vec<_> = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.cell_context.is_some() && run.text == value => {
                    Some((node.bbox, node.bbox.y + run.baseline))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            cell_runs.len(),
            1,
            "셀 문장은 중복·누락 없이 한 번만 표시: {value}"
        );
        let (bbox, cell_baseline) = cell_runs[0];
        assert!(bbox.y >= previous_bottom);
        // 글꼴의 여유 상자 끝과 실제 글줄 기준선을 혼동하지 않는다.
        assert!(cell_baseline <= table.y + table.height);
        previous_bottom = bbox.y + bbox.height;
    }
}

#[test]
fn hancom_saved_object_row_keeps_its_character_border() {
    let mut core = DocumentCore::from_bytes(FIXTURE).expect("Hancom saved fixture");
    let tree = core.build_page_render_tree(0).expect("render");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table { .. }))
        .unwrap();
    let footer = text(&nodes, "          Footer");
    let outlines: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Rectangle(_))
                && node.bbox.y < table.bbox.y
                && node.bbox.y + node.bbox.height >= footer.y + footer.height
                && node.bbox.height > table.bbox.height
        })
        .collect();
    assert_eq!(outlines.len(), 1, "표와 뒤 문장을 소유하는 문단 외곽");
    // 정상 한컴 저장본의 첫 공백 줄은 높이를 점유하고 0 간격을 남긴다.
    // 표 원점은 해당 저장 줄 끝을 따르며 글꼴 상대 크기100%로 재조판하지 않는다.
    let source = &core.document().sections[0].paragraphs[1];
    let first = &source.line_segs[0];
    let object = &source.line_segs[1];
    assert_eq!(first.line_spacing, 0);
    let prefix = rhwp::renderer::hwpunit_to_px(object.vertical_pos - first.vertical_pos, 96.0);
    assert!(
        (table.bbox.y - outlines[0].bbox.y - prefix).abs() < 0.5,
        "저장 공백 줄의 0 간격이 표 앞 흐름에서 보존되어야 함"
    );
    let row_borders: Vec<_> = table
        .children
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Line(_))
                && node.bbox.width > table.bbox.width
                && node.bbox.y > table.bbox.y + table.bbox.height
                && node.bbox.y < footer.y
        })
        .collect();
    assert_eq!(
        row_borders.len(),
        1,
        "개체 줄은 두 뒤 공백까지 문자 테두리를 소유하며 Footer를 포함하지 않아야 함"
    );
}

fn saved_fixture_nodes() -> Vec<RenderNode> {
    let mut core = DocumentCore::from_bytes(FIXTURE).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    nodes
}

#[test]
fn empty_saved_paragraph_keeps_its_physical_border() {
    let nodes = saved_fixture_nodes();
    let empty = nodes
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(0)))
        .expect("빈 선행 문단의 물리 글줄");
    let following = nodes
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(1)))
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .expect("표를 소유한 뒤 문단의 첫 글줄");
    assert!(empty.bbox.height > 0.0, "빈 문단도 물리 줄을 소유한다");
    let borders: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Rectangle(_))
                && node.bbox.y <= empty.bbox.y
                && node.bbox.y + node.bbox.height >= empty.bbox.y + empty.bbox.height
                && node.bbox.x <= empty.bbox.x
                && node.bbox.x + node.bbox.width >= empty.bbox.x + empty.bbox.width
        })
        .collect();
    assert_eq!(borders.len(), 1, "빈 선행 문단 테두리의 단일 소유");
    // 독립 PDF에서 앞 빈 문단의 아래 선은 뒤 문단 첫 줄의 위 선과 맞닿는다.
    // 용지 좌표·폭·줄 높이를 고정하지 않고 두 문단 사이의 물리 줄 소유를 검사한다.
    let bottom = borders[0].bbox.y + borders[0].bbox.height;
    assert!(
        (bottom - following.bbox.y).abs() < 1e-9,
        "빈 줄의 후행 간격까지 테두리가 감싸고 뒤 문단은 그 아래에서 시작해야 한다"
    );
}

#[test]
fn one_paragraph_keeps_one_border_across_table_and_text_items() {
    let mut core = DocumentCore::from_bytes(FIXTURE).expect("한컴 저장본");
    let host = &core.document().sections[0].paragraphs[1];
    let shape = &core.document().doc_info.para_shapes[host.para_shape_id as usize];
    assert_eq!(
        shape.attr1 & (1 << 28),
        0,
        "문단 간 테두리 연결이 꺼진 입력"
    );
    assert_eq!(core.page_count(), 1);
    let pages = core.dump_page_items_json(Some(0));
    let items = pages[0]["columns"][0]["items"]
        .as_array()
        .expect("본문 항목");
    let kinds: Vec<_> = items
        .iter()
        .filter(|item| item["paraIndex"] == 1)
        .map(|item| item["kind"].as_str().expect("항목 종류"))
        .collect();
    assert_eq!(
        kinds,
        ["partialParagraph", "table", "partialParagraph"],
        "같은 문단의 실제 텍스트/표/뒤 텍스트 경로를 실행해야 함"
    );
    let tree = core.build_page_render_tree(0).expect("render");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let tables: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Table { .. }))
        .collect();
    assert_eq!(tables.len(), 1);
    let table = tables[0].bbox;
    let footer = text(&nodes, "          Footer");
    // 연결 속성은 서로 다른 문단의 경계를 잇는다. 같은 문단의 항목 분할은
    // 테두리를 복제하지 않으며 앞 공백 줄·표·뒤 문장을 한 외곽이 소유한다.
    let outlines: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Rectangle(_))
                && node.bbox.x <= table.x
                && node.bbox.x + node.bbox.width >= table.x + table.width
                && node.bbox.y < table.y
                && node.bbox.y + node.bbox.height >= footer.y + footer.height
        })
        .collect();
    assert_eq!(
        outlines.len(),
        1,
        "같은 문단의 표와 뒤 문장을 소유하는 외곽은 하나"
    );
}

#[test]
fn object_row_border_does_not_take_ownership_of_visible_text_on_the_same_row() {
    let mut core = DocumentCore::from_bytes(FIXTURE).unwrap();
    let mut doc = core.document().clone();
    let host = &mut doc.sections[0].paragraphs[1];
    let table_index = host
        .controls
        .iter()
        .position(|c| matches!(c, rhwp::model::control::Control::Table(_)))
        .unwrap();
    let index = host.control_text_positions()[table_index];
    let mut chars: Vec<_> = host.text.chars().collect();
    assert_eq!(chars[index], ' ', "first space beside the object");
    chars[index] = 'A';
    host.text = chars.into_iter().collect();
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table { .. }))
        .unwrap();
    assert!(
        !table
            .children
            .iter()
            .any(|n| matches!(n.node_type, RenderNodeType::Line(_))
                && n.bbox.width > table.bbox.width + 1.0
                && n.bbox.y > table.bbox.y + table.bbox.height + 1.0),
        "mixed text/object row must not receive a second object-only character border"
    );
}

#[test]
fn unbordered_trailing_space_does_not_erase_the_object_border() {
    let mut core = DocumentCore::from_bytes(FIXTURE).unwrap();
    let mut doc = core.document().clone();
    let mut plain = doc.doc_info.char_shapes[0].clone();
    plain.border_fill_id = 0;
    let id = doc.doc_info.char_shapes.len() as u32;
    doc.doc_info.char_shapes.push(plain);
    // Two spaces follow the table's raw [15,23) control slot. Changing their
    // style must stop decoration extension, not steal the table's own style.
    doc.sections[0].paragraphs[1]
        .char_shapes
        .push(rhwp::model::paragraph::CharShapeRef {
            start_pos: 23,
            char_shape_id: id,
        });
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table { .. }))
        .unwrap();
    let border = table
        .children
        .iter()
        .find(|n| {
            matches!(n.node_type, RenderNodeType::Line(_))
                && n.bbox.width > 100.0
                && n.bbox.y > table.bbox.y + table.bbox.height + 1.0
        })
        .expect("the table keeps its own character decoration");
    assert!(
        (border.bbox.width - table.bbox.width).abs() < 0.5,
        "unbordered spaces must not extend the table's border: {:?}",
        border.bbox
    );
}

#[test]
fn saved_whitespace_row_keeps_its_character_border_width() {
    let nodes = saved_fixture_nodes();
    let spaces = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextRun(run) if run.cell_context.is_none() && run.text == " ".repeat(58))
        })
        .expect("saved 58-space row between table and Footer");
    // Hancom character border: x=36..326.52pt. Do not squeeze the stored
    // whitespace row to the 288pt paragraph frame merely to hide overflow.
    assert!(
        (spaces.bbox.width - (326.52 - 36.0) * 4.0 / 3.0).abs() < 1.0,
        "whitespace decoration width {:?}",
        spaces.bbox
    );
}

#[test]
fn whitespace_only_paragraph_keeps_its_pdf_underline_extent() {
    // Unlike the soft-wrapped separator row in FIXTURE, these spaces are a
    // complete paragraph used as a rule. Its line-fit contract still applies.
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(include_bytes!(
        "../../samples/hwpx/issue_157.hwpx"
    ))
    .expect("public whitespace underline fixture");
    let svg = doc.render_page_svg_native(1).expect("second page");
    let svg = roxmltree::Document::parse(&svg).expect("SVG XML");
    let x2 = svg
        .descendants()
        .filter(|n| n.has_tag_name("line"))
        .find_map(|n| {
            let y1 = n.attribute("y1")?.parse::<f64>().ok()?;
            let x2 = n.attribute("x2")?.parse::<f64>().ok()?;
            ((y1 - 411.07).abs() < 1.0 && x2 > 700.0).then_some(x2)
        })
        .expect("whitespace paragraph underline");
    // pdf/hwpx/issue_157-2022.pdf p2: x2=559.859008789pt.
    // One pixel covers the integer endpoint emitted by the underline painter.
    assert!(
        (x2 - 559.859008789 * 4.0 / 3.0).abs() < 1.0,
        "paragraph-end spaces keep their fitted underline: {x2}"
    );
}

#[test]
fn object_only_placeholder_keeps_border_without_stored_rows() {
    for text in ["", "\u{fffc}"] {
        let mut core = DocumentCore::from_bytes(FIXTURE).unwrap();
        let mut doc = core.document().clone();
        let host = &mut doc.sections[0].paragraphs[1];
        host.text = text.into();
        host.char_offsets.clear();
        host.line_segs.clear();
        core.set_document(doc);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut nodes = Vec::new();
        collect(&tree.root, &mut nodes);
        let table = nodes
            .iter()
            .find(|n| matches!(n.node_type, RenderNodeType::Table { .. }))
            .unwrap();
        // Both encodings denote an object-only paragraph, not visible text.
        // The existing character decoration extends past the physical table.
        assert!(
            table
                .children
                .iter()
                .any(|n| matches!(n.node_type, RenderNodeType::Line(_))
                    && n.bbox.width >= table.bbox.width
                    && n.bbox.y > table.bbox.y + table.bbox.height + 1.0),
            "object-only carrier {text:?} must retain its character border"
        );
    }
}
