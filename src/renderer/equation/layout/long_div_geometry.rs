//! 긴 나눗셈의 제수·몫·피제수와 괘선을 한 공통 배치로 발행한다.
//!
//! 독립 한컴 Print의 숫자/문자/복합 피제수 대조에 근거한다.
//! 몫은 피제수 열의 오른쪽에 정렬하며, 두 줄의 논리 높이와 괘선 여백을 예약한다.

use super::{EqLayout, EqNode, LayoutBox, LayoutKind};
use crate::renderer::equation::GlyphOutlineCommand as Command;

impl EqLayout {
    pub(super) fn layout_long_div(
        &self,
        divisor: &EqNode,
        quotient: &EqNode,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        let mut divisor = self.matrix_row_box(self.layout_node(divisor, fs), fs);
        let mut quotient = self.matrix_row_box(self.layout_node(quotient, fs), fs);
        let mut body = self.matrix_row_box(self.layout_node(body, fs), fs);
        let upper_height = quotient.height;
        let lower_baseline = divisor.baseline.max(body.baseline);
        let lower_height =
            lower_baseline + (divisor.height - divisor.baseline).max(body.height - body.baseline);
        let gap = fs * 0.2;
        let lower_y = upper_height + gap;
        let column_width = quotient.width.max(body.width);
        let column_x = divisor.width + fs * 0.4;
        let width = column_x + column_width + fs * 0.2;
        let height = lower_y + lower_height;
        let (line_height, line_baseline) =
            super::super::font_frame_metrics::line_at_dpi(fs, self.dpi).unwrap_or((fs, fs * 0.86));
        // 긴 나눗셈은 가운데 글줄 축을 전체 상자로 옮긴 정수 % 기준선을 쓴다.
        // 독립 세 저장본은 2.2em 높이에 66%를 발행한다. 행렬 피치의 올림을
        // 재사용하면 서로 다른 구조의 기준선 계약을 섞게 된다.
        let baseline_percent =
            ((height / 2.0 + line_baseline - line_height / 2.0) / height * 100.0).round();
        let baseline = height * baseline_percent / 100.0;
        divisor.y = lower_y + lower_baseline - divisor.baseline;
        quotient.x = column_x + column_width - quotient.width;
        body.x = column_x;
        body.y = lower_y + lower_baseline - body.baseline;

        // 일반 곡선과 수평선으로 구성하며 사설 글리프 윤곽을 복제하지 않는다.
        let left = divisor.width + fs * 0.05;
        let top = upper_height + fs * 0.06;
        let bottom = lower_y + lower_height;
        let bow = fs * 0.28;
        let thickness = fs * 0.05;
        let mut outline = vec![
            Command::Move((left, top)),
            Command::Cubic([
                (left + bow, top + (bottom - top) / 3.0),
                (left + bow, bottom - (bottom - top) / 3.0),
                (left, bottom),
            ]),
            Command::Line((left + thickness, bottom)),
            Command::Cubic([
                (left + bow + thickness, bottom - (bottom - top) / 3.0),
                (left + bow + thickness, top + (bottom - top) / 3.0),
                (left + thickness, top),
            ]),
            Command::Close,
        ];
        let right = column_x + column_width + fs * 0.1;
        outline.extend([
            Command::Move((left, top)),
            Command::Line((right, top)),
            Command::Line((right, top + thickness)),
            Command::Line((left, top + thickness)),
            Command::Close,
        ]);
        let rule = LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline,
            kind: LayoutKind::FontGlyph {
                text: String::new(),
                font_family: "serif",
                paint: None,
                outline: Some(outline),
            },
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline,
            kind: LayoutKind::Row(vec![divisor, quotient, body, rule]),
        }
    }
}
