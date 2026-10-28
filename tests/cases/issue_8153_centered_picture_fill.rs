//! 관계형·고정형: 가운데 그림 채움의 셀 영역과 독립 Print 프레임을 보존한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 123쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 123쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본 123쪽은 변경 전 Native·fresh WASM raw 98% 이상이며,
//! 독립 한컴 2024 Print PDF의 이미지와 클립을 확대 대조했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::style::ImageFillMode,
    paint::{ClipKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::{BoundingBox, NodeId, RenderNode, RenderNodeType},
    DocumentCore,
};

fn source(node: &RenderNode, matches: &mut Vec<NodeId>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.bin_data_id == 293 {
            assert_eq!(image.fill_mode, Some(ImageFillMode::Center));
            matches.push(node.id);
        }
    }
    for child in &node.children {
        source(child, matches);
    }
}

fn painted(
    node: &LayerNode,
    id: NodeId,
    clip: Option<BoundingBox>,
    matches: &mut Vec<(BoundingBox, BoundingBox)>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                painted(child, id, clip, matches);
            }
        }
        LayerNodeKind::ClipRect {
            clip: area,
            child,
            clip_kind,
        } => {
            let own = *clip_kind == ClipKind::PrintBitmapFill && node.source_node_id == Some(id);
            painted(child, id, if own { Some(*area) } else { clip }, matches);
        }
        LayerNodeKind::ClipPath { child, .. } => painted(child, id, clip, matches),
        LayerNodeKind::Leaf { ops } if node.source_node_id == Some(id) => {
            for op in ops {
                if let PaintOp::Image { bbox, .. } = op {
                    matches.push((clip.expect("원본 그림의 독립 Print 클립"), *bbox));
                }
            }
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

fn verify(ext: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{ext}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(122).unwrap();
    let mut source_ids = Vec::new();
    source(&tree.root, &mut source_ids);
    assert_eq!(source_ids.len(), 1, "그림 원본 소유와 중복 방지");
    let layer = core
        .build_page_layer_tree_with_profile(122, RenderProfile::Print)
        .unwrap();
    let mut images = Vec::new();
    painted(&layer.root, source_ids[0], None, &mut images);
    assert_eq!(images.len(), 1, "그림 출력의 누락과 중복 방지");
    let (clip, full) = images[0];
    assert!(full.x <= clip.x && full.y <= clip.y);
    assert!(full.x + full.width >= clip.x + clip.width);
    assert!(full.y + full.height >= clip.y + clip.height);
    // 독립 Print PDF 좌표(pt)를 96dpi의 좌상단 원점으로 변환한다.
    let expected = [
        460.422 * 4.0 / 3.0,
        (754.0 - 144.939) * 4.0 / 3.0,
        (468.933 - 460.422) * 4.0 / 3.0,
        (144.939 - 137.140) * 4.0 / 3.0,
    ];
    for (actual, reference) in [clip.x, clip.y, clip.width, clip.height]
        .into_iter()
        .zip(expected)
    {
        assert!(
            (actual - reference).abs() <= 2.0,
            "독립 Print 클립: {actual} / {reference}"
        );
    }
}

#[test]
fn hwp_centered_cell_picture_keeps_print_clip() {
    verify("hwp");
}

#[test]
fn hwpx_centered_cell_picture_keeps_print_clip() {
    verify("hwpx");
}
