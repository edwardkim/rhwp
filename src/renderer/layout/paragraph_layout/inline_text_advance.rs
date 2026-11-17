//! 인라인 표 문단의 측정·배치·glyph가 공유하는 원본 문자별 전진.

use super::super::text_measurement::{build_cluster_len, char_width_decision};
use super::{estimate_text_width_exact, TextStyle};
use crate::renderer::composer::is_lang_neutral;
use crate::renderer::style_resolver::detect_lang_category;

pub(super) struct StoredLineFrame {
    pub column_x: f64,
    pub first_x: f64,
    pub continuation_x: f64,
    pub dpi: f64,
}

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

    /// 개체를 담지 않은 저장 글줄도 일반 본문과 같은 양쪽 정렬 계획을 소비한다.
    /// 표가 든 줄·재조판 줄·탭/필드/겹친 글자는 각 owner의 기존 계약에 남긴다.
    pub(super) fn justify_stored_lines(
        &mut self,
        chars: &[char],
        para: &crate::model::paragraph::Paragraph,
        composed: Option<&super::ComposedParagraph>,
        styles: &super::ResolvedStyleSet,
        frame: StoredLineFrame,
        mut style_at: impl FnMut(usize, usize) -> TextStyle,
    ) {
        let Some(composed) = composed.filter(|composed| {
            !para.stored_text_partition_is_dirty()
                && !para.cell_format_vpos_dirty
                && !para.layout_object_projection
                && composed.frame_geometry.is_none()
                && composed.lines.len() == para.line_segs.len()
        }) else {
            return;
        };
        let Some(paragraph_style) = styles.para_styles.get(composed.para_style_id as usize) else {
            return;
        };
        let cluster_len = build_cluster_len(chars);
        for (line_index, line) in composed.lines.iter().enumerate() {
            let stored = &para.line_segs[line_index];
            let start = line.char_start;
            let end = composed
                .lines
                .get(line_index + 1)
                .map_or(chars.len(), |next| next.char_start);
            if !super::needs_word_distribution(
                paragraph_style.alignment,
                line_index + 1 == composed.lines.len(),
                line.has_line_break,
            ) || start >= end
                || end > chars.len()
                || stored.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || stored.column_start < 0
                || stored.segment_width <= 0
                || composed
                    .inline_controls
                    .iter()
                    .any(|control| {
                        // 원본 주소가 있는 컨트롤은 실제 저장 줄을 권위로 쓴다.
                        // 요약의 기본 line_index=0이 앞선 일반 본문 줄까지 막지 않는다.
                        super::super::control_line_seg_index(para, control.control_index)
                            .map_or(control.line_index == line_index, |owner| owner == line_index)
                    })
                // 요약에 빠진 컨트롤도 원본의 실제 저장 줄을 확인한다.
                // 문자열에서 빠진 TAC 폭을 공백 여유로 중복 배분하지 않는다.
                || para.controls.iter().enumerate().any(|(index, control)| {
                    control.is_treat_as_char_object()
                        && super::super::control_line_seg_index(para, index) == Some(line_index)
                })
                || line.runs.iter().any(|run| {
                    run.char_overlap.is_some()
                        || run.footnote_marker.is_some()
                        || run.display_text.is_some()
                        || run.inserted_control_text
                })
                || chars[start..end]
                    .iter()
                    .any(|ch| ch.is_control() || *ch == '\u{FFFC}')
                || !line
                    .runs
                    .iter()
                    .flat_map(|run| run.text.chars())
                    .eq(chars[start..end].iter().copied())
            {
                continue;
            }
            let plan = super::LeadingSpacePlan::new(line, None);
            let fixed_prefix = super::unjustified_leading_spaces_by_run(line, usize::MAX);
            let line_x = if line_index == 0 {
                frame.first_x
            } else {
                frame.continuation_x
            };
            let right = frame.column_x
                + super::hwpunit_to_px(stored.column_start + stored.segment_width, frame.dpi);
            let available = right - line_x;
            if available <= 0.0 {
                continue;
            }
            let mut dash_plan = None;
            let (word, character, dash) = super::compute_line_extra_spacing_with_dash_plan(
                line,
                plan.leading_spaces,
                0,
                end - start,
                &plan,
                0.0,
                super::terminal_tracking_for_distribution(line, styles, end - start),
                &fixed_prefix,
                styles,
                None,
                paragraph_style.alignment,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
                end - start,
                self.width(start, end),
                available,
                paragraph_style.default_tab_width,
                paragraph_style.condense_min_space,
                &mut dash_plan,
                0,
            );
            // 각 run의 자연 대시 복원량까지 동일한 문자 전진에 반영한다.
            let mut index = start;
            for (run_index, run) in plan.line.runs.iter().enumerate() {
                for _ in run.text.chars() {
                    let mut style = style_at(index, self.languages[index]);
                    style.extra_word_spacing = plan.spacing(run_index, word);
                    style.extra_char_spacing = plan.spacing(run_index, character);
                    style.extra_dash_advance = dash_plan
                        .as_ref()
                        .and_then(|values| values.get(run_index))
                        .copied()
                        .unwrap_or(dash);
                    style.unjustified_leading_spaces = start + plan.leading_spaces;
                    self.widths[index] =
                        char_width_decision(chars, &cluster_len, index, &style, true)
                            .final_width_px;
                    index += 1;
                }
            }
        }
        self.prefix.clear();
        self.prefix.push(0.0);
        for width in &self.widths {
            self.prefix
                .push(self.prefix.last().copied().unwrap_or(0.0) + width);
        }
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
