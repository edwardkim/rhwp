//! Saved picture bottom and successor LINESEG independently own the same boundary.
//! Hancom 2020 PDF physical 16: body baselines 974.72 and 1020.32 px at 96 dpi.
//! The reduced fixture retains layout; the full original also verifies the image.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::float_placement::stored_picture_successor_placement;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const REDUCED: &str = "samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp";
const FULL: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
fn bytes(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}
fn runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        runs(child, out);
    }
}
fn check_sample(path: &str) {
    let core = DocumentCore::from_bytes(&bytes(path)).unwrap();
    let mut found = false;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        runs(&tree.root, &mut nodes);
        if !nodes.iter().any(|node| {
            matches!(&node.node_type,
            RenderNodeType::TextRun(run) if run.text == "최종 표시도안(예시)")
        }) {
            continue;
        }
        let target = nodes.iter().find(|node| {
            matches!(&node.node_type,
            RenderNodeType::TextRun(run) if run.text.contains("규제영향분석서 작성"))
        });
        let Some(target) = target else {
            continue;
        };
        found = true;
        let RenderNodeType::TextRun(run) = &target.node_type else {
            unreachable!()
        };
        assert!(
            (target.bbox.y + run.baseline - 1020.32).abs() < 1.0,
            "Hancom PDF baseline: {:?}, baseline {}",
            target.bbox,
            run.baseline
        );
        let preceding = nodes.iter().find(|node| matches!(&node.node_type,
            RenderNodeType::TextRun(run) if run.text.contains("안전확인대상생활화학제품 및 살생물제품 표시기준"))).unwrap();
        let RenderNodeType::TextRun(run) = &preceding.node_type else {
            unreachable!()
        };
        assert!((preceding.bbox.y + run.baseline - 974.72).abs() < 1.0);
    }
    assert!(found, "content-matched PDF page must exist");
}
#[test]
fn issue_7048_reduced_body_baselines_match_hancom() {
    check_sample(REDUCED);
}
#[test]
fn issue_7048_full_original_body_baselines_match_hancom() {
    check_sample(FULL);
}

#[test]
fn issue_7048_saved_picture_boundary_requires_independent_source_agreement() {
    let doc = rhwp::parse_document(&bytes(REDUCED)).unwrap();
    let para = &doc.sections[0].paragraphs[147];
    let next = &doc.sections[0].paragraphs[148];
    let resolve = |p: &rhwp::model::paragraph::Paragraph, n: &rhwp::model::paragraph::Paragraph| {
        stored_picture_successor_placement(p, n, 8.0, 20.0, 0, 96.0)
    };
    let placement = resolve(para, next).expect("frame bottom equals next stored line");
    assert!((placement.occupied_bottom - 796.0).abs() < 0.001);
    assert!((placement.paragraph_end(900.0, 0.0) - 776.0).abs() < 0.001);
    let mut stale = next.clone();
    stale.line_segs[0].vertical_pos += 75;
    assert!(resolve(para, &stale).is_none());
    let mut text_successor = next.clone();
    text_successor.text = "actual following text owns its own line flow".into();
    assert!(resolve(para, &text_successor).is_none());
    assert!(stored_picture_successor_placement(para, next, 8.0, 0.0, 0, 96.0).is_none());
    let mut broken = para.clone();
    broken.line_segs.clear();
    assert!(resolve(&broken, next).is_none());
    let mut synthetic = para.clone();
    synthetic.line_segs[0].tag |= 0x8000_0000;
    assert!(resolve(&synthetic, next).is_none());
    let mut text = para.clone();
    text.text = "visible host".into();
    assert!(resolve(&text, next).is_none());
    let mut beside = para.clone();
    beside.line_segs[0].segment_width = 10000;
    assert!(resolve(&beside, next).is_none());
    let mut page_break = next.clone();
    page_break.column_type = rhwp::model::paragraph::ColumnBreakType::Page;
    assert!(resolve(para, &page_break).is_none());
    let mut inline = para.clone();
    if let rhwp::model::control::Control::Picture(p) = &mut inline.controls[0] {
        p.common.treat_as_char = true;
    }
    assert!(resolve(&inline, next).is_none());
}

#[test]
fn issue_7048_cellbreak_boundary_closes_both_saved_fragments() {
    use rhwp::model::control::Control;
    use rhwp::renderer::float_placement::stored_cellbreak_fragment_row_end;
    let doc = rhwp::parse_document(&bytes(REDUCED)).unwrap();
    let host = &doc.sections[2].paragraphs[162];
    let next = &doc.sections[2].paragraphs[163];
    let Control::Table(table) = &host.controls[0] else {
        panic!("source table")
    };
    assert_eq!(
        stored_cellbreak_fragment_row_end(table, host, next),
        Some(22)
    );
    let mut stale = next.clone();
    stale.line_segs[0].vertical_pos += 1;
    assert!(stored_cellbreak_fragment_row_end(table, host, &stale).is_none());
    let mut changed = table.clone();
    changed.common.height += 1;
    assert!(stored_cellbreak_fragment_row_end(&changed, host, next).is_none());
    let mut no_repeat = table.clone();
    no_repeat.repeat_header = false;
    assert!(stored_cellbreak_fragment_row_end(&no_repeat, host, next).is_none());
    let mut crossed = table.clone();
    crossed
        .cells
        .iter_mut()
        .find(|cell| cell.row == 21)
        .unwrap()
        .row_span = 2;
    assert!(stored_cellbreak_fragment_row_end(&crossed, host, next).is_none());
}

#[test]
fn issue_7048_table_first_fragment_keeps_the_footer_clear() {
    fn tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if table.section_index == Some(2) && table.para_index == Some(162))
        {
            out.push(node);
        }
        for child in &node.children {
            tables(child, out);
        }
    }
    for path in [REDUCED, FULL] {
        let core = DocumentCore::from_bytes(&bytes(path)).unwrap();
        let mut fragment_count = 0;
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut fragments = Vec::new();
            tables(&tree.root, &mut fragments);
            for table in fragments {
                if fragment_count == 0 {
                    let last_row = table
                        .children
                        .iter()
                        .filter_map(|cell| match &cell.node_type {
                            RenderNodeType::TableCell(c) => Some(c.row + c.row_span - 1),
                            _ => None,
                        })
                        .max()
                        .unwrap();
                    assert_eq!(last_row, 21, "Hancom PDF 57 ends before 품목별 특성 row");
                    assert!(
                        (table.bbox.y + table.bbox.height - 1006.6).abs() < 1.0,
                        "Hancom first fragment bottom: {:?}",
                        table.bbox
                    );
                }
                fragment_count += 1;
            }
        }
        assert_eq!(fragment_count, 2);
    }
}
