//! 저장된 공백 호스트의 표·줄 소유를 같은 단 프레임에서 보존한다.
//! 독립 Print의 참석장과 위임장은 같은 쪽이며 마지막 안내문도 보존된다.
//! 실제 소유 문단과 상자 관계를 검사하며 화면 좌표로 대상을 고르지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

fn body_nodes<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    result.push(node);
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    for child in &node.children {
        body_nodes(child, result);
    }
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn stored_table_before_whitespace_host_uses_the_current_column_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/hancom-hwp/issue_157.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본 참석장"))
        .expect("저장 문서 열기");
    let paragraphs = &core.document().sections[0].paragraphs;
    let (owner, control) = paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, paragraph)| {
            paragraph.controls.iter().enumerate().find_map(|(ci, c)| {
                let Control::Table(table) = c else {
                    return None;
                };
                table
                    .cells
                    .iter()
                    .any(|cell| {
                        cell.paragraphs
                            .iter()
                            .any(|p| compact(&p.text).contains("실질주주참석자"))
                    })
                    .then_some((pi, ci))
            })
        })
        .expect("원본 참석자 표의 소유 문단");
    assert!(paragraphs[owner].text.trim().is_empty());
    assert_eq!(core.page_count(), 2, "독립 Print의 전체 쪽수");
    let tree = core.build_page_render_tree(1).expect("참석장·위임장 쪽");
    let mut nodes = Vec::new();
    body_nodes(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(owner) && t.control_index == Some(control))
        })
        .expect("참석자 표가 끝 안내문과 같은 쪽에 존재한다");
    let lines =
        |pi| {
            nodes.iter().filter(|n| {
        matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(pi))
    }).copied().collect::<Vec<_>>()
        };
    let previous = lines(owner - 1);
    let host = lines(owner);
    let next = lines(owner + 1);
    assert!(!previous.is_empty() && !next.is_empty());
    assert_eq!(host.len(), 1, "공백 호스트 줄은 한 번만 배치한다");
    for line in previous {
        assert!(
            line.bbox.y + line.bbox.height <= table.bbox.y + 1e-8,
            "표는 앞 본문 줄 뒤에 있어야 한다"
        );
    }
    assert!(
        table.bbox.y + table.bbox.height <= host[0].bbox.y + 1e-8,
        "저장된 공백 호스트 줄은 표 뒤에 있으며 표 본체와 겹치지 않는다"
    );
    for line in next {
        assert!(
            host[0].bbox.y + host[0].bbox.height <= line.bbox.y + 1e-8,
            "위임 제목은 공백 호스트 줄 뒤에 있어야 한다"
        );
    }
    let text = compact(&core.extract_page_text_native(1).expect("둘째 쪽 본문"));
    for expected in [
        "주주총회참석장",
        "위임장",
        "대리인",
        "위임인",
        "주주총회참석자는신분증을필히지참하시기바랍니다.",
    ] {
        assert!(
            text.contains(expected),
            "같은 쪽의 원본 내용 보존: {expected}"
        );
    }
}
