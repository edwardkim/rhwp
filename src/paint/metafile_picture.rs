//! 색 보정 WMF의 전체 원본과 물리 crop을 모든 출력 backend에 함께 전달한다.

use crate::model::image::ImageEffect;
use crate::renderer::{
    image_resolver,
    render_tree::{BoundingBox, ImageNode},
};

use super::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, PaintOp, ResolvedImageKind,
    ResolvedImagePayload,
};

pub(super) fn eligible(image: &ImageNode) -> bool {
    image.hancom2024_print_bitmap
        && image.effect == ImageEffect::RealPic
        && (image.brightness != 0 || image.contrast != 0)
        && image.opacity == 1.0
        && image.fill_mode.is_none()
        && !image.transform.has_transform()
        && image
            .data
            .as_deref()
            .is_some_and(|bytes| image_resolver::detect_image_mime_type(bytes) == "image/x-wmf")
}

/// crop 좌표를 bitmap의 정수 창으로 반올림하지 않는다. 원본 전체를 배치하고
/// 같은 물리 crop으로 자른다. 생산 크기와 실제 paint가 이 계산을 공유한다.
pub(super) fn geometry(
    image: &ImageNode,
    frame: BoundingBox,
) -> Option<(BoundingBox, BoundingBox)> {
    let clip = image.paint_bbox(&frame);
    if ![clip.x, clip.y, clip.width, clip.height]
        .into_iter()
        .all(f64::is_finite)
        || clip.width <= 0.0
        || clip.height <= 0.0
    {
        return None;
    }
    let Some((left, top, right, bottom)) = image.crop else {
        return Some((clip, clip));
    };
    let (width, height) = image_resolver::image_crop_reference(image)?;
    if left < 0
        || top < 0
        || right <= left
        || bottom <= top
        || i64::from(right) > i64::from(width)
        || i64::from(bottom) > i64::from(height)
    {
        return None;
    }
    let full_w = clip.width * f64::from(width) / f64::from(right - left);
    let full_h = clip.height * f64::from(height) / f64::from(bottom - top);
    let full = BoundingBox::new(
        clip.x - full_w * f64::from(left) / f64::from(width),
        clip.y - full_h * f64::from(top) / f64::from(height),
        full_w,
        full_h,
    );
    [full.x, full.y, full.width, full.height]
        .into_iter()
        .all(f64::is_finite)
        .then_some((full, clip))
}

pub(crate) fn is_device_picture(
    image: &ImageNode,
    resolved: Option<&ResolvedImagePayload>,
) -> bool {
    eligible(image)
        && resolved.is_some_and(|payload| {
            payload.kind == ResolvedImageKind::OutputDevice
                && payload.suppress_effects
                && image_resolver::png_sampling::print_separate_alpha(&payload.data)
        })
}

/// 원본 render tree는 바꾸지 않는다. 레이어의 출력 사각형과 clip만 함께 만든다.
/// 컨트롤 표시 등 형제 op은 원래 순서와 범위를 보존한다.
pub(super) fn lower(leaf: LayerNode) -> LayerNode {
    let LayerNodeKind::Leaf { ops } = &leaf.kind else {
        return leaf;
    };
    if !ops.iter().any(|op| {
        matches!(op, PaintOp::Image { image, resolved, .. }
        if is_device_picture(image, resolved.as_deref()))
    }) {
        return leaf;
    }
    let LayerNode {
        bounds,
        source_node_id,
        layer,
        kind,
    } = leaf;
    let LayerNodeKind::Leaf { ops } = kind else {
        unreachable!()
    };
    let mut children = Vec::with_capacity(ops.len());
    for mut op in ops {
        let mut physical_clip = None;
        if let PaintOp::Image {
            bbox,
            image,
            resolved,
        } = &mut op
        {
            if is_device_picture(image, resolved.as_deref()) {
                if let Some((full, clip)) = geometry(image, *bbox) {
                    *bbox = full;
                    image.content_inset = None;
                    image.crop = None;
                    image.original_size = None;
                    image.original_size_hu = None;
                    physical_clip = Some(clip);
                }
            }
        }
        let mut child = LayerNode::leaf(op.bounds(), source_node_id, vec![op]).with_layer(layer);
        if let Some(clip) = physical_clip {
            child = LayerNode::clip_rect(clip, source_node_id, clip, child, ClipKind::Generic)
                .with_layer(layer);
        }
        children.push(child);
    }
    if children.len() == 1 {
        children.pop().unwrap()
    } else {
        LayerNode::group(
            bounds,
            source_node_id,
            children,
            CacheHint::None,
            GroupKind::Generic,
        )
        .with_layer(layer)
    }
}
