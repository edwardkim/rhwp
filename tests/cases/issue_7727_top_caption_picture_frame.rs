//! 검사 유형: 관계형 — 위 캡션 뒤 그림과 소유 테두리의 같은 개체 프레임을 검사한다.
//! 검사 유형: 고정형 — Print118쪽의 독립 그림 테두리 네 변을 ±2 CSSpx로 보호한다.
//! 주석 갱신: 2026-11-14T23:28:54+09:00.
//! CBTA118쪽은 Native/fresh WASM 양쪽98.44%, 원본204쪽이다.
//! 2026-11-14 정본 PNG Print / compat2022 / 96DPI / raw2px의 같은 쪽 review·overlay를 판독했다.
//! 독립 한컴2020 Print PDF의 그림 테두리 경계는149.6800/211.9280/656.9267/923.7893 CSSpx다.
//! PDF의 비트맵 원래 외곽은 clip 전 크기이므로 이 고정 기대값으로 혼용하지 않는다.
//! 그림 원점은 위 캡션 때문에 약27px 움직인다. 테두리만 남는 결함을 검출한다.
//! 근거: pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf,
//! mydocs/working/task_m100_7727_stage1.md. 글꼴 설치나 특정 글리프 폭에 의존하지 않는다.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn flatten<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        flatten(child, nodes);
    }
}

#[test]
fn top_caption_moves_the_picture_and_its_owned_border_together() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp"
    ))
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(117).unwrap();
    let mut nodes = Vec::new();
    flatten(&tree.root, &mut nodes);
    let images: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(image)
            if image.para_index == Some(1099) && image.control_index == Some(0))
        })
        .collect();
    assert_eq!(images.len(), 1, "위 캡션 그림의 누락·중복");
    let image = images[0];
    let borders: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Rectangle(rect)
            if rect.image_frame_owner == Some(image.id))
        })
        .collect();
    assert_eq!(borders.len(), 1, "그림 소유 테두리의 누락·중복");
    let border = borders[0];
    let RenderNodeType::Rectangle(rect) = &border.node_type else {
        unreachable!()
    };
    let pen = rect.style.stroke_width;
    // 같은 그림의 프레임 가장자리는 선언 획 안에서 만난다. 캡션 띠를
    // 별도 좌표 허용치로 흡수하지 않으며, 한 HU는 측정 양자만 반영한다.
    let quantum = 96.0 / 7200.0;
    for (label, actual, expected) in [
        ("위", border.bbox.y, image.bbox.y),
        (
            "아래",
            border.bbox.y + border.bbox.height,
            image.bbox.y + image.bbox.height,
        ),
        ("왼쪽", border.bbox.x, image.bbox.x),
        (
            "오른쪽",
            border.bbox.x + border.bbox.width,
            image.bbox.x + image.bbox.width,
        ),
    ] {
        assert!(
            (actual - expected).abs() <= pen + quantum,
            "{label} 테두리가 그림 프레임에서 분리됨: {actual} / {expected}, 획={pen}"
        );
    }
}

#[test]
fn top_caption_border_keeps_its_independent_print_frame() {
    use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp"
    ))
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core
        .build_page_layer_tree_with_profile(117, RenderProfile::Print)
        .unwrap();
    fn borders(node: &LayerNode, out: &mut Vec<rhwp::renderer::render_tree::BoundingBox>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    borders(child, out);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                borders(child, out)
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::Rectangle { bbox, rect } = op {
                        if rect.image_frame_owner.is_some() {
                            out.push(*bbox);
                        }
                    }
                }
            }
        }
    }
    let mut found = Vec::new();
    borders(&tree.root, &mut found);
    assert_eq!(found.len(), 1, "그림 소유 테두리 누락·중복");
    let actual = found[0];
    for (label, observed, independent) in [
        ("왼쪽", actual.x, 149.68000284830728),
        ("위", actual.y, 211.927978515625),
        ("오른쪽", actual.x + actual.width, 656.9266764322916),
        ("아래", actual.y + actual.height, 923.789306640625),
    ] {
        assert!(
            (observed - independent).abs() <= 2.0,
            "{label}: 실제{observed}px, 독립 Print{independent}px"
        );
    }
}
