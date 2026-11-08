//! 인라인 표 문단의 측정·배치·glyph가 공유하는 원본 문자별 전진.

use super::super::text_measurement::{build_cluster_len, char_width_decision};
use super::{estimate_text_width_exact, TextStyle};
use crate::renderer::composer::is_lang_neutral;
use crate::renderer::style_resolver::detect_lang_category;

pub(super) struct InlineTextAdvance {
    widths: Vec<f64>,
    prefix: Vec<f64>,
    languages: Vec<usize>,
}

impl InlineTextAdvance {
    /// 기존 세그먼트의 언어 소유를 보존하고 공통 문자/클러스터 메트릭을 한 번만 잰다.
    pub(super) fn new(
        chars: &[char],
        segments: &[(usize, usize)],
        mut style_at: impl FnMut(usize, usize) -> TextStyle,
    ) -> Self {
        let cluster_len = build_cluster_len(chars);
        let mut widths = vec![0.0; chars.len()];
        let mut languages = vec![0; chars.len()];
        for &(start, end) in segments {
            let mut language = 0;
            for index in start..end {
                if !is_lang_neutral(chars[index]) {
                    language = detect_lang_category(chars[index]);
                }
                languages[index] = language;
                let style = style_at(index, language);
                widths[index] = if chars[index] == '\t' {
                    // 이 스타일은 원본 탭 순번부터의 저장 확장을 이미 가진다.
                    estimate_text_width_exact("\t", &style)
                } else {
                    char_width_decision(chars, &cluster_len, index, &style, true).final_width_px
                };
            }
        }
        let mut prefix = Vec::with_capacity(chars.len() + 1);
        prefix.push(0.0);
        for width in &widths {
            prefix.push(prefix.last().copied().unwrap_or(0.0) + width);
        }
        Self {
            widths,
            prefix,
            languages,
        }
    }

    pub(super) fn advance(&self, index: usize) -> f64 {
        self.widths[index]
    }

    pub(super) fn language(&self, index: usize) -> usize {
        self.languages[index]
    }

    pub(super) fn width(&self, start: usize, end: usize) -> f64 {
        self.prefix[end] - self.prefix[start]
    }

    pub(super) fn positions(&self, start: usize, end: usize) -> Vec<f64> {
        let origin = self.prefix[start];
        self.prefix[start..=end]
            .iter()
            .map(|position| position - origin)
            .collect()
    }
}
