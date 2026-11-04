//! 2026-10-29 source fa254efdfe, base f291d772a1, 96dpi: 보정한 3/12/13쪽은 Native 99.79/99.37/99.53%, fresh WASM 99.80/99.35/99.53%이며 직접 확대 판독을 마쳤다.
//! 세 editable PDF의 전쪽 양쪽 최저 98.48% 이상·독립 PDF 쪽수 일치. 새 ±2px 좌표는 원본 PDF의 PyMuPDF paint baseline에서 읽었다.
//! 현행 증거: mydocs/pr/assets/issue_8174/t09/verification_manifest.json 및 independent-pdf-baselines.json. 기존 본문 소유·정상 연결·편집·양쪽 저장 검사는 유지한다.
//! 검사 유형: 관계형·고정형.
//! 2026-10-27 측정 raw(96dpi, PDFium, PDF editable 가져오기): 21_언어_기출_편집가능본-2010.pdf 7쪽 Native 99.82145% / fresh WASM 99.77620%; 전16쪽 최저 98.80885% / 98.69835%.
//! 21_언어_기출_편집가능본-2020.pdf 7쪽 99.58789% / 99.18794%; 전15쪽 최저 99.13134% / 98.69133%.
//! 21_언어_기출_편집가능본.pdf 7쪽 99.77787% / 99.73748%; 전15쪽 최저 98.51800% / 98.44191%.
//! 수치 쌍은 Native / fresh WASM이며 당시 head의 편집 전 가져오기 결과다. 편집 후 결과는 별도 문서다.
//! 2026-10-28 같은 편집을 HWP로 저장·재열기한 입력 raw(96dpi, PDFium): 4쪽 Native 65.17416% / fresh WASM 64.97946%; 전16쪽 최저 1.58239% / 1.63218%.
//! 편집 저장본 근거: mydocs/pr/assets/pr8227-editable-after-save/evidence.json. 메모리 편집의 직접 측정은 아니다.
//! 근거: mydocs/working/assets/review_integration_20261026/validation/visual-validation.json.
//! 2010 PDF 7쪽 좌표 근거: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/pdf2010-exam-p7-coordinates.json.
//!
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
/// or page boundary.
const PDFS: [(&str, usize, &[&str]); 3] = [
    (
        "21_언어_기출_편집가능본-2020.pdf",
        15,
        &[
            "물론자신이풍경안에갇혀",
            "싱어는위와같은롤스의시도를",
            "물론르포르도새로운권리의",
        ],
    ),
    // The 2010 print draws its page title as paint, so only short running
    // header pieces sit above the columns; they must not open the columns.
    (
        "21_언어_기출_편집가능본-2010.pdf",
        16,
        &[
            "물론자신이풍경안에갇혀",
            "싱어는위와같은롤스의시도를",
            "물론르포르도새로운권리의",
        ],
    ),
    (
        "21_언어_기출_편집가능본.pdf",
        15,
        &["싱어는위와같은롤스의시도를", "물론르포르도새로운권리의"],
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

/// Left edge of each rendered row's first character, by paragraph, in row
/// order. Rows of one paragraph on one page and column only.
fn row_lefts(core: &DocumentCore) -> BTreeMap<(usize, usize, i64), Vec<(usize, f64)>> {
    fn walk(n: &RenderNode, page: usize, rows: &mut BTreeMap<(usize, usize, i64), (usize, f64)>) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if let (Some(0), Some(pi), Some(start), None) =
                (r.section_index, r.para_index, r.char_start, &r.cell_context)
            {
                let e = rows
                    .entry((pi, page, n.bbox.y.round() as i64))
                    .or_insert((start, n.bbox.x));
                if start < e.0 {
                    *e = (start, n.bbox.x);
                }
            }
        }
        for c in &n.children {
            walk(c, page, rows);
        }
    }
    let mut rows = BTreeMap::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        walk(&tree.root, page as usize, &mut rows);
    }
    let mut out: BTreeMap<(usize, usize, i64), Vec<(usize, f64)>> = BTreeMap::new();
    for ((pi, page, _), (start, x)) in rows {
        out.entry((pi, page, i64::from(x >= 560.0)))
            .or_default()
            .push((start, x));
    }
    for rows in out.values_mut() {
        rows.sort_by_key(|r| r.0);
    }
    out
}

/// Character index of each stored row start, read on the paragraph's own
/// stream axis (`line_seg_text_start`), the axis its characters use.
fn stored_rows(p: &Paragraph) -> BTreeSet<usize> {
    (0..p.line_segs.len())
        .map(|i| {
            let start = p.line_seg_text_start(i);
            p.char_offsets.iter().position(|&o| o >= start).unwrap_or(0)
        })
        .collect()
}

