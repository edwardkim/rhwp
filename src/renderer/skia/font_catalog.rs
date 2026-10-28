//! Bounded custom/bundled font catalogs, separate from system font priority.

use std::collections::HashMap;
use std::path::PathBuf;

use skia_safe::{Data, FontMgr, FontStyle, Typeface};

use super::font_lookup::supplied_family_names;

const MAX_FONT_FILES: usize = 256;
const MAX_FONT_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_FONT_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_COLLECTION_FACES: u32 = 64;

/// CoreText-backed Skia may reject a valid nonzero TTC index. Repack only that
/// face's SFNT container; table bytes (including outlines, cmap and metrics) stay
/// unchanged except head.checkSumAdjustment, which belongs to the container.
/// The copy cannot exceed the original bounded input, so catalog retained-byte
/// accounting remains an upper bound even for overlapping malicious tables.
fn standalone_collection_face(bytes: &[u8], face_index: u32) -> Option<Vec<u8>> {
    ttf_parser::fonts_in_collection(bytes)?;
    let face = ttf_parser::Face::parse(bytes, face_index).ok()?;
    let records = face.raw_face().table_records;
    let count = records.len();
    let directory_len = 12usize.checked_add(usize::from(count).checked_mul(16)?)?;
    if directory_len > bytes.len() {
        return None;
    }
    let offset_index = 12usize.checked_add(usize::try_from(face_index).ok()?.checked_mul(4)?)?;
    let source_offset = usize::try_from(u32::from_be_bytes(
        bytes
            .get(offset_index..offset_index.checked_add(4)?)?
            .try_into()
            .ok()?,
    ))
    .ok()?;
    let mut output = bytes
        .get(source_offset..source_offset.checked_add(12)?)?
        .to_vec();
    output.resize(directory_len, 0);
    let mut head_offset = None;
    for (index, record) in records.into_iter().enumerate() {
        let start = usize::try_from(record.offset).ok()?;
        let length = usize::try_from(record.length).ok()?;
        let source = bytes.get(start..start.checked_add(length)?)?;
        let offset = output.len();
        let padded_end = offset.checked_add(length)?.checked_add(3)? & !3;
        if padded_end > bytes.len() {
            return None;
        }
        output.extend_from_slice(source);
        output.resize(padded_end, 0);
        if record.tag == ttf_parser::Tag::from_bytes(b"head") {
            if length < 12 {
                return None;
            }
            output[offset + 8..offset + 12].fill(0);
            head_offset = Some(offset);
        }
        let checksum = output[offset..padded_end]
            .chunks_exact(4)
            .fold(0u32, |sum, chunk| {
                sum.wrapping_add(u32::from_be_bytes(chunk.try_into().unwrap()))
            });
        let entry = 12 + index * 16;
        output[entry..entry + 4].copy_from_slice(&record.tag.to_bytes());
        output[entry + 4..entry + 8].copy_from_slice(&checksum.to_be_bytes());
        output[entry + 8..entry + 12].copy_from_slice(&u32::try_from(offset).ok()?.to_be_bytes());
        output[entry + 12..entry + 16].copy_from_slice(&record.length.to_be_bytes());
    }
    let sum = output.chunks_exact(4).fold(0u32, |sum, chunk| {
        sum.wrapping_add(u32::from_be_bytes(chunk.try_into().unwrap()))
    });
    let offset = head_offset?.checked_add(8)?;
    output[offset..offset + 4].copy_from_slice(&0xB1B0_AFBAu32.wrapping_sub(sum).to_be_bytes());
    Some(output)
}

pub(super) fn typeface_from_data(
    font_mgr: &FontMgr,
    bytes: &[u8],
    face_index: u32,
) -> Option<Typeface> {
    font_mgr
        .new_from_data(Data::new_copy(bytes), Some(face_index))
        .or_else(|| {
            let standalone = standalone_collection_face(bytes, face_index)?;
            font_mgr.new_from_data(Data::new_copy(&standalone), Some(0))
        })
}

#[derive(Clone, Default)]
pub(super) struct TypefaceCatalog {
    families: HashMap<String, Vec<Typeface>>,
    // name ID 6은 특정 face의 식별자이며 family 별칭으로 등록하지 않는다.
    postscript_faces: HashMap<String, Typeface>,
}

