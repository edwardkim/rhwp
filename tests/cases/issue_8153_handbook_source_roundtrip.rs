//! 관계형: 두 원본의 본문·표 소유와 저장 후 바탕쪽 소속을 실제 왕복에서 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 기대값은 원본 입력에서 읽으며 과거 page/px baseline을 사용하지 않는다.
//! 근거: mydocs/plans/issue_8153_handbook_regression_replacement_20261025.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, paragraph::Paragraph};
use rhwp::{parse_document, serializer::hwpx::serialize_hwpx};
use std::path::Path;

fn nested_text(control: &Control) -> Option<(&'static str, &[Paragraph])> {
    match control {
        Control::Header(value) => Some(("header", &value.paragraphs)),
        Control::Footer(value) => Some(("footer", &value.paragraphs)),
        Control::Footnote(value) => Some(("footnote", &value.paragraphs)),
        Control::Endnote(value) => Some(("endnote", &value.paragraphs)),
        _ => None,
    }
}

fn paragraphs(before: &[Paragraph], after: &[Paragraph], path: &str) {
    assert_eq!(before.len(), after.len(), "{path}: 문단 누락·중복 금지");
    for (index, (original, saved)) in before.iter().zip(after).enumerate() {
        let owner = format!("{path}/p{index}");
        assert_eq!(original.text, saved.text, "{owner}: 본문 순서·내용 보존");
        let original_notes: Vec<_> = original.controls.iter().filter_map(nested_text).collect();
        let saved_notes: Vec<_> = saved.controls.iter().filter_map(nested_text).collect();
        assert_eq!(
            original_notes.len(),
            saved_notes.len(),
            "{owner}: 주석·머리말·꼬리말 소유"
        );
        for (ordinal, ((source_kind, source), (target_kind, target))) in
            original_notes.iter().zip(&saved_notes).enumerate()
        {
            assert_eq!(source_kind, target_kind, "{owner}: 중첩 본문 종류·순서");
            paragraphs(source, target, &format!("{owner}/{source_kind}{ordinal}"));
        }
        let original_tables: Vec<_> = original
            .controls
            .iter()
            .filter_map(|control| {
                if let Control::Table(table) = control {
                    Some(table)
                } else {
                    None
                }
            })
            .collect();
        let saved_tables: Vec<_> = saved
            .controls
            .iter()
            .filter_map(|control| {
                if let Control::Table(table) = control {
                    Some(table)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            original_tables.len(),
            saved_tables.len(),
            "{owner}: 표 소유 보존"
        );
        for (ordinal, (source, target)) in original_tables.iter().zip(saved_tables).enumerate() {
            let address = format!("{owner}/table{ordinal}");
            assert_eq!(
                (source.row_count, source.col_count),
                (target.row_count, target.col_count),
                "{address}"
            );
            assert_eq!(
                source.cells.len(),
                target.cells.len(),
                "{address}: 셀 누락·중복 금지"
            );
            for (cell, (left, right)) in source.cells.iter().zip(&target.cells).enumerate() {
                assert_eq!(
                    (left.row, left.col, left.row_span, left.col_span),
                    (right.row, right.col, right.row_span, right.col_span),
                    "{address}/cell{cell}: 셀 소속·병합 보존"
                );
                paragraphs(
                    &left.paragraphs,
                    &right.paragraphs,
                    &format!("{address}/cell{cell}"),
                );
            }
        }
    }
}

fn verify(extension: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let original = parse_document(&std::fs::read(path).expect("편람 원본")).expect("원본 파싱");
    let package = serialize_hwpx(&original).expect("실제 HWPX 저장");
    let saved = parse_document(&package).expect("저장 결과 다시 읽기");
    assert_eq!(
        original.sections.len(),
        saved.sections.len(),
        "구역 누락·중복 금지"
    );
    for (index, (source, target)) in original.sections.iter().zip(&saved.sections).enumerate() {
        assert_eq!(
            source.section_def.master_pages.len(),
            target.section_def.master_pages.len(),
            "구역{index}: 바탕쪽을 다른 구역으로 재배분하지 않는다"
        );
        for (ordinal, (left, right)) in source
            .section_def
            .master_pages
            .iter()
            .zip(&target.section_def.master_pages)
            .enumerate()
        {
            assert_eq!(
                left.apply_to, right.apply_to,
                "구역{index}/바탕쪽{ordinal}: 적용 쪽 소유"
            );
            paragraphs(
                &left.paragraphs,
                &right.paragraphs,
                &format!("section{index}/master{ordinal}"),
            );
        }
        paragraphs(
            &source.paragraphs,
            &target.paragraphs,
            &format!("section{index}"),
        );
    }
}

#[test]
fn hwp_source_owners_and_text_survive_real_hwpx_roundtrip() {
    verify("hwp");
}

#[test]
fn hwpx_source_owners_and_text_survive_real_hwpx_roundtrip() {
    verify("hwpx");
}
