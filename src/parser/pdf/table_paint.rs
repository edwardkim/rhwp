//! Source paint has one physical owner: a real table cell or the remaining page.
//! Nonstandard border patterns remain cell drawings; cell text is actual paragraph text.
use super::lowering::component;
use crate::model::{
    control::Control,
    shape::{CommonObjAttr, ShapeObject},
    style::FillType,
    Point,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const MAX_WORK: usize = 1_000_000;
const MAX_FRAGMENTS: usize = 32_768;

#[derive(Clone, Copy)]
struct Frame {
    x: i64,
    y: i64,
    right: i64,
    bottom: i64,
}

fn coordinate(v: &Value) -> Result<i64, String> {
    let n = v
        .as_f64()
        .filter(|v| v.is_finite() && v.abs() <= 100_000.)
        .ok_or("invalid cell paint coordinate")?;
    Ok((n * 100.).round() as i64)
}
fn index(v: &Value) -> Result<usize, String> {
    v.as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or("invalid cell paint index".into())
}
fn axis(grid: &Value, key: &str, i: usize) -> Result<i64, String> {
    coordinate(grid[key].get(i).ok_or("cell paint axis missing")?)
}
fn bounds(grid: &Value) -> Result<Frame, String> {
    let nx = grid["xBoundaries"]
        .as_array()
        .ok_or("grid axes missing")?
        .len();
    let ny = grid["yBoundaries"]
        .as_array()
        .ok_or("grid axes missing")?
        .len();
    if nx < 2 || ny < 2 {
        return Err("invalid cell paint axes".into());
    }
    Ok(Frame {
        x: axis(grid, "xBoundaries", 0)?,
        right: axis(grid, "xBoundaries", nx - 1)?,
        y: axis(grid, "yBoundaries", 0)?,
        bottom: axis(grid, "yBoundaries", ny - 1)?,
    })
}
fn cell_frame(grid: &Value, c: &Value) -> Result<Frame, String> {
    let (x, y) = (index(&c["column"])?, index(&c["row"])?);
    let (w, h) = (index(&c["colspan"])?, index(&c["rowspan"])?);
    Ok(Frame {
        x: axis(grid, "xBoundaries", x)?,
        right: axis(
            grid,
            "xBoundaries",
            x.checked_add(w).ok_or("cell axis overflow")?,
        )?,
        y: axis(grid, "yBoundaries", y)?,
        bottom: axis(
            grid,
            "yBoundaries",
            y.checked_add(h).ok_or("cell axis overflow")?,
        )?,
    })
}
pub(super) fn dimensions(grid: &Value, cell: Option<&Value>) -> Result<(u32, u32), String> {
    let f = if let Some(c) = cell {
        cell_frame(grid, c)?
    } else {
        bounds(grid)?
    };
    Ok((
        u32::try_from(f.right - f.x).map_err(|_| "invalid cell width")?,
        u32::try_from(f.bottom - f.y).map_err(|_| "invalid cell height")?,
    ))
}
fn common(c: &Control) -> Option<&CommonObjAttr> {
    match c {
        Control::Shape(s) => Some(s.common()),
        Control::Picture(p) => Some(&p.common),
        _ => None,
    }
}
fn common_mut(c: &mut Control) -> Option<&mut CommonObjAttr> {
    match c {
        Control::Shape(s) => Some(s.common_mut()),
        Control::Picture(p) => Some(&mut p.common),
        _ => None,
    }
}
fn source_frame(c: &Control) -> Option<Frame> {
    let c = common(c)?;
    let (x, y) = (
        c.horizontal_offset as i32 as i64,
        c.vertical_offset as i32 as i64,
    );
    Some(Frame {
        x,
        y,
        right: x + c.width as i64,
        bottom: y + c.height as i64,
    })
}
fn owns_axis(at: i64, start: i64, end: i64, outer_start: i64, outer_end: i64) -> bool {
    (at >= start || (start == outer_start && at >= start - 50))
        && (at < end || (end == outer_end && at <= end + 50))
}
// None is area paint; Some(true/false) is a zero-height/zero-width native line.
fn intersection(a: Frame, b: Frame, line: Option<bool>) -> Option<Frame> {
    let f = Frame {
        x: a.x.max(b.x),
        y: a.y.max(b.y),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    };
    if f.right < f.x
        || f.bottom < f.y
        || (f.right == f.x && line != Some(false))
        || (f.bottom == f.y && line != Some(true))
    {
        None
    } else {
        Some(f)
    }
}
fn measure(f: Frame, line: Option<bool>) -> i64 {
    match line {
        Some(true) => f.right - f.x,
        Some(false) => f.bottom - f.y,
        None => (f.right - f.x) * (f.bottom - f.y),
    }
}
fn tick(work: &mut usize) -> Result<(), String> {
    *work += 1;
    if *work > MAX_WORK {
        Err("cell paint partition work budget exceeded".into())
    } else {
        Ok(())
    }
}
fn subtract(
    remaining: &mut Vec<Frame>,
    cut: Frame,
    line: Option<bool>,
    work: &mut usize,
) -> Result<(), String> {
    let mut next = Vec::new();
    let mut covered = 0;
    for r in remaining.drain(..) {
        tick(work)?;
        let Some(c) = intersection(r, cut, line) else {
            next.push(r);
            continue;
        };
        covered += measure(c, line);
        if line == Some(true) {
            if r.x < c.x {
                next.push(Frame { right: c.x, ..r })
            }
            if c.right < r.right {
                next.push(Frame { x: c.right, ..r })
            }
        } else if line == Some(false) {
            if r.y < c.y {
                next.push(Frame { bottom: c.y, ..r })
            }
            if c.bottom < r.bottom {
                next.push(Frame { y: c.bottom, ..r })
            }
        } else {
            if r.y < c.y {
                next.push(Frame { bottom: c.y, ..r })
            }
            if c.bottom < r.bottom {
                next.push(Frame { y: c.bottom, ..r })
            }
            if r.x < c.x {
                next.push(Frame {
                    y: c.y,
                    bottom: c.bottom,
                    right: c.x,
                    ..r
                })
            }
            if c.right < r.right {
                next.push(Frame {
                    x: c.right,
                    y: c.y,
                    bottom: c.bottom,
                    ..r
                })
            }
        }
    }
    if covered != measure(cut, line) {
        return Err("cell paint partition has a gap or duplicate owner".into());
    }
    *remaining = next;
    Ok(())
}

fn fragment(original: &Control, source: Frame, piece: Frame) -> Result<Control, String> {
    let mut c = original.clone();
    let (w, h) = (
        (piece.right - piece.x) as u32,
        (piece.bottom - piece.y) as u32,
    );
    let a = common_mut(&mut c).ok_or("unsupported source paint")?;
    a.horizontal_offset = piece.x as i32 as u32;
    a.vertical_offset = piece.y as i32 as u32;
    a.width = w;
    a.height = h;
    match &mut c {
        Control::Shape(s) => match s.as_mut() {
            ShapeObject::Rectangle(r) => {
                r.x_coords = [0, w as i32, w as i32, 0];
                r.y_coords = [0, 0, h as i32, h as i32];
                r.drawing.shape_attr = component(w, h)
            }
            ShapeObject::Line(l) => {
                let reverse = l.start.x > l.end.x || l.start.y > l.end.y;
                let a = Point { x: 0, y: 0 };
                let b = Point {
                    x: w as i32,
                    y: h as i32,
                };
                (l.start, l.end) = if reverse { (b, a) } else { (a, b) };
                l.drawing.shape_attr = component(w, h)
            }
            _ => return Err("unsupported source paint shape".into()),
        },
        Control::Picture(p) => {
            // A path bitmap has a physical frame and a selected source region.
            // Subdivide both by the same map. Reusing the entire bitmap in each
            // smaller frame would shrink the dash pattern and lose its phase.
            let old = match original {
                Control::Picture(p) => p,
                _ => unreachable!(),
            };
            let crop = old.crop;
            let select = |at: i64, start: i64, end: i64, lo: i32, hi: i32| {
                let width = end - start;
                if width <= 0 || hi <= lo || at < start || at > end {
                    return Err("invalid source paint image selection".to_string());
                }
                let offset = ((at - start) * (i64::from(hi) - i64::from(lo)) + width / 2) / width;
                i32::try_from(i64::from(lo) + offset)
                    .map_err(|_| "source paint image selection overflow".into())
            };
            p.crop = crate::model::image::CropInfo {
                left: select(piece.x, source.x, source.right, crop.left, crop.right)?,
                right: select(piece.right, source.x, source.right, crop.left, crop.right)?,
                top: select(piece.y, source.y, source.bottom, crop.top, crop.bottom)?,
                bottom: select(piece.bottom, source.y, source.bottom, crop.top, crop.bottom)?,
            };
            if p.crop.right <= p.crop.left || p.crop.bottom <= p.crop.top {
                return Err("cell paint image fragment is below source precision".into());
            }
            p.shape_attr = component(w, h);
            // Picture's eight coordinates are packed point pairs (not X/Y arrays).
            p.border_x = [0, 0, w as i32, 0];
            p.border_y = [w as i32, h as i32, 0, h as i32];
        }
        _ => return Err("unsupported source paint control".into()),
    }
    Ok(c)
}

pub(super) struct Target<'a> {
    pub grid: &'a Value,
    pub native_border_controls: Option<&'a [BTreeSet<usize>]>,
}
pub(super) struct PaintPlan {
    pub cells: Vec<Vec<Vec<Control>>>,
    pub removed: BTreeSet<usize>,
    pub retained: Vec<Control>,
    pub native_border_fragments: usize,
    /// Boundary contributors may lie outside a logical content frame. Such
    /// physical paint keeps its original page owner, rather than being moved
    /// into a cell or treated as missing solely because it was not removed.
    pub external_boundary_controls: BTreeSet<usize>,
}

