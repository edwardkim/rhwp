//! PNG의 생산 장치와 최종 소비 장치의 해상도를 구분한다.
//! 문서 배치의 px/golden이 아닌 합성 paint 프로토콜 검사다.
//! 독립 Print와 출처 태그 대조: #7445 stage56. C417의 무조건 선형 재생 가정을 정정한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::paint::{
    LayerNode, PageLayerTree, PaintOp, RenderProfile, ResolvedImageKind, ResolvedImagePayload,
};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode};
use rhwp::renderer::skia::SkiaLayerRenderer;

fn device_pattern() -> Vec<u8> {
    // 원본 샘플의 흑/백과 주기를 명시한다. 문서의 쪽·좌표·글꼴에 의존하지 않는다.
    let source = image::RgbaImage::from_fn(64, 32, |x, _| {
        let gray = if x % 3 == 2 { 255 } else { 0 };
        image::Rgba([gray, gray, gray, 255])
    });
    let mut encoded = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    encoded.into_inner()
}

fn replay(
    renderer: &SkiaLayerRenderer,
    bytes: &[u8],
    kind: ResolvedImageKind,
    width: f64,
    height: f64,
) -> image::RgbaImage {
    let bounds = BoundingBox::new(0.0, 0.0, width, height);
    let op = PaintOp::image(
        bounds.clone(),
        ImageNode::new(1, Some(bytes.to_vec())),
        Some(ResolvedImagePayload {
            data: bytes.to_vec(),
            mime: "image/png",
            kind,
            suppress_effects: false,
        }),
    );
    let tree = PageLayerTree::with_profile(
        bounds.width,
        bounds.height,
        LayerNode::leaf(bounds, None, vec![op]),
        RenderProfile::Print,
    );
    let output = renderer
        .render_raster_with_options(
            &tree,
            RasterRenderOptions {
                transparent: false,
                ..Default::default()
            },
        )
        .expect("실제 Native Print 소비자");
    image::load_from_memory(&output.bytes).unwrap().to_rgba8()
}

fn interior_minimum(image: &image::RgbaImage) -> u8 {
    image
        .enumerate_pixels()
        .filter(|(x, y, _)| *x > 0 && *x + 1 < image.width() && *y > 0 && *y + 1 < image.height())
        .map(|(_, _, pixel)| pixel[0])
        .min()
        .unwrap()
}

#[test]
fn native_print_filters_device_png_at_the_consumer_resolution() {
    let bytes = device_pattern();
    let renderer = SkiaLayerRenderer::new();
    for (width, height) in [(16.0, 8.0), (64.0, 32.0), (128.0, 64.0)] {
        let original = replay(
            &renderer,
            &bytes,
            ResolvedImageKind::FormatConverted,
            width,
            height,
        );
        let device = replay(
            &renderer,
            &bytes,
            ResolvedImageKind::OutputDevice,
            width,
            height,
        );
        assert_eq!(
            device, original,
            "동일 PNG와 최종 장치 사각형은 생산 출처 태그만으로 표본 선택이 바뀌지 않는다"
        );
        if width == 16.0 {
            // 네 source 픽셀마다 흰색이 적어도 하나 있다. 축소 구간의 흰색을
            // 건너뛰고 검은 점만 택하는 alias를 최종 표본 보존으로 인정하지 않는다.
            assert!(
                interior_minimum(&device) > 32,
                "축소 구간의 흰색 coverage를 보존한다"
            );
        } else if width == 64.0 {
            assert_eq!(
                device,
                image::load_from_memory(&bytes).unwrap().to_rgba8(),
                "소비 장치와 1:1인 PNG는 원래 픽셀을 보존한다"
            );
        }
        assert_eq!(
            replay(
                &renderer,
                &bytes,
                ResolvedImageKind::FormatConverted,
                width,
                height
            ),
            original,
            "장치 경로 뒤에도 원본 경로의 cache와 필터를 보존한다"
        );
        assert_eq!(
            replay(
                &renderer,
                &bytes,
                ResolvedImageKind::OutputDevice,
                width,
                height
            ),
            device,
            "같은 장치 표본의 반복 재생"
        );
    }
}

