//! [#3744] explicit clause 문맥 만료·날짜·`목` confidence 회귀 가드.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

use rhwp::document_core::queries::structure::{
    build_structure, StructureDoc, StructureMode, StructureNode,
};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::ParaShape;
use rhwp::wasm_api::HwpDocument;

fn sample_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load_document(relative: &str) -> Document {
    let path = sample_path(relative);
    let bytes = fs::read(&path)
        .unwrap_or_else(|error| panic!("sample 읽기 실패 ({}): {error}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("sample 파싱 실패 ({}): {error:?}", path.display()))
        .document()
        .clone()
}

fn load_structure(relative: &str) -> StructureDoc {
    build_structure(&load_document(relative), StructureMode::Clause)
}

fn synthetic_structure(texts: &[&str]) -> StructureDoc {
    synthetic_structure_with_shape(texts, ParaShape::default())
}

fn synthetic_structure_with_shape(texts: &[&str], para_shape: ParaShape) -> StructureDoc {
    let mut document = Document::default();
    document.doc_info.para_shapes.push(para_shape);
    document.sections.push(Section {
        paragraphs: texts
            .iter()
            .map(|text| Paragraph {
                text: (*text).to_string(),
                ..Paragraph::new_empty()
            })
            .collect(),
        ..Section::default()
    });
    build_structure(&document, StructureMode::Clause)
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
fn stale_anchor_does_not_promote_oracle_sql_steps() {
    let structure = load_structure("samples/hwp3-sample10.hwp");

    assert_eq!(
        find_at(&structure.roots, 0, 2269).map(|node| (node.kind, node.marker.as_str())),
        Some(("항", "①"))
    );
    assert_eq!(
        find_at(&structure.roots, 0, 2270).map(|node| (node.kind, node.marker.as_str())),
        Some(("항", "②"))
    );
    for paragraph in [2303, 2312, 2313] {
        assert!(
            find_at(&structure.roots, 0, paragraph).is_none(),
            "stale 항 anchor 뒤 SQL 문단 {paragraph}은 호 node가 아니어야 한다"
        );
    }

    let text = all_text(&structure);
    for sql in [
        "1) back up the datafiles",
        "1) startup nomount;",
        "2) alter database mount standby database;",
    ] {
        assert!(
            text.iter().any(|line| line.trim() == sql),
            "거부된 SQL 문단도 body text로 보존되어야 한다: {sql}"
        );
    }
}

#[test]
fn compound_number_boundary_expires_only_the_current_weak_anchor() {
    let structure = synthetic_structure(&[
        "① 일반 번호 문맥",
        "1. 첫째",
        "2. 둘째",
        "2.1 일반 문서의 세부 번호",
        "1) 경계 뒤 일반 목록",
        "② 새 항",
        "1. 새 항의 첫째 호",
    ]);

    assert_eq!(
        find_at(&structure.roots, 0, 1).map(|node| (node.kind, node.marker.as_str())),
        Some(("호", "1."))
    );
    assert_eq!(
        find_at(&structure.roots, 0, 2).map(|node| (node.kind, node.marker.as_str())),
        Some(("호", "2."))
    );
    assert!(find_at(&structure.roots, 0, 3).is_none());
    assert!(find_at(&structure.roots, 0, 4).is_none());
    assert_eq!(
        find_at(&structure.roots, 0, 6).map(|node| (node.kind, node.marker.as_str())),
        Some(("호", "1."))
    );

    let text = all_text(&structure);
    assert!(text.iter().any(|line| line == "2.1 일반 문서의 세부 번호"));
    assert!(text.iter().any(|line| line == "1) 경계 뒤 일반 목록"));
}

#[test]
fn compound_number_boundary_recovers_a_continuing_ho_sequence() {
    for compound_body in [
        "3.5퍼센트를 곱한 금액으로 한다.",
        "0.5퍼센트를 가산한다.",
        "3.14 이하인 경우",
        "2.1항의 규정에 따른다.",
    ] {
        let structure = synthetic_structure(&[
            "제12조(보험료율)",
            "1. 일반요율",
            "2. 특별요율",
            compound_body,
            "3. 할인요율",
            "4. 할증요율",
        ]);

        assert!(
            find_at(&structure.roots, 0, 3).is_none(),
            "복합 번호처럼 보이는 본문은 호 node가 아니어야 한다: {compound_body}"
        );
        for (paragraph, marker) in [(4, "3."), (5, "4.")] {
            assert_eq!(
                find_at(&structure.roots, 0, paragraph)
                    .map(|node| (node.kind, node.marker.as_str())),
                Some(("호", marker)),
                "복합 번호 본문 뒤 정상 호는 회복되어야 한다: {compound_body}"
            );
        }
        assert!(all_text(&structure)
            .iter()
            .any(|text| text == compound_body));
    }
}

#[test]
fn revision_date_inside_article_remains_body_text() {
    let structure = synthetic_structure(&[
        "제1조(목적)",
        "2022. 1. 1. 일부개정",
        "1. 정상 후속 호",
        "적용한다.",
    ]);

    assert_eq!(structure.node_count, 2);
    assert!(find_at(&structure.roots, 0, 1).is_none());
    assert_eq!(
        find_at(&structure.roots, 0, 2).map(|node| (node.kind, node.marker.as_str())),
        Some(("호", "1.")),
        "날짜 거부는 현재 조 anchor를 만료하지 않아야 한다"
    );
    assert!(all_text(&structure)
        .iter()
        .any(|text| text == "2022. 1. 1. 일부개정"));
}

#[test]
fn direct_mok_under_section_is_a_heading() {
    let structure = synthetic_structure(&["제1장 총칙", "제1절 일반", "가. 본문 제목", "설명"]);

    let mok = find_at(&structure.roots, 0, 2).expect("절 직속 가. 제목");
    assert_eq!((mok.kind, mok.marker.as_str()), ("목", "가."));
    assert_eq!(mok.body, vec!["설명"]);
}

#[test]
fn real_handbook_keeps_representative_direct_mok_headings() {
    let structure = load_structure("samples/2025 행정업무운영 편람(최종).hwp");

    for (section, paragraph, marker) in [
        (1, 7, "가."),
        (1, 9, "나."),
        (2, 219, "가."),
        (2, 222, "나."),
    ] {
        assert_eq!(
            find_at(&structure.roots, section, paragraph)
                .map(|node| (node.kind, node.marker.as_str())),
            Some(("목", marker)),
            "편람 section={section} paragraph={paragraph}"
        );
    }
}

#[test]
fn real_market_report_keeps_matching_direct_mok_heading() {
    let structure = load_structure(
        "samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp",
    );

    assert_eq!(
        find_at(&structure.roots, 0, 355).map(|node| (
            node.kind,
            node.marker.as_str(),
            node.heading.trim()
        )),
        Some(("목", "가.", "가. 표준산업분류"))
    );
}

#[test]
fn toc_mok_with_page_tail_remains_body_text() {
    let structure = synthetic_structure(&["제1장 총칙", "제1절 일반", "가. 개요\t9"]);

    assert_eq!(structure.node_count, 2);
    assert!(find_at(&structure.roots, 0, 2).is_none());
    assert!(all_text(&structure)
        .iter()
        .any(|text| text == "가. 개요\t9"));
}

#[test]
fn toc_mok_with_dotted_leader_remains_body_text() {
    for toc_line in [
        "가. 개요 ·········· 9",
        "가. 개요 .......... 9",
        "가. 개요 …… 9",
    ] {
        let structure = synthetic_structure(&["제1장 총칙", "제1절 일반", toc_line]);

        assert_eq!(structure.node_count, 2, "점선 목차: {toc_line}");
        assert!(find_at(&structure.roots, 0, 2).is_none());
        assert!(all_text(&structure).iter().any(|text| text == toc_line));
    }
}

#[test]
fn direct_mok_shape_gate_rejects_indented_or_nested_paragraphs() {
    let mut accepted_boundary = ParaShape::default();
    accepted_boundary.indent = -1280;
    let accepted = synthetic_structure_with_shape(
        &["제1장 총칙", "제1절 일반", "가. 본문 제목"],
        accepted_boundary,
    );
    assert_eq!(
        find_at(&accepted.roots, 0, 2).map(|node| (node.kind, node.marker.as_str())),
        Some(("목", "가."))
    );

    let mut shapes = Vec::new();
    let mut margin = ParaShape::default();
    margin.margin_left = 1000;
    shapes.push(("margin_left", margin));

    let mut indent = ParaShape::default();
    indent.indent = -1281;
    shapes.push(("indent", indent));

    let mut level = ParaShape::default();
    level.para_level = 1;
    shapes.push(("para_level", level));

    for (axis, shape) in shapes {
        let structure = synthetic_structure_with_shape(
            &["제1장 총칙", "제1절 일반", "가. 일반 하위 목록"],
            shape,
        );
        assert_eq!(structure.node_count, 2, "shape negative axis: {axis}");
        assert!(find_at(&structure.roots, 0, 2).is_none());
        assert!(all_text(&structure)
            .iter()
            .any(|text| text == "가. 일반 하위 목록"));
    }
}
