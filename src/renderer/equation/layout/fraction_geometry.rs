//! 표시 글꼴의 MATH 메트릭으로 분자·분모의 기준선과 점유를 함께 계산한다.
//!
//! LatinModernMath-Regular.woff2, UPM 1000, SHA-256:
//! 1177d9025d1e797c54b3c7676976365abd90f903423091de7f0f4e7a4b372162.
//! 배치 계약: https://www.w3.org/TR/mathml-core/#fraction-with-nonzero-line-thickness

use super::LayoutBox;

fn hancom_position(
    numer: &mut LayoutBox,
    denom: &mut LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    let frame = super::super::hancom_frame::fraction(numer, denom, fs, dpi)?;
    let (line_height, line_baseline) = super::super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    let (_, numer_baseline) = super::super::hancom_frame::metrics(numer, fs, dpi)?;
    let (denom_height, denom_baseline) = super::super::hancom_frame::metrics(denom, fs, dpi)?;
    // 분자의 상행 확장과 분모의 하행 확장을 각자 소비한다. 장식·첨자의
    // 전체 높이를 같은 방향으로 더하면 기호의 본체 기준선까지 이동한다.
    let above = (numer_baseline - line_baseline).max(0.0);
    let below = (denom_height - denom_baseline - (line_height - line_baseline)).max(0.0);
    numer.y = line_baseline + above - numer.baseline;
    denom.y = frame.0 - line_height + line_baseline - below - denom.baseline;
    Some(frame)
}

fn source_position(
    numer: &mut LayoutBox,
    denom: &mut LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    let (height, baseline) =
        super::super::fraction_sup_frame::source_fraction_frame(numer, denom, fs, dpi)?;
    let above = super::super::fraction_sup_frame::line_extension(numer, fs, dpi)?;
    let (line_height, line_baseline) = super::super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    // 논리 글줄의 기준선과 자식 glyph의 자연 기준선을 분리한다.
    // 측정에 쓴 독립 분수 높이의 첫/마지막 글줄에 실제 표시도 배치한다.
    numer.y = line_baseline + above - numer.baseline;
    denom.y = height - line_height + line_baseline - denom.baseline;
    Some((height, baseline))
}

pub(super) fn position(
    numer: &mut LayoutBox,
    denom: &mut LayoutBox,
    fs: f64,
    axis: f64,
) -> (f64, f64) {
    let half_rule = fs * super::FRAC_LINE_THICK * 0.5;
    let gap = fs * 120.0 / 1000.0;
    // FractionNumeratorDisplayStyleShiftUp와 FractionDenominatorDisplayStyleShiftDown.
    // 복합 분수에서도 자식의 기준선·하강부를 소비해 괘선과 겹치지 않게 한다.
    let numer_shift =
        (fs * 677.0 / 1000.0).max(axis + half_rule + gap + numer.height - numer.baseline);
    let denom_shift = (fs * 686.0 / 1000.0).max(half_rule + gap + denom.baseline - axis);
    let ascent = (numer_shift + numer.baseline)
        .max(denom.baseline - denom_shift)
        .max(axis + half_rule);
    let descent = (numer.height - numer.baseline - numer_shift)
        .max(denom_shift + denom.height - denom.baseline)
        .max(half_rule - axis)
        .max(0.0);
    numer.y = ascent - numer_shift - numer.baseline;
    denom.y = ascent + denom_shift - denom.baseline;
    (ascent + descent, ascent)
}

use super::{
    EqLayout, EqNode, LayoutKind, AXIS_HEIGHT, FRAC_LINE_PAD, RADICAL_DEGREE_SCALE, SQRT_PAD,
};

