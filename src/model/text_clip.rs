//! Editable text paint clipping. Geometry is independent of line metrics/content.
//! TextBox stores HWPUNIT-local paths; RenderNode/LayerNode project them to page px.
//! The private sidecar preserves this IR contract, not a native Hancom attribute.
use super::{
    control::Control,
    document::Document,
    paragraph::Paragraph,
    shape::{ShapeObject, TextBox},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
pub const HWP_STREAM: &str = "/RhwpTextClips";
pub const HWPX_ENTRY: &str = "META-INF/rhwp-text-clips.json";
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClipCommand {
    Move(f64, f64),
    Line(f64, f64),
    Cubic(f64, f64, f64, f64, f64, f64),
    Close,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextClipPath {
    pub commands: Vec<ClipCommand>,
    pub even_odd: bool,
}
impl TextClipPath {
    pub fn map(&self, mut f: impl FnMut(f64, f64) -> (f64, f64)) -> Self {
        Self {
            even_odd: self.even_odd,
            commands: self
                .commands
                .iter()
                .map(|v| match *v {
                    ClipCommand::Move(x, y) => {
                        let (x, y) = f(x, y);
                        ClipCommand::Move(x, y)
                    }
                    ClipCommand::Line(x, y) => {
                        let (x, y) = f(x, y);
                        ClipCommand::Line(x, y)
                    }
                    ClipCommand::Cubic(a, b, c, d, e, g) => {
                        let (a, b) = f(a, b);
                        let (c, d) = f(c, d);
                        let (e, g) = f(e, g);
                        ClipCommand::Cubic(a, b, c, d, e, g)
                    }
                    ClipCommand::Close => ClipCommand::Close,
                })
                .collect(),
        }
    }
}
pub fn valid(paths: &[TextClipPath]) -> bool {
    !paths.is_empty()
        && paths.len() <= 32
        && paths.iter().map(|p| p.commands.len()).sum::<usize>() <= 16384
        && paths.iter().all(|p| {
            !p.commands.is_empty()
                && matches!(p.commands[0], ClipCommand::Move(..))
                && p.commands.iter().all(|c| {
                    let valid = |v: f64| v.is_finite() && v.abs() <= 1_000_000_000.;
                    match *c {
                        ClipCommand::Move(x, y) | ClipCommand::Line(x, y) => valid(x) && valid(y),
                        ClipCommand::Cubic(a, b, c, d, e, f) => {
                            [a, b, c, d, e, f].into_iter().all(valid)
                        }
                        ClipCommand::Close => true,
                    }
                })
        })
}
fn visit_shape(s: &mut ShapeObject, f: &mut impl FnMut(u32, &mut TextBox), depth: usize) {
    if depth > 64 {
        return;
    }
    let id = s.common().instance_id;
    if let Some(tb) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
        f(id, tb);
        visit(&mut tb.paragraphs, f, depth + 1);
    }
    if let ShapeObject::Group(g) = s {
        for c in &mut g.children {
            visit_shape(c, f, depth + 1)
        }
    }
}
fn visit(paras: &mut [Paragraph], f: &mut impl FnMut(u32, &mut TextBox), depth: usize) {
    if depth > 64 {
        return;
    }
    for p in paras {
        for c in &mut p.controls {
            match c {
                Control::Shape(s) => visit_shape(s, f, depth + 1),
                Control::Table(t) => {
                    for c in &mut t.cells {
                        visit(&mut c.paragraphs, f, depth + 1)
                    }
                }
                Control::Header(h) => visit(&mut h.paragraphs, f, depth + 1),
                Control::Footer(h) => visit(&mut h.paragraphs, f, depth + 1),
                Control::Footnote(n) => visit(&mut n.paragraphs, f, depth + 1),
                Control::Endnote(n) => visit(&mut n.paragraphs, f, depth + 1),
                _ => {}
            }
        }
    }
}
fn fingerprint(tb: &TextBox) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in &tb.paragraphs {
        h.update((p.text.len() as u64).to_le_bytes());
        h.update(p.text.as_bytes());
    }
    h.finalize().into()
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: u32,
    fingerprint: Vec<u8>,
    paths: Vec<TextClipPath>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u8,
    records: Vec<Record>,
}
#[derive(Serialize)]
struct RecordRef<'a> {
    id: u32,
    fingerprint: [u8; 32],
    paths: &'a [TextClipPath],
}
#[derive(Serialize)]
struct EnvelopeRef<'a> {
    version: u8,
    records: &'a [RecordRef<'a>],
}
fn collect<'a, S: crate::storage::Storage>(
    paras: &'a [Paragraph],
    records: &mut Vec<RecordRef<'a>>,
    counts: &mut BTreeMap<u32, usize>,
    depth: usize,
    storage: &S,
) -> Result<(), String>
where
    S::Error: std::fmt::Display,
{
    if depth > 64 {
        return Ok(());
    }
    fn shape<'a, S: crate::storage::Storage>(
        s: &'a ShapeObject,
        records: &mut Vec<RecordRef<'a>>,
        counts: &mut BTreeMap<u32, usize>,
        depth: usize,
        storage: &S,
    ) -> Result<(), String>
    where
        S::Error: std::fmt::Display,
    {
        if depth > 64 {
            return Ok(());
        }
        if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
            let id = s.common().instance_id;
            if !counts.contains_key(&id) {
                storage
                    .tree_entry::<u32, usize>()
                    .map_err(|e| e.to_string())?;
            }
            *counts.entry(id).or_default() += 1;
            if let Some(paths) = &tb.paint_clip {
                storage
                    .push(
                        records,
                        RecordRef {
                            id,
                            fingerprint: fingerprint(tb),
                            paths,
                        },
                    )
                    .map_err(|e| e.to_string())?;
            }
            collect(&tb.paragraphs, records, counts, depth + 1, storage)?;
        }
        if let ShapeObject::Group(g) = s {
            for s in &g.children {
                shape(s, records, counts, depth + 1, storage)?;
            }
        }
        Ok(())
    }
    for p in paras {
        for c in &p.controls {
            match c {
                Control::Shape(s) => shape(s, records, counts, depth + 1, storage)?,
                Control::Table(t) => {
                    for c in &t.cells {
                        collect(&c.paragraphs, records, counts, depth + 1, storage)?;
                    }
                }
                Control::Header(h) => collect(&h.paragraphs, records, counts, depth + 1, storage)?,
                Control::Footer(h) => collect(&h.paragraphs, records, counts, depth + 1, storage)?,
                Control::Footnote(n) => {
                    collect(&n.paragraphs, records, counts, depth + 1, storage)?
                }
                Control::Endnote(n) => collect(&n.paragraphs, records, counts, depth + 1, storage)?,
                _ => {}
            }
        }
    }
    Ok(())
}
pub fn encode(doc: &Document) -> Result<Option<Vec<u8>>, String> {
    encode_with_storage(doc, &crate::storage::Unbounded)
}
pub(crate) fn encode_with_storage<S: crate::storage::Storage>(
    doc: &Document,
    storage: &S,
) -> Result<Option<Vec<u8>>, String>
where
    S::Error: std::fmt::Display,
{
    let mut records = Vec::new();
    let mut counts = BTreeMap::new();
    for s in &doc.sections {
        collect(&s.paragraphs, &mut records, &mut counts, 0, storage)?;
        // SectionDef controls mirror this canonical list. Visiting both copies
        // would falsely report every master-page TextBox as a duplicate owner.
        for master in &s.section_def.master_pages {
            collect(&master.paragraphs, &mut records, &mut counts, 0, storage)?;
        }
    }
    let mut ids = BTreeSet::new();
    for r in &records {
        if r.id == 0 || counts.get(&r.id) != Some(&1) || ids.contains(&r.id) || !valid(r.paths) {
            return Err("invalid editable text clip metadata".into());
        }
        storage
            .insert_tree_set(&mut ids, r.id)
            .map_err(|e| e.to_string())?;
    }
    if records.len() > 4096 {
        return Err("invalid editable text clip metadata".into());
    }
    if records.is_empty() {
        return Ok(None);
    }
    let mut writer = crate::serializer::budget_cursor::BudgetCursor::new(storage, MAX_BYTES);
    serde_json::to_writer(
        &mut writer,
        &EnvelopeRef {
            version: 1,
            records: &records,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(Some(writer.into_inner()))
}
pub fn native_visibility_loss() -> crate::serializer::content_loss::ContentLoss {
    match native_visibility_loss_with_storage(&crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn native_visibility_loss_with_storage<S: crate::storage::Storage>(
    storage: &S,
) -> Result<crate::serializer::content_loss::ContentLoss, S::Error> {
    use crate::serializer::content_loss::*;
    Ok(ContentLoss {
        code: ContentLossCode::MetadataReduced,
        subject: ContentLossSubject::Control,
        path: storage.string("글상자 텍스트 클립: rhwp 왕복 보존, 한컴 기본 표현 미지원")?,
        reason: ContentLossReason::UnsupportedByOutputFormat,
        resource_id: None,
    })
}
pub fn decode(doc: &mut Document, bytes: &[u8]) {
    if bytes.len() > MAX_BYTES {
        return;
    }
    let Ok(data) = serde_json::from_slice::<Envelope>(bytes) else {
        return;
    };
    if data.version != 1
        || data.records.len() > 4096
        || data.records.iter().any(|r| !valid(&r.paths))
    {
        return;
    }
    let mut records = BTreeMap::new();
    for r in data.records {
        if records.insert(r.id, r).is_some() {
            return;
        }
    }
    let mut counts = BTreeMap::<u32, usize>::new();
    for s in &mut doc.sections {
        visit(
            &mut s.paragraphs,
            &mut |id, _| *counts.entry(id).or_default() += 1,
            0,
        );
        for master in &mut s.section_def.master_pages {
            visit(
                &mut master.paragraphs,
                &mut |id, _| *counts.entry(id).or_default() += 1,
                0,
            );
        }
    }
    for s in &mut doc.sections {
        let mut restore = |id, tb: &mut TextBox| {
            if counts.get(&id) != Some(&1) {
                return;
            }
            if let Some(r) = records.get(&id) {
                if r.fingerprint == fingerprint(tb) {
                    tb.paint_clip = Some(r.paths.clone());
                }
            }
        };
        visit(&mut s.paragraphs, &mut restore, 0);
        for master in &mut s.section_def.master_pages {
            visit(&mut master.paragraphs, &mut restore, 0);
        }
    }
}
