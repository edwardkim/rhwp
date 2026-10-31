//! Shared glyph-origin alignment with request-owned scratch storage.
//! The matching, threshold and interpolation rules are the existing producer.
use crate::{
    error::Result,
    import_limits::ImportBudget,
    import_storage::ImportStorage,
    layout::{TextChar, TextSpan},
};

pub(crate) fn stamp(
    accurate: &[TextChar],
    spans: &mut [TextSpan],
    budget: Option<&ImportBudget>,
) -> Result<()> {
    let allocation = ImportStorage(budget);
    allocation.bytes(0)?;
    if accurate.is_empty() {
        return Ok(());
    }
    // Baseline index: char positions ordered by `origin_y`, so each span's
    // baseline slice is a binary-searched range rather than a linear scan
    // over every glyph on the page (that scan made this pass
    // O(spans x chars) — the dominant per-page cost on long documents).
    let count = u32::try_from(accurate.len())
        .map_err(|_| allocation.reject("text alignment index size"))?;
    let mut by_y = allocation.collect(0..count)?;
    allocation.sort_by(&mut by_y, |&a, &b| {
        crate::utils::safe_float_cmp(accurate[a as usize].origin_y, accurate[b as usize].origin_y)
    })?;
    let ys = allocation.collect(by_y.iter().map(|&i| accurate[i as usize].origin_y))?;

    for span in spans.iter_mut() {
        // Rotated-content spans are in the displayed frame (mapped on 90/270)
        // and their glyphs run vertically; a horizontal-x stamp from the raw
        // chars would not correspond, so leave them to the prefix-sum path.
        if span.rotation_degrees != 0.0 {
            continue;
        }
        // Start clean: any offsets carried over via struct-update from a
        // source span must not be trusted for this (possibly edited) text.
        span.char_x_offsets.clear();

        let mut glyphs = allocation.vec(span.text.chars().count())?;
        glyphs.extend(span.text.chars());
        let n = glyphs.len();
        if n == 0 {
            continue;
        }

        // Accurate chars sharing this span's baseline, left-to-right.
        let baseline_tol = 0.6 * span.font_size.max(1.0);
        // Chars sharing this span's baseline, left-to-right. The y-sorted
        // index brackets a candidate range; the exact `abs()` predicate then
        // selects from it, so the result matches a full linear scan even
        // where the bracket arithmetic rounds differently. The widened
        // bracket keeps that range a superset. Ordering by
        // (origin_x, source index) reproduces the previous stable
        // filter-then-sort: ties on origin_x keep their `accurate` order.
        let bracket = baseline_tol + baseline_tol.abs() * 1e-6 + f32::EPSILON;
        let lo = ys.partition_point(|&y| y < span.bbox.y - bracket);
        let hi = ys.partition_point(|&y| y <= span.bbox.y + bracket);
        let selected = || {
            by_y[lo..hi]
                .iter()
                .copied()
                .filter(|&i| (accurate[i as usize].origin_y - span.bbox.y).abs() <= baseline_tol)
        };
        let mut idx = allocation.vec(selected().count())?;
        idx.extend(selected());
        if idx.is_empty() {
            continue;
        }
        allocation.sort_by(&mut idx, |&a, &b| {
            crate::utils::safe_float_cmp(
                accurate[a as usize].origin_x,
                accurate[b as usize].origin_x,
            )
            .then(a.cmp(&b))
        })?;
        let line = allocation.collect(idx.iter().map(|&i| &accurate[i as usize]))?;

        // Greedy per-glyph alignment. Anchor the scan cursor at the accurate
        // char nearest this span's left edge, then walk the span's glyphs,
        // matching each to the next equal accurate char within a small
        // forward window. Unlike an all-or-nothing contiguous match, a single
        // unmatched glyph (an inserted word-boundary space, a ligature split,
        // a combining mark) no longer discards the whole span — such glyphs
        // are interpolated below so `char_x_offsets` still fills every index.
        let start = line
            .iter()
            .position(|c| c.origin_x >= span.bbox.x - 0.5)
            .unwrap_or(0);
        let mut assigned = allocation.vec(n)?;
        assigned.resize(n, None);
        let mut li = start;
        for (k, &g) in glyphs.iter().enumerate() {
            let mut j = li;
            let mut steps = 0;
            while j < line.len() && steps < 6 {
                if line[j].char == g {
                    assigned[k] = Some(line[j].origin_x);
                    li = j + 1;
                    break;
                }
                j += 1;
                steps += 1;
            }
        }

        // Need enough real anchors to trust the run; otherwise fall back.
        let anchors = assigned.iter().filter(|a| a.is_some()).count();
        if anchors * 5 < n * 3 {
            // < 60% matched
            continue;
        }

        // Fill the gaps: each unmatched glyph takes the nearest preceding
        // anchor plus the prefix sum of the (locally accurate) char_widths
        // between them; if there is no preceding anchor, walk back from the
        // nearest following one. Over the short spans between anchors the
        // cumulative drift these interpolations reintroduce is sub-point.
        let cw = &span.char_widths;
        let width_at = |i: usize| -> f32 {
            if cw.len() == n {
                cw[i]
            } else {
                span.bbox.width / n as f32
            }
        };
        let mut offs = allocation.vec(n)?;
        offs.resize(n, 0.0f32);
        // forward pass from preceding anchors
        let mut last: Option<(usize, f32)> = None;
        for k in 0..n {
            if let Some(x) = assigned[k] {
                offs[k] = x;
                last = Some((k, x));
            } else if let Some((lk, lx)) = last {
                let acc: f32 = (lk..k).map(width_at).sum();
                offs[k] = lx + acc;
            }
        }
        // backfill any leading None from the first following anchor
        if assigned[0].is_none() {
            if let Some(fk) = assigned.iter().position(|a| a.is_some()) {
                let fx = assigned[fk].unwrap();
                for k in 0..fk {
                    let acc: f32 = (k..fk).map(width_at).sum();
                    offs[k] = fx - acc;
                }
            }
        }
        span.char_x_offsets = offs;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geometry::Rect, import_limits::PdfImportLimits};
    fn source() -> (Vec<TextChar>, Vec<TextSpan>) {
        let chars = [
            ('A', 10., 20.),
            ('B', 13., 20.),
            ('C', 16., 20.),
            ('A', 99., 50.),
        ]
        .into_iter()
        .map(|(char, origin_x, origin_y)| TextChar {
            char,
            origin_x,
            origin_y,
            ..Default::default()
        })
        .collect();
        let spans = vec![TextSpan {
            text: "A BC".into(),
            bbox: Rect::new(10., 20., 12., 10.),
            font_size: 10.,
            char_widths: vec![3., 3., 3., 3.],
            char_x_offsets: vec![-99.; 4],
            ..Default::default()
        }];
        (chars, spans)
    }
    #[test]
    fn scratch_failure_precedes_alignment_and_a_fresh_request_recovers() {
        let (chars, mut spans) = source();
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(stamp(&chars, &mut spans, Some(&tiny)).is_err());
        assert_eq!(tiny.snapshot().decoded_bytes, 0);
        assert_eq!(spans[0].char_x_offsets, vec![-99.; 4]);
        assert!(stamp(&chars, &mut spans, Some(&tiny)).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        stamp(&chars, &mut spans, Some(&fresh)).unwrap();
        assert_eq!(spans[0].char_x_offsets, vec![10., 13., 13., 16.]);
        assert!(fresh.check().is_ok());
        let (_, mut ordinary) = source();
        stamp(&chars, &mut ordinary, None).unwrap();
        assert_eq!(ordinary[0].char_x_offsets, spans[0].char_x_offsets);
    }
    #[test]
    fn rotated_and_unmatched_runs_keep_the_existing_fallback_contract() {
        let (chars, mut spans) = source();
        spans[0].rotation_degrees = 90.;
        stamp(&chars, &mut spans, None).unwrap();
        assert_eq!(spans[0].char_x_offsets, vec![-99.; 4]);
        spans[0].rotation_degrees = 0.;
        spans[0].text = "WXYZ".into();
        stamp(&chars, &mut spans, None).unwrap();
        assert!(spans[0].char_x_offsets.is_empty());
    }
    #[test]
    fn an_exhausted_scratch_budget_cannot_return_partial_alignment() {
        let (chars, mut spans) = source();
        let low = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 100,
            ..Default::default()
        })
        .unwrap();
        assert!(stamp(&chars, &mut spans, Some(&low)).is_err());
        assert!(low.check().is_err());
        assert!(spans[0].char_x_offsets.is_empty());
        assert!(stamp(&chars, &mut spans, Some(&low)).is_err());
    }
}
