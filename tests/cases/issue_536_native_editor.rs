//! Native editor state/undo contracts; these tests do not assert visual parity.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{
    NativeCellPathEntry, NativeTextPosition, NativeTextSelection,
};
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::native_host::commands::{NativeEditCommand, NativeParagraphAlignment};
use rhwp::native_host::editor::{NativeCompositionFocusLoss, NativeEditor};
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::DocumentCore;

fn position(paragraph: u32, scalar_offset: u32) -> NativeTextPosition {
    NativeTextPosition {
        section: 0,
        paragraph,
        scalar_offset,
        cell_path: Vec::new(),
    }
}

fn selection(anchor: u32, focus: u32) -> NativeTextSelection {
    NativeTextSelection {
        anchor: position(0, anchor),
        focus: position(0, focus),
    }
}

fn core_with_text(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    core
}

fn editor(text: &str) -> NativeEditor {
    NativeEditor::new(
        NativeDocumentSession::from_core(core_with_text(text)),
        selection(0, 0),
    )
    .unwrap()
}

fn text(editor: &NativeEditor) -> &str {
    &editor.session().document().document().sections[0].paragraphs[0].text
}

fn revision(editor: &NativeEditor) -> u64 {
    editor.session().revision()
}

fn cell_editor(protected: bool) -> (NativeEditor, NativeTextSelection) {
    let mut core = core_with_text("");
    core.paste_html_native(
        0,
        0,
        0,
        "<table><tr><td><p>가😀나</p><p>다음 문단</p></td><td><p>이웃 칸</p></td></tr></table>",
    )
    .unwrap();
    let mut document = core.document().clone();
    let (paragraph, control) = document.sections[0]
        .paragraphs
        .iter_mut()
        .enumerate()
        .find_map(|(paragraph, host)| {
            host.controls
                .iter_mut()
                .enumerate()
                .find_map(|(control, item)| match item {
                    Control::Table(table) => {
                        table.cells[0].set_cell_protect(protected);
                        Some((paragraph as u32, control as u32))
                    }
                    _ => None,
                })
        })
        .expect("fixture table");
    core.set_document(document);
    let anchor = NativeTextPosition {
        section: 0,
        paragraph,
        scalar_offset: 2,
        cell_path: vec![NativeCellPathEntry {
            control,
            cell: 0,
            paragraph: 0,
        }],
    };
    let mut focus = anchor.clone();
    focus.scalar_offset = 1;
    let selection = NativeTextSelection { anchor, focus };
    let editor = NativeEditor::new(NativeDocumentSession::from_core(core), selection.clone())
        .expect("cell selection is available");
    (editor, selection)
}

fn table_at<'a>(editor: &'a NativeEditor, position: &NativeTextPosition) -> &'a Table {
    let section = &editor.session().document().document().sections[position.section as usize];
    let paragraph = &section.paragraphs[position.paragraph as usize];
    let Control::Table(table) = &paragraph.controls[position.cell_path[0].control as usize] else {
        panic!("fixture table moved");
    };
    table
}

#[test]
fn issue_536_native_editor_replaces_scalar_selection_and_restores_it() {
    let mut editor = editor("가😀나다");
    // Reversed anchor/focus and a non-BMP character use scalar indices.
    editor.set_selection(0, selection(3, 1)).unwrap();
    let changed = editor.replace_selection(0, "한글").unwrap();
    assert_eq!(text(&editor), "가한글다");
    assert_eq!(changed.selection, selection(3, 3));
    assert!(changed.document_changed && changed.history_changed);
    editor.undo(changed.revision).unwrap();
    assert_eq!(text(&editor), "가😀나다");
    assert_eq!(editor.selection(), &selection(3, 1));
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "가한글다");
}

