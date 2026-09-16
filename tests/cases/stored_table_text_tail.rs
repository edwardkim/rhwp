//! 내용에 따라 커진 TAC 표 다음 줄은 실제 표 흐름을 이어받는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::io::{Cursor, Read, Write};

const FIXTURE: &[u8] = include_bytes!("../../samples/stored-table-text-tail/native-8-0.hwpx");

fn fixture_bytes(profile: &str, count: usize, gap: i32) -> Vec<u8> {
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
            let footer_position = "vertpos=\"7000\"";
            assert_eq!(
                xml.matches(footer_position).count(),
                1,
                "saved Footer position"
            );
            xml = xml.replacen(footer_position, &format!("vertpos=\"{}\"", 7000 + gap), 1);
            output
                .start_file(name, zip::write::SimpleFileOptions::default())
                .expect("section entry");
            output.write_all(xml.as_bytes()).expect("section contents");
        } else {
            output.raw_copy_file(entry).expect("copy unchanged entry");
        }
    }
    output.finish().expect("fixture ZIP finish").into_inner()
}

fn collect(node: &RenderNode, nodes: &mut Vec<RenderNode>) {
    nodes.push(node.clone());
    for child in &node.children {
        collect(child, nodes);
    }
}

fn render(profile: &str, count: usize, gap: i32, spacing_after: Option<i32>) -> Vec<RenderNode> {
    let name = format!("{profile}-{count}-{gap}");
    let mut core = DocumentCore::from_bytes(&fixture_bytes(profile, count, gap))
        .expect("parse synthetic document");
    assert_eq!(
        core.document().layout_profile().hwp5_origin_hwpx(),
        profile == "native"
    );
    if let Some(spacing_after) = spacing_after {
        let mut doc = core.document().clone();
        let host = &mut doc.sections[0].paragraphs[1];
        let mut shape = doc.doc_info.para_shapes[host.para_shape_id as usize].clone();
        shape.spacing_after = spacing_after;
        host.para_shape_id = doc.doc_info.para_shapes.len() as u16;
        doc.doc_info.para_shapes.push(shape);
        core.set_document(doc);
    }
    assert_eq!(core.page_count(), 1, "{name}");
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
                    let nodes = render(profile, count, gap, Some(spacing));
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
                let nodes = render(profile, count, gap, None);
                let tables: Vec<_> = nodes
                    .iter()
                    .filter(|node| matches!(node.node_type, RenderNodeType::Table { .. }))
                    .collect();
                assert_eq!(tables.len(), 1, "{name}");
                let table = tables[0].bbox;
                let footer = text(&nodes, "Footer");
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
