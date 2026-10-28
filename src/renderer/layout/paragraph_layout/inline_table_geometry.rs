//! TAC 표 문단의 실제 배치에서 얻은 흐름 끝과 물리 점유 끝.

/// 정렬 폭·줄 나눔·실제 펜 전진이 함께 소비하는 문자 메트릭.
/// 글자별 정수 반올림은 공백이 누적될수록 표 원점을 이동시키므로 하지 않는다.
pub(super) struct InlineTableTextMetrics {
    advances: Vec<f64>,
    languages: Vec<usize>,
}

impl InlineTableTextMetrics {
    pub(super) fn new(
        chars: &[char],
        segments: &[(usize, usize)],
        mut advance: impl FnMut(usize, usize) -> f64,
    ) -> Self {
        let mut result = Self {
            advances: vec![0.0; chars.len()],
            languages: vec![0; chars.len()],
        };
        for &(start, end) in segments {
            let mut language = 0;
            for (index, &ch) in chars.iter().enumerate().take(end).skip(start) {
                if !crate::renderer::composer::is_lang_neutral(ch) {
                    language = crate::renderer::style_resolver::detect_lang_category(ch);
                }
                result.languages[index] = language;
                result.advances[index] = advance(index, language);
            }
        }
        result
    }

    pub(super) fn width(&self, start: usize, end: usize) -> f64 {
        self.advances[start..end].iter().sum()
    }

    pub(super) fn at(&self, index: usize) -> (usize, f64) {
        (self.languages[index], self.advances[index])
    }
}

/// 후행 간격을 포함한 전진과, 표·글줄이 점유하는 끝을 구분한다.
/// 페인트 원점을 이동해도 논리 흐름만 원래 문단 원점으로 되돌린다.
pub(crate) struct InlineTableParagraphGeometry {
    pub(crate) flow_bottom: f64,
    pub(crate) content_bottom: f64,
    pub(crate) line_box_bottom: f64,
}

impl InlineTableParagraphGeometry {
    pub(crate) fn at_flow_origin(mut self, paint_origin: f64, flow_origin: f64) -> Self {
        // 공백 캐리어의 기존 전진 계약을 유지한다. 내용 경계는 페인트 좌표다.
        self.flow_bottom = flow_origin + (self.flow_bottom - paint_origin).max(0.0);
        self.line_box_bottom += flow_origin - paint_origin;
        self
    }
}