#[test]
fn issue_536_native_editor_replaces_one_cell_paragraph_and_restores_it() {
    let (mut editor, original) = cell_editor(false);
    let changed = editor.replace_selection(0, "한글").unwrap();
    let mut caret = original.focus.clone();
    caret.scalar_offset = 3;
    assert_eq!(changed.selection.anchor, caret);
    assert_eq!(changed.selection.focus, caret);
    assert!(changed.document_changed && changed.history_changed);
    let table = table_at(&editor, &caret);
    assert_eq!(table.cells[0].paragraphs[0].text, "가한글나");
    assert_eq!(table.cells[0].paragraphs[1].text, "다음 문단");
    assert_eq!(table.cells[1].paragraphs[0].text, "이웃 칸");

    assert!(matches!(
        editor.replace_selection(0, "늦은 입력"),
        Err(NativeHostError::StaleRevision { .. })
    ));
    editor.undo(changed.revision).unwrap();
    assert_eq!(
        table_at(&editor, &caret).cells[0].paragraphs[0].text,
        "가😀나"
    );
    assert_eq!(editor.selection(), &original);
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(
        table_at(&editor, &caret).cells[0].paragraphs[0].text,
        "가한글나"
    );

    editor.replace_selection(revision(&editor), "😀").unwrap();
    assert_eq!(
        table_at(&editor, &caret).cells[0].paragraphs[0].text,
        "가한글😀나"
    );
    let mut after_insert = caret.clone();
    after_insert.scalar_offset += 1;
    editor
        .set_selection(
            revision(&editor),
            NativeTextSelection {
                anchor: caret.clone(),
                focus: after_insert,
            },
        )
        .unwrap();
    editor.replace_selection(revision(&editor), "").unwrap();
    assert_eq!(
        table_at(&editor, &caret).cells[0].paragraphs[0].text,
        "가한글나"
    );
}

