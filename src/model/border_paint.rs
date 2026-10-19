//! Provenance of a retained cell-border drawing, independent of its visible paint.
//! Native files keep the drawing. A bounded rhwp sidecar keeps its editing owner.
use super::{
    bin_data::BinDataContent,
    control::Control,
    document::Document,
    image::Picture,
    paragraph::Paragraph,
    shape::{CommonObjAttr, HorzRelTo, RectangleShape, ShapeObject, TextWrap, VertRelTo},
    style::FillType,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const HWP_STREAM: &str = "/RhwpCellBorderPaint";
pub const HWPX_ENTRY: &str = "META-INF/rhwp-cell-border-paint.bin";
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"RBPO\0\0\0\x01";
const MAGIC_OUTSIDE: &[u8; 8] = b"RBPO\0\0\0\x02";
const MAGIC_RASTER: &[u8; 8] = b"RBPO\0\0\0\x03";
const MAGIC_COMPOUND: &[u8; 8] = b"RBPO\0\0\0\x04";
const HEADER_BYTES: usize = 12;
const RECORD_BYTES: usize = 65;
const MAX_RECORDS: usize = (MAX_BYTES - HEADER_BYTES) / RECORD_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellBorderPaint {
    pub side: CellSide,
    /// Opaque origin identity shared by physical pieces of the same source paint.
    /// It is not the current drawing's ID and is never used to locate a control.
    pub source_id: u32,
    /// Zero is the legacy single side. Nonzero is a certified union of sides.
    #[serde(default, skip_serializing_if = "zero")]
    pub sides: u8,
}
fn zero(v: &u8) -> bool {
    *v == 0
}
impl CellSide {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Top => 2,
            Self::Bottom => 3,
        }
    }
    pub(crate) fn from_index(i: usize) -> Self {
        [Self::Left, Self::Right, Self::Top, Self::Bottom][i]
    }
}
impl CellBorderPaint {
    pub(crate) fn from_mask(mask: u8, source_id: u32) -> Option<Self> {
        if mask == 0 || mask & !15 != 0 {
            return None;
        }
        Some(Self {
            side: CellSide::from_index(mask.trailing_zeros() as usize),
            source_id,
            sides: if mask.count_ones() > 1 { mask } else { 0 },
        })
    }
    pub(crate) fn mask(self) -> Option<u8> {
        if self.sides == 0 {
            Some(1 << self.side.index())
        } else if self.sides & !15 == 0
            && self.sides.count_ones() > 1
            && self.sides.trailing_zeros() as usize == self.side.index()
        {
            Some(self.sides)
        } else {
            None
        }
    }
}
/// Called only after validating the current binding. Remaining sides keep the
/// same physical crop/resource; the journal captures the old role for undo.
pub(crate) fn keep_sides(c: &mut Control, mask: u8, outside: bool) -> Option<()> {
    let a = match c {
        Control::Picture(p) => &mut p.common,
        Control::Shape(s) => s.common_mut(),
        _ => return None,
    };
    if outside {
        let tag = a.outside_cell_border_paint.as_mut()?;
        tag.paint = CellBorderPaint::from_mask(mask, tag.paint.source_id)?;
    } else {
        let tag = a.cell_border_paint?;
        a.cell_border_paint = Some(CellBorderPaint::from_mask(mask, tag.source_id)?);
    }
    Some(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    table: u32,
    row: u16,
    col: u16,
    row_span: u16,
    col_span: u16,
    width: u32,
    height: u32,
    left: i16,
    top: i16,
}

/// A page drawing keeps its physical owner; this is only its editing link.
/// The producer certifies closure of the original stroke, never a bbox guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutsideCellBorderPaint {
    owner: Owner,
    paint: CellBorderPaint,
    table_fingerprint: [u8; 32],
    fingerprint: [u8; 32],
}

fn table_fingerprint(t: &super::table::Table) -> [u8; 32] {
    let a = &t.common;
    let mut h = Sha256::new();
    for v in [
        a.instance_id,
        a.horizontal_offset,
        a.vertical_offset,
        a.width,
        a.height,
    ] {
        h.update(v.to_le_bytes());
    }
    h.update(
        serde_json::to_vec(&(
            a.horz_rel_to,
            a.vert_rel_to,
            a.horz_align,
            a.vert_align,
            a.treat_as_char,
            a.flow_with_text,
            a.text_wrap,
        ))
        .unwrap(),
    );
    for c in &t.cells {
        h.update(serde_json::to_vec(&Owner::new(a.instance_id, c)).unwrap());
    }
    h.finalize().into()
}

