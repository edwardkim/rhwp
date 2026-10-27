//! #8153: Q 셀 배경의 물리 Print 창과 본문 clip의 소유를 구분한다.
//! 두 원본 291·301쪽은 Native/fresh DEV WASM에서 각각 raw 100%·미대응 0이다.
//! 독립 Print 확대 판독과 수정 전 FAIL/후 PASS는
//! `mydocs/plans/issue_8153_cell_bitmap_clip_20261027.md`에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::style::ImageFillMode,
    paint::{ClipKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::{NodeId, RenderNode, RenderNodeType},
    DocumentCore,
};

type ClipOwner = (ClipKind, Option<NodeId>);

fn source_backgrounds(
    node: &RenderNode,
    expected_fill: ImageFillMode,
    found: &mut Vec<(NodeId, NodeId)>,
) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        for child in &node.children {
            let RenderNodeType::Image(image) = &child.node_type else {
                continue;
            };
            // 두 원본에서 Q 아이콘을 공급하는 BinData다. 분할 쪽 컷은 별도 계약이다.
            if image.bin_data_id != 352 || !cell.clip || cell.page_fragment {
                continue;
            }
            assert_eq!(image.fill_mode, Some(expected_fill));
            assert_eq!((cell.row, cell.col), (0, 0), "원본 Q 배경의 셀 소유");
            found.push((node.id, child.id));
        }
    }
    for child in &node.children {
        source_backgrounds(child, expected_fill, found);
    }
}

fn image_clips(
    node: &LayerNode,
    image_id: NodeId,
    ancestors: &mut Vec<ClipOwner>,
    found: &mut Vec<Vec<ClipOwner>>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                image_clips(child, image_id, ancestors, found);
            }
        }
        LayerNodeKind::ClipRect {
            clip_kind, child, ..
        } => {
            ancestors.push((*clip_kind, node.source_node_id));
            image_clips(child, image_id, ancestors, found);
            ancestors.pop();
        }
        LayerNodeKind::ClipPath { child, .. } => {
            image_clips(child, image_id, ancestors, found);
        }
        LayerNodeKind::Leaf { ops } if node.source_node_id == Some(image_id) => {
            for op in ops {
                if matches!(op, PaintOp::Image { .. }) {
                    found.push(ancestors.clone());
                }
            }
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

fn verify(extension: &str, page: u32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let semantic = core.build_page_render_tree(page - 1).unwrap();
    let mut sources = Vec::new();
    // HWP5 표 31의 값 5와 HWPX imgBrush의 TOTAL을 원본 그대로 구분한다.
    let expected_fill = match extension {
        "hwp" => ImageFillMode::FitToSize,
        "hwpx" => ImageFillMode::Total,
        _ => unreachable!("검증 대상 두 원본 형식"),
    };
    source_backgrounds(&semantic.root, expected_fill, &mut sources);
    assert!(!sources.is_empty(), "검사할 일반 셀의 Q 배경이 있어야 한다");

    let layer = core
        .build_page_layer_tree_with_profile(page - 1, RenderProfile::Print)
        .unwrap();
    for (cell_id, image_id) in sources {
        let mut found = Vec::new();
        image_clips(&layer.root, image_id, &mut Vec::new(), &mut found);
        assert_eq!(found.len(), 1, "원본 배경의 paint 누락·중복 방지");
        let clips = &found[0];
        assert!(
            clips.contains(&(ClipKind::PrintBitmapFill, Some(image_id))),
            "그림 자체의 물리 Print 창은 유지한다"
        );
        assert!(
            !clips.contains(&(ClipKind::TableCell, Some(cell_id))),
            "일반 셀 배경의 물리 Print 창에 본문 논리 clip을 다시 적용하지 않는다"
        );
        assert!(
            clips.iter().any(|(kind, _)| *kind == ClipKind::Body),
            "배경도 상위 본문 clip의 소유를 유지한다"
        );
    }
}

#[test]
fn hwp_p291_background_keeps_own_print_clip() {
    verify("hwp", 291);
}

#[test]
fn hwp_p301_background_keeps_own_print_clip() {
    verify("hwp", 301);
}

#[test]
fn hwpx_p291_background_keeps_own_print_clip() {
    verify("hwpx", 291);
}

#[test]
fn hwpx_p301_background_keeps_own_print_clip() {
    verify("hwpx", 301);
}