#[test]
fn issue_536_native_editor_keeps_body_commands_and_protected_cells_read_only() {
    let (mut editor, selection) = cell_editor(true);
    let mut sibling = selection.focus.clone();
    sibling.cell_path[0].cell = 1;
    assert!(matches!(
        editor.set_selection(
            0,
            NativeTextSelection {
                anchor: selection.anchor.clone(),
                focus: sibling
            }
        ),
        Err(NativeHostError::Unsupported(_))
    ));
    let mut next_paragraph = selection.focus.clone();
    next_paragraph.cell_path[0].paragraph = 1;
    assert!(matches!(
        editor.set_selection(
            0,
            NativeTextSelection {
                anchor: selection.anchor.clone(),
                focus: next_paragraph
            }
        ),
        Err(NativeHostError::Unsupported(_))
    ));
    let mut invalid = selection.clone();
    invalid.anchor.scalar_offset = 4;
    assert!(matches!(
        editor.set_selection(0, invalid),
        Err(NativeHostError::InvalidArgument(_))
    ));
    let mut caption = selection.focus.clone();
    caption.cell_path[0].cell = 65_534;
    assert!(matches!(
        editor.set_selection(
            0,
            NativeTextSelection {
                anchor: caption.clone(),
                focus: caption,
            }
        ),
        Err(NativeHostError::Unsupported(_))
    ));

    for result in [
        editor.replace_selection(0, "한"),
        editor
            .execute_command(0, NativeEditCommand::SplitParagraph {})
            .map(|result| result.outcome),
        editor.begin_composition(0),
        editor.paste_text(0, "새\n내용"),
        editor.paste_html(0, "<p>새 내용</p>"),
    ] {
        assert!(matches!(result, Err(NativeHostError::Unsupported(_))));
    }
    let copied = editor.copy_selection(0).unwrap();
    assert_eq!(copied.plain_text, "😀");
    assert_eq!(copied.html, None);
    assert!(matches!(
        editor.ime_context(0),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(editor.selection(), &selection);
    assert_eq!(revision(&editor), 0);
    assert!(!editor.can_undo() && !editor.can_redo());
    let table = table_at(&editor, &selection.focus);
    assert!(table.cells[0].cell_protect());
    assert_eq!(table.cells[0].paragraphs[0].text, "가😀나");
    assert_eq!(table.cells[1].paragraphs[0].text, "이웃 칸");
}

#[test]
fn issue_536_native_editor_rejects_writes_through_a_protected_nested_cell_ancestor() {
    let bytes = include_bytes!("../../samples/issue5599_oracle/3191107_leave_request_form.hwpx");
    let point = NativeTextPosition {
        section: 0,
        paragraph: 1,
        scalar_offset: 0,
        cell_path: vec![
            NativeCellPathEntry {
                control: 0,
                cell: 1,
                paragraph: 1,
            },
            NativeCellPathEntry {
                control: 0,
                cell: 5,
                paragraph: 0,
            },
        ],
    };
    let selection = NativeTextSelection {
        anchor: point.clone(),
        focus: point.clone(),
    };

    for lock_outer_table in [false, true] {
        let mut core = DocumentCore::from_bytes(bytes).unwrap();
        let mut document = core.document().clone();
        let Control::Table(outer) = &mut document.sections[0].paragraphs[1].controls[0] else {
            panic!("missing outer table");
        };
        if lock_outer_table {
            outer.common.locked = true;
        } else {
            outer.cells[1].set_cell_protect(true);
        }
        let Control::Table(inner) = &outer.cells[1].paragraphs[1].controls[0] else {
            panic!("missing applicant table");
        };
        assert!(!inner.common.locked);
        assert!(!inner.cells[5].cell_protect());
        assert!(inner.cells[5].paragraphs[0].text.is_empty());
        core.set_document(document);

        let mut editor =
            NativeEditor::new(NativeDocumentSession::from_core(core), selection.clone())
                .expect("a protected cell permits read-only selection");
        assert!(matches!(
            editor.replace_selection(0, "1991.07.23"),
            Err(NativeHostError::Unsupported(_))
        ));
        assert!(matches!(
            editor.execute_command(
                0,
                NativeEditCommand::SetParagraphAlignment {
                    alignment: NativeParagraphAlignment::Right,
                },
            ),
            Err(NativeHostError::Unsupported(_))
        ));
        assert!(matches!(
            editor.begin_composition(0),
            Err(NativeHostError::Unsupported(_))
        ));
        assert_eq!(editor.selection(), &selection);
        assert_eq!(revision(&editor), 0);
        assert!(!editor.can_undo() && !editor.can_redo());
        let outer = table_at(&editor, &point);
        let Control::Table(inner) = &outer.cells[1].paragraphs[1].controls[0] else {
            panic!("missing applicant table");
        };
        assert!(inner.cells[5].paragraphs[0].text.is_empty());
    }
}

#[test]
fn issue_536_native_editor_ime_commits_one_undo_without_reinserting() {
    let mut editor = editor("");
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    for preedit in ["ㄱ", "가", "각", "각😀"] {
        let result = editor
            .update_composition(revision(&editor), id, preedit)
            .unwrap();
        assert!(result.document_changed);
        assert!(!result.history_changed);
        assert!(!editor.can_undo());
        assert_eq!(text(&editor), preedit);
    }
    let final_revision = revision(&editor);
    let committed = editor.commit_composition(final_revision, id).unwrap();
    assert_eq!(committed.revision, final_revision);
    assert!(!committed.document_changed);
    assert!(committed.history_changed);
    assert_eq!(text(&editor), "각😀");
    editor.undo(final_revision).unwrap();
    assert_eq!(text(&editor), "");
    assert!(!editor.can_undo());
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "각😀");
}

#[test]
fn issue_536_native_editor_cancel_restores_selection_and_formatting() {
    let mut core = core_with_text("가😀나다");
    core.apply_char_format_native(0, 0, 1, 3, r#"{"bold":true}"#)
        .unwrap();
    let before = serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap();
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), selection(1, 3)).unwrap();
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "한").unwrap();
    assert_eq!(text(&editor), "가한다");
    let cancelled = editor.cancel_composition(revision(&editor), id).unwrap();
    assert!(cancelled.document_changed);
    assert!(!cancelled.history_changed);
    assert_eq!(editor.selection(), &selection(1, 3));
    let restored =
        serde_json::to_value(&editor.session().document().document().sections[0].paragraphs[0])
            .unwrap();
    assert_eq!(
        restored, before,
        "cancel must restore metadata, not merely reinsert text"
    );
    assert!(!editor.can_undo());
}

