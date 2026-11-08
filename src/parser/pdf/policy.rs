//! One product policy for pre-copy checks, page selection and backend accounting.
use pdf_oxide::import_limits::PdfImportLimits;
use serde::{Deserialize, Serialize};

/// Product profiles select finite request counters, not process heap/time limits.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PdfImportProfile {
    #[default]
    Standard,
    Constrained,
}

impl std::str::FromStr for PdfImportProfile {
    type Err = super::PdfImportError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "standard" => Ok(Self::Standard),
            "constrained" => Ok(Self::Constrained),
            _ => Err(super::PdfImportError::new(
                "PDF_RESOURCE_POLICY",
                "unknown resource profile; use standard or constrained",
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportPolicy {
    pub schema_version: u32,
    pub max_source_pages: usize,
    pub max_selected_pages: usize,
    pub max_path_operations: usize,
    pub max_document_path_operations: usize,
    /// Emitted image placements across selected pages, including repeated paint
    /// of one resource. Pixel/decode limits independently bound bitmap work.
    pub max_document_image_placements: usize,
    pub max_image_pixels: usize,
    pub max_document_image_pixels: usize,
    pub max_text_bytes: usize,
    pub backend: PdfImportLimits,
}

impl Default for PdfImportPolicy {
    fn default() -> Self {
        let backend = PdfImportLimits {
            // Absolute request ceilings. Page count does not predict content
            // complexity or shared resources. These are not live-heap limits.
            operators: 8_000_000,
            // Includes scalar values inside dictionaries/arrays, not just xref
            // objects. A 57-page authenticated input needs 81,182 values.
            objects: 200_000,
            // A 64-megapixel RGBA image fits in one bounded output buffer.
            // Keep scratch, cumulative work and host memory limits independent.
            stream_bytes: 256 * 1024 * 1024,
            codec_allocation_bytes: 256 * 1024 * 1024,
            // Independent interpreter passes revisit the same parsed operators.
            // Keep their cumulative work independently finite.
            xobject_visits: 4_000_000,
            reference_visits: 4_000_000,
            spans: 120_000,
            entries: 400_000,
            // Cumulative allocations/copies, not simultaneously resident memory.
            // Large image documents exceed 4 GiB of work on wasm32 as well.
            decoded_bytes: 8 * 1024 * 1024 * 1024,
            codec_work_units: 2_000_000_000,
            // Full CJK faces commonly carry tens of thousands of CID widths
            // and TrueType mappings each, even on a single page. Allow several
            // such faces while retaining a finite request-wide expansion cap.
            // Every entry still charges 128 decoded bytes before allocation;
            // the independently narrowed decoded budget and explicit host caps
            // remain in force. This does not change the raw decoder defaults.
            font_entries: 1_000_000,
            ..PdfImportLimits::default()
        };
        Self {
            schema_version: 1,
            // Source traversal is independently bounded by the backend visit budget.
            max_source_pages: PdfImportLimits::default().reference_visits,
            max_selected_pages: super::MAX_PAGES,
            max_path_operations: 20_000,
            max_document_path_operations: 1_000_000,
            // Same finite request scale as XObject visits; inline images also
            // consume a placement even though they do not invoke an XObject.
            max_document_image_placements: backend.xobject_visits,
            max_image_pixels: 64_000_000,
            max_document_image_pixels: 2_000_000_000,
            max_text_bytes: 2 * 1024 * 1024,
            backend,
        }
    }
}

impl PdfImportPolicy {
    pub fn for_profile(profile: PdfImportProfile) -> Self {
        match profile {
            PdfImportProfile::Standard => Self::default(),
            PdfImportProfile::Constrained => {
                let backend = PdfImportLimits {
                    input_bytes: 16 * 1024 * 1024,
                    ..PdfImportLimits::default()
                };
                Self {
                    max_source_pages: backend.reference_visits,
                    max_selected_pages: 64,
                    max_document_path_operations: 80_000,
                    max_document_image_placements: backend.xobject_visits,
                    max_image_pixels: 4_000_000,
                    max_document_image_pixels: 16_000_000,
                    backend,
                    ..Self::default()
                }
            }
        }
    }

    /// Apply a strict JSON object of lower caps to this profile. Absent fields
    /// retain their profile value; null, duplicate/unknown keys and non-integer
    /// values are errors. Trusted Rust hosts can still construct an explicit
    /// finite policy; this adapter never raises the selected product profile.
    pub fn with_limits_json(mut self, json: &str) -> Result<Self, super::PdfImportError> {
        // Serde struct visitors also accept positional arrays. JSON policy
        // configuration deliberately has a named-object contract. Validate the
        // shape first, then deserialize the original text (not this Value), so
        // duplicate keys still reach the strict derived visitors.
        let shape: serde_json::Value = serde_json::from_str(json).map_err(|_| {
            super::PdfImportError::new("PDF_RESOURCE_POLICY", "invalid resource limits JSON")
        })?;
        if !shape.is_object() || shape.get("backend").is_some_and(|v| !v.is_object()) {
            return Err(super::PdfImportError::new(
                "PDF_RESOURCE_POLICY",
                "resource limits must be JSON objects",
            ));
        }
        let limits: PdfImportOverrides = serde_json::from_str(json).map_err(|_| {
            super::PdfImportError::new("PDF_RESOURCE_POLICY", "invalid resource limits JSON")
        })?;
        self = limits.apply_to(self)?;
        Ok(self)
    }

    /// Validate before adapters read/copy an input. The backend is the authority
    /// for its own counters, so its zero-limit validation is reused here.
    pub fn validate(&self) -> Result<(), super::PdfImportError> {
        if self.schema_version != 1
            || self.max_selected_pages > u16::MAX as usize
            || [
                self.max_source_pages,
                self.max_selected_pages,
                self.max_path_operations,
                self.max_document_path_operations,
                self.max_document_image_placements,
                self.max_image_pixels,
                self.max_document_image_pixels,
                self.max_text_bytes,
            ]
            .contains(&0)
        {
            return Err(super::PdfImportError::budget("invalid PDF import policy"));
        }
        pdf_oxide::import_limits::ImportBudget::new(self.backend)
            .map(|_| ())
            .map_err(super::PdfImportError::backend)
    }

    pub(crate) fn for_selected_pages(mut self, pages: usize) -> Self {
        let selected_work_limit = pages.saturating_mul(100_000).max(250_000);
        // One dense page may need more operators than a long text document.
        // Keep the profile/host request ceiling; collection buffers and decoded
        // work remain independently bounded before allocation.
        // Each explicit host cap governs its own counter. A small operator
        // budget must not also reduce reference/XObject traversal to that
        // number before even the selected page's content can be visited.
        self.backend.xobject_visits = self.backend.xobject_visits.min(selected_work_limit);
        self.backend.reference_visits = self.backend.reference_visits.min(selected_work_limit);
        self.backend.spans = self
            .backend
            .spans
            .min(pages.saturating_mul(1_000).max(40_000));
        // Collection entries cover shared container/xref data, content operand
        // arrays and paint-order/clip metadata, not just emitted page elements.
        // Keep the finite document-wide cap (and any lower host cap). Selected
        // page count cannot predict these independently bounded structures.
        // Decoded work now follows the whole request through Core, first save
        // and diagnostics. Shared source resources and repeated format copies
        // cannot be predicted from selected page count. Keep the finite
        // profile/host ceiling, just as for document-wide collection entries.
        // Path work follows the same request-wide rule as parsed operators.
        // Individual paths retain max_path_operations and lower host caps.
        self
    }

    pub fn max_input_bytes(&self) -> usize {
        self.backend.input_bytes
    }
}

// Deserialize to an integer wrapper rather than Option<usize>: an explicit null
// must not silently turn into an absent cap. Derived struct visitors also reject
// duplicate keys, including inside backend, before any input document is read.
#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct Limit(u64);

fn lower_cap<T: Copy + TryFrom<u64> + TryInto<u64>>(
    target: &mut T,
    value: Limit,
) -> Result<(), super::PdfImportError> {
    if value.0 == 0 || value.0 > (*target).try_into().unwrap_or(0) {
        return Err(super::PdfImportError::new(
            "PDF_RESOURCE_POLICY",
            "resource limits must be positive and not exceed the selected profile",
        ));
    }
    *target = T::try_from(value.0).map_err(|_| {
        super::PdfImportError::new(
            "PDF_RESOURCE_POLICY",
            "resource limit is not representable on this target",
        )
    })?;
    Ok(())
}

macro_rules! overrides {
    ($name:ident, $target:ty, $( $field:ident ),+ $(,)?) => {
        #[derive(Default, Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct $name {
            $(#[serde(default)] $field: PresentLimit,)+
        }
        impl $name {
            fn apply(self, target: &mut $target) -> Result<(), super::PdfImportError> {
                $(if let Some(value) = self.$field.0 {
                    lower_cap(&mut target.$field, value)?;
                })+
                Ok(())
            }
        }
    };
}

#[derive(Default)]
struct PresentLimit(Option<Limit>);

impl<'de> Deserialize<'de> for PresentLimit {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Limit::deserialize(d).map(|value| Self(Some(value)))
    }
}

overrides!(
    BackendOverrides,
    PdfImportLimits,
    input_bytes,
    stream_bytes,
    codec_allocation_bytes,
    codec_work_units,
    decoded_bytes,
    filters,
    nesting,
    objects,
    entries,
    operators,
    graphics_stack,
    string_bytes,
    text_bytes,
    spans,
    cmap_bytes,
    cmap_entries,
    cmap_scan_bytes,
    font_entries,
    reference_depth,
    reference_visits,
    form_depth,
    xobject_visits,
);

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PdfImportOverrides {
    #[serde(default)]
    max_source_pages: PresentLimit,
    #[serde(default)]
    max_selected_pages: PresentLimit,
    #[serde(default)]
    max_path_operations: PresentLimit,
    #[serde(default)]
    max_document_path_operations: PresentLimit,
    #[serde(default)]
    max_document_image_placements: PresentLimit,
    #[serde(default)]
    max_image_pixels: PresentLimit,
    #[serde(default)]
    max_document_image_pixels: PresentLimit,
    #[serde(default)]
    max_text_bytes: PresentLimit,
    #[serde(default)]
    backend: BackendOverrides,
}

impl PdfImportOverrides {
    pub fn apply_to(
        self,
        mut policy: PdfImportPolicy,
    ) -> Result<PdfImportPolicy, super::PdfImportError> {
        let target = &mut policy;
        for (field, value) in [
            (&mut target.max_source_pages, self.max_source_pages),
            (&mut target.max_selected_pages, self.max_selected_pages),
            (&mut target.max_path_operations, self.max_path_operations),
            (
                &mut target.max_document_path_operations,
                self.max_document_path_operations,
            ),
            (
                &mut target.max_document_image_placements,
                self.max_document_image_placements,
            ),
            (&mut target.max_image_pixels, self.max_image_pixels),
            (
                &mut target.max_document_image_pixels,
                self.max_document_image_pixels,
            ),
            (&mut target.max_text_bytes, self.max_text_bytes),
        ] {
            if let Some(value) = value.0 {
                lower_cap(field, value)?;
            }
        }
        self.backend.apply(&mut target.backend)?;
        policy.validate()?;
        Ok(policy)
    }
}
