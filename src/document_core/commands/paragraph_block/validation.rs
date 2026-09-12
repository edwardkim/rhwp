//! Strict source support and reference closure, separate from insertion and paste.
mod references;
mod support;

use super::{ParagraphBlockBudget, RepeatParagraphBlockRequest};
use crate::{
    document_core::DocumentCore,
    error::HwpError,
    model::{control::Control, paragraph::Paragraph, shape::ShapeObject},
};
use serde::Serialize;

/// Source-relative owned path. These are not DSEL addresses or persistent IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

impl DocumentCore {
    /// Validate supported controls and field/connector closure without editing.
    /// Resource/style existence and save compatibility are NOT certified here.
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
        Ok(budget)
    }
}