#[test]
fn issue_536_native_editor_focus_loss_requires_explicit_commit_or_cancel() {
    let mut editor = editor("");
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "가").unwrap();
    editor
        .focus_lost(revision(&editor), NativeCompositionFocusLoss::Cancel)
        .unwrap();
    assert_eq!(text(&editor), "");
    assert!(!editor.can_undo());
    let id = editor
        .begin_composition(revision(&editor))
        .unwrap()
        .composition_id
        .unwrap();
    editor
        .update_composition(revision(&editor), id, "나")
        .unwrap();
    editor
        .focus_lost(revision(&editor), NativeCompositionFocusLoss::Commit)
        .unwrap();
    assert_eq!(text(&editor), "나");
    assert!(editor.can_undo());
}

#[test]
fn issue_536_native_editor_noop_and_cancel_preserve_redo() {
    let mut editor = editor("가");
    editor.replace_selection(0, "나").unwrap();
    editor.undo(revision(&editor)).unwrap();
    assert!(editor.can_redo());
    let before = revision(&editor);
    let noop = editor.replace_selection(before, "").unwrap();
    assert_eq!(noop.revision, before);
    assert!(!noop.history_changed && !noop.document_changed);
    editor.set_selection(before, selection(0, 1)).unwrap();
    editor.replace_selection(before, "가").unwrap();
    assert!(
        editor.can_redo(),
        "identical selection replacement preserves redo"
    );
    let id = editor
        .begin_composition(before)
        .unwrap()
        .composition_id
        .unwrap();
    editor.update_composition(before, id, "ㄴ").unwrap();
    editor.cancel_composition(revision(&editor), id).unwrap();
    assert!(editor.can_redo());
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "나가");
    editor.undo(revision(&editor)).unwrap();
    editor.replace_selection(revision(&editor), "다").unwrap();
    assert!(
        !editor.can_redo(),
        "a committed new edit discards the old future"
    );
}

#[test]
fn issue_536_native_editor_ime_returning_to_original_does_not_record_history() {
    let mut editor = editor("가");
    editor.replace_selection(0, "나").unwrap();
    editor.undo(revision(&editor)).unwrap();
    editor
        .set_selection(revision(&editor), selection(0, 1))
        .unwrap();
    let id = editor
        .begin_composition(revision(&editor))
        .unwrap()
        .composition_id
        .unwrap();
    editor
        .update_composition(revision(&editor), id, "다")
        .unwrap();
    editor
        .update_composition(revision(&editor), id, "가")
        .unwrap();
    let result = editor.commit_composition(revision(&editor), id).unwrap();
    assert!(!result.document_changed && !result.history_changed);
    assert_eq!(text(&editor), "가");
    assert!(editor.can_redo());
}

