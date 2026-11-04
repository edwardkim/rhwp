//! Local filesystem adapter for the byte-only ingest image builder.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};

use rhwp::document_core::builders::exam_paper::{
    ingest_image_references, IngestImage, IngestImageLimits,
};
use rhwp::error::HwpError;
use rhwp::parser::ingest::schema::IngestDocument;

pub(super) fn load_images(
    ingest: &IngestDocument,
    media_dir: Option<&str>,
    limits: IngestImageLimits,
) -> Result<HashMap<String, IngestImage>, HwpError> {
    let refs = ingest_image_references(ingest);
    if refs.is_empty() {
        return Ok(HashMap::new());
    }
    let fail = |detail: String| HwpError::InvalidFile(format!("INGEST_IMAGE_PATH: {detail}"));
    let base = media_dir.ok_or_else(|| fail("--media-dir is required for images".into()))?;
    let base = Path::new(base)
        .canonicalize()
        .map_err(|e| fail(e.to_string()))?;
    if !base.is_dir() {
        return Err(fail("--media-dir must be a directory".into()));
    }
    let mut cache = HashMap::new();
    let mut result = HashMap::new();
    let mut bytes = 0usize;
    let mut pixels = 0u64;
    for id in refs {
        let path = Path::new(&id);
        if id.is_empty()
            || path.is_absolute()
            || path.components().any(|p| {
                matches!(
                    p,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(fail(format!(
                "image reference must stay within --media-dir: {id}"
            )));
        }
        let full = base
            .join(path)
            .canonicalize()
            .map_err(|e| fail(format!("{id}: {e}")))?;
        if !full.starts_with(&base) || !full.is_file() {
            return Err(fail(format!(
                "image reference is not a file within --media-dir: {id}"
            )));
        }
        if let Some(image) = cache.get(&full) {
            result.insert(id, IngestImage::clone(image));
            continue;
        }
        let file = File::open(&full).map_err(|e| fail(format!("{id}: {e}")))?;
        let mut data = Vec::new();
        file.take(
            limits
                .max_image_bytes
                .min(rhwp::model::bin_data::MAX_BIN_DATA_BYTES) as u64
                + 1,
        )
        .read_to_end(&mut data)
        .map_err(|e| fail(format!("{id}: {e}")))?;
        bytes = bytes
            .checked_add(data.len())
            .ok_or_else(|| fail("image byte count overflow".into()))?;
        if bytes > limits.max_total_image_bytes {
            return Err(HwpError::InvalidFile(
                "INGEST_IMAGE_LIMIT: total image bytes exceeded".into(),
            ));
        }
        let image = IngestImage::from_bytes(data, limits)?;
        let (width, height) = image.pixel_size();
        pixels = pixels
            .checked_add(u64::from(width) * u64::from(height))
            .ok_or_else(|| fail("image pixel count overflow".into()))?;
        if pixels > limits.max_total_image_pixels {
            return Err(HwpError::InvalidFile(
                "INGEST_IMAGE_LIMIT: total image pixels exceeded".into(),
            ));
        }
        cache.insert(full, image.clone());
        result.insert(id, image);
    }
    Ok(result)
}
