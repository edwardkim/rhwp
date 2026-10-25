//! Whitespace partitions establish owners before body lines are assembled.
//! Geometry proposes reading regions; it does not recover authored PDF structure.
use pdf_oxide::{geometry::Rect, layout::TextSpan};

const MAX_PARTITION_WORK: usize = 1_000_000;
const MAX_REGIONS: usize = 256;

pub(super) struct Partition {
    pub bounds: Rect,
    pub spans: Vec<usize>,
}

struct Item {
    index: usize,
    bounds: Rect,
    baseline: f32,
    font: f32,
}

struct Work(usize);
impl Work {
    fn charge(&mut self, n: usize) -> Result<(), String> {
        self.0 = self
            .0
            .checked_add(n)
            .ok_or("body partition work overflow")?;
        if self.0 > MAX_PARTITION_WORK {
            return Err("body partition work limit; retain positioned content".into());
        }
        Ok(())
    }
}

fn gaps(items: &[&Item], horizontal: bool, work: &mut Work) -> Result<Vec<(f32, f32)>, String> {
    work.charge(items.len())?;
    let mut intervals: Vec<_> = items
        .iter()
        .map(|i| {
            if horizontal {
                (i.bounds.y, i.bounds.y + i.bounds.height)
            } else {
                (i.bounds.x, i.bounds.x + i.bounds.width)
            }
        })
        .collect();
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut result = Vec::new();
    let mut end = intervals.first().map_or(0., |i| i.1);
    for (start, next) in intervals.into_iter().skip(1) {
        // Abutting boxes still separate cleanly: no item straddles the edge.
        // A spanning heading often ends exactly where the columns begin.
        if start >= end {
            result.push((end, start));
        }
        end = end.max(next);
    }
    result.sort_by(|a, b| (b.1 - b.0).total_cmp(&(a.1 - a.0)));
    Ok(result)
}

