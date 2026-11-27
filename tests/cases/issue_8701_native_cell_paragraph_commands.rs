#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{
    NativeCellPathEntry, NativeTextPosition, NativeTextSelection,
};
use rhwp::model::{control::Control, paragraph::OrphanFieldEnd};
use rhwp::native_host::commands::NativeEditCommand;
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::DocumentCore;

const FORM: &[u8] =
    include_bytes!("../../samples/issue5599_oracle/3191107_leave_request_form.hwpx");

fn selection(paragraph: u32, anchor: u32, focus: u32) -> NativeTextSelection {
    let point = NativeTextPosition {
        section: 0,
        paragraph: 1,
        scalar_offset: anchor,
        cell_path: vec![
            NativeCellPathEntry {
                control: 0,
                cell: 1,
                paragraph: 1,
            },
            NativeCellPathEntry {
                control: 0,
                cell: 5,
                paragraph,
            },
        ],
    };
    let mut end = point.clone();
    end.scalar_offset = focus;
    NativeTextSelection {
        anchor: point,
        focus: end,
    }
}

#[test]
fn issue_8701_nested_cell_paragraph_commands_publish_one_undo_entry_each() {
    let core = DocumentCore::from_bytes(FORM).unwrap();
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), selection(0, 0, 0)).unwrap();
    let initial = editor.replace_selection(0, "가😀나다").unwrap().revision;
    let range = selection(0, 3, 1);
    editor.set_selection(initial, range.clone()).unwrap();
    let split = editor
        .execute_command(initial, NativeEditCommand::SplitParagraph {})
        .unwrap()
        .outcome;
    assert_eq!(split.revision, initial + 1);
    assert_eq!(split.selection, selection(1, 0, 0));
    assert!(split.document_changed && split.history_changed);
    assert_eq!(editor.ime_context(split.revision).unwrap().text, "다");

    let undone = editor.undo(split.revision).unwrap();
    assert_eq!(undone.selection, range);
    assert_eq!(
        editor.ime_context(undone.revision).unwrap().text,
        "가😀나다"
    );
    let redone = editor.redo(undone.revision).unwrap();
    assert_eq!(redone.selection, split.selection);

    let join = NativeEditCommand::JoinPreviousParagraph {};
    let joined = editor
        .execute_command(redone.revision, join.clone())
        .unwrap()
        .outcome;
    assert_eq!(joined.revision, redone.revision + 1);
    assert_eq!(joined.selection, selection(0, 1, 1));
    assert!(joined.document_changed && joined.history_changed);
    assert_eq!(editor.ime_context(joined.revision).unwrap().text, "가다");
    let boundary = editor
        .execute_command(joined.revision, join)
        .unwrap()
        .outcome;
    assert_eq!(boundary.revision, joined.revision);
    assert_eq!(boundary.selection, joined.selection);
    assert!(!boundary.document_changed && !boundary.history_changed);
    let undone_join = editor.undo(joined.revision).unwrap();
    assert_eq!(undone_join.selection, split.selection);
    assert_eq!(editor.ime_context(undone_join.revision).unwrap().text, "다");

    editor.begin_composition(undone_join.revision).unwrap();
    assert!(matches!(
        editor.execute_command(undone_join.revision, NativeEditCommand::SplitParagraph {}),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert_eq!(editor.selection(), &undone_join.selection);
}

#[test]
fn issue_8701_nested_cell_join_checks_the_preceding_paragraph_fields() {
    let mut core = DocumentCore::from_bytes(FORM).unwrap();
    let mut document = core.document().clone();
    let Control::Table(parent) = &mut document.sections[0].paragraphs[1].controls[0] else {
        panic!("expected outer table");
    };
    let Control::Table(table) = &mut parent.cells[1].paragraphs[1].controls[0] else {
        panic!("expected nested table");
    };
    let paragraphs = &mut table.cells[5].paragraphs;
    let original = paragraphs[0].clone();
    paragraphs.push(original);
    paragraphs[0].orphan_field_ends.push(OrphanFieldEnd {
        char_idx: 0,
        field_id: 7,
        ..Default::default()
    });
    core.set_document(document);
    let start = selection(1, 0, 0);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), start.clone()).unwrap();
    assert!(matches!(
        editor.execute_command(0, NativeEditCommand::JoinPreviousParagraph {}),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(editor.selection(), &start);
    assert_eq!(editor.session().revision(), 0);
    assert!(!editor.can_undo());
}
