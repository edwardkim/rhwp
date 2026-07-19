//! Issue #2318: 바탕쪽 개체가 studio 다층 canvas 합성에서 본문 텍스트를 가림.
//!
//! shortcut.hwp 의 바탕쪽 글상자는 wrap=InFrontOfText(글 앞으로)로 저장되어 있다.
//! 한컴 의미론에서 바탕쪽 개체의 wrap 은 바탕쪽 내부 개체 간 순서에만 적용되고
//! 바탕쪽 전체는 항상 본문 뒤에 깔린다 (한컴 2022 실기 확인, 2026-07-17).
//!
//! paint replay plane 분류가 이 wrap 을 본문 기준으로 해석하면 바탕쪽 개체가
//! inFrontOfText plane 으로 승격되어 plane 재생 backend 전체(studio front
//! overlay canvas 합성 + skia per-plane 재생)에서 본문 텍스트를 가린다
//! (skia PNG 실측: 바탕쪽 쪽번호 "1" 이 "Ctrl+Y" 를 덮음). 정정:
//! RenderLayerInfo.master_page provenance 로 plane 을 BehindText 상한 고정
//! (전지 배경 강등 가드는 별도 유지).
//!
//! SVG 는 node_z_plane 이 MasterPage 노드를 plane 1 로 직접 배치(#1167)하므로
//! 이 분류와 무관하게 원래 정상이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::paint::{
    paint_op_replay_plane_with_layer, GroupKind, LayerNode, LayerNodeKind, PaintReplayPlane,
};
use rhwp::renderer::render_tree::RenderLayerInfo;

fn load_shortcut_core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/basic/shortcut.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse shortcut.hwp")
}

/// MasterPage 그룹을 찾아 반환한다 (layer 상속 컨텍스트 포함).
fn find_master_group(
    node: &LayerNode,
    inherited: Option<RenderLayerInfo>,
) -> Option<(LayerNode, Option<RenderLayerInfo>)> {
    let active = node.layer.or(inherited);
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            if matches!(group_kind, GroupKind::MasterPage) {
                return Some((node.clone(), inherited));
            }
            children
                .iter()
                .find_map(|child| find_master_group(child, active))
        }
        LayerNodeKind::ClipRect { child, .. } => find_master_group(child, active),
        LayerNodeKind::Leaf { .. } => None,
    }
}

/// 그룹 내부 모든 op 의 replay plane 을 수집한다 (layer 상속 규칙 동일).
fn collect_planes(
    node: &LayerNode,
    inherited: Option<RenderLayerInfo>,
    out: &mut Vec<PaintReplayPlane>,
) {
    let active = node.layer.or(inherited);
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_planes(child, active, out);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => collect_planes(child, active, out),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                out.push(paint_op_replay_plane_with_layer(op, active));
            }
        }
    }
}

/// 바탕쪽 그룹 내부의 모든 op 는 BehindText 이하 plane 이어야 한다.
/// 본문 기준 front/flow plane 으로 승격되면 studio 합성에서 본문 텍스트를 가린다.
#[test]
fn issue_2318_master_page_ops_capped_at_behind_text() {
    let core = load_shortcut_core();
    let tree = core.build_page_layer_tree(0).expect("page 1 PageLayerTree");

    let (master, inherited) =
        find_master_group(&tree.root, None).expect("shortcut.hwp p1 에 MasterPage 그룹 존재");

    let mut planes = Vec::new();
    collect_planes(&master, inherited, &mut planes);
    assert!(!planes.is_empty(), "바탕쪽 그룹에 paint op 존재");

    let escalated: Vec<_> = planes
        .iter()
        .filter(|p| {
            !matches!(
                p,
                PaintReplayPlane::Background | PaintReplayPlane::BehindText
            )
        })
        .collect();
    assert!(
        escalated.is_empty(),
        "바탕쪽 op {}개가 본문 기준 plane 으로 승격됨 (한컴=바탕쪽은 본문 뒤): {:?}",
        escalated.len(),
        escalated
    );
}

/// 바탕쪽 wrap=InFrontOfText 원본 속성 자체는 보존되어야 한다 —
/// 바탕쪽 내부 정렬(sort_paper_render_nodes)의 근거이므로 layer 를 소거하는
/// 방식(wrap 덮어쓰기)이 아니라 plane 분류층에서 cap 해야 한다.
#[test]
fn issue_2318_master_page_layer_wrap_preserved_for_internal_order() {
    let core = load_shortcut_core();
    let tree = core.build_page_layer_tree(0).expect("page 1 PageLayerTree");

    let (master, _) =
        find_master_group(&tree.root, None).expect("shortcut.hwp p1 에 MasterPage 그룹 존재");

    fn has_front_wrap_layer(node: &LayerNode) -> bool {
        let self_front = node
            .layer
            .and_then(|l| l.text_wrap)
            .map(|w| matches!(w, rhwp::model::shape::TextWrap::InFrontOfText))
            .unwrap_or(false);
        if self_front {
            return true;
        }
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().any(has_front_wrap_layer),
            LayerNodeKind::ClipRect { child, .. } => has_front_wrap_layer(child),
            LayerNodeKind::Leaf { .. } => false,
        }
    }
    assert!(
        has_front_wrap_layer(&master),
        "바탕쪽 글상자의 원본 wrap(InFrontOfText) layer 는 내부 정렬용으로 보존"
    );
}
