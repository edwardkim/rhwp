//! HWPX cannot reopen a table containing a row without a cell origin (#7998).
//! Project only redundant row cuts, leaving cell geometry/content and the live IR
//! unchanged. A bounded sidecar restores the latent split grid in rhwp; native
//! Hancom sees the ordinary projected table and may drop this private metadata.
use super::{
    control::Control,
    document::Document,
    paragraph::Paragraph,
    shape::ShapeObject,
    table::{Table, MAX_TABLE_GRID_CELLS},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

pub(crate) const HWPX_ENTRY: &str = "META-INF/rhwp-table-row-grids.json";
pub(crate) const MAX_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: u32,
    fingerprint: [u8; 32],
    /// Projected row boundary -> original logical row boundary.
    boundaries: Vec<u16>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u8,
    records: Vec<Record>,
}

fn fingerprint(t: &Table) -> [u8; 32] {
    fingerprint_with_storage(t, &crate::storage::Unbounded).expect("unbounded fingerprint")
}
fn budget_error(error: impl std::fmt::Display) -> String {
    format!("HWPX table grid: {error}")
}
fn fingerprint_with_storage<S: crate::storage::Storage>(
    t: &Table,
    storage: &S,
) -> Result<[u8; 32], String>
where
    S::Error: std::fmt::Display,
{
    let mut h = Sha256::new();
    for value in [t.common.instance_id, t.common.width, t.common.height] {
        h.update(value.to_le_bytes());
    }
    h.update(t.row_count.to_le_bytes());
    h.update(t.col_count.to_le_bytes());
    h.update((t.cells.len() as u64).to_le_bytes());
    let mut cells: Vec<_> = storage.collect(t.cells.iter()).map_err(budget_error)?;
    storage
        .stable_sort_by(&mut cells, |a, b| (a.row, a.col).cmp(&(b.row, b.col)))
        .map_err(budget_error)?;
    for c in cells {
        for value in [c.row, c.col, c.row_span, c.col_span] {
            h.update(value.to_le_bytes());
        }
        h.update(c.width.to_le_bytes());
        h.update(c.height.to_le_bytes());
    }
    h.update((t.zones.len() as u64).to_le_bytes());
    for z in &t.zones {
        for value in [z.start_row, z.end_row, z.start_col, z.end_col] {
            h.update(value.to_le_bytes());
        }
    }
    Ok(h.finalize().into())
}

fn projection(t: &Table) -> Result<Option<Vec<u16>>, String> {
    projection_with_storage(t, &crate::storage::Unbounded)
}
fn projection_with_storage<S: crate::storage::Storage>(
    t: &Table,
    storage: &S,
) -> Result<Option<Vec<u16>>, String>
where
    S::Error: std::fmt::Display,
{
    let mut origins = storage
        .filled(usize::from(t.row_count), false)
        .map_err(budget_error)?;
    for c in &t.cells {
        if let Some(row) = origins.get_mut(usize::from(c.row)) {
            *row = true;
        }
    }
    if origins.iter().all(|v| *v) {
        return Ok(None);
    }
    let error = || "HWPX 표의 빈 논리 행을 내용·경계 손실 없이 표현할 수 없습니다".to_string();
    let slots = usize::from(t.row_count) * usize::from(t.col_count);
    if slots == 0 || slots > MAX_TABLE_GRID_CELLS {
        return Err(error());
    }
    // Prove complete, nonoverlapping ownership from actual cells, not a cached
    // grid or the absence of visible border ink. A physical gap is not removable.
    let mut covered = storage.filled(slots, false).map_err(budget_error)?;
    let mut boundaries = BTreeSet::new();
    for value in [0, t.row_count] {
        storage.tree_entry::<u16, ()>().map_err(budget_error)?;
        boundaries.insert(value);
    }
    for c in &t.cells {
        let end_row = c.row.checked_add(c.row_span).ok_or_else(error)?;
        let end_col = c.col.checked_add(c.col_span).ok_or_else(error)?;
        if c.row_span == 0 || c.col_span == 0 || end_row > t.row_count || end_col > t.col_count {
            return Err(error());
        }
        for value in [c.row, end_row] {
            if !boundaries.contains(&value) {
                storage.tree_entry::<u16, ()>().map_err(budget_error)?;
                boundaries.insert(value);
            }
        }
        for row in c.row..end_row {
            for col in c.col..end_col {
                let index = usize::from(row) * usize::from(t.col_count) + usize::from(col);
                if std::mem::replace(&mut covered[index], true) {
                    return Err(error());
                }
            }
        }
    }
    if covered.iter().any(|v| !v) {
        return Err(error());
    }
    // Zone boundaries carry meaning even when no cell starts there.
    for z in &t.zones {
        let end = z.end_row.checked_add(1).ok_or_else(error)?;
        if z.start_row >= end || end > t.row_count {
            return Err(error());
        }
        for value in [z.start_row, end] {
            if !boundaries.contains(&value) {
                storage.tree_entry::<u16, ()>().map_err(budget_error)?;
                boundaries.insert(value);
            }
        }
    }
    let boundaries: Vec<_> = storage.collect(boundaries).map_err(budget_error)?;
    if boundaries[..boundaries.len() - 1]
        .iter()
        .any(|r| !origins[usize::from(*r)])
    {
        return Err(error());
    }
    Ok(Some(boundaries))
}

