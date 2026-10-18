//! Independent source geometry, actual nested owners and final paint.
//! Qualification / before FAIL and after PASS: task_m100_7998_stage4.md.
#![cfg(feature = "pdf-import")]

use rhwp::{
    model::{control::Control, paragraph::Paragraph, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn base() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998")
}
fn table_count(controls: &[Control]) -> usize {
    controls
        .iter()
        .map(|c| {
            if let Control::Table(t) = c {
                1 + t
                    .cells
                    .iter()
                    .flat_map(|c| &c.paragraphs)
                    .map(|p| table_count(&p.controls))
                    .sum::<usize>()
            } else {
                0
            }
        })
        .sum()
}
fn model_text(paras: &[Paragraph], out: &mut Vec<char>) {
    for p in paras {
        out.extend(p.text.chars().filter(|c| !c.is_whitespace()));
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &t.cells {
                        model_text(&cell.paragraphs, out)
                    }
                }
                Control::Shape(s) => {
                    if let Some(t) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        model_text(&t.paragraphs, out)
                    }
                }
                _ => {}
            }
        }
    }
}
fn paint(n: &RenderNode, out: &mut Vec<char>, length: &mut f64) {
    match &n.node_type {
        RenderNodeType::TextRun(t) => out.extend(t.text.chars().filter(|c| !c.is_whitespace())),
        RenderNodeType::Rectangle(r) if r.style.fill_color == Some(0) => {
            *length += n.bbox.width * n.bbox.height * (72. / 96.) * (72. / 96.)
        }
        _ => {}
    }
    for c in &n.children {
        paint(c, out, length);
    }
}
fn own_flow(
    n: &RenderNode,
    tables: &mut Vec<(f64, f64)>,
    images: &mut Vec<(f64, f64, f64, f64)>,
    lines: &mut Vec<(String, f64, f64)>,
) {
    if matches!(n.node_type, RenderNodeType::Table(_)) {
        tables.push((n.bbox.y, n.bbox.y + n.bbox.height));
        return;
    }
    if matches!(n.node_type, RenderNodeType::Image(_)) {
        images.push((n.bbox.x, n.bbox.y, n.bbox.width, n.bbox.height));
    }
    if matches!(n.node_type, RenderNodeType::TextLine(_)) {
        let text: String = n
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNodeType::TextRun(t) = &c.node_type {
                    Some(t.text.as_str())
                } else {
                    None
                }
            })
            .collect();
        if !text.is_empty() {
            lines.push((text, n.bbox.y, n.bbox.y + n.bbox.height));
        }
    }
    for c in &n.children {
        own_flow(c, tables, images, lines);
    }
}
fn order(n: &RenderNode) {
    if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        let (mut tables, mut images, mut lines) = (Vec::new(), Vec::new(), Vec::new());
        for c in &n.children {
            own_flow(c, &mut tables, &mut images, &mut lines);
        }
        if !tables.is_empty() {
            let top = tables.iter().map(|t| t.0).fold(f64::INFINITY, f64::min);
            let bottom = tables.iter().map(|t| t.1).fold(f64::NEG_INFINITY, f64::max);
            for (text, y, end) in &lines {
                if text.trim() == "BEFORE CHILD" {
                    assert!(
                        *end <= top + 0.5,
                        "before paragraph remains before child table"
                    );
                }
                if text.trim() == "AFTER CHILD" {
                    assert!(
                        bottom <= *y + 0.5,
                        "after paragraph remains after child table"
                    );
                }
            }
        }
        for (_, y, end) in &lines {
            for (_, iy, _, ih) in &images {
                assert!(
                    *end <= *iy + 0.5 || *y + 0.5 >= iy + ih,
                    "text and cell-owned picture may not overlap"
                );
            }
        }
    }
    for c in &n.children {
        order(c);
    }
}
fn geometry(n: &RenderNode, cell: Option<(f64, f64, f64, f64)>) {
    let b = &n.bbox;
    let c = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        Some((b.x, b.y, b.width, b.height))
    } else {
        cell
    };
    if matches!(
        n.node_type,
        RenderNodeType::Table(_) | RenderNodeType::Image(_) | RenderNodeType::TextRun(_)
    ) {
        if let Some((x, y, w, h)) = cell {
            assert!(
                b.x + 0.5 >= x
                    && b.y + 0.5 >= y
                    && b.x + b.width <= x + w + 0.5
                    && b.y + b.height <= y + h + 0.5,
                "nested content remains in its physical cell: {:?} vs {:?}",
                b,
                cell
            );
        }
    }
    for ch in &n.children {
        geometry(ch, c);
    }
}
fn verify(core: &DocumentCore, tables: usize, expected: &[char], line_length: f64) {
    assert_eq!(core.page_count(), 1);
    assert_eq!(
        table_count(&core.document().sections[0].paragraphs[0].controls),
        tables,
        "real nested Table owners, not diagnostic candidates"
    );
    let mut model = Vec::new();
    model_text(&core.document().sections[0].paragraphs, &mut model);
    model.sort_unstable();
    assert_eq!(
        model, expected,
        "source characters belong to the model once"
    );
    let tree = core.build_page_render_tree(0).unwrap();
    let (mut chars, mut length) = (Vec::new(), 0.);
    paint(&tree.root, &mut chars, &mut length);
    chars.sort_unstable();
    assert_eq!(
        chars, expected,
        "every model character reaches actual paint once"
    );
    assert!(
        (length - line_length * 0.5).abs() < 0.5,
        "independent source stroke-quad area must be conserved: {length} vs {line_length}"
    );
    geometry(&tree.root, None);
    order(&tree.root);
}
fn picture_count(controls: &[Control]) -> usize {
    controls
        .iter()
        .map(|c| match c {
            Control::Picture(_) => 1,
            Control::Table(t) => t
                .cells
                .iter()
                .flat_map(|c| &c.paragraphs)
                .map(|p| picture_count(&p.controls))
                .sum(),
            _ => 0,
        })
        .sum()
}

