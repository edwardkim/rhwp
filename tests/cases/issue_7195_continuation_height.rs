//! Fragment bounds and source preservation, not a fixed total-page oracle.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn find_body(n: &RenderNode) -> Option<&RenderNode> {
    if matches!(n.node_type, RenderNodeType::Body { .. }) {
        return Some(n);
    }
    n.children.iter().find_map(find_body)
}
fn find_table(n: &RenderNode) -> Option<&RenderNode> {
    if matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(15)) {
        return Some(n);
    }
    n.children.iter().find_map(find_table)
}
fn text(n: &RenderNode) -> String {
    if let RenderNodeType::TextRun(t) = &n.node_type {
        return t.text.clone();
    }
    n.children.iter().map(text).collect()
}
fn normalized(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn giant_continuation_nested_origin_preserves_owned_empty_lines() {
    // Source paragraphs 639..652 are real empty lines (1200HU line boxes,
    // 720HU line spacing), not zero-height controls. A paragraph-relative
    // nested table must not rewind over the lines owned by the same fragment.
    fn check(node: &RenderNode, body_bottom: f64) -> usize {
        let mut found = 0;
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            for (index, child) in node.children.iter().enumerate() {
                if !matches!(&child.node_type, RenderNodeType::Table(t) if t.para_index == Some(652))
                {
                    continue;
                }
                let lines: Vec<_> = node.children[..index]
                    .iter()
                    .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
                    .collect();
                if lines.is_empty() {
                    continue;
                }
                assert!(lines.iter().all(|line| normalized(&text(line)).is_empty()));
                let bottom = lines
                    .iter()
                    .map(|line| line.bbox.y + line.bbox.height)
                    .fold(f64::NEG_INFINITY, f64::max);
                assert!(
                    child.bbox.y >= bottom - 0.5,
                    "nested top {} rewinds over owned empty lines ending at {bottom}",
                    child.bbox.y
                );
                assert!(
                    child.bbox.y + child.bbox.height <= body_bottom + 0.5,
                    "nested frame exceeds physical body: {:?}, body bottom {body_bottom}",
                    child.bbox
                );
                found += 1;
            }
        }
        found
            + node
                .children
                .iter()
                .map(|child| check(child, body_bottom))
                .sum::<usize>()
    }
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/table_giant_cell_overfill.hwpx"),
    )
    .expect("sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let mut found = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render");
        let body = find_body(&tree.root).expect("body");
        found += check(body, body.bbox.y + body.bbox.height);
    }
    assert_eq!(
        found, 1,
        "exercise the fragment owning both empty lines and the nested table"
    );
}

#[test]
fn single_row_source_frame_keeps_its_last_line_without_overflow() {
    fn table43(n: &RenderNode) -> Option<&RenderNode> {
        if matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(43)) {
            return Some(n);
        }
        n.children.iter().find_map(table43)
    }
    fn paragraph8(n: &RenderNode) -> String {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            return if run.para_index == Some(8) {
                run.text.clone()
            } else {
                String::new()
            };
        }
        n.children.iter().map(paragraph8).collect()
    }
    fn check_lines(n: &RenderNode, top: f64, bottom: f64) {
        if matches!(n.node_type, RenderNodeType::TextLine(_)) {
            assert!(n.bbox.y >= top - 0.5);
            assert!(n.bbox.y + n.bbox.height <= bottom + 0.5, "{n:?}");
        }
        for child in &n.children {
            check_lines(child, top, bottom);
        }
    }
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/86712_regulatory_analysis.hwp"),
    )
    .expect("sample");
    for bottom_delta in [0, 1200] {
        let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
        if bottom_delta != 0 {
            let mut doc = core.document().clone();
            doc.sections[0].section_def.page_def.margin_bottom += bottom_delta;
            core.set_document(doc);
        }
        let Control::Table(table) = &core.document().sections[0].paragraphs[43].controls[0] else {
            panic!("table");
        };
        assert_eq!((table.row_count, table.col_count), (1, 1));
        let para = &table.cells[0].paragraphs[8];
        assert!(para.controls.is_empty());
        assert!(para.line_segs[0].vertical_pos > 0);
        assert_eq!(para.line_segs[1].vertical_pos, 0);
        let source: Vec<_> = para.text.encode_utf16().collect();
        let boundary = para.line_segs[1].text_start as usize;
        let first = normalized(&String::from_utf16(&source[..boundary]).unwrap());
        let rest = normalized(&String::from_utf16(&source[boundary..]).unwrap());
        let mut actual = Vec::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).expect("render");
            let Some(table) = table43(&tree.root) else {
                continue;
            };
            let body = find_body(&tree.root).expect("body").bbox;
            let bottom = body.y + body.height;
            assert!(table.bbox.y >= body.y - 0.5);
            assert!(table.bbox.y + table.bbox.height <= bottom + 0.5);
            check_lines(table, body.y, bottom);
            actual.push(normalized(&paragraph8(table)));
        }
        assert_eq!(
            actual.concat(),
            normalized(&para.text),
            "source exactly once"
        );
        if bottom_delta == 0 {
            // The actual saved input puts its first line before the reset.
            // No absolute page number or total page count is the oracle.
            assert_eq!(actual, vec![first, rest], "stored frame line ownership");
        } else {
            // Less room must still respect physical bounds, not force the old cut.
            assert!(actual.first().is_some_and(String::is_empty));
        }
    }
}

