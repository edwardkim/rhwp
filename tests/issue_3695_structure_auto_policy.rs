//! [#3695] `export-structure --mode auto` 증거 우선순위 회귀 테스트.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::queries::structure::{build_structure, StructureDoc, StructureMode};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{HeadType, ParaShape};

fn synthetic_document(paragraphs: &[(&str, HeadType, u8)]) -> Document {
    let mut document = Document::default();
    for (_, head_type, para_level) in paragraphs {
        document.doc_info.para_shapes.push(ParaShape {
            head_type: *head_type,
            para_level: *para_level,
            ..ParaShape::default()
        });
    }
    document.sections.push(Section {
        paragraphs: paragraphs
            .iter()
            .enumerate()
            .map(|(index, (text, _, _))| Paragraph {
                text: (*text).to_string(),
                para_shape_id: index as u16,
                ..Paragraph::new_empty()
            })
            .collect(),
        ..Section::default()
    });
    document
}

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

fn load_auto(relative: &str) -> StructureDoc {
    let document = load_document(relative);
    build_structure(&document, StructureMode::Auto)
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
fn auto_primary_clause_marker_outweighs_ambiguous_number() {
    let document = synthetic_document(&[
        ("제1조(목적)", HeadType::None, 0),
        ("이 규정은 목적을 정한다.", HeadType::None, 0),
        ("자동번호가 지정된 일반 문단", HeadType::Number, 0),
    ]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "clause");
    assert_eq!(structure.node_count, 1);
    assert_eq!(structure.roots[0].marker, "제1조");
    assert_eq!(
        structure.roots[0].body,
        vec!["이 규정은 목적을 정한다.", "자동번호가 지정된 일반 문단"]
    );
}

#[test]
fn auto_article_title_starting_with_particle_syllable_remains_clause() {
    let document = synthetic_document(&[
        ("제1조의무 규정", HeadType::None, 0),
        ("자동번호 참고 목록", HeadType::Number, 0),
    ]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "clause");
    assert_eq!(structure.roots[0].marker, "제1조");
}

#[test]
fn explicit_modes_remain_available_for_mixed_evidence() {
    let document = synthetic_document(&[
        ("제1조(목적)", HeadType::None, 0),
        ("자동번호 문단", HeadType::Number, 2),
    ]);

    let outline = build_structure(&document, StructureMode::Outline);
    assert_eq!(outline.mode, "outline");
    assert_eq!(outline.node_count, 1);
    assert_eq!(outline.roots[0].heading, "자동번호 문단");
    assert_eq!(outline.roots[0].level, 3);

    let clause = build_structure(&document, StructureMode::Clause);
    assert_eq!(clause.mode, "clause");
    assert_eq!(clause.node_count, 1);
    assert_eq!(clause.roots[0].marker, "제1조");
}

#[test]
fn auto_explicit_outline_is_authoritative() {
    let document = synthetic_document(&[("제1조처럼 보이는 개요 제목", HeadType::Outline, 1)]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 1);
    assert_eq!(structure.roots[0].kind, "outline");
    assert_eq!(structure.roots[0].level, 2);
}

#[test]
fn auto_single_number_only_document_remains_outline() {
    let document = synthetic_document(&[("유일한 자동번호 제목", HeadType::Number, 0)]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 1);
    assert_eq!(structure.roots[0].heading, "유일한 자동번호 제목");
}

#[test]
fn auto_outline_cross_reference_does_not_become_clause_evidence() {
    let document = synthetic_document(&[
        ("1. 사업 개요", HeadType::Number, 0),
        ("가. 추진 배경", HeadType::Number, 1),
        ("나. 기대 효과", HeadType::Number, 1),
        (
            "제3조의 규정에 따라 관계기관 협의를 거쳐 시행한다.",
            HeadType::None,
            0,
        ),
    ]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 3);
    assert_eq!(structure.roots[0].heading, "1. 사업 개요");
}

#[test]
fn auto_toc_article_page_tail_does_not_become_clause_evidence() {
    let document = synthetic_document(&[
        ("사업 개요", HeadType::Number, 0),
        ("제1조 목적\t12", HeadType::None, 0),
    ]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 1);
    assert_eq!(structure.roots[0].heading, "사업 개요");
}

#[test]
fn auto_pure_clause_document_remains_clause() {
    let document = synthetic_document(&[
        ("제1조(목적)", HeadType::None, 0),
        ("① 이 규정은 목적을 정한다.", HeadType::None, 0),
    ]);

    let structure = build_structure(&document, StructureMode::Auto);

    assert_eq!(structure.mode, "clause");
    assert_eq!(structure.node_count, 2);
    assert_eq!(structure.roots[0].marker, "제1조");
    assert_eq!(structure.roots[0].children[0].marker, "①");
}

#[test]
fn real_explicit_outline_document_remains_outline() {
    let structure = load_auto("mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp");

    assert_eq!(structure.mode, "outline");
    assert!(structure.node_count > 100, "실제 개요 노드가 사라짐");
}

#[test]
fn real_number_only_document_remains_outline() {
    let structure = load_auto("samples/biz_plan.hwp");

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 20);
}

#[test]
fn real_single_number_document_remains_outline() {
    let structure = load_auto("samples/추진일정.hwp");

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 1);
}

#[test]
fn real_market_report_with_section_markers_remains_outline() {
    let structure =
        load_auto("samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp");

    assert_eq!(structure.mode, "outline");
    assert_eq!(structure.node_count, 3);
}

#[test]
fn real_clause_document_with_ambiguous_number_evidence_remains_clause() {
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
