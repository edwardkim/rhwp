//! 공개 Native raster 경로의 기존 단위 검사를 integration 경로로 이관한다.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::{LayerRasterRenderer, RasterRenderOptions};
use rhwp::renderer::render_tree::{
    BoundingBox, ImageNode, PageBackgroundNode, PathNode, RawSvgNode, RectangleNode, TextRunNode,
};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::{GradientFillInfo, PathCommand, ShapeStyle, TextStyle};
fn decode_rgba(bytes: &[u8]) -> image::RgbaImage {
    image::load_from_memory(bytes).unwrap().to_rgba8()
}
fn assert_channel(pixel: image::Rgba<u8>, channel: usize, min: u8, max: u8) {
    assert!(
        pixel[channel] >= min && pixel[channel] <= max,
        "pixel={pixel:?} channel={channel} expected={min}..={max}"
    );
}
fn count_ink(image: &image::RgbaImage) -> usize {
    image.pixels().filter(|p| p[3] > 0).count()
}

#[test]
fn renders_gradient_background_hard_stops_opacity_and_radial_shapes() {
    let linear = GradientFillInfo {
        gradient_type: 1,
        angle: 90,
        center_x: 0,
        center_y: 50,
        colors: vec![0x000000ff, 0x00ff0000],
        positions: vec![0.0, 1.0],
    };
    let background = PageBackgroundNode {
        gradient: Some(Box::new(linear.clone())),
        background_color: None,
        border_color: None,
        border_width: 0.0,
        image: None,
    };
    let hard_stop = GradientFillInfo {
        colors: vec![0x000000ff, 0x000000ff, 0x00ff0000, 0x00ff0000],
        positions: vec![0.0, 0.5, 0.5, 1.0],
        ..linear.clone()
    };
    let radial = GradientFillInfo {
        gradient_type: 2,
        center_x: 50,
        ..linear.clone()
    };
    let tree = PageLayerTree::new(
        120.0,
        120.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 120.0, 120.0),
            None,
            vec![
                PaintOp::page_background(BoundingBox::new(0.0, 0.0, 120.0, 120.0), background),
                PaintOp::rectangle(
                    BoundingBox::new(10.0, 10.0, 40.0, 20.0),
                    RectangleNode::new(
                        0.0,
                        ShapeStyle {
                            opacity: 0.5,
                            ..Default::default()
                        },
                        Some(Box::new(hard_stop)),
                    ),
                ),
                PaintOp::ellipse(
                    BoundingBox::new(10.0, 50.0, 40.0, 20.0),
                    rhwp::renderer::render_tree::EllipseNode::new(
                        ShapeStyle::default(),
                        Some(Box::new(radial)),
                    ),
                ),
                PaintOp::path(
                    BoundingBox::new(60.0, 50.0, 40.0, 20.0),
                    PathNode::new(
                        vec![
                            PathCommand::MoveTo(60.0, 50.0),
                            PathCommand::LineTo(100.0, 50.0),
                            PathCommand::LineTo(100.0, 70.0),
                            PathCommand::LineTo(60.0, 70.0),
                            PathCommand::ClosePath,
                        ],
                        ShapeStyle::default(),
                        Some(Box::new(linear)),
                    ),
                ),
            ],
        ),
    );
    let output = SkiaLayerRenderer::new()
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .expect("render gradient consumers");
    let image = decode_rgba(&output.bytes);
    let left = *image.get_pixel(2, 115);
    let right = *image.get_pixel(117, 115);
    assert_channel(left, 0, 240, 255);
    assert_channel(right, 2, 240, 255);
    let before_stop = *image.get_pixel(29, 20);
    let after_stop = *image.get_pixel(30, 20);
    assert!(
        before_stop[0] > after_stop[0] + 100,
        "hard stop must remain abrupt"
    );
    assert!(before_stop[2] + 100 < after_stop[2]);
    assert_channel(after_stop, 0, 85, 105); // 반투명 파랑과 빨강/파랑 배경의 합성
    assert_channel(*image.get_pixel(30, 60), 0, 240, 255);
    assert_channel(*image.get_pixel(12, 60), 2, 180, 230);
    assert_channel(*image.get_pixel(65, 60), 0, 200, 230);
    assert_channel(*image.get_pixel(95, 60), 2, 210, 240);
}

