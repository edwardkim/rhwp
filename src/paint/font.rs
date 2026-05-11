/// Export-local identifier for a font blob or collection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontBlobKey(pub String);

/// Export-local identifier for an exact font face inside a blob or collection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontFaceKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontDigest {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryResourceRef {
    pub kind: BinaryResourceKind,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryResourceKind {
    FontBlob,
    ExternalFont,
}

impl BinaryResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FontBlob => "fontBlob",
            Self::ExternalFont => "externalFont",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontExternalRef {
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontResourceSource {
    Embedded,
    Bundled,
    SystemResolved,
    ExternalUrl,
    UnresolvedFallback,
}

impl FontResourceSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::Bundled => "bundled",
            Self::SystemResolved => "systemResolved",
            Self::ExternalUrl => "externalUrl",
            Self::UnresolvedFallback => "unresolvedFallback",
        }
    }
}

/// Whether a font resource is sufficient for portable glyph-id replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontPortability {
    PortableBlob {
        digest: FontDigest,
        data_ref: BinaryResourceRef,
    },
    ExternalVerified {
        digest: FontDigest,
        external_ref: FontExternalRef,
    },
    ResolvedButNotEmbedded {
        digest: Option<FontDigest>,
    },
    SystemNameOnly,
    UnresolvedFallback,
}

impl FontPortability {
    pub fn kind(&self) -> FontPortabilityKind {
        match self {
            Self::PortableBlob { .. } => FontPortabilityKind::PortableBlob,
            Self::ExternalVerified { .. } => FontPortabilityKind::ExternalVerified,
            Self::ResolvedButNotEmbedded { .. } => FontPortabilityKind::ResolvedButNotEmbedded,
            Self::SystemNameOnly => FontPortabilityKind::SystemNameOnly,
            Self::UnresolvedFallback => FontPortabilityKind::UnresolvedFallback,
        }
    }

    pub fn is_self_contained_replayable(&self) -> bool {
        matches!(self, Self::PortableBlob { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontPortabilityKind {
    PortableBlob,
    ExternalVerified,
    ResolvedButNotEmbedded,
    SystemNameOnly,
    UnresolvedFallback,
}

impl FontPortabilityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PortableBlob => "portableBlob",
            Self::ExternalVerified => "externalVerified",
            Self::ResolvedButNotEmbedded => "resolvedButNotEmbedded",
            Self::SystemNameOnly => "systemNameOnly",
            Self::UnresolvedFallback => "unresolvedFallback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedName {
    pub locale: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontBlobResource {
    pub id: FontBlobKey,
    pub digest: Option<FontDigest>,
    pub source: FontResourceSource,
    pub data_ref: Option<BinaryResourceRef>,
    pub portability: FontPortability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFaceResource {
    pub id: FontFaceKey,
    pub blob_key: FontBlobKey,
    /// Explicit TTC/OTC face index. This stays visible so backend diagnostics
    /// and future CanvasKit/native Skia typeface construction are unambiguous.
    pub face_index: u32,
    pub postscript_name: Option<String>,
    pub family_names: Vec<LocalizedName>,
    pub style_names: Vec<LocalizedName>,
    pub weight_class: Option<u16>,
    pub width_class: Option<u16>,
    pub italic: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontResourceTable {
    pub blobs: Vec<FontBlobResource>,
    pub faces: Vec<FontFaceResource>,
}

impl FontResourceTable {
    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty() && self.faces.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariationAxisValue {
    pub tag: String,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenTypeFeatureSetting {
    pub tag: String,
    pub enabled: bool,
    pub value: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptTag(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapingEngineId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFallbackPolicyId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextDirection {
    Ltr,
    Rtl,
    Auto,
}

impl TextDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
}

impl WritingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
            Self::VerticalLr => "vertical-lr",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontInstanceKey {
    pub face_key: FontFaceKey,
    pub size_px: f64,
    pub variations: Vec<VariationAxisValue>,
    pub synthetic_bold: bool,
    pub synthetic_italic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeKey {
    pub font_instance: FontInstanceKey,
    pub direction: TextDirection,
    pub writing_mode: WritingMode,
    pub script: Option<ScriptTag>,
    pub language: Option<LanguageTag>,
    pub features: Vec<OpenTypeFeatureSetting>,
    pub shaping_engine: ShapingEngineId,
    pub fallback_policy: FontFallbackPolicyId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphRunReplayEligibility {
    Portable,
    ConditionalExternalFont,
    LocalDiagnosticOnly,
    NotReplayable,
}

impl GlyphRunReplayEligibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Portable => "portable",
            Self::ConditionalExternalFont => "conditionalExternalFont",
            Self::LocalDiagnosticOnly => "localDiagnosticOnly",
            Self::NotReplayable => "notReplayable",
        }
    }
}

impl From<FontPortabilityKind> for GlyphRunReplayEligibility {
    fn from(kind: FontPortabilityKind) -> Self {
        match kind {
            FontPortabilityKind::PortableBlob => Self::Portable,
            FontPortabilityKind::ExternalVerified => Self::ConditionalExternalFont,
            FontPortabilityKind::ResolvedButNotEmbedded | FontPortabilityKind::SystemNameOnly => {
                Self::LocalDiagnosticOnly
            }
            FontPortabilityKind::UnresolvedFallback => Self::NotReplayable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_portability_maps_to_replay_eligibility() {
        assert_eq!(
            GlyphRunReplayEligibility::from(FontPortabilityKind::PortableBlob),
            GlyphRunReplayEligibility::Portable
        );
        assert_eq!(
            GlyphRunReplayEligibility::from(FontPortabilityKind::ExternalVerified),
            GlyphRunReplayEligibility::ConditionalExternalFont
        );
        assert_eq!(
            GlyphRunReplayEligibility::from(FontPortabilityKind::ResolvedButNotEmbedded),
            GlyphRunReplayEligibility::LocalDiagnosticOnly
        );
        assert_eq!(
            GlyphRunReplayEligibility::from(FontPortabilityKind::SystemNameOnly),
            GlyphRunReplayEligibility::LocalDiagnosticOnly
        );
        assert_eq!(
            GlyphRunReplayEligibility::from(FontPortabilityKind::UnresolvedFallback),
            GlyphRunReplayEligibility::NotReplayable
        );
    }
}