/// One source paint contract consumed by serialization and native editing.
#[derive(Clone, Copy)]
pub(crate) enum PaintRef<'a> {
    Rectangle(&'a RectangleShape),
    Picture(&'a Picture),
}
impl<'a> PaintRef<'a> {
    pub(crate) fn from_control(c: &'a Control) -> Option<Self> {
        match c {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Rectangle(r) => Some(Self::Rectangle(r)),
                _ => None,
            },
            Control::Picture(p) => Some(Self::Picture(p)),
            _ => None,
        }
    }
    pub(crate) fn common(self) -> &'a CommonObjAttr {
        match self {
            Self::Rectangle(r) => &r.common,
            Self::Picture(p) => &p.common,
        }
    }
    fn raster(self) -> bool {
        matches!(self, Self::Picture(_))
    }
    fn digest(self, resources: &mut Resources<'_>) -> Option<[u8; 32]> {
        match self {
            Self::Rectangle(r) => geometry_digest(r),
            Self::Picture(p) => picture_digest(p, resources),
        }
    }
    fn sealed(self, hash: [u8; 32]) -> bool {
        match self {
            Self::Rectangle(_) => true,
            Self::Picture(p) => p.border_paint_fingerprint == Some(hash),
        }
    }
}
enum PaintMut<'a> {
    Rectangle(&'a mut RectangleShape),
    Picture(&'a mut Picture),
}
impl PaintMut<'_> {
    fn as_ref(&self) -> PaintRef<'_> {
        match self {
            Self::Rectangle(r) => PaintRef::Rectangle(r),
            Self::Picture(p) => PaintRef::Picture(p),
        }
    }
    fn common(&mut self) -> &mut CommonObjAttr {
        match self {
            Self::Rectangle(r) => &mut r.common,
            Self::Picture(p) => &mut p.common,
        }
    }
    fn seal(&mut self, hash: [u8; 32]) {
        if let Self::Picture(p) = self {
            p.border_paint_fingerprint = Some(hash);
        }
    }
}
/// Hash each referenced PNG once per operation, using the renderer's index mapping.
/// Only producer-generated PNG stroke bitmaps can acquire this role.
pub(crate) struct Resources<'a> {
    contents: &'a [BinDataContent],
    hashes: BTreeMap<u16, Option<([u8; 32], (u32, u32))>>,
}
impl<'a> Resources<'a> {
    pub(crate) fn new(contents: &'a [BinDataContent]) -> Self {
        Self {
            contents,
            hashes: BTreeMap::new(),
        }
    }
    fn image(&mut self, id: u16) -> Option<([u8; 32], (u32, u32))> {
        *self.hashes.entry(id).or_insert_with(|| {
            let i = super::bin_data::content_index(self.contents, id)?;
            let bytes = self.contents[i]
                .data
                .load_limited(super::bin_data::MAX_BIN_DATA_BYTES)?;
            if bytes.get(..16)? != b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR" {
                return None;
            }
            let w = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?).checked_mul(75)?;
            let h = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?).checked_mul(75)?;
            if w == 0 || h == 0 {
                return None;
            }
            Some((Sha256::digest(&bytes).into(), (w, h)))
        })
    }
}
fn picture_digest(p: &Picture, resources: &mut Resources<'_>) -> Option<[u8; 32]> {
    let a = &p.common;
    let s = &p.shape_attr;
    let (image, dim) = resources.image(p.image_attr.bin_data_id)?;
    // HWP has no imgDim field; its canonical PNG coordinate system is 75HU/px.
    if (p.img_dim != (0, 0) && p.img_dim != dim)
        || a.instance_id == 0
        || a.width == 0
        || a.height == 0
        || s.rotation_angle != 0
        || s.horz_flip
        || s.vert_flip
        || s.offset_x != 0
        || s.offset_y != 0
        || s.render_sx != 1.0
        || s.render_sy != 1.0
        || s.render_tx != 0.0
        || s.render_ty != 0.0
        || s.render_b != 0.0
        || s.render_c != 0.0
        || s.current_width != a.width
        || s.current_height != a.height
        || p.border_width != 0
        || p.reverse
        || p.caption.is_some()
        || p.effects.shadow.is_some()
        || p.href.is_some()
        || p.image_attr.brightness != 0
        || p.image_attr.contrast != 0
        || p.image_attr.transparency != 0
        || p.image_attr.external_path.is_some()
        || !matches!(p.image_attr.effect, super::image::ImageEffect::RealPic)
        || p.border_x != [0, 0, a.width as i32, 0]
        || p.border_y != [a.width as i32, a.height as i32, 0, a.height as i32]
        || p.crop.left < 0
        || p.crop.top < 0
        || p.crop.right <= p.crop.left
        || p.crop.bottom <= p.crop.top
        || p.crop.right as u32 > dim.0
        || p.crop.bottom as u32 > dim.1
        || p.padding.left != 0
        || p.padding.right != 0
        || p.padding.top != 0
        || p.padding.bottom != 0
    {
        return None;
    }
    let mut h = Sha256::new();
    for v in [a.horizontal_offset, a.vertical_offset, a.width, a.height] {
        h.update(v.to_le_bytes());
    }
    h.update(a.z_order.to_le_bytes());
    for v in [p.crop.left, p.crop.top, p.crop.right, p.crop.bottom] {
        h.update(v.to_le_bytes());
    }
    h.update(image);
    Some(h.finalize().into())
}
fn outside_fingerprint(p: PaintRef<'_>, resources: &mut Resources<'_>) -> Option<[u8; 32]> {
    let a = p.common();
    if a.treat_as_char
        || a.flow_with_text
        || a.horz_rel_to != HorzRelTo::Paper
        || a.vert_rel_to != VertRelTo::Paper
        || a.text_wrap != TextWrap::InFrontOfText
    {
        return None;
    }
    p.digest(resources)
}
pub(crate) fn seal_picture(c: &mut Control, resources: &mut Resources<'_>) -> Option<()> {
    if let Control::Picture(p) = c {
        p.border_paint_fingerprint = Some(picture_digest(p, resources)?);
    }
    Some(())
}
pub(crate) fn bind_outside(
    c: &mut Control,
    t: &super::table::Table,
    cell: usize,
    paint: CellBorderPaint,
    resources: &mut Resources<'_>,
) -> Option<()> {
    paint.mask()?;
    let owner = Owner::new(t.common.instance_id, t.cells.get(cell)?);
    let fingerprint = outside_fingerprint(PaintRef::from_control(c)?, resources)?;
    if owner.table == 0 || paint.source_id == 0 {
        return None;
    }
    let tag = OutsideCellBorderPaint {
        owner,
        paint,
        table_fingerprint: table_fingerprint(t),
        fingerprint,
    };
    match c {
        Control::Shape(s) => s.common_mut().outside_cell_border_paint = Some(tag),
        Control::Picture(p) => {
            p.border_paint_fingerprint = Some(fingerprint);
            p.common.outside_cell_border_paint = Some(tag);
        }
        _ => return None,
    }
    Some(())
}
pub(crate) fn outside_owner(
    p: PaintRef<'_>,
    t: &super::table::Table,
    resources: &mut Resources<'_>,
) -> Option<(usize, CellBorderPaint)> {
    let tag = p.common().outside_cell_border_paint?;
    tag.paint.mask()?;
    if tag.owner.table != t.common.instance_id
        || tag.table_fingerprint != table_fingerprint(t)
        || outside_fingerprint(p, resources) != Some(tag.fingerprint)
        || !p.sealed(tag.fingerprint)
    {
        return None;
    }
    Some((
        t.cells
            .iter()
            .position(|c| Owner::new(t.common.instance_id, c) == tag.owner)?,
        tag.paint,
    ))
}
pub(crate) fn outside_binding(p: PaintRef<'_>) -> Option<(u32, u16, u16, CellBorderPaint)> {
    p.common()
        .outside_cell_border_paint
        .map(|v| (v.owner.table, v.owner.row, v.owner.col, v.paint))
}
impl Owner {
    fn new(id: u32, c: &super::table::Cell) -> Self {
        Self {
            table: id,
            row: c.row,
            col: c.col,
            row_span: c.row_span,
            col_span: c.col_span,
            width: c.width,
            height: c.height,
            left: c.padding.left,
            top: c.padding.top,
        }
    }
}

