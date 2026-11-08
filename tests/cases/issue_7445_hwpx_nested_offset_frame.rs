//! 저장 HWPX의 문단 오프셋·온전한 자식 클립·내용에 맞춘 머리행과 총높이 계약.
//! 독립 한컴 Print: issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf, 115쪽.
//! 4쪽의 유일한 양의 문단 오프셋 표를 보호한다. 좌표는 PDF pt × 96/72, ±2 CSSpx.
//! 전쪽 Native/fresh WASM 및 확대 판독 선행 조건은 증적 README에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn walk<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        walk(c, out);
    }
}
#[test]
fn a_stored_hwpx_host_keeps_the_nested_tables_own_paragraph_offset() {
    let mut d = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/issue1949_giant_cell_nested_tables_perf.hwpx"
        ))
        .unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(false);
    assert_eq!(d.page_count(), 115);
    let Control::Table(t) = &d.document().sections[0].paragraphs[0].controls[2] else {
        panic!()
    };
    let c = t.cells.iter().find(|c| c.row == 2 && c.col == 0).unwrap();
    let p = &c.paragraphs[81];
    let previous = &c.paragraphs[80];
    assert!(p.text.trim().is_empty() && previous.text.trim().is_empty());
    assert_eq!(previous.line_segs.len(), 1);
    let Control::Table(ch) = &p.controls[0] else {
        panic!()
    };
    assert!(!ch.common.treat_as_char);
    assert!(ch.common.vertical_offset > 0);
    let slot = &previous.line_segs[0];
    let source_advance = slot.line_height
        + slot.line_spacing
        + ch.common.vertical_offset as i32
        + i32::from(ch.outer_margin_top);
    let expected = f64::from(source_advance) * 96.0 / 7200.0;
    let tree = d.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables=nodes.iter().filter(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(81)&&t.control_index==Some(0))).collect::<Vec<_>>();
    assert_eq!(tables.len(), 1);
    let lines = nodes
        .iter()
        .filter(|n| matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(80)))
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1);
    let actual = tables[0].bbox.y - lines[0].bbox.y;
    assert!((actual-expected).abs()<=2.0*96.0/7200.0,"stored host line advance + own paragraph offset + top margin: actual={actual}, source={expected}");
}
#[test]
fn the_complete_source_child_is_visible_through_every_ancestor_clip() {
    let mut d = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/issue1949_giant_cell_nested_tables_perf.hwpx"
        ))
        .unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(false);
    let tree = d.build_page_render_tree(3).unwrap();
    fn visit(n: &RenderNode, clips: &mut Vec<f64>, found: &mut usize) {
        let clips_here = matches!(&n.node_type,RenderNodeType::TableCell(c)if c.clip);
        if clips_here {
            clips.push(n.bbox.y + n.bbox.height);
        }
        if matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(81)&&t.control_index==Some(0))
        {
            *found += 1;
            let bottom = n.bbox.y + n.bbox.height;
            assert!(!clips.is_empty());
            for clip in clips.iter() {
                assert!(
                    *clip + 1e-7 >= bottom,
                    "accepted complete child end={bottom}, ancestor clip end={clip}"
                );
            }
            let last = n
                .children
                .iter()
                .filter(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==2))
                .collect::<Vec<_>>();
            assert_eq!(last.len(), 3);
            assert!(last
                .iter()
                .all(|n| n.bbox.y + n.bbox.height <= bottom + 1e-7));
        }
        for c in &n.children {
            visit(c, clips, found);
        }
        if clips_here {
            clips.pop();
        }
    }
    let mut found = 0;
    visit(&tree.root, &mut Vec::new(), &mut found);
    assert_eq!(found, 1);
    let Control::Table(parent) = &d.document().sections[0].paragraphs[0].controls[2] else {
        panic!("source parent")
    };
    let host = parent
        .cells
        .iter()
        .find(|c| c.row == 2 && c.col == 0)
        .unwrap();
    let Control::Table(source_child) = &host.paragraphs[81].controls[0] else {
        panic!("source child")
    };
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let child = nodes.iter().find(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(81) && t.control_index == Some(0))).unwrap();
    for source_cell in source_child.cells.iter().filter(|c| c.row == 2) {
        let painted = child.children.iter().find(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == source_cell.row && c.col == source_cell.col)).unwrap();
        let mut content_nodes = Vec::new();
        walk(painted, &mut content_nodes);
        let actual: String = content_nodes
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .flat_map(str::chars)
            .filter(|c| !c.is_whitespace())
            .collect();
        let expected: String = source_cell
            .paragraphs
            .iter()
            .flat_map(|p| p.text.chars())
            .filter(|c| !c.is_whitespace())
            .collect();
        assert!(!expected.is_empty());
        assert_eq!(
            actual, expected,
            "last row text order/ownership, without loss or duplication"
        );
    }
    let next = d.build_page_render_tree(4).unwrap();
    let mut next_nodes = Vec::new();
    walk(&next.root, &mut next_nodes);
    assert!(!next_nodes.iter().any(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(81)&&t.control_index==Some(0))),"complete source child must not repeat on the next page");
}
#[test]
fn content_fitted_headers_keep_the_declared_whole_table_height() {
    let mut d = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/issue1949_giant_cell_nested_tables_perf.hwpx"
        ))
        .unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(false);
    let Control::Table(t) = &d.document().sections[0].paragraphs[0].controls[2] else {
        panic!()
    };
    let c = t.cells.iter().find(|c| c.row == 2 && c.col == 0).unwrap();
    let Control::Table(ch) = &c.paragraphs[81].controls[0] else {
        panic!()
    };
    let tree = d.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables=nodes.iter().filter(|n|matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(81)&&t.control_index==Some(0))).collect::<Vec<_>>();
    assert_eq!(tables.len(), 1);
    let rendered = tables[0];
    let expected = f64::from(ch.common.height) * 96.0 / 7200.0;
    assert!(
        (rendered.bbox.height - expected).abs() <= 2.0 * 96.0 / 7200.0,
        "independent original whole height: actual={},source={expected}",
        rendered.bbox.height
    );
    for row in [0u16, 1] {
        let cell = ch
            .cells
            .iter()
            .find(|c| c.row == row && c.row_span == 1)
            .unwrap();
        let line = &cell.paragraphs[0].line_segs[0];
        let pad = cell.effective_padding(&ch.padding);
        let floor = f64::from(line.line_height + i32::from(pad.top) + i32::from(pad.bottom)) * 96.0
            / 7200.0;
        let painted=rendered.children.iter().find(|n|matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==row&&c.col==cell.col)).unwrap();
        assert!(
            (painted.bbox.height - floor).abs() <= 2.0 * 96.0 / 7200.0,
            "header remains content fitted, no proportional shrinking"
        );
    }
}

#[test]
fn the_completed_offset_table_preserves_independent_print_top_and_bottom() {
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/issue1949_giant_cell_nested_tables_perf.hwpx"
        ))
        .unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 115);
    let tree = core.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables = nodes.iter().filter(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(81) && t.control_index == Some(0))).collect::<Vec<_>>();
    assert_eq!(tables.len(), 1);
    let table = tables[0];
    for (actual, expected) in [
        (table.bbox.y, 969.65869140625),
        (table.bbox.y + table.bbox.height, 1071.3080240885417),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "independent Print table boundary: actual={actual}, expected={expected}"
        );
    }
}
