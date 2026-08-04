//! DocLang 내보내기 — HWP 5.0 / HWPX 문서를 DocLang v0.6 XML 로 변환한다.
//!
//! 원래 `hangulang` 크레이트였던 코드를 rhwp 안으로 이식한 모듈이다.
//!
//! # Pipeline
//!
//! ```text
//! [crate::parse_document(&[u8])]  →  Document (rhwp IR)
//!         │  adapter: Phase A — Adapter/Lowering
//!         ▼
//! [Semantic IR (SirDocument)]
//!         │  eqedit pass: Formula::raw_eqedit → Formula::latex
//!         │  writer: Phase B — DocLang Mapping
//!         ▼
//! [DocLang XML]  +  LossReport
//! ```
//!
//! rhwp 타입에 의존하도록 허용된 유일한 모듈은 [`adapter`] 다.
//!
//! # Quick start
//!
//! ```no_run
//! use rhwp::doclang::{convert, ConvertOptions};
//!
//! let data = std::fs::read("document.hwp").unwrap();
//! let outcome = convert(&data, &ConvertOptions::default()).unwrap();
//! println!("{}", outcome.xml);
//! ```

pub mod adapter;
pub mod eqedit;
pub mod error;
pub mod ir;
pub mod loss;
pub mod options;
pub mod resources;
pub mod writer;

pub use error::ConvertError;
pub use loss::report::{LossEntry, LossKind, LossReport};
pub use options::{ConvertOptions, Mode};
pub use resources::{ResourceAsset, ResourcePolicy};

/// DocLang spec version this crate targets. 0.x minor versions are breaking;
/// the emitted root element carries this literal version attribute.
pub const DOCLANG_VERSION: &str = "0.6";

/// The output of a successful [`convert`] call.
#[derive(Debug, Clone)]
pub struct ConvertOutcome {
    /// The serialised DocLang v0.6 XML string.
    pub xml: String,
    /// Binary assets referenced by `xml` when `ResourcePolicy::AssetDir` is
    /// selected. Empty for inline and URI-prefix resource policies.
    pub assets: Vec<ResourceAsset>,
    /// Loss report: information that could not be represented in DocLang.
    ///
    /// In `Mode::Lean` this may be non-empty; in `Mode::Preserve` it is
    /// typically empty because unmappable properties are emitted as `<custom>`
    /// elements.
    pub loss: LossReport,
}

/// Parsed and normalised semantic document before any concrete exporter runs.
#[derive(Debug, Clone)]
pub struct SemanticOutcome {
    /// rhwp-agnostic Semantic IR.
    pub document: ir::SirDocument,
    /// Resolved layout boxes keyed by provenance. Empty when location extraction
    /// is disabled or the layout pass cannot resolve a block.
    pub locations: ir::prov::LocationMap,
    /// Parser/adapter/eqedit losses collected before exporter-specific losses.
    pub loss: LossReport,
}

/// Convert raw HWP 5.0 / HWPX bytes to DocLang v0.6 XML.
///
/// # Pipeline
///
/// 1. **adapter** — detect format, parse via rhwp, lower to Semantic IR,
///    collecting a [`LossReport`] for properties that DocLang cannot express.
/// 2. **eqedit pass** — walk every [`ir::Formula`] in the SIR; for formulas
///    where `latex` is `None`, attempt [`eqedit::convert`].  On success the
///    `latex` field is populated; on failure the formula keeps `latex = None`
///    and a [`LossKind::FormulaFallback`] entry (including the raw EqEdit
///    script snippet) is appended to the loss report.
/// 3. **writer** — serialise the annotated SIR to DocLang XML, merging any
///    writer-side loss entries into the same report.
///
/// # Errors
///
/// Returns [`ConvertError`] for unsupported / encrypted / distribution
/// documents, unrecognised format, or XML serialisation failures.
pub fn convert(data: &[u8], opts: &ConvertOptions) -> Result<ConvertOutcome, ConvertError> {
    let mut semantic = extract_semantic(data, opts)?;

    // Phase B: serialise to DocLang XML, collecting any writer-side loss.
    let mut writer_loss = LossReport::new();
    let xml = writer::write_doclang_with_locations(
        &semantic.document,
        opts,
        &semantic.locations,
        &mut writer_loss,
    )?;
    semantic.loss.merge(writer_loss);

    let assets = resources::collect_assets(&semantic.document, &opts.resource_policy);

    Ok(ConvertOutcome {
        xml,
        assets,
        loss: semantic.loss,
    })
}

/// Parse and lower raw HWP 5.0 / HWPX bytes to the public Semantic IR.
pub fn extract_semantic(
    data: &[u8],
    opts: &ConvertOptions,
) -> Result<SemanticOutcome, ConvertError> {
    // Phase A: parse + lower to Semantic IR. Keep the parsed document so the v2
    // geometry pass can reuse it without a second parse.
    let (mut sir, mut loss, document) = adapter::build_sir_with_document(data, opts.mode)?;
    sir.doclang_version = opts.doclang_version;

    // eqedit pass: attempt LaTeX conversion for every Formula that has not yet
    // been converted (latex == None).  The adapter already sets raw_eqedit.
    run_eqedit_pass(&mut sir, &mut loss);

    // v2 geometry pass (gated): build the Prov -> Location map by laying out the
    // pages and joining render-tree bounding boxes onto the IR provenance. When
    // disabled the map stays empty and the writer emits no <location> elements,
    // producing byte-identical output to a location-free build.
    let locs = if opts.with_location {
        adapter::build_location_map(data, &document, &mut loss)
    } else {
        ir::prov::LocationMap::new()
    };

    Ok(SemanticOutcome {
        document: sir,
        locations: locs,
        loss,
    })
}

