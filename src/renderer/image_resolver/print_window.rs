//! 원본 crop을 팔레트·장치 전달에서 공통으로 소비하는 정수 픽셀 창.

use crate::renderer::render_tree::ImageNode;

#[derive(Clone, Copy)]
pub(super) struct SourceWindow {
    pub bounds: [u32; 4],
    pub full_width: u32,
    pub full_height: u32,
    /// crop 전체가 원본 안에 있을 때만 잘린 바이트로 원본을 대체할 수 있다.
    pub contained: bool,
}

impl SourceWindow {
    pub fn for_image(image: &ImageNode, width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let mut contained = true;
        let bounds = if let Some((left, top, right, bottom)) = image.crop {
            if right.checked_sub(left)? <= 0 || bottom.checked_sub(top)? <= 0 {
                return None;
            }
            if let Some((ref_width, ref_height)) =
                super::image_crop_reference(image).filter(|&(w, h)| w > 0 && h > 0)
            {
                // 팔레트는 실제 원본과의 교집합을 센다. 원본 밖 여백이 있으면
                // 잘린 바이트로 frame을 채우지 않고 기존 crop 전송을 유지한다.
                contained = left >= 0
                    && top >= 0
                    && i64::from(right) <= i64::from(ref_width)
                    && i64::from(bottom) <= i64::from(ref_height);
                let edge = |value: i32, pixels: u32, reference: u32, ceil: bool| {
                    let numerator = i64::from(value) * i64::from(pixels);
                    let reference = i64::from(reference);
                    let whole = numerator.div_euclid(reference);
                    (whole + i64::from(ceil && numerator.rem_euclid(reference) != 0))
                        .clamp(0, i64::from(pixels)) as u32
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
                if ![x, y, w, h].iter().all(|v| v.is_finite()) {
                    return None;
                }
                contained =
                    x >= 0.0 && y >= 0.0 && x + w <= f64::from(width) && y + h <= f64::from(height);
                [
                    x.ceil().clamp(0.0, f64::from(width)) as u32,
                    y.ceil().clamp(0.0, f64::from(height)) as u32,
                    (x + w).floor().clamp(0.0, f64::from(width)) as u32,
                    (y + h).floor().clamp(0.0, f64::from(height)) as u32,
                ]
            }
        } else {
            [0, 0, width, height]
        };
        (bounds[0] < bounds[2] && bounds[1] < bounds[3]).then_some(Self {
            bounds,
            full_width: width,
            full_height: height,
            contained,
        })
    }

    pub fn is_partial(self) -> bool {
        self.bounds != [0, 0, self.full_width, self.full_height]
    }
}
