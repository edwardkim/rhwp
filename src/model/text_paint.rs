//! Explicit editable glyph paint, independent of font weight and line metrics.
//! Width is stored in thousandths of a point; it is not an HWP outline preset.
use super::{document::Document, style::CharShape};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextPaint {
    pub fill: bool,
    pub width_mpt: u32,
    pub stroke_color: u32,
    pub fill_alpha: u8,
    pub stroke_alpha: u8,
    pub line_join: u8,
    pub miter_limit_milli: u32,
}
impl TextPaint {
    pub fn valid(self) -> bool {
        self.width_mpt <= 100_000_000
            && self.stroke_color <= 0x00ff_ffff
            && self.line_join <= 2
            && (1_000..=1_000_000).contains(&self.miter_limit_milli)
    }
    pub fn at_dpi(self, dpi: f64) -> TextPaintStyle {
        TextPaintStyle {
            paint: self,
            width: f64::from(self.width_mpt) * dpi / 72_000.0,
        }
    }
}
/// Resolved stroke width in the same page pixels as the glyph origins.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TextPaintStyle {
    pub paint: TextPaint,
    pub width: f64,
}
crate::storage::clone_copy!(TextPaintStyle);

pub const HWP_STREAM: &str = "/RhwpTextPaint";
pub const HWPX_ENTRY: &str = "META-INF/rhwp-text-paint.json";
pub const MAX_BYTES: usize = 1024 * 1024;
const MAX_RECORDS: usize = 8192;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: usize,
    fingerprint: [u8; 32],
    paint: TextPaint,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u8,
    records: Vec<Record>,
}

// Match the persisted style, including actual face names rather than potentially
// remapped font IDs. Ignore raw record bytes and the extension being restored.
fn fingerprint(doc: &Document, cs: &CharShape) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(cs.base_size.to_le_bytes());
    h.update(cs.text_color.to_le_bytes());
    h.update([u8::from(cs.bold), u8::from(cs.italic)]);
    h.update(cs.ratios);
    h.update(cs.relative_sizes);
    for value in cs.spacings {
        h.update(value.to_le_bytes());
    }
    for value in cs.char_offsets {
        h.update(value.to_le_bytes());
    }
    for (lang, id) in cs.font_ids.iter().enumerate() {
        let name = doc.doc_info.font_faces[lang]
            .get(usize::from(*id))
            .map_or("", |font| font.name.as_str());
        h.update((name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
    }
    h.finalize().into()
}

pub(crate) fn encode_with_storage<S: crate::storage::Storage>(
    doc: &Document,
    storage: &S,
) -> Result<Option<Vec<u8>>, String>
where
    S::Error: std::fmt::Display,
{
    let mut records = Vec::new();
    for (id, cs) in doc.doc_info.char_shapes.iter().enumerate() {
        if let Some(paint) = cs.text_paint {
            if !paint.valid() || records.len() >= MAX_RECORDS {
                return Err("invalid editable text paint metadata".into());
            }
            storage
                .push(
                    &mut records,
                    Record {
                        id,
                        fingerprint: fingerprint(doc, cs),
                        paint,
                    },
                )
                .map_err(|e| e.to_string())?;
        }
    }
    if records.is_empty() {
        return Ok(None);
    }
    let mut writer = crate::serializer::budget_cursor::BudgetCursor::new(storage, MAX_BYTES);
    serde_json::to_writer(
        &mut writer,
        &Envelope {
            version: 1,
            records,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(Some(writer.into_inner()))
}

pub fn decode(doc: &mut Document, bytes: &[u8]) {
    if bytes.len() > MAX_BYTES {
        return;
    }
    let Ok(data) = serde_json::from_slice::<Envelope>(bytes) else {
        return;
    };
    if data.version != 1
        || data.records.len() > MAX_RECORDS
        || data
            .records
            .iter()
            .any(|r| !r.paint.valid() || r.id >= doc.doc_info.char_shapes.len())
        || data.records.windows(2).any(|p| p[0].id >= p[1].id)
    {
        return;
    }
    for r in data.records {
        if fingerprint(doc, &doc.doc_info.char_shapes[r.id]) == r.fingerprint {
            doc.doc_info.char_shapes[r.id].text_paint = Some(r.paint);
        }
    }
}

pub(crate) fn native_visibility_loss_with_storage<S: crate::storage::Storage>(
    storage: &S,
) -> Result<crate::serializer::content_loss::ContentLoss, S::Error> {
    use crate::serializer::content_loss::*;
    Ok(ContentLoss {
        code: ContentLossCode::MetadataReduced,
        subject: ContentLossSubject::Control,
        path: storage.string("정밀 문자 획: rhwp 왕복 보존, 한컴 기본 표현 미지원")?,
        reason: ContentLossReason::UnsupportedByOutputFormat,
        resource_id: None,
    })
}
