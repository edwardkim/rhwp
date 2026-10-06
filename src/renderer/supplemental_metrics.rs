//! Session-local measurements for glyphs missing from the embedded metrics DB.
//!
//! These are natural advances, not ink bounds or document spacing. Providers prepare
//! a complete batch outside layout; consumers use an immutable, generation-bound
//! snapshot. Backend measurements do not identify an exact font face.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use unicode_segmentation::UnicodeSegmentation;

use super::TextStyle;

pub const MAX_SUPPLEMENTAL_ENTRIES: usize = 4096;
pub const MAX_SUPPLEMENTAL_KEY_BYTES: usize = 4 * 1024 * 1024;
const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const MAX_TOTAL_FONT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricBackend {
    Canvas2d,
    CanvasKit,
    NativeSkia,
    Svg,
}

/// Generation counters belong to the rendering session, not the saved document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricContext {
    pub document_generation: u64,
    pub font_generation: u64,
    pub backend: MetricBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricError {
    EditInProgress,
    ContextMismatch,
    InvalidStyle,
    UnsupportedCluster,
    InvalidAdvance,
    InvalidDescriptor,
    MalformedFont,
    MissingGlyph,
    LimitExceeded,
    DuplicateKey,
}

/// Paint must consume this evidence too. A verified source is not, by itself,
/// proof that a particular backend has loaded the font.
#[derive(Debug, Clone)]
pub enum MetricEvidence {
    BackendMeasured {
        descriptor: String,
    },
    VerifiedSource {
        sha256: [u8; 32],
        face_index: u32,
        glyph_id: u16,
        bytes: Arc<[u8]>,
    },
}

impl PartialEq for MetricEvidence {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BackendMeasured { descriptor: a }, Self::BackendMeasured { descriptor: b }) => {
                a == b
            }
            (
                Self::VerifiedSource {
                    sha256: a,
                    face_index: af,
                    glyph_id: ag,
                    ..
                },
                Self::VerifiedSource {
                    sha256: b,
                    face_index: bf,
                    glyph_id: bg,
                    ..
                },
            ) => a == b && af == bf && ag == bg,
            _ => false,
        }
        // Hashes are computed from validated bytes by the private entry builder.
        // Do not compare a multi-MiB font payload again for every cached glyph.
    }
}

type StyleKey = (u64, u64, bool, bool, char);

#[derive(Debug, Clone, PartialEq)]
pub struct SupplementalMetric {
    family: String,
    size_bits: u64,
    ratio_bits: u64,
    bold: bool,
    italic: bool,
    character: char,
    natural_advance_px: f64,
    evidence: MetricEvidence,
}

fn effective_size(style: &TextStyle) -> Result<f64, MetricError> {
    if !style.font_size.is_finite() || style.font_size <= 0.0 || style.font_family.is_empty() {
        return Err(MetricError::InvalidStyle);
    }
    Ok(style.script_draw_metrics(style.font_size, 0.0).0)
}

fn ratio_key(style: &TextStyle) -> Result<u64, MetricError> {
    if !style.ratio.is_finite() {
        return Err(MetricError::InvalidStyle);
    }
    // Match layout and paint's existing nonpositive-ratio fallback.
    Ok(if style.ratio > 0.0 { style.ratio } else { 1.0 }.to_bits())
}

fn single_scalar_cluster(text: &str) -> Result<char, MetricError> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) if !ch.is_control() => Ok(ch),
        _ => Err(MetricError::UnsupportedCluster),
    }
}

impl SupplementalMetric {
    /// Canvas measureText returns an advance at the actual (possibly condensed)
    /// CSS size. Convert it once before the common layout applies document ratio.
    /// The owner must still verify session/backend and normal positioned-paint
    /// capability; this constructor is not permission to reuse another backend.
    pub fn from_canvas_measurement(
        style: &TextStyle,
        cluster: &str,
        requested_descriptor: &str,
        resolved_descriptor: String,
        measured_advance_px: f64,
    ) -> Result<Self, MetricError> {
        effective_size(style)?;
        ratio_key(style)?;
        if style.font_family.len() > MAX_SUPPLEMENTAL_KEY_BYTES
            || requested_descriptor.len() > MAX_SUPPLEMENTAL_KEY_BYTES
            || resolved_descriptor.len() > MAX_SUPPLEMENTAL_KEY_BYTES
        {
            return Err(MetricError::LimitExceeded);
        }
        let font = super::canvas_text_font::CanvasTextFont::for_positioned_text(style, 0.0);
        if requested_descriptor != font.descriptor() {
            return Err(MetricError::InvalidDescriptor);
        }
        let natural = font
            .natural_advance(measured_advance_px)
            .ok_or(MetricError::InvalidAdvance)?;
        Self::backend_measured(style, cluster, resolved_descriptor, natural)
    }

