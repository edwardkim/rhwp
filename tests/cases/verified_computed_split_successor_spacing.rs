//! 독립 한컴 Print로 검증한 분할 표 뒤 문단의 간격·쪽 소유 계약.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn walk<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        walk(c, out);
    }
}
fn check(key: &str) {
    let expected: Value = serde_json::from_str(include_str!(
        "../../mydocs/pr/assets/computed_successor_spacing_20261025/spacing_coordinates.json"
    ))
    .unwrap();
    let e = &expected[key];
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(e["input"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(sha(&bytes), e["input_sha256"].as_str().unwrap());
    let pdf = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(e["pdf"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(sha(&pdf), e["pdf_sha256"].as_str().unwrap());
    let c = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(c.page_count(), e["page_count"].as_u64().unwrap() as u32);
    let mut owned = Vec::new();
    let mut all_text = String::new();
    for page in 0..c.page_count() {
        let tree = c.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        for n in &nodes {
            if let RenderNodeType::TextRun(r) = &n.node_type {
                all_text.push_str(&r.text);
                if !r.text.trim().is_empty() {
                    assert_eq!(r.section_index, Some(0));
                    assert_eq!(r.para_index, Some(1), "뒤 본문의 원래 문단 소유를 보존한다");
                    owned.push((page, n.bbox));
                }
            }
        }
        let tables: Vec<_> = nodes
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)))
            .collect();
        assert_eq!(tables.len(), 1, "표 조각은 각 쪽에 한 번 배정한다");
        if page == e["successor"]["page"].as_u64().unwrap() as u32 {
            for (p, b) in &owned {
                if *p == page {
                    assert!(
                        b.y >= tables[0].bbox.y + tables[0].bbox.height,
                        "뒤 본문이 마지막 표 조각과 겹치지 않는다"
                    );
                }
            }
        }
    }
    assert_eq!(
        all_text.split_whitespace().collect::<String>(),
        "다음문단",
        "누락·중복·다른 쪽으로 이동을 허용하지 않는다"
    );
    assert_eq!(owned.len(), 1);
    let (page, b) = owned[0];
    assert_eq!(page, e["successor"]["page"].as_u64().unwrap() as u32);
    for (actual, index) in [(b.x, 0), (b.y, 1)] {
        let independent = e["successor"]["bbox"][index].as_f64().unwrap();
        assert!(
            (actual - independent).abs() <= 2.0,
            "{key}: 독립 Print 원점 {index}, actual={actual}, independent={independent}"
        );
    }
}

#[test]
fn verified_split_successor_before_0_hwp() {
    check("split-successor-before-0-hwp");
}

#[test]
fn verified_split_successor_before_0_hwpx() {
    check("split-successor-before-0-hwpx");
}

#[test]
fn verified_split_successor_before_30000_hwp() {
    check("split-successor-before-30000-hwp");
}

#[test]
fn verified_split_successor_before_30000_hwpx() {
    check("split-successor-before-30000-hwpx");
}

#[test]
fn verified_split_successor_before_105000_hwp() {
    check("split-successor-before-105000-hwp");
}

#[test]
fn verified_split_successor_before_105000_hwpx() {
    check("split-successor-before-105000-hwpx");
}
