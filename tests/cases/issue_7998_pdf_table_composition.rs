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

// These independent sources declare 0.5pt black strokes. A continuous part can
// now be a native cell border (0.2mm / five Hancom 600dpi ticks), rather than a
// filled quad. Preserve its centerline extent once, including shared edges;
// do not mistake the removal of the duplicate quad for missing source paint.
fn native_border_source_area(root: &RenderNode) -> f64 {
    use std::collections::BTreeMap;
    fn collect(n: &RenderNode, lines: &mut BTreeMap<(bool, i64), Vec<(i64, i64)>>) {
        if let RenderNodeType::Line(l) = &n.node_type {
            if !n.editor_only && l.style.color == 0 {
                assert!((l.style.width - 0.8).abs() < 0.01);
                let hu = |px: f64| (px * 72. / 96. * 100.).round() as i64;
                if (l.y1 - l.y2).abs() < 0.01 {
                    lines
                        .entry((true, hu(l.y1)))
                        .or_default()
                        .push((hu(l.x1.min(l.x2)), hu(l.x1.max(l.x2))));
                } else if (l.x1 - l.x2).abs() < 0.01 {
                    lines
                        .entry((false, hu(l.x1)))
                        .or_default()
                        .push((hu(l.y1.min(l.y2)), hu(l.y1.max(l.y2))));
                }
            }
        }
        for child in &n.children {
            collect(child, lines);
        }
    }
    let mut lines = BTreeMap::new();
    collect(root, &mut lines);
    let mut length = 0i64;
    for intervals in lines.values_mut() {
        intervals.sort_unstable();
        let mut end = i64::MIN;
        for &(a, b) in intervals.iter() {
            if b > end {
                length += b - a.max(end);
                end = b;
            }
        }
    }
    length as f64 / 100. * 0.5
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
    length += native_border_source_area(&tree.root);
    chars.sort_unstable();
    assert_eq!(
        chars, expected,
        "every model character reaches actual paint once"
    );
    assert!(
        (length - line_length * 0.5).abs() < 0.5,
        "independent source stroke extent must be conserved by quads or native borders: {length} vs {line_length}"
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
fn parent_background_is_one_native_fill_and_nested_content_keeps_its_owner() {
    let core = DocumentCore::from_pdf_bytes(
        &std::fs::read(base().join("nested-periodic-background.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    fn owners(core: &DocumentCore, controls: &[Control], depth: usize) -> usize {
        let mut count = 0;
        for control in controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    let fill = &core.document().doc_info.border_fills
                        [cell.border_fill_id as usize - 1]
                        .fill;
                    if fill.fill_type == rhwp::model::style::FillType::Solid {
                        if let Some(solid) = fill.solid.filter(|s| s.background_color != 0xffffff) {
                            // Independent PDF declares one 0.9/0.95/1 rectangle
                            // behind the parent; it does not declare a child fill.
                            assert_eq!(solid.background_color, 0xfff2e6);
                            assert_eq!((depth, cell.row, cell.col), (0, 0, 0));
                            count += 1;
                        }
                    }
                    for p in &cell.paragraphs {
                        count += owners(core, &p.controls, depth + 1);
                    }
                }
            }
        }
        count
    }
    fn background(n: &RenderNode, owner: Option<&RenderNode>) -> usize {
        let owner = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            Some(n)
        } else {
            owner
        };
        let mut count = 0;
        if let RenderNodeType::Rectangle(r) = &n.node_type {
            if r.style.fill_color == Some(0xfff2e6) {
                let cell = owner.expect("background belongs to a real cell");
                // Standard full-cell fill, including the accepted source inset
                // normalization. Do not re-pin the former ten drawing areas.
                assert!(
                    (n.bbox.x - cell.bbox.x).abs() < 0.5 && (n.bbox.y - cell.bbox.y).abs() < 0.5
                );
                assert!(
                    (n.bbox.width - cell.bbox.width).abs() < 0.5
                        && (n.bbox.height - cell.bbox.height).abs() < 0.5
                );
                count += 1;
            }
        }
        count
            + n.children
                .iter()
                .map(|c| background(c, owner))
                .sum::<usize>()
    }
    let saved = [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ];
    for c in std::iter::once(core).chain(saved.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
    {
        let root = &c.document().sections[0].paragraphs;
        assert_eq!(owners(&c, &root[0].controls, 0), 1);
        assert_eq!(table_count(&root[0].controls), 2);
        let tree = c.build_page_render_tree(0).unwrap();
        assert_eq!(background(&tree.root, None), 1);
        let mut source = Vec::new();
        model_text(root, &mut source);
        let mut actual = Vec::new();
        let mut lines = 0.;
        paint(&tree.root, &mut actual, &mut lines);
        let mut inside = Vec::new();
        for control in &root[0].controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    model_text(&cell.paragraphs, &mut inside);
                }
            }
        }
        assert_eq!(
            inside.iter().collect::<String>(),
            "BEFORECHILDABCDAFTERCHILDUPPERRIGHTLOWERLEFTLOWERRIGHT"
        );
        fn first_table(n: &RenderNode) -> Option<&RenderNode> {
            if matches!(n.node_type, RenderNodeType::Table(_)) {
                Some(n)
            } else {
                n.children.iter().find_map(first_table)
            }
        }
        let mut inside_paint = Vec::new();
        paint(
            first_table(&tree.root).unwrap(),
            &mut inside_paint,
            &mut lines,
        );
        assert_eq!(inside_paint, inside);
        assert_eq!(
            actual.len(),
            inside.len() + "OUTSIDEABOVEOUTSIDEAFTER".len()
        );
        let all = actual.iter().collect::<String>();
        assert_eq!(all.matches("OUTSIDEABOVE").count(), 1);
        assert_eq!(all.matches("OUTSIDEAFTER").count(), 1);
        geometry(&tree.root, None);
        order(&tree.root);
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

