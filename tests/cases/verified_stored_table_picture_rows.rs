//! 독립 한컴 2024 Print 18쪽의 표·그림·표 원시 줄 소유 계약.
//! Native/fresh WASM 전쪽 및 해당 쪽 확대 판독의 근거는 단계 1258에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::{model::control::Control, DocumentCore};

fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect(child, nodes);
    }
}
fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}
fn text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    collect(node, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.display_or_text()),
            _ => None,
        })
        .collect()
}

#[test]
fn print_verified_stored_table_picture_rows_preserve_content_and_source_ownership() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5.hwpx");
    let bytes = std::fs::read(path).expect("시각 검증한 원본");
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let title = "가. 주전산센터 목표시스템 구성(안)";
    let owners: Vec<_> = source.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            matches!(p.controls.first(),Some(Control::Table(t)) if t.cells.iter().any(|cell|
            cell.paragraphs.iter().any(|para|compact(&para.text)==compact(title))))
        })
        .collect();
    assert_eq!(owners.len(), 1, "원본 제목을 소유한 문단은 하나다");
    let (owner, paragraph) = owners[0];
    assert_eq!(paragraph.controls.len(), 3, "원본 표·그림·표의 세 컨트롤");
    assert!(matches!(&paragraph.controls[0],Control::Table(t) if t.common.treat_as_char));
    assert!(matches!(&paragraph.controls[1],Control::Picture(p) if p.common.treat_as_char));
    assert!(matches!(&paragraph.controls[2],Control::Table(t) if t.common.treat_as_char));
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(17).unwrap();
    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes);
    let table = |control| {
        let found:Vec<_>=nodes.iter().copied().filter(|node|matches!(&node.node_type,
            RenderNodeType::Table(t) if t.section_index==Some(0)&&t.para_index==Some(owner)&&t.control_index==Some(control))).collect();
        assert_eq!(
            found.len(),
            1,
            "같은 원본 표를 중복 배치하거나 누락하면 안 된다"
        );
        found[0]
    };
    let first = table(0);
    let last = table(2);
    let pictures:Vec<_>=nodes.iter().copied().filter(|node|matches!(&node.node_type,
        RenderNodeType::Image(p) if p.section_index==Some(0)&&p.para_index==Some(owner)&&p.control_index==Some(1))).collect();
    assert_eq!(pictures.len(), 1, "그림은 원본 둘째 줄에 한 번만 배치한다");
    let picture = pictures[0];
    assert_eq!(
        compact(&text(first)),
        compact(title),
        "글꼴별로 런이 나뉘어도 제목 전체를 보존한다"
    );
    assert_eq!(compact(&text(last)), compact("나. 주요 과업내용"));
    assert!(
        first.bbox.y + first.bbox.height <= picture.bbox.y,
        "첫 표 뒤에 그림 줄이 온다"
    );
    assert!(
        picture.bbox.y + picture.bbox.height <= last.bbox.y,
        "그림 아래에 마지막 표가 온다"
    );
    assert!(
        picture.bbox.x >= page.root.bbox.x
            && picture.bbox.x + picture.bbox.width <= page.root.bbox.x + page.root.bbox.width
    );
    // 그림 내부의 채색 차이가 남아 완료 페이지의 px 핀은 추가하지 않는다.
    // 제목·그림의 행 순서와 셀 소유를 보호하고 도형 paint 차이는 별도로 보정한다.
    for (table_node, control) in [(first, 0), (last, 2)] {
        let mut title_nodes = Vec::new();
        collect(table_node, &mut title_nodes);
        let mut end = 0;
        for node in title_nodes {
            let RenderNodeType::TextRun(run) = &node.node_type else {
                continue;
            };
            let context = run
                .cell_context
                .as_ref()
                .expect("제목은 원본 표 셀에 속한다");
            assert_eq!(context.parent_para_index, owner);
            assert_eq!(context.path[0].control_index, control);
            let start = run.char_start.expect("원문 문자 시작");
            assert!(start >= end, "글꼴별 분할 후에도 문자 순서를 유지한다");
            end = start + run.text.chars().count();
        }
    }
}
