//! 양쪽 정렬의 문단 앞 공백과 낱말 사이 공백을 같은 줄 계획에서 구분한다.

use super::super::text_measurement::estimate_text_width_unrounded;
use crate::renderer::{composer::ComposedLine, style_resolver::ResolvedStyleSet};
use std::borrow::Cow;

pub(super) struct LeadingSpacePlan<'a> {
    pub line: Cow<'a, ComposedLine>,
    pub leading_spaces: usize,
    pub natural_prefix_runs: usize,
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
            line: Cow::Owned(prepared),
            leading_spaces,
            natural_prefix_runs,
        }
    }

    pub fn natural_width(&self, styles: &ResolvedStyleSet, tab_width: f64) -> f64 {
        self.line
            .runs
            .iter()
            .take(self.natural_prefix_runs)
            .map(|run| {
                let mut style = run.text_style(styles);
                style.default_tab_width = tab_width;
                estimate_text_width_unrounded(&run.text, &style)
            })
            .sum()
    }

    pub fn spacing(&self, run_index: usize, extra: f64) -> f64 {
        if run_index < self.natural_prefix_runs {
            0.0
        } else {
            extra
        }
    }
}