#[test]
fn stored_text_frames_keep_their_last_line_inside_the_body() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/86712_regulatory_analysis.hwp"),
    )
    .expect("sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let Control::Table(table) = &core.document().sections[0].paragraphs[15].controls[0] else {
        panic!("table");
    };
    let cell = table
        .cells
        .iter()
        .find(|c| c.row == 1 && c.col == 1)
        .unwrap();
    // Independent source LineSeg resets, not a renderer-generated page count.
    // This fixture's plain-text stream records both intra/inter-paragraph cuts.
    let mut expected = vec![String::new()];
    let mut previous_vpos = 0;
    for para in &cell.paragraphs {
        assert!(para.controls.is_empty());
        let utf16: Vec<_> = para.text.encode_utf16().collect();
        for (i, seg) in para.line_segs.iter().enumerate() {
            assert_eq!(
                seg.tag & rhwp::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                0
            );
            if previous_vpos > 0 && seg.vertical_pos <= 0 {
                expected.push(String::new());
            }
            let start = (seg.text_start as usize).min(utf16.len());
            let end = para
                .line_segs
                .get(i + 1)
                .map_or(utf16.len(), |next| next.text_start as usize)
                .min(utf16.len());
            expected.last_mut().unwrap().push_str(&normalized(
                &String::from_utf16(&utf16[start..end]).expect("source line"),
            ));
            previous_vpos = seg.vertical_pos;
        }
    }
    let mut actual = Vec::new();
    for p in 0..core.page_count() {
        let tree = core.build_page_render_tree(p).expect("render");
        let Some(table) = find_table(&tree.root) else {
            continue;
        };
        let cell = table.children.iter().find(|n| {
            matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 1 && c.col == 1)
        }).expect("body cell");
        actual.push(normalized(&text(cell)));
    }
    assert!(expected.len() > 3, "exercise repeated page boundaries");
    assert_eq!(actual.len(), expected.len(), "source frame ownership");
    for (i, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(
            actual,
            expected,
            "source frame {} last-line ownership",
            i + 1
        );
    }
}

#[test]
fn continued_text_table_fits_body_and_preserves_each_cell_source() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/86712_regulatory_analysis.hwp"),
    )
    .expect("sample");
    for bottom_delta in [0, 1200] {
        let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
        if bottom_delta != 0 {
            let mut doc = core.document().clone();
            doc.sections[0].section_def.page_def.margin_bottom += bottom_delta;
            core.set_document(doc);
        }
        let Control::Table(t) = &core.document().sections[0].paragraphs[15].controls[0] else {
            panic!("table");
        };
        let mut expected = [String::new(), String::new()];
        for c in &t.cells {
            if c.row == 1 {
                expected[c.col as usize] =
                    c.paragraphs.iter().map(|p| normalized(&p.text)).collect();
            }
        }
        let mut actual = [String::new(), String::new()];
        let mut fragments = 0;
        for p in 0..core.page_count() {
            let tree = core.build_page_render_tree(p).expect("render");
            let Some(t) = find_table(&tree.root) else {
                continue;
            };
            fragments += 1;
            let b = find_body(&tree.root).expect("body").bbox;
            assert!(
                t.bbox.y + t.bbox.height <= b.y + b.height + 0.5,
                "p{} delta={} table={:?} body={:?}",
                p + 1,
                bottom_delta,
                t.bbox,
                b
            );
            for c in &t.children {
                if let RenderNodeType::TableCell(cell) = &c.node_type {
                    if cell.row == 1 {
                        actual[cell.col as usize].push_str(&normalized(&text(c)));
                    }
                }
            }
        }
        assert!(fragments > 1, "exercise continuation");
        assert_eq!(
            actual, expected,
            "each source cell exactly once; delta={bottom_delta}"
        );
    }
}
