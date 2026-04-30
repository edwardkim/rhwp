//! Canvas 2D 렌더러 (1차 백엔드)
//!
//! 브라우저의 Canvas 2D API를 사용하여 렌더링한다.
//! WASM 환경에서 web-sys를 통해 Canvas에 직접 그린다.

use crate::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp};

use super::layer_renderer::{LayerRenderResult, LayerRenderer};
use super::render_tree::{BoundingBox, PageRenderTree, RenderNode, RenderNodeType, ShapeTransform};
use super::{LineStyle, PathCommand, Renderer, ShapeStyle, TextStyle};

/// Canvas 2D 렌더러
///
/// web-sys의 CanvasRenderingContext2d를 래핑한다.
/// 현재는 구조만 정의하고, WASM 연동은 4단계에서 구현한다.
pub struct CanvasRenderer {
    /// 페이지 폭 (px)
    width: f64,
    /// 페이지 높이 (px)
    height: f64,
    /// 렌더링된 명령 기록 (테스트용)
    commands: Vec<CanvasCommand>,
}

/// Canvas 렌더링 명령 (테스트/디버깅용)
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasCommand {
    BeginPage(f64, f64),
    EndPage,
    FillRect(f64, f64, f64, f64, String),
    StrokeRect(f64, f64, f64, f64, String),
    FillText(String, f64, f64),
    DrawLine(f64, f64, f64, f64),
    DrawEllipse(f64, f64, f64, f64),
    DrawImage(f64, f64, f64, f64),
    BeginPath,
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    /// SVG arc: (rx, ry, x_rotation, large_arc, sweep, x, y)
    ArcTo(f64, f64, f64, bool, bool, f64, f64),
    ClosePath,
    Fill,
    Stroke,
    /// 패턴 채우기 사각형: (x, y, w, h, pattern_type, pattern_color, background_color)
    FillPatternRect(f64, f64, f64, f64, i32, String, String),
    /// 패턴으로 현재 경로 채우기: (pattern_type, pattern_color, background_color)
    FillPattern(i32, String, String),
    /// 상태 저장 (ctx.save)
    Save,
    /// 상태 복원 (ctx.restore)
    Restore,
    /// 아핀 변환: translate(tx, ty) → rotate(rad) → scale(sx, sy) 순서
    SetTransform {
        tx: f64,
        ty: f64,
        rotation_rad: f64,
        sx: f64,
        sy: f64,
    },
}