#[derive(Debug, Default)]
pub(crate) struct Writer {
    ids: BTreeMap<u32, usize>,
    records: Vec<Record>,
    encoded_budget: usize,
    failure: Option<String>,
}

impl Writer {
    pub(crate) fn table<'a>(&mut self, t: &'a Table) -> Result<Cow<'a, Table>, String> {
        self.table_with_storage(t, &crate::storage::Unbounded)
    }
    pub(crate) fn table_with_storage<'a, S: crate::storage::Storage>(
        &mut self,
        t: &'a Table,
        storage: &S,
    ) -> Result<Cow<'a, Table>, String>
    where
        S::Error: std::fmt::Display,
    {
        let result = self.project_with_storage(t, storage);
        if let Err(error) = &result {
            if self.failure.is_none() {
                self.failure = Some(storage.string(error).map_err(budget_error)?);
            }
        }
        result
    }

    fn project<'a>(&mut self, t: &'a Table) -> Result<Cow<'a, Table>, String> {
        self.project_with_storage(t, &crate::storage::Unbounded)
    }
    fn project_with_storage<'a, S: crate::storage::Storage>(
        &mut self,
        t: &'a Table,
        storage: &S,
    ) -> Result<Cow<'a, Table>, String>
    where
        S::Error: std::fmt::Display,
    {
        if !self.ids.contains_key(&t.common.instance_id) {
            storage.tree_entry::<u32, usize>().map_err(budget_error)?;
        }
        *self.ids.entry(t.common.instance_id).or_default() += 1;
        let Some(boundaries) = projection_with_storage(t, storage)? else {
            return Ok(Cow::Borrowed(t));
        };
        // Bound the record allocation before serializing JSON (each u16 takes
        // at most six bytes including its separator; the fixed record fits 512).
        self.encoded_budget = self
            .encoded_budget
            .checked_add(boundaries.len() * 6 + 512)
            .filter(|bytes| *bytes <= MAX_BYTES)
            .ok_or("HWPX 표 격자 보존 정보가 크기 한도를 넘습니다")?;
        let index = |row| boundaries.binary_search(&row).expect("proven row boundary") as u16;
        let mut projected = storage.clone_table(t).map_err(budget_error)?;
        projected.row_count = (boundaries.len() - 1) as u16;
        for c in &mut projected.cells {
            let end = index(c.row + c.row_span);
            c.row = index(c.row);
            c.row_span = end - c.row;
        }
        for z in &mut projected.zones {
            z.start_row = index(z.start_row);
            z.end_row = index(z.end_row + 1) - 1;
        }
        projected
            .rebuild_row_sizes_with_storage(storage)
            .map_err(budget_error)?;
        projected
            .rebuild_grid_with_storage(storage)
            .map_err(budget_error)?;
        storage
            .push(
                &mut self.records,
                Record {
                    id: t.common.instance_id,
                    fingerprint: fingerprint_with_storage(&projected, storage)?,
                    boundaries,
                },
            )
            .map_err(budget_error)?;
        Ok(Cow::Owned(projected))
    }

    pub(crate) fn encode(&self) -> Result<Option<Vec<u8>>, String> {
        self.encode_with_storage(&crate::storage::Unbounded)
    }
    pub(crate) fn encode_with_storage<S: crate::storage::Storage>(
        &self,
        storage: &S,
    ) -> Result<Option<Vec<u8>>, String>
    where
        S::Error: std::fmt::Display,
    {
        // The paragraph writer can catch an individual control's error. Do not
        // turn a failed table projection into a successful package missing it.
        if let Some(error) = &self.failure {
            return Err(storage.string(error).map_err(budget_error)?);
        }
        if self.records.is_empty() {
            return Ok(None);
        }
        if self.records.iter().any(|r| self.ids.get(&r.id) != Some(&1)) {
            return Err("HWPX 표 격자 보존 대상의 개체 ID가 중복됩니다".into());
        }
        let mut writer = crate::serializer::budget_cursor::BudgetCursor::new(storage, MAX_BYTES);
        serde_json::to_writer(
            &mut writer,
            &EnvelopeRef {
                version: 1,
                records: &self.records,
            },
        )
        .map_err(budget_error)?;
        let bytes = writer.into_inner();
        if bytes.len() > MAX_BYTES {
            return Err("HWPX 표 격자 보존 정보가 크기 한도를 넘습니다".into());
        }
        Ok(Some(bytes))
    }
}

#[derive(Serialize)]
struct EnvelopeRef<'a> {
    version: u8,
    records: &'a [Record],
}

