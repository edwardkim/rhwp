//! 검사 유형: 관계형·고정형.
//! 2026-10-27 측정 raw(96dpi, PDFium, PDF fidelity 가져오기): 21_언어_기출_편집가능본-2010.pdf 7쪽 Native 99.83396% / fresh WASM 99.77416%; 전16쪽 최저 98.80885% / 98.69530%.
//! 21_언어_기출_편집가능본-2020.pdf 7쪽 99.59383% / 99.19060%; 전15쪽 최저 99.22374% / 98.69112%.
//! 21_언어_기출_편집가능본.pdf 7쪽 99.77490% / 99.73443%; 전15쪽 최저 98.52190% / 98.44719%.
//! 각 수치 쌍은 Native / fresh WASM 순서이며 현재 head의 새 측정이 아닌 당시 측정이다.
//! 근거: mydocs/working/assets/review_integration_20261026/validation/visual-validation.json,
//! mydocs/working/assets/review_integration_20261026/validation/tsv/native/pdf-2010-fidelity.tsv,
//! mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/pdf-2010-fidelity.tsv.
//! 2010 PDF 7쪽 원본 글줄 좌표: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/pdf2010-exam-p7-coordinates.json.
//!
//! #8220: untagged exam PDFs printed from one HWP. Imported body paragraphs
//! follow the author's paragraph boundaries. The independent expectation is the source
//! HWP's own paragraph text; the additional coordinate comes from the source PDF.
//!
//! Boundaries exercised here, each on real Hancom output:
//! - a short last line followed by an indented or flush next line,
//! - item markers (`①`..`⑤`) on the first-line start of hanging choices,
//! - a centered box title (`<보기>`) above a full box line whose text carries
//!   no space characters, so words end at gaps between source spans,
//! - a spanning title that abuts the two text columns,
//! - columns whose baselines are offset from each other.
#![cfg(feature = "pdf-import")]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::{
    model::{control::Control, paragraph::Paragraph, style::PARA_ATTR2_LINE_WRAP_MASK},
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::collections::BTreeMap;

const SOURCE: &str = "samples/21_언어_기출_편집가능본.hwp";

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
                Control::Table(t) => {
                    for cell in &t.cells {
                        collect(&cell.paragraphs, out);
                    }
                }
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

/// Wrapping body paragraphs per output section (one per PDF page). Positioned
/// source spans keep a single-line paragraph shape and are not body text.
fn body_paragraphs(core: &DocumentCore) -> Vec<Vec<String>> {
    let shapes = &core.document().doc_info.para_shapes;
    core.document()
        .sections
        .iter()
        .map(|section| {
            let mut out = Vec::new();
            for p in &section.paragraphs {
                for control in &p.controls {
                    let Control::Shape(s) = control else { continue };
                    let Some(b) = s.drawing().and_then(|d| d.text_box.as_ref()) else {
                        continue;
                    };
                    for q in &b.paragraphs {
                        let wraps = shapes
                            .get(q.para_shape_id as usize)
                            .is_some_and(|ps| ps.attr2 & PARA_ATTR2_LINE_WRAP_MASK == 0);
                        let text = norm(&q.text);
                        if wraps && !text.is_empty() {
                            out.push(text);
                        }
                    }
                }
            }
            out
        })
        .collect()
}

fn import(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &PdfImportOptions::default())
        .unwrap()
}

/// The single source paragraph that starts with `prefix`.
fn source(sources: &[String], prefix: &str) -> String {
    let found: Vec<_> = sources.iter().filter(|t| t.starts_with(prefix)).collect();
    assert_eq!(found.len(), 1, "source paragraph `{prefix}` is unique");
    found[0].clone()
}

/// The five choices that follow `question` in the source, in order.
fn choices(sources: &[String], question: &str) -> Vec<String> {
    let at = sources
        .iter()
        .position(|t| t == question)
        .expect("question paragraph");
    ["①", "②", "③", "④", "⑤"]
        .iter()
        .map(|m| {
            sources[at + 1..]
                .iter()
                .find(|t| t.starts_with(m))
                .unwrap()
                .clone()
        })
        .collect()
}

