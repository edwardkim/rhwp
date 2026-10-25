//! 비분배 줄의 수용 예산을 원 글자 대신 내부 공백에 배정한다.

use super::{
    effective_text_for_metrics, estimate_text_width_exact, LeadingSpacePlan, ResolvedStyleSet,
};
use crate::renderer::{composer::condense_space_savings_px, space_compression::SpaceCompression};

pub(super) struct CondensePlan {
    pub runs: Vec<Option<SpaceCompression>>,
    pub saving_px: f64,
}

impl CondensePlan {
    pub fn prepare(
        line: &LeadingSpacePlan<'_>,
        styles: &ResolvedStyleSet,
        percent: u8,
        fit_width: f64,
        available: f64,
        dpi: f64,
    ) -> Option<Self> {
        let needed = fit_width - available;
        if percent == 0
            || percent > 75
            || needed <= 0.0
            || !needed.is_finite()
            || dpi <= 0.0
            || !dpi.is_finite()
        {
            return None;
        }
        let chars = line.spacing_chars();
        let end = chars.iter().rposition(|(ch, _)| *ch != ' ')? + 1;
        let mut position = 0;
        let mut entries = Vec::new();
        let mut budget = 0.0;
        for (index, run) in line.line.runs.iter().enumerate() {
            let text = effective_text_for_metrics(run);
            let field = line.body_field_runs.get(index).copied().unwrap_or(false);
            let length = if field { 1 } else { text.chars().count() };
            let eligible_end = end.saturating_sub(position).min(length);
            position += length;
            if index < line.natural_prefix_runs || field || run.char_overlap.is_some() {
                entries.push(None);
                continue;
            }
            // 원자적 표시 투영에서 새 공백 기회를 추측하지 않는다.
            if text.chars().count() != run.source_scalar_count() && text.contains(' ') {
                return None;
            }
            let count = text
                .chars()
                .take(eligible_end)
                .filter(|ch| *ch == ' ')
                .count();
            if count == 0 {
                entries.push(None);
                continue;
            }
            let style = run.text_style(styles);
            let natural = estimate_text_width_exact(" ", &style);
            let mut untracked = style;
            untracked.letter_spacing = 0.0;
            if let Some(latin) = untracked.ascii_punct_latin.as_mut() {
                latin.letter_spacing = 0.0;
            }
            let allowed =
                condense_space_savings_px(estimate_text_width_exact(" ", &untracked), percent, dpi);
            if !natural.is_finite() || natural <= 0.0 || !allowed.is_finite() || allowed <= 0.0 {
                return None;
            }
            budget += allowed * count as f64;
            entries.push(Some((eligible_end, count, allowed, natural)));
        }
        if needed > budget || budget <= 0.0 {
            return None;
        }
        let share = needed / budget;
        let mut runs = Vec::new();
        let mut saving_px = 0.0;
        for entry in entries {
            if let Some((end, count, allowed, natural)) = entry {
                let reduction_px = allowed * share;
                if reduction_px >= natural {
                    return None;
                }
                saving_px += reduction_px * count as f64;
                runs.push(Some(SpaceCompression {
                    start: 0,
                    end,
                    reduction_px,
                }));
            } else {
                runs.push(None);
            }
        }
        Some(Self { runs, saving_px })
    }
}