fn repeated_rows(a: &[&Item], b: &[&Item], tolerance: f32) -> bool {
    let rows = |items: &[&Item]| {
        let mut rows: Vec<_> = items.iter().map(|i| i.baseline).collect();
        rows.sort_by(f32::total_cmp);
        rows.dedup_by(|a, b| (*a - *b).abs() <= tolerance);
        rows
    };
    let (a, b) = (rows(a), rows(b));
    let (mut i, mut j, mut paired) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        if (a[i] - b[j]).abs() <= tolerance {
            paired += 1;
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    if paired >= 2 {
        return true;
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }
    // Columns need not share baselines: a heading or a picture shifts one
    // column's grid. Both sides still run several rows across the same band.
    let (top, bottom) = (a[0].max(b[0]), a[a.len() - 1].min(b[b.len() - 1]));
    let inside = |rows: &[f32]| rows.iter().filter(|r| **r >= top && **r <= bottom).count();
    top < bottom && inside(&a) >= 3 && inside(&b) >= 3
}

fn vertical_cut(items: &[&Item], work: &mut Work) -> Result<Option<f32>, String> {
    // Word spacing follows the running text size. One larger heading in the
    // band must not raise the gutter threshold above the actual gutter.
    let mut fonts: Vec<f32> = items.iter().map(|i| i.font).collect();
    fonts.sort_by(f32::total_cmp);
    let font = fonts.get(fonts.len() / 2).copied().unwrap_or(0.);
    for (a, b) in gaps(items, false, work)? {
        // A large inter-word space on one row does not establish a column.
        if b - a < font * 2. {
            continue;
        }
        work.charge(items.len())?;
        let (left, right): (Vec<_>, Vec<_>) = items.iter().copied().partition(|i| i.bounds.x < b);
        if repeated_rows(&left, &right, font * 0.2) {
            return Ok(Some((a + b) / 2.));
        }
    }
    Ok(None)
}

fn row_count(items: &[&Item], tolerance: f32) -> usize {
    let mut rows: Vec<_> = items.iter().map(|i| i.baseline).collect();
    rows.sort_by(f32::total_cmp);
    rows.dedup_by(|a, b| (*a - *b).abs() <= tolerance);
    rows.len()
}

/// A running header or footer band: one or two rows at the top or bottom of
/// the page, set off from multi-column body text by a gap wider than one and
/// a half lines. Its pieces (page number, booklet marks) need not cross the
/// gutter, so the column cut alone would hand them to the columns as their
/// first or last rows. Returns the horizontal cut, top band first.
fn running_band_cut(items: &[&Item], work: &mut Work) -> Result<Option<f32>, String> {
    let mut fonts: Vec<f32> = items.iter().map(|i| i.font).collect();
    fonts.sort_by(f32::total_cmp);
    let font = fonts.get(fonts.len() / 2).copied().unwrap_or(0.);
    let tolerance = font * 0.2;
    let mut bands = gaps(items, true, work)?;
    bands.retain(|(a, b)| b - a >= 1.5 * font);
    bands.sort_by(|x, y| x.0.total_cmp(&y.0));
    // The first wide gap from the top, then the first from the bottom.
    let top = bands.first().copied();
    let bottom = bands.last().copied();
    for (gap, band_above) in [(top, true), (bottom, false)] {
        let Some((a, b)) = gap else { continue };
        work.charge(items.len())?;
        let (above, below): (Vec<_>, Vec<_>) = items.iter().copied().partition(|i| i.bounds.y < b);
        let (band, body) = if band_above {
            (&above, &below)
        } else {
            (&below, &above)
        };
        let rows = row_count(band, tolerance);
        if (1..=2).contains(&rows)
            && row_count(body, tolerance) > 2
            && vertical_cut(body, work)?.is_some()
        {
            return Ok(Some((a + b) / 2.));
        }
    }
    Ok(None)
}

fn divide(
    items: Vec<&Item>,
    owner: Rect,
    work: &mut Work,
    out: &mut Vec<Partition>,
    depth: usize,
) -> Result<(), String> {
    if depth > 16 || out.len() >= MAX_REGIONS {
        return Err("body partition depth/region limit".into());
    }
    // Page-level bands come off before columns: once the columns are cut,
    // band pieces would read as the columns' own rows.
    if depth <= 1 {
        if let Some(cut) = running_band_cut(&items, work)? {
            let (top, bottom): (Vec<_>, Vec<_>) = items.into_iter().partition(|i| i.bounds.y < cut);
            divide(
                top,
                Rect::new(owner.x, owner.y, owner.width, cut - owner.y),
                work,
                out,
                depth + 1,
            )?;
            divide(
                bottom,
                Rect::new(owner.x, cut, owner.width, owner.y + owner.height - cut),
                work,
                out,
                depth + 1,
            )?;
            return Ok(());
        }
    }
    if let Some(cut) = vertical_cut(&items, work)? {
        let (left, right) = items.into_iter().partition(|i| i.bounds.x < cut);
        divide(
            left,
            Rect::new(owner.x, owner.y, cut - owner.x, owner.height),
            work,
            out,
            depth + 1,
        )?;
        divide(
            right,
            Rect::new(cut, owner.y, owner.x + owner.width - cut, owner.height),
            work,
            out,
            depth + 1,
        )?;
        return Ok(());
    }
    // A spanning heading may hide a gutter. Cut a vertical band only when it
    // exposes repeated column evidence, rather than splitting every visual line.
    for (a, b) in gaps(&items, true, work)? {
        work.charge(items.len())?;
        let (top, bottom): (Vec<_>, Vec<_>) = items.iter().copied().partition(|i| i.bounds.y < b);
        if vertical_cut(&top, work)?.is_some() || vertical_cut(&bottom, work)?.is_some() {
            let cut = (a + b) / 2.;
            divide(
                top,
                Rect::new(owner.x, owner.y, owner.width, cut - owner.y),
                work,
                out,
                depth + 1,
            )?;
            divide(
                bottom,
                Rect::new(owner.x, cut, owner.width, owner.y + owner.height - cut),
                work,
                out,
                depth + 1,
            )?;
            return Ok(());
        }
    }
    out.push(Partition {
        // Available owner width follows page/gutter partitions. An ink union
        // cannot establish the alignment frame of a centered paragraph.
        bounds: owner,
        spans: items.iter().map(|i| i.index).collect(),
    });
    Ok(())
}

pub(super) fn partition(
    spans: &[TextSpan],
    span_clips: &[Option<std::sync::Arc<pdf_oxide::elements::PathClip>>],
    indices: &[usize],
    media: (f32, f32, f32, f32),
) -> Result<Vec<Partition>, String> {
    if indices.is_empty() {
        return Ok(Vec::new());
    }
    let items: Vec<_> = indices
        .iter()
        .map(|&index| {
            let s = &spans[index];
            Item {
                index,
                // Partition the same spatial owners that recognition validates.
                // A clipped blank advance can belong to its source clip rather
                // than its unpainted, out-of-clip advance box. Neither the source
                // span nor the glyph/line metrics are changed by this selection.
                bounds: super::table_detection::span_ownership_bounds(
                    s,
                    span_clips.get(index).and_then(|clip| clip.as_deref()),
                    media,
                ),
                baseline: media.3 - s.bbox.y,
                font: s.font_size,
            }
        })
        .collect();
    let mut out = Vec::new();
    divide(
        items.iter().collect(),
        Rect::new(0., 0., media.2 - media.0, media.3 - media.1),
        &mut Work(0),
        &mut out,
        0,
    )?;
    Ok(out)
}
