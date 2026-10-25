//! #8220: the editable PDF layout stores recognized body paragraphs as the
//! section's own text flow. Independent expectations: the source HWP's
//! paragraph text and the PDF's page count. Checks are relational: paragraph
//! ownership, joins across column and page boundaries, and that the renderer
//! keeps every stored row instead of breaking the text again.
#![cfg(feature = "pdf-import")]

use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    parser::pdf::{PdfImportLayout, PdfImportOptions},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "samples/21_언어_기출_편집가능본.hwp";
/// Source PDF, its page count, and source paragraphs the PDF cut at a column
/// or page boundary. The 2010 print keeps running headers inside its column
/// regions, so its column tops carry no continuation evidence.
const PDFS: [(&str, usize, &[&str]); 3] = [
    (
        "21_언어_기출_편집가능본-2020.pdf",
        15,
        &["물론자신이풍경안에갇혀", "싱어는위와같은롤스의시도를"],
    ),
    ("21_언어_기출_편집가능본-2010.pdf", 16, &[]),
    (
        "21_언어_기출_편집가능본.pdf",
        15,
        &["싱어는위와같은롤스의시도를"],
    ),
];

fn norm(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn collect(paragraphs: &[Paragraph], out: &mut Vec<String>) {
    for p in paragraphs {
        let text = norm(&p.text);
        if !text.is_empty() {
            out.push(text);
        }
        for control in &p.controls {
            match control {
                Control::Table(t) => t.cells.iter().for_each(|c| collect(&c.paragraphs, out)),
                Control::Shape(s) => {
                    if let Some(b) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        collect(&b.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

fn source_paragraphs() -> Vec<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut out = Vec::new();
    for section in &core.document().sections {
        collect(&section.paragraphs, &mut out);
    }
    out
}

fn source(sources: &[String], prefix: &str) -> String {
    let found: Vec<_> = sources.iter().filter(|t| t.starts_with(prefix)).collect();
    assert_eq!(found.len(), 1, "source paragraph `{prefix}` is unique");
    found[0].clone()
}

fn import(name: &str, layout: PdfImportLayout) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    let options = PdfImportOptions {
        layout,
        ..PdfImportOptions::default()
    };
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &options).unwrap()
}

/// Text of the section's own (flow) paragraphs.
fn flow(core: &DocumentCore) -> Vec<String> {
    core.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| norm(&p.text))
        .filter(|t| !t.is_empty())
        .collect()
}

/// For every flow paragraph, the first character of each rendered row and
/// the (page, column-left) places its rows were painted in.
fn rendered_rows(
    core: &DocumentCore,
) -> BTreeMap<usize, (BTreeSet<usize>, BTreeSet<(usize, i64)>)> {
    fn walk(n: &RenderNode, page: usize, rows: &mut BTreeMap<(usize, usize, i64), (usize, i64)>) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if let (Some(0), Some(pi), Some(start), None) =
                (r.section_index, r.para_index, r.char_start, &r.cell_context)
            {
                let key = (pi, page, n.bbox.y.round() as i64);
                let x = n.bbox.x.round() as i64;
                let e = rows.entry(key).or_insert((start, x));
                if start < e.0 {
                    *e = (start, x);
                }
            }
        }
        for c in &n.children {
            walk(c, page, rows);
        }
    }
    let mut rows = BTreeMap::new();
    for page in 0..core.page_count() {
        walk(
            &core.build_page_render_tree(page).unwrap().root,
            page as usize,
            &mut rows,
        );
    }
    let mut out: BTreeMap<usize, (BTreeSet<usize>, BTreeSet<(usize, i64)>)> = BTreeMap::new();
    for ((pi, page, _), (start, x)) in rows {
        let entry = out.entry(pi).or_default();
        entry.0.insert(start);
        // Column of the row: left or right half of the A3 page (1121px).
        entry.1.insert((page, i64::from(x >= 560)));
    }
    out
}

/// Character index of each stored row start.
fn stored_rows(p: &Paragraph) -> BTreeSet<usize> {
    p.line_segs
        .iter()
        .map(|seg| {
            p.char_offsets
                .iter()
                .position(|&o| o >= seg.text_start)
                .unwrap_or(0)
        })
        .collect()
}

fn verify(core: &DocumentCore, sources: &[String], pages: usize, joined: &[&str], label: &str) {
    assert_eq!(
        core.page_count() as usize,
        pages,
        "{label}: one page per source page"
    );
    let doc = core.document();
    assert_eq!(
        doc.sections.len(),
        1,
        "{label}: equally sized pages share one section"
    );
    let columns = doc.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::ColumnDef(cd) => Some(cd.column_count),
            _ => None,
        });
    assert_eq!(
        columns,
        Some(2),
        "{label}: the section opens with the two-column grid"
    );
    let body = flow(core);
    let mut wanted: Vec<String> = [
        "프로세스마이닝은정보시스템을통해확보한",
        "적합성검증이란기존의프로세스모델과",
        "4.윗글과일치하지않는것은?",
        "◯◯병원에서는외래환자의",
    ]
    .iter()
    .map(|p| source(sources, p))
    .collect();
    wanted.extend(joined.iter().map(|p| source(sources, p)));
    for text in &wanted {
        let head: String = text.chars().take(24).collect();
        assert_eq!(
            body.iter().filter(|t| *t == text).count(),
            1,
            "{label}: one flow paragraph owns `{head}`"
        );
    }
    let rows = rendered_rows(core);
    let paragraphs = &doc.sections[0].paragraphs;
    for prefix in joined {
        let text = source(sources, prefix);
        let pi = paragraphs
            .iter()
            .position(|p| norm(&p.text) == text)
            .unwrap();
        let places = &rows[&pi].1;
        assert!(
            places.len() >= 2,
            "{label}: `{prefix}` continues into the next column or page: {places:?}"
        );
    }
    // The renderer keeps every stored row of every flow paragraph.
    for (pi, (starts, _)) in &rows {
        let p = &paragraphs[*pi];
        if p.text.trim().is_empty() || p.line_segs.is_empty() {
            continue;
        }
        let head: String = p.text.chars().take(24).collect();
        assert_eq!(
            *starts,
            stored_rows(p),
            "{label}: rows of `{head}` are the stored rows"
        );
    }
}

