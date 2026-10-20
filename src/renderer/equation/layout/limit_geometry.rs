//! 극한 이름과 하한의 글자 크기·기준선을 측정과 모든 paint에 함께 전달한다.

use super::{estimate_text_width, EqLayout, EqNode, LayoutBox, LayoutKind};

impl EqLayout {
    pub(super) fn layout_limit(
        &self,
        is_upper: bool,
        sub: &Option<Box<EqNode>>,
        fs: f64,
    ) -> LayoutBox {
        // HYhwpEQ의 독립 Print에서 이름은 BaseUnit의 1.2배, 하한은 0.68배다.
        // 다른 수식 face의 기존 크기 계약은 유지한다.
        let name_font_size = if self.hancom_math_face { fs * 1.2 } else { fs };
        let sub_font_size = self.script_font_size(fs);
        let name = if is_upper { "Lim" } else { "lim" };
        let baseline = name_font_size * 0.8;
        // upstream의 실제 한컴 이름 글리프도 같은 크기와 기준선을 소비한다.
        let name_box = self
            .layout_hancom_glyphs(
                name,
                super::super::hancom_glyphs::Role::Variable { italic: false },
                name_font_size,
                0.0,
            )
            .map(|mut name| {
                name.baseline = baseline;
                name
            });
        let name_w = name_box.as_ref().map_or_else(
            || estimate_text_width(name, name_font_size, false),
            |name| name.width,
        );
        let mut sub_box = sub.as_ref().map(|s| self.layout_node(s, sub_font_size));
        let width = name_w.max(sub_box.as_ref().map_or(0.0, |b| b.width));
        let height = if let Some(child) = sub_box.as_mut() {
            child.x = (width - child.width) / 2.0;
            child.y = if self.hancom_math_face {
                // 표시 글꼴 Latin Modern Math의 lim/Lim 글리프는 하강부가 0이다.
                // MATH LowerLimitGapMin=167, LowerLimitBaselineDropMin=600 / UPM 1000.
                // 복합 하한은 같은 자식 기준선을 소비해 필요한 공간을 예약한다.
                let drop = (fs * 0.6).max(fs * 0.167 + child.baseline);
                baseline + drop - child.baseline
            } else {
                name_font_size
            };
            name_font_size.max(child.y + child.height + fs * 0.05)
        } else {
            name_font_size
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: height.max(name_box.as_ref().map_or(0.0, |name| name.height)),
            baseline,
            kind: LayoutKind::Limit {
                is_upper,
                name: name_box.map(Box::new),
                name_font_size,
                sub_font_size,
                sub: sub_box.map(Box::new),
            },
        }
    }
}
