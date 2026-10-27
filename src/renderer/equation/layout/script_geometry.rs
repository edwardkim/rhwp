//! 위첨자의 기준선과 하단 여유를 같은 배치 결과로 계산한다.

use super::{is_integral_symbol, EqLayout, EqNode, LayoutBox, LayoutKind};

// 배포 Latin Modern Math의 MATH 상수(UPM 1,000). 문서 좌표가 아닌 em 비율이다.
const SUPERSCRIPT_SHIFT_UP: f64 = 0.363;
const SUPERSCRIPT_BOTTOM_MIN: f64 = 0.108;

/// 한컴의 Latin 대문자 뒤 위첨자는 정체/이탤릭 모두 0.15em을 예약한다.
/// 명시 공백 또는 닫는 구분자 뒤에는 이 문자 전진을 중복 추가하지 않는다.
fn ends_with_uppercase_letter(node: &EqNode) -> bool {
    match node.ungrouped() {
        EqNode::Text(text) => text
            .chars()
            .last()
            .is_some_and(|ch| ch.is_ascii_uppercase()),
        EqNode::FontStyle { body, .. } => ends_with_uppercase_letter(body),
        EqNode::Row(items) => items.last().is_some_and(ends_with_uppercase_letter),
        _ => false,
    }
}

impl EqLayout {
    /// 빈 본체도 첨자의 기준선 프레임을 소유한다. 잉크와 전진폭은 없다.
    /// 일반 Empty나 공개 LaTeX의 배치를 빈 글줄로 바꾸지 않는다.
    pub(super) fn layout_script_base(&self, base: &EqNode, fs: f64) -> LayoutBox {
        let mut value = if self.hancom_math_face && matches!(base.ungrouped(), EqNode::Empty) {
            self.layout_row(&[], fs)
        } else {
            self.layout_node(base, fs)
        };
        value.width += self.script_base_operator_gap(base, fs);
        value
    }

    fn script_uppercase_gap(&self, base: &EqNode, fs: f64) -> f64 {
        if self.hancom_math_face && ends_with_uppercase_letter(base) {
            fs * 0.15
        } else {
            0.0
        }
    }

    pub(super) fn script_superscript_shift(
        &self,
        base: &LayoutBox,
        sup: &LayoutBox,
        fs: f64,
    ) -> f64 {
        self.hancom_math_face
            .then(|| super::super::logical_font_frame::superscript_shift(base, sup, fs, self.dpi))
            .flatten()
            .unwrap_or_else(|| {
                (fs * SUPERSCRIPT_SHIFT_UP)
                    .max(sup.height - sup.baseline + fs * SUPERSCRIPT_BOTTOM_MIN)
            })
    }

    pub(super) fn script_subscript_geometry(
        &self,
        base: &LayoutBox,
        sub: &LayoutBox,
        fs: f64,
    ) -> (f64, f64) {
        if self.hancom_eqn
            && super::super::logical_font_frame::font_line(base, fs)
            && super::super::logical_font_frame::font_line(sub, sub.font_size)
        {
            // 두 face·네 크기의 독립 Print에서 아래첨자의 기준선은 본체보다
            // 1/4em 낮다. 자식의 실제 글자 크기와 기준선을 그대로 소비한다.
            let y = base.baseline + fs * 0.25 - sub.baseline;
            return (y, base.height.max(y + sub.height));
        }
        let source = (self.hancom_math_face || self.hancom_eqn)
            .then(|| super::super::logical_font_frame::subscript_geometry(base, sub, fs, self.dpi))
            .flatten();
        let y = source.map_or(base.baseline * 0.4, |(y, _)| y);
        let height = source
            .map_or(base.height, |(_, height)| height)
            .max(y + sub.height);
        (y, height)
    }

