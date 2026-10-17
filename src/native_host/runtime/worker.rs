//! Native worker가 소유하는 문서·명령 실행과 completion 수명 처리.

use super::*;

enum DocumentOwner {
    Viewer(Box<NativeDocumentSession>),
    Editor(Box<NativeEditor>),
}
struct OwnedDocument {
    id: NativeRequestId,
    owner: DocumentOwner,
}
impl OwnedDocument {
    fn session(&self) -> &NativeDocumentSession {
        match &self.owner {
            DocumentOwner::Viewer(session) => session,
            DocumentOwner::Editor(editor) => editor.session(),
        }
    }
    fn editor(&mut self) -> Result<&mut NativeEditor, NativeHostError> {
        match &mut self.owner {
            DocumentOwner::Editor(editor) => Ok(editor),
            _ => Err(NativeHostError::Unsupported("beginEdit is required".into())),
        }
    }
    fn check(&self, target: NativeRuntimeTarget) -> Result<(), NativeHostError> {
        self.check_id(target.document_id)?;
        self.session().check_revision(target.revision)
    }
    fn check_id(&self, id: NativeRequestId) -> Result<(), NativeHostError> {
        if id == self.id {
            Ok(())
        } else {
            Err(NativeHostError::InvalidArgument(
                "stale or foreign document ID".into(),
            ))
        }
    }
    fn stamp(&self) -> Stamp {
        Stamp {
            target: NativeRuntimeTarget {
                document_id: self.id,
                revision: self.session().revision(),
            },
            font_generation: self.session().font_generation(),
        }
    }
    fn info(&self) -> NativeRuntimeInfo {
        let editor = match &self.owner {
            DocumentOwner::Editor(editor) => Some(editor.as_ref()),
            _ => None,
        };
        NativeRuntimeInfo {
            document_id: self.id,
            revision: self.session().revision(),
            font_generation: self.session().font_generation(),
            page_count: self.session().document().page_count(),
            dirty: self.session().is_dirty(),
            editing: editor.is_some(),
            selection: editor.map(|e| e.selection().clone()),
            composition_id: editor.and_then(NativeEditor::composition_id),
            can_undo: editor.is_some_and(NativeEditor::can_undo),
            can_redo: editor.is_some_and(NativeEditor::can_redo),
        }
    }
    fn cancel_save(&mut self, token: NativeSaveToken) -> Result<NativeSaveStatus, NativeHostError> {
        match &mut self.owner {
            DocumentOwner::Viewer(session) => session.cancel_save(token),
            DocumentOwner::Editor(editor) => editor.cancel_save(token),
        }
    }
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}
fn document(owner: &mut Option<OwnedDocument>) -> Result<&mut OwnedDocument, NativeHostError> {
    owner.as_mut().ok_or_else(|| invalid("no document is open"))
}
fn replacement(
    owner: &Option<OwnedDocument>,
    target: Option<NativeRuntimeTarget>,
) -> Result<(), NativeHostError> {
    match (owner, target) {
        (None, None) => Ok(()),
        (Some(owner), Some(target)) => owner.check(target),
        _ => Err(invalid(
            "replacement must name the current document revision",
        )),
    }
}

fn compact_clipboard_payload(payload: NativeClipboardPayload) -> NativeClipboardPayload {
    // NativeEditor checks the combined UTF-8 length before cut mutates the
    // document. Remove allocator slack so the unread output fits its reservation.
    NativeClipboardPayload {
        plain_text: payload.plain_text.into_boxed_str().into_string(),
        html: payload.html.map(|html| html.into_boxed_str().into_string()),
    }
}