#[test]
fn renders_complete_svg_image_larger_than_a_raw_fragment() {
    // WMF/EPS의 완결 그림은 내장 래스터 때문에 조각보다 크다.
    // 큰 입력도 실제 그림을 그려야 하며 회색 decode fallback이면 실패한다.
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8\" height=\"8\"><!--{}--><rect width=\"8\" height=\"8\" fill=\"#ff0000\"/></svg>",
        " ".repeat(4 * 1024 * 1024),
    );
    let tree = PageLayerTree::new(
        8.0,
        8.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 8.0, 8.0),
            None,
            vec![PaintOp::image(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                ImageNode::new(1, Some(svg.into_bytes())),
                None,
            )],
        ),
    );
    let output = SkiaLayerRenderer::new()
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .expect("render complete SVG image");
    let image = decode_rgba(&output.bytes);
    let pixel = *image.get_pixel(4, 4);
    assert_channel(pixel, 0, 250, 255);
    assert_channel(pixel, 1, 0, 5);
    assert_channel(pixel, 2, 0, 5);
    assert_eq!(pixel[3], 255);
}

#[test]
fn text_glyph_fit_uses_layout_advance_without_distribution_spacing() {
    let render = |advance: f64, distribution: f64, letter_spacing: f64| {
        let bounds = BoundingBox::new(8.0, 0.0, 56.0, 56.0);
        let run = TextRunNode {
            text: "m".into(),
            style: TextStyle {
                font_family: "Noto Sans KR".into(),
                font_size: 32.0,
                extra_char_spacing: distribution,
                letter_spacing,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 40.0,
            field_marker: Default::default(),
            layout_positions: Some(vec![0.0, advance]),
            display_text: None,
        };
        let tree = PageLayerTree::new(
            72.0,
            56.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 72.0, 56.0),
                None,
                vec![PaintOp::text_run(bounds, run)],
            ),
        );
        let out = SkiaLayerRenderer::new()
            .with_font_paths(&[
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ttfs/opensource")
            ])
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .unwrap();
        decode_rgba(&out.bytes)
    };
    let compact = render(8.0, 0.0, 0.0);
    let distributed = render(20.0, 12.0, 0.0);
    assert_eq!(
        compact, distributed,
        "배분 간격은 다음 글리프 원점을 옮기며 윤곽을 늘리지 않는다"
    );
    let ink_width = |image: &image::RgbaImage| {
        let columns: Vec<_> = (0..image.width())
            .filter(|x| (0..image.height()).any(|y| image.get_pixel(*x, y)[3] > 32))
            .collect();
        columns.last().unwrap() - columns.first().unwrap() + 1
    };
    assert!(
        ink_width(&compact) <= 9,
        "합의한 8px advance 안의 글리프가 뒤 글자를 침범하지 않는다: {}",
        ink_width(&compact)
    );
    let negative = render(8.0, 0.0, -2.0);
    assert!(
        ink_width(&negative) > ink_width(&compact) * 2,
        "음수 자간은 후속 원점의 겹침을 허용하며 글리프 자체를 압축하지 않는다"
    );
}

#[test]
fn renders_arc_path_segments_as_ink() {
    let path = PathNode::new(
        vec![
            PathCommand::MoveTo(4.0, 12.0),
            PathCommand::ArcTo(8.0, 8.0, 0.0, false, true, 20.0, 12.0),
        ],
        ShapeStyle {
            stroke_color: Some(0x000000ff),
            stroke_width: 2.0,
            ..Default::default()
        },
        None,
    );
    let tree = PageLayerTree::new(
        24.0,
        18.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 24.0, 18.0),
            None,
            vec![PaintOp::path(BoundingBox::new(4.0, 4.0, 16.0, 12.0), path)],
        ),
    );
    let output = SkiaLayerRenderer::new()
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .expect("render arc path");
    let image = decode_rgba(&output.bytes);

    assert!(count_ink(&image) > 8);
}

#[test]
fn renders_raw_svg_fragment_as_colored_ink() {
    let tree = PageLayerTree::new(
        32.0,
        24.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 32.0, 24.0),
            None,
            vec![PaintOp::raw_svg(
                BoundingBox::new(4.0, 4.0, 18.0, 12.0),
                RawSvgNode::new(
                    "<rect x=\"0\" y=\"0\" width=\"18\" height=\"12\" fill=\"#00ff00\"/>"
                        .to_string(),
                ),
            )],
        ),
    );
    let output = SkiaLayerRenderer::new()
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .expect("render raw svg");
    let image = decode_rgba(&output.bytes);
    let green_ink = image
        .pixels()
        .filter(|pixel| pixel[0] < 48 && pixel[1] > 180 && pixel[2] < 48 && pixel[3] > 0)
        .count();

    assert!(
        green_ink > 100,
        "raw SVG fragment should render as green ink"
    );
}