    pub(super) fn layout_superscript(&self, base: &EqNode, sup: &EqNode, fs: f64) -> LayoutBox {
        if matches!(base.ungrouped(), EqNode::MathSymbol(symbol) if is_integral_symbol(symbol)) {
            return self
                .integral_placement(base, None, Some(sup), fs)
                .into_script(fs);
        }
        let mut base_box = self.layout_script_base(base, fs);
        let mut sup_box = self.layout_node(sup, self.script_font_size(fs));
        let letter_gap = self.script_uppercase_gap(base, fs);

        // 위첨자의 깊이를 포함해 본체 기준선 위에 하단 여유를 확보한다.
        // 키 큰 본체는 원래 기준선을 유지하므로 위첨자를 맞추려고 본체를 올리지 않는다.
        let shift = self.script_superscript_shift(&base_box, &sup_box, fs);
        let baseline = base_box.baseline.max(sup_box.baseline + shift);
        base_box.x = 0.0;
        base_box.y = baseline - base_box.baseline;
        sup_box.x = base_box.width + letter_gap;
        sup_box.y = baseline - shift - sup_box.baseline;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: sup_box.x + sup_box.width,
            height: (base_box.y + base_box.height).max(sup_box.y + sup_box.height),
            baseline,
            kind: LayoutKind::Superscript {
                base: Box::new(base_box),
                sup: Box::new(sup_box),
            },
        }
    }
    pub(super) fn layout_subscript(&self, base: &EqNode, sub: &EqNode, fs: f64) -> LayoutBox {
        if matches!(base.ungrouped(), EqNode::MathSymbol(symbol) if is_integral_symbol(symbol)) {
            return self
                .integral_placement(base, Some(sub), None, fs)
                .into_script(fs);
        }
        let b = self.layout_script_base(base, fs);
        let s = self.layout_node(sub, self.script_font_size(fs));

        let (sub_shift, total_h) = self.script_subscript_geometry(&b, &s, fs);

        let mut base_box = b;
        base_box.x = 0.0;
        base_box.y = 0.0;

        let mut sub_box = s;
        sub_box.x = base_box.width;
        sub_box.y = sub_shift;

        let total_w = base_box.width + sub_box.width;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: total_h,
            baseline: base_box.baseline,
            kind: LayoutKind::Subscript {
                base: Box::new(base_box),
                sub: Box::new(sub_box),
            },
        }
    }

    pub(super) fn layout_subsup(
        &self,
        base: &EqNode,
        sub: &EqNode,
        sup: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        if matches!(base.ungrouped(), EqNode::MathSymbol(symbol) if is_integral_symbol(symbol)) {
            return self
                .integral_placement(base, Some(sub), Some(sup), fs)
                .into_script(fs);
        }
        let b = self.layout_script_base(base, fs);
        let sb = self.layout_node(sub, self.script_font_size(fs));
        let sp = self.layout_node(sup, self.script_font_size(fs));

        let shift = self.script_superscript_shift(&b, &sp, fs);
        let sup_y = b.baseline - shift - sp.baseline;
        let letter_gap = self.script_uppercase_gap(base, fs);
        let (sub_y, sub_height) = self.script_subscript_geometry(&b, &sb, fs);
        let base_y = (-sup_y).max(-sub_y).max(0.0);

        let mut base_box = b;
        base_box.x = 0.0;
        base_box.y = base_y;

        let mut sup_box = sp;
        sup_box.x = base_box.width + letter_gap;
        sup_box.y = base_y + sup_y;

        let mut sub_box = sb;
        sub_box.x = base_box.width;
        sub_box.y = base_y + sub_y;
        let total_h = (base_y + sub_height)
            .max(base_box.y + base_box.height)
            .max(sup_box.y + sup_box.height)
            .max(sub_box.y + sub_box.height);

        let total_w = (sup_box.x + sup_box.width).max(sub_box.x + sub_box.width);

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: total_h,
            baseline: base_box.y + base_box.baseline,
            kind: LayoutKind::SubSup {
                base: Box::new(base_box),
                sub: Box::new(sub_box),
                sup: Box::new(sup_box),
            },
        }
    }
}
