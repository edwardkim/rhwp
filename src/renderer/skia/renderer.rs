use skia_safe::{
    paint, surfaces, Canvas, Color, Data, EncodedImageFormat, FilterMode, Font, Image, MipmapMode,
    Paint, PathBuilder, PathEffect, Rect, SamplingOptions,
};

use crate::error::HwpError;
use crate::model::ColorRef;
use crate::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp};
use crate::renderer::layer_renderer::{
    LayerRasterRenderer, LayerRenderResult, RasterOutputFormat, RasterRenderOptions,
    RasterRenderOutput,
};
use crate::renderer::{svg_arc_to_beziers, LineStyle, PathCommand, ShapeStyle, StrokeDash};

pub struct SkiaLayerRenderer;

impl SkiaLayerRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_raster_with_options(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        if let Some(dpi) = options.dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                return Err(HwpError::RenderError(format!("invalid raster dpi: {dpi}")));
            }
        }
        if options.format != RasterOutputFormat::Png {
            return Err(HwpError::RenderError(
                "Skia raster renderer currently supports PNG output".to_string(),
            ));
        }

        let raster_dimension = |value: f64, label: &str| -> LayerRenderResult<i32> {
            if !value.is_finite() || value <= 0.0 {
                return Err(HwpError::RenderError(format!(
                    "invalid page {label}: {value}"
                )));
            }
            if !options.scale.is_finite() || options.scale <= 0.0 {
                return Err(HwpError::RenderError(format!(
                    "invalid raster scale: {}",
                    options.scale
                )));
            }
            if options.max_dimension <= 0 {
                return Err(HwpError::RenderError(format!(
                    "invalid raster max dimension: {}",
                    options.max_dimension
                )));
            }
            let scaled = (value * options.scale).ceil();
            if !scaled.is_finite() || scaled <= 0.0 || scaled > options.max_dimension as f64 {
                return Err(HwpError::RenderError(format!(
                    "raster {label} out of range: {scaled}"
                )));
            }
            Ok(scaled as i32)
        };
        let width = raster_dimension(tree.page_width, "width")?;
        let height = raster_dimension(tree.page_height, "height")?;

        let mut surface = surfaces::raster_n32_premul((width, height))
            .ok_or_else(|| HwpError::RenderError("Skia raster surface 생성 실패".to_string()))?;
        let canvas = surface.canvas();
        let clear_color = if let Some(color) = options.background_color {
            colorref_to_skia(color, 1.0)
        } else if options.transparent {
            Color::from_argb(0, 0, 0, 0)
        } else {
            Color::WHITE
        };
        canvas.clear(clear_color);
        if options.scale != 1.0 {
            canvas.scale((options.scale as f32, options.scale as f32));
        }
        self.render_node(canvas, &tree.root, tree.output_options.clip_enabled);

        let image = surface.image_snapshot();
        let data = image
            .encode(None, EncodedImageFormat::PNG, None)
            .ok_or_else(|| HwpError::RenderError("Skia PNG 인코딩 실패".to_string()))?;
        Ok(RasterRenderOutput {
            bytes: data.as_bytes().to_vec(),
            format: RasterOutputFormat::Png,
            width,
            height,
            dpi: options.dpi,
            color_space: options.color_space,
        })
    }

    fn render_node(&self, canvas: &Canvas, node: &LayerNode, clip_enabled: bool) {
        let apply_dash = |paint: &mut Paint, dash: StrokeDash| {
            let base_width = paint.stroke_width().max(1.0);
            let intervals: Option<[f32; 6]> = match dash {
                StrokeDash::Solid => None,
                StrokeDash::Dash => Some([6.0, 3.0, 0.0, 0.0, 0.0, 0.0]),
                StrokeDash::Dot => Some([2.0, 2.0, 0.0, 0.0, 0.0, 0.0]),
                StrokeDash::DashDot => Some([6.0, 3.0, 2.0, 3.0, 0.0, 0.0]),
                StrokeDash::DashDotDot => Some([6.0, 3.0, 2.0, 3.0, 2.0, 3.0]),
            };
            if let Some(intervals) = intervals {
                let intervals = intervals
                    .into_iter()
                    .filter(|value| *value > 0.0)
                    .map(|value| value * base_width)
                    .collect::<Vec<_>>();
                if let Some(effect) = PathEffect::dash(&intervals, 0.0) {
                    paint.set_path_effect(effect);
                }
            }
        };
        let make_fill_paint = |style: &ShapeStyle| -> Option<Paint> {
            let color = style
                .pattern
                .map(|pattern| pattern.background_color)
                .or(style.fill_color)?;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Fill);
            paint.set_color(colorref_to_skia(color, style.opacity as f32));
            Some(paint)
        };
        let make_stroke_paint = |style: &ShapeStyle| -> Option<Paint> {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Stroke);
            paint.set_stroke_width(if style.stroke_width > 0.0 {
                style.stroke_width as f32
            } else {
                1.0
            });
            paint.set_color(colorref_to_skia(style.stroke_color?, style.opacity as f32));
            apply_dash(&mut paint, style.stroke_dash);
            Some(paint)
        };
        let make_line_paint = |style: &LineStyle| {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Stroke);
            paint.set_stroke_width(if style.width > 0.0 {
                style.width as f32
            } else {
                1.0
            });
            paint.set_color(colorref_to_skia(style.color, 1.0));
            apply_dash(&mut paint, style.dash);
            paint
        };
        let draw_placeholder = |bbox: crate::renderer::render_tree::BoundingBox, label: &str| {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return;
            }
            let rect = Rect::from_xywh(
                bbox.x as f32,
                bbox.y as f32,
                bbox.width as f32,
                bbox.height as f32,
            );
            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(Color::from_argb(48, 96, 96, 96));
            canvas.draw_rect(rect, &fill);
            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(Color::from_argb(160, 96, 96, 96));
            canvas.draw_rect(rect, &stroke);
            let mut font = Font::default();
            font.set_size(10.0);
            let mut text = Paint::default();
            text.set_anti_alias(true);
            text.set_color(Color::from_argb(220, 64, 64, 64));
            canvas.draw_str(
                label,
                (bbox.x as f32 + 4.0, (bbox.y + bbox.height / 2.0) as f32),
                &font,
                &text,
            );
        };
        let draw_image = |data: &[u8], bbox: crate::renderer::render_tree::BoundingBox| {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return;
            }
            if let Some(image) = Image::from_encoded(Data::new_copy(data)) {
                let dst = Rect::from_xywh(
                    bbox.x as f32,
                    bbox.y as f32,
                    bbox.width as f32,
                    bbox.height as f32,
                );
                let paint = Paint::default();
                canvas.draw_image_rect_with_sampling_options(
                    &image,
                    None,
                    dst,
                    SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
                    &paint,
                );
            } else {
                draw_placeholder(bbox, "image");
            }
        };
        let draw_text = |text: &str,
                         bbox: crate::renderer::render_tree::BoundingBox,
                         style: &crate::renderer::TextStyle,
                         baseline: f64,
                         rotation: f64| {
            if text.is_empty() {
                return;
            }
            let mut font = Font::default();
            font.set_size(if style.font_size > 0.0 {
                style.font_size as f32
            } else {
                12.0
            });
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(colorref_to_skia(style.color, 1.0));
            let y = if baseline > 0.0 {
                bbox.y + baseline
            } else {
                bbox.y + bbox.height
            };
            if rotation != 0.0 {
                canvas.save();
                canvas.rotate(
                    rotation as f32,
                    Some(
                        (
                            (bbox.x + bbox.width / 2.0) as f32,
                            (bbox.y + bbox.height / 2.0) as f32,
                        )
                            .into(),
                    ),
                );
                canvas.draw_str(text, (bbox.x as f32, y as f32), &font, &paint);
                canvas.restore();
            } else {
                canvas.draw_str(text, (bbox.x as f32, y as f32), &font, &paint);
            }
        };
        let open_shape_transform =
            |transform: crate::renderer::render_tree::ShapeTransform,
             bbox: &crate::renderer::render_tree::BoundingBox| {
                canvas.save();
                let cx = (bbox.x + bbox.width / 2.0) as f32;
                let cy = (bbox.y + bbox.height / 2.0) as f32;
                if transform.horz_flip {
                    canvas.translate((cx * 2.0, 0.0));
                    canvas.scale((-1.0, 1.0));
                }
                if transform.vert_flip {
                    canvas.translate((0.0, cy * 2.0));
                    canvas.scale((1.0, -1.0));
                }
                if transform.rotation != 0.0 {
                    canvas.rotate(transform.rotation as f32, Some((cx, cy).into()));
                }
            };

        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.render_node(canvas, child, clip_enabled);
                }
            }
            LayerNodeKind::ClipRect { clip, child, .. } => {
                if !clip_enabled {
                    self.render_node(canvas, child, clip_enabled);
                    return;
                }
                canvas.save();
                canvas.clip_rect(
                    Rect::from_xywh(
                        clip.x as f32,
                        clip.y as f32,
                        clip.width as f32,
                        clip.height as f32,
                    ),
                    None,
                    Some(true),
                );
                self.render_node(canvas, child, clip_enabled);
                canvas.restore();
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    match op {
                        PaintOp::PageBackground { bbox, background } => {
                            let rect = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(color) = background
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .or(background.background_color)
                            {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_style(paint::Style::Fill);
                                paint.set_color(colorref_to_skia(color, 1.0));
                                canvas.draw_rect(rect, &paint);
                            }
                            if let Some(image) = &background.image {
                                draw_image(&image.data, *bbox);
                            }
                            if let Some(color) = background.border_color {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_style(paint::Style::Stroke);
                                paint.set_stroke_width(if background.border_width > 0.0 {
                                    background.border_width as f32
                                } else {
                                    1.0
                                });
                                paint.set_color(colorref_to_skia(color, 1.0));
                                canvas.draw_rect(rect, &paint);
                            }
                        }
                        PaintOp::TextRun { bbox, run } => {
                            draw_text(&run.text, *bbox, &run.style, run.baseline, run.rotation);
                        }
                        PaintOp::FootnoteMarker { bbox, marker } => {
                            let style = crate::renderer::TextStyle {
                                font_family: marker.font_family.clone(),
                                font_size: (marker.base_font_size * 0.55).max(7.0),
                                color: marker.color,
                                ..Default::default()
                            };
                            draw_text(&marker.text, *bbox, &style, bbox.height * 0.4, 0.0);
                        }
                        PaintOp::Line { bbox, line } => {
                            if line.transform.has_transform() {
                                open_shape_transform(line.transform, bbox);
                            }
                            canvas.draw_line(
                                (line.x1 as f32, line.y1 as f32),
                                (line.x2 as f32, line.y2 as f32),
                                &make_line_paint(&line.style),
                            );
                            if line.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Rectangle { bbox, rect } => {
                            if rect.transform.has_transform() {
                                open_shape_transform(rect.transform, bbox);
                            }
                            let sk_rect = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(fill) = rect
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        rect.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&rect.style))
                            {
                                if rect.corner_radius > 0.0 {
                                    canvas.draw_round_rect(
                                        sk_rect,
                                        rect.corner_radius as f32,
                                        rect.corner_radius as f32,
                                        &fill,
                                    );
                                } else {
                                    canvas.draw_rect(sk_rect, &fill);
                                }
                            }
                            if let Some(stroke) = make_stroke_paint(&rect.style) {
                                if rect.corner_radius > 0.0 {
                                    canvas.draw_round_rect(
                                        sk_rect,
                                        rect.corner_radius as f32,
                                        rect.corner_radius as f32,
                                        &stroke,
                                    );
                                } else {
                                    canvas.draw_rect(sk_rect, &stroke);
                                }
                            }
                            if rect.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Ellipse { bbox, ellipse } => {
                            if ellipse.transform.has_transform() {
                                open_shape_transform(ellipse.transform, bbox);
                            }
                            let oval = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(fill) = ellipse
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        ellipse.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&ellipse.style))
                            {
                                canvas.draw_oval(oval, &fill);
                            }
                            if let Some(stroke) = make_stroke_paint(&ellipse.style) {
                                canvas.draw_oval(oval, &stroke);
                            }
                            if ellipse.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Path { bbox, path } => {
                            if path.transform.has_transform() {
                                open_shape_transform(path.transform, bbox);
                            }
                            let mut builder = PathBuilder::new();
                            let mut current = (0.0, 0.0);
                            for command in &path.commands {
                                match *command {
                                    PathCommand::MoveTo(x, y) => {
                                        builder.move_to((x as f32, y as f32));
                                        current = (x, y);
                                    }
                                    PathCommand::LineTo(x, y) => {
                                        builder.line_to((x as f32, y as f32));
                                        current = (x, y);
                                    }
                                    PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                                        builder.cubic_to(
                                            (x1 as f32, y1 as f32),
                                            (x2 as f32, y2 as f32),
                                            (x as f32, y as f32),
                                        );
                                        current = (x, y);
                                    }
                                    PathCommand::ArcTo(
                                        rx,
                                        ry,
                                        rotation,
                                        large_arc,
                                        sweep,
                                        x,
                                        y,
                                    ) => {
                                        for segment in svg_arc_to_beziers(
                                            current.0, current.1, rx, ry, rotation, large_arc,
                                            sweep, x, y,
                                        ) {
                                            if let PathCommand::CurveTo(x1, y1, x2, y2, ex, ey) =
                                                segment
                                            {
                                                builder.cubic_to(
                                                    (x1 as f32, y1 as f32),
                                                    (x2 as f32, y2 as f32),
                                                    (ex as f32, ey as f32),
                                                );
                                                current = (ex, ey);
                                            }
                                        }
                                    }
                                    PathCommand::ClosePath => {
                                        builder.close();
                                    }
                                }
                            }
                            let sk_path = builder.detach();
                            if let Some(fill) = path
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        path.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&path.style))
                            {
                                canvas.draw_path(&sk_path, &fill);
                            }
                            if let Some(stroke) = make_stroke_paint(&path.style) {
                                canvas.draw_path(&sk_path, &stroke);
                            }
                            if path.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Image { bbox, image } => {
                            if image.transform.has_transform() {
                                open_shape_transform(image.transform, bbox);
                            }
                            if let Some(data) = image.data.as_deref() {
                                draw_image(data, *bbox);
                            } else {
                                draw_placeholder(*bbox, "image");
                            }
                            if image.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Equation { bbox, .. } => draw_placeholder(*bbox, "equation"),
                        PaintOp::FormObject { bbox, form } => {
                            draw_placeholder(*bbox, form.caption.as_str());
                        }
                        PaintOp::Placeholder { bbox, placeholder } => {
                            draw_placeholder(*bbox, placeholder.label.as_str());
                        }
                        PaintOp::RawSvg { bbox, .. } => draw_placeholder(*bbox, "svg"),
                    }
                }
            }
        }
    }
}

