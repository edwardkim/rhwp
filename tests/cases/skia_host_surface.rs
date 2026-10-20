//! Host-owned Skia surfaces preserve compositor state and explicit error policy.

#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::paint::{
    ClipKind, LayerNode, PageLayerTree, PaintOp, TextSourceEntry, TextSourceId, TextSourceRange,
};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::render_tree::{BoundingBox, ImageNode, RectangleNode};
use rhwp::renderer::skia::{SkiaLayerRenderer, SkiaSurfaceRenderOptions};
use rhwp::renderer::ShapeStyle;
use skia_safe::{surfaces, AlphaType, Color, ColorType, ImageInfo, Matrix, Surface};

fn rect(bounds: BoundingBox, color: u32) -> PaintOp {
    PaintOp::rectangle(
        bounds,
        RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(color),
                ..Default::default()
            },
            None,
        ),
    )
}

fn tree(ops: Vec<PaintOp>) -> PageLayerTree {
    PageLayerTree::new(
        24.0,
        18.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 24.0, 18.0), None, ops),
    )
}

fn surface() -> Surface {
    let mut surface = surfaces::raster_n32_premul((64, 64)).unwrap();
    surface.canvas().clear(Color::MAGENTA);
    surface
}

fn pixels(surface: &mut Surface) -> Vec<u8> {
    let info = ImageInfo::new(
        (surface.width(), surface.height()),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut bytes = vec![0; info.min_row_bytes() * surface.height() as usize];
    assert!(surface.read_pixels(&info, &mut bytes, info.min_row_bytes(), (0, 0)));
    bytes
}

fn pixel(surface: &mut Surface, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * surface.width() as usize + x) * 4;
    pixels(surface)[offset..offset + 4].try_into().unwrap()
}

