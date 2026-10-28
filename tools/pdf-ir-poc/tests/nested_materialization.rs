use rhwp::model::{control::Control, document::Document, paragraph::Paragraph, table::Table};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp_pdf_oxide_poc::{compile_page_painted_rules, compile_page_tables};

fn synthetic(extra: &str) -> Vec<u8> {
    let content = format!(
        "{}\n{extra}",
        include_str!("../fixtures/nested-grid.content")
    );
    let objects=[
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream",content.len()),
    ];
    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{obj}\nendobj\n", i + 1));
    }
    let xref = pdf.len();
    pdf.push_str("xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
    ));
    pdf.into_bytes()
}
fn input() -> Vec<u8> {
    let repo = std::env::var("RHWP_POC_REPO").unwrap();
    std::fs::read(
        std::path::Path::new(&repo).join("pdf/issue5875/nested_table_text_caption-2020.pdf"),
    )
    .unwrap()
}
fn tables(controls: &[Control]) -> Vec<&Table> {
    controls
        .iter()
        .filter_map(|c| {
            if let Control::Table(t) = c {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .collect()
}
fn children(t: &Table) -> Vec<&Table> {
    t.cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| tables(&p.controls))
        .collect()
}
fn roots(d: &Document) -> Vec<&Table> {
    d.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| tables(&p.controls))
        .collect()
}
fn compare(a: &Table, b: &Table) {
    assert_eq!(
        (a.row_count, a.col_count, a.cells.len()),
        (b.row_count, b.col_count, b.cells.len())
    );
    assert_eq!(a.cell_grid, b.cell_grid);
    assert_eq!(a.common.instance_id, b.common.instance_id);
    assert_eq!(a.outer_margin_left, b.outer_margin_left);
    assert_eq!(a.common.treat_as_char, b.common.treat_as_char);
    assert_eq!(a.common.flow_with_text, b.common.flow_with_text);
    if a.common.treat_as_char {
        assert!(
            a.common.flow_with_text,
            "nested table must follow its cell text"
        );
    }
    for (a, b) in a.cells.iter().zip(&b.cells) {
        assert_eq!(
            (a.row, a.col, a.row_span, a.col_span, a.width, a.height),
            (b.row, b.col, b.row_span, b.col_span, b.width, b.height)
        );
        assert_eq!(a.paragraphs.len(), b.paragraphs.len());
        assert_eq!(
            (
                a.padding.left,
                a.padding.right,
                a.padding.top,
                a.padding.bottom
            ),
            (
                b.padding.left,
                b.padding.right,
                b.padding.top,
                b.padding.bottom
            )
        );
        for (p, q) in a.paragraphs.iter().zip(&b.paragraphs) {
            assert_eq!(p.text, q.text);
            assert_eq!(p.controls.len(), q.controls.len());
            assert_eq!(p.line_segs.len(), q.line_segs.len());
            for (a, b) in p.controls.iter().zip(&q.controls) {
                if let (Control::Shape(a), Control::Shape(b)) = (a, b) {
                    assert!(a.common().flow_with_text);
                    assert!(b.common().flow_with_text);
                    assert_eq!(a.common().horizontal_offset, b.common().horizontal_offset);
                    assert_eq!(a.common().vertical_offset, b.common().vertical_offset);
                }
            }
        }
    }
    let (a, b) = (children(a), children(b));
    assert_eq!(a.len(), b.len());
    for (a, b) in a.into_iter().zip(b) {
        compare(a, b);
    }
}
fn collect_text(p: &Paragraph, out: &mut String) {
    out.push_str(&p.text);
    for c in &p.controls {
        match c {
            Control::Table(t) => {
                for cell in &t.cells {
                    for p in &cell.paragraphs {
                        collect_text(p, out);
                    }
                }
            }
            Control::Shape(s) => {
                if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                    for p in &tb.paragraphs {
                        collect_text(p, out);
                    }
                }
            }
            _ => {}
        }
    }
}
fn text(d: &Document) -> String {
    let mut out = String::new();
    for s in &d.sections {
        for p in &s.paragraphs {
            collect_text(p, &mut out);
        }
    }
    out
}
fn chars(s: &str) -> Vec<char> {
    let mut chars: Vec<_> = s.chars().filter(|c| !c.is_whitespace()).collect();
    chars.sort_unstable();
    chars
}
fn render_relations(node: &RenderNode, owner: Option<BoundingBox>, count: &mut usize) {
    let mut owner = owner;
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        *count += 1;
        if let Some(cell) = owner {
            assert!(
                node.bbox.x > cell.x,
                "nested leading inset must survive paint"
            );
            assert!(node.bbox.y >= cell.y - 0.5);
            assert!(node.bbox.x + node.bbox.width <= cell.x + cell.width + 0.5);
            assert!(node.bbox.y + node.bbox.height <= cell.y + cell.height + 0.5);
        }
        owner = None;
    }
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        owner = Some(node.bbox);
    }
    for child in &node.children {
        render_relations(child, owner, count);
    }
}

#[test]
fn three_level_tables_are_real_controls_after_serialization_and_paint() {
    let c = compile_page_tables(&synthetic(""), 0).unwrap();
    let loaded = rhwp::parse_document(&c.hwpx).unwrap();
    assert_eq!(c.report["lowering"]["semanticTables"], 3);
    assert_eq!(c.report["lowering"]["semanticNestedTables"], 2);
    let (a, b) = (roots(&c.document), roots(&loaded));
    assert_eq!(b.len(), 1);
    compare(a[0], b[0]);
    let child = children(b[0]);
    assert_eq!(child.len(), 1);
    let grandchild = children(child[0]);
    assert_eq!(grandchild.len(), 1);
    assert!(child[0].common.treat_as_char && grandchild[0].common.treat_as_char);
    assert_eq!(chars(&text(&loaded)), vec!['A', 'B', 'C']);
    let core = rhwp::DocumentCore::from_bytes(&c.hwpx).unwrap();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut count = 0;
    render_relations(&tree.root, None, &mut count);
    assert_eq!(count, 3);
}