fn expected(key: &str) -> Vec<char> {
    let mut strings = vec![
        "OUTSIDE ABOVE",
        "OUTSIDE AFTER",
        "UPPER RIGHT",
        "LOWER LEFT",
        "LOWER RIGHT",
        "AFTER CHILD",
    ];
    if !key.ends_with("first") {
        strings.push("BEFORE CHILD")
    }
    if key.ends_with("siblings") {
        strings.extend(["A", "B", "C", "D", "E", "F", "G", "H"])
    } else {
        strings.extend(["B", "C", "D"]);
        if !key.ends_with("image") {
            strings.push("A")
        }
    }
    if key.ends_with("deep") {
        strings.extend(["RIGHT", "BOTTOM", "END"])
    }
    let mut out: Vec<_> = strings
        .into_iter()
        .flat_map(str::chars)
        .filter(|c| !c.is_whitespace())
        .collect();
    out.sort_unstable();
    out
}
fn leaf_path(
    table: &Table,
    at: usize,
    target: &str,
    path: &mut Vec<(usize, usize, usize)>,
) -> bool {
    for (ci, cell) in table.cells.iter().enumerate() {
        for (pi, p) in cell.paragraphs.iter().enumerate() {
            path.push((at, ci, pi));
            if p.text == target {
                return true;
            }
            for (idx, c) in p.controls.iter().enumerate() {
                if let Control::Table(t) = c {
                    if leaf_path(t, idx, target, path) {
                        return true;
                    }
                }
            }
            path.pop();
        }
    }
    false
}
#[test]
fn native_periodic_families_keep_actual_cell_owners_paint_and_both_save_formats() {
    for (key, count, length) in [
        ("nested-periodic-child", 2, 2480.),
        ("nested-periodic-background", 2, 2480.),
        ("nested-periodic-parent", 2, 2250.),
        ("nested-periodic-both", 2, 2150.),
        ("nested-periodic-first", 2, 2480.),
        ("nested-periodic-image", 2, 2480.),
        ("nested-periodic-siblings", 3, 2830.),
        ("nested-periodic-deep", 3, 2955.),
    ] {
        let bytes = std::fs::read(base().join(format!("{key}.pdf"))).unwrap();
        let mut core = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
        let want = expected(key);
        verify(&core, count, &want, length);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify(
                &DocumentCore::from_bytes(&bytes).unwrap(),
                count,
                &want,
                length,
            )
        }
        let root = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        let Control::Table(t) = &core.document().sections[0].paragraphs[0].controls[root] else {
            panic!()
        };
        let mut path = Vec::new();
        assert!(leaf_path(
            t,
            root,
            if key.ends_with("image") { "B" } else { "A" },
            &mut path
        ));
        core.insert_text_in_cell_by_path(0, 0, &path, 0, "X")
            .unwrap();
        let mut changed = want.clone();
        changed.push('X');
        changed.sort_unstable();
        verify(&core, count, &changed, length);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify(
                &DocumentCore::from_bytes(&bytes).unwrap(),
                count,
                &changed,
                length,
            )
        }
        core.delete_text_in_cell_by_path(0, 0, &path, 0, 1).unwrap();
        verify(&core, count, &want, length);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify(
                &DocumentCore::from_bytes(&bytes).unwrap(),
                count,
                &want,
                length,
            );
        }
        println!("{key}: source, child edit, deletion, HWP/HWPX, actual paint and geometry passed");
    }
}

