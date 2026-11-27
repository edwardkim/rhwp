#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{
    NativeCellPathEntry, NativeTextPosition, NativeTextSelection,
};
use rhwp::model::control::Control;
use rhwp::native_host::commands::{NativeEditCommand, NativeTableCellDirection};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::DocumentCore;

const FORM: &[u8] =
    include_bytes!("../../samples/issue5599_oracle/3191107_leave_request_form.hwpx");

fn selection(cell: u32, scalar_offset: u32) -> NativeTextSelection {
    let point = NativeTextPosition {
        section: 0,
        paragraph: 1,
        scalar_offset,
        cell_path: vec![
            NativeCellPathEntry {
                control: 0,
                cell: 1,
                paragraph: 1,
            },
            NativeCellPathEntry {
                control: 0,
                cell,
                paragraph: 0,
            },
        ],
    };
    NativeTextSelection {
        anchor: point.clone(),
        focus: point,
    }
}

#[test]
fn issue_8701_native_table_navigation_follows_anchors_inside_nested_table() {
    let mut core = DocumentCore::from_bytes(FORM).unwrap();
    let mut document = core.document().clone();
    let Control::Table(outer) = &mut document.sections[0].paragraphs[1].controls[0] else {
        panic!("expected outer table");
    };
    let Control::Table(table) = &mut outer.cells[1].paragraphs[1].controls[0] else {
        panic!("expected nested table");
    };
    assert_eq!(table.cells[1].col_span, 3);
    // Move two stored indices away from row-major position to prove that the
    // command uses each cell's (row, column) anchor rather than vector order.
    table.cells.swap(0, 5);
    table.cells[2].set_cell_protect(true);
    table.rebuild_grid();
    core.set_document(document);

    use NativeTableCellDirection::{Next, Previous};
    let command = |direction| NativeEditCommand::NavigateTableCell { direction };
    let start = selection(5, 1);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), start.clone()).unwrap();
    for (direction, expected) in [
        (Previous, start),
        (Next, selection(1, 0)),
        (Next, selection(3, 0)),
        (Next, selection(4, 0)),
        (Next, selection(0, 0)),
        (Next, selection(0, 0)),
        (Previous, selection(4, 0)),
        (Previous, selection(3, 0)),
        (Previous, selection(1, 0)),
        (Previous, selection(5, 0)),
    ] {
        let result = editor.execute_command(0, command(direction)).unwrap();
        assert_eq!(result.outcome.selection, expected);
        assert_eq!(result.outcome.revision, 0);
        assert_eq!(result.preferred_x, None);
        assert!(!result.outcome.document_changed && !result.outcome.history_changed);
    }
    assert!(!editor.can_undo() && !editor.can_redo());

    // A caret can be placed in a protected cell for read-only interaction.
    // Tab can leave it even though adjacent navigation skips it as a target.
    editor.set_selection(0, selection(2, 0)).unwrap();
    let escaped = editor.execute_command(0, command(Next)).unwrap().outcome;
    assert_eq!(escaped.selection, selection(3, 0));

    editor.begin_composition(0).unwrap();
    assert!(matches!(
        editor.execute_command(0, command(Next)),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert_eq!(editor.selection(), &escaped.selection);
}

#[test]
fn issue_8701_native_table_navigation_respects_a_protected_enclosing_cell() {
    let mut core = DocumentCore::from_bytes(FORM).unwrap();
    let mut document = core.document().clone();
    let Control::Table(table) = &mut document.sections[0].paragraphs[1].controls[0] else {
        panic!("expected outer table");
    };
    table.cells[1].set_cell_protect(true);
    core.set_document(document);
    let start = selection(0, 0);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), start.clone()).unwrap();
    assert!(matches!(
        editor.execute_command(
            0,
            NativeEditCommand::NavigateTableCell {
                direction: NativeTableCellDirection::Next
            },
        ),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(editor.selection(), &start);
    assert_eq!(editor.session().revision(), 0);
}
