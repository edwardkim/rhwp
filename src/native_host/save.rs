//! Two-phase, host-owned document saves. Preparing bytes does not mark a document
//! clean. Acknowledge only after successful host I/O and acceptance of the attached
//! content-loss report; cancel after failure or cancellation. No filesystem I/O is
//! performed here, and the host must serialize/cancel competing destination writes.
//!
//! Only the latest *successful* prepare token remains valid. Tokens are single-use
//! and process-unique across sessions. Failed preparation preserves the earlier
//! pending checkpoint. Acknowledging an older revision never marks newer edits
//! clean. Dirty tracks revisions, not byte equality (undo can remain dirty).

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use super::{NativeDocumentSession, NativeHostError};
use crate::document_core::native_interaction::NativeTextPosition;
use crate::serializer::{ContentLossReport, SerializedDocument};
use crate::DocumentCore;

static NEXT_SAVE_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeSaveFormat {
    Hwp,
    Hwpx,
}

/// What the serializer actually does with saved-caret metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeSaveCaretPolicy {
    StoredCore,
    CurrentFocus,
    /// Existing source settings.xml, or the serializer's existing default.
    /// The current editor focus is not persisted by this HWPX path.
    HwpxSourceSettings,
}

/// An untrusted transport value until matched against the session's pending save.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NativeSaveToken(u64);

impl NativeSaveToken {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Reconstitute a token received from a host. Zero, foreign and stale tokens
    /// are rejected by acknowledge/cancel, without consuming a valid checkpoint.
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

/// The bytes and report from exactly one canonical serializer invocation.
/// Ownership is independent of the session; superseding a token does not revoke
/// these bytes, but prevents their acknowledgement as the current pending save.
#[derive(Debug)]
#[must_use = "persist bytes with their content-loss report before acknowledging the token"]
pub struct PreparedSave {
    serialized: SerializedDocument,
    revision: u64,
    token: NativeSaveToken,
    caret_policy: NativeSaveCaretPolicy,
    caret_preserved_from_source: bool,
}

impl PreparedSave {
    pub fn bytes(&self) -> &[u8] {
        self.serialized.bytes()
    }

