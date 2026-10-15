//! Explicit font snapshots for a native host, separate from layout declarations.

use std::path::PathBuf;

use serde::Serialize;

use super::{NativeDocumentSession, NativeHostError};
use crate::renderer::font_environment::FontEnvironment;

pub const MAX_FONT_FILES: usize = 256;
pub const MAX_FONT_FILE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_FONT_TOTAL_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_FONT_COLLECTION_FACES: u32 = 64;

/// One validated source snapshot. A collection contributes every face, in order.
/// The digest identifies supplied bytes, not glyph coverage or installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeFontSource {
    pub blake3: String,
    pub byte_length: usize,
    pub face_count: u32,
}

/// Sources successfully installed in this renderer and a separate caller claim
/// about layout substitutions. System fallback availability is not fingerprinted
/// or certified by this state. Source order is significant; paths are not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeFontState {
    pub fingerprint: String,
    pub sources: Vec<NativeFontSource>,
    pub environment: Option<FontEnvironment>,
}

impl Default for NativeFontState {
    fn default() -> Self {
        Self::from_environment(None)
    }
}

impl NativeFontState {
    /// Preserve a preconfigured core's declaration without claiming any supplied
    /// renderer fonts. Used when the host takes ownership of an existing core.
    pub(super) fn from_environment(environment: Option<FontEnvironment>) -> Self {
        Self::new(Vec::new(), environment)
    }

    fn new(sources: Vec<NativeFontSource>, environment: Option<FontEnvironment>) -> Self {
        let mut hash = blake3::Hasher::new();
        hash.update(b"rhwp-native-font-snapshot-v1\0");
        hash.update(&(sources.len() as u64).to_le_bytes());
        for source in &sources {
            hash.update(&(source.byte_length as u64).to_le_bytes());
            hash.update(&source.face_count.to_le_bytes());
            hash.update(source.blake3.as_bytes());
        }
        // This concrete type contains only strings, vectors and BTreeMaps, so
        // serialization is infallible and map order is deterministic.
        let declaration = serde_json::to_vec(&environment)
            .expect("FontEnvironment contains only JSON-serializable values");
        hash.update(&(declaration.len() as u64).to_le_bytes());
        hash.update(&declaration);
        Self {
            fingerprint: hash.finalize().to_hex().to_string(),
            sources,
            environment,
        }
    }
}

impl NativeDocumentSession {
    pub fn font_state(&self) -> &NativeFontState {
        &self.fonts
    }

    /// Replace explicit TTF/OTF/TTC files and the layout declaration atomically.
    ///
    /// Files must be regular, nonempty and within the public limits above. Every
    /// collection face must parse and load; retained per-face copies are also
    /// bounded to 64 MiB. No directory or environment-path search is performed.
    /// Symlinks to regular files are accepted. The pre-open metadata check avoids
    /// ordinary FIFO blocking, but is not an adversarial filesystem race guard.
    ///
    /// Bytes are read once with a bounded reader and supplied directly to Skia.
    /// Later file changes cannot mutate this snapshot. Identical ordered contents
    /// and declaration return false, even at new paths. A successful change
    /// advances only font_generation, leaving document revision/dirty unchanged.
    /// Any ordinary error preserves the previous renderer, state and declaration.
    /// Without native-skia this returns FeatureDisabled before opening any file.
    pub fn set_font_sources(
        &mut self,
        paths: &[PathBuf],
        environment: Option<FontEnvironment>,
    ) -> Result<bool, NativeHostError> {
        #[cfg(feature = "native-skia")]
        {
            if let Some(environment) = &environment {
                environment.validate()?;
            }
            let (bytes, sources) = read_font_snapshots(paths)?;
            let next = NativeFontState::new(sources, environment);
            if next == self.fonts {
                return Ok(false);
            }
            let generation = self.font_generation.checked_add(1).ok_or_else(|| {
                NativeHostError::LimitExceeded("font generation exhausted".into())
            })?;
            let renderer = crate::renderer::skia::SkiaLayerRenderer::new()
                .with_cache_limits(
                    super::MAX_RENDER_CACHE_ENTRIES,
                    super::MAX_RENDER_CACHE_BYTES,
                )
                .try_with_font_data(&bytes)?;

            // Validation and renderer construction finish before the core can
            // change. set_font_environment checks batch/validation errors before
            // mutating; only infallible assignments follow its success here.
            self.core.set_font_environment(next.environment.clone())?;
            self.renderer = std::cell::OnceCell::from(renderer);
            self.fonts = next;
            self.font_generation = generation;
            Ok(true)
        }
        #[cfg(not(feature = "native-skia"))]
        {
            let _ = (paths, environment);
            Err(NativeHostError::FeatureDisabled)
        }
    }
}

#[cfg(feature = "native-skia")]
fn read_font_snapshots(
    paths: &[PathBuf],
) -> Result<(Vec<Vec<u8>>, Vec<NativeFontSource>), NativeHostError> {
    use std::fs::File;
    use std::io::Read;

    if paths.len() > MAX_FONT_FILES {
        return Err(NativeHostError::LimitExceeded(
            "native font file count exceeds 256".into(),
        ));
    }
    let mut snapshots = Vec::with_capacity(paths.len());
    let mut sources = Vec::with_capacity(paths.len());
    let mut total_bytes = 0usize;
    let mut retained_bytes = 0usize;
    for (index, path) in paths.iter().enumerate() {
        let io_error =
            |error| NativeHostError::InvalidArgument(format!("font source {index}: {error}"));
        let check_file = |metadata: &std::fs::Metadata| {
            if !metadata.is_file() || metadata.len() == 0 {
                return Err(NativeHostError::InvalidArgument(format!(
                    "font source {index} must be a nonempty regular file"
                )));
            }
            if metadata.len() > MAX_FONT_FILE_BYTES as u64 {
                return Err(NativeHostError::LimitExceeded(format!(
                    "font source {index} exceeds 32 MiB"
                )));
            }
            Ok(())
        };
        check_file(&std::fs::metadata(path).map_err(io_error)?)?;
        let file = File::open(path).map_err(io_error)?;
        check_file(&file.metadata().map_err(io_error)?)?;
        let allowed = MAX_FONT_FILE_BYTES.min(MAX_FONT_TOTAL_BYTES - total_bytes);
        let mut bytes = Vec::new();
        file.take(allowed as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > allowed {
            return Err(NativeHostError::LimitExceeded(
                "native font source exceeds 32 MiB or total inputs exceed 64 MiB".into(),
            ));
        }
        if bytes.is_empty() {
            return Err(NativeHostError::InvalidArgument(format!(
                "font source {index} became empty while reading"
            )));
        }
        let face_count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
        if face_count == 0 || face_count > MAX_FONT_COLLECTION_FACES {
            return Err(NativeHostError::LimitExceeded(format!(
                "font source {index} collection face count must be in 1..=64"
            )));
        }
        retained_bytes = bytes
            .len()
            .checked_mul(face_count as usize)
            .and_then(|size| retained_bytes.checked_add(size))
            .filter(|size| *size <= MAX_FONT_TOTAL_BYTES)
            .ok_or_else(|| {
                NativeHostError::LimitExceeded("native font retained copies exceed 64 MiB".into())
            })?;
        total_bytes += bytes.len();
        sources.push(NativeFontSource {
            blake3: blake3::hash(&bytes).to_hex().to_string(),
            byte_length: bytes.len(),
            face_count,
        });
        snapshots.push(bytes);
    }
    Ok((snapshots, sources))
}