fn execute(
    owner: &mut Option<OwnedDocument>,
    id: NativeRequestId,
    command: NativeRuntimeCommand,
) -> Result<NativeRuntimeOutput, NativeHostError> {
    use NativeRuntimeCommand as C;
    use NativeRuntimeOutput as O;
    match command {
        C::New { replace } => {
            replacement(owner, replace)?;
            let session = NativeDocumentSession::new_blank()?;
            *owner = Some(OwnedDocument {
                id,
                owner: DocumentOwner::Viewer(Box::new(session)),
            });
            Ok(O::Info(document(owner)?.info()))
        }
        C::Open {
            replace,
            bytes,
            password,
        } => {
            replacement(owner, replace)?;
            let session = NativeDocumentSession::open_bytes(&bytes, password.as_deref())?;
            *owner = Some(OwnedDocument {
                id,
                owner: DocumentOwner::Viewer(Box::new(session)),
            });
            Ok(O::Info(document(owner)?.info()))
        }
        C::Info {} => Ok(O::Info(document(owner)?.info())),
        C::ExecuteCommand { target, command } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .execute_command(target.revision, command)
                .map(O::CommandExecuted)
        }
        C::ImeContext { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .ime_context(target.revision)
                .map(O::ImeContext)
        }
        C::CopySelection { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let payload = doc.editor()?.copy_selection(target.revision)?;
            Ok(O::Clipboard {
                payload: compact_clipboard_payload(payload),
                outcome: None,
            })
        }
        C::CutSelection { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let (payload, outcome) = doc.editor()?.cut_selection(target.revision)?;
            Ok(O::Clipboard {
                payload: compact_clipboard_payload(payload),
                outcome: Some(outcome),
            })
        }
        C::PasteText { target, text } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .paste_text(target.revision, &text)
                .map(O::Edited)
        }
        C::PasteHtml { target, html } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .paste_html(target.revision, &html)
                .map(O::Edited)
        }
        C::PageSize { target, page } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.session()
                .page_size(page)
                .map(|size| O::PageSize { target, page, size })
        }
        C::CaretRect { target, position } => {
            let doc = document(owner)?;
            doc.check(target)?;
            Ok(O::CaretRect {
                target,
                caret: Some(doc.session().document().native_caret_rect(&position)?),
            })
        }
        C::HitTest { target, page, x, y } => {
            let doc = document(owner)?;
            doc.check(target)?;
            Ok(O::HitTest {
                target,
                hit: doc.session().document().native_hit_test(page, x, y)?,
            })
        }
        C::SelectionRects { target, selection } => {
            let doc = document(owner)?;
            doc.check(target)?;
            Ok(O::SelectionRects {
                target,
                rects: doc
                    .session()
                    .document()
                    .native_selection_rects(&selection)?,
            })
        }
        C::CloseDocument { target } => {
            document(owner)?.check(target)?;
            *owner = None;
            Ok(O::DocumentClosed)
        }
        C::Accessibility { target, request } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let selection = match &doc.owner {
                DocumentOwner::Editor(editor) => Some(editor.selection()),
                DocumentOwner::Viewer(_) => None,
            };
            Ok(O::Accessibility {
                target,
                snapshot: doc.session().accessibility_snapshot(&request, selection)?,
            })
        }
        C::RenderRegion {
            target,
            font_generation,
            page,
            region,
            scale,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            if doc.session().font_generation() != font_generation {
                return Err(invalid("stale font generation"));
            }
            let frame = doc.session().render_region(
                page,
                RasterRegion {
                    x: region.x,
                    y: region.y,
                    width: region.width,
                    height: region.height,
                },
                scale,
            )?;
            Ok(O::Frame {
                document_id: doc.id,
                frame,
            })
        }
        C::BeginEdit { target, selection } => {
            document(owner)?.check(target)?;
            if matches!(&document(owner)?.owner, DocumentOwner::Editor(_)) {
                return Err(invalid("editor already active"));
            }
            let doc = owner.take().expect("validated document");
            let DocumentOwner::Viewer(session) = doc.owner else {
                unreachable!("validated viewer");
            };
            match NativeEditor::try_new(*session, selection) {
                Ok(editor) => {
                    *owner = Some(OwnedDocument {
                        id: doc.id,
                        owner: DocumentOwner::Editor(Box::new(editor)),
                    });
                    Ok(O::Info(document(owner)?.info()))
                }
                Err(failed) => {
                    let (session, error) = *failed;
                    *owner = Some(OwnedDocument {
                        id: doc.id,
                        owner: DocumentOwner::Viewer(Box::new(session)),
                    });
                    Err(error)
                }
            }
        }
        C::SetSelection { target, selection } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .set_selection(target.revision, selection)
                .map(O::Edited)
        }
        C::Replace { target, text } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .replace_selection(target.revision, &text)
                .map(O::Edited)
        }
        C::BeginComposition { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .begin_composition(target.revision)
                .map(O::Edited)
        }
        C::UpdateComposition {
            target,
            composition_id,
            text,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .update_composition(target.revision, composition_id, &text)
                .map(O::Edited)
        }
        C::CommitComposition {
            target,
            composition_id,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .commit_composition(target.revision, composition_id)
                .map(O::Edited)
        }
        C::CancelComposition {
            target,
            composition_id,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .cancel_composition(target.revision, composition_id)
                .map(O::Edited)
        }
        C::Undo { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?.undo(target.revision).map(O::Edited)
        }
        C::Redo { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?.redo(target.revision).map(O::Edited)
        }
        C::PrepareSave { target, format } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let prepared = match &mut doc.owner {
                DocumentOwner::Viewer(session) => session.prepare_save(format),
                DocumentOwner::Editor(editor) => editor.prepare_save(format),
            }?;
            if prepared.bytes().len() > MAX_DOCUMENT_BYTES {
                let _ = doc.cancel_save(prepared.token());
                return Err(NativeHostError::LimitExceeded(
                    "runtime save exceeds its reserved byte limit".into(),
                ));
            }
            Ok(O::PreparedSave {
                document_id: doc.id,
                prepared: NativeRuntimePreparedSave::from_prepared(prepared),
            })
        }
        C::AcknowledgeSave { document_id, token } => {
            let doc = document(owner)?;
            doc.check_id(document_id)?;
            let status = match &mut doc.owner {
                DocumentOwner::Viewer(session) => session.acknowledge_save(token),
                DocumentOwner::Editor(editor) => editor.acknowledge_save(token),
            }?;
            Ok(O::SaveStatus {
                document_id,
                status,
            })
        }
        C::CancelSave { document_id, token } => {
            let doc = document(owner)?;
            doc.check_id(document_id)?;
            doc.cancel_save(token).map(|status| O::SaveStatus {
                document_id,
                status,
            })
        }
    }
}

