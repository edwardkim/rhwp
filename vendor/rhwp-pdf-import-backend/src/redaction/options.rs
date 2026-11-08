//! Public configuration and reporting types for destructive redaction
//! (#231, feature plan §5.1).
//!
//! These cross every binding surface, so they follow the workspace rule
//! for public option/report types: `#[non_exhaustive]` + `Default` so a
//! later field is non-breaking. They carry no logic (SRP) — the engine
//! consumes [`RedactionOptions`] and produces a [`RedactionReport`].

use super::region::DEFAULT_EDGE_PADDING;

/// What to do with Optional Content Groups (layers) during sanitization
/// (ISO 32000-1:2008 §8.11; feature plan §4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum OcgPolicy {
    /// Keep all OCGs as-is (only geometric redaction is applied).
    Keep,
    /// Remove OFF-by-default groups and their `BDC … EMC` spans so a
    /// hidden layer cannot be toggled back on. The default — a hidden
    /// layer is the classic redaction leak.
    #[default]
    StripHidden,
    /// Flatten to the default-visible configuration and discard the rest
    /// of `/OCProperties`.
    Flatten,
}

/// Options controlling a destructive redaction / sanitization pass.
///
/// Defaults are *safe*: metadata, JavaScript and embedded files are
/// scrubbed, hidden layers stripped, and an opaque overlay is drawn even
/// when the source `/Redact` annotation had no `/IC` (a transparent
/// "removed" area is visually confusing and risks operator error —
/// feature plan §4.2 step 4).
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct RedactionOptions {
    /// Scrub document/XMP/image metadata (`/Info`, `/Metadata`,
    /// `/PieceInfo`). Default `true`.
    pub scrub_metadata: bool,
    /// Remove document/field JavaScript and `/OpenAction`/`/AA`.
    /// Default `true`.
    pub remove_javascript: bool,
    /// Remove `/EmbeddedFiles`, file annotations and `/AF` arrays.
    /// Default `true`.
    pub remove_embedded_files: bool,
    /// Optional-content (layer) handling. Default [`OcgPolicy::StripHidden`].
    pub optional_content: OcgPolicy,
    /// Minimum region edge padding in points. Default
    /// [`DEFAULT_EDGE_PADDING`]; the effective padding is
    /// `max(this, 0.02 * region_height)` (feature plan §4.1).
    pub edge_padding: f32,
    /// Overlay fill when the source has no `/IC` (DeviceRGB).
    /// Default `[0.0, 0.0, 0.0]` (black).
    pub default_fill: [f32; 3],
    /// Draw a solid overlay even when no `/IC` was supplied.
    /// Default `true`.
    pub draw_overlay_when_no_ic: bool,
    /// Emit ISO 32000-2 `Redaction` artifact tags when a structure tree
    /// is present. Default `false`.
    pub emit_redaction_artifacts: bool,
}

impl Default for RedactionOptions {
    fn default() -> Self {
        Self {
            scrub_metadata: true,
            remove_javascript: true,
            remove_embedded_files: true,
            optional_content: OcgPolicy::StripHidden,
            edge_padding: DEFAULT_EDGE_PADDING,
            default_fill: [0.0, 0.0, 0.0],
            draw_overlay_when_no_ic: true,
            emit_redaction_artifacts: false,
        }
    }
}

/// What a redaction pass actually removed — returned to every binding so
/// callers can assert the redaction did real work (feature plan §5.1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub struct RedactionReport {
    /// Number of regions applied.
    pub regions: usize,
    /// Glyphs physically removed from content streams.
    pub glyphs_removed: usize,
    /// Images whose covered pixels were overwritten and re-encoded.
    pub images_modified: usize,
    /// Images deleted entirely (fully covered).
    pub images_removed: usize,
    /// Path subpaths dropped or geometry-clipped.
    pub paths_pruned: usize,
    /// Shared XObjects/patterns/Type3 fonts cloned-and-specialized.
    pub xobjects_specialized: usize,
    /// Count of removed top-level objects. For destructive redaction
    /// this is annotations dropped (`/Redact` and overlapping markup).
    /// `RedactionReport` is also the return type of
    /// [`crate::editor::DocumentEditor::sanitize_document`], where this
    /// instead counts the sanitized document-level roots that were
    /// stripped (`/Info`, catalog XMP `/Metadata`, document JavaScript,
    /// `/Names/EmbeddedFiles`) — i.e. "top-level items removed", not
    /// necessarily annotations. Treat it as a did-work signal, not an
    /// annotation-specific metric, when consuming a sanitize report.
    pub annotations_removed: usize,
    /// Fonts whose `/Widths`/`/ToUnicode` were scrubbed.
    pub fonts_scrubbed: usize,
    /// Total bytes removed from the document (best-effort estimate).
    pub bytes_removed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_safe() {
        let o = RedactionOptions::default();
        assert!(o.scrub_metadata);
        assert!(o.remove_javascript);
        assert!(o.remove_embedded_files);
        assert_eq!(o.optional_content, OcgPolicy::StripHidden);
        assert_eq!(o.edge_padding, DEFAULT_EDGE_PADDING);
        assert_eq!(o.default_fill, [0.0, 0.0, 0.0]);
        assert!(o.draw_overlay_when_no_ic);
        assert!(!o.emit_redaction_artifacts);
    }

    #[test]
    fn default_ocg_policy_strips_hidden() {
        assert_eq!(OcgPolicy::default(), OcgPolicy::StripHidden);
    }

    #[test]
    fn report_default_is_all_zero() {
        let r = RedactionReport::default();
        assert_eq!(r, RedactionReport::default());
        assert_eq!(r.regions, 0);
        assert_eq!(r.glyphs_removed, 0);
        assert_eq!(r.images_modified, 0);
        assert_eq!(r.images_removed, 0);
        assert_eq!(r.paths_pruned, 0);
        assert_eq!(r.xobjects_specialized, 0);
        assert_eq!(r.annotations_removed, 0);
        assert_eq!(r.fonts_scrubbed, 0);
        assert_eq!(r.bytes_removed, 0);
    }

    #[test]
    fn options_overridable_field_by_field() {
        let o = RedactionOptions {
            scrub_metadata: false,
            optional_content: OcgPolicy::Keep,
            edge_padding: 3.0,
            ..RedactionOptions::default()
        };
        assert!(!o.scrub_metadata);
        assert_eq!(o.optional_content, OcgPolicy::Keep);
        assert_eq!(o.edge_padding, 3.0);
        // untouched fields keep safe defaults
        assert!(o.remove_javascript);
        assert!(o.draw_overlay_when_no_ic);
    }
}
