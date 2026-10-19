//! 한컴의 실제 합 글리프와 한계값의 논리 기준선.
//!
//! 두 face·네 크기의 독립 Save/Print(1607단계 근거)를 대조했다.
//! 아래 값은 BaseUnit별 글꼴 프레임·원점의 HWPUNIT이며 문서 좌표가 아니다.

use super::super::{font_frame_metrics, hancom_glyphs::Role, logical_font_frame};
use super::{EqLayout, EqNode, LayoutBox, LayoutKind};

// BaseUnit, 전체 H/BL%, 합 glyph/상한/하한 기준선의 본문 기준선 상대 HWPUNIT.
const FRAMES: [[i32; 6]; 4] = [
    [600, 1538, 64, 192, -624, 540],
    [900, 2438, 63, 240, -1008, 828],
    [1200, 3281, 63, 336, -1368, 1092],
    [1800, 4819, 64, 516, -2004, 1620],
];

impl EqLayout {
    pub(super) fn layout_hancom_sum(
        &self,
        sub: &Option<Box<EqNode>>,
        sup: &Option<Box<EqNode>>,
        fs: f64,
    ) -> Option<LayoutBox> {
        let unit = font_frame_metrics::base_unit_at_dpi(fs, self.dpi);
        let row = FRAMES.iter().find(|row| row[0] as u32 == unit)?;
        let mut lower = self.layout_node(sub.as_deref()?, self.script_font_size(fs));
        let mut upper = self.layout_node(sup.as_deref()?, self.script_font_size(fs));
        // 복합 분수·다줄 한계값의 점유를 단일 글줄 메트릭으로 접지 않는다.
        if !logical_font_frame::font_line(&lower, lower.font_size)
            || !logical_font_frame::font_line(&upper, upper.font_size)
        {
            return None;
        }
        let mut glyph = self.layout_hancom_glyphs("∑", Role::BigOperator, fs * 1.8, 0.0)?;
        let width = lower.width.max(upper.width).max(glyph.width);
        let height = crate::renderer::hwpunit_to_px(row[1], self.dpi);
        let baseline = height * f64::from(row[2]) / 100.0;
        for (child, offset) in [
            (&mut glyph, row[3]),
            (&mut upper, row[4]),
            (&mut lower, row[5]),
        ] {
            child.x = (width - child.width) / 2.0;
            child.y = baseline + crate::renderer::hwpunit_to_px(offset, self.dpi) - child.baseline;
        }
        Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline,
            kind: LayoutKind::Row(vec![glyph, upper, lower]),
        })
    }
}
