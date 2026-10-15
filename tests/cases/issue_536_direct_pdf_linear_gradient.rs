//! Direct PDF accepts bounded linear shaders without raster or solid-color substitution.

#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::pdf::layer_trees_to_pdf;
use rhwp::renderer::render_tree::{
    BoundingBox, EllipseNode, PageBackgroundNode, PathNode, RectangleNode,
};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::{GradientFillInfo, PathCommand, PatternFillInfo, ShapeStyle};

fn gradient(angle: i16) -> GradientFillInfo {
    GradientFillInfo {
        gradient_type: 1,
        angle,
        center_x: 50,
        center_y: 50,
        colors: vec![0x0000_00ff, 0x00ff_0000],
        positions: vec![0.0, 1.0],
    }
}

fn tree(kind: &str, gradient: GradientFillInfo, bounds: BoundingBox) -> PageLayerTree {
    let gradient = Some(Box::new(gradient));
    let op = match kind {
        "background" => PaintOp::page_background(
            bounds,
            PageBackgroundNode {
                background_color: None,
                border_color: None,
                border_width: 0.0,
                gradient,
                image: None,
            },
        ),
        "rectangle" => PaintOp::rectangle(
            bounds,
            RectangleNode::new(0.0, ShapeStyle::default(), gradient),
        ),
        "ellipse" => PaintOp::ellipse(bounds, EllipseNode::new(ShapeStyle::default(), gradient)),
        "path" => PaintOp::path(
            bounds,
            PathNode::new(
                vec![
                    PathCommand::MoveTo(bounds.x, bounds.y),
                    PathCommand::LineTo(bounds.x + bounds.width, bounds.y),
                    PathCommand::LineTo(bounds.x, bounds.y + bounds.height),
                    PathCommand::ClosePath,
                ],
                ShapeStyle::default(),
                gradient,
            ),
        ),
        _ => unreachable!("known vector primitive"),
    };
    PageLayerTree::with_profile(
        96.0,
        64.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 96.0, 64.0), None, vec![op]),
        RenderProfile::Print,
    )
}

fn bounds() -> BoundingBox {
    BoundingBox::new(8.0, 8.0, 80.0, 24.0)
}

fn assert_axial_vector_pdf(tree: PageLayerTree) {
    let pdf = layer_trees_to_pdf(&[tree]).expect("bounded linear gradient should export");
    assert!(pdf.starts_with(b"%PDF-"));
    let dictionaries = String::from_utf8_lossy(&pdf);
    // A PDF axial shading is independently identifiable by its standard type.
    // A solid fallback or a page image would not satisfy this contract.
    assert!(dictionaries.contains("/ShadingType 2"));
    assert!(!dictionaries.contains("/Subtype /Image"));
    assert!(dictionaries.contains("/MediaBox [0 0 72 48]"));
}

#[test]
fn direct_pdf_records_linear_gradient_shapes_as_axial_vectors() {
    for kind in ["background", "rectangle", "ellipse", "path"] {
        for angle in [0, 45, 90, 135, -270, i16::MAX] {
            assert_axial_vector_pdf(tree(kind, gradient(angle), bounds()));
        }
    }
}

#[test]
fn direct_pdf_preserves_even_spacing_hard_stops_and_the_color_limit() {
    let mut evenly_spaced = gradient(90);
    evenly_spaced.positions.clear();
    assert_axial_vector_pdf(tree("rectangle", evenly_spaced, bounds()));

    let mut hard_stop = gradient(45);
    hard_stop.colors = vec![0x0000_00ff, 0x0000_00ff, 0x00ff_0000, 0x00ff_0000];
    hard_stop.positions = vec![0.0, 0.5, 0.5, 1.0];
    assert_axial_vector_pdf(tree("rectangle", hard_stop, bounds()));

    let mut maximum = gradient(90);
    maximum.colors = (0..4096)
        .map(|index| if index % 2 == 0 { 0xff } else { 0xff0000 })
        .collect();
    maximum.positions.clear();
    assert_axial_vector_pdf(tree("rectangle", maximum, bounds()));
}

#[test]
fn direct_pdf_rejects_invalid_stops_instead_of_normalizing_them() {
    let valid = tree("rectangle", gradient(90), bounds());
    for positions in [
        vec![0.0],
        vec![0.0, 0.5, 1.0],
        vec![-0.1, 1.0],
        vec![0.0, 1.1],
        vec![1.0, 0.0],
        vec![f64::NAN, 1.0],
        vec![0.0, f64::INFINITY],
    ] {
        let mut invalid = gradient(90);
        invalid.positions = positions;
        let error = layer_trees_to_pdf(&[valid.clone(), tree("rectangle", invalid, bounds())])
            .expect_err("the entire selected document must fail preflight");
        assert!(error.contains("direct PDF page 2"));
        assert!(error.contains("invalid gradient stop positions"));
        assert!(error.contains("use the svg backend"));
    }
    for count in [0, 1, 4097] {
        let mut invalid = gradient(90);
        invalid.colors.resize(count, 0xff);
        invalid.positions.clear();
        let error = layer_trees_to_pdf(&[tree("rectangle", invalid, bounds())]).unwrap_err();
        assert!(error.contains("gradient color count outside 2..=4096"));
    }
}

