//! 한컴 Print의 600dpi 그림 프레임과 정수 표본 재구성.
//!
//! 모델 좌표를 바꾸지 않고 참조 프레임·상대 위치를 각각 장치 좌표로 반올림한다.
//! 독립 용지/원점/여백 표본으로 확인한 공통 규칙이다. 재구성은 PDFium의
//! CStretchEngine(Chromium 7999)의 16.16 면적/선형 표본 계약을 따른다.
//! https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/dib/cstretchengine.cpp

use super::render_tree::BoundingBox;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PrintRasterFrame {
    // 원본 HU 용지와 600dpi 장치의 image/clip 프레임. 출력 DPI와 독립적이다.
    values: [f64; 8],
}

pub(crate) struct Raster {
    pub rgba: Vec<u8>,
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

impl PrintRasterFrame {
    pub(crate) fn new(
        page: (f64, f64),
        origin: (f64, f64),
        bbox: BoundingBox,
        dpi: f64,
    ) -> Option<Self> {
        if !dpi.is_finite() || dpi <= 0.0 {
            return None;
        }
        let hu = 7200.0 / dpi;
        let snap = |position: f64, origin: f64| {
            (origin * hu / 12.0).round() + ((position - origin) * hu / 12.0).round()
        };
        let values = [
            page.0 * hu,
            page.1 * hu,
            snap(bbox.x, origin.0),
            snap(bbox.y, origin.1),
            (bbox.width * hu / 12.0).round(),
            (bbox.height * hu / 12.0).round(),
            snap(bbox.x + bbox.width, origin.0),
            snap(bbox.y + bbox.height, origin.1),
        ];
        Self::from_values(values)
    }

    pub(crate) fn from_values(values: [f64; 8]) -> Option<Self> {
        (values
            .iter()
            .all(|v| v.is_finite() && v.abs() < f64::from(i32::MAX))
            && values[0] >= 100.0
            && values[1] >= 100.0
            && values[4] > 0.0
            && values[5] > 0.0
            && values[6] > values[2]
            && values[7] > values[3])
            .then_some(Self { values })
    }

    pub(crate) fn values(self) -> [f64; 8] {
        self.values
    }

    // 기존 rhWP sampling 선언 다음에 독립 APP15를 둔다. decoder는 이를 무시한다.
    pub(crate) fn attach(self, bytes: &mut Vec<u8>) {
        if !bytes.starts_with(&[0xff, 0xd8]) {
            return;
        }
        let mut payload = b"rhPF\x01".to_vec();
        for value in self.values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        payload.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
        let mut segment = vec![0xff, 0xef];
        segment.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        segment.extend_from_slice(&payload);
        let offset = if super::image_resolver::png_sampling::print_jpeg_transfer(bytes) {
            16
        } else {
            2
        };
        bytes.splice(offset..offset, segment);
    }

    pub(crate) fn read(bytes: &[u8]) -> Option<Self> {
        if !bytes.starts_with(&[0xff, 0xd8]) {
            return None;
        }
        let offset = if super::image_resolver::png_sampling::print_jpeg_transfer(bytes) {
            16
        } else {
            2
        };
        let segment = bytes.get(offset..offset + 77)?;
        if segment[..4] != [0xff, 0xef, 0, 75] || &segment[4..9] != b"rhPF\x01" {
            return None;
        }
        let checksum = u32::from_le_bytes(segment[73..77].try_into().ok()?);
        if crc32fast::hash(&segment[4..73]) != checksum {
            return None;
        }
        let mut values = [0.0; 8];
        for (index, value) in values.iter_mut().enumerate() {
            *value = f64::from_le_bytes(segment[9 + index * 8..17 + index * 8].try_into().ok()?);
        }
        Self::from_values(values)
    }