/// Walk every [`ir::Formula`] in the SIR tree and attempt `eqedit::convert`
/// for each formula that has `latex == None`.  Successful conversions populate
/// `formula.latex`; failures append a [`LossKind::FormulaFallback`] entry.
fn run_eqedit_pass(sir: &mut ir::SirDocument, loss: &mut LossReport) {
    for (si, section) in sir.sections.iter_mut().enumerate() {
        run_eqedit_blocks(&mut section.blocks, &format!("section[{si}]"), loss);
    }
}

fn run_eqedit_blocks(blocks: &mut [ir::Block], loc: &str, loss: &mut LossReport) {
    use ir::block::Block;

    for (bi, block) in blocks.iter_mut().enumerate() {
        let bloc = format!("{loc}/block[{bi}]");
        match block {
            Block::Formula(formula) => {
                if formula.latex.is_none() {
                    match eqedit::convert_with_degraded(&formula.raw_eqedit) {
                        Ok(outcome) => {
                            // Always keep the usable LaTeX. If any command-like
                            // tokens degraded to `\text{…}`, the conversion is
                            // semantically lossy: still set `latex`, but record
                            // a FormulaFallback entry so the loss is reported
                            // rather than passing as silently-correct LaTeX.
                            if !outcome.degraded.is_empty() {
                                loss.push(LossEntry {
                                    kind: LossKind::FormulaFallback,
                                    location: bloc.clone(),
                                    detail: format!(
                                        "eqedit conversion degraded {} token(s) to \\text{{…}}: [{}]; raw script: {}",
                                        outcome.degraded.len(),
                                        outcome.degraded.join(", "),
                                        formula.raw_eqedit
                                    ),
                                });
                            }
                            formula.latex = Some(outcome.latex);
                        }
                        Err(err) => {
                            loss.push(LossEntry {
                                kind: LossKind::FormulaFallback,
                                location: bloc.clone(),
                                detail: format!(
                                    "eqedit conversion failed ({err}); raw script: {}",
                                    formula.raw_eqedit
                                ),
                            });
                        }
                    }
                }
            }
            // Recurse into block types that may contain nested blocks.
            Block::Footnote { content, .. }
            | Block::PageHeader { content, .. }
            | Block::PageFooter { content, .. } => {
                run_eqedit_blocks(content, &bloc, loss);
            }
            Block::List { items, .. } => {
                for (ii, item) in items.iter_mut().enumerate() {
                    let item_loc = format!("{bloc}/ldiv[{ii}]");
                    run_eqedit_blocks(&mut item.content, &item_loc, loss);
                }
            }
            Block::Table(table) => {
                for (ci, cell) in table.cells.iter_mut().enumerate() {
                    let cell_loc = format!("{bloc}/cell[{ci}]");
                    run_eqedit_blocks(&mut cell.content, &cell_loc, loss);
                }
            }
            // Leaf blocks or blocks with no nested formulas.
            Block::Paragraph { .. }
            | Block::Heading { .. }
            | Block::Picture { .. }
            | Block::PageBreak
            | Block::ThreadStart { .. }
            | Block::ThreadContinuation { .. }
            | Block::Custom { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `convert()` must return a typed error — not panic — for completely
    /// invalid input bytes (no recognised format signature).
    #[test]
    fn convert_invalid_bytes_returns_error() {
        let garbage = b"this is not a valid hwp or hwpx file at all";
        let result = convert(garbage, &ConvertOptions::default());
        assert!(
            result.is_err(),
            "expected an error for garbage input, got Ok"
        );
        let err = result.unwrap_err();
        // Must be an UnsupportedFormat variant — the format gate fires first.
        assert!(
            matches!(err, ConvertError::UnsupportedFormat(_)),
            "expected UnsupportedFormat, got {err:?}"
        );
    }

    /// Empty slice also triggers the format gate (not a panic or parse error).
    #[test]
    fn convert_empty_slice_returns_unsupported_format() {
        let result = convert(&[], &ConvertOptions::default());
        assert!(matches!(
            result.unwrap_err(),
            ConvertError::UnsupportedFormat(_)
        ));
    }

    /// Truncated CFB header (valid magic, garbage body) should be a Parse error.
    #[test]
    fn convert_truncated_cfb_is_parse_error() {
        // CFB magic bytes so the format gate accepts it, then garbage body.
        let mut data = vec![0xD0u8, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        data.extend_from_slice(&[0u8; 64]);
        let err = convert(&data, &ConvertOptions::default()).unwrap_err();
        assert!(
            matches!(err, ConvertError::Parse(_)),
            "expected Parse error, got {err:?}"
        );
    }
}