/// Recheck the producer's binding against the current physical cell.
/// A stale tag never authorizes deletion of a drawing.
pub(crate) fn valid_paint(
    table: u32,
    cell: &super::table::Cell,
    p: PaintRef<'_>,
    resources: &mut Resources<'_>,
) -> Option<CellBorderPaint> {
    let paint = p.common().cell_border_paint?;
    let hash = fingerprint(p, Owner::new(table, cell), paint, resources)?;
    p.sealed(hash).then_some(paint)
}

fn geometry_digest(r: &RectangleShape) -> Option<[u8; 32]> {
    let a = &r.common;
    let s = &r.drawing.shape_attr;
    let color = r.drawing.fill.solid.as_ref()?.background_color;
    if a.instance_id == 0
        || a.width == 0
        || a.height == 0
        || r.drawing.text_box.is_some()
        || r.drawing.fill.fill_type != FillType::Solid
        || r.drawing.fill.alpha != 0
        || r.drawing.border_line.width != 0
        || r.round_rate != 0
        || r.drawing.shadow_type != 0
        || s.rotation_angle != 0
        || s.horz_flip
        || s.vert_flip
        || s.offset_x != 0
        || s.offset_y != 0
        || s.render_sx != 1.0
        || s.render_sy != 1.0
        || s.render_tx != 0.0
        || s.render_ty != 0.0
        || s.render_b != 0.0
        || s.render_c != 0.0
        || r.x_coords != [0, a.width as i32, a.width as i32, 0]
        || r.y_coords != [0, 0, a.height as i32, a.height as i32]
    {
        return None;
    }
    let mut h = Sha256::new();
    for v in [
        a.horizontal_offset,
        a.vertical_offset,
        a.width,
        a.height,
        color,
    ] {
        h.update(v.to_le_bytes());
    }
    h.update(a.z_order.to_le_bytes());
    Some(h.finalize().into())
}