impl CanvasRenderer {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            commands: Vec::new(),
        }
    }

    /// 기록된 명령 목록 반환 (테스트용)
    pub fn commands(&self) -> &[CanvasCommand] {
        &self.commands
    }

    /// 기록된 명령 수 반환
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// 렌더 트리를 Canvas에 렌더링한다.
    pub fn render_tree(&mut self, tree: &PageRenderTree) {
        self.render_node(&tree.root);
    }

    /// 레이어 트리를 Canvas 명령으로 재생한다.
    pub fn render_layer_tree(&mut self, tree: &PageLayerTree) {
        self.begin_page(tree.page_width, tree.page_height);
        self.render_layer_node(&tree.root);
    }

    /// 개별 노드를 렌더링한다.
    fn render_node(&mut self, node: &RenderNode) {
        if !node.visible {
            return;
        }

        match &node.node_type {
            RenderNodeType::Page(page) => {
                self.begin_page(page.width, page.height);
            }
            RenderNodeType::PageBackground(bg) => {
                if let Some(color) = bg.background_color {
                    let color_str = color_to_css(color);
                    self.commands.push(CanvasCommand::FillRect(
                        node.bbox.x,
                        node.bbox.y,
                        node.bbox.width,
                        node.bbox.height,
                        color_str,
                    ));
                }
            }
            RenderNodeType::TextRun(run) => {
                self.draw_text(
                    &run.text,
                    node.bbox.x,
                    node.bbox.y + node.bbox.height,
                    &run.style,
                );
            }
            RenderNodeType::Rectangle(rect) => {
                self.open_shape_transform(&rect.transform, &node.bbox);
                self.draw_rect(
                    node.bbox.x,
                    node.bbox.y,
                    node.bbox.width,
                    node.bbox.height,
                    rect.corner_radius,
                    &rect.style,
                );
            }
            RenderNodeType::Line(line) => {
                self.open_shape_transform(&line.transform, &node.bbox);
                self.draw_line(line.x1, line.y1, line.x2, line.y2, &line.style);
            }
            RenderNodeType::Ellipse(ellipse) => {
                self.open_shape_transform(&ellipse.transform, &node.bbox);
                let cx = node.bbox.x + node.bbox.width / 2.0;
                let cy = node.bbox.y + node.bbox.height / 2.0;
                self.draw_ellipse(
                    cx,
                    cy,
                    node.bbox.width / 2.0,
                    node.bbox.height / 2.0,
                    &ellipse.style,
                );
            }
            RenderNodeType::Image(img) => {
                self.open_shape_transform(&img.transform, &node.bbox);
                if let Some(ref data) = img.data {
                    self.draw_image(
                        data,
                        node.bbox.x,
                        node.bbox.y,
                        node.bbox.width,
                        node.bbox.height,
                    );
                }
            }
            RenderNodeType::Path(path) => {
                self.open_shape_transform(&path.transform, &node.bbox);
                self.draw_path(&path.commands, &path.style);
            }
            _ => {
                // 구조 노드(Header, Footer, Body, Column 등)는 자식만 렌더링
            }
        }

        // 자식 노드 재귀 렌더링
        for child in &node.children {
            self.render_node(child);
        }

        // 도형 변환 상태 복원
        self.close_shape_transform(&node.node_type);
    }

    fn render_layer_node(&mut self, node: &LayerNode) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.render_layer_node(child);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => {
                self.render_layer_node(child);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    match op {
                        PaintOp::PageBackground { bbox, background } => {
                            if let Some(color) = background.background_color {
                                self.commands.push(CanvasCommand::FillRect(
                                    bbox.x,
                                    bbox.y,
                                    bbox.width,
                                    bbox.height,
                                    color_to_css(color),
                                ));
                            }
                        }
                        PaintOp::TextRun { bbox, run } => {
                            self.draw_text(&run.text, bbox.x, bbox.y + bbox.height, &run.style);
                        }
                        PaintOp::Rectangle { bbox, rect } => {
                            self.open_shape_transform(&rect.transform, bbox);
                            self.draw_rect(
                                bbox.x,
                                bbox.y,
                                bbox.width,
                                bbox.height,
                                rect.corner_radius,
                                &rect.style,
                            );
                            self.close_shape_transform_value(&rect.transform);
                        }
                        PaintOp::Line { bbox, line } => {
                            self.open_shape_transform(&line.transform, bbox);
                            self.draw_line(line.x1, line.y1, line.x2, line.y2, &line.style);
                            self.close_shape_transform_value(&line.transform);
                        }
                        PaintOp::Ellipse { bbox, ellipse } => {
                            self.open_shape_transform(&ellipse.transform, bbox);
                            let cx = bbox.x + bbox.width / 2.0;
                            let cy = bbox.y + bbox.height / 2.0;
                            self.draw_ellipse(
                                cx,
                                cy,
                                bbox.width / 2.0,
                                bbox.height / 2.0,
                                &ellipse.style,
                            );
                            self.close_shape_transform_value(&ellipse.transform);
                        }
                        PaintOp::Image { bbox, image } => {
                            self.open_shape_transform(&image.transform, bbox);
                            if let Some(ref data) = image.data {
                                self.draw_image(data, bbox.x, bbox.y, bbox.width, bbox.height);
                            }
                            self.close_shape_transform_value(&image.transform);
                        }
                        PaintOp::Path { bbox, path } => {
                            self.open_shape_transform(&path.transform, bbox);
                            self.draw_path(&path.commands, &path.style);
                            self.close_shape_transform_value(&path.transform);
                        }
                        PaintOp::FootnoteMarker { .. }
                        | PaintOp::Equation { .. }
                        | PaintOp::FormObject { .. }
                        | PaintOp::Placeholder { .. }
                        | PaintOp::RawSvg { .. } => {}
                    }
                }
            }
        }
    }

    /// 도형 변환(회전/대칭)이 있으면 Save + SetTransform 커맨드를 추가한다.
    fn open_shape_transform(&mut self, transform: &ShapeTransform, bbox: &BoundingBox) {
        if !transform.has_transform() {
            return;
        }
        let cx = bbox.x + bbox.width / 2.0;
        let cy = bbox.y + bbox.height / 2.0;
        let sx = if transform.horz_flip { -1.0 } else { 1.0 };
        let sy = if transform.vert_flip { -1.0 } else { 1.0 };
        let rotation_rad = transform.rotation.to_radians();
        self.commands.push(CanvasCommand::Save);
        self.commands.push(CanvasCommand::SetTransform {
            tx: cx,
            ty: cy,
            rotation_rad,
            sx,
            sy,
        });
    }

    /// 도형 변환 상태를 복원한다 (open_shape_transform에 대응).
    fn close_shape_transform(&mut self, node_type: &RenderNodeType) {
        let transform = match node_type {
            RenderNodeType::Rectangle(r) => &r.transform,
            RenderNodeType::Line(l) => &l.transform,
            RenderNodeType::Ellipse(e) => &e.transform,
            RenderNodeType::Image(i) => &i.transform,
            RenderNodeType::Path(p) => &p.transform,
            _ => return,
        };
        if transform.has_transform() {
            self.commands.push(CanvasCommand::Restore);
        }
    }

    fn close_shape_transform_value(&mut self, transform: &ShapeTransform) {
        if transform.has_transform() {
            self.commands.push(CanvasCommand::Restore);
        }
    }
}