impl EqLayout {
    pub(super) fn layout_fraction(&self, numer: &EqNode, denom: &EqNode, fs: f64) -> LayoutBox {
        if self.legacy_hft_metrics {
            return self.layout_legacy_fraction(numer, denom, fs);
        }
        let n = self.layout_node(numer, fs);
        let d = self.layout_node(denom, fs);

        // 한컴이 새로 생성한 단순 분수는 반각 숫자와 양쪽 1/4 em을 예약한다.
        // 세로 기준선 간격과 수평 괘선 여백을 같은 임의 비율로 묶지 않는다.
        let pad = fs
            * if self.hancom_math_face || self.hancom_eqn {
                0.25
            } else {
                FRAC_LINE_PAD
            };
        let w = n.width.max(d.width) + pad * 2.0;

        let mut n_box = n;
        n_box.x = (w - n_box.width) / 2.0;
        let mut d_box = d;
        d_box.x = (w - d_box.width) / 2.0;
        let axis_height = self.math_axis_height(fs);
        let source = if self.hancom_eqn {
            hancom_position(&mut n_box, &mut d_box, fs, self.dpi)
        } else {
            self.hancom_math_face
                .then(|| source_position(&mut n_box, &mut d_box, fs, self.dpi))
                .flatten()
        };
        let (total_h, baseline) =
            source.unwrap_or_else(|| position(&mut n_box, &mut d_box, fs, axis_height));

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: total_h,
            baseline,
            kind: LayoutKind::Fraction {
                numer: Box::new(n_box),
                denom: Box::new(d_box),
                axis_height,
                // HancomEQN의 U+E06D bbox 높이는 UPM 1024에서 32다.
                rule_thickness: fs
                    * if self.hancom_eqn {
                        32.0 / 1024.0
                    } else {
                        super::FRAC_LINE_THICK
                    },
            },
        }
    }

    pub(super) fn layout_atop(&self, top: &EqNode, bottom: &EqNode, fs: f64) -> LayoutBox {
        if self.hancom_math_face {
            return self.layout_source_atop(top, bottom, fs);
        }
        let t = self.layout_node(top, fs);
        let b = self.layout_node(bottom, fs);

        let pad = fs * FRAC_LINE_PAD;
        let axis = fs * AXIS_HEIGHT;
        let w = t.width.max(b.width) + pad * 2.0;

        let top_h = t.height + pad;
        let bottom_h = b.height + pad;
        let baseline = top_h + axis;
        let total_h = top_h + bottom_h;

        let mut top_box = t;
        top_box.x = (w - top_box.width) / 2.0;
        top_box.y = pad;

        let mut bottom_box = b;
        bottom_box.x = (w - bottom_box.width) / 2.0;
        bottom_box.y = top_h;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: total_h,
            baseline,
            kind: LayoutKind::Atop {
                top: Box::new(top_box),
                bottom: Box::new(bottom_box),
            },
        }
    }

    pub(super) fn layout_sqrt(
        &self,
        index: &Option<Box<EqNode>>,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        let raw_body = self.layout_node(body, fs);
        let b = if self.hancom_math_face {
            self.matrix_row_box(raw_body, fs)
        } else {
            raw_body
        };
        let pad = fs * SQRT_PAD;
        // 수식 face의 논리 전진폭. HYhwpEQ의 수학 radical은 1em,
        // 배포 Latin Modern Math의 U+221A는 833/1000em이다.
        let sign_w = if self.hancom_math_face {
            super::source_roman_advance::em_advance(fs, self.dpi).unwrap_or(fs)
        } else {
            fs * 0.833
        };
        // 정상 한컴 Save/Print의 숫자·변수·분수·중첩 근호 70건에서
        // 기호의 논리 em과 별도로 0.17em의 끝 여백을 예약한다.
        // 세로 여백은 같은 가로 비율로 덮어쓰지 않는다.
        let right_pad = if self.hancom_math_face {
            fs * (0.17 + if ends_round_delimiter(&b) { 0.07 } else { 0.0 })
        } else {
            pad
        };
        let body_w = b.width + right_pad;
        let top_band = if self.hancom_math_face {
            super::super::radical_line_frame::top_band(&b, fs, self.dpi).unwrap_or_else(|| {
                super::super::font_frame_metrics::line_at_dpi(fs, self.dpi)
                    .map_or(fs, |(height, _)| height)
                    * 0.15
            })
        } else {
            pad
        };
        let body_h = b.height
            + if self.hancom_math_face {
                top_band
            } else {
                pad * 2.0
            };

        let idx = index.as_ref().map(|i| {
            let mut ib = self.layout_node(i, fs * RADICAL_DEGREE_SCALE);
            // 한컴 지수는 기호의 절반 슬롯에 오른쪽 정렬한다.
            // 일반 MATH의 before-degree kern은 별도 계약이다.
            ib.x = if self.hancom_math_face {
                (sign_w / 2.0 - ib.width).max(0.0)
            } else {
                fs * 0.278
            };
            // 한컴 Print의 근 지수 기준선은 근호 높이의 40%에 놓인다.
            // 일반 MATH 경로는 기존 하단 기준의 raise 계약을 유지한다.
            ib.y = body_h * 0.4
                - if self.hancom_math_face {
                    ib.baseline
                } else {
                    ib.height
                };
            ib
        });
        // 근 지수가 슬롯을 넘친 폭만 기호 앞에 추가한다. 근호의 닫는
        // 괄호 경계 여백도 이 예약에 포함하며 각 backend는 같은 원점을 쓴다.
        let degree_extension = idx
            .as_ref()
            .map(|i| {
                let clearance = if self.hancom_math_face && ends_round_delimiter(i) {
                    i.font_size * 0.07
                } else {
                    0.0
                };
                let overlap = if self.hancom_math_face {
                    sign_w / 2.0
                } else {
                    fs * 0.556
                };
                (i.x + i.width + clearance - overlap).max(0.0)
            })
            .unwrap_or(0.0);
        let total_w = degree_extension + sign_w + body_w;

        let mut body_box = b;
        body_box.x = total_w - body_w;
        body_box.y = top_band;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: body_h,
            baseline: body_box.y + body_box.baseline,
            kind: LayoutKind::Sqrt {
                index: idx.map(Box::new),
                body: Box::new(body_box),
            },
        }
    }
}