    /// The descriptor must be the final Canvas2D paint descriptor, not a presence
    /// probe. The provider is responsible for measuring after fonts are ready.
    pub fn backend_measured(
        style: &TextStyle,
        cluster: &str,
        descriptor: String,
        natural_advance_px: f64,
    ) -> Result<Self, MetricError> {
        if descriptor.is_empty() || descriptor.chars().any(char::is_control) {
            return Err(MetricError::InvalidDescriptor);
        }
        Self::new(
            style,
            cluster,
            natural_advance_px,
            MetricEvidence::BackendMeasured { descriptor },
        )
    }

    /// Read the selected face's cmap and hmtx; never synthesize a square advance.
    /// Call outside the layout/paint hot path. The provider selects the style face.
    pub fn from_font(
        style: &TextStyle,
        cluster: &str,
        bytes: Arc<[u8]>,
        face_index: u32,
    ) -> Result<Self, MetricError> {
        if bytes.len() > MAX_FONT_BYTES {
            return Err(MetricError::LimitExceeded);
        }
        let character = single_scalar_cluster(cluster)?;
        let size = effective_size(style)?;
        let face =
            ttf_parser::Face::parse(&bytes, face_index).map_err(|_| MetricError::MalformedFont)?;
        let glyph = face
            .glyph_index(character)
            .filter(|id| id.0 != 0)
            .ok_or(MetricError::MissingGlyph)?;
        let advance = face
            .glyph_hor_advance(glyph)
            .ok_or(MetricError::MissingGlyph)?;
        let natural_advance_px = f64::from(advance) * size / f64::from(face.units_per_em());
        let evidence = MetricEvidence::VerifiedSource {
            sha256: Sha256::digest(&bytes).into(),
            face_index,
            glyph_id: glyph.0,
            bytes,
        };
        Self::new(style, cluster, natural_advance_px, evidence)
    }

    fn new(
        style: &TextStyle,
        cluster: &str,
        natural_advance_px: f64,
        evidence: MetricEvidence,
    ) -> Result<Self, MetricError> {
        if !natural_advance_px.is_finite() || natural_advance_px < 0.0 {
            return Err(MetricError::InvalidAdvance);
        }
        let descriptor_bytes = match &evidence {
            MetricEvidence::BackendMeasured { descriptor } => descriptor.len(),
            _ => 0,
        };
        if style.font_family.len() + descriptor_bytes + cluster.len() > MAX_SUPPLEMENTAL_KEY_BYTES {
            return Err(MetricError::LimitExceeded);
        }
        Ok(Self {
            family: style.font_family.clone(),
            size_bits: effective_size(style)?.to_bits(),
            ratio_bits: ratio_key(style)?,
            bold: style.bold,
            italic: style.italic,
            character: single_scalar_cluster(cluster)?,
            natural_advance_px,
            evidence,
        })
    }

    pub fn natural_advance_px(&self) -> f64 {
        self.natural_advance_px
    }

    pub fn evidence(&self) -> &MetricEvidence {
        &self.evidence
    }

