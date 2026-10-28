//! #8153: 글자처럼 취급되는 그림 뒤의 공백은 가운데 정렬을 이동시키지 않는다.
//! HWP 5.0 표 44·70의 정렬/흐름과 93쪽 한컴 Print를 대조한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};
fn find_source(node: &RenderNode, out: &mut Vec<(BoundingBox, BoundingBox, BoundingBox)>) {
    if matches!(&node.node_type,RenderNodeType::TextLine(line) if line.para_index==Some(654) && line.section_index==Some(2))
    {
        let pictures: Vec<_> = node
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::Image(im) if im.bin_data_id == 371 => Some(n.bbox),
                _ => None,
            })
            .collect();
        let spaces: Vec<_> = node
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TextRun(run) if run.text == "     " => Some(n.bbox),
                _ => None,
            })
            .collect();
        if pictures.len() == 1 && spaces.len() == 1 {
            out.push((node.bbox, pictures[0], spaces[0]));
        }
    }
    for child in &node.children {
        find_source(child, out)
    }
}
fn find_paint(node: &LayerNode, out: &mut Vec<BoundingBox>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                find_paint(child, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            find_paint(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image { bbox, image, .. } = op {
                    if image.bin_data_id == 371 && image.para_index == Some(654) {
                        out.push(*bbox)
                    }
                }
            }
        }
    }
}
fn verify(ext: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join(format!("samples/2025 행정업무운영 편람(최종).{ext}"))).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let p = &core.document().sections[2].paragraphs[654];
    assert_eq!(p.text, "     ");
    assert_eq!(p.control_text_positions(), vec![0]);
    let render = core.build_page_render_tree(92).unwrap();
    let mut source = Vec::new();
    find_source(&render.root, &mut source);
    assert_eq!(source.len(), 1, "그림과 공백의 줄 소유 및 누락 방지");
    let (line, picture, space) = source[0];
    let expected_center = line.x + line.width / 2.;
    let picture_center = picture.x + picture.width / 2.;
    assert!(
        (picture_center - expected_center).abs() <= 1.0 / 75.0,
        "원본 줄의 중앙과 글자처럼 취급되는 그림의 중앙: {picture_center} / {expected_center}"
    );
    assert!(
        (space.x - (picture.x + picture.width)).abs() <= 2.,
        "후행 공백은 그림 뒤에 남는다"
    );
    let layer = core
        .build_page_layer_tree_with_profile(92, RenderProfile::Print)
        .unwrap();
    let mut painted = Vec::new();
    find_paint(&layer.root, &mut painted);
    assert_eq!(painted.len(), 1);
    assert!(
        (painted[0].x - 103.80799865722656 * 4. / 3.).abs() <= 2.,
        "독립 Print의 그림 시작 영역"
    );
}
#[test]
fn hwp_centered_inline_picture_ignores_trailing_space_width() {
    verify("hwp")
}
#[test]
fn hwpx_centered_inline_picture_ignores_trailing_space_width() {
    verify("hwpx")
}