#[test]
fn issue_536_native_editor_stale_revision_rejects_before_state_changes() {
    let mut editor = editor("");
    editor.replace_selection(0, "가").unwrap();
    let live_revision = revision(&editor);
    for result in [
        editor.set_selection(0, selection(0, 0)),
        editor.replace_selection(0, "나"),
        editor.begin_composition(0),
        editor.undo(0),
        editor.redo(0),
        editor.clear_history(0),
    ] {
        assert!(
            matches!(result, Err(NativeHostError::StaleRevision { expected: 0, actual }) if actual == live_revision)
        );
    }
    assert_eq!(text(&editor), "가");
    assert_eq!(editor.selection(), &selection(1, 1));
    assert!(editor.can_undo());
    let id = editor
        .begin_composition(live_revision)
        .unwrap()
        .composition_id
        .unwrap();
    assert!(editor.update_composition(0, id, "나").is_err());
    assert!(editor.commit_composition(0, id).is_err());
    assert!(editor.cancel_composition(0, id).is_err());
    assert_eq!(editor.composition_id(), Some(id));
    assert_eq!(text(&editor), "가");
}

#[test]
fn issue_536_native_editor_rejects_late_ime_events_without_revision_change() {
    let mut editor = editor("");
    let old = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.cancel_composition(0, old).unwrap();
    let current = editor.begin_composition(0).unwrap().composition_id.unwrap();
    assert_ne!(old, current);
    assert!(editor.update_composition(0, old, "가").is_err());
    assert!(editor.commit_composition(0, old).is_err());
    assert!(editor.cancel_composition(0, old).is_err());
    assert_eq!(editor.composition_id(), Some(current));
    assert_eq!(revision(&editor), 0);
    assert_eq!(text(&editor), "");
}

#[test]
fn issue_536_native_editor_rejects_invalid_ranges_cells_and_control_input() {
    let mut editor = editor("😀");
    assert!(matches!(
        editor.set_selection(0, selection(2, 2)),
        Err(NativeHostError::InvalidArgument(_))
    ));
    let mut cell = selection(0, 0);
    cell.anchor.cell_path.push(NativeCellPathEntry {
        control: 0,
        cell: 0,
        paragraph: 0,
    });
    assert!(matches!(
        editor.set_selection(0, cell),
        Err(NativeHostError::Unsupported(_))
    ));
    for input in ["\n", "\r", "\t", "\0"] {
        assert!(matches!(
            editor.replace_selection(0, input),
            Err(NativeHostError::Unsupported(_))
        ));
    }
    assert_eq!(revision(&editor), 0);
    assert_eq!(text(&editor), "😀");
    assert!(!editor.can_undo());
}

#[test]
fn issue_536_native_editor_cross_paragraph_replacement_is_one_action() {
    let mut core = core_with_text("가나다라마");
    core.split_paragraph_native(0, 0, 2, None).unwrap();
    let range = NativeTextSelection {
        anchor: position(0, 1),
        focus: position(1, 2),
    };
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), range.clone()).unwrap();
    editor.replace_selection(0, "😀").unwrap();
    assert_eq!(text(&editor), "가😀마");
    assert_eq!(
        editor.session().document().document().sections[0]
            .paragraphs
            .len(),
        1
    );
    editor.undo(revision(&editor)).unwrap();
    let document = editor.session().document().document();
    assert_eq!(document.sections[0].paragraphs[0].text, "가나");
    assert_eq!(document.sections[0].paragraphs[1].text, "다라마");
    assert_eq!(editor.selection(), &range);
}

