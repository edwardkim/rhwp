//! Structural edits may inherit a native border only after proving the complete
//! retained source ink has that native equivalent. Arbitrary paint is not a style.
use crate::{
    document_core::DocumentCore,
    error::HwpError,
    model::{
        bin_data::{self, BinDataContent},
        border_paint::{
            outside_binding, outside_owner, picture_digest, valid_paint, CellBorderPaint, PaintRef,
            Resources,
        },
        control::Control,
        document::Document,
        paragraph::Paragraph,
        pdf_stroke::PdfStroke,
        shape::{HorzAlign, HorzRelTo, VertAlign, VertRelTo},
        style::{border_width_index, BorderFill, BorderLineType, BORDER_WIDTHS},
        table::Table,
    },
};
use std::collections::BTreeMap;

const UNSUPPORTED: &str = "PDF 경계의 행·열 편집에는 완전한 연속 불투명 실선의 동등성이 필요합니다";
fn error() -> HwpError {
    HwpError::RenderError(UNSUPPORTED.into())
}
type Rect = [f64; 4];
type Key = (bool, usize, usize); // horizontal, boundary index, interval index
struct Edge {
    color: u32,
    width: Option<u8>,
    rectangle: bool,
    pen: f64,
    source_pen: f64,
    rects: Vec<Rect>,
}
struct Proof<'a> {
    table: &'a Table,
    x: Vec<f64>,
    y: Vec<f64>,
    resources: Resources<'a>,
    contents: &'a [BinDataContent],
    strokes: BTreeMap<u16, (PdfStroke, [f64; 2])>,
    edges: BTreeMap<Key, Edge>,
    work: usize,
    z: [[Option<i32>; 2]; 2], // inside/outside, vertical/horizontal
    layer_range: Option<(i32, i32)>,
}
fn boundaries(sizes: Vec<u32>) -> Vec<f64> {
    let mut out = vec![0.];
    for n in sizes {
        out.push(out.last().unwrap() + f64::from(n));
    }
    out
}
fn key(c: &crate::model::table::Cell, side: usize) -> Key {
    match side {
        0 => (false, c.col as usize, c.row as usize),
        1 => (false, c.col as usize + 1, c.row as usize),
        2 => (true, c.row as usize, c.col as usize),
        _ => (true, c.row as usize + 1, c.col as usize),
    }
}
fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let r = [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ];
    (r[0] < r[2] && r[1] < r[3]).then_some(r)
}
impl Proof<'_> {
    fn tick(&mut self) -> Result<(), HwpError> {
        self.work += 1;
        if self.work > 200_000 {
            return Err(error());
        }
        Ok(())
    }
    fn add(
        &mut self,
        ctrl: &Control,
        ci: usize,
        tag: CellBorderPaint,
        outside: bool,
    ) -> Result<(), HwpError> {
        self.tick()?;
        let z = PaintRef::from_control(ctrl)
            .ok_or_else(error)?
            .common()
            .z_order;
        let range = self.layer_range.get_or_insert((z, z));
        range.0 = range.0.min(z);
        range.1 = range.1.max(z);
        let mask = tag.mask().ok_or_else(error)?;
        for side in 0..4 {
            if mask & (1 << side) != 0 {
                let slot = &mut self.z[usize::from(outside)][side / 2];
                slot.get_or_insert(z);
            }
        }
        if let Control::Shape(shape) = ctrl {
            return self.add_rectangle(shape, ci, tag, outside);
        }
        let Control::Picture(p) = ctrl else {
            return Err(error());
        };
        let id = p.image_attr.bin_data_id;
        if let std::collections::btree_map::Entry::Vacant(e) = self.strokes.entry(id) {
            let i = bin_data::content_index(self.contents, id).ok_or_else(error)?;
            let bytes = self.contents[i]
                .data
                .load_limited(bin_data::MAX_BIN_DATA_BYTES)
                .ok_or_else(error)?;
            let s = PdfStroke::read(&bytes).ok_or_else(error)?;
            if s.alpha != 1. || !s.dash.is_empty() {
                return Err(error());
            }
            let dim = [
                u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as f64 * 75.,
                u32::from_be_bytes(bytes[20..24].try_into().unwrap()) as f64 * 75.,
            ];
            e.insert((s, dim));
        }
        let (s, dim) = &self.strokes[&id];
        let c = &self.table.cells[ci];
        let a = &p.common;
        let origin = if outside {
            [
                f64::from(a.horizontal_offset as i32)
                    - f64::from(self.table.common.horizontal_offset as i32),
                f64::from(a.vertical_offset as i32)
                    - f64::from(self.table.common.vertical_offset as i32),
            ]
        } else {
            [
                self.x[c.col as usize]
                    + f64::from(c.padding.left)
                    + f64::from(a.horizontal_offset as i32),
                self.y[c.row as usize]
                    + f64::from(c.padding.top)
                    + f64::from(a.vertical_offset as i32),
            ]
        };
        let crop = [
            f64::from(p.crop.left) / dim[0] * s.frame[0],
            f64::from(p.crop.top) / dim[1] * s.frame[1],
            f64::from(p.crop.right) / dim[0] * s.frame[0],
            f64::from(p.crop.bottom) / dim[1] * s.frame[1],
        ];
        let scale = [
            f64::from(a.width) / (crop[2] - crop[0]),
            f64::from(a.height) / (crop[3] - crop[1]),
        ];
        let map = |r: Rect| {
            [
                origin[0] + (r[0] - crop[0]) * scale[0],
                origin[1] + (r[1] - crop[1]) * scale[1],
                origin[0] + (r[2] - crop[0]) * scale[0],
                origin[1] + (r[3] - crop[1]) * scale[1],
            ]
        };
        let mask = tag.mask().ok_or_else(error)?;
        for side in 0..4 {
            if mask & (1 << side) == 0 {
                continue;
            }
            let k = key(c, side);
            let (at, lo, hi) = if k.0 {
                (self.y[k.1], self.x[k.2], self.x[k.2 + 1])
            } else {
                (self.x[k.1], self.y[k.2], self.y[k.2 + 1])
            };
            let pen = s.width * scale[if k.0 { 1 } else { 0 }];
            let width = border_width_index(pen * 25.4 / 7200.);
            // One source HWPUNIT of quantization, never a nearest-bucket visual approximation.
            let width = ((BORDER_WIDTHS[width as usize].0 * 7200. / 25.4 - pen).abs() <= 1.00001)
                .then_some(width);
            // PdfStroke already carries the model's ColorRef, like BorderFill.
            let color = s.color;
            let edge = self.edges.entry(k).or_insert_with(|| Edge {
                color,
                width,
                pen: width.map_or(pen, |w| BORDER_WIDTHS[w as usize].0 * 7200. / 25.4),
                source_pen: pen,
                rectangle: false,
                rects: vec![],
            });
            // Promote only when every fragment proves the same native width.
            // Near a bucket boundary, independently quantized halves can have
            // different native eligibility while still describing one source
            // pen. Preserve that pen instead of rejecting it or rounding it.
            let same_native = edge.width.is_some() && edge.width == width;
            if edge.rectangle
                || edge.color != color
                || ((if same_native {
                    edge.pen
                } else {
                    edge.source_pen
                }) - pen)
                    .abs()
                    > 1.00001
            {
                return Err(error());
            }
            if !same_native {
                edge.width = None;
                edge.pen = edge.source_pen;
            }
            for e in std::iter::once(s.endpoints).chain(s.segments.iter().copied()) {
                self.work += 1;
                if self.work > 200_000 {
                    return Err(error());
                }
                if (e[1] == e[3]) != k.0 {
                    continue;
                }
                let center = map(e);
                if ((if k.0 { center[1] } else { center[0] }) - at).abs() > 1. {
                    continue;
                }
                let band = if k.0 {
                    [
                        e[0].min(e[2]),
                        e[1] - s.width / 2.,
                        e[0].max(e[2]),
                        e[1] + s.width / 2.,
                    ]
                } else {
                    [
                        e[0] - s.width / 2.,
                        e[1].min(e[3]),
                        e[0] + s.width / 2.,
                        e[1].max(e[3]),
                    ]
                };
                if let Some(r) = intersect(band, crop).map(map) {
                    let want = if k.0 {
                        [lo, at - pen / 2., hi, at + pen / 2.]
                    } else {
                        [at - pen / 2., lo, at + pen / 2., hi]
                    };
                    if let Some(r) = intersect(r, want) {
                        edge.rects.push(r);
                    }
                }
            }
        }
        Ok(())
    }
    fn add_rectangle(
        &mut self,
        shape: &crate::model::shape::ShapeObject,
        ci: usize,
        tag: CellBorderPaint,
        outside: bool,
    ) -> Result<(), HwpError> {
        use crate::model::shape::ShapeObject;
        let ShapeObject::Rectangle(r) = shape else {
            return Err(error());
        };
        let mask = tag.mask().ok_or_else(error)?;
        // A single axial band has an unambiguous cross-section. Other retained
        // fills/compound roles are never reinterpreted as a complete solid pen.
        if mask.count_ones() != 1 {
            return Err(error());
        }
        let side = mask.trailing_zeros() as usize;
        let c = &self.table.cells[ci];
        let a = &r.common;
        let x = f64::from(a.horizontal_offset as i32)
            + if outside {
                -f64::from(self.table.common.horizontal_offset as i32)
            } else {
                self.x[c.col as usize] + f64::from(c.padding.left)
            };
        let y = f64::from(a.vertical_offset as i32)
            + if outside {
                -f64::from(self.table.common.vertical_offset as i32)
            } else {
                self.y[c.row as usize] + f64::from(c.padding.top)
            };
        let color = r
            .drawing
            .fill
            .solid
            .as_ref()
            .ok_or_else(error)?
            .background_color;
        let edge = self.edges.entry(key(c, side)).or_insert_with(|| Edge {
            color,
            width: None,
            pen: 0.,
            source_pen: 0.,
            rectangle: true,
            rects: Vec::new(),
        });
        if !edge.rectangle || edge.color != color {
            return Err(error());
        }
        edge.rects
            .push([x, y, x + f64::from(a.width), y + f64::from(a.height)]);
        Ok(())
    }
    fn complete(&mut self) -> Result<(), HwpError> {
        for (&(h, line, _), edge) in &mut self.edges {
            if !edge.rectangle {
                continue;
            }
            let at = if h { self.y[line] } else { self.x[line] };
            let cross = if h { 1 } else { 0 };
            let lo = edge
                .rects
                .iter()
                .map(|r| r[cross])
                .fold(f64::INFINITY, f64::min);
            let hi = edge
                .rects
                .iter()
                .map(|r| r[cross + 2])
                .fold(f64::NEG_INFINITY, f64::max);
            if ((lo + hi) / 2. - at).abs() > 0.5 {
                return Err(error());
            }
            edge.pen = hi - lo;
            // Retain the exact band on subsequent edits; do not promote the
            // integer split of an odd pen into a slightly wider native bucket.
        }
        // Cover the full physical pen, including the neighbour/outside half.
        // A narrow crop or a partially erased side never acquires a full border.
        for (&(h, line, interval), edge) in &self.edges {
            let (at, lo, hi) = if h {
                (self.y[line], self.x[interval], self.x[interval + 1])
            } else {
                (self.x[line], self.y[interval], self.y[interval + 1])
            };
            let along = if h { 0 } else { 1 };
            let cross = 1 - along;
            let mut cuts = vec![lo, hi];
            for r in &edge.rects {
                // Physical HWP crop endpoints are integer HWPUNITs; their
                // inverse source projection can differ by one such unit.
                // This bounds proof uncertainty, never changes painted geometry.
                cuts.extend([r[along] - 1., r[along + 2] + 1.]);
            }
            cuts.sort_by(f64::total_cmp);
            cuts.dedup();
            for pair in cuts.windows(2) {
                self.work += edge.rects.len();
                if self.work > 200_000 {
                    return Err(error());
                }
                let mid = (pair[0] + pair[1]) / 2.;
                if mid < lo || mid > hi {
                    continue;
                }
                let mut bands: Vec<_> = edge
                    .rects
                    .iter()
                    .filter(|r| r[along] - 1. <= mid && r[along + 2] + 1. >= mid)
                    .map(|r| (r[cross], r[cross + 2]))
                    .collect();
                bands.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut end = at - edge.pen / 2.;
                for (a, b) in bands {
                    if a > end + 1. {
                        return Err(error());
                    }
                    end = end.max(b);
                }
                if end < at + edge.pen / 2. - 1. {
                    return Err(error());
                }
            }
        }
        Ok(())
    }
}

