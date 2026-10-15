//! Bounded custom/bundled font catalogs, separate from system font priority.

use std::collections::HashMap;
use std::path::PathBuf;

use skia_safe::{Data, FontMgr, FontStyle, Typeface};

use super::font_lookup::supplied_family_names;

const MAX_FONT_FILES: usize = 256;
const MAX_FONT_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_FONT_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_COLLECTION_FACES: u32 = 64;

#[derive(Clone, Default)]
pub(super) struct TypefaceCatalog {
    families: HashMap<String, Vec<Typeface>>,
}

impl TypefaceCatalog {
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
                    if let Some(face) =
                        font_mgr.new_from_data(Data::new_copy(&bytes), Some(face_index))
                    {
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
            if loaded_files >= MAX_FONT_FILES {
                break;
            }
        }
        catalog
    }

    fn insert_face(&mut self, data: &[u8], face_index: u32, face: Typeface) {
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
