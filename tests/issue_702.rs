//! Issue #702: 배분 다단과 쪽·단 나누기 뒤의 내용 소유를 검증한다.
//!
//! 독립 한컴 PDF와 Native/fresh WASM 전 7쪽 최저 96.13%로 검증한 문서다.
//! 7쪽 소유와 첫 쪽 지우기 항목의 양쪽 단 분배를 검사한다.
//! 제목·본문의 실제 위치는 Visual Sweep 증적에서 판정하고 픽셀 기대값으로 고정하지 않는다.
//! 증적: mydocs/pr/archives/pr_7382_review.md 보정293~294.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn shortcut_core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/shortcut.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("단축키 문서 읽기")
}

fn visible_text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run
            .display_or_text()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
    }
    node.children.iter().map(visible_text).collect()
}

#[test]
fn shortcut_distribute_short_column_split() {
    let core = shortcut_core();
    assert_eq!(core.page_count(), 7, "독립 한컴 PDF와 같은 7쪽 소유");
    let page_sections: &[&[&str]] = &[
        &["커서이동", "지우기"],
        &["파일", "미리보기상태에서", "편집"],
        &["보기", "입력", "문자표"],
        &["글상자에서", "상용구에서", "글자속성"],
        &["문단속성", "개요번호", "머리말/꼬리말"],
        &["도구", "매크로에서", "F5셀블록상태에서"],
        &["줄/칸삽입", "셀합치기", "도움말"],
    ];
    for (page, expected) in page_sections.iter().enumerate() {
        let tree = core.build_page_render_tree(page as u32).expect("쪽 트리");
        let text = visible_text(&tree.root);
        for section in *expected {
            assert!(
                text.contains(section),
                "{}쪽에 {section} 내용이 있어야 한다",
                page + 1
            );
        }
    }
    let first = core.build_page_render_tree(0).expect("첫 쪽 트리");
    let body = first
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문 영역");
    let columns: Vec<_> = body
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .map(visible_text)
        .collect();
    for expected in [
        ["뒤글자지우기", "앞글자지우기", "한단어지우기"],
        ["앞단어지우기", "한줄지우기", "줄뒤지우기"],
    ] {
        assert_eq!(
            columns
                .iter()
                .filter(|column| expected.iter().all(|text| column.contains(text)))
                .count(),
            1,
            "지우기 항목 세 개는 같은 단 한 곳에서만 소유해야 한다: {expected:?}"
        );
    }
}

#[test]
fn shortcut_page2_has_three_sections() {
    let core = shortcut_core();
    let tree = core.build_page_render_tree(1).expect("둘째 쪽 트리");
    let text = visible_text(&tree.root);
    for section in ["파일", "미리보기상태에서", "편집"] {
        assert!(
            text.contains(section),
            "둘째 쪽에서 {section} 구역이 다음 쪽으로 밀리면 안 된다"
        );
    }
}