fn fingerprint(
    p: PaintRef<'_>,
    o: Owner,
    paint: CellBorderPaint,
    resources: &mut Resources<'_>,
) -> Option<[u8; 32]> {
    let a = p.common();
    if o.table == 0
        || paint.source_id == 0
        || o.row_span == 0
        || o.col_span == 0
        || a.treat_as_char
        || !a.flow_with_text
        || a.horz_rel_to != HorzRelTo::Column
        || a.vert_rel_to != VertRelTo::Para
        || a.text_wrap != TextWrap::BehindText
    {
        return None;
    }
    let x = i64::from(a.horizontal_offset as i32) + i64::from(o.left);
    let y = i64::from(a.vertical_offset as i32) + i64::from(o.top);
    let right = x + i64::from(a.width);
    let bottom = y + i64::from(a.height);
    let mask = paint.mask()?;
    let edges = [
        x,
        right - i64::from(o.width),
        y,
        bottom - i64::from(o.height),
    ];
    // One source HWPUNIT of quantization; never relocate the drawing to fit.
    if edges
        .iter()
        .enumerate()
        .any(|(i, edge)| mask & (1 << i) != 0 && edge.abs() > 1)
        || x < -1
        || y < -1
        || right > i64::from(o.width) + 1
        || bottom > i64::from(o.height) + 1
    {
        return None;
    }
    p.digest(resources)
}

