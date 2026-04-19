use skia_safe::{
    surfaces, Canvas, Color, EncodedImageFormat, FontMgr, Paint, PathBuilder, Point, Rect,
};

use crate::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp};
use crate::renderer::layout::{compute_char_positions, split_into_clusters};
use crate::renderer::render_tree::{BoundingBox, TextRunNode};
use crate::renderer::{LineRenderType, UnderlineType};

use super::equation_conv::render_equation;
use super::image_conv::draw_image_bytes;
use super::paint_conv::{
    colorref_to_skia, make_fill_paint, make_font, make_line_paint, make_stroke_paint,
    make_text_paint,
};
use super::path_conv::to_skia_path;

pub struct SkiaLayerRenderer {
    font_mgr: FontMgr,
}

impl SkiaLayerRenderer {
    pub fn new() -> Self {
        Self {
            font_mgr: FontMgr::default(),
        }
    }

    pub fn render_png(&self, tree: &PageLayerTree) -> Result<Vec<u8>, String> {
        let width = tree.page_width.max(1.0).ceil() as i32;
        let height = tree.page_height.max(1.0).ceil() as i32;
        let mut surface = surfaces::raster_n32_premul((width, height))
            .ok_or_else(|| "Skia raster surface 생성 실패".to_string())?;
        let canvas = surface.canvas();
        canvas.clear(Color::from_argb(0, 0, 0, 0));
        self.render_node(canvas, &tree.root);
        let image = surface.image_snapshot();
        let data = image
            .encode(None, EncodedImageFormat::PNG, None)
            .ok_or_else(|| "Skia PNG 인코딩 실패".to_string())?;
        Ok(data.as_bytes().to_vec())
    }

