//! Observed PDF indentation remains in editable paragraphs through both saves.
//! Independent PDF/Hancom Print and before/after evidence: task_m100_7998_stage33.md.
#![cfg(feature = "pdf-import")]

use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn owners(core: &DocumentCore) -> Vec<usize> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| match control {
            Control::Shape(shape)
                if shape
                    .drawing()
                    .and_then(|d| d.text_box.as_ref())
                    .is_some_and(|t| t.paragraphs.len() > 1) =>
            {
                Some(index)
            }
            _ => None,
        })
        .collect()
}

fn paragraph(core: &DocumentCore, owner: usize, index: usize) -> &Paragraph {
    let Control::Shape(shape) = &core.document().sections[0].paragraphs[0].controls[owner] else {
        unreachable!()
    };
    &shape
        .drawing()
        .unwrap()
        .text_box
        .as_ref()
        .unwrap()
        .paragraphs[index]
}

fn rows(core: &DocumentCore, owner: usize) -> Vec<String> {
    let para = paragraph(core, owner, 0);
    let text: Vec<_> = para.text.encode_utf16().collect();
    let starts: Vec<_> = para
        .line_segs
        .iter()
        .map(|l| l.text_start as usize)
        .chain(std::iter::once(text.len()))
        .collect();
    starts
        .windows(2)
        .map(|range| {
            String::from_utf16_lossy(&text[range[0]..range[1]])
                .trim()
                .to_owned()
        })
        .collect()
}

fn baseline(core: &DocumentCore, owner: usize, index: usize) -> i32 {
    fn find(node: &RenderNode, owner: usize, index: usize) -> Option<i32> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.cell_context.as_ref().is_some_and(|ctx| {
                ctx.path.len() == 1
                    && ctx.path[0].control_index == owner
                    && ctx.path[0].cell_para_index == index
            }) {
                let (_, y) = run
                    .style
                    .script_draw_metrics(run.style.font_size, node.bbox.y + run.baseline);
                return Some((y * 75.).round() as i32);
            }
        }
        node.children.iter().find_map(|n| find(n, owner, index))
    }
    find(&core.build_page_render_tree(0).unwrap().root, owner, index).unwrap()
}

fn assert_painted_once(core: &DocumentCore, owner: usize) {
    fn collect(node: &RenderNode, owner: usize, text: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run
                .cell_context
                .as_ref()
                .is_some_and(|ctx| ctx.path.len() == 1 && ctx.path[0].control_index == owner)
            {
                text.extend(run.text.chars().filter(|c| !c.is_whitespace()));
            }
        }
        for child in &node.children {
            collect(child, owner, text);
        }
    }
    let model = format!(
        "{}{}",
        paragraph(core, owner, 0).text,
        paragraph(core, owner, 1).text
    );
    let mut painted = String::new();
    collect(
        &core.build_page_render_tree(0).unwrap().root,
        owner,
        &mut painted,
    );
    assert_eq!(
        painted,
        model
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>(),
        "both paragraphs reach final paint once in source order"
    );
}