// SectionDef controls mirror the canonical master list; do not visit both copies.
fn walk(
    ps: &[Paragraph],
    owner: Option<Owner>,
    f: &mut impl FnMut(&CommonObjAttr, Option<Owner>, Option<PaintRef<'_>>),
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    fn shape(
        s: &ShapeObject,
        owner: Option<Owner>,
        f: &mut impl FnMut(&CommonObjAttr, Option<Owner>, Option<PaintRef<'_>>),
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        f(
            s.common(),
            owner,
            if let ShapeObject::Rectangle(r) = s {
                Some(PaintRef::Rectangle(r))
            } else {
                None
            },
        );
        if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
            walk(&tb.paragraphs, None, f, depth + 1);
        }
        if let ShapeObject::Group(g) = s {
            for c in &g.children {
                shape(c, None, f, depth + 1);
            }
        }
    }
    for p in ps {
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    f(&t.common, None, None);
                    for cell in &t.cells {
                        walk(
                            &cell.paragraphs,
                            Some(Owner::new(t.common.instance_id, cell)),
                            f,
                            depth + 1,
                        );
                    }
                }
                Control::Shape(s) => shape(s, owner, f, depth + 1),
                Control::Picture(p) => f(&p.common, owner, Some(PaintRef::Picture(p))),
                Control::Header(h) => walk(&h.paragraphs, None, f, depth + 1),
                Control::Footer(h) => walk(&h.paragraphs, None, f, depth + 1),
                Control::Footnote(n) => walk(&n.paragraphs, None, f, depth + 1),
                Control::Endnote(n) => walk(&n.paragraphs, None, f, depth + 1),
                _ => {}
            }
        }
    }
}
fn walk_mut(
    ps: &mut [Paragraph],
    owner: Option<Owner>,
    f: &mut impl FnMut(PaintMut<'_>, Option<Owner>),
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    fn shape(
        s: &mut ShapeObject,
        owner: Option<Owner>,
        f: &mut impl FnMut(PaintMut<'_>, Option<Owner>),
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        if let ShapeObject::Rectangle(r) = s {
            f(PaintMut::Rectangle(r), owner);
        }
        if let Some(tb) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
            walk_mut(&mut tb.paragraphs, None, f, depth + 1);
        }
        if let ShapeObject::Group(g) = s {
            for c in &mut g.children {
                shape(c, None, f, depth + 1);
            }
        }
    }
    for p in ps {
        for c in &mut p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &mut t.cells {
                        let o = Owner::new(t.common.instance_id, cell);
                        walk_mut(&mut cell.paragraphs, Some(o), f, depth + 1);
                    }
                }
                Control::Shape(s) => shape(s, owner, f, depth + 1),
                Control::Picture(p) => f(PaintMut::Picture(p), owner),
                Control::Header(h) => walk_mut(&mut h.paragraphs, None, f, depth + 1),
                Control::Footer(h) => walk_mut(&mut h.paragraphs, None, f, depth + 1),
                Control::Footnote(n) => walk_mut(&mut n.paragraphs, None, f, depth + 1),
                Control::Endnote(n) => walk_mut(&mut n.paragraphs, None, f, depth + 1),
                _ => {}
            }
        }
    }
}
fn each(doc: &Document, f: &mut impl FnMut(&CommonObjAttr, Option<Owner>, Option<PaintRef<'_>>)) {
    for s in &doc.sections {
        walk(&s.paragraphs, None, f, 0);
        for m in &s.section_def.master_pages {
            walk(&m.paragraphs, None, f, 0);
        }
    }
}

struct Record {
    id: u32,
    owner: Owner,
    paint: CellBorderPaint,
    fingerprint: [u8; 32],
    outside: Option<[u8; 32]>,
    raster: bool,
}
impl Record {
    fn write(&self, out: &mut Vec<u8>, compound: bool) {
        let o = self.owner;
        out.extend(self.id.to_le_bytes());
        out.extend(o.table.to_le_bytes());
        for v in [o.row, o.col, o.row_span, o.col_span] {
            out.extend(v.to_le_bytes());
        }
        for v in [o.width, o.height] {
            out.extend(v.to_le_bytes());
        }
        for v in [o.left, o.top] {
            out.extend(v.to_le_bytes());
        }
        out.push(
            (if compound {
                self.paint.mask().expect("validated paint mask")
            } else {
                match self.paint.side {
                    CellSide::Left => 0,
                    CellSide::Right => 1,
                    CellSide::Top => 2,
                    CellSide::Bottom => 3,
                }
            }) | if self.outside.is_some() { 0x80 } else { 0 }
                | if self.raster { 0x40 } else { 0 },
        );
        out.extend(self.paint.source_id.to_le_bytes());
        out.extend(self.fingerprint);
        if let Some(frame) = self.outside {
            out.extend(frame);
        }
    }
    fn read(bytes: &[u8], compound: bool) -> Option<Self> {
        fn take<const N: usize>(b: &[u8], i: &mut usize) -> Option<[u8; N]> {
            let v = b.get(*i..*i + N)?.try_into().ok()?;
            *i += N;
            Some(v)
        }
        let mut i = 0;
        let id = u32::from_le_bytes(take(bytes, &mut i)?);
        let owner = Owner {
            table: u32::from_le_bytes(take(bytes, &mut i)?),
            row: u16::from_le_bytes(take(bytes, &mut i)?),
            col: u16::from_le_bytes(take(bytes, &mut i)?),
            row_span: u16::from_le_bytes(take(bytes, &mut i)?),
            col_span: u16::from_le_bytes(take(bytes, &mut i)?),
            width: u32::from_le_bytes(take(bytes, &mut i)?),
            height: u32::from_le_bytes(take(bytes, &mut i)?),
            left: i16::from_le_bytes(take(bytes, &mut i)?),
            top: i16::from_le_bytes(take(bytes, &mut i)?),
        };
        let flag = take::<1>(bytes, &mut i)?[0];
        let side = if compound {
            CellBorderPaint::from_mask(flag & !0xc0, 1)?.side
        } else {
            match flag & !0xc0 {
                0 => CellSide::Left,
                1 => CellSide::Right,
                2 => CellSide::Top,
                3 => CellSide::Bottom,
                _ => return None,
            }
        };
        let source_id = u32::from_le_bytes(take(bytes, &mut i)?);
        Some(Self {
            id,
            owner,
            paint: if compound {
                CellBorderPaint::from_mask(flag & !0xc0, source_id)?
            } else {
                CellBorderPaint {
                    side,
                    source_id,
                    sides: 0,
                }
            },
            fingerprint: take(bytes, &mut i)?,
            raster: flag & 0x40 != 0,
            outside: if flag & 0x80 != 0 {
                Some(take(bytes, &mut i)?)
            } else {
                None
            },
        })
    }
}

pub fn encode(doc: &Document) -> Result<Option<Vec<u8>>, String> {
    let mut counts = BTreeMap::<u32, usize>::new();
    let mut records = Vec::new();
    let mut over_budget = false;
    let mut resources = Resources::new(&doc.bin_data_content);
    each(doc, &mut |a, owner, rect| {
        *counts.entry(a.instance_id).or_default() += 1;
        if let (Some(paint), Some(owner), Some(rect)) = (a.cell_border_paint, owner, rect) {
            // A drawing moved out of its old edge no longer has that binding.
            // Its paint is kept; stale provenance never authorizes an edit.
            if let Some(fingerprint) =
                fingerprint(rect, owner, paint, &mut resources).filter(|h| rect.sealed(*h))
            {
                if records.len() == MAX_RECORDS {
                    over_budget = true;
                    return;
                }
                records.push(Record {
                    id: a.instance_id,
                    owner,
                    paint,
                    fingerprint,
                    outside: None,
                    raster: rect.raster(),
                });
            }
        }
        if let (Some(tag), None, Some(rect)) = (a.outside_cell_border_paint, owner, rect) {
            let table = doc
                .sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .flat_map(|p| &p.controls)
                .find_map(|c| match c {
                    Control::Table(t) if t.common.instance_id == tag.owner.table => {
                        Some(t.as_ref())
                    }
                    _ => None,
                });
            if table.is_some_and(|t| outside_owner(rect, t, &mut resources).is_some()) {
                if records.len() == MAX_RECORDS {
                    over_budget = true;
                    return;
                }
                records.push(Record {
                    id: a.instance_id,
                    owner: tag.owner,
                    paint: tag.paint,
                    fingerprint: tag.fingerprint,
                    outside: Some(tag.table_fingerprint),
                    raster: rect.raster(),
                });
            }
        }
    });
    if records.is_empty() {
        return Ok(None);
    }
    let size = HEADER_BYTES
        + records
            .iter()
            .map(|r| RECORD_BYTES + if r.outside.is_some() { 32 } else { 0 })
            .sum::<usize>();
    if over_budget || size > MAX_BYTES {
        return Err("cell border paint metadata budget exceeded".into());
    }
    if records
        .iter()
        .any(|r| counts.get(&r.id) != Some(&1) || counts.get(&r.owner.table) != Some(&1))
    {
        return Err("ambiguous cell border paint ownership".into());
    }
    let mut bytes = Vec::with_capacity(size);
    let compound = records.iter().any(|r| r.paint.sides != 0);
    bytes.extend(if compound {
        MAGIC_COMPOUND
    } else if records.iter().any(|r| r.raster) {
        MAGIC_RASTER
    } else if records.iter().any(|r| r.outside.is_some()) {
        MAGIC_OUTSIDE
    } else {
        MAGIC
    });
    bytes.extend((records.len() as u32).to_le_bytes());
    for r in records {
        r.write(&mut bytes, compound);
    }
    Ok(Some(bytes))
}
pub fn decode(doc: &mut Document, bytes: &[u8]) {
    if bytes.len() > MAX_BYTES {
        return;
    }
    if bytes.len() < HEADER_BYTES
        || (&bytes[..8] != MAGIC
            && &bytes[..8] != MAGIC_OUTSIDE
            && &bytes[..8] != MAGIC_RASTER
            && &bytes[..8] != MAGIC_COMPOUND)
    {
        return;
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let compound = &bytes[..8] == MAGIC_COMPOUND;
    let outside_version = &bytes[..8] == MAGIC_OUTSIDE || &bytes[..8] == MAGIC_RASTER || compound;
    let raster_version = &bytes[..8] == MAGIC_RASTER || compound;
    if count > MAX_RECORDS || bytes.len() < HEADER_BYTES + count * RECORD_BYTES {
        return;
    }
    let mut ids = BTreeSet::new();
    let mut records = BTreeMap::new();
    let mut offset = HEADER_BYTES;
    for _ in 0..count {
        let Some(r) = Record::read(&bytes[offset..], compound) else {
            return;
        };
        if (r.outside.is_some() && !outside_version)
            || (r.raster && !raster_version)
            || r.owner.table == 0
            || r.paint.source_id == 0
        {
            return;
        }
        offset += RECORD_BYTES + if r.outside.is_some() { 32 } else { 0 };
        if r.id == 0 || !ids.insert(r.id) {
            return;
        }
        records.insert(r.id, r);
    }
    if offset != bytes.len() {
        return;
    }
    let mut counts = BTreeMap::<u32, usize>::new();
    each(doc, &mut |a, _, _| {
        *counts.entry(a.instance_id).or_default() += 1
    });
    let tables = doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            Control::Table(t) => Some((
                t.common.instance_id,
                (
                    table_fingerprint(t),
                    t.cells
                        .iter()
                        .map(|c| Owner::new(t.common.instance_id, c))
                        .collect::<Vec<_>>(),
                ),
            )),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let mut resources = Resources::new(&doc.bin_data_content);
    let mut restore = |mut r: PaintMut<'_>, owner: Option<Owner>| {
        let Some(record) = records.get(&r.as_ref().common().instance_id) else {
            return;
        };
        if counts.get(&record.id) != Some(&1)
            || counts.get(&record.owner.table) != Some(&1)
            || r.as_ref().raster() != record.raster
        {
            return;
        }
        if let Some(frame) = record.outside {
            if owner.is_none()
                && tables
                    .get(&record.owner.table)
                    .is_some_and(|(hash, cells)| *hash == frame && cells.contains(&record.owner))
                && outside_fingerprint(r.as_ref(), &mut resources) == Some(record.fingerprint)
            {
                r.seal(record.fingerprint);
                r.common().outside_cell_border_paint = Some(OutsideCellBorderPaint {
                    owner: record.owner,
                    paint: record.paint,
                    table_fingerprint: frame,
                    fingerprint: record.fingerprint,
                });
            }
        } else if counts.get(&record.id) == Some(&1)
            && counts.get(&record.owner.table) == Some(&1)
            && owner == Some(record.owner)
            && fingerprint(r.as_ref(), record.owner, record.paint, &mut resources)
                == Some(record.fingerprint)
        {
            r.seal(record.fingerprint);
            r.common().cell_border_paint = Some(record.paint);
        }
    };
    for s in &mut doc.sections {
        walk_mut(&mut s.paragraphs, None, &mut restore, 0);
        for m in &mut s.section_def.master_pages {
            walk_mut(&mut m.paragraphs, None, &mut restore, 0);
        }
    }
}
pub fn native_editing_loss() -> crate::serializer::content_loss::ContentLoss {
    use crate::serializer::content_loss::*;
    ContentLoss {
        code: ContentLossCode::MetadataReduced,
        subject: ContentLossSubject::Control,
        path: "셀 테두리 그림 소유: rhwp 왕복 보존, 한컴 편집 소유 미지원".into(),
        reason: ContentLossReason::UnsupportedByOutputFormat,
        resource_id: None,
    }
}
