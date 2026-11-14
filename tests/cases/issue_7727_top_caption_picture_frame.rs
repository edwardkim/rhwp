//! 검사 유형: 관계형 — 위 캡션 뒤의 그림과 소유 테두리가 같은 개체 프레임을 유지한다.
//! 주석 갱신: 2026-11-14T21:18:23+09:00.
//! CBTA 전204쪽은 보정 전 Native/fresh WASM 최저96.35%다. 98% 미만인 상태에서
//! 독립 Print의 절대 좌표를 고정하지 않고, 그림 ID로 연결된 테두리의 소유·상대 배치를 검사한다.
//! 원본118쪽의 위 캡션은 이미지 원점을 약27px 옮긴다. 테두리만 캡션 위에 남으면 실패한다.
//! 시스템 글꼴의 설치 여부나 특정 글리프 폭을 기대값으로 쓰지 않는다.

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
