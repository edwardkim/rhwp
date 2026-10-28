//! #8153: 묶음 안의 잘린 그림은 전체 원본과 물리 Print clip을 보존한다.
//! 131·140쪽은 두 원본의 Native/fresh WASM raw 100% 및 독립 Print 확대 대조를 통과했다.
//! Print PDF의 점 단위 clip을 96dpi 좌상단 좌표로 변환해 ±2px로 검증한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{ClipKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::{BoundingBox, NodeId, RenderNode, RenderNodeType},
    DocumentCore,
};
fn source(node: &RenderNode, group: bool, bin: u16, out: &mut Vec<(NodeId, BoundingBox)>) {
    let group = group || matches!(node.node_type, RenderNodeType::Group(_));
    if let RenderNodeType::Image(i) = &node.node_type {
        if group && i.bin_data_id == bin {
            assert!(i.crop.is_some());
            out.push((node.id, node.bbox));
        }
    }
    for c in &node.children {
        source(c, group, bin, out)
    }
}
fn clipped(
    node: &LayerNode,
    id: NodeId,
    clips: &mut Vec<(ClipKind, Option<NodeId>, BoundingBox)>,
    out: &mut Vec<(BoundingBox, BoundingBox)>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                clipped(c, id, clips, out)
            }
        }
        LayerNodeKind::ClipRect {
            clip,
            child,
            clip_kind,
        } => {
            clips.push((*clip_kind, node.source_node_id, *clip));
            clipped(child, id, clips, out);
            clips.pop();
        }
        LayerNodeKind::ClipPath { child, .. } => clipped(child, id, clips, out),
        LayerNodeKind::Leaf { ops } if node.source_node_id == Some(id) => {
            for op in ops {
                if let PaintOp::Image {
                    bbox,
                    image,
                    resolved,
                } = op
                {
                    let own: Vec<_> = clips
                        .iter()
                        .filter(|(k, owner, _)| {
                            *k == ClipKind::PrintBitmapFill && *owner == Some(id)
                        })
                        .collect();
                    assert_eq!(own.len(), 1, "그림의 물리 창 소유");
                    assert!(image.crop.is_none(), "전체 그림 사용 후 물리 창 적용");
                    let p = resolved.as_ref().expect("전체 그림 출력 자원");
                    assert_eq!(p.mime, "image/jpeg");
                    out.push((own[0].2, *bbox));
                }
            }
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}
fn verify(ext: &str, page: u32, bin: u16, clip_pt: [f64; 4]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{ext}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(page - 1).unwrap();
    let mut matches = Vec::new();
    source(&tree.root, false, bin, &mut matches);
    assert_eq!(matches.len(), 1, "묶음 그림의 원본 소유");
    let layer = core
        .build_page_layer_tree_with_profile(page - 1, RenderProfile::Print)
        .unwrap();
    let mut painted = Vec::new();
    clipped(&layer.root, matches[0].0, &mut Vec::new(), &mut painted);
    assert_eq!(painted.len(), 1, "묶음 그림의 누락·중복 방지");
    let (clip, full) = painted[0];
    let [left, bottom, right, top] = clip_pt;
    let expected = [
        left * 4. / 3.,
        (754. - top) * 4. / 3.,
        (right - left) * 4. / 3.,
        (top - bottom) * 4. / 3.,
    ];
    for (actual, reference) in [clip.x, clip.y, clip.width, clip.height]
        .into_iter()
        .zip(expected)
    {
        assert!(
            (actual - reference).abs() <= 2.,
            "Print crop 경계: {actual} / {reference}"
        );
    }
    assert!(
        full.x <= clip.x + 0.01
            && full.y <= clip.y + 0.01
            && full.x + full.width >= clip.x + clip.width - 0.01
            && full.y + full.height >= clip.y + clip.height - 0.01,
        "전체 원본이 물리 창을 포함한다"
    );
}
#[test]
fn hwp_p131_grouped_picture_print_window() {
    verify("hwp", 131, 377, [73.600, 126.941, 470.012, 369.437]);
}
#[test]
fn hwp_p140_grouped_picture_print_window() {
    verify("hwp", 140, 385, [127.422, 191.735, 440.763, 276.566]);
}
#[test]
fn hwpx_p131_grouped_picture_print_window() {
    verify("hwpx", 131, 377, [73.600, 126.941, 470.012, 369.437]);
}
#[test]
fn hwpx_p140_grouped_picture_print_window() {
    verify("hwpx", 140, 385, [127.422, 191.735, 440.763, 276.566]);
}
