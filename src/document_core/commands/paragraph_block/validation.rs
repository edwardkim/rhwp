//! Strict source support and reference closure, separate from insertion and paste.
mod references;
mod resources;
mod support;
pub(super) use support::paths;

use super::{ParagraphBlockBudget, RepeatParagraphBlockRequest};
use crate::{
    document_core::DocumentCore,
    error::HwpError,
    model::{control::Control, paragraph::Paragraph, shape::ShapeObject},
};
use serde::Serialize;

/// Source-relative owned path. These are not DSEL addresses or persistent IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", content = "index", rename_all = "camelCase")]
pub enum ParagraphBlockPathStep {
    Paragraph(usize),
    Control(usize),
    Shape,
    Cell(usize),
    TextBox,
    Caption,
    GroupChild(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphBlockValidationError {
    pub code: String,
    /// Relative to request.source_start in request.section_index.
    pub path: Vec<ParagraphBlockPathStep>,
    pub detail: String,
}

impl std::fmt::Display for ParagraphBlockValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "paragraph block {} at {:?}: {}",
            self.code, self.path, self.detail
        )
    }
}
impl std::error::Error for ParagraphBlockValidationError {}
impl From<HwpError> for ParagraphBlockValidationError {
    fn from(error: HwpError) -> Self {
        Self {
            code: "budgetOrAddress".into(),
            path: vec![],
            detail: error.to_string(),
        }
    }
}

pub(super) fn reject(
    path: &[ParagraphBlockPathStep],
    code: &str,
    detail: impl Into<String>,
) -> ParagraphBlockValidationError {
    ParagraphBlockValidationError {
        code: code.into(),
        path: path.to_vec(),
        detail: detail.into(),
    }
}

#[derive(Clone, Copy)]
enum SourceNode<'a> {
    Paragraph(&'a Paragraph),
    Control(&'a Control),
    Shape(&'a ShapeObject),
}
struct Located<'a> {
    node: SourceNode<'a>,
    path: Vec<ParagraphBlockPathStep>,
}

/// Reuse the bounded ownership walk for template selection; paths stay source-relative.
pub(super) fn paragraphs<'a>(
    paragraphs: &'a [Paragraph],
    request: &RepeatParagraphBlockRequest,
) -> Result<Vec<(Vec<ParagraphBlockPathStep>, &'a Paragraph)>, ParagraphBlockValidationError> {
    Ok(support::inspect(paragraphs, request)?
        .into_iter()
        .filter_map(|located| match located.node {
            SourceNode::Paragraph(p) => Some((located.path, p)),
            _ => None,
        })
        .collect())
}

/// Validate a detached, bounded source against the original document's references.
/// The caller must budget the source before cloning it; this does not allocate IDs.
pub(super) fn row_source(
    doc: &crate::model::document::Document,
    paras: &[Paragraph],
    original: &crate::model::table::Table,
    cells: &[usize],
    request: &RepeatParagraphBlockRequest,
) -> Result<(), ParagraphBlockValidationError> {
    let nodes = support::inspect(paras, request)?;
    // Closure uses actual owner addresses in the original document. A cloned
    // projection would wrongly classify even a same-cell field end as external.
    let mut originals = Vec::new();
    for (relative, index) in cells.iter().enumerate() {
        for mut located in support::inspect(&original.cells[*index].paragraphs, request)? {
            let mut path = vec![
                ParagraphBlockPathStep::Paragraph(0),
                ParagraphBlockPathStep::Control(0),
                ParagraphBlockPathStep::Cell(relative),
            ];
            path.append(&mut located.path);
            located.path = path;
            originals.push(located);
        }
    }
    references::validate(doc, &originals, request.limits.max_document_nodes)?;
    resources::validate(doc, &nodes, request)
}

impl DocumentCore {
    /// Validate supported controls and field/connector closure without editing.
    /// Also checks modeled shared resource references without loading BinData.
    /// Resource bytes, opaque DocInfo payloads and save compatibility are not certified.
    pub fn validate_paragraph_block_native(
        &self,
        request: &RepeatParagraphBlockRequest,
    ) -> Result<ParagraphBlockBudget, ParagraphBlockValidationError> {
        // Address/option validation first, with count=0 semantics, before walking.
        let no_op = RepeatParagraphBlockRequest {
            count: 0,
            ..request.clone()
        };
        let zero = self.paragraph_block_budget_native(&no_op)?;
        if request.count == 0 {
            return Ok(zero);
        }
        if request.count > request.limits.max_copies
            || (request.source_end - request.source_start)
                .checked_mul(request.count)
                .is_none_or(|n| n > request.limits.max_paragraphs)
        {
            return Err(reject(
                &[],
                "budgetOrAddress",
                "copy/paragraph budget exceeded",
            ));
        }
        let paragraphs = &self.document().sections[request.section_index].paragraphs
            [request.source_start..request.source_end];
        // Support walk also has node/depth/path caps before any path allocations.
        // In particular it reports Form's exact path before invoking serde costs.
        let nodes = support::inspect(paragraphs, request)?;
        let budget = self.paragraph_block_budget_native(request)?;
        references::validate(self.document(), &nodes, request.limits.max_document_nodes)?;
        resources::validate(self.document(), &nodes, request)?;
        Ok(budget)
    }
}
