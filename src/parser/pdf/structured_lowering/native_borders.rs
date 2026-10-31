//! Promote only source separators whose complete paint has a native equivalent.
//! Arbitrary gaps, phase, occlusion and compound paint keep their source owners.
use super::*;
use crate::model::style::{BorderFill, BorderLine};
use pdf_oxide::elements::{LineCap, LineJoin, PathOperation};

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
    endpoint_extension: f32,
}

// A neighboring rule's short end extension may fall inside this logical side.
// It must not impose its style on the entire side when a same-color crossing
// pen already covers that exact ink. Short independent strokes remain evidence.
fn covered_join(
    rules: &[Rule],
    rule: &Rule,
    from: f32,
    to: f32,
    a: f32,
    b: f32,
    work: &mut usize,
) -> Option<bool> {
    let boundary = if rule.from < from && rule.to < to {
        from
    } else if rule.to > to && rule.from > from {
        to
    } else {
        return Some(false);
    };
    for cross in rules {
        *work += 1;
        if *work > 1_000_000 {
            return None;
        }
        let half = cross.width as f32 / 200.;
        let own_half = rule.width as f32 / 200.;
        if cross.horizontal != rule.horizontal
            && cross.color == rule.color
            && (cross.at - boundary).abs() <= 0.0101
            && a >= cross.at - half - 0.0101
            && b <= cross.at + half + 0.0101
            && rule.at - own_half >= cross.from - 0.0101
            && rule.at + own_half <= cross.to + 0.0101
        {
            return Some(true);
        }
    }
    Some(false)
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
    allocation: Allocation<'_>,
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
            allocation.push(&mut actual, (a, b)).ok()?;
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
    let allocation = Allocation(&page.import_budget);
    let mut rules = Vec::new();
    let mut work = 0usize;
    for path in refs(&grid["sourcePathIndices"], allocation).ok()? {
        let source = page.paths.get(path)?;
        // A genuine closed rectangular stroke has the same four editable
        // cell sides as four separate strokes. Do not keep a cell-sized bitmap
        // merely because the PDF emitted `re S` instead of four `m/l/S` paths.
        // Clipped/rounded/transparent/filled/compound paths retain their paint.
        if matches!(source.operations.as_slice(), [PathOperation::Rectangle(..)]) {
            let clip_complete = match source.paint.clip {
                pdf_oxide::extractors::image_clip::ImageClip::Unbounded => true,
                pdf_oxide::extractors::image_clip::ImageClip::Rectangle(clip) => {
                    let bounds = source.rendered_bbox();
                    bounds.x >= clip.x
                        && bounds.y >= clip.y
                        && bounds.x + bounds.width <= clip.x + clip.width
                        && bounds.y + bounds.height <= clip.y + clip.height
                }
                _ => false,
            };
            if source.fill_color.is_some()
                || source.dash_pattern.is_some()
                || source.matrix.is_some()
                || source.paint.stroke_alpha != 1.
                || source.line_join != LineJoin::Miter
                || !clip_complete
                || !source.stroke_width.is_finite()
                || source.stroke_width <= 0.
            {
                return None;
            }
            let color = super::super::lowering::rgb(source.stroke_color?);
            let control = *rule_controls.get(path)?.as_ref()?;
            let observations =
                page.snapshot["tableDetection"]["boundaryObservations"].as_array()?;
            let mut count = 0;
            for observed in observations {
                work += 1;
                if work > 1_000_000 {
                    return None;
                }
                if observed["path"].as_u64()? as usize != path {
                    continue;
                }
                if observed["paintKind"] != "stroke" {
                    return None;
                }
                let horizontal = observed["horizontal"].as_bool()?;
                let at = number(&observed["at"]).ok()?;
                let from = number(&observed["from"]).ok()?;
                let to = number(&observed["to"]).ok()?;
                if ![at, from, to].iter().all(|n| n.is_finite()) || to <= from {
                    return None;
                }
                allocation
                    .push(
                        &mut rules,
                        Rule {
                            control,
                            horizontal,
                            at,
                            from,
                            to,
                            color,
                            width: hu(source.stroke_width).ok()?,
                            endpoint_extension: source.stroke_width / 2.,
                        },
                    )
                    .ok()?;
                count += 1;
            }
            if count != 4 {
                return None;
            }
            continue;
        }
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
        allocation
            .push(
                &mut rules,
                Rule {
                    control: ci,
                    horizontal,
                    at,
                    from,
                    to,
                    color: r.drawing.fill.solid.as_ref()?.background_color,
                    width: if horizontal { c.height } else { c.width },
                    endpoint_extension: 0.,
                },
            )
            .ok()?;
    }
    if rules.is_empty() {
        return None;
    }
    let rule_ids = allocation.set_from(rules.iter().map(|r| r.control)).ok()?;
    for rule in &rules {
        let source = match &controls[rule.control] {
            Control::Shape(s) => s.common(),
            Control::Picture(p) => &p.common,
            _ => return None,
        };
        // Test the actual edge ink, not a transparent rectangle bitmap's whole
        // frame. Text inside a border does not occlude that border. Miter corners
        // extend by half a pen; flat-cap straight strokes keep their endpoints.
        let half = rule.width as f32 / 200.;
        let (x, y, w, h) = if rule.horizontal {
            (
                rule.from - rule.endpoint_extension,
                rule.at - half,
                rule.to - rule.from + 2. * rule.endpoint_extension,
                2. * half,
            )
        } else {
            (
                rule.at - half,
                rule.from - rule.endpoint_extension,
                2. * half,
                rule.to - rule.from + 2. * rule.endpoint_extension,
            )
        };
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
            let xx = c.horizontal_offset as i32 as f32 / 100.;
            let yy = c.vertical_offset as i32 as f32 / 100.;
            if xx < x + w
                && x < xx + c.width as f32 / 100.
                && yy < y + h
                && y < yy + c.height as f32 / 100.
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
    let mut rule_owners = allocation
        .collect((0..rules.len()).map(|_| Vec::new()))
        .ok()?;
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
        let mut used = allocation.collect((0..4).map(|_| BTreeSet::new())).ok()?;
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
                if covered_join(&rules, rule, from, to, a, b, &mut work)? {
                    continue;
                }
                allocation.push(&mut segments, (a, b)).ok()?;
                allocation
                    .insert_set(&mut styles, (rule.color, rule.width))
                    .ok()?;
                allocation.insert_set(&mut used[side], rule.control).ok()?;
                allocation
                    .push(&mut rule_owners[ri], (cell_index, side))
                    .ok()?;
            }
            allocation
                .sort_by(&mut segments, |a, b| a.0.total_cmp(&b.0))
                .ok()?;
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
            } else if native_dot_coverage(
                &segments,
                from,
                to,
                width,
                observed_width,
                &mut work,
                allocation,
            )? {
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
        allocation.push(&mut result.borders, border).ok()?;
        allocation.push(&mut side_controls, used).ok()?;
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
                allocation.insert_set(&mut blocked, rule.control).ok()?;
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
    result.controls = allocation
        .try_collect(
            side_controls
                .into_iter()
                .map(|sides| allocation.set_from(sides.into_iter().flatten())),
        )
        .ok()?;
    if result.controls.iter().all(BTreeSet::is_empty) {
        return None;
    }
    Some(result)
}
