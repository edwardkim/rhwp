//! Exact-width opaque solid bands after a complete source-ink proof.
//! These are owned border paint, not a nearest native BorderFill width.
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
#[derive(Clone)]
pub(super) struct Bands {
    /// Full physical pens of vertical/horizontal grid lines, in grid order.
    pub pens: [Vec<u32>; 2],
    pub color: u32,
    pub cell_z: [i32; 2],
    pub outside_z: [i32; 2],
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
    mut bands: Bands,
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
    // Parallel lines keep their source styles when the other axis changes.
    // A uniform axis has an unambiguous style for each new boundary. Changing
    // a nonuniform axis needs a template/conflict contract; never guess which
    // of two unlike neighbouring source lines should win the new boundary.
    for (axis, count) in [t.col_count, t.row_count].into_iter().enumerate() {
        let pens = &mut bands.pens[axis];
        let pen = *pens.first().ok_or_else(error)?;
        let count = usize::from(count) + 1;
        if pens.len() != count {
            if pens.iter().any(|&p| p != pen) {
                return Err(error());
            }
            pens.resize(count, pen);
        }
    }
    let widths = t.get_column_widths();
    let heights = t.get_row_heights();
    let offsets = |sizes: &[u32]| -> Vec<i64> {
        let mut at = 0i64;
        sizes
            .iter()
            .map(|&size| {
                let n = at;
                at += i64::from(size);
                n
            })
            .collect()
    };
    let xs = offsets(&widths);
    let ys = offsets(&heights);
    let ox = i64::from(t.common.horizontal_offset as i32);
    let oy = i64::from(t.common.vertical_offset as i32);
    let mut outside = Vec::new();
    for (ci, cell) in t.cells.iter_mut().enumerate() {
        let pens = [
            bands.pens[0][usize::from(cell.col)],
            bands.pens[0][usize::from(cell.col) + 1],
            bands.pens[1][usize::from(cell.row)],
            bands.pens[1][usize::from(cell.row) + 1],
        ];
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
            shape.common_mut().cell_border_paint = CellBorderPaint::from_mask(1 << side, source);
            append(&mut cell.paragraphs[0], control);
        }
        let x = ox + xs[cell.col as usize];
        let y = oy + ys[cell.row as usize];
        let outer = [
            (cell.col == 0, [x - negative[0], y, negative[0], h]),
            (cell.col + 1 == t.col_count, [x + w, y, positive[1], h]),
            (cell.row == 0, [x, y - negative[2], w, negative[2]]),
            (cell.row + 1 == t.row_count, [x, y + h, w, positive[3]]),
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
