use crate::doclang::resources::ResourcePolicy;

/// DocLang spec version targeted by this crate.
///
/// The orchestrator re-exports this from `crate::doclang::DOCLANG_VERSION`; this local
/// definition exists so that `options.rs` is self-contained and rhwp-agnostic.
pub const DOCLANG_VERSION: &str = "0.6";

/// Conversion fidelity mode.
///
/// - `Lean` — emit only DocLang v0.6 standard elements; information that has
///   no DocLang representation is recorded in a `LossReport` and discarded.
/// - `Preserve` — unmappable HWP properties are emitted as namespaced
///   `<custom ns="hwp:…">` elements so that round-trip tooling can recover them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Mode {
    /// Output only standard DocLang elements; losses are reported.
    #[default]
    Lean,
    /// Preserve unmappable properties in `<custom>` elements.
    Preserve,
}

/// Options passed to the `convert()` entry point.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvertOptions {
    /// Fidelity mode (default: `Lean`).
    pub mode: Mode,
    /// DocLang version written into the output root element attribute.
    /// Defaults to [`DOCLANG_VERSION`].
    pub doclang_version: &'static str,
    /// Emit v2 `<location>` bounding-box elements on blocks whose geometry can
    /// be resolved from the rhwp render tree (default: `false`).
    ///
    /// When `false` the converter produces byte-identical output to a build
    /// without location support — the geometry pass (which re-layouts every
    /// page and is therefore not free) is skipped entirely.
    ///
    /// When `true`, the pipeline additionally runs `DocumentCore::from_bytes`
    /// then per-page render-tree layout, joins the resulting bounding boxes to
    /// the Semantic IR by model provenance `(section, paragraph, control)`, and
    /// the writer emits four `<location value="N" resolution="512"/>` elements
    /// `x_min, y_min, x_max, y_max` in element-head order on each block that
    /// resolved a box. Blocks with no resolvable geometry are emitted unchanged.
    pub with_location: bool,
    /// How picture and attachment bytes are referenced by exporters.
    ///
    /// The default is [`ResourcePolicy::Inline`], preserving the original
    /// self-contained XML output. Asset policies produce deterministic resource
    /// names from block locations so XML, Markdown, and semantic payloads can
    /// agree on the same references.
    pub resource_policy: ResourcePolicy,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        ConvertOptions {
            mode: Mode::Lean,
            doclang_version: DOCLANG_VERSION,
            with_location: false,
            resource_policy: ResourcePolicy::Inline,
        }
    }
}
