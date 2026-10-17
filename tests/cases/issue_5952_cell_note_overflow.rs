//! [#5952] 편람 유의사항의 내용과 상자·사이드바·다음 제목의 관계를 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwp";

fn page() -> rhwp::renderer::render_tree::PageRenderTree {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("편람 원본")).expect("편람 파싱");
    core.set_hangul2024_compat(true);
    core.build_page_render_tree(68).expect("독립 Print 69쪽")
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

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![root];
    for child in &root.children {
        result.extend(nodes(child));
    }
    result
}

fn note(root: &RenderNode) -> &RenderNode {
    let found: Vec<_> = nodes(root)
        .into_iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::Table(_))
                && text(n).contains("기안문 제목과 붙임물의 제목")
        })
        .collect();
    assert_eq!(found.len(), 1, "유의사항 상자의 누락·중복");
    found[0]
}

#[test]
fn note_box_lines_do_not_overlap_the_gongmunseo_sidebar() {
    let page = page();
    let note = note(&page.root);
    let sidebar: Vec<_> = nodes(&page.root)
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextBox) && compact(&text(n)) == "공문서")
        .collect();
    assert_eq!(sidebar.len(), 1, "같은 쪽의 공문서 사이드바");
    let sidebar = sidebar[0];
    let runs: Vec<_> = nodes(note)
        .into_iter()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TextRun(r) if !r.text.trim().is_empty()))
        .collect();
    assert!(!runs.is_empty(), "상자 본문이 있어야 한다");
    for run in runs {
        let right = run.bbox.x + run.bbox.width;
        assert!(
            run.bbox.x >= note.bbox.x && right <= note.bbox.x + note.bbox.width,
            "상자 밖 글자: {}",
            text(run)
        );
        assert!(
            right <= sidebar.bbox.x,
            "사이드바와 겹치는 글자: {}",
            text(run)
        );
    }
    // 독립 한컴2024 KoPub Print 69쪽의 상자 전체 내용. PDF 추출 공백만 정규화한다.
    let expected = concat!(
        "※ 기안문 제목과 붙임물의 제목이 다른 경우가 많아 기안문과 붙임물 간의 연계·유지를 위한 편철 관리 필요",
        "☞ 비전자 형태로 생산되는 대장류 기록물은 반드시 전자기록생산시스템(업무관리시스템 또는 전자문서시스템)에 등록하여야 함.",
        "☞ 전자우편으로 수신한 문서는 출력 후 전자기록생산시스템(업무관리시스템 또는 전자문서시스템)에 비전자기록물 등록기능을 이용하여 반드시 등록하여야 함",
        "※ 전자메일‧팩스‧우편 등으로 수신한 비전자기록물이 미등록되지 않도록 가급적이면 전자기록생산시스템에 탑재된 ‘문서 24’를 통해 전자기록물 형태로 수신을 권장함",
        "☞ 붙임파일에 비밀번호를 설정할 경우, 해당 비밀번호를 지속적으로 관리할 수 없어 붙임파일에 대한 내용 확인 불가",
        "※ 국가안보 및 개인정보 등이 포함된 기록물(붙임파일 포함)은 「보안업무규정」 「개인정보 보호법」 등 관련 규정에 따라 해당 정보가 유출되지 않도록 기록물 생산 시 문서 보안 또는 열람 범위 지정(열람불가, 부서, 실국, 기관) 기능을 활용하여 열람을 제한하는 방법 활용"
    );
    assert_eq!(
        compact(&text(note)),
        compact(expected),
        "상자 전체 내용·순서 보존"
    );
}

#[test]
fn note_box_bottom_stays_separate_from_following_body_heading() {
    let page = page();
    let note = note(&page.root);
    let heading: Vec<_> = nodes(&page.root)
        .into_iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::TextLine(_))
                && compact(&text(n)).starts_with("4)문서의“끝”표시")
        })
        .collect();
    assert_eq!(heading.len(), 1, "상자 다음 본문 제목의 소속·누락·중복");
    let heading = heading[0];
    assert!(
        note.bbox.y + note.bbox.height <= heading.bbox.y,
        "유의사항 상자가 다음 본문 제목을 침범한다"
    );
    for line in nodes(note)
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
    {
        assert!(
            line.bbox.y >= note.bbox.y
                && line.bbox.y + line.bbox.height <= note.bbox.y + note.bbox.height,
            "유의사항 줄은 상자 안에 있어야 한다: {}",
            text(line)
        );
    }
}

/// [#5952] 저장 2줄이 composed 1줄로 접혀 셀 내폭을 넘치면 fresh 재래핑한다.
/// source-side `#[cfg(test)]` 가 아니라 integration 으로 둔다 (RustUnitTier).
#[test]
fn collapsed_stored_two_lines_rewrap_when_over_cell() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    use rhwp::renderer::composer::{
        compose_paragraph, recompose_stored_single_line_if_overflowing,
    };
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let styles = ResolvedStyleSet::default();
    let text = format!("※{}", "가".repeat(39));
    let n = text.chars().count();
    let collapsed = Paragraph {
        text: text.clone(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    };
    let stored_two = Paragraph {
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
            LineSeg {
                text_start: 20,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
        ],
        ..collapsed.clone()
    };
    let mut composed = compose_paragraph(&collapsed);
    assert_eq!(composed.lines.len(), 1, "1-ls 문단은 1줄로 시작한다");
    recompose_stored_single_line_if_overflowing(&mut composed, &stored_two, 40.0, &styles, 96.0);
    assert!(
        composed.lines.len() > 1,
        "유의사항 bullet 저장 2줄이 셀을 넘치면 재래핑돼야 함 (#5952)"
    );
}

/// 끝의 빈 LINE_SEG는 저장 행이 접힌 결함이 아니므로 재래핑하지 않는다.
#[test]
fn trailing_empty_stored_line_seg_is_preserved() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
    use rhwp::renderer::composer::{
        compose_paragraph, recompose_stored_single_line_if_overflowing,
    };
    use rhwp::renderer::style_resolver::ResolvedStyleSet;

    let styles = ResolvedStyleSet::default();
    let text = "가".repeat(40);
    let n = text.chars().count();
    let stored_two = Paragraph {
        text: text.clone(),
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
            LineSeg {
                text_start: n as u32,
                line_height: 800,
                baseline_distance: 640,
                segment_width: 3000,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut composed = compose_paragraph(&stored_two);
    assert_eq!(
        composed.lines.len(),
        2,
        "빈 둘째 저장 행도 composed에 남는다"
    );
    recompose_stored_single_line_if_overflowing(&mut composed, &stored_two, 40.0, &styles, 96.0);
    assert!(
        composed.lines[1].runs.is_empty(),
        "끝의 빈 저장 행은 fresh 재래핑으로 채우지 않아야 함 (#5952)"
    );
}