#[test]
fn observed_indentation_survives_reflow_and_both_save_formats() {
    fn physical_starts(core: &DocumentCore, owner: usize) -> Vec<(f64, f64)> {
        fn walk(n: &RenderNode, owner: usize, out: &mut Vec<(f64, f64)>) {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                if run.cell_context.as_ref().is_some_and(|c| {
                    c.path.len() == 1
                        && c.path[0].control_index == owner
                        && c.path[0].cell_para_index == 0
                }) {
                    let y = n.bbox.y + run.baseline;
                    if out
                        .last()
                        .is_none_or(|&(_, last_y)| (y - last_y).abs() > 0.25)
                    {
                        out.push((n.bbox.x, y));
                    }
                }
            }
            for child in &n.children {
                walk(child, owner, out);
            }
        }
        let mut out = Vec::new();
        walk(
            &core.build_page_render_tree(0).unwrap().root,
            owner,
            &mut out,
        );
        out
    }
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998");
    // Expected rows and paragraph boundaries come from the independent PDF text
    // show commands, not the inferred width or the importer's current LineSegs.
    for (name, expected) in [
        (
            "body-indent",
            vec![(vec!["ALPHA BETA", "GAMMA DELTA"], "SECOND")],
        ),
        (
            "body-list",
            vec![(vec!["* LIST ENTRY", "CONTINUED ITEM"], "NEXT ITEM")],
        ),
    ] {
        let pdf = std::fs::read(base.join(format!("{name}.pdf"))).unwrap();
        let original = DocumentCore::from_pdf_bytes(&pdf, &PdfImportOptions::default()).unwrap();
        let hwp = original.export_hwp_with_adapter_snapshot().unwrap();
        let hwpx = original.export_hwpx_native().unwrap();
        for (route, bytes) in [("pdf", &pdf), ("hwp", &hwp), ("hwpx", &hwpx)] {
            let mut core = if route == "pdf" {
                DocumentCore::from_pdf_bytes(bytes, &PdfImportOptions::default()).unwrap()
            } else {
                DocumentCore::from_bytes(bytes).unwrap()
            };
            let groups = owners(&core);
            assert_eq!(groups.len(), expected.len(), "source paragraph owners");
            for (ordinal, (&owner, (source_rows, following))) in
                groups.iter().zip(&expected).enumerate()
            {
                let text = paragraph(&core, owner, 0).text.clone();
                assert_eq!(text, source_rows.join(" "));
                assert!(!text.contains(['\r', '\n', '\u{b}']));
                assert_eq!(paragraph(&core, owner, 1).text, *following);
                assert_eq!(rows(&core, owner), *source_rows);
                let starts = physical_starts(&core, owner);
                assert_eq!(starts.len(), 2);
                let delta = if name == "body-indent" { 12. } else { -15. };
                assert!(
                    (starts[0].0 - starts[1].0 - delta * 96. / 72.).abs() < 0.25,
                    "independent PDF first/continuation origin relation"
                );
                let first_y = baseline(&core, owner, 0);
                let next_y = baseline(&core, owner, 1);
                assert!(next_y > first_y);
                let path = vec![(owner, 0, 0)];
                core.insert_text_in_cell_by_path(0, 0, &path, 0, "X")
                    .unwrap();
                // Exercise paint between edits; direct deletion is not Snapshot undo.
                assert_painted_once(&core, owner);
                core.delete_text_in_cell_by_path(0, 0, &path, 0, 1).unwrap();
                assert_eq!(paragraph(&core, owner, 0).text, text);
                assert_eq!(
                    rows(&core, owner),
                    *source_rows,
                    "{name}/{route}/{ordinal}: ordinary properties reproduce source soft wraps"
                );
                assert_eq!(paragraph(&core, owner, 1).text, *following);
                assert!((baseline(&core, owner, 0) - first_y).abs() <= 1);
                assert!(
                    (baseline(&core, owner, 1) - next_y).abs() <= 1,
                    "following paragraph consumes the same final row pitch"
                );
                assert_painted_once(&core, owner);
                let restored = physical_starts(&core, owner);
                assert_eq!(restored.len(), starts.len());
                for (a, b) in starts.iter().zip(&restored) {
                    assert!(
                        (a.0 - b.0).abs() < 0.25 && (a.1 - b.1).abs() < 0.25,
                        "inverse edit retains both physical row origins"
                    );
                }
                for saved in [
                    core.export_hwp_with_adapter_snapshot().unwrap(),
                    core.export_hwpx_native().unwrap(),
                ] {
                    let reread = DocumentCore::from_bytes(&saved).unwrap();
                    let reopened_owner = owners(&reread)[ordinal];
                    assert_eq!(rows(&reread, reopened_owner), *source_rows);
                    assert_eq!(paragraph(&reread, reopened_owner, 1).text, *following);
                    assert!((baseline(&reread, reopened_owner, 1) - next_y).abs() <= 1);
                    assert_painted_once(&reread, reopened_owner);
                    let saved_starts = physical_starts(&reread, reopened_owner);
                    assert_eq!(saved_starts.len(), starts.len());
                    for (a, b) in starts.iter().zip(&saved_starts) {
                        assert!(
                            (a.0 - b.0).abs() < 0.25 && (a.1 - b.1).abs() < 0.25,
                            "physical starts survive both formats"
                        );
                    }
                }
            }
        }
    }
}
