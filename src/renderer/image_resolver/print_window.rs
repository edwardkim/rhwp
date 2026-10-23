//! 원본 crop을 팔레트·장치 전달에서 공통으로 소비하는 정수 픽셀 창.

use crate::renderer::render_tree::ImageNode;

#[derive(Clone, Copy)]
pub(super) struct SourceWindow {
    pub bounds: [u32; 4],
    pub full_width: u32,
    pub full_height: u32,
}

impl SourceWindow {
    pub fn for_image(image: &ImageNode, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let bounds = if let Some((left, top, right, bottom)) = image.crop {
            if right.checked_sub(left)? <= 0 || bottom.checked_sub(top)? <= 0 {
                return None;
            }
            if let Some((ref_width, ref_height)) =
                super::image_crop_reference(image).filter(|&(w, h)| w > 0 && h > 0)
            {
                // 원본 밖의 여백은 별도 전송 계약이다. 교집합을 frame 전체로 늘리지 않는다.
                if left < 0
                    || top < 0
                    || i64::from(right) > i64::from(ref_width)
                    || i64::from(bottom) > i64::from(ref_height)
                {
                    return None;
                }
                let edge = |value: i32, pixels: u32, reference: u32, ceil: bool| {
                    let numerator = i64::from(value) * i64::from(pixels);
                    let reference = i64::from(reference);
                    let whole = numerator.div_euclid(reference);
                    (whole + i64::from(ceil && numerator.rem_euclid(reference) != 0)) as u32
                };
                [
                    edge(left, width, ref_width, true),
                    edge(top, height, ref_height, true),
                    edge(right, width, ref_width, false),
                    edge(bottom, height, ref_height, false),
                ]
            } else {
                let (x, y, w, h) = crate::renderer::svg::compute_image_crop_src(
                    (left, top, right, bottom),
                    None,
                    f64::from(width),
                    f64::from(height),
                );
                if ![x, y, w, h].iter().all(|v| v.is_finite())
                    || x < 0.0
                    || y < 0.0
                    || x + w > f64::from(width)
                    || y + h > f64::from(height)
                {
                    return None;
                }
                [
                    x.ceil() as u32,
                    y.ceil() as u32,
                    (x + w).floor() as u32,
                    (y + h).floor() as u32,
                ]
            }
        } else {
            [0, 0, width, height]
        };
        (bounds[0] < bounds[2] && bounds[1] < bounds[3]).then_some(Self {
            bounds,
            full_width: width,
            full_height: height,
        })
    }

    pub fn is_partial(self) -> bool {
        self.bounds != [0, 0, self.full_width, self.full_height]
    }
}
