//! 검사 유형: 관계형. 설치 글꼴을 사용하지 않는 공개 메모리 PNG 대조군.
//! 주석 갱신: 2026-11-10T05:12:00+09:00.
//! 독립 한컴 2020/2024의 단색·구멍·alpha 128 대조군에서 확인한 효과 소유를 보호한다.
//! 실제 원본 interview의 보정 전 Native/fresh WASM 최저는 97.19457/97.32378%다.
//! 고정 px 좌표나 새 효과의 시각 통과를 주장하지 않는다.

use rhwp::paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{
    BoundingBox, ImageNode, PageRenderTree, RenderNode, RenderNodeType,
};
use std::io::Cursor;

fn source(alpha: u8, hole: bool) -> Vec<u8> {
    let mut pixels = image::RgbaImage::from_pixel(400, 240, image::Rgba([64, 128, 192, alpha]));
    if hole {
        for y in 96..144 {
            for x in 160..240 {
                pixels.get_pixel_mut(x, y)[3] = 0;
            }
        }
    }
    let mut bytes = Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

fn paint(image: ImageNode) -> (ImageNode, image::RgbaImage) {
    fn leaf(node: &LayerNode) -> Option<(&ImageNode, &[u8])> {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().find_map(leaf),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                leaf(child)
            }
            LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| match op {
                PaintOp::Image {
                    image, resolved, ..
                } => Some((
                    image.as_ref(),
                    resolved
                        .as_ref()
                        .map_or_else(|| image.data.as_deref().unwrap(), |p| p.data.as_slice()),
                )),
                _ => None,
            }),
        }
    }
    let mut tree = PageRenderTree::new(0, 800.0, 600.0);
    tree.root.children.push(RenderNode::new(
        1,
        RenderNodeType::Image(image.clone()),
        BoundingBox::new(10.0, 10.0, 400.0, 240.0),
    ));
    let layers = LayerBuilder::new(RenderProfile::Screen).build(&tree);
    let (prepared, bytes) = leaf(&layers.root).expect("실제 이미지 PaintOp");
    let result = image::load_from_memory(bytes).unwrap().to_rgba8();
    if let RenderNodeType::Image(original) = &tree.root.children[0].node_type {
        assert_eq!(
            original.data, image.data,
            "공통 준비가 원본 소스를 변경하지 않는다"
        );
        assert_eq!(original.crop, image.crop);
    }
    if image.soft_edge_radius_hu.is_some_and(|r| r > 0.0) {
        fn assert_prepared_key(node: &LayerNode) {
            match &node.kind {
                LayerNodeKind::Group { children, .. } => {
                    children.iter().for_each(assert_prepared_key)
                }
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                    assert_prepared_key(child)
                }
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        if let PaintOp::Image { resolved, .. } = op {
                            assert_eq!(
                                resolved
                                    .as_ref()
                                    .expect("효과는 원본 bin 재조회와 구분")
                                    .kind,
                                rhwp::paint::ResolvedImageKind::SourceWindow
                            );
                        }
                    }
                }
            }
        }
        assert_prepared_key(&layers.root);
    }
    (prepared.clone(), result)
}

#[test]
fn soft_edge_paint_feathers_outer_and_transparent_hole_edges_without_squaring_alpha() {
    for alpha in [128, 255] {
        let mut image = ImageNode::new(1, Some(source(alpha, true)));
        image.soft_edge_radius_hu = Some(1000.0);
        image.opacity = 0.75;
        let (prepared, pixels) = paint(image);
        assert_eq!(prepared.opacity, 0.75, "전역 투명도는 별도로 한 번만 소비");
        let mid = pixels.get_pixel(80, 120);
        assert_eq!(mid.0, [64, 128, 192, alpha], "중앙 RGB·기존 alpha 보존");
        let edge = pixels.get_pixel(2, 120)[3];
        assert!(
            edge < alpha,
            "반경이 전달만 되고 경계 도색에서 버려지면 실패"
        );
        assert!(
            pixels.get_pixel(2, 2)[3] < edge,
            "모서리는 두 축의 감쇠를 소비"
        );
        assert_eq!(
            pixels.get_pixel(200, 120)[3],
            0,
            "원래 투명한 구멍을 채우지 않는다"
        );
        assert!(
            pixels.get_pixel(156, 120)[3] < alpha,
            "내부 구멍의 불투명 쪽도 감쇠"
        );
    }
}

#[test]
fn soft_edge_is_applied_after_crop_while_zero_and_missing_radius_preserve_source() {
    let bytes = source(255, false);
    for radius in [None, Some(0.0)] {
        let mut image = ImageNode::new(2, Some(bytes.clone()));
        image.soft_edge_radius_hu = radius;
        let (prepared, _) = paint(image);
        assert_eq!(
            prepared.data.as_deref(),
            Some(bytes.as_slice()),
            "무효과는 재인코딩하지 않는다"
        );
    }
    let mut image = ImageNode::new(3, Some(bytes));
    image.soft_edge_radius_hu = Some(1000.0);
    image.original_size_hu = Some((40000, 24000));
    image.crop = Some((4000, 2400, 36000, 21600));
    let (_, pixels) = paint(image);
    assert!(
        pixels.get_pixel(2, 120)[3] < 255,
        "원본 외곽이 아닌 새 crop 경계에서 감쇠"
    );
}

#[test]
fn soft_edge_does_not_downsample_or_blur_the_rgb_picture() {
    let source = image::RgbaImage::from_fn(800, 480, |x, _| {
        if x % 2 == 0 {
            image::Rgba([10, 30, 90, 255])
        } else {
            image::Rgba([200, 150, 100, 255])
        }
    });
    let mut bytes = Cursor::new(Vec::new());
    source
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let mut node = ImageNode::new(4, Some(bytes.into_inner()));
    node.soft_edge_radius_hu = Some(1000.0);
    let (_, pixels) = paint(node);
    assert_eq!(
        pixels.dimensions(),
        source.dimensions(),
        "효과용 저해상도 alpha가 사진 해상도를 낮추지 않는다"
    );
    for x in 300..500 {
        assert_eq!(
            pixels.get_pixel(x, 240),
            source.get_pixel(x, 240),
            "alpha 효과가 중앙 RGB의 한 픽셀 세부를 흐리지 않는다"
        );
    }
}
