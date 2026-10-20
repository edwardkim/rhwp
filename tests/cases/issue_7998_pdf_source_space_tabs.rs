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
fn word_x(c: &DocumentCore, path: &Path, word: &str) -> f64 {
    let layout: serde_json::Value =
        serde_json::from_str(&c.get_page_text_layout_native(0).unwrap()).unwrap();
    let mut chars = Vec::new();
    for run in layout["runs"].as_array().unwrap() {
        let Some(entries) = run["cellPath"].as_array() else {
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
        for (i, ch) in run["text"].as_str().unwrap().chars().enumerate() {
            chars.push((
                ch,
                run["x"].as_f64().unwrap() + run["charX"][i].as_f64().unwrap(),
            ));
        }
    }
    let needle: Vec<_> = word.chars().collect();
    let at = chars
        .windows(needle.len())
        .rposition(|v| v.iter().map(|x| x.0).eq(needle.iter().copied()))
        .unwrap();
    chars[at].1
}
fn exercise(
    name: &str,
    expected: &str,
    stop_words: &[(&str, f64)],
    flowing_word: Option<(&str, f64)>,
) {
    let b = std::env::var_os("RHWP_SOURCE_SPACE_INPUTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/source-space-tabs")
        });
    let bytes = std::fs::read(b.join(format!("{name}.pdf"))).unwrap();
    let original = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    let source_path = owner(&original, "ALPHA");
    assert_eq!(
        paragraph(&original, &source_path).text.replace('\t', ""),
        expected.replace('\t', ""),
        "source spaces and both rows share one editable source paragraph"
    );
    let source_stops: Vec<_> = stop_words
        .iter()
        .map(|&(word, x)| {
            let actual = word_x(&original, &source_path, word);
            assert!(
                (actual - x * 96. / 72.).abs() <= 2.,
                "independent PDF destination within2px: {name} {word}"
            );
            (word, actual)
        })
        .collect();
    let flowing = flowing_word.map(|(word, x)| {
        let actual = word_x(&original, &source_path, word);
        assert!(
            (actual - x * 96. / 72.).abs() <= 2.,
            "independent source word origin"
        );
        (word, actual)
    });
    let files = [
        original.export_hwp_with_adapter_snapshot().unwrap(),
        original.export_hwpx_native().unwrap(),
    ];
    let mut live = Some(original);
    for route in 0..3 {
        let mut c = if route == 0 {
            live.take().unwrap()
        } else {
            DocumentCore::from_bytes(&files[route - 1]).unwrap()
        };
        for state in 0..3 {
            let path = owner(&c, "ALPHA");
            if state == 1 {
                c.insert_text_in_cell_by_path(0, 0, &path, 0, "X").unwrap();
            }
            if state == 2 {
                c.delete_text_in_cell_by_path(0, 0, &path, 0, 1).unwrap();
            }
            let target = if state == 1 {
                format!("X{expected}")
            } else {
                expected.to_owned()
            };
            assert_eq!(
                paragraph(&c, &path).text.replace('\t', ""),
                target.replace('\t', "")
            );
            assert_paint(&c, &path);
            if let Some((word, x)) = flowing {
                let actual = word_x(&c, &path, word);
                if state == 1 {
                    assert!(
                        actual > x,
                        "representable ordinary source spaces flow after inserted text"
                    );
                } else {
                    assert!(
                        (actual - x).abs() < 0.25,
                        "direct deletion restores ordinary source spacing"
                    );
                }
            }
            let para = paragraph(&c, &path);
            let chars: Vec<_> = para.text.chars().collect();
            assert_eq!(
                para.tab_extended.len(),
                chars.iter().filter(|&&ch| ch == '\t').count()
            );
            for (i, ch) in chars.iter().enumerate() {
                if *ch == '\t' {
                    assert_eq!(
                        para.char_offsets[i + 1] - para.char_offsets[i],
                        8,
                        "every navigation tab uses the HWP stream axis"
                    );
                }
            }
            assert!(
                !para.text.contains(['\n', '\r', '\u{b}']),
                "observed soft rows are not forced hard breaks"
            );
            assert_eq!(rows(&c, &path).len(), 2, "source two rows survive reflow");
            for &(word, source_x) in &source_stops {
                assert!((word_x(&c,&path,word)-source_x).abs()<0.25,"source destination after standard next-stop navigation: {word}, {name}, route {route}, state {state}");
            }
            for bytes in [
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
            ] {
                let reread = DocumentCore::from_bytes(&bytes).unwrap();
                let rp = owner(&reread, "ALPHA");
                assert_eq!(paragraph(&reread, &rp).text, paragraph(&c, &path).text);
                assert_paint(&reread, &rp);
                assert_eq!(rows(&reread, &rp).len(), 2);
                for &(word, x) in &source_stops {
                    assert!(
                        (word_x(&reread, &rp, word) - x).abs() < 0.25,
                        "saved source destination: {name} {word}"
                    );
                }
            }
        }
    }
}
#[test]
fn actual_source_space_survives_shared_stop_navigation() {
    exercise(
        "source-space-cross",
        "ALPHA \tBETA GAMMA DELTA GAMMA \t\tEPSILON",
        &[("BETA", 70.), ("EPSILON", 80.)],
        None,
    );
}
#[test]
fn consecutive_source_spaces_are_not_replaced_by_tabs() {
    exercise(
        "source-double-space",
        "ALPHA  \tBETA GAMMA DELTA GAMMA  \t\tEPSILON",
        &[("EPSILON", 80.)],
        Some(("BETA", 70.)),
    );
}
#[test]
fn long_source_prefix_skips_an_earlier_shared_stop() {
    exercise(
        "source-space-long",
        "ALPHA \tBETA GAMMA DELTA EPSILON LONGGAMMALONG \tDELTA",
        &[("BETA", 70.), ("DELTA", 140.)],
        None,
    );
}
#[test]
fn ordinary_source_spaces_need_no_positioning_tabs() {
    exercise(
        "source-space-normal",
        "ALPHA BETA GAMMA DELTA EPSILON GAMMA",
        &[],
        None,
    );
}