#[test]
fn editable_layout_flows_body_paragraphs_through_columns_and_pages() {
    let sources = source_paragraphs();
    for (name, pages, joined) in PDFS {
        let core = import(name, PdfImportLayout::Editable);
        verify(&core, &sources, pages, joined, name);
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify(
                &DocumentCore::from_bytes(&data).unwrap(),
                &sources,
                pages,
                joined,
                name,
            );
        }
    }
}

#[test]
fn fidelity_layout_stays_the_default() {
    let (name, pages, _) = PDFS[0];
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    let core =
        DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &PdfImportOptions::default())
            .unwrap();
    // One positioned section per page; body text lives in frames, not flow.
    assert_eq!(core.document().sections.len(), pages);
    assert!(
        flow(&core).is_empty(),
        "fidelity layout keeps no body text in section flow"
    );
}

#[test]
fn editing_a_joined_paragraph_reflows_it_in_its_column() {
    let sources = source_paragraphs();
    let mut core = import(PDFS[0].0, PdfImportLayout::Editable);
    let text = source(&sources, "싱어는위와같은롤스의시도를");
    let pi = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| norm(&p.text) == text)
        .unwrap();
    core.insert_text_native(0, pi, 0, "IR:").unwrap();
    let edited = norm(&core.document().sections[0].paragraphs[pi].text);
    assert_eq!(edited, format!("IR:{text}"));
    // The edit reflows only its own paragraph: every other flow paragraph is
    // still painted on its stored rows.
    let rows = rendered_rows(&core);
    let paragraphs = &core.document().sections[0].paragraphs;
    for (other, (starts, _)) in &rows {
        let p = &paragraphs[*other];
        if *other == pi || p.text.trim().is_empty() || p.line_segs.is_empty() {
            continue;
        }
        let head: String = p.text.chars().take(24).collect();
        assert_eq!(
            *starts,
            stored_rows(p),
            "rows of `{head}` survive an edit elsewhere"
        );
    }
}
