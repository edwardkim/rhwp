//! 큰 연산자의 표시 값을 측정과 paint에 공통으로 발행한다.

use super::{
    estimate_text_width, is_integral_symbol, EqLayout, EqNode, LayoutBox, LayoutKind, AXIS_HEIGHT,
    BIG_OP_SCALE, BIG_OP_TRAIL_PAD, INTEGRAL_SCALE,
};

impl EqLayout {
    pub(super) fn layout_big_op(
        &self,
        symbol: &str,
        sub: &Option<Box<EqNode>>,
        sup: &Option<Box<EqNode>>,
        fs: f64,
    ) -> LayoutBox {
        // 적분 기호: nolimits 스타일 (큰 기호 + 오른쪽 위/아래 첨자)
        if is_integral_symbol(symbol) {
            return self.layout_integral(symbol, sub, sup, fs);
        }
        if self.hancom_eqn && symbol == "∑" {
            if let Some(value) = self.layout_hancom_sum(sub, sup, fs) {
                return value;
            }
        }
        // ∑, ∏ 등: limits 스타일 (위/아래 중앙)
        // 원본 수식 face의 큰 합·곱은 세로 범위가 같은 형제 글리프(U+E067·U+E068,
        // 2..702/1024, 두 face 동일)를 1.8em으로 그린다. 한글 2022 4645 Print의 곱도
        // 본문 글자의 150/83배다. 합의 원본 face 기하를 같은 프레임을 쓰는 곱에도 쓴다.
        let source_sum = (self.hancom_math_face && symbol == "∑")
            || (self.source_limit_face() && matches!(symbol, "∑" | "∏"));
        let op_fs = if source_sum {
            fs * 1.8 * 700.0 / 1024.0
        } else {
            fs * BIG_OP_SCALE
        };
        // 표시 글리프는 공개 글꼴이지만 전진은 원본 PUA의 GDI 전진을 쓴다(곱의 공개 전진
        // 944/1000은 U+E068보다 좁아 그대로 두면 뒤 글자와 겹친다). 공개 글리프는 원본
        // PUA처럼(잉크 좌우 여백이 거의 같다) 그 칸의 가운데에 둔다.
        let glyph_w = estimate_text_width(symbol, op_fs, false);
        let op_w = source_sum
            .then(|| symbol.chars().next())
            .flatten()
            .and_then(|ch| super::super::hancom_glyphs::big_operator_advance(ch, fs, self.dpi))
            .unwrap_or(glyph_w);
        let op_h = op_fs;

        let sub_box = sub
            .as_ref()
            .map(|s| self.layout_node(s, self.script_font_size(fs)));
        let sup_box = sup
            .as_ref()
            .map(|s| self.layout_node(s, self.script_font_size(fs)));

        let sup_h = sup_box
            .as_ref()
            .map(|b| b.height + fs * 0.05)
            .unwrap_or(0.0);
        let sub_h = sub_box
            .as_ref()
            .map(|b| b.height + fs * 0.05)
            .unwrap_or(0.0);

        let total_h = sup_h + op_h + sub_h;
        let max_w = [
            op_w,
            sub_box.as_ref().map(|b| b.width).unwrap_or(0.0),
            sup_box.as_ref().map(|b| b.width).unwrap_or(0.0),
        ]
        .iter()
        .copied()
        .fold(0.0f64, f64::max);

        // 표시 글리프의 축과 원본 논리 축의 차이를 부모 기준선에 반영한다.
        // 기호만 이동하지 않고 위·아래 첨자도 같은 부모 원점을 소비한다.
        let source_axis = self.math_axis_height(fs);
        let baseline = sup_h
            + op_h * 0.6
            + if source_sum {
                source_axis - fs * AXIS_HEIGHT
            } else {
                0.0
            };

        let sup_laid = sup_box.map(|mut b| {
            b.x = (max_w - b.width) / 2.0;
            b.y = 0.0;
            b
        });
        let sub_laid = sub_box.map(|mut b| {
            b.x = (max_w - b.width) / 2.0;
            b.y = sup_h + op_h;
            b
        });

        let op_baseline = if source_sum {
            baseline + op_fs * AXIS_HEIGHT - source_axis
        } else {
            sup_h + op_fs * 0.8
        };
        // 합 기호의 실제 공개 글리프 bounds는 ascent 0.75, descent 0.25 em이다.
        // 위첨자가 없을 때도 실제 ink를 숨기지 않고 필요한 머리 공간을 예약한다.
        let head = if source_sum {
            (op_fs * 0.75 - op_baseline).max(0.0)
        } else {
            0.0
        };
        let sup_laid = sup_laid.map(|mut value| {
            value.y += head;
            value
        });
        let sub_laid = sub_laid.map(|mut value| {
            value.y += head;
            value
        });
        let value = LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            // 정상 한컴의 합은 첨자 상자와 다음 피연산자의 전진을 그대로
            // 잇는다. 제거된 가로 압축을 보상하던 여백을 다시 예약하지 않는다.
            // 다른 표시 계약은 기존 trailing을 유지한다.
            width: max_w
                + if source_sum {
                    0.0
                } else {
                    fs * BIG_OP_TRAIL_PAD
                },
            height: if source_sum {
                (total_h + head).max(op_baseline + head + op_fs * 0.25)
            } else {
                total_h
            },
            baseline: baseline + head,
            kind: LayoutKind::BigOp {
                symbol: symbol.to_string(),
                op_font_size: op_fs,
                op_width: glyph_w,
                op_x: (max_w - glyph_w) / 2.0,
                op_y: 0.0,
                op_baseline: op_baseline + head,
                sub: sub_laid.map(Box::new),
                sup: sup_laid.map(Box::new),
            },
        };
        self.with_hancom_limit_frame(value, max_w, fs)
    }

    /// 한컴 합·곱 프레임과 원본 face 연산자 기하를 함께 쓰는 수식 face.
    /// 둘이 같은 조건을 써야 프레임의 연산자 칸과 실제 연산자 글리프가 어긋나지 않는다.
    fn source_limit_face(&self) -> bool {
        (self.hancom_math_face || self.hancom_eqn)
            && !self.legacy_math_face
            && !self.legacy_hft_metrics
    }

    /// 원본 수식 face의 합·곱 프레임(`hancom_frame::limit_operator`)을 측정과 표시에 쓴다.
    /// 원본 face의 합은 뒤 피연산자를 바로 잇는다는 기존 계약을 곱에도 같이 적용해
    /// trailing 여백을 두지 않는다. 연산자 글리프는 `layout_big_op`의 원본 face 기하
    /// (크기·폭·본문 기준선에 대한 위치)를 그대로 쓴다. 한 줄이 아닌 한계값(분수·첨자
    /// 겹침)은 이 프레임으로 접지 않는다.
    fn with_hancom_limit_frame(&self, mut value: LayoutBox, max_w: f64, fs: f64) -> LayoutBox {
        if !self.source_limit_face() {
            return value;
        }
        let LayoutKind::BigOp {
            symbol,
            op_baseline,
            sub,
            sup,
            ..
        } = &mut value.kind
        else {
            return value;
        };
        let line = |limit: &Option<Box<LayoutBox>>| {
            limit.as_deref().is_none_or(|limit| {
                super::super::logical_font_frame::font_line(limit, limit.font_size)
            })
        };
        if !matches!(symbol.as_str(), "∑" | "∏") || !line(sub) || !line(sup) {
            return value;
        }
        let (height, baseline, op_height, limit_height) =
            super::super::hancom_frame::limit_operator(
                fs,
                self.script_font_size(fs),
                self.dpi,
                sup.is_some(),
                sub.is_some(),
            );
        let sup_height = if sup.is_some() { limit_height } else { 0.0 };
        *op_baseline += baseline - value.baseline;
        if let Some(limit) = sup {
            limit.y = (limit_height - limit.height) / 2.0;
        }
        if let Some(limit) = sub {
            limit.y = sup_height + op_height + (limit_height - limit.height) / 2.0;
        }
        value.width = max_w;
        value.height = height;
        value.baseline = baseline;
        value
    }

    /// 적분 기호 레이아웃: 큰 기호 + 오른쪽 위/아래 첨자 (nolimits 스타일)
    pub(super) fn layout_integral(
        &self,
        symbol: &str,
        sub: &Option<Box<EqNode>>,
        sup: &Option<Box<EqNode>>,
        fs: f64,
    ) -> LayoutBox {
        let base = EqNode::MathSymbol(symbol.to_string());
        let placed = self.integral_placement(&base, sub.as_deref(), sup.as_deref(), fs);
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: placed.width,
            height: placed.height,
            baseline: placed.baseline,
            kind: LayoutKind::BigOp {
                symbol: symbol.to_string(),
                op_font_size: fs * INTEGRAL_SCALE,
                op_width: placed.base.width,
                op_x: placed.base.x,
                op_y: placed.base.y,
                op_baseline: placed.baseline,
                sub: placed.sub.map(Box::new),
                sup: placed.sup.map(Box::new),
            },
        }
    }
}
