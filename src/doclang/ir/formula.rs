/// A mathematical formula captured from an HWP `Equation` control.
///
/// `raw_eqedit` always contains the verbatim EqEdit script (e.g. `"1 over 2"`).
/// `latex` is populated by the `eqedit` module if conversion succeeds; it is
/// `None` when conversion fails or has not yet been attempted, in which case
/// the writer emits a placeholder and records a `LossReport` warning.
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    /// Raw EqEdit script as stored in the HWP file.
    pub raw_eqedit: String,
    /// LaTeX equivalent, produced by the `eqedit` conversion module.
    /// `None` means conversion failed or was not attempted.
    pub latex: Option<String>,
    /// Model provenance for v2 `<location>` join; `None` when location is
    /// disabled or the formula is cell-internal (no global provenance).
    pub prov: Option<super::prov::Prov>,
}