impl TypefaceCatalog {
    /// Load only the supplied byte snapshots. Unlike the path-based fallback
    /// loader, every face must parse and construct successfully or the whole
    /// replacement catalog is discarded. No environment paths are consulted.
    pub fn try_from_data(
        font_mgr: &FontMgr,
        sources: &[Vec<u8>],
    ) -> Result<Self, crate::error::HwpError> {
        use crate::error::HwpError;
        let invalid = |message: &str| HwpError::RenderError(message.into());
        if sources.len() > MAX_FONT_FILES {
            return Err(invalid("native font file count exceeds 256"));
        }
        let mut catalog = Self::default();
        let mut total_bytes = 0u64;
        let mut retained_bytes = 0u64;
        for bytes in sources {
            let length = bytes.len() as u64;
            if length == 0 || length > MAX_FONT_FILE_BYTES {
                return Err(invalid("native font size must be in 1..=32 MiB"));
            }
            total_bytes = total_bytes
                .checked_add(length)
                .ok_or_else(|| invalid("native font input byte count overflow"))?;
            if total_bytes > MAX_FONT_TOTAL_BYTES {
                return Err(invalid("native font input bytes exceed 64 MiB"));
            }
            let faces = ttf_parser::fonts_in_collection(bytes).unwrap_or(1);
            if faces == 0 || faces > MAX_COLLECTION_FACES {
                return Err(invalid(
                    "native font collection face count must be in 1..=64",
                ));
            }
            let retained = length
                .checked_mul(u64::from(faces))
                .and_then(|bytes| retained_bytes.checked_add(bytes))
                .ok_or_else(|| invalid("native font retained byte count overflow"))?;
            if retained > MAX_FONT_TOTAL_BYTES {
                return Err(invalid("native font retained copies exceed 64 MiB"));
            }
            retained_bytes = retained;
            for face_index in 0..faces {
                ttf_parser::Face::parse(bytes, face_index)
                    .map_err(|_| invalid("native font face could not be parsed"))?;
                let face = typeface_from_data(font_mgr, bytes, face_index)
                    .ok_or_else(|| invalid("native font face could not be constructed by Skia"))?;
                let family = face.family_name().trim().to_lowercase();
                if family.is_empty() {
                    return Err(invalid("native font face has no family name"));
                }
                catalog.insert_face(bytes, face_index, face);
            }
        }
        Ok(catalog)
    }

