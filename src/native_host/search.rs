//! Revision-stamped search over the selectable body and table-cell text.

use std::mem::size_of;

use serde::{Deserialize, Serialize};

use super::accessibility::{
    NativeAccessibilityLimitation, NativeAccessibilityRange, NativeAccessibilityRequest,
};
use super::editor::NativeEditor;
use super::{NativeDocumentSession, NativeHostError};
use crate::document_core::native_interaction::{
    NativeCellPathEntry, NativeTextSelection, NATIVE_CELL_PATH_MAX_DEPTH,
};
use crate::document_core::queries::search_query::find_matches_limited;

pub const MAX_SEARCH_QUERY_BYTES: usize = 256;
pub const MAX_SEARCH_MATCHES: usize = 128;
pub const MAX_SEARCH_TEXT_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeSearchRequest {
    pub query: String,
    #[serde(default)]
    pub case_sensitive: bool,
    pub max_matches: usize,
    pub range: Option<NativeAccessibilityRange>,
}

impl NativeSearchRequest {
    pub(super) fn validate(&self) -> Result<(), NativeHostError> {
        if self.query.is_empty() || self.query.len() > MAX_SEARCH_QUERY_BYTES {
            return Err(NativeHostError::InvalidArgument(
                "search query must contain 1..=256 UTF-8 bytes".into(),
            ));
        }
        if !(1..=MAX_SEARCH_MATCHES).contains(&self.max_matches) {
            return Err(NativeHostError::InvalidArgument(
                "search match limit must be in 1..=128".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn reservation_bytes(&self) -> Option<usize> {
        // An unread result retains both endpoints of every matching range.
        const LIMIT_PER_MATCH: usize = size_of::<NativeSearchMatch>()
            + 2 * NATIVE_CELL_PATH_MAX_DEPTH * size_of::<NativeCellPathEntry>();
        self.max_matches
            .checked_mul(LIMIT_PER_MATCH)?
            .checked_add(self.query.capacity())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSearchMatch {
    pub selection: NativeTextSelection,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSearchResult {
    pub matches: Vec<NativeSearchMatch>,
    pub truncated: bool,
    pub omitted_content: bool,
}

impl NativeSearchResult {
    pub(super) fn retained_bytes(&self) -> usize {
        let mut bytes = self
            .matches
            .capacity()
            .saturating_mul(size_of::<NativeSearchMatch>());
        for matched in &self.matches {
            for position in [&matched.selection.anchor, &matched.selection.focus] {
                bytes = bytes.saturating_add(
                    position
                        .cell_path
                        .capacity()
                        .saturating_mul(size_of::<NativeCellPathEntry>()),
                );
            }
        }
        bytes
    }
}

impl NativeDocumentSession {
    /// Query committed text with the same structural scope and traversal limits
    /// as accessibility. The worker checks the document identity and revision.
    pub fn search_text(
        &self,
        request: &NativeSearchRequest,
    ) -> Result<NativeSearchResult, NativeHostError> {
        request.validate()?;
        let snapshot = self.accessibility_snapshot(
            &NativeAccessibilityRequest {
                range: request.range,
                max_text_bytes: MAX_SEARCH_TEXT_BYTES,
                ..Default::default()
            },
            None,
        )?;
        let mut result = NativeSearchResult {
            matches: Vec::with_capacity(request.max_matches),
            truncated: snapshot.truncated,
            omitted_content: snapshot.limitations.iter().any(|limit| {
                use NativeAccessibilityLimitation::*;
                !matches!(
                    limit,
                    NodeLimit | TextLimit | DepthLimit | TraversalLimit | GeometryLimit
                )
            }),
        };
        let core = self.document();
        for node in snapshot.nodes {
            // A byte-limited prefix is incomplete. Matches are only offered in
            // paragraphs whose whole text was available to the matcher.
            if !node.text_complete {
                continue;
            }
            let remaining = request.max_matches - result.matches.len();
            let spans = find_matches_limited(
                &node.text,
                &request.query,
                request.case_sensitive,
                remaining + 1,
            );
            if spans.is_empty() {
                continue;
            }
            let start = NativeTextSelection {
                anchor: node.start.clone(),
                focus: node.start,
            };
            if NativeEditor::validate_selection(core, &start).is_err() {
                result.omitted_content = true;
                continue;
            }
            let editable = NativeEditor::check_editable_text_span(core, &start).is_ok();
            for &(offset, length) in spans.iter().take(remaining) {
                let mut selection = start.clone();
                // Snapshot paragraph text is bounded well below u32::MAX.
                selection.anchor.scalar_offset = offset as u32;
                selection.focus.scalar_offset = (offset + length) as u32;
                result.matches.push(NativeSearchMatch {
                    selection,
                    editable,
                });
            }
            if spans.len() > remaining {
                result.truncated = true;
                break;
            }
        }
        Ok(result)
    }
}