/// Snapshot nodes contain variable text, keys and nested structural addresses;
/// counting only the outer node vector would leave queued completions uncharged.
fn accessibility_retained_bytes(snapshot: &NativeAccessibilitySnapshot) -> usize {
    let position_bytes = |position: &NativeTextPosition| {
        position
            .cell_path
            .capacity()
            .saturating_mul(std::mem::size_of::<
                crate::document_core::native_interaction::NativeCellPathEntry,
            >())
    };
    let mut bytes = std::mem::size_of::<NativeAccessibilitySnapshot>()
        .saturating_add(
            snapshot
                .nodes
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    crate::native_host::accessibility::NativeAccessibilityNode,
                >()),
        )
        .saturating_add(
            snapshot
                .limitations
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    crate::native_host::accessibility::NativeAccessibilityLimitation,
                >()),
        );
    for node in &snapshot.nodes {
        bytes = bytes
            .saturating_add(node.key.capacity())
            .saturating_add(node.text.capacity())
            .saturating_add(position_bytes(&node.start))
            .saturating_add(position_bytes(&node.end));
    }
    if let Some(selection) = &snapshot.selection {
        bytes = bytes
            .saturating_add(position_bytes(&selection.anchor))
            .saturating_add(position_bytes(&selection.focus));
    }
    if let Some(geometry) = &snapshot.selection_geometry {
        bytes = bytes.saturating_add(
            geometry
                .rectangles
                .capacity()
                .saturating_mul(std::mem::size_of::<NativeSelectionRect>()),
        );
    }
    bytes
}

fn discard_prepared(
    owner: &mut Option<OwnedDocument>,
    result: &Result<NativeRuntimeOutput, NativeRuntimeFailure>,
) {
    if let Ok(NativeRuntimeOutput::PreparedSave {
        document_id,
        prepared,
    }) = result
    {
        if let Some(doc) = owner.as_mut().filter(|doc| doc.id == *document_id) {
            let _ = doc.cancel_save(prepared.token);
        }
    }
}

// Return false only for an impossible full completion queue. Disconnect/close
// is normal; neither path blocks or leaks an undisclosed save checkpoint.
pub(super) fn worker_loop(
    requests: Receiver<Request>,
    results: SyncSender<NativeRuntimeCompletion>,
    shared: &Shared,
) -> bool {
    let mut owner = None; // All !Send document/renderer state is born here.
    while let Ok(request) = requests.recv() {
        if shared.lifecycle.load(Ordering::Acquire) != RUNNING {
            break;
        }
        let render = request.command.is_render();
        let mut result = if request
            .phase
            .compare_exchange(QUEUED, CLAIMED, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            Ok(NativeRuntimeOutput::Cancelled)
        } else if render && shared.superseded(request.id) {
            Ok(NativeRuntimeOutput::Skipped(
                NativeRuntimeSkipReason::SupersededRender,
            ))
        } else {
            execute(&mut owner, request.id, request.command).map_err(NativeRuntimeFailure::from)
        };
        shared.set_current(owner.as_ref().map(OwnedDocument::stamp));
        let retained = match &result {
            Ok(NativeRuntimeOutput::Frame { frame, .. }) => frame.pixels.bytes.capacity(),
            Ok(NativeRuntimeOutput::PreparedSave { prepared, .. }) => prepared.bytes.len(),
            Ok(NativeRuntimeOutput::SelectionRects { rects, .. }) => rects
                .capacity()
                .checked_mul(std::mem::size_of::<NativeSelectionRect>())
                .unwrap_or(usize::MAX),
            Ok(NativeRuntimeOutput::Accessibility { snapshot, .. }) => {
                accessibility_retained_bytes(snapshot)
            }
            _ => 0,
        };
        if retained > request.reservation {
            discard_prepared(&mut owner, &result);
            result = Err(NativeHostError::LimitExceeded(
                "runtime output exceeds its reserved byte limit".into(),
            )
            .into());
        } else if render
            && shared.superseded(request.id)
            && !matches!(&result, Ok(NativeRuntimeOutput::Cancelled))
        {
            result = Ok(NativeRuntimeOutput::Skipped(
                NativeRuntimeSkipReason::SupersededRender,
            ));
        }
        request.phase.store(COMPLETED, Ordering::Release);
        if shared.lifecycle.load(Ordering::Acquire) != RUNNING {
            discard_prepared(&mut owner, &result);
            break;
        }
        let completion = NativeRuntimeCompletion {
            request_id: request.id,
            result,
        };
        match results.try_send(completion) {
            Ok(()) => (),
            Err(TrySendError::Disconnected(completion)) => {
                discard_prepared(&mut owner, &completion.result);
                break;
            }
            Err(TrySendError::Full(completion)) => {
                discard_prepared(&mut owner, &completion.result);
                return false;
            }
        }
    }
    true
}
