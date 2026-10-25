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

fn contains_marker(nodes: &[StructureNode], marker: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.marker == marker || contains_marker(&node.children, marker))
}

/// preamble·heading·body 를 모두 모은다.
///
/// negative 회귀는 "구조 노드가 없다"만 검사하면 문서를 통째로 못 읽어도 통과한다. 거부된 후보가
/// 텍스트로는 남아 있음을 함께 고정해 공허한 통과를 막는다.
fn all_text(structure: &StructureDoc) -> Vec<String> {
    fn walk(nodes: &[StructureNode], out: &mut Vec<String>) {
        for node in nodes {
            out.push(node.heading.clone());
            out.extend(node.body.iter().cloned());
            walk(&node.children, out);
        }
    }
    let mut out = structure.preamble.clone();
    walk(&structure.roots, &mut out);
    out
}

#[test]
fn real_work_plan_date_is_not_a_clause_item() {
    let structure = load_structure("samples/2022년 국립국어원 업무계획.hwp");

    assert!(!contains_marker(&structure.roots, "2022."));
    assert!(find_at(&structure.roots, 0, 10).is_none());

    // 날짜 문단이 구조 노드가 아닌 텍스트로 보존되는지까지 확인한다.
    let texts = all_text(&structure);
    assert!(
        texts
            .iter()
            .any(|text| text.trim_start().starts_with("2022.")),
        "날짜 문단이 텍스트로도 사라졌다면 negative 단언이 공허하게 통과한 것이다"
    );
}
