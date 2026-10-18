//! Promote only source separators whose complete paint has a native equivalent.
//! Arbitrary gaps, phase, occlusion and compound paint keep their source owners.
use super::*;
use crate::model::style::{BorderFill, BorderLine};
use pdf_oxide::elements::{LineCap, PathOperation};

pub(super) struct Plan {
    pub borders: Vec<BorderFill>,
    pub controls: Vec<BTreeSet<usize>>,
}

struct Rule {
    control: usize,
    horizontal: bool,
    at: f32,
    from: f32,
    to: f32,
    color: u32,
    width: u32,
}

// Compare actual painted intervals, not a regularized carrier or coverage ratio.
// At least two complete pitches distinguish a native repeat from a single gap.
fn native_dot_coverage(
    segments: &[(f32, f32)],
    from: f32,
    to: f32,
    width: u8,
    observed_width: u32,
    work: &mut usize,
) -> Option<bool> {
    let Some((pattern, stroke)) = crate::model::style::cell_dot_metrics_pt(width) else {
        return Some(false);
    };
    // A nearest HWP width bucket does not prove equivalent source ink. Allow
    // only the one-HWP-unit quantization of the independently printed pen.
    if (observed_width as f64 / 100. - stroke).abs() > 0.0101 {
        return Some(false);
    }
    let [ink, gap] = pattern.map(|v| v as f32);
    let pitch = ink + gap;
    if to - from < 2.0 * pitch || segments.len() < 2 {
        return Some(false);
    }
    let mut actual: Vec<(f32, f32)> = Vec::new();
    for &(a, b) in segments {
        *work += 1;
        if *work > 1_000_000 {
            return None;
        }
        if let Some(last) = actual.last_mut().filter(|last| a <= last.1 + 0.0101) {
            last.1 = last.1.max(b);
        } else {
            actual.push((a, b));
        }
    }
    let mut at = from;
    let mut index = 0;
    while at < to - 0.0101 {
        *work += 1;
        if *work > 1_000_000 {
            return None;
        }
        let Some(&(a, b)) = actual.get(index) else {
            return Some(false);
        };
        if (a - at).abs() > 0.0101 || (b - (at + ink).min(to)).abs() > 0.0101 {
            return Some(false);
        }
        index += 1;
        at = from + index as f32 * pitch;
    }
    Some(index == actual.len())
}