#[test]
fn issue_536_native_editor_semantic_middle_paragraph_rejects_replacement_and_preedit() {
    let mut table = Table::default();
    table.common.treat_as_char = false;
    for control in [
        Control::Table(Box::new(table)),
        Control::Bookmark(Default::default()),
    ] {
        let mut core = core_with_text("ABC");
        core.split_paragraph_native(0, 0, 2, None).unwrap();
        core.split_paragraph_native(0, 0, 1, None).unwrap();
        let mut document = core.document().clone();
        document.sections[0].paragraphs[1].controls.push(control);
        core.set_document(document);
        let mut editor =
            NativeEditor::new(NativeDocumentSession::from_core(core), selection(0, 0)).unwrap();
        editor.replace_selection(0, "x").unwrap();
        editor.undo(revision(&editor)).unwrap();
        let live_revision = revision(&editor);
        let range = NativeTextSelection {
            anchor: position(2, 0),
            focus: position(0, 1),
        };
        editor.set_selection(live_revision, range.clone()).unwrap();
        let before = format!("{:?}", editor.session().document().document());
        for replacement in ["", "가😀"] {
            assert!(matches!(
                editor.replace_selection(live_revision, replacement),
                Err(NativeHostError::Unsupported(_))
            ));
            assert_eq!(
                format!("{:?}", editor.session().document().document()),
                before
            );
            assert_eq!(editor.selection(), &range);
            assert_eq!(revision(&editor), live_revision);
            assert!(editor.can_redo());
            assert!(!editor.can_undo());
        }
        let id = editor
            .begin_composition(live_revision)
            .unwrap()
            .composition_id
            .unwrap();
        assert!(matches!(
            editor.update_composition(live_revision, id, "가"),
            Err(NativeHostError::Unsupported(_))
        ));
        assert_eq!(
            format!("{:?}", editor.session().document().document()),
            before
        );
        assert_eq!(editor.selection(), &range);
        assert_eq!(revision(&editor), live_revision);
        assert_eq!(editor.composition_id(), Some(id));
        editor.cancel_composition(live_revision, id).unwrap();
        assert!(editor.can_redo());
        assert!(!editor.can_undo());
    }
}

#[test]
fn issue_536_native_editor_semantic_selection_and_identical_noops_preserve_redo() {
    let mut core = core_with_text("AB");
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    let mut document = core.document().clone();
    document.sections[0].paragraphs[1]
        .controls
        .push(Control::Bookmark(Default::default()));
    core.set_document(document);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), selection(0, 0)).unwrap();
    editor.replace_selection(0, "x").unwrap();
    editor.undo(revision(&editor)).unwrap();
    let live_revision = revision(&editor);
    let range = NativeTextSelection {
        anchor: position(1, 0),
        focus: position(1, 1),
    };
    editor.set_selection(live_revision, range.clone()).unwrap();
    let before = format!("{:?}", editor.session().document().document());
    let noop = editor.replace_selection(live_revision, "B").unwrap();
    assert!(!noop.document_changed && !noop.history_changed);
    assert_eq!(noop.selection.anchor, position(1, 1));
    assert_eq!(noop.selection.focus, position(1, 1));
    editor.set_selection(live_revision, range.clone()).unwrap();
    let id = editor
        .begin_composition(live_revision)
        .unwrap()
        .composition_id
        .unwrap();
    let preview = editor.update_composition(live_revision, id, "B").unwrap();
    assert!(!preview.document_changed && !preview.history_changed);
    let committed = editor.commit_composition(live_revision, id).unwrap();
    assert!(!committed.document_changed && !committed.history_changed);
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
    assert_eq!(revision(&editor), live_revision);
    assert!(editor.can_redo());
    assert!(!editor.can_undo());
}

#[test]
fn issue_536_native_editor_long_preedit_uses_normal_edit_path() {
    let mut editor = editor("");
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    let preedit = "가나다라마바사아자차😀";
    editor.update_composition(0, id, preedit).unwrap();
    assert_eq!(text(&editor), preedit);
    assert_eq!(editor.selection(), &selection(11, 11));
    editor.cancel_composition(revision(&editor), id).unwrap();
    assert_eq!(text(&editor), "");
}