    pub(crate) fn rasterize(
        self,
        rgba: &[u8],
        source: (u32, u32),
        // positive axis-aligned scale and device translation of the logical page.
        matrix: [f64; 4],
    ) -> Option<Raster> {
        let [pw, ph, l, t, mw, mh, r, b] = self.values;
        if source.0 == 0
            || source.1 == 0
            || rgba.len()
                != (source.0 as usize)
                    .checked_mul(source.1 as usize)?
                    .checked_mul(4)?
            || matrix.iter().any(|v| !v.is_finite())
            || matrix[0] <= 0.0
            || matrix[1] <= 0.0
        {
            return None;
        }
        let (ptw, pth) = ((pw / 100.0 + 1e-9).floor(), (ph / 100.0 + 1e-9).floor());
        let (dw, dh) = (
            (ptw / 0.75 * matrix[0]).ceil(),
            (pth / 0.75 * matrix[1]).ceil(),
        );
        if dw > f64::from(i32::MAX) || dh > f64::from(i32::MAX) {
            return None;
        }
        let (sx, sy) = (ptw / (pw / 12.0).round(), pth / (ph / 12.0).round());
        // Print PDF 좌표는 소수 3자리, PDFium 행렬은 f32다. 양쪽 반올림을 보존한다.
        let decimal = |v: f64| ((v * 1000.0).round() / 1000.0) as f32;
        let (dx, dy) = ((dw / ptw) as f32, (dh / pth) as f32);
        let il = decimal(l * sx) * dx;
        let ib = dh as f32 - decimal(pth - (t + mh) * sy) * dy;
        let it = ib - decimal(mh * sy) * dy;
        let ir = il + decimal(mw * sx) * dx;
        let (ox, oy) = (il.floor(), it.floor());
        let (iw, ih) = ((ir.ceil() - ox) as u32, (ib.ceil() - oy) as u32);
        let cl = (decimal(l * sx) * dx).floor().max(ox);
        let ct = (dh as f32 - decimal(pth - t * sy) * dy).floor().max(oy);
        let cr = (decimal(r * sx) * dx).ceil().min(ox + iw as f32);
        let cb = (dh as f32 - decimal(pth - b * sy) * dy)
            .ceil()
            .min(oy + ih as f32);
        let (width, height) = ((cr - cl) as u32, (cb - ct) as u32);
        if iw == 0
            || ih == 0
            || width == 0
            || height == 0
            || u64::from(width) * u64::from(height) > 4_194_304
        {
            return None;
        }
        let bilinear =
            u64::from(ih / 8) < u64::from(source.0) * u64::from(source.1) / u64::from(iw);
        let xs = weights(source.0, iw, (cl - ox) as u32, width, bilinear)?;
        let ys = weights(source.1, ih, (ct - oy) as u32, height, bilinear)?;
        let temp_len = (source.1 as usize)
            .checked_mul(width as usize)?
            .checked_mul(4)?;
        if temp_len > 64 * 1024 * 1024 {
            return None;
        }
        let mut temp = vec![0u8; temp_len];
        for sy in 0..source.1 as usize {
            for (dx, weights) in xs.iter().enumerate() {
                for c in 0..4 {
                    let sum: u64 = weights
                        .iter()
                        .map(|&(sx, w)| {
                            u64::from(rgba[(sy * source.0 as usize + sx) * 4 + c]) * u64::from(w)
                        })
                        .sum();
                    temp[(sy * width as usize + dx) * 4 + c] = (sum >> 16).min(255) as u8;
                }
            }
        }
        let mut out = vec![0u8; width as usize * height as usize * 4];
        for (dy, weights) in ys.iter().enumerate() {
            for x in 0..width as usize {
                for c in 0..4 {
                    let sum: u64 = weights
                        .iter()
                        .map(|&(sy, w)| {
                            u64::from(temp[(sy * width as usize + x) * 4 + c]) * u64::from(w)
                        })
                        .sum();
                    out[(dy * width as usize + x) * 4 + c] = (sum >> 16).min(255) as u8;
                }
            }
        }
        Some(Raster {
            rgba: out,
            left: cl as i32,
            top: ct as i32,
            width,
            height,
        })
    }
}

fn weights(
    source: u32,
    dest: u32,
    start: u32,
    count: u32,
    bilinear: bool,
) -> Option<Vec<Vec<(usize, u32)>>> {
    let scale = f64::from(source) / f64::from(dest);
    if scale.max(1.0) * f64::from(count) > 1_048_576.0 {
        return None;
    }
    let mut out = Vec::with_capacity(count as usize);
    for i in start..start.checked_add(count)? {
        if scale < 1.0 {
            let p = (f64::from(i) + 0.5) * scale;
            if !bilinear {
                out.push(vec![((p.floor() as usize).min(source as usize - 1), 65536)]);
                continue;
            }
            let l = (p - 0.5).floor().max(0.0) as usize;
            let r = (p + 0.5).floor().min(f64::from(source) - 1.0) as usize;
            if l >= r {
                out.push(vec![(l, 65536)]);
                continue;
            }
            let b = ((p - l as f64 - 0.5) * 65536.0).round() as u32;
            out.push(vec![(l, 65536 - b), (l + 1, b)]);
        } else {
            let a = f64::from(i) * scale;
            let b = a + scale;
            let first = a.floor() as usize;
            let last = (b.floor() as usize).min(source as usize - 1);
            let mut weights = Vec::new();
            let mut remain = 65536i64;
            let mut error = 0.0;
            for j in first..last {
                let w = ((j as f64 + 1.0) / scale).min(f64::from(i) + 1.0)
                    - (j as f64 / scale).max(f64::from(i));
                let raw = ((w.max(0.0) + error) * 65536.0).round().max(0.0) as u32;
                weights.push((j, raw));
                remain -= i64::from(raw);
                error = w - f64::from(raw) / 65536.0;
            }
            if remain > 0 {
                weights.push((last, remain as u32));
            } else if let Some((_, w)) = weights.last_mut() {
                *w = (i64::from(*w) + remain).try_into().ok()?;
            }
            out.push(weights);
        }
    }
    Some(out)
}