/// A failed family must not discard independent roots. Only explicitly
/// identified unconsumed targets may be isolated; all other errors stay atomic.
pub(super) struct PartitionError {
    pub reason: String,
    pub unconsumed_targets: Vec<usize>,
}
impl From<String> for PartitionError {
    fn from(reason: String) -> Self {
        Self {
            reason,
            unconsumed_targets: Vec::new(),
        }
    }
}
impl From<&str> for PartitionError {
    fn from(reason: &str) -> Self {
        reason.to_owned().into()
    }
}
impl std::fmt::Display for PartitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

pub(super) fn partition(
    controls: &[Control],
    rule_controls: &[Option<usize>],
    targets: &[Target<'_>],
    detection: &Value,
    work: &mut usize,
) -> Result<PaintPlan, PartitionError> {
    let mut orientations = BTreeMap::<usize, Option<bool>>::new();
    for boundary in detection["boundaryObservations"]
        .as_array()
        .ok_or("boundary observations missing")?
    {
        let path = index(&boundary["path"])?;
        let horizontal = boundary["horizontal"]
            .as_bool()
            .ok_or("invalid boundary orientation")?;
        orientations
            .entry(path)
            .and_modify(|known| {
                if *known != Some(horizontal) {
                    *known = None
                }
            })
            .or_insert(Some(horizontal));
    }
    let regions = targets
        .iter()
        .map(|t| bounds(t.grid))
        .collect::<Result<Vec<_>, _>>()?;
    let frames = targets
        .iter()
        .map(|t| {
            t.grid["cells"]
                .as_array()
                .ok_or("cell grid missing")?
                .iter()
                .map(|c| cell_frame(t.grid, c))
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let required = targets
        .iter()
        .map(|t| {
            t.grid["sourcePathIndices"]
                .as_array()
                .ok_or("source paint refs missing")?
                .iter()
                .map(|v| {
                    rule_controls
                        .get(index(v)?)
                        .copied()
                        .flatten()
                        .ok_or("source boundary has no editable paint".into())
                })
                .collect::<Result<BTreeSet<_>, String>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = PaintPlan {
        cells: frames.iter().map(|v| vec![Vec::new(); v.len()]).collect(),
        removed: BTreeSet::new(),
        retained: Vec::new(),
        native_border_fragments: 0,
        external_boundary_controls: BTreeSet::new(),
    };
    // A path bitmap is still source paint, not a separately placed content
    // image. Intersections between two such paths keep their original z-order.
    let path_controls = rule_controls
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut count = 0;
    for (path, control) in rule_controls.iter().enumerate() {
        let Some(control) = *control else { continue };
        let original = controls.get(control).ok_or("invalid paint control")?;
        let Some(source) = source_frame(original) else {
            continue;
        };
        if !regions.iter().any(|b| {
            source.x <= b.right + 50
                && source.right >= b.x - 50
                && source.y <= b.bottom + 50
                && source.bottom >= b.y - 50
        }) {
            continue;
        }
        let line = match original {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Rectangle(r)
                    if r.drawing.text_box.is_none()
                        && r.drawing.fill.fill_type == FillType::Solid
                        && r.drawing.border_line.width == 0
                        && r.x_coords == [0, r.common.width as i32, r.common.width as i32, 0]
                        && r.y_coords == [0, 0, r.common.height as i32, r.common.height as i32] =>
                {
                    None
                }
                ShapeObject::Line(l) if l.start.y == l.end.y => Some(true),
                ShapeObject::Line(l) if l.start.x == l.end.x => Some(false),
                _ => return Err("unsupported cell paint shape requires further lowering".into()),
            },
            // These controls come exclusively from positional path lowering,
            // never the separate source-image list. Their crop keeps native
            // path winding, alpha and clip; the physical partition stays shared.
            Control::Picture(_) => None,
            _ => return Err("unsupported cell paint requires further lowering".into()),
        };
        // Area paint includes its physical width, whether a bitmap or a filled
        // stroke quad. Its two sides must belong to their physical cells.
        // Giving the entire band to one side lets that cell's clip erase the
        // other half. Outside-root pieces retain their original page owner.
        let image = matches!(original, Control::Picture(_));
        let orientation = if image {
            None
        } else {
            line.or_else(|| orientations.get(&path).copied().flatten())
        };
        let mut remaining = vec![source];
        let mut owned = 0;
        for (ti, cells) in frames.iter().enumerate() {
            let region = regions[ti];
            let on_axis = orientation.filter(|h| {
                let at = if *h {
                    (source.y + source.bottom) / 2
                } else {
                    (source.x + source.right) / 2
                };
                targets[ti].grid[if *h { "yBoundaries" } else { "xBoundaries" }]
                    .as_array()
                    .is_some_and(|a| {
                        a.iter()
                            .any(|v| coordinate(v).is_ok_and(|x| (x - at).abs() <= 50))
                    })
            });
            for (ci, cell) in cells.iter().enumerate() {
                tick(work)?;
                let mut clip = *cell;
                let native_border = targets[ti]
                    .native_border_controls
                    .and_then(|cells| cells.get(ci))
                    .is_some_and(|set| set.contains(&control));
                if let Some(h) = on_axis.filter(|_| line.is_some() || native_border) {
                    let (at, lo, hi, outer_lo, outer_hi) = if h {
                        (
                            (source.y + source.bottom) / 2,
                            cell.y,
                            cell.bottom,
                            region.y,
                            region.bottom,
                        )
                    } else {
                        (
                            (source.x + source.right) / 2,
                            cell.x,
                            cell.right,
                            region.x,
                            region.right,
                        )
                    };
                    if !owns_axis(at, lo, hi, outer_lo, outer_hi) {
                        continue;
                    }
                    if h {
                        clip.y = source.y;
                        clip.bottom = source.bottom
                    } else {
                        clip.x = source.x;
                        clip.right = source.right
                    }
                }
                if (line.is_some() || native_border) && required[ti].contains(&control) {
                    if cell.x == region.x {
                        clip.x = clip.x.min(source.x)
                    }
                    if cell.right == region.right {
                        clip.right = clip.right.max(source.right)
                    }
                    if cell.y == region.y {
                        clip.y = clip.y.min(source.y)
                    }
                    if cell.bottom == region.bottom {
                        clip.bottom = clip.bottom.max(source.bottom)
                    }
                }
                // A deeper cell already owns its part of this source. Parent
                // cells can consume only the remaining physical pieces. The
                // same subtraction/conservation checks protect flat siblings.
                let mut pieces = Vec::new();
                for &unowned in &remaining {
                    tick(work)?;
                    if let Some(piece) = intersection(unowned, clip, line) {
                        pieces.push(piece);
                    }
                }
                for piece in pieces {
                    // Only background paint can move below cell text/images. Borders
                    // have independent geometric separator evidence; compound border
                    // pictures may have a large transparent frame.
                    if on_axis.is_none() && !required[ti].contains(&control) {
                        for (source_control, c) in controls.iter().enumerate() {
                            tick(work)?;
                            if path_controls.contains(&source_control) {
                                continue;
                            }
                            let text_or_image = matches!(c, Control::Picture(_))
                                || matches!(c,Control::Shape(s)if s.drawing().is_some_and(|d|d.text_box.is_some()));
                            if text_or_image
                                && common(c)
                                    .is_some_and(|c| c.z_order < common(original).unwrap().z_order)
                                && source_frame(c)
                                    .is_some_and(|f| intersection(f, piece, None).is_some())
                            {
                                return Err(
                                    "interleaved cell background requires further lowering".into(),
                                );
                            }
                        }
                    }
                    subtract(&mut remaining, piece, line, work)?;
                    owned += measure(piece, line);
                    count += 1;
                    if count > MAX_FRAGMENTS {
                        return Err("cell paint fragment budget exceeded".into());
                    }
                    if native_border {
                        // This exact physical piece is represented by the
                        // cell's native border. Keep subtraction/conservation
                        // and all other owners; do not paint it a second time.
                        result.native_border_fragments += 1;
                    } else {
                        result.cells[ti][ci].push(fragment(original, source, piece)?);
                    }
                }
            }
        }
        if owned == 0 {
            continue;
        }
        if owned + remaining.iter().map(|r| measure(*r, line)).sum::<i64>() != measure(source, line)
        {
            return Err("source paint measure changed".into());
        }
        for r in remaining {
            count += 1;
            if count > MAX_FRAGMENTS {
                return Err("cell paint fragment budget exceeded".into());
            }
            result.retained.push(fragment(original, source, r)?)
        }
        result.removed.insert(control);
    }
    let mut unconsumed_targets = Vec::new();
    for (ti, required) in required.iter().enumerate() {
        let mut unresolved = false;
        for &control in required.difference(&result.removed) {
            tick(work)?;
            let original = controls.get(control).ok_or("invalid boundary control")?;
            let source = source_frame(original).ok_or("boundary paint frame missing")?;
            let line = match original {
                Control::Shape(s) => match s.as_ref() {
                    ShapeObject::Line(l) if l.start.y == l.end.y => Some(true),
                    ShapeObject::Line(l) if l.start.x == l.end.x => Some(false),
                    _ => None,
                },
                _ => None,
            };
            if measure(source, line) > 0 && intersection(source, regions[ti], line).is_none() {
                // No physical piece belongs to this table. The unchanged
                // control is still in the page's unremoved source list. A
                // geometric contributor within the inference tolerance does
                // not authorize changing its position or clipping its ink.
                result.external_boundary_controls.insert(control);
            } else {
                unresolved = true;
            }
        }
        if unresolved {
            unconsumed_targets.push(ti);
        }
    }
    if !unconsumed_targets.is_empty() {
        return Err(PartitionError {
            reason: "unconsumed source boundaries".into(),
            unconsumed_targets,
        });
    }
    Ok(result)
}

pub(super) fn assign_id(c: &mut Control, id: u32) {
    if let Some(common) = common_mut(c) {
        common.instance_id = id
    }
    match c {
        Control::Shape(s) => {
            if let Some(d) = s.drawing_mut() {
                d.inst_id = id
            }
        }
        Control::Picture(p) => p.instance_id = id,
        _ => {}
    }
}
pub(super) fn anchor(
    c: &mut Control,
    cell: &crate::model::table::Cell,
    bounds: &Value,
    id: u32,
) -> Result<(), String> {
    use crate::model::shape::{HorzRelTo, TextWrap, VertRelTo};
    // Boundary paint belongs to the cell frame, independently of text indentation.
    // Both picture and shape Column anchors consume the inner cell origin.
    let x = coordinate(&bounds["x"])? + cell.padding.left as i64;
    let y = coordinate(&bounds["y"])? + cell.padding.top as i64;
    let a = common_mut(c).ok_or("cell paint control frame missing")?;
    a.horizontal_offset = (a.horizontal_offset as i32 as i64 - x) as i32 as u32;
    a.vertical_offset = (a.vertical_offset as i32 as i64 - y) as i32 as u32;
    a.horz_rel_to = HorzRelTo::Column;
    a.vert_rel_to = VertRelTo::Para;
    // Hancom's restrict-in-page bit establishes the cell-local frame here.
    // It also enforces content extent: the host's paragraph lead must not be
    // fabricated as cell padding and added to this paint's full frame height.
    a.flow_with_text = true;
    a.treat_as_char = false;
    a.text_wrap = TextWrap::BehindText;
    assign_id(c, id);
    Ok(())
}
