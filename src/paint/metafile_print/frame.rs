//! 원본 조판 프레임·600dpi 장치·최종 픽셀의 좌표 변환을 함께 소유한다.

use crate::renderer::{print_raster::PrintRasterFrame, render_tree::BoundingBox};

use super::xml::numbers;

pub(super) const ATTRIBUTE: &str = "data-rhwp-print-wmf-frame";

#[derive(Clone, Copy)]
pub(super) struct Frame {
    // PrintRasterFrame의 HU 용지와 장치 image/clip 사각형.
    values: [f64; 8],
    pub(super) bbox: BoundingBox,
    dpi: f64,
}

impl Frame {
    pub(super) fn new(
        page: (f64, f64),
        origin: (f64, f64),
        bbox: BoundingBox,
        dpi: f64,
    ) -> Option<Self> {
        Some(Self {
            values: PrintRasterFrame::new(page, origin, bbox, dpi)?.values(),
            bbox,
            dpi,
        })
    }

    pub(super) fn encode(self) -> String {
        std::iter::once(1.0)
            .chain(self.values)
            .chain([
                self.bbox.x,
                self.bbox.y,
                self.bbox.width,
                self.bbox.height,
                self.dpi,
            ])
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
    pub(super) fn read(value: &str) -> Option<Self> {
        let values = numbers::<14>(value)?;
        if values[0] != 1.0 || values[13] <= 0.0 || values[11] <= 0.0 || values[12] <= 0.0 {
            return None;
        }
        let source = PrintRasterFrame::from_values(values[1..9].try_into().ok()?)?;
        Some(Self {
            values: source.values(),
            bbox: BoundingBox::new(values[9], values[10], values[11], values[12]),
            dpi: values[13],
        })
    }

    pub(super) fn device_scale(self, viewport: (f64, f64)) -> Option<(f64, f64)> {
        if viewport.0 <= 0.0 || viewport.1 <= 0.0 {
            return None;
        }
        Some((self.values[4] / viewport.0, self.values[5] / viewport.1))
    }

    /// 논리 창을 PDF의 양수 X·아래 방향 Y 행렬과 원점으로 변환한다.
    pub(super) fn text_matrix(self, window: [f64; 8], compact: bool) -> Option<[f64; 4]> {
        // 뒤집기나 비단위 원본 viewport 배율은 기존 SVG의 변환 해석을 보존한다.
        if window[2] <= 0.0 || window[3] <= 0.0 || window[4..] != [1.0, 1.0, 0.0, 0.0] {
            return None;
        }
        let (pw, ph) = self.points();
        let (px, py) = (
            pw / (self.values[0] / 12.0).round(),
            ph / (self.values[1] / 12.0).round(),
        );
        let sx = self.values[4] / window[2];
        let sy = self.values[5] / window[3];
        let precision = if compact { 1000.0 } else { 1_000_000.0 };
        let decimal = |value: f64| (value * precision).round() / precision;
        Some([
            decimal(sx * px),
            decimal(sy * py),
            decimal((self.values[2] - window[0] * sx) * px),
            decimal(ph - (self.values[3] - window[1] * sy) * py),
        ])
    }

    fn points(self) -> (f64, f64) {
        (
            (self.values[0] / 100.0 + 1e-9).floor(),
            (self.values[1] / 100.0 + 1e-9).floor(),
        )
    }

    /// source SVG를 그리는 실제 page CTM. 클립 전역을 픽셀로 스냅하지 않는다.
    #[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
    pub(super) fn root_matrix(self, viewport: (f64, f64), matrix: [f64; 4]) -> Option<[f64; 4]> {
        if matrix.iter().any(|v| !v.is_finite())
            || matrix[0] <= 0.0
            || matrix[1] <= 0.0
            || viewport.0 <= 0.0
            || viewport.1 <= 0.0
        {
            return None;
        }
        Some([
            self.bbox.width / viewport.0 * matrix[0],
            self.bbox.height / viewport.1 * matrix[1],
            self.bbox.x * matrix[0] + matrix[2],
            self.bbox.y * matrix[1] + matrix[3],
        ])
    }

    #[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
    pub(super) fn text_device_matrix(
        self,
        pdf: [f64; 4],
        window: [f64; 8],
        matrix: [f64; 4],
    ) -> Option<[f64; 4]> {
        let (pw, ph) = self.points();
        if pw <= 0.0 || ph <= 0.0 || matrix[0] <= 0.0 || matrix[1] <= 0.0 {
            return None;
        }
        let (dx, dy) = (
            (pw * self.dpi / 72.0 * matrix[0]).ceil() / pw,
            (ph * self.dpi / 72.0 * matrix[1]).ceil() / ph,
        );
        Some([
            pdf[0] * dx,
            pdf[1] * dy,
            (pdf[2] + pdf[0] * window[0]) * dx + matrix[2],
            (ph - pdf[3] + pdf[1] * window[1]) * dy + matrix[3],
        ])
    }

    pub(super) fn nested_raster(
        self,
        rect: [f64; 4],
        viewport: (f64, f64),
    ) -> Option<PrintRasterFrame> {
        let (sx, sy) = self.device_scale(viewport)?;
        let l = self.values[2] + rect[0] * sx;
        let t = self.values[3] + rect[1] * sy;
        let w = rect[2] * sx;
        let h = rect[3] * sy;
        PrintRasterFrame::from_values([self.values[0], self.values[1], l, t, w, h, l + w, t + h])
    }

    #[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
    pub(super) fn raster_matrix(self, matrix: [f64; 4]) -> [f64; 4] {
        [
            matrix[0] * self.dpi / 96.0,
            matrix[1] * self.dpi / 96.0,
            matrix[2],
            matrix[3],
        ]
    }
}
