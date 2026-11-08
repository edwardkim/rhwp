//! #8160: sample16 전용 회귀. 이전 등록: tests/issue_3695_structure_auto_policy.rs
//! [#3695] `export-structure --mode auto` 증거 우선순위 회귀 테스트.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::queries::structure::{build_structure, StructureMode};
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{HeadType, ParaShape};

fn sample_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load_document(relative: &str) -> Document {
    let path = sample_path(relative);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("sample 읽기 실패 ({}): {error}", path.display()));
    rhwp::parser::parse_document(&bytes)
        .unwrap_or_else(|error| panic!("sample 파싱 실패 ({}): {error:?}", path.display()))
}

fn contains_marker(
    nodes: &[rhwp::document_core::queries::structure::StructureNode],
    marker: &str,
) -> bool {
    nodes
        .iter()
        .any(|node| node.marker == marker || contains_marker(&node.children, marker))
}

#[test]
fn issue_8160_real_clause_document_with_ambiguous_number_evidence_remains_clause() {
    let mut document = load_document("samples/hwp3-sample16-hwp5.hwp");
    let number_shape_id = document.doc_info.para_shapes.len() as u16;
    document.doc_info.para_shapes.push(ParaShape {
        head_type: HeadType::Number,
        para_level: 0,
        ..ParaShape::default()
    });
    document.sections[0].paragraphs.push(Paragraph {
        text: "자동번호 참고 목록".to_string(),
        para_shape_id: number_shape_id,
        ..Paragraph::new_empty()
    });

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "clause");
    assert!(contains_marker(&structure.roots, "제1조"));
}