    pub fn width_source(&self) -> &'static str {
        match self.evidence {
            MetricEvidence::BackendMeasured { .. } => "supplementalBackendMeasured",
            MetricEvidence::VerifiedSource { .. } => "supplementalVerifiedSource",
        }
    }

    fn key(&self) -> StyleKey {
        (
            self.size_bits,
            self.ratio_bits,
            self.bold,
            self.italic,
            self.character,
        )
    }

    pub(crate) fn request_key(&self) -> String {
        Sha256::digest(format!("{:?}", (&self.family, self.key())))
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

#[derive(Debug)]
pub struct SupplementalMetricSnapshot {
    context: MetricContext,
    active: AtomicBool,
    families: BTreeMap<String, BTreeMap<StyleKey, SupplementalMetric>>,
}

impl PartialEq for SupplementalMetricSnapshot {
    fn eq(&self, other: &Self) -> bool {
        // TextStyle equality is frequent; entries are immutable and shared.
        std::ptr::eq(self, other)
    }
}

impl SupplementalMetricSnapshot {
    pub fn context(&self) -> MetricContext {
        self.context
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn lookup(
        &self,
        context: MetricContext,
        style: &TextStyle,
        character: char,
    ) -> Option<&SupplementalMetric> {
        if context != self.context || !self.is_active() {
            return None;
        }
        let size = effective_size(style).ok()?;
        self.families.get(style.font_family.as_str())?.get(&(
            size.to_bits(),
            ratio_key(style).ok()?,
            style.bold,
            style.italic,
            character,
        ))
    }
}

/// Single-session owner. Failed batches leave the previous snapshot intact.
/// Reset invalidates even snapshots retained by old styles; the owner must also
/// discard the associated layout/paint caches before rendering the next frame.
pub struct SupplementalMetricStore {
    context: MetricContext,
    snapshot: Option<Arc<SupplementalMetricSnapshot>>,
}

impl SupplementalMetricStore {
    pub fn new(context: MetricContext) -> Self {
        Self {
            context,
            snapshot: None,
        }
    }

    pub fn snapshot(&self) -> Option<Arc<SupplementalMetricSnapshot>> {
        self.snapshot.clone()
    }

    pub fn replace(
        &mut self,
        context: MetricContext,
        entries: Vec<SupplementalMetric>,
    ) -> Result<(), MetricError> {
        if context != self.context {
            return Err(MetricError::ContextMismatch);
        }
        if entries.len() > MAX_SUPPLEMENTAL_ENTRIES {
            return Err(MetricError::LimitExceeded);
        }
        let mut families: BTreeMap<String, BTreeMap<StyleKey, SupplementalMetric>> =
            BTreeMap::new();
        let mut sources: BTreeMap<[u8; 32], Arc<[u8]>> = BTreeMap::new();
        let mut key_bytes = 0;
        let mut font_bytes = 0;
        for mut entry in entries {
            key_bytes += entry.family.len() + entry.character.len_utf8();
            match &mut entry.evidence {
                MetricEvidence::BackendMeasured { descriptor } => {
                    if context.backend != MetricBackend::Canvas2d {
                        return Err(MetricError::ContextMismatch);
                    }
                    key_bytes += descriptor.len();
                }
                MetricEvidence::VerifiedSource { sha256, bytes, .. } => {
                    if let Some(shared) = sources.get(sha256) {
                        *bytes = Arc::clone(shared);
                    } else {
                        font_bytes += bytes.len();
                        sources.insert(*sha256, Arc::clone(bytes));
                    }
                }
            }
            if key_bytes > MAX_SUPPLEMENTAL_KEY_BYTES || font_bytes > MAX_TOTAL_FONT_BYTES {
                return Err(MetricError::LimitExceeded);
            }
            if families
                .entry(entry.family.clone())
                .or_default()
                .insert(entry.key(), entry)
                .is_some()
            {
                return Err(MetricError::DuplicateKey);
            }
        }
        // Identical registrations do not invalidate styles or require a relayout.
        if self
            .snapshot
            .as_ref()
            .is_some_and(|old| old.families == families)
        {
            return Ok(());
        }
        self.invalidate();
        self.snapshot = Some(Arc::new(SupplementalMetricSnapshot {
            context,
            active: AtomicBool::new(true),
            families,
        }));
        Ok(())
    }

    pub fn bind_style(
        &self,
        context: MetricContext,
        style: &mut TextStyle,
    ) -> Result<(), MetricError> {
        if context != self.context {
            return Err(MetricError::ContextMismatch);
        }
        style.supplemental_metrics = self.snapshot();
        Ok(())
    }

    pub fn reset_context(&mut self, context: MetricContext) {
        self.invalidate();
        self.context = context;
    }

    fn invalidate(&mut self) {
        if let Some(old) = self.snapshot.take() {
            old.active.store(false, Ordering::Release);
        }
    }
}

impl Drop for SupplementalMetricStore {
    fn drop(&mut self) {
        self.invalidate();
    }
}

/// Exact descriptor prepared by Canvas, never an inferred fallback face name.
pub fn canvas_measured_descriptor<'a>(style: &'a TextStyle, cluster: &str) -> Option<&'a str> {
    let ch = single_scalar_cluster(cluster).ok()?;
    let snapshot = style.supplemental_metrics.as_ref()?;
    let entry = snapshot.lookup(snapshot.context(), style, ch)?;
    match entry.evidence() {
        MetricEvidence::BackendMeasured { descriptor } => Some(descriptor),
        _ => None,
    }
}

/// The current publication contract is scalar-positioned. Multi-scalar graphemes
/// must stay wholly on the old path, rather than partially replacing one scalar.
/// No additional segmentation/allocation occurs without an active snapshot.
pub(crate) fn standalone_scalar_mask(text: &str, style: &TextStyle) -> Option<Vec<bool>> {
    if !style.supplemental_metrics.as_ref()?.is_active() {
        return None;
    }
    Some(scalar_eligibility(text))
}

/// One Unicode grapheme policy for paragraph preparation and local measurement.
pub(crate) fn scalar_eligibility(text: &str) -> Vec<bool> {
    text.graphemes(true)
        .flat_map(|g| {
            let count = g.chars().count();
            std::iter::repeat_n(count == 1, count)
        })
        .collect()
}