#[test]
fn transparent_print_reduction_integrates_fractional_coverage_without_hidden_rgb() {
    // 8×8 source의 왼쪽 절반만 불투명한 빨강이다. 투명한 파랑은
    // 흰 종이에 기여하지 않는다. 2×2 장치 사각형의 소수 원점에서
    // 첫 줄의 빨강 coverage는 3/4×3/4와 1/4×3/4로 독립 계산한다.
    let source = image::RgbaImage::from_fn(8, 8, |x, _| {
        if x < 4 {
            image::Rgba([255, 0, 0, 255])
        } else {
            image::Rgba([0, 0, 255, 0])
        }
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let bytes = bytes.into_inner();
    let renderer = SkiaLayerRenderer::new();
    let replay = |x: f64, kind: ResolvedImageKind| {
        let frame = BoundingBox::new(x, 0.25, 2.0, 2.0);
        let op = PaintOp::image(
            frame,
            ImageNode::new(1, Some(bytes.clone())),
            Some(ResolvedImagePayload {
                data: bytes.clone(),
                mime: "image/png",
                kind,
                suppress_effects: false,
            }),
        );
        let tree = PageLayerTree::with_profile(
            4.0,
            4.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 4.0, 4.0), None, vec![op]),
            RenderProfile::Print,
        );
        let result = renderer
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    transparent: false,
                    ..Default::default()
                },
            )
            .unwrap();
        image::load_from_memory(&result.bytes).unwrap().to_rgba8()
    };
    for kind in [
        ResolvedImageKind::FormatConverted,
        ResolvedImageKind::OutputDevice,
    ] {
        let first = replay(0.25, kind);
        for (x, expected) in [(0, 112u8), (1, 207u8)] {
            let pixel = first.get_pixel(x, 0);
            assert_eq!(pixel[0], 255);
            assert!(
                pixel[1].abs_diff(expected) <= 1,
                "흰 종이 합성에서 실제 coverage를 보존한다: {pixel:?}"
            );
            assert_eq!(pixel[1], pixel[2], "투명한 파랑의 숨은 RGB가 새지 않는다");
        }
        let shifted = replay(0.75, kind);
        assert_eq!(
            shifted.get_pixel(0, 0),
            first.get_pixel(1, 0),
            "같은 정수 크기라도 소수 원점이 다르면 cache 표본을 바꾼다"
        );
        assert_eq!(shifted.get_pixel(1, 0), first.get_pixel(0, 0));
        assert_eq!(
            replay(0.25, kind),
            first,
            "반복 요청은 같은 coverage를 보존한다"
        );
    }
}

#[test]
fn print_jpeg_crop_replays_the_integer_source_window_without_editing_layout() {
    use rhwp::paint::LayerBuilder;
    use rhwp::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

    // 합성 source의 원점과 길이를 독립적으로 정수화하는 Print 자원 계약.
    // 실물 근거: PrEP Print의114/124/125/129/137쪽 JPEG 자원 크기와
    // 원본 source 픽셀 대조(stage23 repair831). 문서의 배치 px 핀이 아니다.
    let source = image::RgbImage::from_fn(64, 32, |x, y| {
        image::Rgb([(x * 4) as u8, (y * 8) as u8, ((x + y) * 2) as u8])
    });
    let mut encoded = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Jpeg)
        .unwrap();
    let bytes = encoded.into_inner();
    let frame = BoundingBox::new(16.0, 16.0, 160.0, 80.0);
    let mut image = ImageNode::new(7, Some(bytes.clone()));
    image.section_index = Some(0);
    image.para_index = Some(3);
    image.control_index = Some(2);
    image.crop = Some((11, 11, 639, 319));
    image.original_size_hu = Some((640, 320));
    let mut tree = PageRenderTree::new(0, 240.0, 160.0);
    tree.root
        .children
        .push(RenderNode::new(8, RenderNodeType::Image(image), frame));
    let original = serde_json::to_value(&tree.root).unwrap();

    let mut expected = tree.clone();
    let RenderNodeType::Image(expected_image) = &mut expected.root.children[0].node_type else {
        unreachable!()
    };
    // 원본(1.1,1.1), 크기62.8×30.8 -> 원점(1,1), 크기62×30.
    // 보이는 팔레트의 완전 내포 창(2,2)..(63,31)과 구분한다.
    expected_image.crop = Some((1, 1, 63, 31));
    expected_image.original_size_hu = Some((64, 32));
    let mut builder = LayerBuilder::new(RenderProfile::Print).with_hancom2024_print_bitmap(true);
    let actual = builder.build(&tree);
    let expected = builder.build(&expected);
    let renderer = SkiaLayerRenderer::new();
    let options = RasterRenderOptions {
        transparent: false,
        ..Default::default()
    };
    let actual_png = renderer
        .render_raster_with_options(&actual, options)
        .unwrap();
    let expected_png = renderer
        .render_raster_with_options(&expected, options)
        .unwrap();
    assert!(
        image::load_from_memory(&actual_png.bytes)
            .unwrap()
            .to_rgba8()
            == image::load_from_memory(&expected_png.bytes)
                .unwrap()
                .to_rgba8(),
        "Print의 실제 source 창은 원점·길이를 각각 정수화한 대조군과 같아야 한다"
    );
    assert_eq!(
        serde_json::to_value(&tree.root).unwrap(),
        original,
        "원본 crop·소속·배치를 바꾸지 않는다"
    );
    let legacy_print = LayerBuilder::new(RenderProfile::Print).build(&tree);
    let legacy_json: serde_json::Value = serde_json::from_str(&legacy_print.to_json()).unwrap();
    assert!(
        legacy_json.to_string().contains("\"left\":11"),
        "2024 Print로 지정하지 않은 출력은 원본 crop을 유지한다"
    );
    let screen = LayerBuilder::new(RenderProfile::Screen)
        .with_hancom2024_print_bitmap(true)
        .build(&tree);
    let screen_json: serde_json::Value = serde_json::from_str(&screen.to_json()).unwrap();
    assert!(
        screen_json.to_string().contains("\"left\":11"),
        "Screen은 원본 crop을 유지한다"
    );
}