impl LayerRenderer for CanvasRenderer {
    fn render_page(&mut self, tree: &PageLayerTree) -> LayerRenderResult<()> {
        self.render_layer_tree(tree);
        Ok(())
    }
}

impl Renderer for CanvasRenderer {
    fn begin_page(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
        self.commands.push(CanvasCommand::BeginPage(width, height));
    }

    fn end_page(&mut self) {
        self.commands.push(CanvasCommand::EndPage);
    }

    fn draw_text(&mut self, text: &str, x: f64, y: f64, _style: &TextStyle) {
        self.commands
            .push(CanvasCommand::FillText(text.to_string(), x, y));
    }

    fn draw_rect(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        _corner_radius: f64,
        style: &ShapeStyle,
    ) {
        if let Some(ref pat) = style.pattern {
            self.commands.push(CanvasCommand::FillPatternRect(
                x,
                y,
                w,
                h,
                pat.pattern_type,
                color_to_css(pat.pattern_color),
                color_to_css(pat.background_color),
            ));
        } else if let Some(fill) = style.fill_color {
            self.commands
                .push(CanvasCommand::FillRect(x, y, w, h, color_to_css(fill)));
        }
        if let Some(stroke) = style.stroke_color {
            self.commands
                .push(CanvasCommand::StrokeRect(x, y, w, h, color_to_css(stroke)));
        }
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, _style: &LineStyle) {
        self.commands.push(CanvasCommand::DrawLine(x1, y1, x2, y2));
    }

    fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, _style: &ShapeStyle) {
        self.commands
            .push(CanvasCommand::DrawEllipse(cx, cy, rx, ry));
    }

    fn draw_image(&mut self, _data: &[u8], x: f64, y: f64, w: f64, h: f64) {
        self.commands.push(CanvasCommand::DrawImage(x, y, w, h));
    }

    fn draw_path(&mut self, commands: &[PathCommand], _style: &ShapeStyle) {
        self.commands.push(CanvasCommand::BeginPath);
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(x, y) => {
                    self.commands.push(CanvasCommand::MoveTo(*x, *y));
                }
                PathCommand::LineTo(x, y) => {
                    self.commands.push(CanvasCommand::LineTo(*x, *y));
                }
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    self.commands
                        .push(CanvasCommand::CurveTo(*x1, *y1, *x2, *y2, *x, *y));
                }
                PathCommand::ArcTo(rx, ry, x_rot, large_arc, sweep, x, y) => {
                    self.commands.push(CanvasCommand::ArcTo(
                        *rx, *ry, *x_rot, *large_arc, *sweep, *x, *y,
                    ));
                }
                PathCommand::ClosePath => {
                    self.commands.push(CanvasCommand::ClosePath);
                }
            }
        }
        if let Some(ref pat) = _style.pattern {
            self.commands.push(CanvasCommand::FillPattern(
                pat.pattern_type,
                color_to_css(pat.pattern_color),
                color_to_css(pat.background_color),
            ));
        } else {
            self.commands.push(CanvasCommand::Fill);
        }
    }
}

