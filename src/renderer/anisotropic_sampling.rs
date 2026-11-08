//! 가로·세로 확대와 축소가 섞인 그림을 축별로 저역 통과한다.
//! 좌표는 현재 CTM의 장치 좌표다. 원본 RGB에 alpha를 먼저 곱하고 마지막에만 되돌린다.

pub(crate) struct DeviceRaster {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn mixed_reduction(source: (f64, f64), device: (f64, f64)) -> bool {
    [source.0, source.1, device.0, device.1]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
        && ((device.0 >= source.0 && device.1 < source.1)
            || (device.1 >= source.1 && device.0 < source.0))
}

fn axis(
    origin: f64,
    extent: f64,
    source: u32,
    start: f64,
    count: u32,
) -> Option<Vec<Vec<(usize, f64)>>> {
    let radius = (f64::from(source) / extent).max(1.0);
    if radius * f64::from(count) > 1_048_576.0 {
        return None;
    }
    Some(
        (0..count)
            .map(|i| {
                let position =
                    (start + f64::from(i) + 0.5 - origin) * f64::from(source) / extent - 0.5;
                let mut weights = Vec::new();
                let mut sum = 0.0;
                for index in (position - radius).ceil() as i64..=(position + radius).floor() as i64
                {
                    let value = (1.0 - (index as f64 - position).abs() / radius).max(0.0);
                    if value > 0.0 {
                        weights.push((index.clamp(0, i64::from(source) - 1) as usize, value));
                        sum += value;
                    }
                }
                for (_, value) in &mut weights {
                    *value /= sum;
                }
                weights
            })
            .collect(),
    )
}

pub(crate) fn rasterize(
    rgba: &[u8],
    source: (u32, u32),
    destination: (f64, f64, f64, f64),
) -> Option<DeviceRaster> {
    let (x, y, w, h) = destination;
    if !x.is_finite()
        || !y.is_finite()
        || !mixed_reduction((f64::from(source.0), f64::from(source.1)), (w, h))
        || rgba.len()
            != (source.0 as usize)
                .checked_mul(source.1 as usize)?
                .checked_mul(4)?
    {
        return None;
    }
    let (left, top) = (x.floor(), y.floor());
    let width = (x + w).ceil() - left;
    let height = (y + h).ceil() - top;
    if width <= 0.0 || height <= 0.0 || width * height > 4_194_304.0 {
        return None;
    }
    let (width, height) = (width as u32, height as u32);
    let xs = axis(x, w, source.0, left, width)?;
    let ys = axis(y, h, source.1, top, height)?;
    let mut output = vec![0; width as usize * height as usize * 4];
    for (dy, yy) in ys.iter().enumerate() {
        for (dx, xx) in xs.iter().enumerate() {
            let mut color = [0.0; 4];
            for &(sy, wy) in yy {
                for &(sx, wx) in xx {
                    let p = &rgba[(sy * source.0 as usize + sx) * 4..][..4];
                    let a = f64::from(p[3]) / 255.0;
                    for c in 0..3 {
                        color[c] += f64::from(p[c]) * a * wx * wy;
                    }
                    color[3] += a * wx * wy;
                }
            }
            let px = left + dx as f64;
            let py = top + dy as f64;
            let coverage = ((px + 1.0).min(x + w) - px.max(x)).max(0.0)
                * ((py + 1.0).min(y + h) - py.max(y)).max(0.0);
            let target = &mut output[(dy * width as usize + dx) * 4..][..4];
            if color[3] > 0.0 {
                for c in 0..3 {
                    target[c] = (color[c] / color[3]).round().clamp(0.0, 255.0) as u8;
                }
            }
            target[3] = (color[3].clamp(0.0, 1.0) * coverage * 255.0).round() as u8;
        }
    }
    Some(DeviceRaster {
        rgba: output,
        width,
        height,
    })
}