#[test]
fn cropped_jpeg_minification_matches_the_same_source_pixels_materialized_before_replay() {
    use rhwp::paint::{LayerBuilder, LayerJsonOptions};
    use rhwp::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

    let source = image::load_from_memory(&device_pattern()).unwrap();
    let mut jpeg = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut jpeg, image::ImageFormat::Jpeg)
        .unwrap();
    let jpeg = jpeg.into_inner();
    // 같은 JPEG를 디코드한 정수 source 창을 별도 PNG로 준비한 대조군이다.
    // 문서나 PDF의 답안 그림을 넣지 않고 실제 source 픽셀의 동등성을 검사한다.
    let source = image::load_from_memory(&jpeg).unwrap();
    let mut window = std::io::Cursor::new(Vec::new());
    source
        .crop_imm(1, 1, 62, 30)
        .write_to(&mut window, image::ImageFormat::Png)
        .unwrap();
    let frame = BoundingBox::new(16.0, 16.0, 16.0, 8.0);
    let mut image = ImageNode::new(7, Some(jpeg));
    image.crop = Some((11, 11, 639, 319));
    image.original_size_hu = Some((640, 320));
    let mut tree = PageRenderTree::new(0, 64.0, 64.0);
    tree.root
        .children
        .push(RenderNode::new(8, RenderNodeType::Image(image), frame));
    let original = serde_json::to_value(&tree.root).unwrap();
    let mut reference = tree.clone();
    let RenderNodeType::Image(image) = &mut reference.root.children[0].node_type else {
        unreachable!()
    };
    image.data = Some(window.into_inner());
    image.crop = None;
    image.original_size_hu = None;
    let mut builder = LayerBuilder::new(RenderProfile::Print).with_hancom2024_print_bitmap(true);
    let actual = builder.build(&tree);
    let reference = builder.build(&reference);
    let renderer = SkiaLayerRenderer::new();
    let options = RasterRenderOptions {
        transparent: false,
        ..Default::default()
    };
    let actual_png = renderer
        .render_raster_with_options(&actual, options)
        .unwrap();
    let reference_png = renderer
        .render_raster_with_options(&reference, options)
        .unwrap();
    assert!(
        image::load_from_memory(&actual_png.bytes)
            .unwrap()
            .to_rgba8()
            == image::load_from_memory(&reference_png.bytes)
                .unwrap()
                .to_rgba8(),
        "같은 source 픽셀 창은 crop의 유무로 축소 필터가 달라지면 안 된다"
    );
    assert_eq!(serde_json::to_value(&tree.root).unwrap(), original);
    let json: serde_json::Value =
        serde_json::from_str(&actual.to_json_with_options(LayerJsonOptions {
            omit_image_bytes: true,
            ..Default::default()
        }))
        .unwrap();
    fn check_image(value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Object(object) => {
                let found =
                    usize::from(object.get("type").and_then(|v| v.as_str()) == Some("image"));
                if found == 1 {
                    assert!(
                        object.get("sourceImageKey").is_none(),
                        "원본 bin 키로 다른 source 창을 되찾게 하지 않는다"
                    );
                    assert!(
                        object
                            .get("base64")
                            .and_then(|v| v.as_str())
                            .is_some_and(|s| !s.is_empty()),
                        "실제 창의 바이트를 소비자에게 전달한다"
                    );
                }
                found + object.values().map(check_image).sum::<usize>()
            }
            serde_json::Value::Array(array) => array.iter().map(check_image).sum(),
            _ => 0,
        }
    }
    assert_eq!(check_image(&json), 1, "source 그림의 누락·중복 방지");
}

