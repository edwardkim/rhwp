//! [Issue #6078] HWP3 표의 캡션·본문·후행 용지 규격 문단은 한 쪽에서
//! 원본 순서대로 보여야 한다. 표가 속한 저장 글줄과 캡션의 원본 점유 높이를
//! 함께 소비해야 후행 문단이 용지 밖으로 밀리거나 캡션과 표가 겹치지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwp3-table-caption.hwp";

#[test]
fn issue_6078_caption_table_and_paper_spec_keep_reading_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 1, "원본 한컴 PDF와 같은 한 쪽");

    let Control::Table(source_table) = &core.document().sections[0].paragraphs[0].controls[2]
    else {
        panic!("HWP3 원본의 캡션 표");
    };
    let caption = source_table.caption.as_ref().expect("표 위 캡션");
    assert!(caption.spacing > 0, "원본 캡션의 물리 점유가 보존돼야 함");
    let recommendation_checks: usize = source_table
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .filter(|para| para.text.contains("추천기관") || para.text.contains("한국자금"))
        .map(|para| para.text.chars().filter(|ch| *ch == '□').count())
        .sum();
    assert_eq!(recommendation_checks, 3, "세 추천기관의 빈 체크박스");

    let page = core.build_page_render_tree(0).expect("첫 쪽 렌더 트리");
    let body = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역");
    let table = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(0) && t.control_index == Some(2))
    })
    .expect("캡션을 가진 표");
    let caption_run = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains("별지"))
    })
    .expect("표 위 캡션 글줄");
    let paper_spec = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains("신문용지"))
    })
    .expect("표 뒤 용지 규격 문단");

    assert!(
        table.bbox.y > caption_run.bbox.y + caption_run.bbox.height,
        "캡션 글줄과 표 테두리가 겹치지 않아야 함"
    );
    assert!(
        table.bbox.x + table.bbox.width / 2.0 > body.bbox.x + body.bbox.width / 2.0,
        "두 번째 저장 글줄의 내어쓰기를 반영해 표를 가운데 정렬해야 함"
    );
    assert!(
        paper_spec.bbox.y >= table.bbox.y + table.bbox.height,
        "용지 규격 문단은 표 뒤에 있어야 함"
    );
    assert!(
        paper_spec.bbox.y + paper_spec.bbox.height <= page.root.bbox.y + page.root.bbox.height,
        "용지 규격 문단은 같은 쪽 안에 보여야 함"
    );
}

fn find_node<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, predicate))
}
