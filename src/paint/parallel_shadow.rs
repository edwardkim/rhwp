//! 도형 그림자(평행·작게·크게·기울임)를 backend 공통의 일반 도형과 변환 레이어로 내린다.

use crate::paint::layer_tree::{CacheHint, GroupKind, LayerNode};
use crate::renderer::render_tree::{BoundingBox, GroupNode, PathNode, RenderNode, RenderNodeType};
use crate::renderer::{alpha_to_opacity, PathCommand, ShadowStyle, ShapeStyle};

pub(super) struct ShadowParts {
    pub body: RenderNode,
    pub silhouette: RenderNode,
    pub matrix: [f64; 6],
}

fn shadow_style(node: &RenderNode) -> Option<&ShadowStyle> {
    match &node.node_type {
        RenderNodeType::Rectangle(rect) => rect.style.shadow.as_ref(),
        RenderNodeType::Ellipse(ellipse) => ellipse.style.shadow.as_ref(),
        RenderNodeType::Line(line) => line.style.shadow.as_ref(),
        RenderNodeType::Path(path) => path.style.shadow.as_ref().or_else(|| {
            path.line_style
                .as_ref()
                .and_then(|style| style.shadow.as_ref())
        }),
        _ => None,
    }
}

fn clear_shadow(node: &mut RenderNode) {
    match &mut node.node_type {
        RenderNodeType::Rectangle(rect) => rect.style.shadow = None,
        RenderNodeType::Ellipse(ellipse) => ellipse.style.shadow = None,
        RenderNodeType::Line(line) => line.style.shadow = None,
        RenderNodeType::Path(path) => {
            path.style.shadow = None;
            if let Some(style) = &mut path.line_style {
                style.shadow = None;
            }
        }
        _ => {}
    }
}

/// 원 획과 그림자의 불투명도를 곱한 뒤 가장 가까운 HWP 투명도 바이트로 되돌린다.
fn shadow_stroke_alpha(stroke: u8, shadow: u8) -> u8 {
    let opacity = u16::from(255 - stroke) * u16::from(255 - shadow);
    255 - ((opacity + 127) / 255) as u8
}

fn recolor(style: &mut ShapeStyle, shadow: &ShadowStyle, has_gradient: bool) {
    style.fill_color = (style.fill_color.is_some() || style.pattern.is_some() || has_gradient)
        .then_some(shadow.color);
    style.pattern = None;
    style.stroke_color = style.stroke_color.map(|_| shadow.color);
    style.opacity *= alpha_to_opacity(shadow.alpha);
    style.stroke_alpha = shadow_stroke_alpha(style.stroke_alpha, shadow.alpha);
    style.shadow = None;
}

/// 그림자 실루엣을 본체 상자에서 그림자 자리로 옮기는 쪽 공간 행렬.
/// 그림자 레이어의 재생과 잉크 경계 계산이 함께 소비한다.
pub(super) fn matrix_px(shadow: &ShadowStyle, body: BoundingBox) -> Option<[f64; 6]> {
    crate::model::shape::shadow::matrix(
        shadow.shadow_type,
        [body.x, body.y, body.width, body.height],
        crate::renderer::DEFAULT_DPI / crate::renderer::HWPUNIT_PER_INCH,
        (shadow.offset_x, shadow.offset_y),
    )
}

pub(super) fn split(node: &RenderNode) -> Option<ShadowParts> {
    let shadow = shadow_style(node)?;
    let matrix = matrix_px(shadow, node.bbox)?;

    let mut body = node.clone();
    clear_shadow(&mut body);
    let mut silhouette = body.clone();
    // 내부 텍스트·이미지·편집 부호를 그림자 도형으로 복제하지 않는다.
    silhouette.children.clear();
    silhouette.control_code = crate::renderer::render_tree::ControlCode::Hidden;
    match &mut silhouette.node_type {
        RenderNodeType::Rectangle(rect) => {
            recolor(&mut rect.style, shadow, rect.gradient.is_some());
            rect.gradient = None;
            rect.image_frame_owner = None;
            rect.picture_border_width_hu = None;
        }
        RenderNodeType::Ellipse(ellipse) => {
            recolor(&mut ellipse.style, shadow, ellipse.gradient.is_some());
            ellipse.gradient = None;
        }
        RenderNodeType::Path(path) => {
            recolor(&mut path.style, shadow, path.gradient.is_some());
            path.gradient = None;
            if let Some(style) = &mut path.line_style {
                style.color = shadow.color;
                style.alpha = shadow_stroke_alpha(style.alpha, shadow.alpha);
            }
        }
        RenderNodeType::Line(line) => {
            let alpha = shadow_stroke_alpha(line.style.alpha, shadow.alpha);
            let style = ShapeStyle {
                stroke_color: Some(shadow.color),
                stroke_width: line.style.width,
                stroke_dash: line.style.dash,
                stroke_alpha: alpha,
                ..ShapeStyle::default()
            };
            let mut path = PathNode::new(
                vec![
                    PathCommand::MoveTo(line.x1, line.y1),
                    PathCommand::LineTo(line.x2, line.y2),
                ],
                style,
                None,
            );
            let mut line_style = line.style.clone();
            line_style.color = shadow.color;
            line_style.alpha = alpha;
            path.line_style = Some(line_style);
            path.connector_endpoints = Some((line.x1, line.y1, line.x2, line.y2));
            path.transform = line.transform;
            path.section_index = line.section_index;
            path.para_index = line.para_index;
            path.control_index = line.control_index;
            path.cell_index = line.cell_index;
            path.cell_para_index = line.cell_para_index;
            path.outer_table_control_index = line.outer_table_control_index;
            silhouette.node_type = RenderNodeType::Path(path);
        }
        _ => return None,
    }
    Some(ShadowParts {
        body,
        silhouette,
        matrix,
    })
}

/// 도형 변환 뒤의 페이지 좌표에서 그림자 자리로 옮긴다. 같은 레이어를 재생·경계 계산이 소비한다.
pub(super) fn transformed(layer: LayerNode, matrix: [f64; 6]) -> LayerNode {
    let source = layer.source_node_id;
    let metadata = layer.layer;
    let mut group = LayerNode::group(
        layer.bounds,
        source,
        vec![layer],
        CacheHint::None,
        GroupKind::Group(GroupNode {
            paint_transform: Some(matrix),
            section_index: None,
            para_index: None,
            control_index: None,
            cell_context: None,
        }),
    )
    .with_layer(metadata);
    group.bounds = group.visual_bounds(true);
    group
}