#[test]
fn oversized_jpeg_source_window_fits_the_existing_decode_budget_without_accepting_full_source() {
    use rhwp::paint::{LayerBuilder, LayerNodeKind};
    use rhwp::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

    // 원본은 기존32Mpix 한도를 조금 넘고, 절반의 실제 창은 한도 안이다.
    let (width, height) = (8192, 4097);
    let source = image::RgbImage::from_pixel(width, height, image::Rgb([71, 97, 131]));
    let mut encoded = std::io::Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Jpeg)
        .unwrap();
    drop(source);
    let mut image = ImageNode::new(7, Some(encoded.into_inner()));
    image.original_size_hu = Some((width, height));
    image.crop = Some((0, 0, width as i32 / 2, height as i32));
    let mut tree = PageRenderTree::new(0, 64.0, 64.0);
    tree.root.children.push(RenderNode::new(
        8,
        RenderNodeType::Image(image),
        BoundingBox::new(16.0, 16.0, 16.0, 8.0),
    ));
    let original = serde_json::to_value(&tree.root).unwrap();
    let mut builder = LayerBuilder::new(RenderProfile::Print).with_hancom2024_print_bitmap(true);
    let cropped = builder.build(&tree);
    fn first_image(node: &LayerNode) -> Option<&PaintOp> {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().find_map(first_image),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                first_image(child)
            }
            LayerNodeKind::Leaf { ops } => {
                ops.iter().find(|op| matches!(op, PaintOp::Image { .. }))
            }
        }
    }
    let PaintOp::Image {
        resolved: Some(payload),
        ..
    } = first_image(&cropped.root).unwrap()
    else {
        panic!("한도 안의 실제 source 창을 공통 producer에서 발행해야 한다");
    };
    assert_eq!(payload.kind, ResolvedImageKind::SourceWindow);
    let cropped_policy = rhwp::renderer::canvaskit_policy::analyze_canvaskit_replay_plan(
        &cropped,
        rhwp::renderer::canvaskit_policy::CanvasKitReplayMode::Default,
    );
    assert_eq!(
        cropped_policy.summary.unsupported_items, 0,
        "정적 지원 계획도 실제 창의 바이트를 검사한다"
    );
    assert_eq!(cropped_policy.summary.hidden_overlay_violations, 0);
    let dimensions = image::ImageReader::new(std::io::Cursor::new(&payload.data))
        .with_guessed_format()
        .unwrap()
        .into_dimensions()
        .unwrap();
    assert_eq!(dimensions, (width / 2, height));
    assert!(u64::from(dimensions.0) * u64::from(dimensions.1) <= 32 * 1024 * 1024);
    assert_eq!(serde_json::to_value(&tree.root).unwrap(), original);
    let RenderNodeType::Image(image) = &mut tree.root.children[0].node_type else {
        unreachable!()
    };
    image.crop = None;
    let full = builder.build(&tree);
    let PaintOp::Image { resolved, .. } = first_image(&full.root).unwrap() else {
        unreachable!()
    };
    assert!(
        resolved.is_none(),
        "한도를 넘는 원본 전체를 지원 자원으로 표시하지 않는다"
    );
    let full_policy = rhwp::renderer::canvaskit_policy::analyze_canvaskit_replay_plan(
        &full,
        rhwp::renderer::canvaskit_policy::CanvasKitReplayMode::Default,
    );
    assert_eq!(
        full_policy.summary.hidden_overlay_violations, 1,
        "원본 전체는 직접 재생이 불가능하며 숨은 overlay 우회도 허용하지 않는다"
    );
}
