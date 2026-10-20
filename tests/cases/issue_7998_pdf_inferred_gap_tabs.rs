//! Independent source gaps stay editable, without rewriting authored spaces.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
type Path = Vec<(usize, usize, usize)>;
fn locate(p: &Paragraph, needle: &str, path: &mut Path, out: &mut Vec<Path>) {
    if p.text.contains(needle) {
        out.push(path.clone());
    }
    for (i, c) in p.controls.iter().enumerate() {
        match c {
            Control::Table(t) => {
                for (j, cell) in t.cells.iter().enumerate() {
                    for (k, p) in cell.paragraphs.iter().enumerate() {
                        path.push((i, j, k));
                        locate(p, needle, path, out);
                        path.pop();
                    }
                }
            }
            Control::Shape(s) => {
                if let Some(t) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                    for (k, p) in t.paragraphs.iter().enumerate() {
                        path.push((i, 0, k));
                        locate(p, needle, path, out);
                        path.pop();
                    }
                }
            }
            _ => {}
        }
    }
}
fn owner(c: &DocumentCore, needle: &str) -> Path {
    let mut paths = vec![];
    locate(
        &c.document().sections[0].paragraphs[0],
        needle,
        &mut vec![],
        &mut paths,
    );
    assert_eq!(
        paths.len(),
        1,
        "inferred source gap must belong to one editable paragraph: {needle}"
    );
    paths.pop().unwrap()
}
fn paragraph<'a>(c: &'a DocumentCore, path: &Path) -> &'a Paragraph {
    let mut p = &c.document().sections[0].paragraphs[0];
    for &(i, j, k) in path {
        p = match &p.controls[i] {
            Control::Table(t) => &t.cells[j].paragraphs[k],
            Control::Shape(s) => &s.drawing().unwrap().text_box.as_ref().unwrap().paragraphs[k],
            _ => panic!("text owner"),
        };
    }
    p
}
fn rows(c: &DocumentCore, path: &Path) -> Vec<String> {
    let mut rows: Vec<(f64, String)> = vec![];
    for (text, _, y) in painted(c, path) {
        if let Some((_, line)) = rows.last_mut().filter(|(last, _)| (y - *last).abs() < 0.25) {
            line.push_str(&text);
        } else {
            rows.push((y, text));
        }
    }
    rows.into_iter()
        .map(|(_, text)| text.trim_end().to_owned())
        .collect()
}
fn painted(c: &DocumentCore, path: &Path) -> Vec<(String, f64, f64)> {
    fn walk(n: &RenderNode, path: &Path, out: &mut Vec<(String, f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            if run.cell_context.as_ref().is_some_and(|ctx| {
                ctx.path
                    .iter()
                    .map(|s| (s.control_index, s.cell_index, s.cell_para_index))
                    .collect::<Vec<_>>()
                    == *path
            }) {
                out.push((run.text.clone(), n.bbox.x, n.bbox.y + run.baseline));
            }
        }
        for child in &n.children {
            walk(child, path, out)
        }
    }
    let mut out = vec![];
    walk(&c.build_page_render_tree(0).unwrap().root, path, &mut out);
    out
}
fn visible(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn assert_paint(c: &DocumentCore, path: &Path) {
    fn inside_cell<'a>(
        n: &'a RenderNode,
        path: &Path,
        cell: Option<&'a rhwp::renderer::render_tree::BoundingBox>,
    ) {
        let cell = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            Some(&n.bbox)
        } else {
            cell
        };
        if let (Some(cell), RenderNodeType::TextRun(run)) = (cell, &n.node_type) {
            if !run.text.trim().is_empty()
                && run.cell_context.as_ref().is_some_and(|ctx| {
                    ctx.path
                        .iter()
                        .map(|s| (s.control_index, s.cell_index, s.cell_para_index))
                        .collect::<Vec<_>>()
                        == *path
                })
            {
                assert!(
                    n.bbox.x >= cell.x - 0.5
                        && n.bbox.y >= cell.y - 0.5
                        && n.bbox.x + n.bbox.width <= cell.x + cell.width + 0.5
                        && n.bbox.y + n.bbox.height <= cell.y + cell.height + 0.5,
                    "actual glyph run stays inside its owning cell after reflow/save"
                );
            }
        }
        for child in &n.children {
            inside_cell(child, path, cell);
        }
    }
    inside_cell(&c.build_page_render_tree(0).unwrap().root, path, None);
    let p = paragraph(c, path);
    let actual = painted(c, path);
    assert_eq!(
        visible(
            &actual
                .iter()
                .map(|(s, _, _)| s.as_str())
                .collect::<String>()
        ),
        visible(&p.text),
        "source order and unique final paint"
    );
    let mut last: Option<(f64, f64)> = None;
    for (_, x, y) in actual {
        if let Some((px, py)) = last {
            assert!(
                y > py + 0.1 || (y - py).abs() < 0.1 && x >= px - 0.1,
                "paint order within each source row"
            );
        }
        last = Some((x, y));
    }
}
fn after_tab_x(c: &DocumentCore, path: &Path) -> f64 {
    let layout: serde_json::Value =
        serde_json::from_str(&c.get_page_text_layout_native(0).unwrap()).unwrap();
    let mut seen = false;
    for r in layout["runs"].as_array().unwrap() {
        let Some(entries) = r["cellPath"].as_array() else {
            continue;
        };
        let actual: Path = entries
            .iter()
            .map(|e| {
                (
                    e["controlIndex"].as_u64().unwrap() as usize,
                    e["cellIndex"].as_u64().unwrap() as usize,
                    e["cellParaIndex"].as_u64().unwrap() as usize,
                )
            })
            .collect();
        if actual != *path {
            continue;
        }
        for (i, ch) in r["text"].as_str().unwrap().chars().enumerate() {
            if seen && !ch.is_whitespace() {
                return r["x"].as_f64().unwrap() + r["charX"][i].as_f64().unwrap();
            }
            if ch == '\t' {
                seen = true;
            }
        }
    }
    panic!("painted source word after the tab");
}
fn assert_tab(p: &Paragraph) {
    let chars: Vec<_> = p.text.chars().collect();
    let i = chars.iter().position(|&c| c == '\t').unwrap();
    assert_eq!(
        p.char_offsets[i + 1] - p.char_offsets[i],
        8,
        "standard extended HWP tab, not a one-unit fake separator"
    );
    assert_eq!(p.tab_extended.len(), 1);
    assert!(
        p.tab_extended[0][0] > 0
            || rhwp::model::paragraph::tab_ext_is_placeholder(&p.tab_extended[0])
    );
    assert!(!p.text.contains(['\r', '\n', '\u{b}']));
}
fn source(page: usize) -> DocumentCore {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/pr7704/6778-square-2020.pdf"),
    )
    .unwrap();
    DocumentCore::from_pdf_bytes(
        &bytes,
        &PdfImportOptions {
            source_pages: Some(vec![page]),
            include_table_inference: true,
        },
    )
    .unwrap()
}
fn exercise(original: DocumentCore, needle: &str, expected: &str, expected_rows: Option<&[&str]>) {
    let pdf_hwp = original.export_hwp_with_adapter_snapshot().unwrap();
    let pdf_hwpx = original.export_hwpx_native().unwrap();
    let mut live = Some(original);
    for route in 0..3 {
        let mut c = if route == 0 {
            live.take().unwrap()
        } else {
            DocumentCore::from_bytes(if route == 1 { &pdf_hwp } else { &pdf_hwpx }).unwrap()
        };
        let path = owner(&c, needle);
        assert_eq!(paragraph(&c, &path).text, expected);
        assert_tab(paragraph(&c, &path));
        let source_rows = rows(&c, &path);
        if let Some(expected_rows) = expected_rows {
            assert_eq!(source_rows, expected_rows);
        }
        assert_paint(&c, &path);
        let original_paint = painted(&c, &path);
        let stop_origin = after_tab_x(&c, &path);
        c.insert_text_in_cell_by_path(0, 0, &path, 0, "X").unwrap();
        assert!((after_tab_x(&c, &path) - stop_origin).abs() < 0.25,"insertion before a positioning tab preserves its stop, rather than freezing the old gap width");
        assert_paint(&c, &path);
        c.delete_text_in_cell_by_path(0, 0, &path, 0, 1).unwrap();
        assert_eq!(paragraph(&c, &path).text, expected);
        assert_eq!(
            rows(&c, &path),
            source_rows,
            "direct inverse preserves soft-wrap cuts after the tab"
        );
        assert_tab(paragraph(&c, &path));
        assert_paint(&c, &path);
        let restored = painted(&c, &path);
        assert_eq!(original_paint.len(), restored.len());
        for (a, z) in original_paint.iter().zip(&restored) {
            assert_eq!(a.0, z.0);
            assert!(
                (a.1 - z.1).abs() < 0.25 && (a.2 - z.2).abs() < 0.25,
                "same restored text uses the same common origins"
            );
        }
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let reread = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(reread.page_count(), 1);
            let rp = owner(&reread, needle);
            assert_eq!(paragraph(&reread, &rp).text, expected);
            assert_eq!(rows(&reread, &rp), source_rows);
            assert_tab(paragraph(&reread, &rp));
            assert_paint(&reread, &rp);
        }
    }
}
#[test]
fn inferred_gap_keeps_real_report_words_in_one_editable_paragraph() {
    exercise(
        source(8),
        "붙임3",
        "붙임3\t책자 표지",
        Some(&["붙임3\t책자 표지"]),
    );
    exercise(
        source(6),
        "구성하고",
        "·비슷한 산차로 그룹을\t구성하고RFID 부착 후 군사시설 입실",
        None,
    );
}
#[test]
fn inferred_gap_tab_and_later_soft_wrap_share_the_hwp_stream_axis() {
    let path = std::env::var_os("RHWP_GAP_FIXTURE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/body-gap-tab.pdf")
        });
    let bytes = std::fs::read(path).unwrap();
    let c = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    exercise(
        c,
        "ALPHA",
        "ALPHA\tBETA GAMMA DELTA",
        Some(&["ALPHA\tBETA", "GAMMA DELTA"]),
    );
}
#[test]
fn source_spaces_remain_scalars_and_nonblank_width_failures_stay_unverified() {
    for (page, needle) in [(4, "붙임2"), (9, "붙임4")] {
        let c = source(page);
        let m = serde_json::to_value(c.pdf_metadata()).unwrap();
        let f = &m["pages"][0]["tableInference"]["bodyParagraphMapping"]["frames"][0];
        assert_eq!(f["status"], "paragraph_ir");
        let line = m["pages"][0]["tableInference"]["sourceTypography"]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| {
                l["context"] == serde_json::json!(["page", "body-region:0"])
                    && l["lineIndex"] == 0
                    && l["sourceSpaceMovements"]
                        .as_array()
                        .is_some_and(|v| !v.is_empty())
            })
            .unwrap();
        let original: String = line["characters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["character"].as_str().unwrap())
            .collect();
        let path = owner(&c, needle);
        let p = paragraph(&c, &path);
        assert_eq!(
            p.text.replace('\t', ""),
            original,
            "the original spaces survive as scalars, alongside separate inferred movement"
        );
        assert!(p.text.contains(" \t "));
        assert_paint(&c, &path);
    }
    let c = source(5);
    let m = serde_json::to_value(c.pdf_metadata()).unwrap();
    assert!(m["pages"][0]["tableInference"]["sourceTypography"]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["reason"] == "source glyph width exceeds HWP ratio range"));
}

#[test]
fn observed_cell_rows_keep_independent_pdf_glyph_origins() {
    // Original public PDF page 7, MuPDF structured text at 72 dpi. The page is
    // qualified at >=98% Native/fresh WASM; registration follows direct review.
    // These are existing centered peer cells, outside the new gap paragraph.
    let c = source(6);
    for (word, y_pt) in [
        ("적응훈련", 373.03233),
        ("사료조절", 452.7452),
        ("개체관리", 532.57797),
        ("관련사진", 697.1581),
    ] {
        let path = owner(&c, word);
        let first = painted(&c, &path)
            .into_iter()
            .find(|(t, _, _)| !t.trim().is_empty())
            .unwrap();
        assert!(
            (first.1 - 71.961 * 96. / 72.).abs() <= 2.,
            "valid observed centered cell origin must not be rebuilt at its left edge: {word}"
        );
        assert!(
            (first.2 - y_pt * 96. / 72.).abs() <= 2.,
            "independent source baseline: {word}"
        );
    }
}
