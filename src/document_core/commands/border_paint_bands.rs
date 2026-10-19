//! Exact-width opaque solid bands after a complete source-ink proof.
//! These are owned border paint, not a nearest native BorderFill width.
use super::border_paint_structure::{grid, side_keys, Key, StructureEdit};
use crate::{
    error::HwpError,
    model::{
        border_paint::{bind_outside, CellBorderPaint, Resources},
        control::Control,
        document::Document,
        identity::{used_instance_ids, Allocator},
        paragraph::Paragraph,
        shape::{
            HorzAlign, HorzRelTo, RectangleShape, ShapeObject, SizeCriterion, TextWrap, VertAlign,
            VertRelTo,
        },
        style::{FillType, SolidFill},
    },
};
use std::collections::BTreeMap;
type CellPens = BTreeMap<(u16, u16), [u32; 4]>;
type EdgePens = BTreeMap<Key, u32>;

pub(super) fn physical_pens(
    cell: &crate::model::table::Cell,
    edges: &EdgePens,
) -> Result<[u32; 4], HwpError> {
    let mut pens = [0; 4];
    for (side, pen) in pens.iter_mut().enumerate() {
        let mut intervals = side_keys(cell, side);
        *pen = *edges
            .get(&intervals.next().ok_or_else(error)?)
            .ok_or_else(error)?;
        if intervals.any(|key| edges.get(&key) != Some(pen)) {
            return Err(error()); // One own style cannot explain a varying physical side.
        }
    }
    Ok(pens)
}
#[derive(Clone)]
pub(super) struct Bands {
    /// Full physical pen of each grid-line interval, shared by adjacent cells.
    pub pens: EdgePens,
    pub cells: CellPens,
    pub color: u32,
    pub cell_z: [i32; 2],
    pub outside_z: [i32; 2],
}
impl Bands {
    pub(super) fn cell_pens(
        table: &crate::model::table::Table,
        physical: &EdgePens,
    ) -> Result<CellPens, HwpError> {
        let mut out = BTreeMap::new();
        for cell in &table.cells {
            let mut own = physical_pens(cell, physical)?;
            let mut seen = [false; 4];
            for paint in cell.paragraphs.iter().flat_map(|p| &p.controls) {
                let Control::Shape(shape) = paint else {
                    continue;
                };
                let Some(tag) = shape.common().cell_border_paint else {
                    continue;
                };
                let Some(pen) = tag.cell_pen else { continue };
                tag.mask().ok_or_else(error)?;
                let side = tag.side.index();
                if seen[side] || pen > own[side] {
                    return Err(error());
                }
                seen[side] = true;
                own[side] = pen;
            }
            out.insert((cell.row, cell.col), own);
        }
        // A stored losing style must still account for the complete physical
        // edge together with its neighbour. Metadata cannot authorize new ink.
        if Self::resolve(table, &out)? != *physical {
            return Err(error());
        }
        Ok(out)
    }
    fn resolve(table: &crate::model::table::Table, cells: &CellPens) -> Result<EdgePens, HwpError> {
        let mut edges = BTreeMap::<Key, u32>::new();
        for cell in &table.cells {
            let own = cells.get(&(cell.row, cell.col)).ok_or_else(error)?;
            for (side, &own_pen) in own.iter().enumerate() {
                // Complete opaque solid pens of one colour: the wider cell
                // border wins, as in native shared-edge resolution. Preserve
                // the losing own width separately for subsequent deletion.
                for key in side_keys(cell, side) {
                    let pen = edges.entry(key).or_default();
                    *pen = (*pen).max(own_pen);
                }
            }
        }
        if edges.values().any(|&p| p < 2 || p > i32::MAX as u32) {
            return Err(error());
        }
        Ok(edges)
    }
    pub(super) fn edit(
        &mut self,
        table: &mut crate::model::table::Table,
        edit: StructureEdit,
    ) -> Result<(), HwpError> {
        let mut cells = CellPens::new();
        for cell in &table.cells {
            if let Some(at) = edit.destination(cell) {
                let pens = *self.cells.get(&(cell.row, cell.col)).ok_or_else(error)?;
                cells.insert(at, pens);
            }
        }
        let mut missing = false;
        edit.apply(table, |new, template| {
            if let Some(&pens) = cells.get(&(template.row, template.col)) {
                cells.insert((new.row, new.col), pens);
            } else {
                missing = true;
            }
        })
        .map_err(HwpError::RenderError)?;
        if missing || cells.len() != table.cells.len() {
            return Err(error());
        }
        grid(table)?; // The actual edited cell frames must still describe the same tiling.
        self.pens = Self::resolve(table, &cells)?;
        self.cells = cells;
        Ok(())
    }
}
fn error() -> HwpError {
    HwpError::RenderError("연속 실선 경계 띠의 구조 편집 범위를 표현할 수 없습니다".into())
}
fn append(p: &mut Paragraph, c: Control) {
    let mut positions = p.logical_control_positions();
    positions.push(p.text.chars().count());
    p.controls.push(c);
    p.control_mask |= 0x800;
    p.rebuild_char_offsets(&positions);
}
fn rectangle(
    id: u32,
    rect: [i64; 4],
    bands: &Bands,
    outside: bool,
    axis: usize,
) -> Result<Control, HwpError> {
    let [x, y, w, h] = rect;
    let mut r = RectangleShape::default();
    let a = &mut r.common;
    a.instance_id = id;
    a.z_order = if outside {
        bands.outside_z[axis]
    } else {
        bands.cell_z[axis]
    };
    a.ctrl_id = u32::from_be_bytes(*b"gso ");
    a.horizontal_offset = i32::try_from(x).map_err(|_| error())? as u32;
    a.vertical_offset = i32::try_from(y).map_err(|_| error())? as u32;
    a.width = u32::try_from(w).map_err(|_| error())?;
    a.height = u32::try_from(h).map_err(|_| error())?;
    if w <= 0 || h <= 0 || w > i64::from(i32::MAX) || h > i64::from(i32::MAX) {
        return Err(error());
    }
    a.flow_with_text = !outside;
    a.allow_overlap = true;
    a.horz_rel_to = if outside {
        HorzRelTo::Paper
    } else {
        HorzRelTo::Column
    };
    a.vert_rel_to = if outside {
        VertRelTo::Paper
    } else {
        VertRelTo::Para
    };
    a.horz_align = HorzAlign::Left;
    a.vert_align = VertAlign::Top;
    a.width_criterion = SizeCriterion::Absolute;
    a.height_criterion = SizeCriterion::Absolute;
    a.text_wrap = if outside {
        TextWrap::InFrontOfText
    } else {
        TextWrap::BehindText
    };
    r.x_coords = [0, w as i32, w as i32, 0];
    r.y_coords = [0, 0, h as i32, h as i32];
    r.drawing.inst_id = id;
    let s = &mut r.drawing.shape_attr;
    // This is synthesized geometry, not a parsed SHAPE_COMPONENT. Leave the
    // source ID unset so the HWP writer emits the context's required two IDs
    // for a top-level shape (a single ID belongs to group children).
    s.original_width = a.width;
    s.current_width = a.width;
    s.original_height = a.height;
    s.current_height = a.height;
    r.drawing.border_line.width = 0;
    r.drawing.fill.fill_type = FillType::Solid;
    r.drawing.fill.solid = Some(SolidFill {
        background_color: bands.color,
        pattern_type: -1,
        ..Default::default()
    });
    Ok(Control::Shape(Box::new(ShapeObject::Rectangle(r))))
}
pub(super) fn apply(
    doc: &Document,
    parent: &mut Paragraph,
    ti: usize,
    bands: Bands,
) -> Result<(), HwpError> {
    let mut ids = Allocator {
        used: used_instance_ids(doc),
        next: 1,
    };
    let source = ids.id()?;
    let Control::Table(t) = &mut parent.controls[ti] else {
        return Err(error());
    };
    if t.cells.len() > 4096 {
        return Err(error());
    }
    let grid = grid(t)?;
    if bands
        .pens
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        != grid.edges
    {
        return Err(error());
    }
    let xs: Vec<_> = grid.x.iter().map(|&x| x as i64).collect();
    let ys: Vec<_> = grid.y.iter().map(|&y| y as i64).collect();
    let ox = i64::from(t.common.horizontal_offset as i32);
    let oy = i64::from(t.common.vertical_offset as i32);
    let mut outside = Vec::new();
    for (ci, cell) in t.cells.iter_mut().enumerate() {
        let pens = physical_pens(cell, &bands.pens)?;
        let negative = pens.map(|pen| i64::from(pen / 2));
        let positive =
            std::array::from_fn::<_, 4, _>(|side| i64::from(pens[side]) - negative[side]);
        let w = i64::from(cell.width);
        let h = i64::from(cell.height);
        if w <= i64::from(pens[0].max(pens[1])) || h <= i64::from(pens[2].max(pens[3])) {
            return Err(error());
        }
        let inside = [
            [0, 0, positive[0], h],
            [w - negative[1], 0, negative[1], h],
            [0, 0, w, positive[2]],
            [0, h - negative[3], w, negative[3]],
        ];
        if cell.paragraphs.is_empty() {
            cell.paragraphs.push(Paragraph::new_empty());
        }
        for (side, [x, y, r, b]) in inside.into_iter().enumerate() {
            let mut control = rectangle(
                ids.common()?,
                [
                    x - i64::from(cell.padding.left),
                    y - i64::from(cell.padding.top),
                    r,
                    b,
                ],
                &bands,
                false,
                side / 2,
            )?;
            let Control::Shape(shape) = &mut control else {
                unreachable!()
            };
            let mut tag = CellBorderPaint::from_mask(1 << side, source).ok_or_else(error)?;
            let own = bands.cells.get(&(cell.row, cell.col)).ok_or_else(error)?[side];
            tag.cell_pen = (own != pens[side]).then_some(own);
            shape.common_mut().cell_border_paint = Some(tag);
            append(&mut cell.paragraphs[0], control);
        }
        let x = ox + xs[cell.col as usize];
        let y = oy + ys[cell.row as usize];
        let outer = [
            (cell.col == 0, [x - negative[0], y, negative[0], h]),
            (
                cell.col + cell.col_span == t.col_count,
                [x + w, y, positive[1], h],
            ),
            (cell.row == 0, [x, y - negative[2], w, negative[2]]),
            (
                cell.row + cell.row_span == t.row_count,
                [x, y + h, w, positive[3]],
            ),
        ];
        for (side, (is_outer, rect)) in outer.into_iter().enumerate() {
            if is_outer {
                outside.push((
                    ci,
                    side,
                    rectangle(ids.common()?, rect, &bands, true, side / 2)?,
                ));
            }
        }
    }
    let mut resources = Resources::new(&doc.bin_data_content);
    let mut bound = Vec::new();
    for (ci, side, mut c) in outside {
        bind_outside(
            &mut c,
            t,
            ci,
            CellBorderPaint::from_mask(1 << side, source).ok_or_else(error)?,
            &mut resources,
        )
        .ok_or_else(error)?;
        bound.push(c);
    }
    for c in bound {
        append(parent, c);
    }
    Ok(())
}
