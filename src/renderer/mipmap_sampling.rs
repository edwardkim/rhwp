//! 실제 장치 축소율로 선택한 두 mip 레벨을 premultiplied RGBA로 재생한다.
//! 짝수 축은 2표본 평균, 홀수 축은 겹치는 3표본 triangle로 줄인다.
//! Skia의 기본 mipmap 계약과 대조하며 문서·페이지·원본 좌표를 판별하지 않는다.

const MAX_PIXELS: u64 = 32 * 1024 * 1024;

pub(crate) struct DeviceRaster {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

struct Level {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}
impl Level {
    fn reduce(&self) -> Self {
        let width = (self.width / 2).max(1);
        let height = (self.height / 2).max(1);
        let axis = |size: usize| -> &'static [u32] {
            if size == 1 {
                &[1]
            } else if size.is_multiple_of(2) {
                &[1, 1]
            } else {
                &[1, 2, 1]
            }
        };
        let wx = axis(self.width);
        let wy = axis(self.height);
        let divisor = wx.iter().sum::<u32>() * wy.iter().sum::<u32>();
        let mut pixels = vec![0u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                for channel in 0..4 {
                    let mut sum = 0;
                    for (dy, weight_y) in wy.iter().enumerate() {
                        for (dx, weight_x) in wx.iter().enumerate() {
                            sum += u32::from(
                                self.pixels[((y * 2 + dy) * self.width + x * 2 + dx) * 4 + channel],
                            ) * weight_x
                                * weight_y;
                        }
                    }
                    pixels[(y * width + x) * 4 + channel] = (sum / divisor) as u8;
                }
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }
    fn sample(&self, x: f32, y: f32) -> [f32; 4] {
        let x = x.clamp(0.0, (self.width - 1) as f32);
        let y = y.clamp(0.0, (self.height - 1) as f32);
        let ix = x.floor() as usize;
        let iy = y.floor() as usize;
        let fx = x - ix as f32;
        let fy = y - iy as f32;
        let mut value = [0.0; 4];
        for (dy, wy) in [(0, 1.0 - fy), (1, fy)] {
            for (dx, wx) in [(0, 1.0 - fx), (1, fx)] {
                let index = ((iy + dy).min(self.height - 1) * self.width
                    + (ix + dx).min(self.width - 1))
                    * 4;
                for (c, v) in value.iter_mut().enumerate() {
                    *v += f32::from(self.pixels[index + c]) * wx * wy;
                }
            }
        }
        value
    }
}

pub(crate) fn rasterize(
    rgba: &[u8],
    source: (u32, u32),
    rectangle: (f64, f64, f64, f64),
) -> Option<DeviceRaster> {
    let (x, y, w, h) = rectangle;
    let count = u64::from(source.0).checked_mul(u64::from(source.1))?;
    if source.0 == 0
        || source.1 == 0
        || count > MAX_PIXELS
        || rgba.len() != usize::try_from(count.checked_mul(4)?).ok()?
        || ![x, y, w, h].into_iter().all(f64::is_finite)
        || w <= 0.0
        || h <= 0.0
    {
        return None;
    }
    let ratio = ((f64::from(source.0) / w).max(f64::from(source.1) / h)) as f32;
    // GPU와 같은 sharpen LOD의 반 레벨 offset을 사용한다. 확대는 원 painter가 소유한다.
    if ratio <= 2.0 || !ratio.is_finite() {
        return None;
    }
    let level = (ratio.log2() - 0.5).max(0.0);
    let last = 31 - (source.0.max(source.1)).leading_zeros();
    let upper_index = (level.floor() as u32).min(last);
    let lower_index = (upper_index + 1).min(last);
    let blend = if upper_index == lower_index {
        0.0
    } else {
        level - level.floor()
    };
    let left = x.floor();
    let top = y.floor();
    let right = (x + w).ceil();
    let bottom = (y + h).ceil();
    let width = right - left;
    let height = bottom - top;
    if left < i32::MIN as f64
        || top < i32::MIN as f64
        || right > i32::MAX as f64
        || bottom > i32::MAX as f64
        || width < 1.0
        || height < 1.0
        || width * height > MAX_PIXELS as f64
    {
        return None;
    }
    let mut premultiplied = rgba.to_vec();
    for pixel in premultiplied.chunks_exact_mut(4) {
        for c in 0..3 {
            let product = u32::from(pixel[c]) * u32::from(pixel[3]) + 128;
            pixel[c] = ((product + (product >> 8)) >> 8) as u8;
        }
    }
    let mut current = Level {
        width: source.0 as usize,
        height: source.1 as usize,
        pixels: premultiplied,
    };
    for _ in 0..upper_index {
        current = current.reduce();
    }
    let upper = current;
    let lower = if lower_index > upper_index {
        Some(upper.reduce())
    } else {
        None
    };
    let width = width as u32;
    let height = height as u32;
    let mut pixels = vec![0; width as usize * height as usize * 4];
    let (x, y, w, h) = (x as f32, y as f32, w as f32, h as f32);
    for row in 0..height {
        for col in 0..width {
            let px = left as f32 + col as f32;
            let py = top as f32 + row as f32;
            let coverage = ((px + 1.0).min(x + w) - px.max(x)).clamp(0.0, 1.0)
                * ((py + 1.0).min(y + h) - py.max(y)).clamp(0.0, 1.0);
            let tx = (px + 0.5 - x) / w;
            let ty = (py + 0.5 - y) / h;
            let a = upper.sample(
                tx * upper.width as f32 - 0.5,
                ty * upper.height as f32 - 0.5,
            );
            let b = lower.as_ref().map_or(a, |m| {
                m.sample(tx * m.width as f32 - 0.5, ty * m.height as f32 - 0.5)
            });
            let mut value = [0.0; 4];
            for c in 0..4 {
                value[c] = (a[c] * (1.0 - blend) + b[c] * blend) * coverage;
            }
            let offset = (row as usize * width as usize + col as usize) * 4;
            if value[3] > 0.0 {
                for c in 0..3 {
                    pixels[offset + c] =
                        (value[c] * 255.0 / value[3]).round().clamp(0.0, 255.0) as u8;
                }
            }
            pixels[offset + 3] = value[3].round().clamp(0.0, 255.0) as u8;
        }
    }
    Some(DeviceRaster {
        left: left as i32,
        top: top as i32,
        width,
        height,
        rgba: pixels,
    })
}
