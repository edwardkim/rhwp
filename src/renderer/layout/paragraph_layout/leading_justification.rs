//! 양쪽 정렬의 문단 앞 공백과 낱말 사이 공백을 같은 줄 계획에서 구분한다.

use super::super::text_measurement::estimate_text_width_unrounded;
use crate::renderer::{
    composer::{effective_text_for_metrics, ComposedLine},
    style_resolver::ResolvedStyleSet,
};
use std::borrow::Cow;

pub(super) struct LeadingSpacePlan<'a> {
    pub line: Cow<'a, ComposedLine>,
    pub leading_spaces: usize,
    pub natural_prefix_runs: usize,
    pub body_field_runs: Vec<bool>,
    pub terminal_inline_object: bool,
}

impl<'a> LeadingSpacePlan<'a> {
    pub fn new(line: &'a ComposedLine, first_inline_pos: Option<usize>) -> Self {
        let limit = first_inline_pos.map_or(usize::MAX, |pos| pos.saturating_sub(line.char_start));
        let mut leading_spaces = 0;
        for run in &line.runs {
            // 표시 필드·각주·겹친 글자는 원본 공백 문자열과 다른 흐름 요소다.
            if run.display_text.is_some()
                || run.footnote_marker.is_some()
                || run.char_overlap.is_some()
                || run.inserted_control_text
            {
                break;
            }
            let prefix = run.text.chars().take_while(|&ch| ch == ' ').count();
            leading_spaces += prefix.min(limit.saturating_sub(leading_spaces));
            if leading_spaces == limit || prefix < run.text.chars().count() {
                break;
            }
        }
        if leading_spaces == 0 {
            return Self {
                line: Cow::Borrowed(line),
                leading_spaces: 0,
                natural_prefix_runs: 0,
                body_field_runs: Vec::new(),
                terminal_inline_object: false,
            };
        }
        let mut prepared = line.clone();
        prepared.runs.clear();
        let mut remaining = leading_spaces;
        let mut natural_prefix_runs = 0;
        for run in &line.runs {
            if remaining == 0 {
                prepared.runs.push(run.clone());
                continue;
            }
            // 이 범위는 위에서 확인한 ASCII 공백뿐이므로 문자·바이트 경계가 같다.
            let take = remaining.min(run.text.len());
            if take == run.text.len() {
                prepared.runs.push(run.clone());
            } else {
                let mut prefix = run.clone();
                prefix.text.truncate(take);
                prepared.runs.push(prefix);
                let mut rest = run.clone();
                rest.text = run.text[take..].to_owned();
                prepared.runs.push(rest);
            }
            natural_prefix_runs += 1;
            remaining -= take;
        }
        Self {
            body_field_runs: Vec::new(),
            terminal_inline_object: false,
            line: Cow::Owned(prepared),
            leading_spaces,
            natural_prefix_runs,
        }
    }

    /// 마지막 가시 글자 다음에 실제 개체가 있으면 그 사이 자간도 소비한다.
    pub fn with_inline_objects(mut self, offsets: &[(usize, f64, usize)]) -> Self {
        let text_end = self.line.char_start
            + self
                .line
                .runs
                .iter()
                .map(|run| run.source_scalar_count())
                .sum::<usize>();
        let trailing = self
            .line
            .runs
            .iter()
            .rev()
            .flat_map(|run| run.text.chars().rev())
            .take_while(|ch| *ch == ' ')
            .count();
        self.terminal_inline_object = offsets
            .iter()
            .any(|(position, _, _)| *position >= text_end.saturating_sub(trailing));
        self
    }

    /// 표시 숫자는 원본 필드 한 칸에 속한다. 자릿수와 관계없이 내부/뒤에
    /// 정렬 여분을 붙이지 않는다. 일반 공백이나 PUA 표시 확장과 구분한다.
    pub fn with_body_fields(mut self, fields: &[(usize, String)]) -> Self {
        if fields.is_empty() {
            return self;
        }
        self.body_field_runs.resize(self.line.runs.len(), false);
        let mut position = self.line.char_start;
        for (index, run) in self.line.runs.iter().enumerate() {
            if run.inserted_control_text {
                continue;
            }
            let count = run.text.chars().count();
            self.body_field_runs[index] = count == 1
                && run.display_text.is_some()
                && fields.iter().any(|(at, _)| *at == position);
            position += count;
        }
        self
    }

    fn is_field_run(&self, index: usize) -> bool {
        self.body_field_runs.get(index).copied().unwrap_or(false)
    }

    pub fn spacing_chars(&self) -> Vec<(char, bool)> {
        if self.body_field_runs.is_empty() {
            return self
                .line
                .runs
                .iter()
                .flat_map(|run| {
                    effective_text_for_metrics(run)
                        .chars()
                        .map(|ch| (ch, false))
                })
                .collect();
        }
        self.line
            .runs
            .iter()
            .enumerate()
            .flat_map(|(index, run)| {
                if self.is_field_run(index) {
                    vec![('\u{fffc}', true)]
                } else {
                    effective_text_for_metrics(run)
                        .chars()
                        .map(|ch| (ch, false))
                        .collect()
                }
            })
            .collect()
    }

    pub fn field_char_count(&self) -> usize {
        if self.body_field_runs.is_empty() {
            return 0;
        }
        self.line
            .runs
            .iter()
            .enumerate()
            .filter(|(index, _)| self.is_field_run(*index))
            .map(|(_, run)| effective_text_for_metrics(run).chars().count())
            .sum()
    }

    /// 줄 수용의 한 em과 실제 pen 전진의 차이. 긴 쪽번호를 상자 폭에
    /// 맞추려고 압축하면 독립 Print의 자연 숫자·후속 글자 위치를 잃는다.
    pub fn field_fit_adjustment(&self, styles: &ResolvedStyleSet) -> f64 {
        if self.body_field_runs.is_empty() {
            return 0.0;
        }
        self.line
            .runs
            .iter()
            .enumerate()
            .filter(|(index, _)| self.is_field_run(*index))
            .map(|(_, run)| {
                run.line_box_font_size(styles)
                    - estimate_text_width_unrounded(
                        effective_text_for_metrics(run),
                        &run.text_style(styles),
                    )
            })
            .sum()
    }

    pub fn natural_width(&self, styles: &ResolvedStyleSet, tab_width: f64) -> f64 {
        if self.natural_prefix_runs == 0 && self.body_field_runs.is_empty() {
            return 0.0;
        }
        self.line
            .runs
            .iter()
            .enumerate()
            .filter(|(index, _)| *index < self.natural_prefix_runs || self.is_field_run(*index))
            .map(|(_, run)| {
                let mut style = run.text_style(styles);
                style.default_tab_width = tab_width;
                estimate_text_width_unrounded(effective_text_for_metrics(run), &style)
            })
            .sum()
    }

    pub fn spacing(&self, run_index: usize, extra: f64) -> f64 {
        if run_index < self.natural_prefix_runs || self.is_field_run(run_index) {
            0.0
        } else {
            extra
        }
    }
}
