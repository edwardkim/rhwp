//! 논리 조판과 물리 Print 종이의 공통 paint 참조 프레임.

use crate::paint::RenderProfile;
use crate::renderer::image_resolver::detect_image_mime_type;
use crate::renderer::render_tree::{BoundingBox, ImageNode, RenderNode};

#[derive(Clone, Copy)]
pub(crate) struct PagePaintContext {
    print: bool,
    logical_size: (f64, f64),
    output_size: (f64, f64),
}

impl PagePaintContext {
    pub(crate) fn new(profile: RenderProfile, width: f64, height: f64) -> Self {
        let print = profile == RenderProfile::Print;
        // HU에서 변환한 정수 point가 부동소수점 오차로 한 단계 내려가지 않게 한다.
        let physical_extent = |px: f64| (px * 0.75 + 1e-9).floor() / 0.75;
        Self {
            print,
            logical_size: (width, height),
            output_size: if print {
                (physical_extent(width), physical_extent(height))
            } else {
                (width, height)
            },
        }
    }

    pub(crate) fn output_size(self) -> (f64, f64) {
        self.output_size
    }

    pub(crate) fn image_frame(self, node: &RenderNode, image: &ImageNode) -> BoundingBox {
        let is_master = node.layer.as_ref().is_some_and(|layer| layer.master_page);
        let raster = image.data.as_deref().is_some_and(|bytes| {
            matches!(
                detect_image_mime_type(bytes),
                "image/jpeg"
                    | "image/png"
                    | "image/bmp"
                    | "image/tiff"
                    | "image/gif"
                    | "image/webp"
                    | "image/x-pcx"
            )
        });
        if !self.print || !is_master || !raster {
            return node.bbox;
        }
        let (width, height) = self.logical_size;
        let (output_width, output_height) = self.output_size;
        if ![width, height, output_width, output_height]
            .iter()
            .all(|extent| extent.is_finite() && *extent > 0.0)
        {
            // 잘못된 종이 크기는 기존 backend의 page validation이 거부한다.
            return node.bbox;
        }
        // 바탕쪽 raster 그림의 인쇄 목적 프레임만 투영한다. 글자·벡터와 본문은
        // 원본 조판 좌표를 소비하고, crop/inset은 이 사각형 안에서 한 번 적용된다.
        let sx = output_width / width;
        let sy = output_height / height;
        BoundingBox::new(
            node.bbox.x * sx,
            node.bbox.y * sy,
            node.bbox.width * sx,
            node.bbox.height * sy,
        )
    }
}
