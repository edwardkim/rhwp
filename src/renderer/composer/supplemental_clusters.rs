//! Project logical paragraph graphemes across style/language and stored-line splits.
use super::{ComposedParagraph, ComposedTextRun, SpaceMetric};
use crate::renderer::supplemental_metrics::scalar_eligibility;

/// Logical scalar eligibility prepared once, before token/style subdivision.
pub(crate) struct ParagraphMetricScope<'a>(Option<Vec<bool>>, SpaceMetric, &'a [char], &'a [usize]);

impl<'a> ParagraphMetricScope<'a> {
    pub(crate) fn allows(&self, index: usize) -> bool {
        self.0
            .as_ref()
            .is_none_or(|mask| mask.get(index).copied().unwrap_or(false))
    }
    pub(crate) fn new(chars: &'a [char], styles: &super::ResolvedStyleSet) -> Self {
        match Self::try_new(
            chars,
            styles,
            &crate::renderer::style_resolver::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    pub(crate) fn try_new<S: crate::renderer::style_resolver::storage::Storage>(
        chars: &'a [char],
        styles: &super::ResolvedStyleSet,
        storage: &S,
    ) -> Result<Self, S::Error> {
        let mask = if styles.supplemental_metrics.is_some() {
            let text = storage.chars(chars.iter().copied())?;
            Some(crate::renderer::supplemental_metrics::try_scalar_eligibility(&text, storage)?)
        } else {
            None
        };
        Ok(Self(mask, SpaceMetric::Stored, chars, &[]))
    }

    pub(crate) fn with_space_metric(mut self, metric: SpaceMetric) -> Self {
        self.1 = metric;
        self
    }

    pub(crate) fn with_literal_figure_spaces(mut self, indices: &'a [usize]) -> Self {
        self.3 = indices;
        self
    }

    pub(crate) fn style(
        &self,
        styles: &super::ResolvedStyleSet,
        id: u32,
        lang: usize,
        index: usize,
    ) -> super::TextStyle {
        match self.try_style(
            styles,
            id,
            lang,
            index,
            &crate::renderer::style_resolver::storage::Unbounded,
        ) {
            Ok(style) => style,
            Err(never) => match never {},
        }
    }
    pub(crate) fn try_style<S: crate::renderer::style_resolver::storage::Storage>(
        &self,
        styles: &super::ResolvedStyleSet,
        id: u32,
        lang: usize,
        index: usize,
        storage: &S,
    ) -> Result<super::TextStyle, S::Error> {
        let mut style = self.2.get(index).map_or_else(
            || crate::renderer::layout::try_resolved_to_text_style(styles, id, lang, storage),
            |&ch| super::punctuation_slots::try_character_style(ch, lang, styles, id, storage),
        )?;
        if !self.allows(index) {
            style.supplemental_metrics = None;
        }
        style.literal_figure_space =
            self.2.get(index) == Some(&'\u{2007}') && self.3.binary_search(&index).is_ok();
        if style.literal_figure_space {
            style.literal_figure_space_em = styles.literal_figure_space_em(id, lang);
        }
        self.1.apply_to_style(&mut style);
        // 이 scope는 새 줄의 채움·분할·투영을 측정한다.
        style.regenerated_font_metrics = true;
        Ok(style)
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

/// 원문 리터럴 공백의 출처를 표시 변환 전에 런에 전달한다.
pub(super) fn preserve_literal_spaces(composed: &mut ComposedParagraph, indices: &[usize]) {
    match preserve_literal_spaces_with_storage(composed, indices, &crate::storage::Unbounded) {
        Ok(()) => (),
        Err(never) => match never {},
    }
}
pub(super) fn preserve_literal_spaces_with_storage<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    indices: &[usize],
    storage: &S,
) -> Result<(), S::Error> {
    if indices.is_empty() {
        return Ok(());
    }
    for line in &mut composed.lines {
        let mut offset = line.char_start;
        let mut output = Vec::new();
        for run in std::mem::take(&mut line.runs) {
            let count = run.source_scalar_count();
            if count == 0 || !run.text.contains('\u{2007}') {
                offset += count;
                storage.push(&mut output, run)?;
                continue;
            }
            let mut boundaries: Vec<usize> =
                storage.collect(run.text.char_indices().map(|(byte, _)| byte))?;
            storage.push(&mut boundaries, run.text.len())?;
            let flags: Vec<bool> =
                storage.collect(run.text.chars().enumerate().map(|(i, ch)| {
                    ch == '\u{2007}' && indices.binary_search(&(offset + i)).is_ok()
                }))?;
            let mut start = 0;
            for end in 1..=flags.len() {
                if end == flags.len() || flags[end] != flags[start] {
                    storage.push(
                        &mut output,
                        ComposedTextRun {
                            text: storage.string(&run.text[boundaries[start]..boundaries[end]])?,
                            literal_figure_space: flags[start],
                            ..crate::storage::try_clone(&run, storage)?
                        },
                    )?;
                    start = end;
                }
            }
            offset += count;
        }
        line.runs = output;
    }
    Ok(())
}
