//! [#7062] TAC 도해를 담는 줄은 그림 높이를 점유하고 뒤 안내 표와 겹치지 않아야 한다.
//!
//! 동일 입력의 독립 한컴 PDF는10쪽이다. 현재 Native/fresh WASM 전쪽 비교의 최저값은
//! 95.09873%다. 과거 바이너리의 절대 좌표 대신 같은 셀의 줄·그림 포함관계와
//! 안내 표의 순서,1쪽 제목·책임자의 머리 표 소속을 검사한다. 실제 배치는 TSV/PNG로 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue7062/tac_object_host_line_height.hwp";
const AFTER_TEXT: &str = "즉시 추진 가능한 시급한 과제부터";

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("문서 읽기")).expect("문서 열기")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn page_nodes(core: &DocumentCore, page_index: u32) -> Vec<RenderNode> {
    let page = core.build_page_render_tree(page_index).expect("렌더 트리");
    let mut refs = Vec::new();
    walk(&page.root, &mut refs);
    refs.into_iter().cloned().collect()
}

fn contains_text(node: &RenderNode, text: &str) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(text))
        || node.children.iter().any(|child| contains_text(child, text))
}

fn diagram(nodes: &[RenderNode]) -> &RenderNode {
    nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .max_by(|a, b| (a.bbox.width * a.bbox.height).total_cmp(&(b.bbox.width * b.bbox.height)))
        .expect("2쪽의 주 도해")
}

fn smallest_cell<'a>(nodes: &'a [RenderNode], text: &str) -> &'a RenderNode {
    nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TableCell(_)) && contains_text(n, text))
        .min_by(|a, b| (a.bbox.width * a.bbox.height).total_cmp(&(b.bbox.width * b.bbox.height)))
        .expect("본문 소속 셀")
}

fn inside(outer: BoundingBox, inner: BoundingBox) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

#[test]
fn issue_7062_tac_diagram_host_line_carries_object_height() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = diagram(&nodes);
    let cell = nodes
        .iter()
        .find(|n| {
            matches!(n.node_type, RenderNodeType::TableCell(_))
                && n.children.iter().any(|c| {
                    matches!(c.node_type, RenderNodeType::Image(_))
                        && c.bbox.height == image.bbox.height
                })
        })
        .expect("도해 소속 셀");
    let host = cell
        .children
        .iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::TextLine(_))
                && n.bbox.y <= image.bbox.y
                && image.bbox.y <= n.bbox.y + n.bbox.height
        })
        .max_by(|a, b| a.bbox.height.total_cmp(&b.bbox.height))
        .expect("도해를 점유하는 줄");
    assert!(
        inside(host.bbox, image.bbox),
        "TAC 줄은 도해 전체를 점유해야 하며 작은 빈 줄로 붕괴하면 안 된다"
    );
}

#[test]
fn issue_7062_paragraph_after_tac_diagram_clears_the_object() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = diagram(&nodes);
    let after = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextRun(r) if r.text.contains(AFTER_TEXT))
        })
        .expect("그림 뒤 안내 문단");
    let table = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)) && contains_text(n, AFTER_TEXT))
        .min_by(|a, b| (a.bbox.width * a.bbox.height).total_cmp(&(b.bbox.width * b.bbox.height)))
        .expect("안내 문단 소속 표");
    assert!(
        table.bbox.y >= image.bbox.y + image.bbox.height,
        "그림 뒤 안내 표는 도해와 겹치면 안 된다"
    );
    assert!(
        inside(table.bbox, after.bbox),
        "안내 문단은 뒤 표 내부에 있어야 한다"
    );
}

#[test]
fn issue_7062_control_group_page1_and_page_count_unchanged() {
    let core = load();
    assert_eq!(
        core.page_count(),
        10,
        "독립 한컴 PDF와 같은10쪽을 유지해야 한다"
    );
    let nodes = page_nodes(&core, 0);
    let title = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextRun(r) if r.text.contains("보 도 자 료"))
        })
        .expect("1쪽 제목");
    let contact = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextRun(r) if r.text.contains("금융제도팀장"))
        })
        .expect("1쪽 책임자");
    assert!(
        inside(smallest_cell(&nodes, "보 도 자 료").bbox, title.bbox),
        "제목은 머리 표의 소속 셀 안에 있어야 한다"
    );
    assert!(
        inside(smallest_cell(&nodes, "금융제도팀장").bbox, contact.bbox),
        "책임자는 머리 표의 소속 셀 안에 있어야 한다"
    );
    assert!(
        title.bbox.y + title.bbox.height <= contact.bbox.y,
        "머리 표에서 제목 뒤에 책임자가 놓여야 한다"
    );
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n.node_type, RenderNodeType::Table(_))
                && contains_text(n, "보 도 자 료")
                && contains_text(n, "금융제도팀장")),
        "제목과 책임자는 같은1쪽 머리 표에 소속되어야 한다"
    );
}