    fn render_node(&self, canvas: &Canvas, node: &LayerNode) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.render_node(canvas, child);
                }
            }
            LayerNodeKind::ClipRect { clip, child, .. } => {
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
                self.render_node(canvas, child);
                canvas.restore();
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    self.render_op(canvas, op);
                }
            }
        }
    }

    fn render_op(&self, canvas: &Canvas, op: &PaintOp) {
        match op {
            PaintOp::PageBackground { bbox, background } => {
                if let Some(color) = background.background_color {
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(colorref_to_skia(color, 1.0));
                    canvas.draw_rect(
                        Rect::from_xywh(
                            bbox.x as f32,
                            bbox.y as f32,
                            bbox.width as f32,
                            bbox.height as f32,
                        ),
                        &paint,
                    );
                }
                if let Some(image) = &background.image {
                    draw_image_bytes(
                        canvas,
                        &image.data,
                        bbox.x as f32,
                        bbox.y as f32,
                        bbox.width as f32,
                        bbox.height as f32,
                        Some(image.fill_mode),
                        None,
                        None,
                    );
                }
                if let Some(border) = background.border_color {
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_style(skia_safe::paint::Style::Stroke);
                    paint.set_stroke_width(if background.border_width > 0.0 {
                        background.border_width as f32
                    } else {
                        1.0
                    });
                    paint.set_color(colorref_to_skia(border, 1.0));
                    canvas.draw_rect(
                        Rect::from_xywh(
                            bbox.x as f32,
                            bbox.y as f32,
                            bbox.width as f32,
                            bbox.height as f32,
                        ),
                        &paint,
                    );
                }
            }
            PaintOp::TextRun { bbox, run } => self.render_text_run(canvas, bbox, run),
            PaintOp::FootnoteMarker { bbox, marker } => {
                let mut font = make_font(
                    &crate::renderer::TextStyle {
                        font_family: marker.font_family.clone(),
                        font_size: (marker.base_font_size * 0.55).max(7.0),
                        color: marker.color,
                        ..Default::default()
                    },
                    &self.font_mgr,
                    &marker.text,
                );
                font.set_size((marker.base_font_size * 0.55).max(7.0) as f32);
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_color(colorref_to_skia(marker.color, 1.0));
                canvas.draw_str(
                    &marker.text,
                    (bbox.x as f32, (bbox.y + bbox.height * 0.4) as f32),
                    &font,
                    &paint,
                );
            }
            PaintOp::Line { line, .. } => {
                self.with_shape_transform(canvas, line.transform, None, |canvas| {
                    let paint = make_line_paint(&line.style);
                    match line.style.line_type {
                        LineRenderType::Single => canvas.draw_line(
                            (line.x1 as f32, line.y1 as f32),
                            (line.x2 as f32, line.y2 as f32),
                            &paint,
                        ),
                        _ => canvas.draw_line(
                            (line.x1 as f32, line.y1 as f32),
                            (line.x2 as f32, line.y2 as f32),
                            &paint,
                        ),
                    };
                });
            }
            PaintOp::Rectangle { bbox, rect } => {
                self.with_shape_transform(canvas, rect.transform, Some(*bbox), |canvas| {
                    let sk_rect = Rect::from_xywh(
                        bbox.x as f32,
                        bbox.y as f32,
                        bbox.width as f32,
                        bbox.height as f32,
                    );
                    if let Some(fill) = make_fill_paint(&rect.style) {
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
                });
            }
            PaintOp::Ellipse { bbox, ellipse } => {
                self.with_shape_transform(canvas, ellipse.transform, Some(*bbox), |canvas| {
                    let oval = Rect::from_xywh(
                        bbox.x as f32,
                        bbox.y as f32,
                        bbox.width as f32,
                        bbox.height as f32,
                    );
                    if let Some(fill) = make_fill_paint(&ellipse.style) {
                        canvas.draw_oval(oval, &fill);
                    }
                    if let Some(stroke) = make_stroke_paint(&ellipse.style) {
                        canvas.draw_oval(oval, &stroke);
                    }
                });
            }
            PaintOp::Path { path, .. } => {
                self.with_shape_transform(canvas, path.transform, None, |canvas| {
                    let sk_path = to_skia_path(&path.commands);
                    if let Some(fill) = make_fill_paint(&path.style) {
                        canvas.draw_path(&sk_path, &fill);
                    }
                    if let Some(stroke) = make_stroke_paint(&path.style) {
                        canvas.draw_path(&sk_path, &stroke);
                    }
                });
            }
            PaintOp::Image { bbox, image } => {
                self.with_shape_transform(canvas, image.transform, Some(*bbox), |canvas| {
                    if let Some(data) = &image.data {
                        draw_image_bytes(
                            canvas,
                            data,
                            bbox.x as f32,
                            bbox.y as f32,
                            bbox.width as f32,
                            bbox.height as f32,
                            image.fill_mode,
                            image.original_size,
                            image.crop,
                        );
                    }
                });
            }
            PaintOp::Equation { bbox, equation } => {
                render_equation(
                    canvas,
                    &self.font_mgr,
                    &equation.layout_box,
                    bbox.x,
                    bbox.y,
                    equation.color,
                    equation.font_size,
                );
            }
            PaintOp::FormObject { bbox, form } => self.render_form_object(canvas, bbox, form),
        }
    }

    fn render_form_object(
        &self,
        canvas: &Canvas,
        bbox: &BoundingBox,
        form: &crate::renderer::render_tree::FormObjectNode,
    ) {
        let parse_css = |value: &str, fallback: Color| {
            if let Some(hex) = value.strip_prefix('#') {
                if hex.len() == 6 {
                    let parsed = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    );
                    if let (Ok(r), Ok(g), Ok(b)) = parsed {
                        return Color::from_argb(255, r, g, b);
                    }
                }
            }
            fallback
        };
        let rect = Rect::from_xywh(
            bbox.x as f32,
            bbox.y as f32,
            bbox.width as f32,
            bbox.height as f32,
        );
        let mut text_style = crate::renderer::TextStyle {
            font_family: "Noto Sans CJK KR".to_string(),
            ..Default::default()
        };

        match form.form_type {
            crate::model::control::FormType::PushButton => {
                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_color(Color::from_argb(255, 208, 208, 208));
                canvas.draw_rect(rect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(skia_safe::paint::Style::Stroke);
                stroke.set_stroke_width(0.5);
                stroke.set_color(Color::from_argb(255, 160, 160, 160));
                canvas.draw_rect(rect, &stroke);

                if !form.caption.is_empty() {
                    let font_size = (bbox.height * 0.55).clamp(7.0, 12.0);
                    text_style.font_size = font_size;
                    let font =
                        super::paint_conv::make_font(&text_style, &self.font_mgr, &form.caption);
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(Color::from_argb(255, 128, 128, 128));
                    let text_width = form.caption.chars().count() as f32 * font_size as f32 * 0.55;
                    canvas.draw_str(
                        &form.caption,
                        (
                            bbox.x as f32 + bbox.width as f32 / 2.0 - text_width / 2.0,
                            bbox.y as f32 + bbox.height as f32 / 2.0 + font_size as f32 * 0.35,
                        ),
                        &font,
                        &paint,
                    );
                }
            }
            crate::model::control::FormType::CheckBox => {
                let box_size = (bbox.height * 0.7).min(13.0) as f32;
                let box_x = bbox.x as f32 + 2.0;
                let box_y = bbox.y as f32 + (bbox.height as f32 - box_size) / 2.0;

                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_color(Color::WHITE);
                canvas.draw_rect(Rect::from_xywh(box_x, box_y, box_size, box_size), &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(skia_safe::paint::Style::Stroke);
                stroke.set_stroke_width(0.8);
                stroke.set_color(Color::from_argb(255, 96, 96, 96));
                canvas.draw_rect(Rect::from_xywh(box_x, box_y, box_size, box_size), &stroke);

                if form.value != 0 {
                    let mut check = PathBuilder::new();
                    check.move_to((box_x + box_size * 0.2, box_y + box_size * 0.55));
                    check.line_to((box_x + box_size * 0.45, box_y + box_size * 0.8));
                    check.line_to((box_x + box_size * 0.85, box_y + box_size * 0.2));
                    let mut mark = Paint::default();
                    mark.set_anti_alias(true);
                    mark.set_style(skia_safe::paint::Style::Stroke);
                    mark.set_stroke_width(1.5);
                    mark.set_color(Color::BLACK);
                    canvas.draw_path(&check.detach(), &mark);
                }

                if !form.caption.is_empty() {
                    let font_size = (bbox.height * 0.55).clamp(7.0, 12.0);
                    text_style.font_size = font_size;
                    let font =
                        super::paint_conv::make_font(&text_style, &self.font_mgr, &form.caption);
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(parse_css(&form.fore_color, Color::BLACK));
                    canvas.draw_str(
                        &form.caption,
                        (
                            box_x + box_size + 3.0,
                            bbox.y as f32 + bbox.height as f32 / 2.0 + font_size as f32 * 0.35,
                        ),
                        &font,
                        &paint,
                    );
                }
            }
            crate::model::control::FormType::RadioButton => {
                let radius = (bbox.height * 0.3).min(6.5) as f32;
                let cx = bbox.x as f32 + 2.0 + radius;
                let cy = bbox.y as f32 + bbox.height as f32 / 2.0;

                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_color(Color::WHITE);
                canvas.draw_circle((cx, cy), radius, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(skia_safe::paint::Style::Stroke);
                stroke.set_stroke_width(0.8);
                stroke.set_color(Color::from_argb(255, 96, 96, 96));
                canvas.draw_circle((cx, cy), radius, &stroke);

                if form.value != 0 {
                    let mut dot = Paint::default();
                    dot.set_anti_alias(true);
                    dot.set_color(Color::BLACK);
                    canvas.draw_circle((cx, cy), radius * 0.5, &dot);
                }

                if !form.caption.is_empty() {
                    let font_size = (bbox.height * 0.55).clamp(7.0, 12.0);
                    text_style.font_size = font_size;
                    let font =
                        super::paint_conv::make_font(&text_style, &self.font_mgr, &form.caption);
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(parse_css(&form.fore_color, Color::BLACK));
                    canvas.draw_str(
                        &form.caption,
                        (
                            cx + radius + 3.0,
                            bbox.y as f32 + bbox.height as f32 / 2.0 + font_size as f32 * 0.35,
                        ),
                        &font,
                        &paint,
                    );
                }
            }
            crate::model::control::FormType::ComboBox => {
                let btn_w = (bbox.height * 0.8).min(16.0) as f32;
                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_color(Color::WHITE);
                canvas.draw_rect(rect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(skia_safe::paint::Style::Stroke);
                stroke.set_stroke_width(0.8);
                stroke.set_color(Color::from_argb(255, 160, 160, 160));
                canvas.draw_rect(rect, &stroke);

                let button_rect = Rect::from_xywh(
                    bbox.x as f32 + bbox.width as f32 - btn_w,
                    bbox.y as f32,
                    btn_w,
                    bbox.height as f32,
                );
                let mut button_fill = Paint::default();
                button_fill.set_anti_alias(true);
                button_fill.set_color(Color::from_argb(255, 224, 224, 224));
                canvas.draw_rect(button_rect, &button_fill);

                let mut button_stroke = Paint::default();
                button_stroke.set_anti_alias(true);
                button_stroke.set_style(skia_safe::paint::Style::Stroke);
                button_stroke.set_stroke_width(0.5);
                button_stroke.set_color(Color::from_argb(255, 160, 160, 160));
                canvas.draw_rect(button_rect, &button_stroke);

                let arrow_cx = bbox.x as f32 + bbox.width as f32 - btn_w / 2.0;
                let arrow_cy = bbox.y as f32 + bbox.height as f32 / 2.0;
                let arrow_size = (bbox.height * 0.2).min(4.0) as f32;
                let mut arrow = PathBuilder::new();
                arrow.move_to((arrow_cx - arrow_size, arrow_cy - arrow_size * 0.5));
                arrow.line_to((arrow_cx + arrow_size, arrow_cy - arrow_size * 0.5));
                arrow.line_to((arrow_cx, arrow_cy + arrow_size * 0.5));
                arrow.close();
                let mut arrow_paint = Paint::default();
                arrow_paint.set_anti_alias(true);
                arrow_paint.set_color(Color::from_argb(255, 64, 64, 64));
                canvas.draw_path(&arrow.detach(), &arrow_paint);

                if !form.text.is_empty() {
                    let font_size = (bbox.height * 0.55).clamp(7.0, 12.0);
                    text_style.font_size = font_size;
                    let font =
                        super::paint_conv::make_font(&text_style, &self.font_mgr, &form.text);
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(parse_css(&form.fore_color, Color::BLACK));
                    canvas.draw_str(
                        &form.text,
                        (
                            bbox.x as f32 + 3.0,
                            bbox.y as f32 + bbox.height as f32 / 2.0 + font_size as f32 * 0.35,
                        ),
                        &font,
                        &paint,
                    );
                }
            }
            crate::model::control::FormType::Edit => {
                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_color(Color::WHITE);
                canvas.draw_rect(rect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(skia_safe::paint::Style::Stroke);
                stroke.set_stroke_width(0.8);
                stroke.set_color(Color::from_argb(255, 160, 160, 160));
                canvas.draw_rect(rect, &stroke);

                if !form.text.is_empty() {
                    let font_size = (bbox.height * 0.55).clamp(7.0, 12.0);
                    text_style.font_size = font_size;
                    let font =
                        super::paint_conv::make_font(&text_style, &self.font_mgr, &form.text);
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(parse_css(&form.fore_color, Color::BLACK));
                    canvas.draw_str(
                        &form.text,
                        (
                            bbox.x as f32 + 3.0,
                            bbox.y as f32 + bbox.height as f32 / 2.0 + font_size as f32 * 0.35,
                        ),
                        &font,
                        &paint,
                    );
                }
            }
        }
    }

    fn render_text_run(&self, canvas: &Canvas, bbox: &BoundingBox, run: &TextRunNode) {
        let paint = make_text_paint(&run.style);
        let y = (bbox.y + run.baseline) as f32;
        let char_positions = compute_char_positions(&run.text, &run.style);
        let clusters = split_into_clusters(&run.text);
        let metrics_font = make_font(&run.style, &self.font_mgr, &run.text);

        if run.style.shadow_type > 0 {
            let mut shadow_paint = Paint::default();
            shadow_paint.set_anti_alias(true);
            shadow_paint.set_color(colorref_to_skia(run.style.shadow_color, 1.0));
            for (char_idx, cluster) in &clusters {
                if cluster == " " || cluster == "\t" {
                    continue;
                }
                let font = make_font(&run.style, &self.font_mgr, cluster);
                let x = bbox.x + char_positions[*char_idx] + run.style.shadow_offset_x;
                let shadow_y = y + run.style.shadow_offset_y as f32;
                let glyphs = font.text_to_glyphs_vec(cluster);
                let mut glyph_positions = vec![Point::default(); glyphs.len()];
                font.get_pos(
                    &glyphs,
                    &mut glyph_positions,
                    Some(Point::new(x as f32, shadow_y)),
                );
                for (glyph_id, glyph_position) in glyphs.into_iter().zip(glyph_positions) {
                    if let Some(path) = font.get_path(glyph_id) {
                        let path = path.with_offset((glyph_position.x, glyph_position.y));
                        canvas.draw_path(&path, &shadow_paint);
                    }
                }
            }
        }

        for (char_idx, cluster) in &clusters {
            if cluster == " " || cluster == "\t" {
                continue;
            }
            let font = make_font(&run.style, &self.font_mgr, cluster);
            let x = bbox.x + char_positions[*char_idx];
            let glyphs = font.text_to_glyphs_vec(cluster);
            let mut glyph_positions = vec![Point::default(); glyphs.len()];
            font.get_pos(&glyphs, &mut glyph_positions, Some(Point::new(x as f32, y)));
            for (glyph_id, glyph_position) in glyphs.into_iter().zip(glyph_positions) {
                if let Some(path) = font.get_path(glyph_id) {
                    let path = path.with_offset((glyph_position.x, glyph_position.y));
                    canvas.draw_path(&path, &paint);
                }
            }
        }

        let text_width = char_positions.last().copied().unwrap_or(0.0) as f32;
        if !matches!(run.style.underline, UnderlineType::None) {
            let ul_y = match run.style.underline {
                UnderlineType::Top => y - metrics_font.size() + 1.0,
                _ => y + 2.0,
            };
            let mut line_paint = Paint::default();
            line_paint.set_anti_alias(true);
            line_paint.set_style(skia_safe::paint::Style::Stroke);
            line_paint.set_stroke_width(1.0);
            line_paint.set_color(colorref_to_skia(
                if run.style.underline_color != 0 {
                    run.style.underline_color
                } else {
                    run.style.color
                },
                1.0,
            ));
            canvas.draw_line(
                (bbox.x as f32, ul_y),
                ((bbox.x as f32) + text_width, ul_y),
                &line_paint,
            );
        }
        if run.style.strikethrough {
            let strike_y = y - metrics_font.size() * 0.3;
            let mut line_paint = Paint::default();
            line_paint.set_anti_alias(true);
            line_paint.set_style(skia_safe::paint::Style::Stroke);
            line_paint.set_stroke_width(1.0);
            line_paint.set_color(colorref_to_skia(
                if run.style.strike_color != 0 {
                    run.style.strike_color
                } else {
                    run.style.color
                },
                1.0,
            ));
            canvas.draw_line(
                (bbox.x as f32, strike_y),
                ((bbox.x as f32) + text_width, strike_y),
                &line_paint,
            );
        }
    }

    fn with_shape_transform<F>(
        &self,
        canvas: &Canvas,
        transform: crate::renderer::render_tree::ShapeTransform,
        bbox: Option<BoundingBox>,
        draw: F,
    ) where
        F: FnOnce(&Canvas),
    {
        if !transform.has_transform() {
            draw(canvas);
            return;
        }
        let bbox = bbox.unwrap_or(BoundingBox::new(0.0, 0.0, 0.0, 0.0));
        let cx = (bbox.x + bbox.width / 2.0) as f32;
        let cy = (bbox.y + bbox.height / 2.0) as f32;
        canvas.save();
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
        draw(canvas);
        canvas.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::SkiaLayerRenderer;
    use crate::paint::{LayerBuilder, RenderProfile};
    use crate::renderer::render_tree::{
        BoundingBox, PageNode, RectangleNode, RenderNode, RenderNodeType,
    };
    use crate::renderer::ShapeStyle;

    #[test]
    fn renders_basic_rect_to_png() {
        let mut tree = crate::renderer::render_tree::PageRenderTree::new(0, 120.0, 80.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 120.0,
            height: 80.0,
            section_index: 0,
        });
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::Rectangle(RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x0000FF00),
                    stroke_color: Some(0x00000000),
                    stroke_width: 1.0,
                    ..Default::default()
                },
                None,
            )),
            BoundingBox::new(10.0, 10.0, 50.0, 30.0),
        ));
        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        let renderer = SkiaLayerRenderer::new();
        let png = renderer.render_png(&layer_tree).expect("skia png render");
        assert!(!png.is_empty());
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
    }
}
