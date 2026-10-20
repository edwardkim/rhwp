//! 적분의 글리프와 첨자가 공유하는 원점·점유 결과.

use super::{integral_geom, EqLayout, EqNode, LayoutBox, LayoutKind, BIG_OP_TRAIL_PAD};

pub(super) struct IntegralPlacement {
    pub base: LayoutBox,
    pub sub: Option<LayoutBox>,
    pub sup: Option<LayoutBox>,
    pub width: f64,
    pub height: f64,
    pub baseline: f64,
}

impl IntegralPlacement {
    pub fn into_script(self, fs: f64) -> LayoutBox {
        let kind = match (self.sub, self.sup) {
            (Some(sub), Some(sup)) => LayoutKind::SubSup {
                base: Box::new(self.base),
                sub: Box::new(sub),
                sup: Box::new(sup),
            },
            (Some(sub), None) => LayoutKind::Subscript {
                base: Box::new(self.base),
                sub: Box::new(sub),
            },
            (None, Some(sup)) => LayoutKind::Superscript {
                base: Box::new(self.base),
                sup: Box::new(sup),
            },
            (None, None) => self.base.kind,
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: self.width,
            height: self.height,
            baseline: self.baseline,
            kind,
        }
    }
}

impl EqLayout {
    pub(super) fn integral_placement(
        &self,
        base: &EqNode,
        sub: Option<&EqNode>,
        sup: Option<&EqNode>,
        fs: f64,
    ) -> IntegralPlacement {
        let geom = integral_geom(fs);
        let mut base = self.layout_node(base, fs);
        let mut sub = sub.map(|node| self.layout_node(node, self.script_font_size(fs)));
        let mut sup = sup.map(|node| self.layout_node(node, self.script_font_size(fs)));
        let sup_dy = sup.as_ref().map_or(0.0, |value| {
            if self.hancom_math_face {
                base.baseline - fs - value.baseline
            } else {
                geom.top_y - value.height * 0.30
            }
        });
        let head = (-sup_dy).max(0.0);
        base.x = 0.0;
        base.y = head;
        base.width = geom.width;
        let gap_x = fs * 0.22;
        if let Some(value) = sup.as_mut() {
            value.x = if self.hancom_math_face {
                fs * 1.3
            } else {
                geom.top_hook_x + gap_x
            };
            value.y = head + sup_dy;
        }
        if let Some(value) = sub.as_mut() {
            value.x = if self.hancom_math_face {
                fs * 0.75
            } else {
                geom.bottom_hook_x + gap_x
            };
            value.y = if self.hancom_math_face {
                // 독립 Print의 평범한 하한 기준선은 본체보다 0.9 em 낮다.
                // 복합 하한은 표시 글꼴 MATH SubscriptTopMax(344/1000)를 소비한다.
                head + base.baseline + (fs * 0.9).max(value.baseline - fs * 0.344) - value.baseline
            } else {
                head + geom.bottom_y - value.height * 0.72
            };
        }
        let right = [Some(&base), sub.as_ref(), sup.as_ref()]
            .into_iter()
            .flatten()
            .map(|value| value.x + value.width)
            .fold(0.0_f64, f64::max);
        let bottom = [Some(&base), sub.as_ref(), sup.as_ref()]
            .into_iter()
            .flatten()
            .map(|value| value.y + value.height)
            .fold(0.0_f64, f64::max);
        IntegralPlacement {
            baseline: head + base.baseline,
            base,
            sub,
            sup,
            width: right
                + if self.hancom_math_face {
                    0.0
                } else {
                    fs * BIG_OP_TRAIL_PAD
                },
            height: bottom,
        }
    }
}
