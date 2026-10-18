//! Independent untagged PDF sources, actual body Paragraph ownership and final paint.
//! Qualification and before/after producer evidence: task_m100_7998_stage3.md, P3.
#![cfg(feature = "pdf-import")]

use rhwp::{
    model::{control::Control, style::Alignment},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn groups(core: &DocumentCore) -> Vec<(usize, Vec<String>)> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if let Control::Shape(s) = c {
                s.drawing()?
                    .text_box
                    .as_ref()
                    .map(|b| (i, b.paragraphs.iter().map(|p| p.text.clone()).collect()))
            } else {
                None
            }
        })
        .collect()
}
fn verify(core: &DocumentCore, expected: &[Vec<String>], fallback: &[&str]) {
    let actual = groups(core);
    let selected: Vec<_> = actual
        .iter()
        .filter(|(_, v)| expected.contains(v))
        .map(|(_, v)| v.clone())
        .collect();
    assert_eq!(
        selected, expected,
        "actual body paragraph groups, not recognition previews"
    );
    let expected_paint: String = actual
        .iter()
        .flat_map(|(_, v)| v.iter())
        .flat_map(|s| s.chars())
        .filter(|c| !c.is_whitespace())
        .collect();
    fn paint(n: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            out.extend(r.text.chars().filter(|c| !c.is_whitespace()));
        }
        if matches!(n.node_type, RenderNodeType::TextBox) {
            let lines: Vec<_> = n
                .children
                .iter()
                .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
                .collect();
            for pair in lines.windows(2) {
                assert!(
                    pair[1].bbox.y + 0.5 >= pair[0].bbox.y + pair[0].bbox.height,
                    "body lines may not overlap after source/edit/save"
                );
            }
        }
        for c in &n.children {
            paint(c, out);
        }
    }
    let mut actual_paint = String::new();
    paint(
        &core.build_page_render_tree(0).unwrap().root,
        &mut actual_paint,
    );
    assert_eq!(
        actual_paint, expected_paint,
        "all model characters reach final paint once, preserving owner order"
    );
    let fallback_actual: Vec<_> = actual
        .iter()
        .filter(|(_, v)| !expected.contains(v))
        .flat_map(|(_, v)| v.iter().map(String::as_str))
        .collect();
    assert_eq!(
        fallback_actual, fallback,
        "unconsumed source owners retained exactly once"
    );
}
fn style(core: &DocumentCore, key: &str, expected: &[Vec<String>]) {
    if !matches!(key, "body-center" | "body-indent" | "body-list") {
        return;
    }
    let ci = groups(core)
        .iter()
        .find(|(_, v)| v == &expected[0])
        .unwrap()
        .0;
    let Control::Shape(s) = &core.document().sections[0].paragraphs[0].controls[ci] else {
        panic!()
    };
    let p = &s.drawing().unwrap().text_box.as_ref().unwrap().paragraphs[0];
    let ps = &core.document().doc_info.para_shapes[p.para_shape_id as usize];
    match key {
        "body-center" => assert_eq!(
            ps.alignment,
            Alignment::Center,
            "page-centered paragraph is not a hanging indent"
        ),
        "body-indent" => assert!(ps.indent > 0),
        "body-list" => assert!(ps.indent < 0),
        _ => {}
    }
}
fn script(core: &DocumentCore) {
    fn find(n: &RenderNode, target: &str) -> Option<(f64, f64)> {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if r.text == target {
                assert!(!r.style.superscript && !r.style.subscript);
                return Some(
                    r.style
                        .script_draw_metrics(r.style.font_size, n.bbox.y + r.baseline),
                );
            }
        }
        n.children.iter().find_map(|n| find(n, target))
    }
    let tree = core.build_page_render_tree(0).unwrap();
    let (font, y) = find(&tree.root, "H").unwrap();
    let (small, up) = find(&tree.root, "2").unwrap();
    assert!((small / font - 0.6).abs() < 0.001);
    assert!(((up - y) / font + 0.4).abs() < 0.01);
}
#[test]
fn body_regions_are_real_editable_paragraphs_through_both_save_formats() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998");
    for (key, want, fallback) in [
        (
            "body-basic",
            vec![vec!["ALPHA BETA GAMMA DELTA", "SECOND"]],
            vec![],
        ),
        (
            "body-reordered",
            vec![vec!["ALPHA BETA GAMMA DELTA", "SECOND"]],
            vec![],
        ),
        (
            "body-columns",
            vec![
                vec!["ALPHA LEFT GAMMA LEFT", "SECOND LEFT"],
                vec!["ALPHA RIGHT GAMMA RIGHT", "SECOND RIGHT"],
            ],
            vec![],
        ),
        (
            "body-title-columns",
            vec![
                vec!["HEADING ACROSS BOTH TEXT COLUMNS"],
                vec!["ALPHA LEFT GAMMA LEFT", "SECOND LEFT"],
                vec!["ALPHA RIGHT GAMMA RIGHT", "SECOND RIGHT"],
            ],
            vec![],
        ),
        (
            "body-indent",
            vec![vec!["ALPHA BETA GAMMA DELTA", "SECOND"]],
            vec![],
        ),
        (
            "body-list",
            vec![vec!["* LIST ENTRY CONTINUED ITEM", "NEXT ITEM"]],
            vec![],
        ),
        (
            "body-ambiguous",
            vec![vec!["NORMAL BODY NORMAL TAIL"]],
            vec!["LEFT", "RIGHT"],
        ),
        (
            "body-unrepresentable",
            vec![vec!["NORMAL BODY NORMAL TAIL"]],
            vec!["A", "B", "GAMMA"],
        ),
        (
            "body-picture",
            vec![vec!["BEFORE PICTURE"], vec!["AFTER PICTURE"]],
            vec![],
        ),
        (
            "body-close-title",
            vec![vec!["CLOSE TITLE", "ALPHA BETA GAMMA DELTA"]],
            vec![],
        ),
        (
            "body-center",
            vec![vec!["ALPHA BETA GAMMA", "SECOND"]],
            vec![],
        ),
        ("body-baseline", vec![vec!["H2O GAMMA", "SECOND"]], vec![]),
        (
            "body-tj",
            vec![vec!["ALPHA BETA GAMMA DELTA", "SECOND"]],
            vec![],
        ),
    ] {
        let expected: Vec<Vec<String>> = want
            .into_iter()
            .map(|v| v.into_iter().map(str::to_string).collect())
            .collect();
        let bytes = std::fs::read(base.join(format!("{key}.pdf"))).unwrap();
        let mut core = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
        verify(&core, &expected, &fallback);
        style(&core, key, &expected);
        if key == "body-baseline" {
            script(&core);
        }
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let c = DocumentCore::from_bytes(&data).unwrap();
            verify(&c, &expected, &fallback);
            style(&c, key, &expected);
        }
        let ci = groups(&core)
            .iter()
            .find(|(_, v)| v == &expected[0])
            .unwrap()
            .0;
        core.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "IR:")
            .unwrap();
        let mut inserted = expected.clone();
        inserted[0][0].insert_str(0, "IR:");
        verify(&core, &inserted, &fallback);
        core.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 3)
            .unwrap();
        verify(&core, &expected, &fallback);
        core.split_paragraph_in_cell_native(0, 0, ci, 0, 0, 3, None)
            .unwrap();
        let mut split = expected.clone();
        let tail = split[0][0].split_off(3);
        split[0].insert(1, tail);
        verify(&core, &split, &fallback);
        core.merge_paragraph_in_cell_native(0, 0, ci, 0, 1).unwrap();
        verify(&core, &expected, &fallback);
        style(&core, key, &expected);
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let c = DocumentCore::from_bytes(&data).unwrap();
            verify(&c, &expected, &fallback);
            style(&c, key, &expected);
        }
        println!("{key}: source/insert/delete/split/merge/HWP/HWPX passed");
    }
}
#[test]
fn overlapping_paint_keeps_the_original_source_owners() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/body-occlusion.pdf"),
    )
    .unwrap();
    let core = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    verify(&core, &[], &["VISIBLE BEFORE", "VISIBLE AFTER"]);
    assert_eq!(
        core.document().sections[0].paragraphs[0].controls.len(),
        3,
        "two source text owners and interleaved rectangle survive"
    );
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        verify(
            &DocumentCore::from_bytes(&bytes).unwrap(),
            &[],
            &["VISIBLE BEFORE", "VISIBLE AFTER"],
        );
    }
}
#[test]
fn body_owners_keep_columns_heading_and_picture_separate_in_final_paint() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998");
    fn collect<'a>(
        n: &'a RenderNode,
        boxes: &mut Vec<&'a RenderNode>,
        images: &mut Vec<&'a RenderNode>,
    ) {
        match n.node_type {
            RenderNodeType::TextBox => boxes.push(n),
            RenderNodeType::Image(_) => images.push(n),
            _ => {}
        }
        for c in &n.children {
            collect(c, boxes, images);
        }
    }
    for key in ["body-columns", "body-title-columns", "body-picture"] {
        let source = DocumentCore::from_pdf_bytes(
            &std::fs::read(base.join(format!("{key}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let saved = [
            source.export_hwp_native().unwrap(),
            source.export_hwpx_native().unwrap(),
        ];
        for core in std::iter::once(source).chain(
            saved
                .iter()
                .map(|bytes| DocumentCore::from_bytes(bytes).unwrap()),
        ) {
            let tree = core.build_page_render_tree(0).unwrap();
            let (mut boxes, mut images) = (Vec::new(), Vec::new());
            collect(&tree.root, &mut boxes, &mut images);
            if key == "body-picture" {
                assert_eq!(boxes.len(), 2);
                assert_eq!(images.len(), 1);
                assert!(
                    boxes[0].bbox.y + boxes[0].bbox.height <= images[0].bbox.y + 0.5,
                    "before text cannot overlap the original picture"
                );
                assert!(
                    images[0].bbox.y + images[0].bbox.height <= boxes[1].bbox.y + 0.5,
                    "after text remains after the picture"
                );
            } else {
                let offset = usize::from(key == "body-title-columns");
                assert_eq!(boxes.len(), 2 + offset);
                assert!(
                    boxes[offset].bbox.x + boxes[offset].bbox.width
                        <= boxes[offset + 1].bbox.x + 0.5,
                    "source columns retain disjoint owners"
                );
                if offset == 1 {
                    assert!(
                        boxes[0].bbox.y + boxes[0].bbox.height <= boxes[1].bbox.y + 0.5,
                        "spanning heading remains above the columns"
                    );
                }
            }
        }
    }
}
#[test]
fn fractional_width_reconstruction_stays_within_hwp_metric_quantization() {
    let bytes = include_bytes!("../../pdf/issue7998/superscript.pdf");
    let source = DocumentCore::from_pdf_bytes(bytes, &PdfImportOptions::default()).unwrap();
    fn width(n: &RenderNode) -> Option<f64> {
        if matches!(n.node_type, RenderNodeType::TextLine(_)) && n.bbox.x < 100. {
            let runs: Vec<_> = n
                .children
                .iter()
                .filter(|c| matches!(c.node_type, RenderNodeType::TextRun(_)))
                .collect();
            let text: String = runs
                .iter()
                .filter_map(|c| {
                    if let RenderNodeType::TextRun(r) = &c.node_type {
                        Some(r.text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            if text == "SECOND" {
                return Some(
                    (runs
                        .iter()
                        .map(|r| r.bbox.x + r.bbox.width)
                        .fold(f64::NEG_INFINITY, f64::max)
                        - runs.iter().map(|r| r.bbox.x).fold(f64::INFINITY, f64::min))
                        * 72.
                        / 96.,
                );
            }
        }
        n.children.iter().find_map(width)
    }
    // Adobe Helvetica AFM: S667+E667+C722+O778+N722+D722=4278, at 10pt.
    // HWP integer-percent ratio and spacing each permit half a percent rounding;
    // two endpoint HWPUNIT conversions permit another 0.02pt. This is not a px pin.
    let independent = 42.78;
    let rounding_bound = 0.005 * (independent + 6. * 10.) + 0.02;
    let saved = [
        source.export_hwp_native().unwrap(),
        source.export_hwpx_native().unwrap(),
    ];
    for core in std::iter::once(source).chain(
        saved
            .iter()
            .map(|bytes| DocumentCore::from_bytes(bytes).unwrap()),
    ) {
        let observed = width(&core.build_page_render_tree(0).unwrap().root).unwrap();
        assert!((observed-independent).abs()<=rounding_bound,"independent AFM width={independent}, observed={observed}, format quantization bound={rounding_bound}");
    }
}
