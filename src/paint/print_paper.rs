//! 2024 Print: 원본 HU 용지 → 참조 DC → EMF frame → 물리 PDF 용지.
//!
//! DEVMODE의 용지는 0.1mm, ENHMETAHEADER frame은 0.01mm다.
//! 장치 픽셀과 정수 mm를 따로 반올림하며, raster 출력 DPI와 구분한다.

#[derive(Clone, Copy)]
pub(super) struct PrintPaper {
    pub size: (f64, f64),
    pub device_units: (f64, f64),
}

#[derive(Clone, Copy)]
struct Axis {
    points: u64,
    reference_pixels: u64,
    reference_mm: u64,
    frame: u64,
}

fn rounded_ratio(numerator: u64, denominator: u64) -> Option<u64> {
    Some(numerator.checked_add(denominator / 2)? / denominator)
}

pub(super) fn reference_geometry(tenth_mm: u64) -> Option<(u64, u64)> {
    Some((
        rounded_ratio(tenth_mm.checked_mul(600)?, 254)?,
        rounded_ratio(tenth_mm, 10)?,
    ))
}

impl Axis {
    fn from_hu(hu: u64) -> Option<Self> {
        let device = rounded_ratio(hu.checked_mul(600)?, 7200)?;
        let tenth_mm = rounded_ratio(hu.checked_mul(254)?, 7200)?;
        let (reference_pixels, reference_mm) = reference_geometry(tenth_mm)?;
        if device == 0 || reference_pixels == 0 || reference_mm == 0 {
            return None;
        }
        let frame = device.checked_mul(reference_mm)?.checked_mul(100)? / reference_pixels;
        let points = frame.checked_mul(72)? / 2540;
        (frame > 0 && points > 0).then_some(Self {
            points,
            reference_pixels,
            reference_mm,
            frame,
        })
    }

    fn from_css(css: f64) -> Option<Self> {
        let hu = (css * 7200.0 / 96.0).round();
        // 원본 정수 HU로 표현되는 용지만 이 장치 계약에 들어온다.
        // 합성·연속 크기를 가까운 용지로 바꾸거나 비정상 크기를 포화하지 않는다.
        if !css.is_finite()
            || hu <= 0.0
            || hu > f64::from(i32::MAX)
            || (hu * 96.0 / 7200.0).to_bits() != css.to_bits()
        {
            return None;
        }
        Self::from_hu(hu as u64)
    }

    fn size(self) -> f64 {
        self.points as f64 * (96.0 / 72.0)
    }

    fn device_unit(self) -> f64 {
        self.size() * (self.reference_mm * 100) as f64
            / self.reference_pixels as f64
            / self.frame as f64
    }
}

impl PrintPaper {
    pub fn from_css(width: f64, height: f64) -> Option<Self> {
        let x = Axis::from_css(width)?;
        let y = Axis::from_css(height)?;
        Some(Self {
            size: (x.size(), y.size()),
            device_units: (x.device_unit(), y.device_unit()),
        })
    }
}
