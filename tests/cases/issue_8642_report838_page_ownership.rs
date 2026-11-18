//! 관계형: report-838의 본문·각주 참조 소유, 마지막 참고문헌과 전체 쪽수를 함께 보호한다.
//! 주석 갱신: 2026-11-09T20:16:00+09:00.
//! 독립 한컴 2020 Print는 109쪽이다. 고정 devel b768bee1은 110쪽, 후보 eb876d14는 109쪽이다.
//! 후보 전109쪽 raw 최저 Native98.70% / fresh WASM98.69% (96DPI, Print compat2022, 2px).
//! 근거: planet6897_20261106/pr8642/report-838-original.hwp 및
//! pdf/planet6897-20261106/pr8642/report-838-original-2020.pdf.
//! 정식 증적: mydocs/pr/assets/planet6897_20261106/report-838-pagination-final/validation.json
//! 및 native-silhouette.tsv / wasm-silhouette.tsv, 마지막 쪽 review·overlay.
//! 쪽수만 줄이거나 본문·각주를 누락해 통과하지 못하게 하며 실물 절대 px는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::BTreeSet;

fn collect(
    node: &RenderNode,
    in_note: bool,
    body: &mut String,
    notes: &mut String,
    references: &mut BTreeSet<u16>,
) {
    let in_note = in_note || matches!(node.node_type, RenderNodeType::FootnoteArea);
    match &node.node_type {
        RenderNodeType::TextRun(run) => {
            let target = if in_note { &mut *notes } else { &mut *body };
            target.extend(run.text.chars().filter(|c| !c.is_whitespace()));
        }
        RenderNodeType::FootnoteMarker(marker) => {
            assert!(
                references.insert(marker.number),
                "같은 원문 각주 참조를 두 번 배치하지 않는다"
            );
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, in_note, body, notes, references);
    }
}

#[test]
fn report838_preserves_all_note_references_and_the_print_page_owners() {
    let source = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("mydocs/pr/assets/planet6897_20261106/pr8642/report-838-original.hwp"),
    )
    .unwrap();
    let core = DocumentCore::from_bytes(&source).unwrap();
    assert_eq!(core.page_count(), 109, "독립 Print와 같은 전체 쪽수");
    fn source_notes(paragraphs: &[rhwp::model::paragraph::Paragraph], out: &mut BTreeSet<u16>) {
        for paragraph in paragraphs {
            for control in &paragraph.controls {
                match control {
                    Control::Footnote(note) => {
                        assert!(out.insert(note.number), "원문 각주 번호의 유일성");
                        source_notes(&note.paragraphs, out);
                    }
                    Control::Table(table) => {
                        for cell in &table.cells {
                            source_notes(&cell.paragraphs, out);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut expected = BTreeSet::new();
    for section in &core.document().sections {
        source_notes(&section.paragraphs, &mut expected);
    }
    assert_eq!(expected.len(), 295, "원본과 Print의 전체 각주 참조");
    let mut references = BTreeSet::new();
    let mut pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let (mut body, mut notes) = (String::new(), String::new());
        collect(&tree.root, false, &mut body, &mut notes, &mut references);
        pages.push((body, notes));
    }
    assert_eq!(
        references, expected,
        "원문 각주 참조의 누락·중복·소유 변경 없음"
    );
    for (page, expected_body, expected_note) in [
        (90, "특정침해소송", "프랑스PAㆍTA의자격및직무"),
        (
            91,
            "사무변호사와의공동대리는필요하지않으나",
            "특허법제143조제2항",
        ),
    ] {
        assert!(
            pages[page - 1].0.contains(expected_body),
            "Print {page}쪽 본문 소속"
        );
        assert!(
            pages[page - 1].1.contains(expected_note),
            "Print {page}쪽 각주 소속"
        );
        assert_eq!(
            pages
                .iter()
                .filter(|(_, notes)| notes.contains(expected_note))
                .count(),
            1,
            "각주를 인접 쪽에 중복 배치하지 않는다"
        );
    }
    let last = "특허심판원개원5주년기념세미나자료집";
    assert!(
        pages[108].0.contains(last),
        "마지막 참고문헌은 Print 109쪽에 보존한다"
    );
    assert_eq!(
        pages.iter().filter(|(body, _)| body.contains(last)).count(),
        1
    );
    assert!(
        core.build_page_render_tree(109).is_err(),
        "빈 110쪽을 남기지 않는다"
    );
}
