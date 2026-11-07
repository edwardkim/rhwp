//! 관계형·고정형: 공개 Native raster 경로의 기존 단위 검사를 integration 경로로 이관한다.
//! 주석 갱신: 2026-11-07 13:59 +0900. 직전 파일 변경 1ab72b32062a5d694d1ad4e981434eaed365e731
//! (2026-11-05T13:04:12+09:00). 원 기호의 합성 paint 반례를 같은 공개 raster 경로에 추가한다.
//! 주석 갱신: 2026-11-05 11:40 (+0900). 직전 파일 변경 6097253468 (2026-10-24 15:11:35 +0900).
//! 기존 검사는 합성 paint 트리와 저장소 fixture 글꼴만 쓰며 실물 PDF 시각 측정을 대신하지 않는다.
//! 원 기호 추가 근거: 2026-11-07 ef77d740e1의 sungeo.hwp ↔ sungeo-hwp-2020.pdf 전94쪽.
//! 96 DPI·Print·보정 전2px raw 최저 Native99.98%/fresh WASM99.99%, 5쪽100.00%/99.99%.
//! 원본은 samples/basic/, PDF는 pdf/basic/에 있으며 양쪽94쪽 일치·같은5쪽 확대 판독을 마쳤다.
//! SHA·TSV·review·overlay: mydocs/pr/assets/planet6897_20261106/pr8611/r33-source-symbol/.
//! 추가 반례는 독립 PDF Wingdings glyph3의 em 윤곽을 검사하며 페이지 절대 배치 px 핀이 아니다.
//! 고정형은 합성 캔버스의 픽셀 색·잉크 폭 상한이고, 관계형은 같은 렌더러의 다른 입력끼리의 잉크 폭 비교다.
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
fn implicit_source_marker_ink_is_independent_of_display_face_and_scales_in_em() {
    // 독립 Print glyph3: UPEM2048, bbox(173,444)..(765,1036).
    // 원본의 em 계약을 서로 다른 합성 크기·장평·face에서 실제 raster로 소비한다.
    for (size, ratio) in [(32.0, 1.0), (64.0, 0.75)] {
        let mut reference = None;
        for family in ["RHWP Missing Marker Face", "RHWP Host Fixture"] {
            let bounds = BoundingBox::new(12.0, 4.0, size, 80.0);
            let run = TextRunNode {
                text: "\u{f09f}".into(),
                style: TextStyle {
                    font_family: family.into(),
                    font_size: size,
                    ratio,
                    color: 0x00800000,
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
                baseline: 64.0,
                field_marker: Default::default(),
                layout_positions: Some(vec![0.0, size * ratio * 937.0 / 2048.0]),
                display_text: Some("•".into()),
            };
            let tree = PageLayerTree::new(
                100.0,
                90.0,
                LayerNode::leaf(bounds, None, vec![PaintOp::text_run(bounds, run)]),
            );
            let out = SkiaLayerRenderer::new()
                .try_with_font_data(&[include_bytes!(
                    "../fixtures/fonts/RHWPHostFixture-Regular.ttf"
                )
                .to_vec()])
                .unwrap()
                .render_raster_with_options(&tree, RasterRenderOptions::default())
                .unwrap();
            let image = decode_rgba(&out.bytes);
            if let Some(before) = reference.as_ref() {
                assert_eq!(
                    before, &image,
                    "표시 face가 원 기호의 잉크를 덮어쓰면 안 된다"
                );
            }
            let ink: Vec<_> = image
                .enumerate_pixels()
                .filter(|(_, _, p)| p[0] < 24 && p[1] < 24 && p[2] > 80 && p[3] > 200)
                .map(|(x, y, _)| (f64::from(x) + 0.5, f64::from(y) + 0.5))
                .collect();
            assert!(
                !ink.is_empty(),
                "명령만 생성하고 실제 잉크가 없으면 실패한다"
            );
            let cx = bounds.x + size * ratio * 469.0 / 2048.0;
            let cy = bounds.y + 64.0 - size * 740.0 / 2048.0;
            let rx = size * ratio * 296.0 / 2048.0;
            let ry = size * 296.0 / 2048.0;
            assert!(ink
                .iter()
                .all(|(x, y)| (x - cx).abs() <= rx + 1.0 && (y - cy).abs() <= ry + 1.0));
            let min_x = ink.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
            let max_x = ink
                .iter()
                .map(|(x, _)| *x)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_y = ink.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
            let max_y = ink
                .iter()
                .map(|(_, y)| *y)
                .fold(f64::NEG_INFINITY, f64::max);
            assert!((max_x - min_x + 1.0 - 2.0 * rx).abs() <= 2.0);
            assert!((max_y - min_y + 1.0 - 2.0 * ry).abs() <= 2.0);
            reference = Some(image);
        }
    }
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
            text: "A".into(),
            style: TextStyle {
                font_family: "RHWP Host Fixture".into(),
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
            // The bundled fixture's A has a 900/1000em advance. A much
            // narrower source advance must control its painted width, without
            // relying on a contributor's installed system fallback face.
            .try_with_font_data(&[
                include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf").to_vec(),
            ])
            .unwrap()
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
    // 넘치는 잉크만 칸 폭에 맞추는 #8319 계약에서 compact 는 잉크를 8px 로 누른다.
    // 음수 자간은 그 맞춤에서 빠지므로, 같은 원점 위상(advance 30.8 → 가운데 정렬
    // 이동 1px)에서 칸을 넘지 않는 자연 폭 렌더와 잉크 폭이 같아야 한다(#8613).
    let negative = render(8.0, 0.0, -2.0);
    let natural = render(30.8, 0.0, 0.0);
    assert_eq!(
        ink_width(&negative),
        ink_width(&natural),
        "음수 자간은 후속 원점의 겹침을 허용하며 글리프 자체를 압축하지 않는다"
    );
    assert!(
        ink_width(&negative) > ink_width(&compact),
        "음수 자간 글리프는 칸에 맞춘 compact 글리프보다 넓다: {} <= {}",
        ink_width(&negative),
        ink_width(&compact)
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