#[test]
fn host_canvas_composes_origin_scale_and_inherited_clip_without_leaking_state() {
    let mut target = surface();
    let canvas = target.canvas();
    canvas.save();
    canvas.translate((5.0, 7.0));
    canvas.scale((2.0, 2.0));
    canvas.clip_rect(
        skia_safe::Rect::from_xywh(2.0, 3.0, 10.0, 10.0),
        None,
        false,
    );
    let before = (
        canvas.save_count(),
        canvas.local_to_device(),
        canvas.device_clip_bounds(),
    );
    let tree = tree(vec![rect(BoundingBox::new(2.0, 3.0, 4.0, 4.0), 0x0000ff)]);
    SkiaLayerRenderer::new()
        .render_to_canvas_strict(
            canvas,
            &tree,
            SkiaSurfaceRenderOptions {
                scale: 1.5,
                origin_x: 3.0,
                origin_y: 4.0,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        before,
        (
            canvas.save_count(),
            canvas.local_to_device(),
            canvas.device_clip_bounds(),
        )
    );
    assert_eq!(pixel(&mut target, 20, 27), [255, 0, 0, 255]);
    assert_eq!(pixel(&mut target, 20, 34), [255, 0, 255, 255]);
    assert_eq!(pixel(&mut target, 14, 20), [255, 0, 255, 255]);
}

#[test]
fn host_surface_background_and_page_clip_preserve_surrounding_compositor_pixels() {
    let mut target = surface();
    // This paint intentionally exceeds the page; internal tree clipping is off.
    let mut tree = tree(vec![rect(
        BoundingBox::new(20.0, 14.0, 30.0, 30.0),
        0x0000ff,
    )]);
    tree.output_options.clip_enabled = false;
    SkiaLayerRenderer::new()
        .render_to_surface_strict(
            &mut target,
            &tree,
            SkiaSurfaceRenderOptions {
                origin_x: 5.0,
                origin_y: 6.0,
                background_color: Some(0x00ff00),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(pixel(&mut target, 7, 8), [0, 255, 0, 255]);
    assert_eq!(pixel(&mut target, 26, 21), [255, 0, 0, 255]);
    assert_eq!(pixel(&mut target, 30, 21), [255, 0, 255, 255]);
    assert_eq!(pixel(&mut target, 26, 25), [255, 0, 255, 255]);
    assert_eq!(pixel(&mut target, 4, 8), [255, 0, 255, 255]);
}

#[test]
fn host_surface_default_preserves_background_and_explicit_background_respects_host_clip() {
    let mut target = surface();
    let renderer = SkiaLayerRenderer::new();
    renderer
        .render_to_surface_strict(&mut target, &tree(vec![]), Default::default())
        .unwrap();
    assert_eq!(pixel(&mut target, 4, 4), [255, 0, 255, 255]);
    target
        .canvas()
        .clip_rect(skia_safe::Rect::from_xywh(7.0, 7.0, 6.0, 6.0), None, false);
    renderer
        .render_to_surface_strict(
            &mut target,
            &tree(vec![]),
            SkiaSurfaceRenderOptions {
                origin_x: 5.0,
                origin_y: 5.0,
                background_color: Some(0x0000ff),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(pixel(&mut target, 8, 8), [255, 0, 0, 255]);
    assert_eq!(pixel(&mut target, 6, 8), [255, 0, 255, 255]);
    assert_eq!(pixel(&mut target, 14, 8), [255, 0, 255, 255]);
}

#[test]
fn host_surface_replays_the_cpu_raster_paint_at_fractional_scales() {
    let tree = tree(vec![
        rect(BoundingBox::new(2.0, 3.0, 7.0, 5.0), 0x0000ff),
        rect(BoundingBox::new(7.0, 6.0, 8.0, 4.0), 0xff0000),
    ]);
    let renderer = SkiaLayerRenderer::new();
    for scale in [0.5, 1.0, 1.25, 1.5, 2.0] {
        let expected = renderer
            .render_pixels_with_options(
                &tree,
                RasterRenderOptions {
                    scale,
                    ..Default::default()
                },
            )
            .unwrap();
        let mut target = surfaces::raster_n32_premul((expected.width, expected.height)).unwrap();
        target.canvas().clear(Color::TRANSPARENT);
        renderer
            .render_to_surface_strict(
                &mut target,
                &tree,
                SkiaSurfaceRenderOptions {
                    scale,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(pixels(&mut target), expected.bytes, "scale {scale}");
    }
}

#[test]
fn host_surface_rejects_invalid_arguments_without_touching_target_pixels_or_state() {
    let renderer = SkiaLayerRenderer::new();
    let tree = tree(vec![]);
    let mut target = surface();
    let before = pixels(&mut target);
    let options = SkiaSurfaceRenderOptions {
        background_color: Some(0x0000ff),
        ..Default::default()
    };
    for invalid in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        assert!(renderer
            .render_to_surface_strict(
                &mut target,
                &tree,
                SkiaSurfaceRenderOptions {
                    scale: invalid,
                    ..options
                }
            )
            .is_err());
    }
    for invalid in [f64::NAN, f64::INFINITY, f64::MAX] {
        for options in [
            SkiaSurfaceRenderOptions {
                origin_x: invalid,
                ..options
            },
            SkiaSurfaceRenderOptions {
                origin_y: invalid,
                ..options
            },
        ] {
            assert!(renderer
                .render_to_surface_strict(&mut target, &tree, options)
                .is_err());
        }
    }
    for invalid in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        for axis in [0, 1] {
            let mut invalid_tree = tree.clone();
            if axis == 0 {
                invalid_tree.page_width = invalid;
            } else {
                invalid_tree.page_height = invalid;
            }
            assert!(renderer
                .render_to_surface_strict(&mut target, &invalid_tree, options)
                .is_err());
        }
    }
    assert_eq!(pixels(&mut target), before);
    assert_eq!(target.canvas().save_count(), 1);
    assert_eq!(
        target.canvas().local_to_device_as_3x3(),
        Matrix::new_identity()
    );
}

#[test]
fn host_surface_rejects_unusable_inherited_transforms_before_clearing() {
    let renderer = SkiaLayerRenderer::new();
    for matrix in [
        Matrix::scale((0.0, 1.0)),
        Matrix::scale((f32::INFINITY, 1.0)),
        Matrix::new_all(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.01, 0.0, 1.0),
    ] {
        let mut target = surface();
        let before = pixels(&mut target);
        target.canvas().concat(&matrix);
        let inherited = target.canvas().local_to_device_as_3x3();
        let count = target.canvas().save_count();
        let clip = target.canvas().device_clip_bounds();
        assert!(renderer
            .render_to_surface_strict(
                &mut target,
                &tree(vec![]),
                SkiaSurfaceRenderOptions {
                    background_color: Some(0x0000ff),
                    ..Default::default()
                },
            )
            .is_err());
        assert_eq!(pixels(&mut target), before);
        assert_eq!(target.canvas().save_count(), count);
        assert_eq!(target.canvas().device_clip_bounds(), clip);
        // Matrix equality is not reflexive for a NaN matrix; compare entries.
        let after = target.canvas().local_to_device_as_3x3();
        let mut before_values = [0.0; 9];
        let mut after_values = [0.0; 9];
        inherited.get_9(&mut before_values);
        after.get_9(&mut after_values);
        for (before, after) in before_values.iter().zip(&after_values) {
            assert!(before == after || before.is_nan() && after.is_nan());
        }
    }
}

#[test]
fn host_surface_rejects_an_invalid_text_contract_before_clearing() {
    let mut invalid = tree(vec![]);
    invalid.text_sources.entries.push(TextSourceEntry {
        id: TextSourceId(0),
        stable_source_key: None,
        text: "orphan".into(),
        utf8_range: TextSourceRange::new(0, 6),
        utf16_range: TextSourceRange::new(0, 6),
        annotations: vec![],
    });
    let mut target = surface();
    let before = pixels(&mut target);
    let error = SkiaLayerRenderer::new()
        .render_to_surface_strict(
            &mut target,
            &invalid,
            SkiaSurfaceRenderOptions {
                background_color: Some(0x0000ff),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("invalid PageLayerTree text contract"));
    assert_eq!(pixels(&mut target), before);
    assert_eq!(target.canvas().save_count(), 1);
}

#[test]
fn host_surface_resource_error_restores_nested_state_but_retains_partial_paint() {
    let renderer = SkiaLayerRenderer::new();
    let bounds = BoundingBox::new(0.0, 0.0, 24.0, 18.0);
    let mut failure_tree = tree(vec![
        rect(BoundingBox::new(2.0, 2.0, 4.0, 4.0), 0x0000ff),
        PaintOp::image(
            BoundingBox::new(10.0, 2.0, 4.0, 4.0),
            ImageNode::new(41, None),
            None,
        ),
    ]);
    failure_tree.root =
        LayerNode::clip_rect(bounds, None, bounds, failure_tree.root, ClipKind::Generic);
    // The existing CPU API retains its deliberate placeholder policy.
    renderer
        .render_pixels_with_options(&failure_tree, RasterRenderOptions::default())
        .unwrap();
    let mut target = surface();
    target.canvas().save();
    target.canvas().translate((3.0, 4.0));
    let before = (
        target.canvas().save_count(),
        target.canvas().local_to_device(),
        target.canvas().device_clip_bounds(),
    );
    let error = renderer
        .render_to_surface_strict(&mut target, &failure_tree, Default::default())
        .unwrap_err();
    assert!(
        error.to_string().contains("image data is missing"),
        "{error}"
    );
    assert_eq!(
        before,
        (
            target.canvas().save_count(),
            target.canvas().local_to_device(),
            target.canvas().device_clip_bounds()
        )
    );
    assert_eq!(pixel(&mut target, 6, 7), [255, 0, 0, 255]);
    assert_eq!(pixel(&mut target, 14, 7), [255, 0, 255, 255]);
    // A failure does not poison either renderer or borrowed canvas.
    renderer
        .render_to_surface_strict(&mut target, &tree(vec![]), Default::default())
        .unwrap();
}

#[test]
fn host_surface_svg_image_keeps_device_resolution_after_a_quarter_turn() {
    let bounds = BoundingBox::new(0.0, 0.0, 16.0, 16.0);
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><path d="M1 2L15 13" stroke="#163f9a" stroke-width="0.75"/><circle cx="9.25" cy="5.5" r="2.25" fill="#b52139"/></svg>"##;
    let page = PageLayerTree::new(
        16.0,
        16.0,
        LayerNode::leaf(
            bounds,
            None,
            vec![PaintOp::image(
                bounds,
                ImageNode::new(1, Some(svg.to_vec())),
                None,
            )],
        ),
    );
    let renderer = SkiaLayerRenderer::new();
    let mut unrotated = surface();
    unrotated.canvas().scale((4.0, 4.0));
    renderer
        .render_to_surface_strict(&mut unrotated, &page, Default::default())
        .unwrap();
    let reference = pixels(&mut unrotated);
    let mut rotated = surface();
    rotated.canvas().concat(&Matrix::new_all(
        0.0, -4.0, 64.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ));
    renderer
        .render_to_surface_strict(&mut rotated, &page, Default::default())
        .unwrap();
    let actual = pixels(&mut rotated);
    let mut differences = 0;
    for y in 0..64 {
        for x in 0..64 {
            let destination = (y * 64 + x) * 4;
            let source = ((63 - x) * 64 + y) * 4;
            differences +=
                usize::from(actual[destination..destination + 4] != reference[source..source + 4]);
        }
    }
    assert_eq!(
        differences, 0,
        "rotating a host target must preserve the SVG's raster resolution"
    );
}
