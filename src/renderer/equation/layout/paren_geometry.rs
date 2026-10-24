//! 괄호 슬롯·본체 원점·논리 프레임을 한 배치에서 결정한다.
use super::{
    delimiter_advance, legacy_delimiter_width, source_font_advance, EqLayout, EqNode, LayoutBox,
    LayoutKind, PAREN_PAD,
};
impl EqLayout {
    pub(super) fn layout_paren(
        &self,
        left: &str,
        right: &str,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        if self.legacy_hft_metrics {
            return self.layout_legacy_paren(left, right, body, fs);
        }
        let b = self.layout_node(body, fs);
        let logical_frame = self
            .hancom_math_face
            .then(|| super::super::logical_font_frame::metrics_at_dpi(&b, fs, self.dpi))
            .flatten()
            .or_else(|| {
                // 한컴 ATOP이 발행한 상하 행은 이미 논리 프레임이다.
                // 구분자는 그 프레임을 소비하며 일반 캐시 정규화 범위를 넓히지 않는다.
                (self.hancom_math_face && matches!(b.kind, LayoutKind::Atop { .. }))
                    .then_some((b.height, b.baseline))
            });
        let delimiter_height = logical_frame.map_or(b.height, |(height, _)| height);
        let ordinary_height = logical_frame
            .and_then(|_| super::super::font_frame_metrics::line_at_dpi(fs, self.dpi))
            .map_or(fs * 1.2, |(height, _)| height);
        let glyph_delimiters = delimiter_height <= ordinary_height;
        let source_stretch = (!glyph_delimiters && logical_frame.is_some())
            .then(|| delimiter_advance::source_stretch(left, right, fs, self.dpi))
            .flatten();
        let use_stretch_round = !glyph_delimiters && matches!((left, right), ("(", ")"));
        let source_curly = self.hancom_math_face
            && matches!(left, "" | "{" | "}")
            && matches!(right, "" | "{" | "}");
        let source_round = self.hancom_math_face
            && !use_stretch_round
            && matches!(left, "" | "(" | ")")
            && matches!(right, "" | "(" | ")");
        let source_vertical = (self.hancom_math_face && left == "|" && right == "|")
            .then(|| source_font_advance::vertical_slot(fs, self.dpi))
            .flatten();
        let pad = if source_round
            || source_curly
            || source_stretch.is_some()
            || source_vertical.is_some()
        {
            0.0
        } else if use_stretch_round {
            fs * 0.03
        } else {
            fs * PAREN_PAD
        };
        // Times New Roman '(' advance (em 기준) = 0.333. 텍스트 높이 glyph는 이 폭을 유지하고,
        // 큰 둥근 괄호 path는 한컴 HyhwpEQ 출력에 가깝게 더 좁게 잡는다. (Task #283, #1139)
        let paren_w = if let Some(width) = source_vertical {
            width
        } else if source_curly {
            source_font_advance::curly_slot(fs, self.dpi)
        } else if source_round {
            source_font_advance::paren(fs, self.dpi)
        } else if use_stretch_round {
            fs * 0.27
        } else {
            fs * 0.333
        };

        let (left_w, right_w) = source_stretch.unwrap_or((
            if left.is_empty() { 0.0 } else { paren_w },
            if right.is_empty() { 0.0 } else { paren_w },
        ));

        let (height, baseline) = logical_frame.unwrap_or((b.height, b.baseline));
        let delimiter_height = b.height;
        let mut body_box = b;
        body_box.x = left_w + pad;
        body_box.y = baseline - body_box.baseline;

        let total_w = left_w + pad + body_box.width + pad + right_w;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: total_w,
            height,
            baseline,
            kind: LayoutKind::Paren {
                left: left.to_string(),
                right: right.to_string(),
                left_width: left_w,
                right_width: right_w,
                delimiter_height,
                glyph_delimiters,
                left_outline: self.delimiter_outline(
                    left,
                    left_w,
                    delimiter_height,
                    baseline,
                    fs,
                    glyph_delimiters,
                ),
                right_outline: self.delimiter_outline(
                    right,
                    right_w,
                    delimiter_height,
                    baseline,
                    fs,
                    glyph_delimiters,
                ),
                body: Box::new(body_box),
            },
        }
    }
}

impl EqLayout {
    fn layout_legacy_paren(&self, left: &str, right: &str, body: &EqNode, fs: f64) -> LayoutBox {
        let b = self.layout_node(body, fs);
        let use_stretch_round = b.height > fs * 1.2 && matches!((left, right), ("(", ")"));
        let legacy_delimiters = self.legacy_hft_metrics
            && legacy_delimiter_width(left, fs).is_some()
            && legacy_delimiter_width(right, fs).is_some();
        let pad = if legacy_delimiters {
            fs * 0.115
        } else if use_stretch_round {
            fs * 0.03
        } else {
            fs * PAREN_PAD
        };
        // Times New Roman '(' advance (em 기준) = 0.333. 텍스트 높이 glyph는 이 폭을 유지하고,
        // 큰 둥근 괄호 path는 한컴 HyhwpEQ 출력에 가깝게 더 좁게 잡는다. (Task #283, #1139)
        let paren_w = if use_stretch_round {
            fs * 0.27
        } else {
            fs * 0.333
        };

        let left_w = if left.is_empty() {
            0.0
        } else if legacy_delimiters {
            legacy_delimiter_width(left, fs).unwrap_or(paren_w)
        } else {
            paren_w
        };
        let right_w = if right.is_empty() {
            0.0
        } else if legacy_delimiters {
            legacy_delimiter_width(right, fs).unwrap_or(paren_w)
        } else {
            paren_w
        };
        let paint_width = if b.height <= fs * 1.2 {
            fs * 0.333
        } else {
            fs * 0.27
        };
        let left_width = if legacy_delimiters {
            left_w
        } else {
            paint_width
        };
        let right_width = if legacy_delimiters {
            right_w
        } else {
            paint_width
        };

        let delimiter_height = b.height;
        let mut body_box = b;
        body_box.x = left_w + pad;
        body_box.y = 0.0;

        let total_w = left_w + pad + body_box.width + pad + right_w;

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: body_box.height,
            baseline: body_box.baseline,
            kind: LayoutKind::Paren {
                delimiter_height,
                glyph_delimiters: delimiter_height <= fs * 1.2,
                left_outline: None,
                right_outline: None,
                left: left.to_string(),
                right: right.to_string(),
                body: Box::new(body_box),
                left_width,
                right_width,
            },
        }
    }
}