// Source pixel coverage includes cell-owned and retained page fragments. It is
// independent of the partition helper: PNG IHDR supplies the original extent.
fn verify_compound_source_pixels(core: &DocumentCore) {
    use rhwp::model::image::Picture;
    use std::collections::BTreeMap;
    fn visit<'a>(paras: &'a [Paragraph], out: &mut BTreeMap<u16, Vec<&'a Picture>>) {
        for p in paras {
            for c in &p.controls {
                match c {
                    Control::Picture(pic) => {
                        out.entry(pic.image_attr.bin_data_id).or_default().push(pic)
                    }
                    Control::Table(t) => {
                        for cell in &t.cells {
                            visit(&cell.paragraphs, out)
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut pics = BTreeMap::new();
    visit(&core.document().sections[0].paragraphs, &mut pics);
    for bin in &core.document().bin_data_content {
        let bytes = bin.data.load();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let width = i64::from(u32::from_be_bytes(bytes[16..20].try_into().unwrap())) * 75;
        let height = i64::from(u32::from_be_bytes(bytes[20..24].try_into().unwrap())) * 75;
        let values = pics
            .get(&bin.id)
            .expect("original image bytes must have an owner");
        let mut area = 0i64;
        for (i, p) in values.iter().enumerate() {
            let c = p.crop;
            assert!(
                c.left >= 0
                    && c.top >= 0
                    && c.right > c.left
                    && c.bottom > c.top
                    && i64::from(c.right) <= width
                    && i64::from(c.bottom) <= height
            );
            area += i64::from(c.right - c.left) * i64::from(c.bottom - c.top);
            // Source pixels are not resampled/cropped destructively in bin data.
            assert_eq!(p.img_dim, (width as u32, height as u32));
            for q in &values[..i] {
                let d = q.crop;
                assert!(
                    c.right <= d.left
                        || d.right <= c.left
                        || c.bottom <= d.top
                        || d.bottom <= c.top,
                    "source pixel regions have exactly one owner"
                );
            }
        }
        assert_eq!(area,width*height,"all source pixels remain in exactly one selected region, including retained page pieces");
    }
}
fn verify_compound_leaf_owners(core: &DocumentCore, key: &str) {
    fn check(t: &Table, depth: usize, out: &mut Vec<(usize, Vec<String>)>) {
        let nested = t
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .flat_map(|p| &p.controls)
            .any(|c| matches!(c, Control::Table(_)));
        if !nested {
            out.push((
                depth,
                t.cells
                    .iter()
                    .map(|c| {
                        c.paragraphs
                            .iter()
                            .map(|p| p.text.as_str())
                            .collect::<String>()
                    })
                    .collect(),
            ));
        }
        for c in &t.cells {
            for p in &c.paragraphs {
                for control in &p.controls {
                    if let Control::Table(child) = control {
                        check(child, depth + 1, out);
                    }
                }
            }
        }
    }
    let root = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .unwrap();
    let mut leaves = Vec::new();
    check(root, 0, &mut leaves);
    let depth = if key.ends_with("deep") { 2 } else { 1 };
    assert_eq!(leaves[0].0, depth);
    let wanted = if key.ends_with("image") {
        ["", "B", "C", "D"]
    } else {
        ["A", "B", "C", "D"]
    };
    assert_eq!(leaves[0].1, wanted);
    if key.ends_with("siblings") {
        assert_eq!(leaves.len(), 2);
        assert_eq!(leaves[1].1, ["E", "F", "G", "H"])
    } else {
        assert_eq!(leaves.len(), 1)
    }
}

fn verify_compound(core: &DocumentCore, tables: usize, expected: &[char]) {
    assert_eq!(core.page_count(), 1);
    assert_eq!(
        table_count(&core.document().sections[0].paragraphs[0].controls),
        tables,
        "actual Table ownership"
    );
    let mut chars = Vec::new();
    model_text(&core.document().sections[0].paragraphs, &mut chars);
    chars.sort_unstable();
    assert_eq!(chars, expected);
    let tree = core.build_page_render_tree(0).unwrap();
    let (mut painted, mut area) = (Vec::new(), 0.);
    paint(&tree.root, &mut painted, &mut area);
    painted.sort_unstable();
    assert_eq!(painted, expected, "source glyphs reach actual paint once");
    // A cropped boundary bitmap owns only its physical cell piece; unlike its
    // unsplit padded source, it must fit the final clipping cell. This also checks
    // that later anchor reset/clamp did not move the consumed piece.
    geometry(&tree.root, None);
    verify_compound_source_pixels(core);
}
#[test]
fn compound_paths_preserve_cells_source_pixels_and_both_save_formats() {
    for key in [
        "nested-dash-child",
        "nested-dash-parent",
        "nested-dash-both",
        "nested-dash-first",
        "nested-dash-image",
        "nested-dash-siblings",
        "nested-dash-deep",
        "compound-phase-alpha",
        "compound-clipped",
        "compound-backdrop",
    ] {
        let mut core = DocumentCore::from_pdf_bytes(
            &std::fs::read(base().join(format!("{key}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let count = if key.ends_with("siblings") || key.ends_with("deep") {
            3
        } else {
            2
        };
        let want = expected(key);
        verify_compound(&core, count, &want);
        verify_compound_leaf_owners(&core, key);
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let saved = DocumentCore::from_bytes(&data).unwrap();
            verify_compound(&saved, count, &want);
            verify_compound_leaf_owners(&saved, key);
        }
        let root = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[root]
        else {
            panic!()
        };
        let mut path = Vec::new();
        assert!(leaf_path(
            table,
            root,
            if key.ends_with("image") { "B" } else { "A" },
            &mut path
        ));
        core.insert_text_in_cell_by_path(0, 0, &path, 0, "X")
            .unwrap();
        let mut changed = want.clone();
        changed.push('X');
        changed.sort_unstable();
        verify_compound(&core, count, &changed);
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify_compound(&DocumentCore::from_bytes(&data).unwrap(), count, &changed);
        }
        core.delete_text_in_cell_by_path(0, 0, &path, 0, 1).unwrap();
        verify_compound(&core, count, &want);
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify_compound(&DocumentCore::from_bytes(&data).unwrap(), count, &want);
        }
    }
}
#[test]
fn compound_foreground_keeps_original_content_owners() {
    let core = DocumentCore::from_pdf_bytes(
        &std::fs::read(base().join("compound-foreground.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let want = expected("compound-foreground");
    let saved = [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ];
    for c in std::iter::once(core).chain(saved.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
    {
        verify_compound(&c, 0, &want);
    }
}

fn inline_host_case(name: &str, explicit_arial: bool) {
    let key = if name == "two-pictures" {
        "cell-two-pictures"
    } else {
        "cell-picture-first"
    };
    let mut core = DocumentCore::from_pdf_bytes(
        &std::fs::read(base().join(format!("{key}.pdf"))).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    if explicit_arial {
        // Controlled font selection precedes editing; Hancom's PDF embeds
        // ArialMT. The original Helvetica -> Haansoft Batang substitution is
        // separate evidence, not silently treated as this control's output.
        let mut input = core.document().clone();
        for fonts in &mut input.doc_info.font_faces {
            for font in fonts {
                font.name = "Arial".into();
            }
        }
        core.set_document(input);
    }
    let ci = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[ci] else {
        panic!()
    };
    let pi = table.cells[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Picture(_))))
        .unwrap();
    if matches!(name, "two-pictures" | "long-suffix" | "middle") {
        core.merge_paragraph_in_cell_native(0, 0, ci, 0, pi + 1)
            .unwrap();
    }
    if name == "outer-margins" {
        core.set_cell_picture_properties_by_path_native(0,0,
            &format!(r#"[{{"controlIndex":{ci},"cellIndex":0,"cellParaIndex":{pi}}}]"#),0,
            r#"{"outerMarginLeft":1500,"outerMarginRight":1500,"outerMarginTop":300,"outerMarginBottom":375}"#).unwrap();
    }
    let mut original = Vec::new();
    model_text(&core.document().sections[0].paragraphs, &mut original);
    original.sort_unstable();
    let original_bins: Vec<_> = core
        .document()
        .bin_data_content
        .iter()
        .map(|b| b.data.load().to_vec())
        .collect();
    let text = match name {
        "short-prefix" => "EDIT ".into(),
        "outer-margins" => "EDIT EDIT EDIT ".into(),
        "explicit-break" => "EDIT EDIT\nEDIT EDIT EDIT EDIT EDIT EDIT EDIT EDIT EDIT EDIT ".into(),
        _ => "EDIT ".repeat(12),
    };
    let offset = if name == "long-suffix" {
        let Control::Table(t) = &core.document().sections[0].paragraphs[0].controls[ci] else {
            panic!()
        };
        t.cells[0].paragraphs[pi].text.chars().count()
    } else {
        0
    };
    core.insert_text_in_cell_native(0, 0, ci, 0, pi, offset, &text)
        .unwrap();
    let mut changed = original.clone();
    changed.extend(text.chars().filter(|c| !c.is_whitespace()));
    changed.sort_unstable();
    let count = if name == "two-pictures" { 2 } else { 1 };
    verify_inline_host(&core, &changed, count, &original_bins);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        verify_inline_host(
            &DocumentCore::from_bytes(&bytes).unwrap(),
            &changed,
            count,
            &original_bins,
        );
    }
    core.delete_text_in_cell_native(0, 0, ci, 0, pi, offset, text.chars().count())
        .unwrap();
    verify_inline_host(&core, &original, count, &original_bins);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        verify_inline_host(
            &DocumentCore::from_bytes(&bytes).unwrap(),
            &original,
            count,
            &original_bins,
        );
    }
}
fn verify_inline_host(core: &DocumentCore, expected: &[char], images: usize, bins: &[Vec<u8>]) {
    assert_eq!(core.page_count(), 1);
    let actual: Vec<_> = core
        .document()
        .bin_data_content
        .iter()
        .map(|b| b.data.load().to_vec())
        .collect();
    assert_eq!(actual, bins, "editing preserves original image data");
    let mut text = Vec::new();
    model_text(&core.document().sections[0].paragraphs, &mut text);
    text.sort_unstable();
    assert_eq!(text, expected);
    let tree = core.build_page_render_tree(0).unwrap();
    geometry(&tree.root, None);
    let mut rendered = Vec::new();
    paint(&tree.root, &mut rendered, &mut 0.);
    rendered.sort_unstable();
    assert_eq!(rendered, expected, "glyphs are neither lost nor duplicated");
    fn visit(
        n: &RenderNode,
        images: &mut Vec<rhwp::renderer::render_tree::BoundingBox>,
        text: &mut Vec<rhwp::renderer::render_tree::BoundingBox>,
    ) {
        if matches!(n.node_type, RenderNodeType::Image(_)) {
            images.push(n.bbox);
        }
        if let RenderNodeType::TextRun(t) = &n.node_type {
            if !t.text.trim().is_empty() {
                text.push(n.bbox);
            }
        }
        for c in &n.children {
            visit(c, images, text)
        }
    }
    let (mut pics, mut glyphs) = (Vec::new(), Vec::new());
    visit(&tree.root, &mut pics, &mut glyphs);
    assert_eq!(pics.len(), images);
    for p in &pics {
        for t in &glyphs {
            let overlap = (p.x + p.width).min(t.x + t.width) - p.x.max(t.x) > 0.5
                && (p.y + p.height).min(t.y + t.height) - p.y.max(t.y) > 0.5;
            assert!(!overlap,"inline image and visible text occupy separate portions of their rows: {p:?} vs {t:?}");
        }
    }
    if pics.len() == 2 {
        // Same-row pictures retain the shared baseline while their source sizes
        // differ. This relation is independent of the chosen absolute origin.
        let a = pics[0];
        let b = pics[1];
        assert!((a.y + a.height * 0.85 - b.y - b.height * 0.85).abs() < 0.5);
        assert!(a.x + a.width <= b.x + 0.5, "picture order is preserved");
    }
}
#[test]
fn edited_terminal_picture_uses_a_real_row_and_survives_save_delete() {
    inline_host_case("long-prefix", false);
}
#[test]
fn inline_picture_controls_keep_text_breaks_margins_and_shared_row_ownership() {
    for key in [
        "long-prefix",
        "short-prefix",
        "explicit-break",
        "long-suffix",
        "middle",
        "two-pictures",
        "outer-margins",
    ] {
        inline_host_case(key, true);
    }
}