    pub fn content_loss(&self) -> &ContentLossReport {
        self.serialized.content_loss()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn token(&self) -> NativeSaveToken {
        self.token
    }

    pub fn caret_policy(&self) -> NativeSaveCaretPolicy {
        self.caret_policy
    }

    pub fn caret_saved(&self) -> bool {
        self.caret_policy == NativeSaveCaretPolicy::CurrentFocus
    }

    pub fn caret_preserved_from_source(&self) -> bool {
        self.caret_preserved_from_source
    }

    pub fn into_parts(self) -> (Vec<u8>, ContentLossReport, u64, NativeSaveToken) {
        let (bytes, content_loss) = self.serialized.into_parts();
        (bytes, content_loss, self.revision, self.token)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSaveStatus {
    pub revision: u64,
    pub saved_revision: Option<u64>,
    pub dirty: bool,
}

pub(super) struct PendingSaveCheckpoint {
    token: NativeSaveToken,
    revision: u64,
}

impl NativeDocumentSession {
    /// New/from_core sessions are unsaved. open_bytes starts at a saved revision.
    pub fn is_dirty(&self) -> bool {
        self.saved_revision != Some(self.revision())
    }

    /// Preserve the core's current stored caret. NativeEditor supplies its view
    /// selection through prepare_save_with_caret instead.
    pub fn prepare_save(
        &mut self,
        format: NativeSaveFormat,
    ) -> Result<PreparedSave, NativeHostError> {
        let serialized = match format {
            NativeSaveFormat::Hwp => self.core.export_hwp_with_adapter_snapshot_with_report()?,
            NativeSaveFormat::Hwpx => self.core.export_hwpx_native_with_report()?,
        };
        self.retain_prepared_save(
            serialized,
            match format {
                NativeSaveFormat::Hwp => NativeSaveCaretPolicy::StoredCore,
                NativeSaveFormat::Hwpx => NativeSaveCaretPolicy::HwpxSourceSettings,
            },
        )
    }

    /// Stamp a validated body focus only in the final export snapshot. The live
    /// core, its stored caret, revision, history and pending checkpoint are not
    /// modified if validation or serialization fails. Inline affinity/cells are
    /// intentionally outside this initial saved-caret contract. HWPX does not
    /// consume the core caret: use prepare_save(Hwpx) to preserve its existing
    /// source-settings policy without claiming the current focus was saved.
    pub fn prepare_save_with_caret(
        &mut self,
        format: NativeSaveFormat,
        focus: &NativeTextPosition,
    ) -> Result<PreparedSave, NativeHostError> {
        if format != NativeSaveFormat::Hwp {
            return Err(NativeHostError::Unsupported(
                "current-focus persistence is supported only for HWP; use prepare_save for HWPX"
                    .into(),
            ));
        }
        self.core
            .validate_native_text_position(focus)
            .map_err(|error| NativeHostError::InvalidArgument(error.to_string()))?;
        if !focus.cell_path.is_empty() {
            return Err(NativeHostError::Unsupported(
                "saved caret currently supports body text only".into(),
            ));
        }
        self.core
            .check_native_interaction_support(focus)
            .map_err(|error| NativeHostError::Unsupported(error.to_string()))?;
        let lowered = self.core.prepare_hwp_export_snapshot();
        let mut snapshot = lowered.document().clone();
        DocumentCore::set_document_caret_position(
            &mut snapshot,
            focus.section as usize,
            focus.paragraph as usize,
            focus.scalar_offset as usize,
        );
        let serialized =
            crate::serializer::serialize_document_with_report(&snapshot).map_err(|error| {
                NativeHostError::Core(crate::HwpError::RenderError(error.to_string()))
            })?;
        self.retain_prepared_save(serialized, NativeSaveCaretPolicy::CurrentFocus)
    }

    /// Confirm host persistence of the latest prepared bytes. A delayed success
    /// records the revision actually written, leaving any newer revision dirty.
    pub fn acknowledge_save(
        &mut self,
        token: NativeSaveToken,
    ) -> Result<NativeSaveStatus, NativeHostError> {
        let saved_revision = self.pending_save_revision(token)?;
        self.pending_save = None;
        self.saved_revision = Some(saved_revision);
        Ok(self.native_save_status())
    }

    /// Abandon a failed/cancelled write without changing the saved revision.
    pub fn cancel_save(
        &mut self,
        token: NativeSaveToken,
    ) -> Result<NativeSaveStatus, NativeHostError> {
        self.pending_save_revision(token)?;
        self.pending_save = None;
        Ok(self.native_save_status())
    }

    fn retain_prepared_save(
        &mut self,
        serialized: SerializedDocument,
        caret_policy: NativeSaveCaretPolicy,
    ) -> Result<PreparedSave, NativeHostError> {
        let caret_preserved_from_source = caret_policy == NativeSaveCaretPolicy::HwpxSourceSettings
            && self
                .core
                .document()
                .hwpx_aux_entry("settings.xml")
                .is_some();
        let token = NEXT_SAVE_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map(NativeSaveToken)
            .map_err(|_| NativeHostError::LimitExceeded("save token space exhausted".into()))?;
        let revision = self.revision();
        self.pending_save = Some(PendingSaveCheckpoint { token, revision });
        Ok(PreparedSave {
            serialized,
            revision,
            token,
            caret_policy,
            caret_preserved_from_source,
        })
    }

    fn pending_save_revision(&self, token: NativeSaveToken) -> Result<u64, NativeHostError> {
        self.pending_save
            .as_ref()
            .filter(|checkpoint| checkpoint.token == token)
            .map(|checkpoint| checkpoint.revision)
            .ok_or_else(|| {
                NativeHostError::InvalidArgument(
                    "unknown, superseded or consumed save token".into(),
                )
            })
    }

    fn native_save_status(&self) -> NativeSaveStatus {
        NativeSaveStatus {
            revision: self.revision(),
            saved_revision: self.saved_revision,
            dirty: self.is_dirty(),
        }
    }
}
