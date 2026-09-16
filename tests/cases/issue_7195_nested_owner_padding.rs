//! Owner-derived widths must use the host content box, not its border box.
//! Padding variants are contract tests; the unmodified source/PDF comparison
//! remains the independent visual evidence for #2308.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_normalization::{
    RenderNormalizationOverlay, RenderPath, RenderPathEntry,
};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/76076_regulatory_analysis.hwp"),
    )
    .expect("read #2308 source");
    DocumentCore::from_bytes(&bytes).expect("parse #2308 source")
}

fn child(owner: &Table) -> &Table {
    owner.cells[9].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("nested table")
}

#[test]
fn owner_projection_respects_effective_horizontal_padding() {
    let source = core();
    // Expected widths follow the input geometry and padding precedence, not
    // the implementation output or a desired page count.
    for (table_lr, cell_lr, apply, expected) in [
        ((510, 510), (510, 510), false, 37_225),
        ((300, 700), (510, 510), false, 37_245),
        ((510, 510), (100, 200), true, 37_945),
        ((510, 510), (0, 0), true, 38_245),
        ((0, 0), (510, 510), false, 38_245),
    ] {
        let mut document = source.document().clone();
        let Control::Table(owner) = &mut document.sections[0].paragraphs[842].controls[0] else {
            panic!("owner table");
        };
        owner.padding.left = table_lr.0;
        owner.padding.right = table_lr.1;
        owner.cells[9].padding.left = cell_lr.0;
        owner.cells[9].padding.right = cell_lr.1;
        owner.cells[9].apply_inner_margin = apply;
        let source_width = child(owner).common.width;
        let source_cell_width = child(owner).cells[0].width;
        let control_index = owner.cells[9].paragraphs[0]
            .controls
            .iter()
            .position(|control| matches!(control, Control::Table(_)))
            .expect("nested control index (field controls may precede it)");
        let overlay = RenderNormalizationOverlay::from_document(&document);
        let projection = overlay
            .projection_for_path(&RenderPath {
                section_index: 0,
                parent_paragraph_index: 842,
                entries: vec![RenderPathEntry::TableCell {
                    control_index: 0,
                    cell_index: 9,
                    paragraph_index: 0,
                }],
                target_control_index: Some(control_index),
            })
            .expect("owner projection");
        assert_eq!(
            projection.effective_width, expected,
            "{table_lr:?}/{cell_lr:?}/{apply}"
        );
        assert!(
            (projection.width_scale - f64::from(expected) / f64::from(source_width)).abs() < 1e-9
        );
        let Control::Table(owner) = &document.sections[0].paragraphs[842].controls[0] else {
            unreachable!();
        };
        assert_eq!(child(owner).common.width, source_width);
        assert_eq!(child(owner).cells[0].width, source_cell_width);
    }
}

fn find_owner(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(842)) {
        return Some(node);
    }
    node.children.iter().find_map(find_owner)
}

fn check_child(node: &RenderNode, host_right: f64) -> usize {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.row_count == 1 && t.col_count == 1) {
        // 38245HU border box minus the two 510HU table-default margins.
        let content_width = 37_225.0 / 75.0;
        assert!(
            (node.bbox.width - content_width).abs() < 0.02,
            "child {:?}",
            node.bbox
        );
        assert!(node.bbox.x + node.bbox.width <= host_right - 510.0 / 75.0 + 0.02);
        let mut count = 0;
        for cell in &node.children {
            if !matches!(cell.node_type, RenderNodeType::TableCell(_)) {
                continue;
            }
            for line in &cell.children {
                if matches!(line.node_type, RenderNodeType::TextLine(_)) {
                    assert!(line.bbox.x >= cell.bbox.x - 0.02);
                    assert!(
                        line.bbox.x + line.bbox.width <= cell.bbox.x + cell.bbox.width + 0.02,
                        "line {:?} exceeds cell {:?}",
                        line.bbox,
                        cell.bbox
                    );
                    count += 1;
                }
            }
        }
        assert!(count > 0, "actual composed lines required");
        return 1;
    }
    node.children
        .iter()
        .map(|n| check_child(n, host_right))
        .sum()
}

#[test]
fn split_child_lines_and_clip_share_the_host_content_width() {
    let document = core();
    for page in [80, 81] {
        let tree = document
            .build_page_render_tree(page)
            .expect("render fragment");
        let owner = find_owner(&tree.root).expect("owner on both pages");
        let host_right = owner.bbox.x + owner.bbox.width;
        let count: usize = owner
            .children
            .iter()
            .map(|n| check_child(n, host_right))
            .sum();
        assert_eq!(
            count,
            1,
            "exactly one child fragment on physical page {}",
            page + 1
        );
    }
}

fn lines(node: &RenderNode, result: &mut Vec<(String, f64)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect::<String>();
        result.push((text, node.bbox.y));
    }
    for child in &node.children {
        lines(child, result);
    }
}

#[test]
fn continuation_owns_only_remaining_source_lines_without_relying_on_clip() {
    let document = core();
    let mut pages = Vec::new();
    for page in [80, 81] {
        let tree = document
            .build_page_render_tree(page)
            .expect("render fragment");
        let mut text = Vec::new();
        lines(find_owner(&tree.root).expect("owner"), &mut text);
        pages.push(text);
    }
    let previous = pages[0]
        .iter()
        .find(|(text, _)| text.contains("등의 사고"))
        .expect("first source line belongs to physical page 81");
    assert!(
        pages[1].iter().all(|(text, _)| text != &previous.0),
        "page 82 must not emit a previous-page line even outside the clip: {:?}",
        pages[1]
    );
    let next = pages[1]
        .iter()
        .find(|(text, _)| text.starts_with("를 예방함으로써"))
        .expect("continuation starts with the remaining source line");
    assert!(
        (next.1 - 77.1).abs() < 0.1,
        "preserve continuation placement: {next:?}"
    );
}

fn nested_child(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.row_count == 1 && t.col_count == 1) {
        return Some(node);
    }
    node.children.iter().find_map(nested_child)
}

fn without_whitespace(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn changed_page_budget_preserves_all_child_source_exactly_once() {
    // Synthetic contract variants, NOT Hancom fidelity or page-count oracles.
    // More room consumes two lines before continuing into the next paragraph;
    // the original consumes one; less room carries the whole child instead.
    for (bottom_delta, expected_fragment_lines) in
        [(-1600, vec![2, 6]), (0, vec![1, 7]), (800, vec![8])]
    {
        let mut core = core();
        let mut document = core.document().clone();
        let page = &mut document.sections[0].section_def.page_def;
        page.margin_bottom = page.margin_bottom.checked_add_signed(bottom_delta).unwrap();
        let Control::Table(owner) = &document.sections[0].paragraphs[842].controls[0] else {
            panic!("owner");
        };
        let expected = child(owner).cells[0]
            .paragraphs
            .iter()
            .map(|p| without_whitespace(&p.text))
            .collect::<String>();
        core.set_document(document);
        let mut actual = String::new();
        let mut fragment_lines = Vec::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).expect("render variant");
            let Some(owner) = find_owner(&tree.root) else {
                continue;
            };
            let Some(child) = nested_child(owner) else {
                continue;
            };
            let mut text = Vec::new();
            lines(child, &mut text);
            fragment_lines.push(text.len());
            for (line, _) in text {
                actual.push_str(&without_whitespace(&line));
            }
        }
        assert_eq!(
            actual, expected,
            "no lost/duplicated source, delta={bottom_delta}"
        );
        assert_eq!(
            fragment_lines, expected_fragment_lines,
            "exercise cut boundary, delta={bottom_delta}"
        );
    }
}
