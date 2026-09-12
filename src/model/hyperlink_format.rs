//! 링크 색/밑줄의 원래 값을 범위별로 보존한다. 글꼴/굵기 등은 복원 대상이 아니다.
//! HWP의 미정의 ParameterSet ID를 만들지 않는다. rhwp 전용 보조 엔트리는
//! 링크 ID·현재 텍스트/Command 해시가 맞을 때만 읽고, 외부 재저장으로 사라지면
//! 복원 정보 없음으로 처리한다. 한컴 자체의 원래 서식 저장 규약은 아니다.
use super::{
    control::{Control, FieldType},
    document::Document,
    paragraph::Paragraph,
    style::{CharShape, CharShapeMods, UnderlineType},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const HWP_STREAM: &str = "/RhwpHyperlinkFormat";
pub const HWPX_ENTRY: &str = "META-INF/rhwp-hyperlink-format.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Format {
    color: u32,
    underline: u8,
    underline_color: u32,
}
impl Format {
    fn from_shape(s: &CharShape) -> Self {
        Self {
            color: s.text_color,
            underline: match s.underline_type {
                UnderlineType::None => 0,
                UnderlineType::Bottom => 1,
                UnderlineType::Top => 2,
            },
            underline_color: s.underline_color,
        }
    }
    fn mods(self) -> CharShapeMods {
        CharShapeMods {
            text_color: Some(self.color),
            underline_color: Some(self.underline_color),
            underline_type: Some(match self.underline {
                1 => UnderlineType::Bottom,
                2 => UnderlineType::Top,
                _ => UnderlineType::None,
            }),
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Run {
    len: usize,
    format: Format,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OriginalFormat {
    runs: Vec<Run>,
}
impl OriginalFormat {
    pub fn capture(p: &Paragraph, shapes: &[CharShape], start: usize, end: usize) -> Option<Self> {
        let mut out = Self::default();
        for i in start..end {
            let raw = *p.char_offsets.get(i)?;
            let id = p
                .char_shapes
                .iter()
                .rev()
                .find(|s| s.start_pos <= raw)?
                .char_shape_id;
            out.push(1, Format::from_shape(shapes.get(id as usize)?));
        }
        Some(out)
    }
    fn push(&mut self, len: usize, format: Format) {
        if len == 0 {
            return;
        }
        if let Some(last) = self.runs.last_mut() {
            if last.format == format {
                last.len += len;
                return;
            }
        }
        self.runs.push(Run { len, format });
    }
    pub fn valid_for(&self, len: usize) -> bool {
        self.runs
            .iter()
            .all(|r| r.len > 0 && r.format.underline <= 2)
            && self
                .runs
                .iter()
                .try_fold(0usize, |n, r| n.checked_add(r.len))
                == Some(len)
    }
    pub fn edits(&self, start: usize) -> Vec<(usize, usize, CharShapeMods)> {
        let mut offset = start;
        self.runs
            .iter()
            .map(|r| {
                let a = offset;
                offset += r.len;
                (a, offset, r.format.mods())
            })
            .collect()
    }
    pub fn replace(&mut self, start: usize, end: usize, count: usize) {
        let mut out = Self::default();
        let mut offset = 0;
        let inherited = self.runs.iter().find_map(|r| {
            offset += r.len;
            (start.saturating_sub(1) < offset).then_some(r.format)
        });
        offset = 0;
        for r in &self.runs {
            let next = offset + r.len;
            out.push(next.min(start).saturating_sub(offset), r.format);
            offset = next;
        }
        if let Some(format) = inherited {
            out.push(count, format);
        }
        offset = 0;
        for r in &self.runs {
            let next = offset + r.len;
            out.push(next.saturating_sub(offset.max(end)), r.format);
            offset = next;
        }
        *self = out;
    }
    pub fn repeat_first(&mut self, count: usize) {
        if let Some(r) = self.runs.first().cloned() {
            self.runs = vec![Run { len: count, ..r }];
        }
    }
}

/// 텍스트 축을 바꾸는 공통 경로에서 복원 범위도 함께 갱신한다.
pub fn text_edit(p: &mut Paragraph, start: usize, end: usize, count: usize) {
    for range in &p.field_ranges {
        let Some(Control::Field(field)) = p.controls.get_mut(range.control_idx) else {
            continue;
        };
        let Some(format) = &mut field.hyperlink_format else {
            continue;
        };
        let len = range.end_char_idx.saturating_sub(range.start_char_idx);
        if !format.valid_for(len) {
            field.hyperlink_format = None;
            continue;
        }
        if start == end {
            if range.start_char_idx < start && start < range.end_char_idx {
                format.replace(
                    start - range.start_char_idx,
                    start - range.start_char_idx,
                    count,
                );
            }
        } else if start < range.end_char_idx && range.start_char_idx < end {
            format.replace(
                start.saturating_sub(range.start_char_idx),
                end.min(range.end_char_idx) - range.start_char_idx,
                count,
            );
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Record {
    id: u32,
    fingerprint: Vec<u8>,
    format: Box<OriginalFormat>,
}
#[derive(Serialize, Deserialize)]
struct Envelope {
    version: u8,
    records: Vec<Record>,
}
fn fingerprint(p: &Paragraph, start: usize, end: usize, command: &str) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update((command.len() as u64).to_le_bytes());
    h.update(command.as_bytes());
    for c in p.text.chars().skip(start).take(end - start) {
        h.update((c as u32).to_le_bytes());
    }
    h.finalize().to_vec()
}
fn collect(paras: &[Paragraph], out: &mut Vec<Record>) {
    for p in paras {
        for range in &p.field_ranges {
            let Some(Control::Field(f)) = p.controls.get(range.control_idx) else {
                continue;
            };
            let Some(format) = &f.hyperlink_format else {
                continue;
            };
            if f.field_type == FieldType::Hyperlink
                && range.start_char_idx <= range.end_char_idx
                && format.valid_for(range.end_char_idx - range.start_char_idx)
            {
                out.push(Record {
                    id: f.field_id,
                    fingerprint: fingerprint(
                        p,
                        range.start_char_idx,
                        range.end_char_idx,
                        &f.command,
                    ),
                    format: format.clone(),
                });
            }
        }
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &t.cells {
                        collect(&cell.paragraphs, out);
                    }
                }
                Control::Shape(s) => {
                    if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        collect(&tb.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn encode(doc: &Document) -> Option<Vec<u8>> {
    let mut records = Vec::new();
    for s in &doc.sections {
        collect(&s.paragraphs, &mut records);
    }
    if records.is_empty() {
        None
    } else {
        serde_json::to_vec(&Envelope {
            version: 1,
            records,
        })
        .ok()
    }
}
fn attach(paras: &mut [Paragraph], records: &HashMap<u32, Record>) {
    for p in paras {
        for range in &p.field_ranges {
            let Some(Control::Field(f)) = p.controls.get(range.control_idx) else {
                continue;
            };
            let Some(r) = records.get(&f.field_id) else {
                continue;
            };
            if f.field_type != FieldType::Hyperlink
                || range.start_char_idx > range.end_char_idx
                || !r
                    .format
                    .valid_for(range.end_char_idx - range.start_char_idx)
                || r.fingerprint
                    != fingerprint(p, range.start_char_idx, range.end_char_idx, &f.command)
            {
                continue;
            }
            if let Control::Field(f) = &mut p.controls[range.control_idx] {
                f.hyperlink_format = Some(r.format.clone());
            }
        }
        for c in &mut p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &mut t.cells {
                        attach(&mut cell.paragraphs, records);
                    }
                }
                Control::Shape(s) => {
                    if let Some(tb) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
                        attach(&mut tb.paragraphs, records);
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn decode(doc: &mut Document, bytes: &[u8]) {
    // 잘못된 부가 정보가 본문 읽기를 막거나 임의 서식을 적용하지 않게 한다.
    let Ok(data) = serde_json::from_slice::<Envelope>(bytes) else {
        return;
    };
    if data.version != 1 {
        return;
    }
    let mut records = HashMap::new();
    let mut duplicate = std::collections::HashSet::new();
    for r in data.records {
        if records.contains_key(&r.id) {
            duplicate.insert(r.id);
        }
        records.insert(r.id, r);
    }
    records.retain(|id, _| !duplicate.contains(id));
    for s in &mut doc.sections {
        attach(&mut s.paragraphs, &records);
    }
}
