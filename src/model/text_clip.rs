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
                    let v = match c {
                        ClipCommand::Move(x, y) | ClipCommand::Line(x, y) => vec![*x, *y],
                        ClipCommand::Cubic(a, b, c, d, e, f) => vec![*a, *b, *c, *d, *e, *f],
                        ClipCommand::Close => vec![],
                    };
                    v.iter().all(|v| v.is_finite() && v.abs() <= 1_000_000_000.)
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
                _ => {}
            }
        }
    }
}
fn fingerprint(tb: &TextBox) -> Vec<u8> {
    let mut h = Sha256::new();
    for p in &tb.paragraphs {
        h.update((p.text.len() as u64).to_le_bytes());
        h.update(p.text.as_bytes());
    }
    h.finalize().to_vec()
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
fn collect(
    paras: &[Paragraph],
    records: &mut Vec<Record>,
    counts: &mut BTreeMap<u32, usize>,
    depth: usize,
) {
    if depth > 64 {
        return;
    }
    fn shape(
        s: &ShapeObject,
        records: &mut Vec<Record>,
        counts: &mut BTreeMap<u32, usize>,
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
            *counts.entry(s.common().instance_id).or_default() += 1;
            if let Some(paths) = &tb.paint_clip {
                records.push(Record {
                    id: s.common().instance_id,
                    fingerprint: fingerprint(tb),
                    paths: paths.clone(),
                });
            }
            collect(&tb.paragraphs, records, counts, depth + 1);
        }
        if let ShapeObject::Group(g) = s {
            for s in &g.children {
                shape(s, records, counts, depth + 1);
            }
        }
    }
    for p in paras {
        for c in &p.controls {
            match c {
                Control::Shape(s) => shape(s, records, counts, depth + 1),
                Control::Table(t) => {
                    for c in &t.cells {
                        collect(&c.paragraphs, records, counts, depth + 1)
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn encode(doc: &Document) -> Result<Option<Vec<u8>>, String> {
    let mut records = Vec::new();
    let mut counts = BTreeMap::new();
    for s in &doc.sections {
        collect(&s.paragraphs, &mut records, &mut counts, 0);
    }
    let mut ids = BTreeSet::new();
    if records.iter().any(|r| {
        r.id == 0 || counts.get(&r.id) != Some(&1) || !ids.insert(r.id) || !valid(&r.paths)
    }) || records.len() > 4096
    {
        return Err("invalid editable text clip metadata".into());
    }
    if records.is_empty() {
        return Ok(None);
    }
    let bytes = serde_json::to_vec(&Envelope {
        version: 1,
        records,
    })
    .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("text clip metadata budget exceeded".into());
    }
    Ok(Some(bytes))
}
pub fn native_visibility_loss() -> crate::serializer::content_loss::ContentLoss {
    use crate::serializer::content_loss::*;
    ContentLoss {
        code: ContentLossCode::MetadataReduced,
        subject: ContentLossSubject::Control,
        path: "글상자 텍스트 클립: rhwp 왕복 보존, 한컴 기본 표현 미지원".into(),
        reason: ContentLossReason::UnsupportedByOutputFormat,
        resource_id: None,
    }
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
    }
    for s in &mut doc.sections {
        visit(
            &mut s.paragraphs,
            &mut |id, tb| {
                if counts.get(&id) != Some(&1) {
                    return;
                }
                if let Some(r) = records.get(&id) {
                    if r.fingerprint == fingerprint(tb) {
                        tb.paint_clip = Some(r.paths.clone());
                    }
                }
            },
            0,
        );
    }
}