fn verify(core: &DocumentCore, sources: &[String], label: &str) {
    let pages = body_paragraphs(core);
    let all: Vec<&String> = pages.iter().flatten().collect();
    let mut wanted: Vec<String> = [
        // Page 1, below a title that abuts both text columns. The first two
        // end on a short last line before an indented first line.
        "프로세스마이닝은정보시스템을통해확보한",
        "적합성검증이란기존의프로세스모델과",
        "프로세스향상에는두유형이있다.",
        "1.윗글과일치하는것은?",
        "4.윗글과일치하지않는것은?",
        // First box paragraphs below a centered `<보기>` title.
        "◯◯병원에서는외래환자의",
        "상고시대에유배형은",
    ]
    .iter()
    .map(|p| source(sources, p))
    .collect();
    // Hanging choices: each item marker opens its own paragraph.
    let groups = [
        choices(sources, &source(sources, "1.윗글과일치하는것은?")),
        choices(sources, &source(sources, "4.윗글과일치하지않는것은?")),
    ];
    wanted.extend(groups.iter().flatten().cloned());
    // Every expected paragraph becomes exactly one body paragraph, whole.
    for text in &wanted {
        let n = all.iter().filter(|t| **t == text).count();
        let head: String = text.chars().take(30).collect();
        assert_eq!(n, 1, "{label}: one body paragraph owns `{head}`");
    }
    // Choices stay in source order on one page.
    for group in &groups {
        let page = pages
            .iter()
            .find(|p| p.contains(&group[0]))
            .expect("choice page");
        let order: Vec<_> = group
            .iter()
            .map(|c| page.iter().position(|t| t == c))
            .collect();
        assert!(
            order.iter().all(Option::is_some),
            "{label}: choices share one page"
        );
        assert!(
            order.windows(2).all(|w| w[0] < w[1]),
            "{label}: choices keep source order"
        );
    }
    // No body paragraph swallows separate choice items. Short choices the
    // author set on one tabbed line stay one source paragraph.
    for text in &all {
        let markers = text.chars().filter(|c| ('①'..='⑤').contains(c)).count();
        assert!(
            markers <= 1 || sources.contains(text),
            "{label}: separate choice items share `{text}`"
        );
    }
}

#[test]
fn exam_pdf_body_paragraphs_follow_source_paragraph_boundaries() {
    let sources = source_paragraphs();
    for name in [
        "21_언어_기출_편집가능본-2020.pdf",
        "21_언어_기출_편집가능본-2010.pdf",
        "21_언어_기출_편집가능본.pdf",
    ] {
        let core = import(name);
        verify(&core, &sources, name);
        // The same paragraphs survive saving and reopening.
        for data in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            verify(&DocumentCore::from_bytes(&data).unwrap(), &sources, name);
        }
    }
}

#[test]
fn offset_column_baselines_still_separate_the_columns() {
    // 2010 page 7: the right column's grid is shifted by a heading, so no two
    // baselines pair across the gutter. Each column keeps its own paragraphs.
    let sources = source_paragraphs();
    let core = import("21_언어_기출_편집가능본-2010.pdf");
    let pages = body_paragraphs(&core);
    let left = source(&sources, "결국르포르는권력이제어할수있는틀을넘어");
    let mut owners: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, page) in pages.iter().enumerate() {
        *owners.entry(i).or_default() += page.iter().filter(|t| **t == left).count();
    }
    assert_eq!(
        owners.values().sum::<usize>(),
        1,
        "one body paragraph owns the left-column paragraph"
    );

    fn text(node: &RenderNode) -> String {
        let mut result = match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.clone(),
            _ => String::new(),
        };
        for child in &node.children {
            result.push_str(&text(child));
        }
        result
    }
    fn target(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && norm(&text(node)).starts_with("결국르포르는권력이제어할수있는")
        {
            return Some(node);
        }
        node.children.iter().find_map(target)
    }
    let tree = core.build_page_render_tree(6).expect("source PDF page 7");
    let line = target(&tree.root).expect("source paragraph first line");
    assert!(
        (line.bbox.x - 603.98).abs() <= 2.0,
        "source PDF line x: {}",
        line.bbox.x
    );
    assert!(
        (line.bbox.y - 388.54).abs() <= 2.0,
        "source PDF line y: {}",
        line.bbox.y
    );
}
