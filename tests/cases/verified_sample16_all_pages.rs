//! #8160: 새 Hancom Print와 Native/fresh 표준 WASM 전 384쪽 raw 100% 뒤 등록.
//! 세 원본 각각 64쪽의 독립 내용·기준선과 원본 본문/셀/글상자 주소를 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use rhwp::DocumentCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

#[path = "../support/sample16_table_picture_rows.rs"]
mod table_picture_rows;

#[derive(Deserialize)]
struct Contract {
    sample: String,
    sample_sha256: String,
    pdf: String,
    pdf_sha256: String,
    compat2024: bool,
    dpi: u32,
    coordinate_tolerance_px: u32,
    pages: Vec<Page>,
}

#[derive(Deserialize)]
struct Page {
    page: usize,
    anchors: Vec<Anchor>,
}

#[derive(Deserialize)]
struct Anchor {
    text: String,
    section: usize,
    paragraph: usize,
    parent: usize,
    path: Vec<[usize; 3]>,
    textbox: bool,
    start: usize,
    baseline: f64,
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn runs(line: &RenderNode) -> Vec<(&RenderNode, &TextRunNode)> {
    line.children
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                Some((node, run))
            } else {
                None
            }
        })
        .collect()
}

fn lines<'a>(node: &'a RenderNode, body: bool, out: &mut Vec<&'a RenderNode>) {
    let body = body || matches!(node.node_type, RenderNodeType::Body { .. });
    if body && matches!(node.node_type, RenderNodeType::TextLine(_)) && !runs(node).is_empty() {
        out.push(node);
    }
    for child in &node.children {
        lines(child, body, out);
    }
}

fn owned(run: &TextRunNode, anchor: &Anchor) -> bool {
    if (run.section_index, run.para_index) != (Some(anchor.section), Some(anchor.paragraph)) {
        return false;
    }
    match &run.cell_context {
        None => anchor.path.is_empty() && anchor.parent == anchor.paragraph && !anchor.textbox,
        Some(context) => {
            context.parent_para_index == anchor.parent
                && context.in_textbox == anchor.textbox
                && context
                    .path
                    .iter()
                    .map(|e| [e.control_index, e.cell_index, e.cell_para_index])
                    .eq(anchor.path.iter().copied())
        }
    }
}

fn source<'a>(core: &'a DocumentCore, anchor: &Anchor) -> &'a Paragraph {
    let mut paragraph = &core.document().sections[anchor.section].paragraphs[anchor.parent];
    for &[control, cell, inner] in &anchor.path {
        paragraph = match &paragraph.controls[control] {
            Control::Table(table) => &table.cells[cell].paragraphs[inner],
            Control::Shape(shape) => {
                assert_eq!(cell, 0, "글상자 원본 주소");
                let drawing = match shape.as_ref() {
                    ShapeObject::Rectangle(shape) => &shape.drawing,
                    ShapeObject::Ellipse(shape) => &shape.drawing,
                    ShapeObject::Polygon(shape) => &shape.drawing,
                    ShapeObject::Curve(shape) => &shape.drawing,
                    _ => panic!("기준 내용의 실제 원본 글상자"),
                };
                &drawing
                    .text_box
                    .as_ref()
                    .expect("원본 글상자 내용")
                    .paragraphs[inner]
            }
            _ => panic!("기준 내용의 실제 원본 컨테이너"),
        };
    }
    paragraph
}

fn verify(fixture: &str) {
    let contract: Contract = serde_json::from_str(fixture).expect("독립 Print 계약");
    assert_eq!((contract.dpi, contract.coordinate_tolerance_px), (96, 2));
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join(&contract.sample)).unwrap();
    assert_eq!(sha256(&bytes), contract.sample_sha256);
    let pdf = std::fs::read(root.join(&contract.pdf)).unwrap();
    assert_eq!(sha256(&pdf), contract.pdf_sha256);
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(contract.compat2024);
    assert_eq!(contract.pages.len(), 64, "독립 PDF의 전쪽 계약 누락 없음");
    assert_eq!(
        core.page_count() as usize,
        contract.pages.len(),
        "내용 보존과 함께 확인하는 쪽수"
    );

    table_picture_rows::verify(&core);
    let mut seen = BTreeSet::new();
    for (index, page) in contract.pages.iter().enumerate() {
        assert_eq!(page.page, index + 1, "전체 쪽 순서·중복·누락");
        assert!(
            !page.anchors.is_empty(),
            "표지·목차·본문·서식 모두 중요 내용 보호"
        );
        let tree = core.build_page_render_tree(index as u32).unwrap();
        let mut body = Vec::new();
        lines(&tree.root, false, &mut body);
        let mut previous_baseline = f64::NEG_INFINITY;
        for anchor in &page.anchors {
            let found: Vec<_> = body
                .iter()
                .copied()
                .filter(|line| {
                    let text: String = runs(line)
                        .iter()
                        .map(|(_, run)| run.display_or_text())
                        .collect();
                    compact(&text) == anchor.text
                        && runs(line).iter().any(|(_, run)| {
                            owned(run, anchor) && run.char_start == Some(anchor.start)
                        })
                })
                .collect();
            assert_eq!(
                found.len(),
                1,
                "{} {}쪽 원문 소유·누락·중복: {}",
                contract.sample,
                page.page,
                anchor.text
            );
            let line = found[0];
            let original = source(&core, anchor);
            let mut prior_end = None;
            for (node, run) in runs(line) {
                assert!(owned(run, anchor), "같은 줄의 원본 본문/셀/글상자 소유");
                let start = run.char_start.expect("원본 문자 위치");
                let end = start + run.text.chars().count();
                let source_text: String = original
                    .text
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect();
                assert_eq!(run.text, source_text, "원본 런 내용·소유·문자 순서");
                if let Some(prior) = prior_end {
                    assert!(start >= prior, "문자 범위 중복·역전 없음");
                }
                prior_end = Some(end);
                let actual_baseline = node.bbox.y + run.baseline;
                assert!(
                    (actual_baseline - anchor.baseline).abs() <= 2.0,
                    "{} {}쪽 독립 Print 기준선 ±2px: {}, expected={}, actual={}",
                    contract.sample,
                    page.page,
                    anchor.text,
                    anchor.baseline,
                    actual_baseline
                );
                assert!(
                    node.bbox.x >= tree.root.bbox.x
                        && node.bbox.x + node.bbox.width <= tree.root.bbox.x + tree.root.bbox.width,
                    "원문 런의 가로 점유는 실제 쪽에 포함"
                );
            }
            assert!(
                anchor.baseline >= previous_baseline,
                "독립 Print의 위아래 내용 순서"
            );
            previous_baseline = anchor.baseline;
            assert!(
                seen.insert((
                    anchor.section,
                    anchor.parent,
                    anchor.path.clone(),
                    anchor.paragraph,
                    anchor.start
                )),
                "선정 원문을 다른 쪽에 중복 배치하지 않는다"
            );
        }
    }
}

#[test]
fn hwpx_all_64_pages_preserve_independent_print_content_and_owners() {
    verify(include_str!("../fixtures/issue8160/hwpx.json"));
}

#[test]
fn hwp2024_all_64_pages_preserve_independent_print_content_and_owners() {
    verify(include_str!("../fixtures/issue8160/hwp2024.json"));
}

#[test]
fn hwp2022_all_64_pages_preserve_independent_print_content_and_owners() {
    verify(include_str!("../fixtures/issue8160/hwp2022.json"));
}
