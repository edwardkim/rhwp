//! 독립 한컴 Print로 검증한 분할 표 뒤 문단의 간격·쪽 소유 계약.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
fn sha(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}
fn walk<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        walk(c, out);
    }
}
fn text(n: &RenderNode) -> String {
    let mut nodes = vec![];
    walk(n, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| {
            if let RenderNodeType::TextRun(r) = &n.node_type {
                Some(r.text.as_str())
            } else {
                None
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect()
}
fn check(key: &str) {
    let all: Value = serde_json::from_str(include_str!(
        "../../mydocs/pr/assets/computed_successor_spacing_20261025/multiline_coordinates.json"
    ))
    .unwrap();
    let e = &all[key];
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(e["input"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(sha(&bytes), e["input_sha256"].as_str().unwrap());
    assert_eq!(
        sha(&std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(e["pdf"].as_str().unwrap())
        )
        .unwrap()),
        e["pdf_sha256"].as_str().unwrap()
    );
    let c = DocumentCore::from_bytes(&bytes).unwrap();
    let expected = e["pages"].as_array().unwrap();
    assert_eq!(c.page_count() as usize, expected.len());
    let mut source = String::new();
    for (page, wanted) in expected.iter().enumerate() {
        let tree = c.build_page_render_tree(page as u32).unwrap();
        let mut nodes = vec![];
        walk(&tree.root, &mut nodes);
        let actual: Vec<_> = nodes
            .iter()
            .filter(|n| {
                matches!(&n.node_type, RenderNodeType::TextLine(l)
                    if l.section_index == Some(0) && l.para_index == Some(1))
            })
            .collect();
        let wanted = wanted.as_array().unwrap();
        let page_text: String = wanted
            .iter()
            .map(|line| line["text"].as_str().unwrap())
            .collect();
        assert_eq!(
            text(&tree.root),
            page_text,
            "소유 문단 밖의 누락·중복도 검출한다"
        );
        let tables: Vec<_> = nodes
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)))
            .collect();
        assert_eq!(tables.len(), usize::from(page < 2), "두 표 조각의 쪽 소유");
        assert_eq!(
            actual.len(),
            wanted.len(),
            "{key} page{}: 같은 문단 글줄의 쪽 소유",
            page + 1
        );
        for (n, line) in actual.iter().zip(wanted) {
            if let Some(table) = tables.first() {
                assert!(
                    n.bbox.y >= table.bbox.y + table.bbox.height,
                    "뒤 본문은 마지막 표 조각 아래에 놓인다"
                );
            }
            let rendered = text(n);
            assert_eq!(
                rendered,
                line["text"].as_str().unwrap(),
                "줄 순서·누락·중복"
            );
            source.push_str(&rendered);
            for (a, index) in [(n.bbox.x, 0), (n.bbox.y, 1)] {
                let independent = line["bbox"][index].as_f64().unwrap();
                assert!(
                    (a - independent).abs() <= 2.0,
                    "{key} page{} axis{index}: actual={a}, Print={independent}",
                    page + 1
                );
            }
        }
    }
    assert_eq!(source, e["source_text"].as_str().unwrap());
}

#[test]
fn verified_split_successor_multiline_before_105000_hwp() {
    check("split-successor-multiline-before-105000-hwp");
}

#[test]
fn verified_split_successor_multiline_before_105000_hwpx() {
    check("split-successor-multiline-before-105000-hwpx");
}
