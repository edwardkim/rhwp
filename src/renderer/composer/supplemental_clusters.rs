//! Project logical paragraph graphemes across style/language and stored-line splits.
use super::{ComposedParagraph, ComposedTextRun, SpaceMetric};
use crate::renderer::supplemental_metrics::scalar_eligibility;

/// Logical scalar eligibility prepared once, before token/style subdivision.
pub(crate) struct ParagraphMetricScope(Option<Vec<bool>>, SpaceMetric);

impl ParagraphMetricScope {
    pub(crate) fn allows(&self, index: usize) -> bool {
        self.0
            .as_ref()
            .is_none_or(|mask| mask.get(index).copied().unwrap_or(false))
    }
    pub(crate) fn new(chars: &[char], styles: &super::ResolvedStyleSet) -> Self {
        Self(
            styles.supplemental_metrics.as_ref().map(|_| {
                let text: String = chars.iter().collect();
                scalar_eligibility(&text)
            }),
            SpaceMetric::Stored,
        )
    }

    pub(crate) fn with_space_metric(mut self, metric: SpaceMetric) -> Self {
        self.1 = metric;
        self
    }

    pub(crate) fn style(
        &self,
        styles: &super::ResolvedStyleSet,
        id: u32,
        lang: usize,
        index: usize,
    ) -> super::TextStyle {
        let mut style = super::resolved_to_text_style(styles, id, lang);
        if !self.allows(index) {
            style.supplemental_metrics = None;
        }
        self.1.apply_to_style(&mut style);
        style
    }
}

pub(super) fn preserve_boundaries(
    composed: &mut ComposedParagraph,
    text: &str,
    styles: Option<&super::ResolvedStyleSet>,
) {
    let allowed = scalar_eligibility(text);
    if allowed.iter().all(|&value| value) {
        return;
    }
    for line in &mut composed.lines {
        let mut offset = line.char_start;
        let mut runs = Vec::new();
        for run in std::mem::take(&mut line.runs) {
            if run.footnote_marker.is_some() || run.inserted_control_text {
                runs.push(ComposedTextRun {
                    supplemental_metrics_blocked: true,
                    ..run
                });
                continue;
            }
            let len = run.text.chars().count();
            let end = offset.saturating_add(len);
            if run.display_text.is_some() || run.char_overlap.is_some() {
                runs.push(ComposedTextRun {
                    supplemental_metrics_blocked: true,
                    ..run
                });
            } else if styles.is_some_and(|styles| !uses_registered_metric(&run, styles)) {
                // An unrelated supplemental entry must not subdivide this nominal run.
                runs.push(run);
            } else if let Some(mask) = allowed.get(offset..end) {
                split_run(run, mask, &mut runs);
            } else {
                // A synthetic/unmapped run cannot prove source membership.
                runs.push(ComposedTextRun {
                    supplemental_metrics_blocked: true,
                    ..run
                });
            }
            offset = end;
        }
        line.runs = runs;
    }
}

fn uses_registered_metric(run: &ComposedTextRun, styles: &super::ResolvedStyleSet) -> bool {
    let style = run.text_style(styles);
    let Some(snapshot) = &style.supplemental_metrics else {
        return false;
    };
    run.text.chars().any(|ch| {
        let Some(entry) = snapshot.lookup(snapshot.context(), &style, ch) else {
            return false;
        };
        super::super::layout::trace_char_width_decisions(&ch.to_string(), &style)
            .iter()
            .any(|decision| decision.width_source == entry.width_source())
    })
}

fn split_run(run: ComposedTextRun, allowed: &[bool], output: &mut Vec<ComposedTextRun>) {
    if let Some(&first) = allowed.first() {
        if allowed.iter().all(|&value| value == first) {
            output.push(ComposedTextRun {
                supplemental_metrics_blocked: !first,
                ..run
            });
            return;
        }
    }
    let mut boundaries = run
        .text
        .char_indices()
        .map(|(byte, _)| byte)
        .collect::<Vec<_>>();
    boundaries.push(run.text.len());
    let mut start = 0;
    for end in 1..=allowed.len() {
        if end == allowed.len() || allowed[end] != allowed[start] {
            output.push(ComposedTextRun {
                text: run.text[boundaries[start]..boundaries[end]].to_owned(),
                supplemental_metrics_blocked: !allowed[start],
                ..run.clone()
            });
            start = end;
        }
    }
    if allowed.is_empty() {
        output.push(run);
    }
}