#[test]
fn direct_pdf_rejects_collapsed_edges_and_nonportable_gradient_axes() {
    let limit = f64::from(f32::MAX);
    for invalid_bounds in [
        BoundingBox::new(8.0, 8.0, 0.0, 24.0),
        BoundingBox::new(8.0, 8.0, 80.0, 0.0),
        BoundingBox::new(1e10, 0.0, 1.0, 24.0),
        BoundingBox::new(limit, 0.0, limit, 24.0),
    ] {
        let error = layer_trees_to_pdf(&[tree("rectangle", gradient(90), invalid_bounds)])
            .expect_err("the actual f32 shader rectangle must be finite and nonempty");
        assert!(error.contains("invalid gradient replay bounds"));
    }
    for unstable_bounds in [
        BoundingBox::new(0.0, 0.0, 0.5, 0.5),
        BoundingBox::new(0.0, 0.0, 1e30, 1e30),
    ] {
        let error = layer_trees_to_pdf(&[tree("rectangle", gradient(90), unstable_bounds)])
            .expect_err("the bounded vector axis must have stable f32 length");
        assert!(error.contains("invalid gradient axis"));
    }
    assert_axial_vector_pdf(tree(
        "rectangle",
        gradient(90),
        BoundingBox::new(0.0, 0.0, 1.0, 1.0),
    ));
    // All four rectangle components and its edges fit f32, but the projected
    // oblique axis extends beyond the rectangle and exceeds the scalar range.
    let error = layer_trees_to_pdf(&[tree(
        "rectangle",
        gradient(45),
        BoundingBox::new(limit * 0.9, 0.0, limit * 0.05, limit * 0.8),
    )])
    .unwrap_err();
    assert!(error.contains("invalid gradient axis"));
}

#[test]
fn direct_pdf_keeps_non_linear_and_pattern_fills_closed() {
    for kind in [0, 2, 3, 4, 99] {
        let mut unsupported = gradient(90);
        unsupported.gradient_type = kind;
        let error = layer_trees_to_pdf(&[tree("rectangle", unsupported, bounds())]).unwrap_err();
        assert!(error.contains("non-linear gradient fill"));
    }
    let mut unsupported = tree("rectangle", gradient(90), bounds());
    let LayerNodeKind::Leaf { ops } = &mut unsupported.root.kind else {
        unreachable!("leaf fixture")
    };
    let PaintOp::Rectangle { rect, .. } = &mut ops[0] else {
        unreachable!("rectangle fixture")
    };
    rect.style.pattern = Some(PatternFillInfo {
        tile_size: 7.68,
        pattern_type: 0,
        pattern_color: 0,
        background_color: 0xffffff,
    });
    let error = layer_trees_to_pdf(&[unsupported]).unwrap_err();
    assert!(error.contains("pattern fill"));
}

#[test]
fn native_linear_axis_preserves_direction_and_non_square_isolines() {
    let renderer = SkiaLayerRenderer::new();
    // The 0/50/100% Hancom Print controls recorded in PR #7661 establish
    // that the first color is at the source center. An endpoint center
    // therefore isolates axis direction from the default two-sided ramp.
    let mut upward = gradient(0);
    upward.center_y = 100;
    let vertical = renderer
        .render_pixels_with_options(
            &tree("rectangle", upward, bounds()),
            RasterRenderOptions::default(),
        )
        .unwrap();
    let channel =
        |pixels: &rhwp::renderer::raster_pixels::RasterPixels,
         x: usize,
         y: usize,
         channel: usize| { pixels.bytes[y * pixels.stride + 4 * x + channel] };
    // In y-down coordinates angle 0 points upward: the second (blue) stop
    // must dominate above the center and the first (red) stop below it.
    assert!(channel(&vertical, 48, 12, 2) > channel(&vertical, 48, 28, 2));
    assert!(channel(&vertical, 48, 12, 0) < channel(&vertical, 48, 28, 0));

    let mut upward_right = gradient(45);
    upward_right.center_x = 0;
    upward_right.center_y = 100;
    let diagonal = renderer
        .render_pixels_with_options(
            &tree("rectangle", upward_right, bounds()),
            RasterRenderOptions::default(),
        )
        .unwrap();
    // At 45 degrees the gradient varies with x-y, independently of the box's
    // aspect ratio. Moving (+8,+8) remains on the same color isoline.
    // Use a one-sided ramp: a centered ramp would also give equal colors
    // to these reflected points even if the axis were scaled by box size.
    for component in [0, 2] {
        assert!(
            channel(&diagonal, 44, 16, component).abs_diff(channel(&diagonal, 52, 24, component))
                <= 2
        );
    }
}

#[test]
fn native_centered_linear_gradient_preserves_two_sided_color_ramps() {
    let pixels = SkiaLayerRenderer::new()
        .render_pixels_with_options(
            &tree("rectangle", gradient(0), bounds()),
            RasterRenderOptions::default(),
        )
        .unwrap();
    let pixel = |y: usize| {
        let offset = y * pixels.stride + 4 * 48;
        &pixels.bytes[offset..offset + 4]
    };
    // The default 50% center is red, with blue toward both endpoints.
    // Pixel centers 12.5 and 27.5 are equally far from center y=20.
    let (above, below, center) = (pixel(12), pixel(27), pixel(19));
    for channel in 0..4 {
        assert!(above[channel].abs_diff(below[channel]) <= 2);
    }
    for side in [above, below] {
        assert!(side[2] > side[0], "blue must dominate on both sides");
        assert!(center[0].saturating_sub(side[0]) > 100);
        assert!(side[2].saturating_sub(center[2]) > 100);
    }
}