// Visit the canonical owned trees once. SectionDef controls mirror the master
// list and are deliberately not traversed a second time.
fn visit(ps: &mut [Paragraph], f: &mut impl FnMut(&mut Table), depth: usize) {
    if depth > 64 {
        return;
    }
    fn shape(s: &mut ShapeObject, f: &mut impl FnMut(&mut Table), depth: usize) {
        if depth > 64 {
            return;
        }
        if let Some(d) = s.drawing_mut() {
            if let Some(tb) = &mut d.text_box {
                visit(&mut tb.paragraphs, f, depth + 1);
            }
            if let Some(c) = &mut d.caption {
                visit(&mut c.paragraphs, f, depth + 1);
            }
        }
        if let ShapeObject::Group(g) = s {
            if let Some(c) = &mut g.caption {
                visit(&mut c.paragraphs, f, depth + 1);
            }
            for child in &mut g.children {
                shape(child, f, depth + 1);
            }
        }
        if let ShapeObject::Picture(p) = s {
            if let Some(c) = &mut p.caption {
                visit(&mut c.paragraphs, f, depth + 1);
            }
        }
    }
    for p in ps {
        for c in &mut p.controls {
            match c {
                Control::Table(t) => {
                    f(t);
                    for cell in &mut t.cells {
                        visit(&mut cell.paragraphs, f, depth + 1);
                    }
                    if let Some(c) = &mut t.caption {
                        visit(&mut c.paragraphs, f, depth + 1);
                    }
                }
                Control::Shape(s) => shape(s, f, depth + 1),
                Control::Picture(p) => {
                    if let Some(c) = &mut p.caption {
                        visit(&mut c.paragraphs, f, depth + 1);
                    }
                }
                Control::Header(h) => visit(&mut h.paragraphs, f, depth + 1),
                Control::Footer(h) => visit(&mut h.paragraphs, f, depth + 1),
                Control::Footnote(n) => visit(&mut n.paragraphs, f, depth + 1),
                Control::Endnote(n) => visit(&mut n.paragraphs, f, depth + 1),
                Control::HiddenComment(c) => visit(&mut c.paragraphs, f, depth + 1),
                Control::Field(c) => visit(&mut c.memo_paragraphs, f, depth + 1),
                _ => {}
            }
        }
    }
}

fn each(doc: &mut Document, mut f: impl FnMut(&mut Table)) {
    for s in &mut doc.sections {
        visit(&mut s.paragraphs, &mut f, 0);
        for master in &mut s.section_def.master_pages {
            visit(&mut master.paragraphs, &mut f, 0);
        }
    }
}

pub(crate) fn decode(doc: &mut Document, bytes: &[u8]) {
    if bytes.len() > MAX_BYTES {
        return;
    }
    let Ok(envelope) = serde_json::from_slice::<Envelope>(bytes) else {
        return;
    };
    if envelope.version != 1 {
        return;
    }
    let mut records = BTreeMap::new();
    for r in envelope.records {
        if r.boundaries.len() < 2
            || r.boundaries.len() > usize::from(u16::MAX) + 1
            || r.boundaries[0] != 0
            || r.boundaries.windows(2).any(|w| w[0] >= w[1])
            || records.insert(r.id, r).is_some()
        {
            return;
        }
    }
    let mut counts = BTreeMap::<u32, usize>::new();
    each(doc, |t| {
        *counts.entry(t.common.instance_id).or_default() += 1
    });
    each(doc, |t| {
        let Some(r) = records.get(&t.common.instance_id) else {
            return;
        };
        if counts.get(&r.id) != Some(&1)
            || fingerprint(t) != r.fingerprint
            || r.boundaries.len() != usize::from(t.row_count) + 1
        {
            return;
        }
        let rows = *r.boundaries.last().unwrap();
        if usize::from(rows) * usize::from(t.col_count) > MAX_TABLE_GRID_CELLS {
            return;
        }
        // Validate every index before mutation; stale/foreign metadata is ignored.
        if t.cells.iter().any(|c| {
            c.row_span == 0
                || c.row
                    .checked_add(c.row_span)
                    .is_none_or(|e| e > t.row_count)
        }) || t
            .zones
            .iter()
            .any(|z| z.start_row > z.end_row || z.end_row >= t.row_count)
        {
            return;
        }
        for c in &mut t.cells {
            let end = r.boundaries[usize::from(c.row + c.row_span)];
            c.row = r.boundaries[usize::from(c.row)];
            c.row_span = end - c.row;
        }
        for z in &mut t.zones {
            z.start_row = r.boundaries[usize::from(z.start_row)];
            z.end_row = r.boundaries[usize::from(z.end_row) + 1] - 1;
        }
        t.row_count = rows;
        t.rebuild_row_sizes();
        t.rebuild_grid();
    });
}

pub(crate) fn native_grid_loss_with_storage<S: crate::storage::Storage>(
    storage: &S,
) -> Result<crate::serializer::content_loss::ContentLoss, S::Error> {
    use crate::serializer::content_loss::*;
    Ok(ContentLoss {
        code: ContentLossCode::MetadataReduced,
        subject: ContentLossSubject::Control,
        path: storage.string("표의 잠재 행 격자: rhwp 왕복 보존, 한컴에는 실제 셀 경계만 투영")?,
        reason: ContentLossReason::UnsupportedByOutputFormat,
        resource_id: None,
    })
}