/// 보이는 닫는 둥근 괄호와 근호 사이의 여백은 근호가 소유한다.
fn ends_round_delimiter(value: &LayoutBox) -> bool {
    match &value.kind {
        LayoutKind::Paren { right, .. } => right == ")",
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => {
            ends_round_delimiter(body)
        }
        LayoutKind::Row(items) => items.last().is_some_and(ends_round_delimiter),
        _ => false,
    }
}

impl EqLayout {
    fn layout_legacy_fraction(&self, numer: &EqNode, denom: &EqNode, fs: f64) -> LayoutBox {
        let n = self.layout_fraction_operand(numer, fs);
        let d = self.layout_fraction_operand(denom, fs);

        let pad = fs * FRAC_LINE_PAD;
        let side_pad = if self.legacy_hft_metrics {
            fs * 0.24
        } else {
            pad
        };
        let line_thick = fs * FRAC_LINE_THICK;
        let axis = fs * AXIS_HEIGHT;
        let w = n.width.max(d.width) + side_pad * 2.0;

        let numer_h = n.height + pad;
        let denom_h = d.height + pad;

        // TeX 방식: 분수선은 baseline에서 axis_height 위에 배치
        // baseline(상단에서) = 분자높이 + 분수선두께/2 + axis_height
        // 즉, 분수선 y = baseline - axis_height (상단 기준)
        let frac_line_from_top = numer_h + line_thick / 2.0;
        let baseline = frac_line_from_top + axis;
        let total_h = numer_h + line_thick + denom_h;

        let mut n_box = n;
        n_box.x = (w - n_box.width) / 2.0;
        n_box.y = pad;

        let mut d_box = d;
        d_box.x = (w - d_box.width) / 2.0;
        d_box.y = numer_h + line_thick;

        LayoutBox {

            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: total_h,
            baseline,
            kind: LayoutKind::Fraction {
                axis_height: axis,
                rule_thickness: line_thick,
                numer: Box::new(n_box),
                denom: Box::new(d_box),
            },
        }
    }

}
