//! #6706: 같은 가시 문자 위치에 투영된 표와 그림도 원시 UTF-16 저장 줄은 다르다.
//! 실물 HWPX와 기존 Hancom PDF `pdf/hwp3-sample16-hwp5-2022.pdf` 18쪽 대조.
//! PDF 글자 baseline과 그림 좌상단을 구분한다. 같은 줄의 그림+표 반례는 #6754가 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect(child, nodes);
    }
}

#[test]
fn stored_table_picture_table_rows_keep_hancom_positions_and_unique_ownership() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 64);
    let page = core.build_page_render_tree(17).unwrap();
    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes);
    let titles: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextRun(t)
            if t.text == "가. 주전산센터 목표시스템 구성(안)")
        })
        .collect();
    assert_eq!(
        titles.len(),
        1,
        "표 글자가 빠지거나 두 줄에 중복 배치되면 안 된다"
    );
    let title = titles[0];
    let RenderNodeType::TextRun(run) = &title.node_type else {
        unreachable!()
    };
    assert!(
        (title.bbox.x - 107.0).abs() < 0.7,
        "제목 x={}",
        title.bbox.x
    );
    assert!(
        (title.bbox.y + run.baseline - 318.89).abs() < 0.7,
        "제목 baseline={}",
        title.bbox.y + run.baseline
    );

    let pictures: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Image(i)
            if i.para_index == Some(394) && i.control_index == Some(1))
        })
        .collect();
    assert_eq!(
        pictures.len(),
        1,
        "그림은 원시 둘째 줄에 한 번만 배치해야 한다"
    );
    let picture = pictures[0];
    assert!(
        (picture.bbox.x - 107.9).abs() < 0.7,
        "그림 x={}",
        picture.bbox.x
    );
    assert!(
        (picture.bbox.y - 339.8).abs() < 0.7,
        "그림 y={}",
        picture.bbox.y
    );
    assert!(picture.bbox.x + picture.bbox.width < page.root.bbox.width);
    assert!(
        nodes
            .iter()
            .any(|n| matches!(&n.node_type, RenderNodeType::TextRun(t)
        if t.text.contains("나. 주요 과업내용"))
                && n.bbox.y > picture.bbox.y + picture.bbox.height),
        "셋째 줄 제목도 그림 아래에 보존되어야 한다"
    );
}
