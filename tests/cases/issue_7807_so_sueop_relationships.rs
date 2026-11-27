//! 관계형·고정형 — #7807 관계도·후속 본문 소속과 46쪽 미주의 순서·보존.
//! 주석 갱신 2026-11-28T07:25:26+09:00. 직전 파일 커밋 9eb669d2052f45362122549813188a35d8029b24 2026-11-28T04:50:18+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 48ce72ba80f6ca43a2914ec60093348b064df4d2.
//! so-sueop-hwp3 · 기준 pdf/SO-SUEOP-hwp-2020.pdf · 전체 46쪽.
//! 전쪽 최저 Native 98.88% / fresh WASM 98.49%.
//! 22쪽 Native 99.93% / fresh WASM 99.93%.
//! 43쪽 Native 99.82% / fresh WASM 99.92%.
//! 44쪽 Native 99.53% / fresh WASM 99.57%.
//! 45쪽 Native 99.48% / fresh WASM 99.49%.
//! 46쪽 Native 99.73% / fresh WASM 99.74%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/so-sueop-hwp3/에 있다.
//! so-sueop-hwpx · 기준 pdf/SO-SUEOP-hwpx-2024.pdf · 전체 46쪽.
//! 전쪽 최저 Native 99.53% / fresh WASM 98.62%.
//! 43쪽 Native 99.99% / fresh WASM 99.95%.
//! 44쪽 Native 99.53% / fresh WASM 99.42%.
//! 45쪽 Native 99.84% / fresh WASM 99.79%.
//! 46쪽 Native 99.98% / fresh WASM 99.98%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/so-sueop-hwpx/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn walk(node: &RenderNode, visit: &mut impl FnMut(&RenderNode)) {
    visit(node);
    for child in &node.children {
        walk(child, visit);
    }
}

fn sample(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).expect("회귀 원본")).expect("회귀 원본 열기")
}

fn page_text(node: &RenderNode) -> String {
    let mut text = String::new();
    walk(node, &mut |node| {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            text.extend(
                run.display_or_text()
                    .chars()
                    .filter(|ch| !ch.is_whitespace()),
            );
        }
    });
    text
}

#[test]
fn hwp3_relationship_diagram_stays_inside_its_table_before_the_body() {
    let core = sample("samples/SO-SUEOP.hwp");
    let [section, ..] = core.document().sections.as_slice() else {
        panic!("본문 구역");
    };
    let (para_index, paragraph) = section
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, para)| {
            para.text.contains("윤두꺼비 시절 부친 말대가리") && para.text.contains("가문의 영예(")
        })
        .expect("관계도와 각주를 소유한 원본 문단");
    let [Control::Table(source_table), Control::Endnote(note), ..] = paragraph.controls.as_slice()
    else {
        panic!("먼저 배치되는 관계도와 미주");
    };
    assert_eq!(note.number, 118);
    let (before, after) = paragraph
        .text
        .split_once("가문의 영예(")
        .expect("미주 괄호");
    let label = after.split_once(')').expect("닫는 괄호").0;
    let left = before.chars().count() + "가문의 영예(".chars().count();
    let right = left + label.chars().count();
    let anchor = paragraph.control_text_positions()[1];
    assert!(
        (left..=right).contains(&anchor),
        "미주는 괄호 안에 있어야 한다"
    );

    let follow_index = section
        .paragraphs
        .iter()
        .enumerate()
        .skip(para_index + 1)
        .find(|(_, para)| para.text.contains("그와 유사한 인물은?"))
        .expect("관계도 뒤의 질문 문단")
        .0;
    let page = core.build_page_render_tree(21).expect("관계도의 쪽");
    let mut relationship = Vec::new();
    let mut body = Vec::new();
    walk(&page.root, &mut |node| match &node.node_type {
        RenderNodeType::Table(table)
            if table.para_index == Some(para_index) && table.control_index == Some(0) =>
        {
            relationship.push((node.bbox, page_text(node)));
        }
        RenderNodeType::TextRun(run)
            if (run.para_index == Some(para_index) && run.text.contains("윤두꺼비 시절"))
                || (run.para_index == Some(follow_index)
                    && run.text.contains("그와 유사한 인물은?")) =>
        {
            body.push(node.bbox);
        }
        _ => {}
    });
    let [(table, names)] = relationship.as_slice() else {
        panic!("원본 관계도를 해당 쪽에 한 번 그린다");
    };
    let mut remainder = names.as_str();
    for label in ["윤직원", "윤창식", "윤종수", "윤경손", "윤종학"] {
        remainder = remainder.split_once(label).expect("관계도 이름과 순서").1;
    }
    let diagram = &source_table.cells[0].paragraphs;
    assert!(diagram[0].text.contains('\u{F081A}') && diagram[1].text.contains('\u{F0817}'));
    let [detail, follow] = body.as_slice() else {
        panic!("첫 설명과 후속 질문이 관계도 쪽에 각각 한 번 있다");
    };
    assert!(detail.y >= table.y + table.height && follow.y >= detail.y + detail.height);
}

#[test]
fn so_sueop_endnotes_keep_order_across_the_last_four_pages() {
    for path in ["samples/SO-SUEOP.hwp", "samples/SO-SUEOP.hwpx"] {
        let core = sample(path);
        assert_eq!(core.page_count(), 46, "{path}");
        let texts: Vec<_> = (42..46)
            .map(|page| {
                let rendered = core.build_page_render_tree(page).expect("미주가 있는 쪽");
                page_text(&rendered.root)
            })
            .collect();
        for (page, label) in [
            (0, "25)해답"),
            (2, "130)작가와독자가"),
            (2, "161)현실과타협할줄"),
            (3, "192)전재의비극성"),
        ] {
            assert_eq!(texts[page].matches(label).count(), 1, "{path}, {label}");
            assert_eq!(
                texts
                    .iter()
                    .map(|s| s.matches(label).count())
                    .sum::<usize>(),
                1,
                "{path}, {label}"
            );
        }
    }
}
