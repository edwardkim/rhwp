//! 그림과 테두리가 공유하는 최종 개체 프레임과 선 스타일.

use super::LayoutEngine;
use crate::model::image::Picture;
use crate::renderer::hwpunit_to_px;
use crate::renderer::image_geometry::PictureSourceRect;
use crate::renderer::render_tree::{
    BoundingBox, PageLayoutContext, RectangleNode, RenderNode, RenderNodeType, ShapeTransform,
};

/// 저장 묶음 행렬이 이미 적용된 그림에 원본 회전·대칭을 다시 적용하지 않는다.
struct PictureBorderFrame {
    bbox: BoundingBox,
    transform: ShapeTransform,
    pre_rotation_ratio: Option<[f64; 2]>,
    source_rect: Option<PictureSourceRect>,
}

impl LayoutEngine {
    /// 그림 테두리(선) 렌더링
    /// border_attr의 bit 0~5가 선 종류, border_width가 두께 (0이면 기본 0.1mm)
    pub(crate) fn render_picture_border(
        &self,
        tree: &mut PageLayoutContext,
        parent: &mut RenderNode,
        picture: &crate::model::image::Picture,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    ) {
        if let Some(border) =
            self.make_picture_border_node(tree, picture, BoundingBox::new(x, y, w, h))
        {
            parent.children.push(border);
        }
    }

    /// 인라인·떠 있는 그림·묶음 자식은 같은 최종 이미지 프레임으로 테두리를 그린다.
    /// bitmap crop/inset은 그 프레임 안쪽의 내용만 바꾼다.
    pub(crate) fn append_picture_image_node(
        &self,
        tree: &mut PageLayoutContext,
        parent: &mut RenderNode,
        picture: &crate::model::image::Picture,
        image: RenderNode,
    ) {
        let border = match &image.node_type {
            RenderNodeType::Image(node) => self.make_picture_border_frame(
                tree,
                picture,
                PictureBorderFrame {
                    bbox: image.bbox,
                    transform: node.transform,
                    pre_rotation_ratio: node.pre_rotation_ratio,
                    source_rect: node.source_rect,
                },
            ),
            _ => self.make_picture_border_node(tree, picture, image.bbox),
        };
        parent.children.push(image);
        if let Some(border) = border {
            parent.children.push(border);
        }
    }

    pub(crate) fn make_picture_border_node(
        &self,
        tree: &mut PageLayoutContext,
        picture: &crate::model::image::Picture,
        bbox: BoundingBox,
    ) -> Option<RenderNode> {
        self.make_picture_border_frame(
            tree,
            picture,
            PictureBorderFrame {
                bbox,
                transform: crate::renderer::image_geometry::picture_transform(picture),
                pre_rotation_ratio: crate::renderer::image_geometry::picture_pre_rotation_ratio(
                    picture,
                ),
                source_rect: crate::renderer::image_geometry::picture_source_rect(picture),
            },
        )
    }

    fn make_picture_border_frame(
        &self,
        tree: &mut PageLayoutContext,
        picture: &Picture,
        frame: PictureBorderFrame,
    ) -> Option<RenderNode> {
        let line_type = picture.border_attr.attr & 0x3F;
        // 선 종류 0 = 없음
        if line_type == 0 {
            return None;
        }
        let border_w = if picture.border_width > 0 {
            hwpunit_to_px(picture.border_width, self.dpi)
        } else {
            // 기본 선 두께 0.1mm
            0.1 / 25.4 * self.dpi
        };
        let stroke_dash = match line_type {
            2 => super::super::StrokeDash::Dot,        // 점선
            3 => super::super::StrokeDash::Dash,       // 긴 점선 (파선)
            4 => super::super::StrokeDash::DashDot,    // 일점쇄선
            5 => super::super::StrokeDash::DashDotDot, // 이점쇄선
            _ => super::super::StrokeDash::Solid,      // 1=실선, 기타
        };
        let style = super::super::ShapeStyle {
            fill_color: None,
            pattern: None,
            stroke_color: Some(picture.border_color),
            stroke_width: border_w,
            stroke_dash,
            opacity: 1.0,
            shadow: None,
        };
        let border_id = tree.next_id();
        let transform = frame.transform;
        let bbox = crate::renderer::image_geometry::pre_rotation_bbox(
            &frame.bbox,
            transform,
            frame.pre_rotation_ratio,
        );
        // 독립 Print의 그림 외곽선은 한 화면 픽셀까지 그림 안에 겹치고
        // 나머지 굵기는 바깥으로 자란다. 중앙 stroke로 그리면 굵은 선이
        // 그림 전체를 덮는다. source 그림·점유 프레임은 바꾸지 않는다.
        let overlap = border_w.min(hwpunit_to_px(75, self.dpi));
        let outset = border_w / 2.0 - overlap;
        let bbox = BoundingBox::new(
            bbox.x - outset,
            bbox.y - outset,
            bbox.width + 2.0 * outset,
            bbox.height + 2.0 * outset,
        );
        let bbox =
            crate::renderer::image_geometry::source_geometry(bbox, transform, frame.source_rect)
                .viewport;
        let mut rectangle = RectangleNode::new(0.0, style, None);
        rectangle.picture_border = true;
        rectangle.round_stroke_join = !matches!((picture.border_attr.attr >> 6) & 0x0f, 1 | 2);
        rectangle.transform = transform;
        Some(RenderNode::new(
            border_id,
            RenderNodeType::Rectangle(rectangle),
            bbox,
        ))
    }
}
