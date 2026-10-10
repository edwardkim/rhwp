//! #6034 법원 서식의 본문·각주·빈 꼬리쪽 소유 계약.
//! 사용자 수동 Print 정본은 `pdf/2912735_court_report_form-2020.pdf` 2쪽이다.
//! 같은 원본의 전쪽 Native/fresh WASM 최저 91.16037%와 직접 판독 후,
//! 종전 본문 y 범위·각주 줄 수 핀을 내용 보존·순서·비겹침 관계로 교체했다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6034/2912735_court_report_form.hwp";

fn find_footnote_area<'a>(node: &'a RenderNode) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::FootnoteArea) {
        return Some(node);
    }
    node.children.iter().find_map(find_footnote_area)
}

fn body_text_bottom(node: &RenderNode, in_footnote: bool) -> f64 {
    let in_footnote = in_footnote || matches!(&node.node_type, RenderNodeType::FootnoteArea);
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if !in_footnote && !run.text.trim().is_empty() => {
            node.bbox.y + node.bbox.height
        }
        _ => f64::MIN,
    };
    node.children
        .iter()
        .map(|child| body_text_bottom(child, in_footnote))
        .fold(own, f64::max)
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn lines<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        result.push(node);
    }
    for child in &node.children {
        lines(child, result);
    }
}

fn source_notes<'a>(
    paragraphs: &'a [rhwp::model::paragraph::Paragraph],
    result: &mut Vec<&'a rhwp::model::footnote::Footnote>,
) {
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            match control {
                rhwp::model::control::Control::Footnote(note) => result.push(note),
                rhwp::model::control::Control::Table(table) => {
                    for cell in &table.cells {
                        source_notes(&cell.paragraphs, result);
                    }
                }
                _ => {}
            }
        }
    }
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn issue_6034_footnote_area_starts_below_body_text() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("원본 읽기 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("법원 서식 로드");
    // 사용자 직접 Print PDF는 본문 한 쪽과 빈 꼬리쪽이다. 내용 소유도 함께 확인한다.
    assert_eq!(core.page_count(), 2, "독립 Print PDF의 빈 꼬리쪽도 보존");
    let p1 = core.build_page_render_tree(0).expect("첫 쪽 실제 배치");
    let p2 = core.build_page_render_tree(1).expect("빈 꼬리쪽 실제 배치");
    assert!(
        compact(&text(&p2.root)).is_empty(),
        "첨부서면이나 각주가 꼬리쪽으로 밀리지 않음"
    );
    assert!(
        find_footnote_area(&p2.root).is_none(),
        "각주 네 개는 첫 쪽 소유"
    );
    assert!(
        core.dump_page_items(Some(1)).contains("pi=8"),
        "원본 마지막 빈 문단은 꼬리쪽 소유"
    );

    let area = find_footnote_area(&p1.root).expect("첫 쪽 각주 영역");
    let separator = area
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Line(_)))
        .expect("각주 구분선");
    let body_bottom = body_text_bottom(&p1.root, false);
    assert!(
        separator.bbox.y >= body_bottom,
        "실제 구분선은 마지막 본문 글줄 뒤에 위치"
    );

    let mut body_lines = Vec::new();
    lines(&p1.root, &mut body_lines);
    let mut previous = None;
    for paragraph in 5..=7 {
        let owned: Vec<_> = body_lines
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.section_index == Some(0) && line.para_index == Some(paragraph))
            })
            .collect();
        assert_eq!(owned.len(), 1, "첨부서면 각 항목은 독립 PDF처럼 한 줄 소유");
        assert_eq!(
            compact(&text(owned[0])),
            compact(&core.document().sections[0].paragraphs[paragraph].text),
            "첨부서면 누락·중복 없음"
        );
        if let Some(bottom) = previous {
            assert!(owned[0].bbox.y >= bottom, "첨부서면 항목 순서와 비겹침");
        }
        previous = Some(owned[0].bbox.y + owned[0].bbox.height);
    }

    let mut originals = Vec::new();
    source_notes(&core.document().sections[0].paragraphs, &mut originals);
    assert_eq!(originals.len(), 4, "독립 PDF와 같은 네 각주");
    let mut note_lines = Vec::new();
    lines(area, &mut note_lines);
    let first = note_lines.first().expect("각주 본문");
    assert!(
        first.bbox.y >= separator.bbox.y + separator.bbox.height,
        "각주 본문은 구분선 뒤에 위치"
    );
    for pair in note_lines.windows(2) {
        assert!(
            pair[1].bbox.y >= pair[0].bbox.y + pair[0].bbox.height,
            "각주 글줄·빈 문단은 서로 겹치지 않음"
        );
    }
    for (index, original) in originals.iter().enumerate() {
        let owned: Vec<_> = note_lines.iter().copied().filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line) if line.section_index == Some(index))
        }).collect();
        let actual = owned.iter().map(|node| text(node)).collect::<String>();
        let expected = original
            .paragraphs
            .iter()
            .map(|para| para.text.as_str())
            .collect::<String>();
        assert_eq!(
            compact(&actual),
            compact(&expected),
            "각주 {index}의 원본 본문을 한 번씩 보존"
        );
        if let Some(empty) = original
            .paragraphs
            .last()
            .filter(|para| para.text.is_empty())
        {
            let blank = owned.last().expect("각주 끝 빈 문단");
            assert!(compact(&text(blank)).is_empty(), "각주 끝 빈 줄의 소유");
            let shape =
                &core.document().doc_info.char_shapes[empty.char_shapes[0].char_shape_id as usize];
            let source_height =
                rhwp::renderer::hwpunit_to_px(shape.base_size, rhwp::renderer::DEFAULT_DPI);
            let roundoff = source_height * 1e-9;
            assert!(
                blank.bbox.height + roundoff >= source_height,
                "빈 각주 줄은 원본 글자 상자를 수용"
            );
        }
    }
}