#[test]
fn real_nested_pages_preserve_cell_ownership_text_and_unique_tables() {
    let bytes = input();
    // The source HWP and reference PDF each have four inner tables on these selected pages.
    for (page, nested) in [(6, 4), (7, 4)] {
        let c = compile_page_tables(&bytes, page).unwrap();
        let loaded = rhwp::parse_document(&c.hwpx).unwrap();
        assert_eq!(c.report["lowering"]["semanticNestedTables"], nested);
        assert_eq!(
            c.report["lowering"]["deferredTables"],
            serde_json::json!([])
        );
        let (a, b) = (roots(&c.document), roots(&loaded));
        assert_eq!(b.len(), 1);
        compare(a[0], b[0]);
        assert_eq!(children(b[0]).len(), nested);
        let source: String = c.report["extraction"]["spans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["text"].as_str().unwrap())
            .collect();
        assert_eq!(
            chars(&source),
            chars(&text(&loaded)),
            "no omitted or duplicated text"
        );
        let core = rhwp::DocumentCore::from_bytes(&c.hwpx).unwrap();
        assert_eq!(core.page_count(), 1);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut count = 0;
        render_relations(&tree.root, None, &mut count);
        assert_eq!(count, nested + 1, "no duplicate paint");
    }
}

#[test]
fn failed_descendant_rolls_back_the_entire_family_and_style_pools() {
    let bytes = synthetic("1 0 0 RG 25 165 m 35 165 l S");
    let baseline = compile_page_painted_rules(&bytes, 0).unwrap();
    let c = compile_page_tables(&bytes, 0).unwrap();
    assert_eq!(
        c.report["extraction"]["tableDetection"]["tables"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(c.report["lowering"]["semanticTables"], 0);
    assert_eq!(
        c.hwpx, baseline.hwpx,
        "all original text and rules must survive failure"
    );
    assert_eq!(
        c.document.doc_info.para_shapes.len(),
        baseline.document.doc_info.para_shapes.len()
    );
    assert_eq!(
        c.document.doc_info.border_fills.len(),
        baseline.document.doc_info.border_fills.len()
    );
    assert_eq!(
        c.document.doc_info.tab_defs.len(),
        baseline.document.doc_info.tab_defs.len()
    );
    assert_eq!(
        c.report["lowering"]["deferredTables"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn real_pdf_lines_use_editable_cell_text_and_physical_spacing() {
    fn inspect(table: &Table, doc: &Document, tabs: &mut usize) {
        for cell in &table.cells {
            for p in &cell.paragraphs {
                if p.text.is_empty() && p.controls.is_empty() {
                    continue;
                }
                let ps = &doc.doc_info.para_shapes[p.para_shape_id as usize];
                for measure in [
                    ps.margin_left,
                    ps.margin_right,
                    ps.indent,
                    ps.spacing_before,
                    ps.spacing_after,
                ] {
                    assert_eq!(measure % 2, 0, "PDF points must not use CHAR units");
                }
                assert_eq!(
                    ps.attr1 & (1 << 8),
                    0,
                    "PDF line positions do not snap to a document grid"
                );
                assert_eq!(p.text.matches('\t').count(), p.tab_extended.len());
                for extension in &p.tab_extended {
                    assert!(
                        !rhwp::model::paragraph::tab_ext_is_placeholder(extension),
                        "Hancom requires a real PDF span gap, not a zero-width marker"
                    );
                    *tabs += 1;
                }
                for control in &p.controls {
                    match control {
                        Control::Table(t) => inspect(t, doc, tabs),
                        Control::Picture(_) => {}
                        _ => {
                            panic!("PDF lines must remain editable text, not floating cell shapes")
                        }
                    }
                }
            }
        }
    }
    for page in [6, 7] {
        let c = compile_page_tables(&input(), page).unwrap();
        let loaded = rhwp::parse_document(&c.hwpx).unwrap();
        let mut before_tabs = 0;
        let mut after_tabs = 0;
        inspect(roots(&c.document)[0], &c.document, &mut before_tabs);
        inspect(roots(&loaded)[0], &loaded, &mut after_tabs);
        assert!(before_tabs > 0);
        assert_eq!(before_tabs, after_tabs);
        assert_eq!(chars(&text(&c.document)), chars(&text(&loaded)));
    }
}

#[test]
fn nested_cell_text_is_editable_and_reserializes_without_flattening() {
    let c = compile_page_tables(&synthetic(""), 0).unwrap();
    let mut doc = rhwp::parse_document(&c.hwpx).unwrap();
    let Control::Table(root) = doc.sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find(|c| matches!(c, Control::Table(_)))
        .unwrap()
    else {
        unreachable!()
    };
    fn replace(t: &mut Table) -> bool {
        for cell in &mut t.cells {
            for p in &mut cell.paragraphs {
                if p.text == "C" {
                    p.delete_text_at(0, 1);
                    p.insert_text_at(0, "D");
                    return true;
                }
                for c in &mut p.controls {
                    if let Control::Table(t) = c {
                        if replace(t) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    assert!(replace(root));
    let hwpx = rhwp::serializer::serialize_hwpx(&doc).unwrap();
    let loaded = rhwp::parse_document(&hwpx).unwrap();
    assert_eq!(chars(&text(&loaded)), vec!['A', 'B', 'D']);
    let child = children(roots(&loaded)[0]);
    assert_eq!(children(child[0]).len(), 1);
}