    pub fn load(font_mgr: &FontMgr, sources: &[PathBuf]) -> Self {
        let mut catalog = Self::default();
        let mut total_bytes = 0u64;
        let mut retained_bytes = 0u64;
        let mut loaded_files = 0usize;
        let mut seen = std::collections::HashSet::new();
        for source in sources {
            let mut source_catalog = Self::default();
            let mut paths = if source.is_file() {
                vec![source.clone()]
            } else {
                std::fs::read_dir(source)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|entry| entry.path())
                    .collect::<Vec<_>>()
            };
            paths.sort();
            for path in paths {
                let supported = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        matches!(
                            extension.to_ascii_lowercase().as_str(),
                            "ttf" | "otf" | "ttc"
                        )
                    });
                if !supported || !seen.insert(path.clone()) {
                    continue;
                }
                if loaded_files >= MAX_FONT_FILES {
                    break;
                }
                let Ok(metadata) = path.metadata() else {
                    continue;
                };
                if !metadata.is_file()
                    || metadata.len() == 0
                    || metadata.len() > MAX_FONT_FILE_BYTES
                    || total_bytes.saturating_add(metadata.len()) > MAX_FONT_TOTAL_BYTES
                {
                    continue;
                }
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                if bytes.len() as u64 > MAX_FONT_FILE_BYTES
                    || total_bytes.saturating_add(bytes.len() as u64) > MAX_FONT_TOTAL_BYTES
                {
                    continue;
                }
                total_bytes += bytes.len() as u64;
                loaded_files += 1;
                let faces = ttf_parser::fonts_in_collection(&bytes)
                    .unwrap_or(1)
                    .min(MAX_COLLECTION_FACES);
                for face_index in 0..faces {
                    // Each face owns a fresh Data copy of the complete collection.
                    // Account retained copies, not just input files.
                    if retained_bytes.saturating_add(bytes.len() as u64) > MAX_FONT_TOTAL_BYTES {
                        break;
                    }
                    if ttf_parser::Face::parse(&bytes, face_index).is_err() {
                        continue;
                    }
                    if let Some(face) = typeface_from_data(font_mgr, &bytes, face_index) {
                        retained_bytes += bytes.len() as u64;
                        source_catalog.insert_face(&bytes, face_index, face);
                    }
                }
            }
            // The first supplied path owns the family's complete style set.
            // Within that directory retain all bounded faces for CSS matching
            // and P47's per-character coverage fallback;
            // later paths cannot override it with a closer style match.
            for (family, faces) in source_catalog.families {
                catalog.families.entry(family).or_insert(faces);
            }
            for (name, face) in source_catalog.postscript_faces {
                catalog.postscript_faces.entry(name).or_insert(face);
            }
            if loaded_files >= MAX_FONT_FILES {
                break;
            }
        }
        catalog
    }

    fn insert_face(&mut self, data: &[u8], face_index: u32, face: Typeface) {
        if let Some(name) = face.post_script_name() {
            self.postscript_faces.entry(name).or_insert(face.clone());
        }
        for family in supplied_family_names(data, face_index, &face.family_name()) {
            let family = family.trim().to_lowercase();
            if family.is_empty() {
                continue;
            }
            let faces = self.families.entry(family).or_default();
            if !faces
                .iter()
                .any(|existing| existing.unique_id() == face.unique_id())
            {
                faces.push(face.clone());
            }
        }
    }

    /// Link an explicit document family to the supplied family's existing faces.
    /// Cloning Typeface handles shares the retained bytes without reloading them.
    pub fn alias_family(&mut self, requested: &str, supplied: &str) -> bool {
        let requested = requested.trim().to_lowercase();
        if requested.is_empty() {
            return false;
        }
        let Some(faces) = self.families.get(&supplied.trim().to_lowercase()).cloned() else {
            return false;
        };
        self.families.insert(requested, faces);
        true
    }

    pub fn candidates(&self, family: &str, style: FontStyle) -> Vec<Typeface> {
        let mut result = Vec::new();
        for family in family
            .split(',')
            .map(|family| family.trim().trim_matches(['\'', '"']))
        {
            if let Some(faces) = self.families.get(&family.to_lowercase()) {
                let mut faces = faces.clone();
                // Stable sorting keeps the first supplied program on a tie.
                faces.sort_by_key(|face| supplied_style_rank(style, face.font_style()));
                result.extend(faces);
            } else if let Some(face) = self.postscript_faces.get(family) {
                // 명시적 face 요청에는 다른 굵기의 같은 family를 대입하지 않는다.
                result.push(face.clone());
            }
        }
        result
    }
}

/// CSS/Skia matches width, then slant, then weight (including 400..=500).
/// This is the same ordering as SkFontStyleSet::matchStyleCSS3.
fn supplied_style_rank(requested: FontStyle, actual: FontStyle) -> ((i32, i32), usize, (i32, i32)) {
    let rw = *requested.width();
    let cw = *actual.width();
    let width = if rw <= 5 {
        if cw <= rw {
            (0, rw - cw)
        } else {
            (1, cw - rw)
        }
    } else if cw >= rw {
        (0, cw - rw)
    } else {
        (1, rw - cw)
    };
    let slant_order = [[0, 2, 1], [2, 0, 1], [2, 1, 0]];
    let slant = slant_order[requested.slant() as usize][actual.slant() as usize];
    let rw = *requested.weight();
    let cw = *actual.weight();
    let weight = if rw < 400 {
        if cw <= rw {
            (0, rw - cw)
        } else {
            (1, cw - rw)
        }
    } else if rw <= 500 {
        if (rw..=500).contains(&cw) {
            (0, cw - rw)
        } else if cw < rw {
            (1, rw - cw)
        } else {
            (2, cw - 500)
        }
    } else if cw >= rw {
        (0, cw - rw)
    } else {
        (1, rw - cw)
    };
    (width, slant, weight)
}