#[test]
fn issue_536_native_editor_history_evicts_oldest_and_releases_resources() {
    let session = NativeDocumentSession::from_core(core_with_text(""));
    let mut editor = NativeEditor::with_history_capacity(session, selection(0, 0), 2).unwrap();
    for _ in 0..110 {
        editor.replace_selection(revision(&editor), "가").unwrap();
    }
    for _ in 0..2 {
        editor.undo(revision(&editor)).unwrap();
    }
    assert!(!editor.can_undo());
    assert_eq!(text(&editor).chars().count(), 108);
    for _ in 0..2 {
        editor.redo(revision(&editor)).unwrap();
    }
    assert_eq!(text(&editor).chars().count(), 110);
    for _ in 0..110 {
        editor.replace_selection(revision(&editor), "나").unwrap();
        editor.clear_history(revision(&editor)).unwrap();
    }
    assert!(!editor.can_undo());
    let selection = editor.selection().clone();
    let session = match editor.into_session() {
        Ok(session) => session,
        Err(_) => panic!("no active composition"),
    };
    // A leaking discard path would leave too few slots for this maximum capacity.
    assert!(NativeEditor::with_history_capacity(
        session,
        selection,
        (DocumentCore::MAX_SNAPSHOTS - 2) / 2
    )
    .is_ok());
}

#[test]
fn issue_536_native_editor_full_history_keeps_composition_and_undo_snapshots_live() {
    let session = NativeDocumentSession::from_core(core_with_text(""));
    let capacity = (DocumentCore::MAX_SNAPSHOTS - 2) / 2;
    let mut editor =
        NativeEditor::with_history_capacity(session, selection(0, 0), capacity).unwrap();
    for _ in 0..capacity {
        editor.replace_selection(revision(&editor), "가").unwrap();
    }
    let id = editor
        .begin_composition(revision(&editor))
        .unwrap()
        .composition_id
        .unwrap();
    for value in ["ㄴ", "나", "난"] {
        editor
            .update_composition(revision(&editor), id, value)
            .unwrap();
    }
    editor.cancel_composition(revision(&editor), id).unwrap();
    for _ in 0..capacity {
        editor.undo(revision(&editor)).unwrap();
    }
    assert_eq!(text(&editor), "");
    assert!(!editor.can_undo());
    for _ in 0..capacity {
        editor.redo(revision(&editor)).unwrap();
    }
    assert_eq!(text(&editor).chars().count(), capacity);
    let id = editor
        .begin_composition(revision(&editor))
        .unwrap()
        .composition_id
        .unwrap();
    editor
        .update_composition(revision(&editor), id, "나")
        .unwrap();
    editor.commit_composition(revision(&editor), id).unwrap();
    for _ in 0..capacity {
        editor.undo(revision(&editor)).unwrap();
    }
    assert_eq!(
        text(&editor),
        "가",
        "only the oldest committed entry is evicted"
    );
}

#[test]
fn issue_536_native_editor_preserves_preexisting_snapshot_reservation() {
    let mut core = core_with_text("");
    core.save_snapshot_native();
    let session = NativeDocumentSession::from_core(core);
    let available = (DocumentCore::MAX_SNAPSHOTS - 3) / 2;
    let mut editor =
        NativeEditor::with_history_capacity(session, selection(0, 0), available).unwrap();
    for _ in 0..available {
        editor.replace_selection(revision(&editor), "가").unwrap();
    }
    let selected = editor.selection().clone();
    let session = match editor.into_session() {
        Ok(session) => session,
        Err(_) => panic!("no active composition"),
    };
    assert!(matches!(
        NativeEditor::with_history_capacity(session, selected, available + 1),
        Err(NativeHostError::LimitExceeded(_))
    ));
}

#[test]
fn issue_536_native_editor_active_composition_prevents_history_and_session_transfer() {
    let mut editor = editor("");
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    assert!(editor.set_selection(0, selection(0, 0)).is_err());
    assert!(editor.replace_selection(0, "가").is_err());
    assert!(editor.begin_composition(0).is_err());
    assert!(editor.undo(0).is_err());
    assert!(editor.redo(0).is_err());
    assert!(editor.clear_history(0).is_err());
    let mut editor = match editor.into_session() {
        Err(editor) => *editor,
        Ok(_) => panic!("composition must be resolved explicitly"),
    };
    editor.cancel_composition(0, id).unwrap();
    assert!(editor.into_session().is_ok());
}