pub(super) fn plan(
    page: &ExtractedPage,
    controls: &[Control],
    rule_controls: &[Option<usize>],
    grid: &Value,
) -> Option<Plan> {
    let mut rules = Vec::new();
    let mut work = 0usize;
    for path in refs(&grid["sourcePathIndices"]).ok()? {
        let source = page.paths.get(path)?;
        // The positional producer has already resolved clipping and color.
        // Require a genuine opaque, uncapped straight stroke as provenance,
        // rather than promoting a bitmap or a coincident content rectangle.
        if !matches!(
            source.operations.as_slice(),
            [PathOperation::MoveTo(..), PathOperation::LineTo(..)]
        ) || source.dash_pattern.is_some()
            || source.fill_color.is_some()
            || source.matrix.is_some()
            || source.paint.stroke_alpha != 1.0
            || source.line_cap != LineCap::Butt
        {
            return None;
        }
        let ci = *rule_controls.get(path)?.as_ref()?;
        let Control::Shape(shape) = controls.get(ci)? else {
            return None;
        };
        let ShapeObject::Rectangle(r) = shape.as_ref() else {
            return None;
        };
        let c = &r.common;
        let (x, y, w, h) = (
            c.horizontal_offset as i32 as f32 / 100.,
            c.vertical_offset as i32 as f32 / 100.,
            c.width as f32 / 100.,
            c.height as f32 / 100.,
        );
        // A short thick dash can be wider than its own length. The real PDF
        // stroke direction, rather than the quad's aspect, owns the axis.
        let horizontal = match source.operations.as_slice() {
            [PathOperation::MoveTo(x1, y1), PathOperation::LineTo(x2, y2)]
                if y1 == y2 && x1 != x2 =>
            {
                true
            }
            [PathOperation::MoveTo(x1, y1), PathOperation::LineTo(x2, y2)]
                if x1 == x2 && y1 != y2 =>
            {
                false
            }
            _ => return None,
        };
        let at = if horizontal { y + h / 2. } else { x + w / 2. };
        let (from, to) = if horizontal { (x, x + w) } else { (y, y + h) };
        let b = &grid["bbox"];
        let (lo, hi) = if horizontal {
            (
                number(&b["x"]).ok()?,
                number(&b["x"]).ok()? + number(&b["width"]).ok()?,
            )
        } else {
            (
                number(&b["y"]).ok()?,
                number(&b["y"]).ok()? + number(&b["height"]).ok()?,
            )
        };
        if from < lo - 0.5
            || to > hi + 0.5
            || !array(
                &grid[if horizontal {
                    "yBoundaries"
                } else {
                    "xBoundaries"
                }],
            )
            .ok()?
            .iter()
            .any(|v| number(v).is_ok_and(|v| (v - at).abs() <= 0.0101))
        {
            return None;
        }
        rules.push(Rule {
            control: ci,
            horizontal,
            at,
            from,
            to,
            color: r.drawing.fill.solid.as_ref()?.background_color,
            width: if horizontal { c.height } else { c.width },
        });
    }
    if rules.is_empty() {
        return None;
    }
    let rule_ids = rules.iter().map(|r| r.control).collect::<BTreeSet<_>>();
    for rule in &rules {
        let Control::Shape(source) = &controls[rule.control] else {
            return None;
        };
        let source = source.common();
        let x = source.horizontal_offset as i32 as i64;
        let y = source.vertical_offset as i32 as i64;
        for (ci, control) in controls.iter().enumerate() {
            work += 1;
            if work > 1_000_000 {
                return None;
            }
            if rule_ids.contains(&ci) {
                continue;
            }
            let c = match control {
                Control::Shape(s) => s.common(),
                Control::Picture(p) => &p.common,
                _ => continue,
            };
            if c.z_order <= source.z_order {
                continue;
            }
            let xx = c.horizontal_offset as i32 as i64;
            let yy = c.vertical_offset as i32 as i64;
            if xx < x + i64::from(source.width)
                && x < xx + i64::from(c.width)
                && yy < y + i64::from(source.height)
                && y < yy + i64::from(c.height)
            {
                // A later object may cover the separator. Keep its original
                // source paint/order rather than interpreting an invisible
                // stroke as a complete editable side.
                return None;
            }
        }
    }
    let mut result = Plan {
        borders: Vec::new(),
        controls: Vec::new(),
    };
    let mut side_controls = Vec::new();
    let mut rule_owners = vec![Vec::new(); rules.len()];
    for cell in array(&grid["cells"]).ok()? {
        let cell_index = result.borders.len();
        let b = &cell["bbox"];
        let (x, y, w, h) = (
            number(&b["x"]).ok()?,
            number(&b["y"]).ok()?,
            number(&b["width"]).ok()?,
            number(&b["height"]).ok()?,
        );
        let mut border = BorderFill::default();
        for side in &mut border.borders {
            side.line_type = BorderLineType::None;
        }
        let mut used = vec![BTreeSet::new(); 4];
        for (side, (horizontal, at, from, to)) in [
            (false, x, y, y + h),
            (false, x + w, y, y + h),
            (true, y, x, x + w),
            (true, y + h, x, x + w),
        ]
        .into_iter()
        .enumerate()
        {
            let mut segments = Vec::new();
            let mut styles = BTreeSet::new();
            for (ri, rule) in rules.iter().enumerate() {
                work += 1;
                if work > 1_000_000 {
                    return None;
                }
                // One HWP unit is 0.01pt. Larger gaps, including regularized
                // short segments, cannot become an unbroken native border.
                if rule.horizontal != horizontal || (rule.at - at).abs() > 0.0101 {
                    continue;
                }
                let (a, b) = (rule.from.max(from), rule.to.min(to));
                if b <= a {
                    continue;
                }
                segments.push((a, b));
                styles.insert((rule.color, rule.width));
                used[side].insert(rule.control);
                rule_owners[ri].push((cell_index, side));
            }
            segments.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut end = from;
            let mut complete = true;
            for &(a, b) in &segments {
                if a > end + 0.0101 {
                    complete = false;
                }
                end = end.max(b);
            }
            if styles.len() != 1 {
                used[side].clear();
                continue;
            }
            let &(color, observed_width) = styles.first()?;
            let width = border_width_index(observed_width as f64 / 100. * 25.4 / 72.);
            let line_type = if complete && end >= to - 0.0101 {
                BorderLineType::Solid
            } else if native_dot_coverage(&segments, from, to, width, observed_width, &mut work)? {
                BorderLineType::Dot
            } else {
                BorderLineType::None
            };
            if line_type == BorderLineType::None {
                used[side].clear();
                continue;
            }
            border.borders[side] = BorderLine {
                line_type,
                width,
                color,
            };
        }
        result.borders.push(border);
        side_controls.push(used);
    }
    // A rule can contribute to both sides of a shared edge or to several
    // unequal spans. Promote it only if every affected side has full evidence;
    // otherwise leave all those sides as source paint. This closure prevents
    // a short native side from duplicating a longer neighbor's retained paint.
    loop {
        let mut blocked = BTreeSet::new();
        for (ri, rule) in rules.iter().enumerate() {
            work += 1 + rule_owners[ri].len();
            if work > 1_000_000 {
                return None;
            }
            if rule_owners[ri].is_empty()
                || rule_owners[ri].iter().any(|&(ci, si)| {
                    result.borders[ci].borders[si].line_type == BorderLineType::None
                })
            {
                blocked.insert(rule.control);
            }
        }
        let mut changed = false;
        for (ci, sides) in side_controls.iter_mut().enumerate() {
            for (si, controls) in sides.iter_mut().enumerate() {
                work += 1 + controls.len();
                if work > 1_000_000 {
                    return None;
                }
                if !controls.is_disjoint(&blocked) {
                    result.borders[ci].borders[si].line_type = BorderLineType::None;
                    controls.clear();
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    result.controls = side_controls
        .into_iter()
        .map(|sides| sides.into_iter().flatten().collect())
        .collect();
    if result.controls.iter().all(BTreeSet::is_empty) {
        return None;
    }
    Some(result)
}