impl LayerRasterRenderer for SkiaLayerRenderer {
    fn render_raster(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        self.render_raster_with_options(tree, options)
    }
}

fn colorref_to_skia(color: ColorRef, alpha_scale: f32) -> Color {
    let b = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let a = (255.0 * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_argb(a, r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{LayerNode, LayerOutputOptions};
    use crate::renderer::render_tree::{BoundingBox, RectangleNode};

    fn decode_rgba(bytes: &[u8]) -> image::RgbaImage {
        image::load_from_memory(bytes)
            .expect("decode png")
            .to_rgba8()
    }

    fn solid_rect_tree(
        page_width: f64,
        page_height: f64,
        bbox: BoundingBox,
        fill_color: ColorRef,
    ) -> PageLayerTree {
        let style = ShapeStyle {
            fill_color: Some(fill_color),
            ..Default::default()
        };
        PageLayerTree::new(
            page_width,
            page_height,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, page_width, page_height),
                None,
                vec![PaintOp::Rectangle {
                    bbox,
                    rect: RectangleNode::new(0.0, style, None),
                }],
            ),
        )
    }

    #[test]
    fn renders_png_for_basic_layer_tree() {
        let tree = solid_rect_tree(
            32.0,
            24.0,
            BoundingBox::new(4.0, 4.0, 16.0, 12.0),
            0x000000ff,
        );

        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render png");

        assert_eq!(output.format, RasterOutputFormat::Png);
        assert_eq!(output.width, 32);
        assert_eq!(output.height, 24);
        assert_eq!(&output.bytes[..8], b"\x89PNG\r\n\x1a\n");
        let decoded = image::load_from_memory(&output.bytes).expect("decode png");
        assert_eq!(decoded.width(), 32);
        assert_eq!(decoded.height(), 24);
    }

    #[test]
    fn raster_options_scale_output_size() {
        let tree = PageLayerTree::new(
            10.0,
            12.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 12.0), None, vec![]),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    scale: 2.0,
                    transparent: false,
                    ..Default::default()
                },
            )
            .expect("render scaled png");

        assert_eq!(output.width, 20);
        assert_eq!(output.height, 24);
    }

    #[test]
    fn preserves_colorref_channel_order_in_pixels() {
        let tree = solid_rect_tree(12.0, 12.0, BoundingBox::new(2.0, 2.0, 8.0, 8.0), 0x000000ff);
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render red rect");
        let image = decode_rgba(&output.bytes);
        let pixel = image.get_pixel(4, 4);

        assert!(pixel[0] > 220, "red channel should be high: {pixel:?}");
        assert!(pixel[1] < 32, "green channel should be low: {pixel:?}");
        assert!(pixel[2] < 32, "blue channel should be low: {pixel:?}");
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn clears_transparent_by_default_and_opaque_when_requested() {
        let tree = PageLayerTree::new(
            4.0,
            4.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 4.0, 4.0), None, vec![]),
        );
        let renderer = SkiaLayerRenderer::new();
        let transparent = renderer
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render transparent");
        let opaque = renderer
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    transparent: false,
                    ..Default::default()
                },
            )
            .expect("render opaque");

        assert_eq!(decode_rgba(&transparent.bytes).get_pixel(0, 0)[3], 0);
        assert_eq!(
            decode_rgba(&opaque.bytes).get_pixel(0, 0).0,
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn output_options_control_clip_rect_replay() {
        let style = ShapeStyle {
            fill_color: Some(0x000000ff),
            ..Default::default()
        };
        let child = LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            None,
            vec![PaintOp::Rectangle {
                bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                rect: RectangleNode::new(0.0, style, None),
            }],
        );
        let clipped = PageLayerTree::new(
            20.0,
            20.0,
            LayerNode::clip_rect(
                BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                None,
                BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                child.clone(),
                crate::paint::ClipKind::Generic,
            ),
        );
        let unclipped = clipped.clone().with_output_options(LayerOutputOptions {
            clip_enabled: false,
            ..Default::default()
        });
        let renderer = SkiaLayerRenderer::new();
        let clipped_png = renderer
            .render_raster_with_options(&clipped, RasterRenderOptions::default())
            .expect("render clipped");
        let unclipped_png = renderer
            .render_raster_with_options(&unclipped, RasterRenderOptions::default())
            .expect("render unclipped");
        let clipped = decode_rgba(&clipped_png.bytes);
        let unclipped = decode_rgba(&unclipped_png.bytes);

        assert_eq!(clipped.get_pixel(15, 15)[3], 0);
        assert_eq!(unclipped.get_pixel(15, 15)[3], 255);
    }

    #[test]
    fn rejects_invalid_raster_options_before_surface_creation() {
        let tree = PageLayerTree::new(
            10.0,
            10.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
        );
        let renderer = SkiaLayerRenderer::new();

        let invalid_scale = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                scale: 0.0,
                ..Default::default()
            },
        );
        assert!(invalid_scale.is_err());

        let invalid_dpi = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                dpi: Some(0.0),
                ..Default::default()
            },
        );
        assert!(invalid_dpi.is_err());

        let oversized = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_dimension: 8,
                ..Default::default()
            },
        );
        assert!(oversized.is_err());
    }
}