#[test]
fn parent_and_child_backgrounds_partition_the_same_source_area() {
    let core = DocumentCore::from_pdf_bytes(
        &std::fs::read(base().join("nested-periodic-background.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    fn area(n: &RenderNode, depth: usize, out: &mut [f64; 3]) {
        let depth = depth + usize::from(matches!(n.node_type, RenderNodeType::Table(_)));
        if let RenderNodeType::Rectangle(r) = &n.node_type {
            if r.style.fill_color.is_some_and(|c| c != 0 && c != 0xffffff) {
                assert!(depth < 3);
                out[depth] += n.bbox.width * n.bbox.height * 72. / 96. * 72. / 96.;
            }
        }
        for child in &n.children {
            area(child, depth, out);
        }
    }
    let saved = [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ];
    for c in std::iter::once(core).chain(saved.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
    {
        let mut observed = [0.; 3];
        area(&c.build_page_render_tree(0).unwrap().root, 0, &mut observed);
        // Independent source rectangles: background=170*140, enclosed child=140*60.
        for (actual, want) in observed
            .into_iter()
            .zip([0., 170. * 140. - 140. * 60., 140. * 60.])
        {
            assert!((actual-want).abs()<0.5,"colored area must have the correct parent/child physical owner: {actual} vs {want}");
        }
    }
}
#[test]
fn original_cell_picture_and_solid_nested_owners_remain_valid() {
    for (key, tables, pictures) in [
        ("cell-picture", 1, 1),
        ("cell-picture-first", 1, 1),
        ("cell-two-pictures", 1, 2),
        ("cell-nested", 2, 0),
    ] {
        let core = DocumentCore::from_pdf_bytes(
            &std::fs::read(base().join(format!("{key}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let first: Vec<&str> = match key {
            "cell-picture" => vec!["ALPHA BETA GAMMA DELTA", "", "AFTER PICTURE CONTINUED TEXT"],
            "cell-picture-first" => vec!["", "AFTER PICTURE CONTINUED TEXT"],
            "cell-two-pictures" => vec!["BEFORE", "", "", "AFTER"],
            _ => vec!["BEFORE CHILD", "", "AFTER CHILD"],
        };
        let mut strings = vec![
            "OUTSIDE ABOVE",
            "OUTSIDE AFTER",
            "UPPER RIGHT",
            "LOWER LEFT",
            "LOWER RIGHT",
        ];
        strings.extend(first.iter().copied());
        if key == "cell-nested" {
            strings.extend(["A", "B", "C", "D"]);
        }
        let mut wanted: Vec<_> = strings
            .into_iter()
            .flat_map(str::chars)
            .filter(|c| !c.is_whitespace())
            .collect();
        wanted.sort_unstable();
        let saved = [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ];
        for c in std::iter::once(core).chain(
            saved
                .iter()
                .map(|bytes| DocumentCore::from_bytes(bytes).unwrap()),
        ) {
            assert_eq!(
                table_count(&c.document().sections[0].paragraphs[0].controls),
                tables
            );
            assert_eq!(
                picture_count(&c.document().sections[0].paragraphs[0].controls),
                pictures
            );
            let root = c.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .find_map(|v| {
                    if let Control::Table(t) = v {
                        Some(t)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(
                root.cells[0]
                    .paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>(),
                first
            );
            let tree = c.build_page_render_tree(0).unwrap();
            let (mut chars, mut area) = (Vec::new(), 0.);
            paint(&tree.root, &mut chars, &mut area);
            chars.sort_unstable();
            assert_eq!(chars, wanted);
            geometry(&tree.root, None);
            order(&tree.root);
        }
    }
}
#[test]
fn crossing_and_interleaved_picture_owners_remain_unconsumed() {
    for (key, tail) in [
        ("cell-picture-crossing", "AFTER"),
        ("cell-picture-overlap", "OVERLAPPING"),
    ] {
        let core = DocumentCore::from_pdf_bytes(
            &std::fs::read(base().join(format!("{key}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let mut wanted: Vec<_> = [
            "OUTSIDE ABOVE",
            "OUTSIDE AFTER",
            "UPPER RIGHT",
            "LOWER LEFT",
            "LOWER RIGHT",
            "BEFORE",
            tail,
        ]
        .into_iter()
        .flat_map(str::chars)
        .filter(|c| !c.is_whitespace())
        .collect();
        wanted.sort_unstable();
        let saved = [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ];
        for c in std::iter::once(core).chain(
            saved
                .iter()
                .map(|bytes| DocumentCore::from_bytes(bytes).unwrap()),
        ) {
            assert_eq!(
                table_count(&c.document().sections[0].paragraphs[0].controls),
                0
            );
            assert_eq!(
                picture_count(&c.document().sections[0].paragraphs[0].controls),
                1
            );
            let tree = c.build_page_render_tree(0).unwrap();
            let (mut chars, mut area) = (Vec::new(), 0.);
            paint(&tree.root, &mut chars, &mut area);
            chars.sort_unstable();
            assert_eq!(
                chars, wanted,
                "deferred composition cannot omit or duplicate source text"
            );
        }
    }
}