struct Prepared {
    parent: Paragraph,
    fills: Vec<BorderFill>,
    bands: Option<super::border_paint_bands::Bands>,
}

/// Opaque ink of one colour commutes with itself. Coalescing its source ranks
/// is safe only if no noncommuting drawing lies between (or ties) those ranks.
/// Inspect the candidate after removing only authenticated border pieces; the
/// nested table/content controls remain in this scan. Unranked drawing types
/// are not presumed harmless. No bbox proximity or hidden-paint assumption.
fn preserves_layer_order(
    doc: &Document,
    si: usize,
    pi: usize,
    candidate: &Paragraph,
    range: (i32, i32),
    strokes: &mut BTreeMap<u16, (PdfStroke, [f64; 2])>,
    color: u32,
) -> bool {
    if range.0 == range.1 {
        return true;
    }
    let mut stack = Vec::new();
    let mut resources = Resources::new(&doc.bin_data_content);
    for (s, section) in doc.sections.iter().enumerate() {
        for (p, paragraph) in section.paragraphs.iter().enumerate() {
            let paragraph = if s == si && p == pi {
                candidate
            } else {
                paragraph
            };
            if paragraph.controls.len() > 200_000 - stack.len() {
                return false;
            }
            stack.extend(&paragraph.controls);
        }
    }
    let mut work = 0;
    while let Some(control) = stack.pop() {
        work += 1;
        if work > 200_000 {
            return false;
        }
        let common = match control {
            Control::Picture(p) => Some(&p.common),
            Control::Table(t) => {
                for cell in &t.cells {
                    for p in &cell.paragraphs {
                        if p.controls.len() > 200_000 - stack.len() {
                            return false;
                        }
                        stack.extend(&p.controls);
                    }
                }
                Some(&t.common)
            }
            Control::Shape(s) => {
                if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                    for p in &tb.paragraphs {
                        if p.controls.len() > 200_000 - stack.len() {
                            return false;
                        }
                        stack.extend(&p.controls);
                    }
                }
                Some(s.common())
            }
            Control::Equation(e) => Some(&e.common),
            Control::Form(_) | Control::Unknown(_) => return false,
            _ => None,
        };
        if stack.len() > 200_000 {
            return false;
        }
        if !common.is_some_and(|a| (range.0..=range.1).contains(&a.z_order)) {
            continue;
        }
        // After save/reopen every owned border may already be a rectangle.
        // Verify the retained PNG independently of having an owned PNG peer.
        // A successful colour/order proof never grants deletion authority.
        let Control::Picture(p) = control else {
            return false;
        };
        if p.border_attr.width != 0 || picture_digest(p, &mut resources).is_none() {
            return false;
        }
        let id = p.image_attr.bin_data_id;
        if let std::collections::btree_map::Entry::Vacant(entry) = strokes.entry(id) {
            let Some(i) = bin_data::content_index(&doc.bin_data_content, id) else {
                return false;
            };
            let Some(bytes) = doc.bin_data_content[i]
                .data
                .load_limited(bin_data::MAX_BIN_DATA_BYTES)
            else {
                return false;
            };
            let Some(stroke) = PdfStroke::read(&bytes) else {
                return false;
            };
            let dim = [
                u32::from_be_bytes(bytes[16..20].try_into().unwrap()) as f64 * 75.,
                u32::from_be_bytes(bytes[20..24].try_into().unwrap()) as f64 * 75.,
            ];
            entry.insert((stroke, dim));
        }
        let (stroke, _) = &strokes[&id];
        if stroke.color != color || stroke.alpha != 1. {
            return false;
        }
    }
    true
}

