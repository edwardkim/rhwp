//! 관계형: 저장 줄의 쪽/상자 좌표계가 같은 물리 글상자의 크기·글줄을 바꾸지 않는다.
//! 주석 갱신: 2026-11-06T23:54:43+09:00.
//! 2026-11-06 raw(96DPI, PDFium, 2px): local/page 대조군 각 전1쪽,
//! Native/fresh WASM 모두100.00000%; 각 문서 전쪽 최저도 각각100.00000%.
//! 근거: mydocs/pr/assets/issue7375/paper-textbox-minimum-20261106/README.md.
//! 독립 Print 좌표는 별도 고정형 검사, 이 파일은 좌표계와 원본 보존의 관계만 검사한다.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

fn find<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find(child, predicate))
}

fn geometry(key: &str) -> (BoundingBox, BoundingBox, i32) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(format!("pdf/issue7375/paper-textbox-vpos-{key}.hwpx"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let mut stored_vpos = None;
    for section in &core.document().sections {
        for paragraph in &section.paragraphs {
            for control in &paragraph.controls {
                if let Control::Footer(footer) = control {
                    for paragraph in &footer.paragraphs {
                        for control in &paragraph.controls {
                            if let Control::Shape(shape) = control {
                                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref())
                                {
                                    stored_vpos = Some(tb.paragraphs[0].line_segs[0].vertical_pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let before = core.export_hwpx_native().unwrap();
    let page = core.build_page_render_tree(0).unwrap();
    let outer = find(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Rectangle(_))
            && node
                .children
                .iter()
                .any(|child| matches!(child.node_type, RenderNodeType::TextBox))
    })
    .expect("같은 꼬리말 글상자");
    let line = find(outer, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
    .expect("같은 현재 BOX 글줄");
    assert_eq!(
        core.export_hwpx_native().unwrap(),
        before,
        "측정·paint가 저장 vpos 좌표계나 원본을 바꾸지 않는다"
    );
    (outer.bbox, line.bbox, stored_vpos.unwrap())
}

#[test]
fn source_page_origin_is_not_counted_as_box_content_height() {
    let local = geometry("local");
    let page = geometry("page");
    assert_ne!(local.2, page.2, "실제로 서로 다른 저장 좌표계의 반례다");
    for (name, a, b) in [("상자", local.0, page.0), ("글줄", local.1, page.1)] {
        for (a, b) in [
            (a.x, b.x),
            (a.y, b.y),
            (a.width, b.width),
            (a.height, b.height),
        ] {
            assert!(
                (a - b).abs() < 0.01,
                "{name}의 물리 프레임은 저장 vpos 좌표계가 달라도 같아야 한다: {a} / {b}"
            );
        }
    }
}
