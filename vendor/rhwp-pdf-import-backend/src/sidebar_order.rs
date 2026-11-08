//! Publisher-sidebar order from the shared rule, with request-owned scratch.
use crate::{error::Result, import_storage::ImportStorage, layout::TextSpan};
pub(crate) fn order(
    spans: &[TextSpan],
    allocation: ImportStorage<'_>,
) -> Result<Option<Vec<usize>>> {
    use crate::utils::safe_float_cmp;
    allocation.bytes(0)?;
    if spans.len() < 30 {
        return Ok(None);
    }
    let x_min = spans.iter().map(|s| s.bbox.left()).fold(f32::MAX, f32::min);
    let x_max = spans
        .iter()
        .map(|s| s.bbox.right())
        .fold(f32::MIN, f32::max);
    let y_min = spans.iter().map(|s| s.bbox.top()).fold(f32::MAX, f32::min);
    let y_max = spans
        .iter()
        .map(|s| s.bbox.bottom())
        .fold(f32::MIN, f32::max);
    let width = (x_max - x_min).max(1.0);
    let height = (y_max - y_min).max(1.0);
    if !(width.is_finite() && height.is_finite()) {
        return Ok(None);
    }

    // Cluster spans into baseline lines (top→bottom).
    let mut order = allocation.collect(0..spans.len())?;
    allocation.sort_by(&mut order, |&a, &b| {
        safe_float_cmp(spans[b].bbox.bottom(), spans[a].bbox.bottom())
            .then_with(|| safe_float_cmp(spans[a].bbox.left(), spans[b].bbox.left()))
    })?;
    struct Line {
        top_y: f32,
        min_left: f32,
        max_right: f32,
        members: Vec<usize>,
    }
    let mut lines: Vec<Line> = Vec::new();
    for &i in &order {
        let s = &spans[i];
        let h = (s.bbox.bottom() - s.bbox.top()).abs().max(1.0);
        match lines.last_mut() {
            Some(l) if (l.top_y - s.bbox.bottom()).abs() <= h * 0.6 => {
                l.min_left = l.min_left.min(s.bbox.left());
                l.max_right = l.max_right.max(s.bbox.right());
                allocation.push(&mut l.members, i)?;
            },
            _ => {
                let members = allocation.collect([i])?;
                allocation.push(
                    &mut lines,
                    Line {
                        top_y: s.bbox.bottom(),
                        min_left: s.bbox.left(),
                        max_right: s.bbox.right(),
                        members,
                    },
                )?;
            },
        }
    }
    if lines.len() < 12 {
        return Ok(None);
    }

    // Find the body-column left edge: the most common line-start that sits
    // well right of the page left margin (excludes the sidebar/title cluster
    // anchored at x_min). The gutter is just left of it.
    let body_left = {
        const BIN: f32 = 5.0;
        let nbins = ((width / BIN).ceil() as usize).clamp(1, 4096);
        let mut hist = allocation.vec(nbins)?;
        hist.resize(nbins, 0usize);
        for l in &lines {
            if l.min_left > x_min + width * 0.10 {
                let b = (((l.min_left - x_min) / BIN) as usize).min(nbins - 1);
                hist[b] += 1;
            }
        }
        let Some((peak, &cnt)) = hist.iter().enumerate().max_by_key(|(_, &c)| c) else {
            return Ok(None);
        };
        if cnt < 5 {
            return Ok(None); // no consistent body-column start
        }
        x_min + peak as f32 * BIN
    };
    // Gutter must be left-of-centre (a narrow sidebar, not a centred 2-col).
    let gutter = body_left - width * 0.02;
    if !(gutter > x_min + width * 0.12 && gutter < x_min + width * 0.45) {
        return Ok(None);
    }

    // A full-width TITLE/heading row is typeset as many narrow word spans, so
    // no single span satisfies the per-span band test below; its leftmost words
    // (right edge ≤ gutter) would be miswept into the SIDEBAR and emitted last,
    // shattering the title (e.g. an MDPI first page where the title sits in a
    // large font across the full width, above the metadata sidebar + body).
    // Detect these per LINE: a baseline line whose member words FLOW
    // CONTINUOUSLY across the gutter (collective extent crosses it, ≥40% of the
    // page width, and no large internal gap straddling the gutter) is a true
    // full-width band — its words are evenly spaced, unlike a sidebar-label +
    // body-line that merely SHARE a baseline (e.g. "Accepted: 1 March 2021"
    // next to "1. Introduction"), which leaves a wide empty gutter corridor
    // between the two columns. Members of such band lines are forced into the
    // BAND group so the whole title row stays together at its vertical slot.
    // The straddling-gap gate keeps shared sidebar/body baselines split.
    let mut band_line_members = allocation.vec(spans.len())?;
    band_line_members.resize(spans.len(), false);
    const MAX_STRADDLE_GAP_FRAC: f32 = 0.06; // ≈ gutter-corridor width
                                             // The title/author band sits ABOVE the body column (the body starts at the
                                             // affiliations/abstract). Restrict band promotion to lines above the
                                             // topmost PURE-BODY line (a line whose words all begin at/right of the
                                             // gutter, with no left-of-gutter member). Below that Y the page is the
                                             // two-column sidebar+body region, where a wide crossing line is a
                                             // sidebar-label + body-line sharing a baseline (e.g. "Switzerland." next to
                                             // "cancer, atrial fibrillation…") whose tight gutter corridor would
                                             // otherwise pass the straddle-gap gate and wrongly glue the sidebar inline.
                                             // Larger bottom-y == higher on the page (PDF user space).
    let body_top_y = lines
        .iter()
        .filter(|l| l.min_left >= gutter)
        .map(|l| l.top_y)
        .fold(f32::NEG_INFINITY, f32::max);
    for line in &lines {
        if !(line.min_left < gutter
            && line.max_right > gutter
            && (line.max_right - line.min_left) > width * 0.40)
        {
            continue;
        }
        // Only the top-of-page title/author band, above the body column.
        if body_top_y.is_finite() && line.top_y < body_top_y {
            continue;
        }
        // Largest gap between consecutive members that straddles the gutter.
        let mut xs = allocation.collect(
            line.members
                .iter()
                .map(|&i| (spans[i].bbox.left(), spans[i].bbox.right())),
        )?;
        allocation.sort_by(&mut xs, |a, b| safe_float_cmp(a.0, b.0))?;
        let mut max_straddle_gap = 0.0f32;
        let mut prev_right = f32::NEG_INFINITY;
        for &(l, r) in &xs {
            if prev_right.is_finite() && prev_right < gutter && l > gutter {
                max_straddle_gap = max_straddle_gap.max(l - prev_right);
            }
            prev_right = prev_right.max(r);
        }
        if max_straddle_gap < width * MAX_STRADDLE_GAP_FRAC {
            for &i in &line.members {
                band_line_members[i] = true;
            }
        }
    }

    // Classify each SPAN by the gutter. A publisher-metadata sidebar and the
    // body usually SHARE baselines (the metadata column interleaves with body
    // lines by Y), so a per-line cluster would fuse them into one full-width
    // line and hide the sidebar — classify per span instead. BAND = a span
    // genuinely spanning the gutter (a wide full-width title/heading), or a
    // member of a continuous full-width band LINE (above). SIDEBAR = a span
    // entirely left of the gutter. BODY = everything at/right of it.
    let mut band: Vec<usize> = Vec::new();
    let mut sidebar: Vec<usize> = Vec::new();
    let mut body: Vec<usize> = Vec::new();
    for (i, s) in spans.iter().enumerate() {
        let l = s.bbox.left();
        let r = s.bbox.right();
        if band_line_members[i] || (l < gutter && r > gutter && (r - l) > width * 0.40) {
            allocation.push(&mut band, i)?;
        } else if r <= gutter {
            allocation.push(&mut sidebar, i)?;
        } else {
            allocation.push(&mut body, i)?;
        }
    }
    // A real sidebar/body are each multi-line.
    let line_count = |v: &[usize]| -> Result<usize> {
        let mut ys = allocation.collect(v.iter().map(|&i| spans[i].bbox.bottom()))?;
        allocation.sort_by(&mut ys, |a, b| safe_float_cmp(*a, *b))?;
        ys.dedup_by(|a, b| (*a - *b).abs() <= 2.0);
        Ok(ys.len())
    };
    if line_count(&sidebar)? < 5 || line_count(&body)? < 8 {
        return Ok(None);
    }
    // Sidebar genuinely narrower than the body column.
    let col_width = |v: &[usize]| -> f32 {
        let lo = v
            .iter()
            .map(|&i| spans[i].bbox.left())
            .fold(f32::MAX, f32::min);
        let hi = v
            .iter()
            .map(|&i| spans[i].bbox.right())
            .fold(f32::MIN, f32::max);
        (hi - lo).max(0.0)
    };
    let sw = col_width(&sidebar);
    let bw = col_width(&body);
    if sw >= width * 0.45 || sw >= bw * 0.70 {
        return Ok(None); // left column not a narrow sidebar relative to the body
    }
    // ANTI-FORM discriminator. A bare narrow left column is geometrically
    // indistinguishable from a label:value form (Name:/Address:/Date:) or a
    // verse/margin-note page, and these PDFs carry NO background tint to anchor
    // the sidebar. The reliable signal is semantic: a publisher-metadata
    // sidebar carries recognisable furniture labels that never head a form
    // field or a body column. Require >=2 DISTINCT labels so ordinary narrow
    // columns and forms never engage this reordering.
    let text_bytes = sidebar.iter().try_fold(0usize, |total, &i| {
        total
            .checked_add(allocation.lowercase_bytes(&spans[i].text)?)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| allocation.reject("sidebar text size"))
    })?;
    let mut side_text = allocation.string_capacity(text_bytes)?;
    for &i in &sidebar {
        side_text.push_str(&allocation.lowercase(&spans[i].text)?);
        side_text.push(' ');
    }

    const FURNITURE: [&str; 12] = [
        "citation",
        "received",
        "accepted",
        "published",
        "copyright",
        "licensee",
        "academic editor",
        "publisher",
        "doi.org",
        "issn",
        "creative commons",
        "open access",
    ];
    let furniture_hits = FURNITURE.iter().filter(|k| side_text.contains(**k)).count();
    if furniture_hits < 2 {
        return Ok(None);
    }

    // Emit: band + body merged top→bottom (title stays on top, body flows,
    // any mid-body full-width element keeps its vertical slot), then the
    // sidebar furniture last. Spans within a line read left→right.
    let mut main: Vec<usize> = band;
    allocation.reserve(&mut main, body.len())?;
    main.extend(body);
    let key = |idx: &usize| {
        let s = &spans[*idx];
        (s.bbox.bottom(), s.bbox.left())
    };
    allocation.sort_by(&mut main, |a, b| {
        let (ay, ax) = key(a);
        let (by, bx) = key(b);
        safe_float_cmp(by, ay).then_with(|| safe_float_cmp(ax, bx))
    })?;
    allocation.sort_by(&mut sidebar, |a, b| {
        let (ay, ax) = key(a);
        let (by, bx) = key(b);
        safe_float_cmp(by, ay).then_with(|| safe_float_cmp(ax, bx))
    })?;
    allocation.reserve(&mut main, sidebar.len())?;
    main.extend(sidebar);
    Ok(Some(main))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geometry::Rect,
        import_limits::{ImportBudget, PdfImportLimits},
    };
    fn publisher_page() -> (Vec<TextSpan>, Vec<usize>) {
        let mut spans = Vec::new();
        let mut body = Vec::new();
        let mut sidebar = Vec::new();
        for row in 0..20 {
            let y = 800. - row as f32 * 20.;
            if row < 12 {
                sidebar.push(spans.len());
                spans.push(TextSpan {
                    text: match row {
                        0 => "Citation: source".into(),
                        1 => "RECEIVED: ΟΣ İ".into(),
                        _ => format!("metadata-{row}"),
                    },
                    bbox: Rect::new(40., y, 120., 10.),
                    font_size: 10.,
                    ..Default::default()
                });
            }
            body.push(spans.len());
            spans.push(TextSpan {
                text: format!("body-{row}"),
                bbox: Rect::new(200., y, 340., 10.),
                font_size: 10.,
                ..Default::default()
            });
        }
        body.extend(sidebar);
        (spans, body)
    }
    #[test]
    fn budgeted_publisher_order_preserves_every_source_span_and_places_metadata_after_body() {
        let (spans, expected) = publisher_page();
        let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let indices = order(&spans, ImportStorage(Some(&budget)))
            .unwrap()
            .expect("publisher sidebar");
        assert_eq!(indices, expected);
        assert_eq!(indices.len(), spans.len());
        assert_eq!(spans[0].text, "Citation: source");
        assert!(budget.check().is_ok());
    }
    #[test]
    fn an_exhausted_sidebar_request_is_fatal_instead_of_not_matched_and_recovers() {
        let (spans, expected) = publisher_page();
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(order(&spans, ImportStorage(Some(&tiny))).is_err());
        assert_eq!(tiny.snapshot().decoded_bytes, 0);
        assert!(order(&[], ImportStorage(Some(&tiny))).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(order(&spans, ImportStorage(Some(&fresh))).unwrap().unwrap(), expected);
    }
}