fn prepare(doc: &Document, si: usize, pi: usize, ti: usize) -> Result<Option<Prepared>, HwpError> {
    let Some(parent) = doc.sections.get(si).and_then(|s| s.paragraphs.get(pi)) else {
        return Ok(None); // Preserve the ordinary API's index/type diagnostics.
    };
    let Some(Control::Table(table)) = parent.controls.get(ti) else {
        return Ok(None);
    };
    let has_cell = table
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(PaintRef::from_control)
        .any(|p| p.common().cell_border_paint.is_some());
    let has_outside = parent
        .controls
        .iter()
        .filter_map(PaintRef::from_control)
        .filter_map(outside_binding)
        .any(|v| v.0 == table.common.instance_id);
    if !has_cell && !has_outside {
        return Ok(None);
    }
    if table.row_count == 0
        || table.col_count == 0
        || table.cell_spacing != 0
        || !table.zones.is_empty()
        || table.cells.len() > 4096
        || table.cells.iter().any(|c| {
            c.row_span != 1
                || c.col_span != 1
                || c.row >= table.row_count
                || c.col >= table.col_count
        })
        || table.cells.len() != usize::from(table.row_count) * usize::from(table.col_count)
    {
        return Err(error());
    }
    let x = boundaries(table.get_column_widths());
    let y = boundaries(table.get_row_heights());
    let mut seen = std::collections::BTreeSet::new();
    for c in &table.cells {
        if !seen.insert((c.row, c.col))
            || f64::from(c.width) != x[c.col as usize + 1] - x[c.col as usize]
            || f64::from(c.height) != y[c.row as usize + 1] - y[c.row as usize]
        {
            return Err(error());
        }
    }
    if x.last().copied() != Some(f64::from(table.common.width))
        || y.last().copied() != Some(f64::from(table.common.height))
    {
        return Err(error());
    }
    if has_outside
        && (table.common.treat_as_char
            || table.common.horz_rel_to != HorzRelTo::Paper
            || table.common.vert_rel_to != VertRelTo::Paper
            || table.common.horz_align != HorzAlign::Left
            || table.common.vert_align != VertAlign::Top)
    {
        return Err(error());
    }
    let mut proof = Proof {
        table,
        x,
        y,
        resources: Resources::new(&doc.bin_data_content),
        contents: &doc.bin_data_content,
        strokes: BTreeMap::new(),
        edges: BTreeMap::new(),
        work: 0,
        z: [[None; 2]; 2],
        layer_range: None,
    };
    let mut remove = Vec::new();
    let last_table = parent
        .controls
        .iter()
        .rposition(|c| matches!(c, Control::Table(_)))
        .unwrap_or(ti);
    for (i, ctrl) in parent.controls.iter().enumerate() {
        let Some(p) = PaintRef::from_control(ctrl) else {
            continue;
        };
        if !outside_binding(p).is_some_and(|v| v.0 == table.common.instance_id) {
            continue;
        }
        let (ci, tag) = outside_owner(p, table, &mut proof.resources).ok_or_else(error)?;
        if i <= last_table {
            return Err(error());
        }
        proof.add(ctrl, ci, tag, true)?;
        remove.push(i);
    }
    let mut candidate = super::border_paint_edit::without(parent, &remove, &[]);
    let Control::Table(out) = &mut candidate.controls[ti] else {
        unreachable!()
    };
    for (ci, cell) in table.cells.iter().enumerate() {
        for (pi, p) in cell.paragraphs.iter().enumerate() {
            let mut remove = Vec::new();
            for (i, ctrl) in p.controls.iter().enumerate() {
                let Some(paint) = PaintRef::from_control(ctrl) else {
                    continue;
                };
                if paint.common().cell_border_paint.is_none() {
                    continue;
                }
                let tag = valid_paint(table.common.instance_id, cell, paint, &mut proof.resources)
                    .ok_or_else(error)?;
                proof.add(ctrl, ci, tag, false)?;
                remove.push(i);
            }
            out.cells[ci].paragraphs[pi] = super::border_paint_edit::without(p, &remove, &[]);
        }
    }
    proof.complete()?;
    let custom = proof.edges.values().any(|e| e.width.is_none());
    if custom {
        // Cell paint uses Para/Column anchors. table_cell_content positions a
        // later paragraph or a vertically aligned text block at a different
        // origin. This proof uses the top cell frame, so it cannot authorize
        // rewriting those drawings into cell-relative bands.
        for c in &table.cells {
            if c.vertical_align != crate::model::table::VerticalAlign::Top {
                return Err(error());
            }
            for (pi, p) in c.paragraphs.iter().enumerate() {
                for paint in p.controls.iter().filter_map(PaintRef::from_control) {
                    let a = paint.common();
                    if a.cell_border_paint.is_some()
                        && (pi != 0
                            || a.horz_align != HorzAlign::Left
                            || a.vert_align != VertAlign::Top)
                    {
                        return Err(error());
                    }
                }
            }
        }
        if parent
            .controls
            .iter()
            .filter_map(PaintRef::from_control)
            .any(|p| {
                outside_binding(p).is_some_and(|v| v.0 == table.common.instance_id)
                    && (p.common().horz_align != HorzAlign::Left
                        || p.common().vert_align != VertAlign::Top)
            })
        {
            return Err(error());
        }
        let first = proof.edges.values().next().ok_or_else(error)?;
        let expected = usize::from(table.row_count) * (usize::from(table.col_count) + 1)
            + usize::from(table.col_count) * (usize::from(table.row_count) + 1);
        // Each complete parallel line has its own source pen. It must remain
        // continuous through every interval along that line. Mutating the
        // nonuniform axis still requires a new-boundary conflict contract;
        // apply() rejects that case before committing this candidate.
        if proof.edges.len() != expected {
            return Err(error());
        }
        let mut axes = [
            vec![None::<f64>; usize::from(table.col_count) + 1],
            vec![None::<f64>; usize::from(table.row_count) + 1],
        ];
        for (&(horizontal, line, _), edge) in &proof.edges {
            let pen = axes[usize::from(horizontal)][line].get_or_insert(edge.pen);
            if edge.color != first.color || (edge.pen - *pen).abs() > 1.00001 {
                return Err(error());
            }
        }
        if !preserves_layer_order(
            doc,
            si,
            pi,
            &candidate,
            proof.layer_range.ok_or_else(error)?,
            &mut proof.strokes,
            first.color,
        ) {
            return Err(error());
        }
        let mut pens = [Vec::new(), Vec::new()];
        for (axis, lines) in axes.into_iter().enumerate() {
            let axis_pen = lines.first().copied().flatten().ok_or_else(error)?;
            for observed in lines {
                let observed = observed.ok_or_else(error)?;
                // Preserve the existing canonical pen for source values within
                // the proved one-unit crop uncertainty. Canonicalize before
                // integer rounding, which can otherwise turn one uniform axis
                // into different integer pens (e.g. 49/50) on different lines.
                let source_pen = if (observed - first.pen).abs() <= 1.00001 {
                    first.pen
                } else if (observed - axis_pen).abs() <= 1.00001 {
                    axis_pen
                } else {
                    observed
                };
                let pen = source_pen.round();
                if pen < 2. || pen > f64::from(i32::MAX) || (pen - source_pen).abs() > 1. {
                    return Err(error());
                }
                pens[axis].push(pen as u32);
            }
        }
        for c in &table.cells {
            let f = doc
                .doc_info
                .border_fills
                .get(
                    usize::from(c.border_fill_id)
                        .checked_sub(1)
                        .ok_or_else(error)?,
                )
                .ok_or_else(error)?;
            if f.borders[..4]
                .iter()
                .any(|b| b.line_type != BorderLineType::None)
                || c.width <= pens[0][usize::from(c.col)].max(pens[0][usize::from(c.col) + 1])
                || c.height <= pens[1][usize::from(c.row)].max(pens[1][usize::from(c.row) + 1])
            {
                return Err(error());
            }
        }
        return Ok(Some(Prepared {
            parent: candidate,
            fills: Vec::new(),
            bands: Some(super::border_paint_bands::Bands {
                pens,
                color: first.color,
                cell_z: [
                    proof.z[0][0].ok_or_else(error)?,
                    proof.z[0][1].ok_or_else(error)?,
                ],
                outside_z: [
                    proof.z[1][0].ok_or_else(error)?,
                    proof.z[1][1].ok_or_else(error)?,
                ],
            }),
        }));
    }
    let mut fills = Vec::new();
    for (i, c) in table.cells.iter().enumerate() {
        let mut fill = doc
            .doc_info
            .border_fills
            .get(
                usize::from(c.border_fill_id)
                    .checked_sub(1)
                    .ok_or_else(error)?,
            )
            .ok_or_else(error)?
            .clone();
        for side in 0..4 {
            if let Some(e) = proof.edges.get(&key(c, side)) {
                if fill.borders[side].line_type != BorderLineType::None {
                    return Err(error());
                }
                fill.borders[side].line_type = BorderLineType::Solid;
                fill.borders[side].color = e.color;
                fill.borders[side].width = e.width.ok_or_else(error)?;
            }
        }
        fill.raw_data = None;
        let id = if let Some(i) = doc.doc_info.border_fills.iter().position(|f| f == &fill) {
            i + 1
        } else if let Some(i) = fills.iter().position(|f| f == &fill) {
            doc.doc_info.border_fills.len() + i + 1
        } else {
            fills.push(fill);
            doc.doc_info.border_fills.len() + fills.len()
        };
        out.cells[i].border_fill_id = u16::try_from(id).map_err(|_| error())?;
    }
    Ok(Some(Prepared {
        parent: candidate,
        fills,
        bands: None,
    }))
}

impl DocumentCore {
    pub(super) fn mutate_table_structure(
        &mut self,
        si: usize,
        pi: usize,
        ti: usize,
        edit: impl FnOnce(&mut Table) -> Result<(), String>,
    ) -> Result<(u16, u16), HwpError> {
        if let Some(mut prepared) = prepare(&self.document, si, pi, ti)? {
            let Control::Table(table) = &mut prepared.parent.controls[ti] else {
                unreachable!()
            };
            edit(table).map_err(HwpError::RenderError)?;
            let counts = (table.row_count, table.col_count);
            if let Some(bands) = prepared.bands {
                super::border_paint_bands::apply(&self.document, &mut prepared.parent, ti, bands)?;
            }
            self.document.sections[si].paragraphs[pi] = prepared.parent;
            self.document.doc_info.border_fills.extend(prepared.fills);
            self.document.doc_info.raw_stream_dirty = true;
            self.rebuild_resolved_styles();
            Ok(counts)
        } else {
            let table = self.get_table_mut(si, pi, ti)?;
            edit(table).map_err(HwpError::RenderError)?;
            Ok((table.row_count, table.col_count))
        }
    }
}
