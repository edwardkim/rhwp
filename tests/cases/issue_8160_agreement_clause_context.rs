//! #8160: sample16 전용 회귀. 이전 등록: tests/issue_3693_structure_clause_context.rs
//! [#3693] `export-structure --mode clause` marker·문맥 회귀 테스트.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::queries::structure::{
    build_structure, StructureDoc, StructureMode, StructureNode,
};
use rhwp::wasm_api::HwpDocument;

fn sample_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load_structure(relative: &str) -> StructureDoc {
    let path = sample_path(relative);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("sample 읽기 실패 ({}): {error}", path.display()));
    let document = HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("sample 파싱 실패 ({}): {error:?}", path.display()));
    build_structure(document.document(), StructureMode::Clause)
}

fn find_at(nodes: &[StructureNode], section: usize, paragraph: usize) -> Option<&StructureNode> {
    for node in nodes {
        if node.section == section && node.paragraph == paragraph {
            return Some(node);
        }
        if let Some(found) = find_at(&node.children, section, paragraph) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_8160_real_agreement_keeps_items_under_article() {
    let structure = load_structure("samples/hwp3-sample16-hwp5.hwp");
    let article = find_at(&structure.roots, 0, 945).expect("협정서 제1조");

    assert_eq!((article.kind, article.marker.as_str()), ("조", "제1조"));
    let child_markers: Vec<&str> = article
        .children
        .iter()
        .map(|node| node.marker.as_str())
        .collect();
    assert_eq!(child_markers, vec!["1.", "2.", "3."]);
}
