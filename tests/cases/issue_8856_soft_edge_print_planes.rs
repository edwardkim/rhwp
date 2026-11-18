//! 검사 유형: 관계형. 설치 글꼴 없는 메모리 PNG의 공통 Print 자원 소유를 검사한다.
//! 주석 갱신: 2026-11-10T05:40:00+09:00.
//! 독립 한컴 2024 대조군은 1bit 마스크와 RGB를 별도 재표본한다.
//! stage8 alpha-half-r1000-2024 Native/fresh WASM raw는 각각 97.98599%다.
//! 이는 원본 문서의 고정 px 배치 검사가 아니며, raw 100%와 RGB 일치를 구분한다.

use rhwp::paint::{
    LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind,
};
use rhwp::renderer::render_tree::{
    BoundingBox, ImageNode, PageRenderTree, RenderNode, RenderNodeType,
};
use std::io::Cursor;

fn prepared(
    profile: RenderProfile,
    radius: Option<f64>,
) -> (ImageNode, Option<rhwp::paint::ResolvedImagePayload>) {
    fn find(node: &LayerNode) -> Option<(ImageNode, Option<rhwp::paint::ResolvedImagePayload>)> {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().find_map(find),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                find(child)
            }
            LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| match op {
                PaintOp::Image {
                    image, resolved, ..
                } => Some((image.as_ref().clone(), resolved.as_deref().cloned())),
                _ => None,
            }),
        }
    }
    let pixels = image::RgbaImage::from_pixel(400, 240, image::Rgba([64, 128, 192, 128]));
    let mut bytes = Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let mut image = ImageNode::new(1, Some(bytes.into_inner()));
    image.soft_edge_radius_hu = radius;
    let mut tree = PageRenderTree::new(0, 800.0, 600.0);
    tree.root.children.push(RenderNode::new(
        1,
        RenderNodeType::Image(image),
        BoundingBox::new(50.0, 50.0, 400.0, 240.0),
    ));
    let layers = LayerBuilder::new(profile)
        .with_hancom2024_print_bitmap(true)
        .with_metafile_device_pixel_width(Some(96.0 / 600.0))
        .build(&tree);
    find(&layers.root).expect("실제 공통 PaintOp 이미지")
}

#[test]
fn print_effect_owns_device_mask_and_source_alpha_color_as_separate_planes() {
    let (_, payload) = prepared(RenderProfile::Print, Some(1000.0));
    let payload = payload.expect("효과 표본은 원본 bin 조회와 구분한다");
    assert_eq!(
        payload.kind,
        ResolvedImageKind::OutputDeviceWindow,
        "Print가 일반 RGBA 필터로 되돌아가면 실패"
    );
    let pixels = image::load_from_memory(&payload.data).unwrap().to_rgba8();
    assert_eq!(
        pixels.dimensions(),
        (2500, 1500),
        "그림 크기와 600dpi 장치 마스크의 단위 계약"
    );
    let mut opaque = 0;
    for y in 640..672 {
        for x in 1200..1232 {
            let p = pixels.get_pixel(x, y);
            assert!(
                p[3] == 0 || p[3] == 255,
                "연속 alpha를 일반 PNG 필터에 맡기지 않는다"
            );
            if p[3] == 255 {
                opaque += 1;
                assert_eq!(
                    &p.0[..3],
                    &[63, 127, 191],
                    "원 alpha 128의 색 전달을 효과 alpha로 다시 계산하지 않는다"
                );
            } else {
                assert_eq!(
                    &p.0[..3],
                    &[255, 255, 255],
                    "색 평면의 빈 표본은 마스크와 별도로 흰색을 보존"
                );
            }
        }
    }
    assert_eq!(opaque, 512, "Bayer 16x16 마스크의 중앙 alpha 128 밀도");
}

#[test]
fn screen_and_no_effect_keep_their_existing_sampling_contracts() {
    let (_, screen) = prepared(RenderProfile::Screen, Some(1000.0));
    assert_eq!(screen.unwrap().kind, ResolvedImageKind::SourceWindow);
    let (_, no_effect) = prepared(RenderProfile::Print, None);
    assert!(
        no_effect.is_none_or(|p| p.kind != ResolvedImageKind::OutputDeviceWindow),
        "효과 없는 그림의 기존 전달을 확장하지 않는다"
    );
}
