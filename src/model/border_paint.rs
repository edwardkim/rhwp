//! Provenance of a retained cell-border drawing, independent of its visible paint.
//! Native files keep the drawing. A bounded rhwp sidecar keeps its editing owner.
use super::{
    control::Control,
    document::Document,
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
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
pub(crate) fn valid_side(
    table: u32,
    cell: &super::table::Cell,
    r: &RectangleShape,
) -> Option<CellSide> {
    let paint = r.common.cell_border_paint?;
    fingerprint(r, Owner::new(table, cell), paint).map(|_| paint.side)
}

fn fingerprint(r: &RectangleShape, o: Owner, paint: CellBorderPaint) -> Option<[u8; 32]> {
    let a = &r.common;
    let s = &r.drawing.shape_attr;
    let color = r.drawing.fill.solid.as_ref()?.background_color;
    if o.table == 0
        || a.instance_id == 0
        || paint.source_id == 0
        || a.width == 0
        || a.height == 0
        || o.row_span == 0
        || o.col_span == 0
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
    let edge = match paint.side {
        CellSide::Left => x,
        CellSide::Right => right - i64::from(o.width),
        CellSide::Top => y,
        CellSide::Bottom => bottom - i64::from(o.height),
    };
    // One source HWPUNIT of quantization; never relocate the drawing to fit.
    if edge.abs() > 1
        || x < -1
        || y < -1
        || right > i64::from(o.width) + 1
        || bottom > i64::from(o.height) + 1
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

// SectionDef controls mirror the canonical master list; do not visit both copies.
fn walk(
    ps: &[Paragraph],
    owner: Option<Owner>,
    f: &mut impl FnMut(&CommonObjAttr, Option<Owner>, Option<&RectangleShape>),
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    fn shape(
        s: &ShapeObject,
        owner: Option<Owner>,
        f: &mut impl FnMut(&CommonObjAttr, Option<Owner>, Option<&RectangleShape>),
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        f(
            s.common(),
            owner,
            if let ShapeObject::Rectangle(r) = s {
                Some(r)
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
                Control::Picture(p) => f(&p.common, owner, None),
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
    f: &mut impl FnMut(&mut RectangleShape, Option<Owner>),
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    fn shape(
        s: &mut ShapeObject,
        owner: Option<Owner>,
        f: &mut impl FnMut(&mut RectangleShape, Option<Owner>),
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        if let ShapeObject::Rectangle(r) = s {
            f(r, owner);
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
                Control::Header(h) => walk_mut(&mut h.paragraphs, None, f, depth + 1),
                Control::Footer(h) => walk_mut(&mut h.paragraphs, None, f, depth + 1),
                Control::Footnote(n) => walk_mut(&mut n.paragraphs, None, f, depth + 1),
                Control::Endnote(n) => walk_mut(&mut n.paragraphs, None, f, depth + 1),
                _ => {}
            }
        }
    }
}
fn each(
    doc: &Document,
    f: &mut impl FnMut(&CommonObjAttr, Option<Owner>, Option<&RectangleShape>),
) {
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
}
impl Record {
    fn write(&self, out: &mut Vec<u8>) {
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
        out.push(match self.paint.side {
            CellSide::Left => 0,
            CellSide::Right => 1,
            CellSide::Top => 2,
            CellSide::Bottom => 3,
        });
        out.extend(self.paint.source_id.to_le_bytes());
        out.extend(self.fingerprint);
    }
    fn read(bytes: &[u8]) -> Option<Self> {
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
        let side = match take::<1>(bytes, &mut i)?[0] {
            0 => CellSide::Left,
            1 => CellSide::Right,
            2 => CellSide::Top,
            3 => CellSide::Bottom,
            _ => return None,
        };
        let source_id = u32::from_le_bytes(take(bytes, &mut i)?);
        Some(Self {
            id,
            owner,
            paint: CellBorderPaint { side, source_id },
            fingerprint: take(bytes, &mut i)?,
        })
    }
}

pub fn encode(doc: &Document) -> Result<Option<Vec<u8>>, String> {
    let mut counts = BTreeMap::<u32, usize>::new();
    let mut records = Vec::new();
    let mut over_budget = false;
    each(doc, &mut |a, owner, rect| {
        *counts.entry(a.instance_id).or_default() += 1;
        if let (Some(paint), Some(owner), Some(rect)) = (a.cell_border_paint, owner, rect) {
            // A drawing moved out of its old edge no longer has that binding.
            // Its paint is kept; stale provenance never authorizes an edit.
            if let Some(fingerprint) = fingerprint(rect, owner, paint) {
                if records.len() == MAX_RECORDS {
                    over_budget = true;
                    return;
                }
                records.push(Record {
                    id: a.instance_id,
                    owner,
                    paint,
                    fingerprint,
                });
            }
        }
    });
    if records.is_empty() {
        return Ok(None);
    }
    if over_budget {
        return Err("cell border paint metadata budget exceeded".into());
    }
    if records
        .iter()
        .any(|r| counts.get(&r.id) != Some(&1) || counts.get(&r.owner.table) != Some(&1))
    {
        return Err("ambiguous cell border paint ownership".into());
    }
    let mut bytes = Vec::with_capacity(HEADER_BYTES + records.len() * RECORD_BYTES);
    bytes.extend(MAGIC);
    bytes.extend((records.len() as u32).to_le_bytes());
    for r in records {
        r.write(&mut bytes);
    }
    Ok(Some(bytes))
}
pub fn decode(doc: &mut Document, bytes: &[u8]) {
    if bytes.len() > MAX_BYTES {
        return;
    }
    if bytes.len() < HEADER_BYTES || &bytes[..8] != MAGIC {
        return;
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if count > MAX_RECORDS || bytes.len() != HEADER_BYTES + count * RECORD_BYTES {
        return;
    }
    let mut ids = BTreeSet::new();
    let mut records = BTreeMap::new();
    for b in bytes[HEADER_BYTES..].chunks_exact(RECORD_BYTES) {
        let Some(r) = Record::read(b) else {
            return;
        };
        if r.id == 0 || !ids.insert(r.id) {
            return;
        }
        records.insert(r.id, r);
    }
    let mut counts = BTreeMap::<u32, usize>::new();
    each(doc, &mut |a, _, _| {
        *counts.entry(a.instance_id).or_default() += 1
    });
    let mut restore = |r: &mut RectangleShape, owner: Option<Owner>| {
        let Some(record) = records.get(&r.common.instance_id) else {
            return;
        };
        if counts.get(&record.id) == Some(&1)
            && counts.get(&record.owner.table) == Some(&1)
            && owner == Some(record.owner)
            && fingerprint(r, record.owner, record.paint) == Some(record.fingerprint)
        {
            r.common.cell_border_paint = Some(record.paint);
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