/// Every flow paragraph's text with the characters that start its rows.
fn row_starts(core: &DocumentCore) -> Vec<(String, BTreeSet<usize>)> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .filter(|p| !p.text.trim().is_empty())
        .map(|p| (norm(&p.text), stored_rows(p)))
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
    // A first-line indent shows on the paragraph's first row only.
    let mut indented = 0;
    for ((pi, _, _), rows) in row_lefts(core) {
        let p = &paragraphs[pi];
        let shape = &doc.doc_info.para_shapes[p.para_shape_id as usize];
        // Paragraph measures are twice HWPUNIT; 1pt is 100 HWPUNIT, 96/72 px.
        let expected = f64::from(shape.indent) / 2.0 / 100.0 * 96.0 / 72.0;
        if shape.indent >= 1000 && rows.len() >= 2 && rows[0].0 == 0 {
            let head: String = p.text.chars().take(24).collect();
            let shift = rows[0].1 - rows[1].1;
            assert!(
                (shift - expected).abs() <= 1.5,
                "{label}: first row of `{head}` is indented by its paragraph indent \
                 ({shift:.1}px, indent {expected:.1}px)"
            );
            indented += 1;
        }
    }
    assert!(
        indented >= 10,
        "{label}: indented body paragraphs are checked"
    );
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
        let rows = row_starts(&core);
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&data).unwrap();
            verify(&reopened, &sources, pages, joined, name);
            // Saving keeps every paragraph's rows on the same characters,
            // including the section's first paragraphs, whose row positions
            // the containers store on their own axes.
            assert_eq!(row_starts(&reopened), rows, "{name}: rows survive saving");
        }
    }
}

#[test]
fn editable_page_seven_keeps_independent_pdf_line_position() {
    fn text(node: &RenderNode) -> String {
        let mut value = match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.clone(),
            _ => String::new(),
        };
        for child in &node.children {
            value.push_str(&text(child));
        }
        value
    }
    fn target(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && norm(&text(node)).starts_with("결국르포르는권력이제어할수있는")
        {
            return Some(node);
        }
        node.children.iter().find_map(target)
    }
    let core = import(
        "21_언어_기출_편집가능본-2010.pdf",
        PdfImportLayout::Editable,
    );
    let tree = core.build_page_render_tree(6).expect("PDF page 7");
    let line = target(&tree.root).expect("source paragraph first line");
    assert!(
        (line.bbox.x - 603.98).abs() <= 2.0,
        "PDF line x: {}",
        line.bbox.x
    );
    assert!(
        (line.bbox.y - 388.54).abs() <= 2.0,
        "PDF line y: {}",
        line.bbox.y
    );
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

/// Independent baseline coordinates are read from the original PDF paint at
/// 96 dpi, rather than from the inferred HWP line grid. The first two macOS
/// rows remain positioned owners; the following body must occupy the next row.
#[test]
fn editable_keeps_positioned_prefix_and_observed_paragraph_tail_bands() {
    fn text(node: &RenderNode) -> String {
        let mut value = match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.clone(),
            _ => String::new(),
        };
        for child in &node.children {
            value.push_str(&text(child));
        }
        value
    }
    fn first_paint(node: &RenderNode) -> Option<(f64, f64)> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if !run.text.trim().is_empty() {
                return Some((node.bbox.x, node.bbox.y + run.baseline));
            }
        }
        node.children.iter().find_map(first_paint)
    }
    fn positions(node: &RenderNode, prefix: &str, out: &mut Vec<(f64, f64)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && norm(&text(node)).starts_with(prefix)
        {
            out.push(first_paint(node).expect("source row paints text"));
        }
        for child in &node.children {
            positions(child, prefix, out);
        }
    }
    let cases: &[(&str, u32, &[(&str, f64, f64)])] = &[
        (
            "21_언어_기출_편집가능본.pdf",
            2,
            &[
                ("의할", 128.48, 220.80),
                ("그렇다면", 141.76, 244.96),
                ("풍경화가아닌동양의산수화를", 128.48, 269.12),
            ],
        ),
        (
            "21_언어_기출_편집가능본-2020.pdf",
            11,
            &[("아래그래프에서S자곡선은", 617.88, 274.92)],
        ),
        (
            "21_언어_기출_편집가능본-2010.pdf",
            12,
            &[("아래그래프에서S자곡선은", 155.35, 276.36)],
        ),
    ];
    for &(name, page, rows) in cases {
        // Import the complete PDF so that page and column continuations are
        // exercised before observing these source rows.
        let core = import(name, PdfImportLayout::Editable);
        let tree = core.build_page_render_tree(page).unwrap();
        let mut previous_baseline = None;
        for &(prefix, x, baseline) in rows {
            let mut found = Vec::new();
            positions(&tree.root, prefix, &mut found);
            assert_eq!(
                found.len(),
                1,
                "{name} p{} `{prefix}` occurs once",
                page + 1
            );
            let actual = found[0];
            assert!(
                (actual.0 - x).abs() <= 2.0,
                "{name} `{prefix}` source x: {actual:?}"
            );
            assert!(
                (actual.1 - baseline).abs() <= 2.0,
                "{name} `{prefix}` source baseline: {actual:?}"
            );
            if let Some(previous) = previous_baseline {
                assert!(
                    actual.1 > previous,
                    "{name}: prefix and body keep source row order"
                );
            }
            previous_baseline = Some(actual.1);
        }
    }
}