/// COLORREF (BGR) → CSS 색상 문자열 변환
fn color_to_css(color: u32) -> String {
    let b = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let r = color & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::super::render_tree::*;
    use super::*;
    use crate::model::control::FormType;
    use crate::paint::{LayerBuilder, RenderProfile};

    #[test]
    fn test_canvas_renderer_basic() {
        let mut renderer = CanvasRenderer::new();
        renderer.begin_page(800.0, 600.0);
        renderer.draw_text("Hello", 10.0, 20.0, &TextStyle::default());
        renderer.end_page();
        assert_eq!(renderer.command_count(), 3);
    }

    #[test]
    fn test_canvas_renderer_rect() {
        let mut renderer = CanvasRenderer::new();
        let style = ShapeStyle {
            fill_color: Some(0x00FFFFFF),
            stroke_color: Some(0x00000000),
            stroke_width: 1.0,
            ..Default::default()
        };
        renderer.draw_rect(10.0, 20.0, 100.0, 50.0, 0.0, &style);
        assert_eq!(renderer.command_count(), 2); // fill + stroke
    }

    #[test]
    fn test_canvas_renderer_path() {
        let mut renderer = CanvasRenderer::new();
        let commands = vec![
            PathCommand::MoveTo(0.0, 0.0),
            PathCommand::LineTo(100.0, 0.0),
            PathCommand::LineTo(50.0, 100.0),
            PathCommand::ClosePath,
        ];
        renderer.draw_path(&commands, &ShapeStyle::default());
        // BeginPath + 4 commands + Fill = 6
        assert_eq!(renderer.command_count(), 6);
    }

    #[test]
    fn test_color_to_css() {
        // HWP COLORREF: 0x00BBGGRR (BGR)
        assert_eq!(color_to_css(0x000000FF), "#ff0000"); // 빨강
        assert_eq!(color_to_css(0x0000FF00), "#00ff00"); // 초록
        assert_eq!(color_to_css(0x00FF0000), "#0000ff"); // 파랑
        assert_eq!(color_to_css(0x00FFFFFF), "#ffffff"); // 흰색
        assert_eq!(color_to_css(0x00000000), "#000000"); // 검정
    }

    #[test]
    fn test_canvas_render_tree() {
        let mut tree = PageRenderTree::new(0, 800.0, 600.0);
        let bg_id = tree.next_id();
        tree.root.children.push(RenderNode::new(
            bg_id,
            RenderNodeType::PageBackground(PageBackgroundNode {
                background_color: Some(0x00FFFFFF),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            }),
            BoundingBox::new(0.0, 0.0, 800.0, 600.0),
        ));

        let mut renderer = CanvasRenderer::new();
        renderer.render_tree(&tree);
        // BeginPage + FillRect (background)
        assert!(renderer.command_count() >= 2);
    }

    #[test]
    fn canvas_layer_tree_matches_legacy_leaf_ops() {
        let mut tree = PageRenderTree::new(0, 800.0, 600.0);
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::PageBackground(PageBackgroundNode {
                background_color: Some(0x00FFFFFF),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            }),
            BoundingBox::new(0.0, 0.0, 800.0, 600.0),
        ));
        tree.root.children.push(RenderNode::new(
            2,
            RenderNodeType::TextRun(text_run("Canvas")),
            BoundingBox::new(10.0, 20.0, 80.0, 20.0),
        ));
        tree.root.children.push(RenderNode::new(
            3,
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
            BoundingBox::new(20.0, 50.0, 100.0, 40.0),
        ));
        tree.root.children.push(RenderNode::new(
            4,
            RenderNodeType::Line(LineNode::new(
                20.0,
                100.0,
                120.0,
                100.0,
                LineStyle::default(),
            )),
            BoundingBox::new(20.0, 100.0, 100.0, 0.0),
        ));
        tree.root.children.push(RenderNode::new(
            5,
            RenderNodeType::Ellipse(EllipseNode::new(ShapeStyle::default(), None)),
            BoundingBox::new(140.0, 50.0, 60.0, 40.0),
        ));
        tree.root.children.push(RenderNode::new(
            6,
            RenderNodeType::Path(PathNode::new(
                vec![
                    PathCommand::MoveTo(0.0, 0.0),
                    PathCommand::LineTo(20.0, 0.0),
                    PathCommand::ClosePath,
                ],
                ShapeStyle::default(),
                None,
            )),
            BoundingBox::new(200.0, 50.0, 20.0, 20.0),
        ));
        tree.root.children.push(RenderNode::new(
            7,
            RenderNodeType::Image(ImageNode::new(0, Some(vec![1, 2, 3]))),
            BoundingBox::new(240.0, 50.0, 20.0, 20.0),
        ));

        assert_layer_canvas_matches_legacy(&tree);
    }

    #[test]
    fn canvas_layer_tree_matches_legacy_nested_clips() {
        let mut tree = PageRenderTree::new(0, 600.0, 400.0);
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::PageBackground(PageBackgroundNode {
                background_color: Some(0x00FFFFFF),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            }),
            BoundingBox::new(0.0, 0.0, 600.0, 400.0),
        ));

        let mut body = RenderNode::new(
            2,
            RenderNodeType::Body {
                clip_rect: Some(BoundingBox::new(40.0, 40.0, 520.0, 300.0)),
            },
            BoundingBox::new(40.0, 40.0, 520.0, 300.0),
        );
        let mut column = RenderNode::new(
            3,
            RenderNodeType::Column(0),
            BoundingBox::new(40.0, 40.0, 520.0, 300.0),
        );
        column.children.push(RenderNode::new(
            4,
            RenderNodeType::TextRun(text_run("body text")),
            BoundingBox::new(50.0, 60.0, 100.0, 20.0),
        ));

        let mut table = RenderNode::new(
            5,
            RenderNodeType::Table(TableNode {
                row_count: 1,
                col_count: 1,
                border_fill_id: 0,
                section_index: Some(0),
                para_index: Some(0),
                control_index: Some(0),
            }),
            BoundingBox::new(80.0, 100.0, 160.0, 60.0),
        );
        let mut cell = RenderNode::new(
            6,
            RenderNodeType::TableCell(TableCellNode {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                border_fill_id: 0,
                text_direction: 0,
                clip: true,
                model_cell_index: Some(0),
            }),
            BoundingBox::new(80.0, 100.0, 160.0, 60.0),
        );
        cell.children.push(RenderNode::new(
            7,
            RenderNodeType::TextRun(text_run("cell")),
            BoundingBox::new(90.0, 112.0, 60.0, 20.0),
        ));
        table.children.push(cell);
        column.children.push(table);
        body.children.push(column);
        tree.root.children.push(body);

        assert_layer_canvas_matches_legacy(&tree);
    }

    #[test]
    fn canvas_layer_tree_matches_legacy_transformed_shapes() {
        let mut tree = PageRenderTree::new(0, 400.0, 300.0);

        let mut rect = RectangleNode::new(
            2.0,
            ShapeStyle {
                fill_color: Some(0x000000FF),
                stroke_color: Some(0x00000000),
                stroke_width: 1.0,
                ..Default::default()
            },
            None,
        );
        rect.transform = ShapeTransform {
            rotation: 15.0,
            horz_flip: true,
            vert_flip: false,
        };
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::Rectangle(rect),
            BoundingBox::new(20.0, 30.0, 100.0, 40.0),
        ));

        let mut path = PathNode::new(
            vec![
                PathCommand::MoveTo(20.0, 20.0),
                PathCommand::LineTo(90.0, 40.0),
                PathCommand::ClosePath,
            ],
            ShapeStyle::default(),
            None,
        );
        path.transform = ShapeTransform {
            rotation: 45.0,
            horz_flip: false,
            vert_flip: true,
        };
        tree.root.children.push(RenderNode::new(
            2,
            RenderNodeType::Path(path),
            BoundingBox::new(150.0, 80.0, 90.0, 50.0),
        ));

        assert_layer_canvas_matches_legacy(&tree);
    }

    #[test]
    fn canvas_layer_tree_matches_legacy_leaf_nodes_with_children() {
        let mut tree = PageRenderTree::new(0, 400.0, 300.0);
        let mut form = RenderNode::new(
            1,
            RenderNodeType::FormObject(FormObjectNode {
                form_type: FormType::PushButton,
                caption: "OK".to_string(),
                text: String::new(),
                fore_color: "#000000".to_string(),
                back_color: "#ffffff".to_string(),
                value: 0,
                enabled: true,
                section_index: 0,
                para_index: 0,
                control_index: 0,
                name: "button".to_string(),
                cell_location: None,
            }),
            BoundingBox::new(20.0, 30.0, 120.0, 40.0),
        );
        form.children.push(RenderNode::new(
            2,
            RenderNodeType::TextRun(text_run("button label")),
            BoundingBox::new(28.0, 42.0, 90.0, 18.0),
        ));
        tree.root.children.push(form);

        assert_layer_canvas_matches_legacy(&tree);
    }

    fn assert_layer_canvas_matches_legacy(tree: &PageRenderTree) {
        let mut legacy = CanvasRenderer::new();
        legacy.render_tree(&tree);

        let layer_tree = LayerBuilder::new(RenderProfile::Screen).build(tree);
        let mut layer = CanvasRenderer::new();
        layer.render_layer_tree(&layer_tree);

        assert_eq!(layer.commands(), legacy.commands());
    }

    fn text_run(text: &str) -> TextRunNode {
        TextRunNode {
            text: text.to_string(),
            style: TextStyle::default(),
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
            baseline: 12.0,
            field_marker: Default::default(),
        }
    }
}