#[test]
fn issue_536_native_editor_does_not_treat_an_existing_batch_as_a_transaction() {
    let mut core = core_with_text("");
    core.begin_batch_native().unwrap();
    assert!(matches!(
        NativeEditor::new(NativeDocumentSession::from_core(core), selection(0, 0)),
        Err(NativeHostError::Unsupported(_))
    ));
}

#[test]
fn issue_536_native_editor_failed_begin_keeps_viewer_and_allows_retry() {
    let session = NativeDocumentSession::from_core(core_with_text("가😀"));
    let failed = match NativeEditor::try_new(session, selection(3, 3)) {
        Err(failed) => failed,
        Ok(_) => panic!("out-of-range selection must reject editing"),
    };
    let (session, error) = *failed;
    assert!(matches!(error, NativeHostError::InvalidArgument(_)));
    assert_eq!(session.revision(), 0);
    assert_eq!(
        session.document().document().sections[0].paragraphs[0].text,
        "가😀"
    );
    assert!(session.document().get_cursor_rect_native(0, 0, 0).is_ok());
    let mut editor = match NativeEditor::try_new(session, selection(2, 2)) {
        Ok(editor) => editor,
        Err(_) => panic!("the returned viewer session must remain usable"),
    };
    editor.replace_selection(0, "나").unwrap();
    assert_eq!(text(&editor), "가😀나");
}

#[test]
fn issue_536_native_editor_inline_first_paragraph_keeps_viewer_open() {
    use rhwp::model::control::Control;

    let mut core = core_with_text("");
    core.create_table_native(0, 0, 0, 1, 1).unwrap();
    let mut document = core.document().clone();
    let paragraph = document.sections[0]
        .paragraphs
        .iter_mut()
        .enumerate()
        .find_map(|(index, paragraph)| {
            paragraph
                .controls
                .iter_mut()
                .find_map(|control| match control {
                    Control::Table(table) => {
                        table.common.treat_as_char = true;
                        Some(index as u32)
                    }
                    _ => None,
                })
        })
        .unwrap();
    core.set_document(document);
    let before = serde_json::to_value(&core.document().sections[0].paragraphs).unwrap();
    let session = NativeDocumentSession::from_core(core);
    let at_object = NativeTextSelection {
        anchor: position(paragraph, 0),
        focus: position(paragraph, 0),
    };
    let (session, error) = match NativeEditor::try_new(session, at_object) {
        Err(failed) => *failed,
        Ok(_) => panic!("inline-object affinity is outside body-text editing"),
    };
    assert!(matches!(error, NativeHostError::Unsupported(_)));
    assert_eq!(session.revision(), 0);
    assert_eq!(
        serde_json::to_value(&session.document().document().sections[0].paragraphs).unwrap(),
        before
    );
    assert!(!session.document().get_document_info().is_empty());
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_536_native_editor_font_changes_preserve_history_and_reject_preedit() {
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut editor = editor("가");
    editor.replace_selection(0, "나").unwrap();
    let environment =
        FontEnvironment::from_json(r#"{"id":"native-editor-fonts","substitutions":{}}"#).unwrap();
    assert!(editor
        .set_font_sources(revision(&editor), &[], Some(environment.clone()))
        .unwrap());
    assert!(editor.can_undo());
    editor.undo(revision(&editor)).unwrap();
    assert_eq!(
        editor.session().document().font_environment(),
        Some(&environment)
    );
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(
        editor.session().document().font_environment(),
        Some(&environment)
    );
    let composition = editor
        .begin_composition(revision(&editor))
        .unwrap()
        .composition_id
        .unwrap();
    assert!(editor
        .set_font_sources(revision(&editor), &[], None)
        .is_err());
    editor
        .cancel_composition(revision(&editor), composition)
        .unwrap();
    assert_eq!(editor.session().font_generation(), 1);
}
