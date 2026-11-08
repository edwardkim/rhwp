//! 한컴 악센트의 점유 띠와 공개 기본 도형의 잉크 경계를 함께 발행한다.

use super::super::symbols::DecoKind;
use super::super::{font_frame_metrics, GlyphOutlineCommand as Command};
use super::{EqLayout, LayoutBox, LayoutKind};

impl EqLayout {
    pub(super) fn layout_hancom_hat(&self, body: LayoutBox, fs: f64) -> LayoutBox {
        let mut body = self.matrix_row_box(body, fs);
        let band = font_frame_metrics::line_at_dpi(fs, self.dpi)
            .zip(font_frame_metrics::accent_at_dpi(fs, self.dpi, false))
            .map_or(fs * 0.2, |(line, accent)| accent.0 - line.0);
        body.y = band;
        // Print의 HAT는 본체 폭으로 가로만 늘어난다. 실제 원점·bbox와
        // 중심을 공유하고 사설 글꼴의 윤곽 데이터는 복사하지 않는다.
        let left = body.width * 0.30078125 + fs * 0.05;
        let right = left + body.width * 582.0 / 1024.0;
        let middle = (left + right) / 2.0;
        let glyph_baseline = body.y + body.baseline;
        let top = glyph_baseline - fs * (0.35 + 712.0 / 1024.0);
        let bottom = glyph_baseline - fs * (0.35 + 556.0 / 1024.0);
        let thickness = fs * 0.03;
        let outline = vec![
            Command::Move((left, bottom - thickness)),
            Command::Line((middle, top)),
            Command::Line((right, bottom - thickness)),
            Command::Line((right, bottom)),
            Command::Line((middle, top + thickness)),
            Command::Line((left, bottom)),
            Command::Close,
        ];
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: body.width,
            height: body.height + band,
            baseline: body.y + body.baseline,
            kind: LayoutKind::Decoration {
                kind: DecoKind::Hat,
                body: Box::new(body),
                negation: None,
                outline: Some(outline),
            },
        }
    }
}
