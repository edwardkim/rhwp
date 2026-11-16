//! Canvas2D의 목적 영역도 같은 Rust 장치 결과를 사용한다.

use crate::renderer::render_tree::{BoundingBox, ImageNode};
use web_sys::{CanvasRenderingContext2d, Path2d};

/// 실제 목적 영역에 Native와 같은 부분 픽셀 덮임을 공급한다.
pub(super) struct DestinationCoverage {
    context: CanvasRenderingContext2d,
}

impl DestinationCoverage {
    pub(super) fn for_print(
        context: &CanvasRenderingContext2d,
        (x, y, width, height): (f64, f64, f64, f64),
        profile: crate::paint::RenderProfile,
    ) -> Option<Self> {
        if profile != crate::paint::RenderProfile::Print
            || ![x, y, width, height].into_iter().all(f64::is_finite)
            || width <= 0.0
            || height <= 0.0
        {
            return None;
        }
        let matrix = context.get_transform().ok()?;
        // 회전·기울임은 브라우저가 이미 그림 경계의 덮임을 계산한다.
        // 축에 평행한 drawImage에만 Native의 anti-alias 경계와 같은 덮임이 필요하다.
        if matrix.b() != 0.0 || matrix.c() != 0.0 {
            return None;
        }
        let path = Path2d::new().ok()?;
        path.rect(x, y, width, height);
        context.save();
        context.clip_with_path_2d(&path);
        Some(Self {
            context: context.clone(),
        })
    }
}

impl Drop for DestinationCoverage {
    fn drop(&mut self) {
        self.context.restore();
    }
}

pub(super) fn bounds(
    context: &CanvasRenderingContext2d,
    bbox: BoundingBox,
    image: &ImageNode,
    print: bool,
) -> BoundingBox {
    if !print || !crate::paint::device_image::eligible(image) {
        return bbox;
    }
    context
        .get_transform()
        .ok()
        .and_then(|m| {
            crate::paint::device_image::bounds(bbox, [m.a(), m.b(), m.c(), m.d(), m.e(), m.f()])
        })
        .unwrap_or(bbox)
}
