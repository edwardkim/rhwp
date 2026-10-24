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

fn reference_geometry(tenth_mm: u64) -> Option<(u64, u64)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_print_paper_and_integer_device_origins() {
        // 별도 한컴 Print 4개: 용지 높이와 원점 6000/15000/30000 HU.
        // PDF 연산자의 0.001pt 정밀도로 비교하며 엔진 출력에서 기대값을 얻지 않는다.
        for (hu, points, ys) in [
            (75400, 754, [60.003, 150.008, 300.016]),
            (75408, 754, [59.994, 149.985, 299.971]),
            (75420, 753, [59.906, 149.766, 299.531]),
            (75500, 754, [59.918, 149.795, 299.589]),
        ] {
            let axis = Axis::from_hu(hu).unwrap();
            assert_eq!(axis.points, points);
            for (device_y, expected_pt) in [500.0, 1250.0, 2500.0].into_iter().zip(ys) {
                assert!((device_y * axis.device_unit() * 0.75 - expected_pt).abs() < 0.001);
            }
        }
    }

    #[test]
    fn reference_dc_half_millimeter_boundaries() {
        // 표준 GDI GetDeviceCaps, Hancom PDF, 메모리상의 DEVMODE에서 확인했다.
        // 장치 픽셀에서 mm를 역산하면 반 mm 경계에서 이 독립 관측과 다르다.
        for (dm, pixels, mm) in [
            (1954, 4616, 195),
            (1955, 4618, 196),
            (1956, 4620, 196),
            (1964, 4639, 196),
            (1965, 4642, 197),
            (1966, 4644, 197),
            (2654, 6269, 265),
            (2655, 6272, 266),
            (2656, 6274, 266),
            (2664, 6293, 266),
            (2665, 6295, 267),
            (2666, 6298, 267),
        ] {
            assert_eq!(reference_geometry(dm), Some((pixels, mm)));
        }
    }

    #[test]
    fn only_valid_source_hu_paper_is_projected() {
        let paper = PrintPaper::from_css(55559.0 * 96.0 / 7200.0, 75402.0 * 96.0 / 7200.0).unwrap();
        assert_eq!(paper.size.0 * 0.75, 555.0);
        assert_eq!(paper.size.1 * 0.75, 754.0);
        assert!((paper.device_units.1 * 0.75 - 0.11998832302863824).abs() < 1e-15);
        for css in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::MAX, 1.23456789] {
            assert!(PrintPaper::from_css(css, 100.0).is_none());
        }
    }
}