#[test]
fn compressed_source_space_is_not_replaced_with_forward_movement() {
    let b = std::env::var_os("RHWP_SOURCE_SPACE_INPUTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/source-space-tabs")
        });
    let c = DocumentCore::from_pdf_bytes(
        &std::fs::read(b.join("source-space-tight.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let m = serde_json::to_value(c.pdf_metadata()).unwrap();
    let first = &m["pages"][0]["tableInference"]["bodyParagraphMapping"]["frames"][0];
    assert_eq!(first["status"], "retained_positioned_spans");
    assert_eq!(first["reason"],"body glyph metrics require positioned source: source whitespace advance exceeds combined HWP character range");
    assert_eq!(paragraph(&c, &owner(&c, "ALPHA")).text, "ALPHA ");
    assert!(paragraph(&c, &owner(&c, "BETA"))
        .text
        .contains("BETA GAMMA DELTA"));
    assert_eq!(paragraph(&c, &owner(&c, "SECOND")).text, "SECOND");
}

#[test]
fn nonbreaking_source_space_does_not_acquire_a_breakable_tab() {
    let b = std::env::var_os("RHWP_SOURCE_SPACE_INPUTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/source-space-tabs")
        });
    let c = DocumentCore::from_pdf_bytes(
        &std::fs::read(b.join("source-nonbreaking-space.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let m = serde_json::to_value(c.pdf_metadata()).unwrap();
    assert_eq!(
        m["pages"][0]["tableInference"]["bodyParagraphMapping"]["frames"][0]["status"],
        "retained_positioned_spans"
    );
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let reread = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            paragraph(&reread, &owner(&reread, "ALPHA")).text,
            "ALPHA\u{a0}",
            "NBSP retains its scalar and never gains a break opportunity from an inferred tab"
        );
        assert!(paragraph(&reread, &owner(&reread, "BETA"))
            .text
            .contains("BETA\u{a0}GAMMA\u{a0}DELTA"));
        assert_eq!(paragraph(&reread, &owner(&reread, "SECOND")).text, "SECOND");
    }
}
