//! PDF source/edit first-line baselines. Qualification: task_m100_7998_stage31.md.
//! The deferred body-noop line/save boundary is deliberately not a fixture here.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn path(c: &DocumentCore, nested: bool) -> Vec<(usize, usize, usize)> {
    let ci = c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    if nested {
        vec![(ci, 0, 1), (0, 0, 0)]
    } else {
        vec![(ci, 0, 0)]
    }
}
fn paragraph<'a>(c: &'a DocumentCore, path: &[(usize, usize, usize)]) -> &'a Paragraph {
    let mut p = &c.document().sections[0].paragraphs[0];
    for &(ci, cell, para) in path {
        let Control::Table(t) = &p.controls[ci] else {
            panic!("real table owner")
        };
        p = &t.cells[cell].paragraphs[para];
    }
    p
}
fn painted_text(c: &DocumentCore) -> String {
    fn walk(n: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            out.extend(r.text.chars().filter(|c| !c.is_whitespace()));
        }
        for child in &n.children {
            walk(child, out);
        }
    }
    let mut text = String::new();
    walk(&c.build_page_render_tree(0).unwrap().root, &mut text);
    text
}
fn glyphs(c: &DocumentCore, path: &[(usize, usize, usize)], end: usize) -> Vec<(usize, char, i32)> {
    fn walk(
        n: &RenderNode,
        path: &[(usize, usize, usize)],
        end: usize,
        out: &mut Vec<(usize, char, i32)>,
    ) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if r.cell_context.as_ref().is_some_and(|cx| {
                cx.path
                    .iter()
                    .map(|p| (p.control_index, p.cell_index, p.cell_para_index))
                    .eq(path.iter().copied())
            }) {
                let start = r.char_start.unwrap();
                let (_, baseline) = r
                    .style
                    .script_draw_metrics(r.style.font_size, n.bbox.y + r.baseline);
                for (i, ch) in r.text.chars().enumerate() {
                    if start + i < end && !ch.is_whitespace() {
                        out.push((start + i, ch, (baseline * 75.).round() as i32));
                    }
                }
            }
        }
        for c in &n.children {
            walk(c, path, end, out);
        }
    }
    let mut out = vec![];
    walk(
        &c.build_page_render_tree(0).unwrap().root,
        path,
        end,
        &mut out,
    );
    out
}

#[test]
fn pdf_cell_first_line_baselines_survive_painted_inverse_and_both_saves() {
    let pen =
        include_bytes!("../../pdf/issue7998/structure-borders/native-pen-blue.pdf").as_slice();
    let half = include_bytes!("../../pdf/issue7998/paragraph-baselines/half-point.pdf").as_slice();
    let script = include_bytes!("../../pdf/issue7998/solid-baseline.pdf").as_slice();
    // Tm/CTM, owner geometry, and source script offsets are authored independently.
    // Compare unchanged glyphs before/after real reflow; do not pin a .85/.95em
    // implementation constant or force source line counts after natural wrapping.
    for (name, pdf, nested) in [
        ("cell", pen, false),
        ("nested", pen, true),
        ("half-point", half, false),
        ("half-point-nested", half, true),
        ("script", script, false),
    ] {
        let original = DocumentCore::from_pdf_bytes(pdf, &PdfImportOptions::default()).unwrap();
        let hwp = original.export_hwp_with_adapter_snapshot().unwrap();
        let hwpx = original.export_hwpx_native().unwrap();
        for (route, bytes) in [
            ("pdf", pdf),
            ("hwp", hwp.as_slice()),
            ("hwpx", hwpx.as_slice()),
        ] {
            let mut c = if route == "pdf" {
                DocumentCore::from_pdf_bytes(bytes, &PdfImportOptions::default()).unwrap()
            } else {
                DocumentCore::from_bytes(bytes).unwrap()
            };
            let target = path(&c, nested);
            let before = paragraph(&c, &target).clone();
            let end = before
                .line_segs
                .get(1)
                .map_or(before.text.chars().count(), |line| line.text_start as usize);
            let source_glyphs = glyphs(&c, &target, end);
            let source_text = painted_text(&c);
            assert!(!source_glyphs.is_empty());
            c.insert_text_in_cell_by_path(0, 0, &target, 0, "X")
                .unwrap();
            assert_ne!(
                painted_text(&c),
                source_text,
                "actually paint the inserted glyph"
            );
            c.delete_text_in_cell_by_path(0, 0, &target, 0, 1).unwrap();
            assert_eq!(paragraph(&c, &target).text, before.text);
            assert_eq!(
                painted_text(&c),
                source_text,
                "all owner content/order survives the inverse"
            );
            assert_eq!(
                paragraph(&c, &target).line_segs[0].baseline_distance,
                before.line_segs[0].baseline_distance,
                "{name}/{route}: shared import/edit row metric"
            );
            let mut cores = vec![c];
            let hwp = cores[0].export_hwp_with_adapter_snapshot().unwrap();
            let hwpx = cores[0].export_hwpx_native().unwrap();
            cores.extend([
                DocumentCore::from_bytes(&hwp).unwrap(),
                DocumentCore::from_bytes(&hwpx).unwrap(),
            ]);
            for core in cores {
                let target = path(&core, nested);
                assert_eq!(paragraph(&core, &target).text, before.text);
                assert_eq!(painted_text(&core), source_text);
                let actual = glyphs(&core, &target, end);
                assert_eq!(
                    actual.iter().map(|x| (x.0, x.1)).collect::<Vec<_>>(),
                    source_glyphs.iter().map(|x| (x.0, x.1)).collect::<Vec<_>>()
                );
                for (actual, source) in actual.iter().zip(&source_glyphs) {
                    assert!((actual.2 - source.2).abs() <= 1,
                        "{name}/{route}: unchanged glyph's final physical baseline drifted: {actual:?} vs {source:?}");
                }
            }
        }
    }
}
