//! Native command content/history contracts, not a claim of visual parity.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{
    NativeTextPosition, NativeTextSelection, NativeVerticalDirection,
};
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::native_host::commands::{NativeCommandOutcome, NativeEditCommand as Command};
use rhwp::native_host::editor::NativeEditor;
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

fn selected(anchor: NativeTextPosition, focus: NativeTextPosition) -> NativeTextSelection {
    NativeTextSelection { anchor, focus }
}

fn caret(paragraph: u32, offset: u32) -> NativeTextSelection {
    selected(position(paragraph, offset), position(paragraph, offset))
}

fn core(paragraphs: &[&str]) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for (index, text) in paragraphs.iter().enumerate() {
        if index > 0 {
            core.split_paragraph_native(0, index - 1, paragraphs[index - 1].chars().count(), None)
                .unwrap();
        }
        core.insert_text_native(0, index, 0, text).unwrap();
    }
    core
}

fn editor(paragraphs: &[&str]) -> NativeEditor {
    NativeEditor::new(
        NativeDocumentSession::from_core(core(paragraphs)),
        caret(0, 0),
    )
    .unwrap()
}

fn texts(editor: &NativeEditor) -> Vec<&str> {
    editor.session().document().document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect()
}

fn run(editor: &mut NativeEditor, command: Command) -> NativeCommandOutcome {
    editor
        .execute_command(editor.session().revision(), command)
        .unwrap()
}

fn select(editor: &mut NativeEditor, selection: NativeTextSelection) {
    editor
        .set_selection(editor.session().revision(), selection)
        .unwrap();
}

#[test]
fn native_commands_move_by_extended_graphemes_in_scalar_units() {
    let mut editor = editor(&["a\u{301}👩\u{200d}💻🇰🇷👍🏽z"]);
    // Scalar counts per cluster: 2, 3, 2, 2, 1; UTF-16 and byte offsets differ.
    for expected in [2, 5, 7, 9, 10] {
        let outcome = run(
            &mut editor,
            Command::MoveRight {
                extend_selection: false,
            },
        );
        assert_eq!(outcome.outcome.selection, caret(0, expected));
        assert!(!outcome.outcome.document_changed);
        assert_eq!(outcome.outcome.revision, 0);
    }
    for expected in [9, 7, 5, 2, 0] {
        assert_eq!(
            run(
                &mut editor,
                Command::MoveLeft {
                    extend_selection: false
                }
            )
            .outcome
            .selection,
            caret(0, expected)
        );
    }
    assert!(!editor.can_undo());
}

#[test]
fn native_commands_interior_scalar_caret_moves_to_grapheme_edges() {
    let mut editor = editor(&["a\u{301}👩\u{200d}💻z"]);
    select(&mut editor, caret(0, 3));
    assert_eq!(
        run(
            &mut editor,
            Command::MoveLeft {
                extend_selection: false
            }
        )
        .outcome
        .selection,
        caret(0, 2)
    );
    select(&mut editor, caret(0, 3));
    assert_eq!(
        run(
            &mut editor,
            Command::MoveRight {
                extend_selection: false
            }
        )
        .outcome
        .selection,
        caret(0, 5)
    );
}

#[test]
fn native_commands_extension_keeps_anchor_and_arrow_collapses_range() {
    let mut editor = editor(&["가😀나다"]);
    select(&mut editor, caret(0, 2));
    run(
        &mut editor,
        Command::MoveLeft {
            extend_selection: true,
        },
    );
    assert_eq!(
        editor.selection(),
        &selected(position(0, 2), position(0, 1))
    );
    run(
        &mut editor,
        Command::MoveRight {
            extend_selection: false,
        },
    );
    assert_eq!(editor.selection(), &caret(0, 2));
    run(
        &mut editor,
        Command::End {
            extend_selection: true,
        },
    );
    assert_eq!(
        editor.selection(),
        &selected(position(0, 2), position(0, 4))
    );
    run(
        &mut editor,
        Command::MoveLeft {
            extend_selection: false,
        },
    );
    assert_eq!(editor.selection(), &caret(0, 2));
    run(
        &mut editor,
        Command::Home {
            extend_selection: false,
        },
    );
    assert_eq!(editor.selection(), &caret(0, 0));
}

#[test]
fn native_commands_backspace_restores_cluster_and_original_caret_on_undo() {
    let mut editor = editor(&["a\u{301}b"]);
    select(&mut editor, caret(0, 2));
    let outcome = run(&mut editor, Command::DeleteBackward {}).outcome;
    assert_eq!(texts(&editor), ["b"]);
    assert_eq!(editor.selection(), &caret(0, 0));
    assert!(outcome.document_changed && outcome.history_changed);
    editor.undo(outcome.revision).unwrap();
    assert_eq!(texts(&editor), ["a\u{301}b"]);
    assert_eq!(editor.selection(), &caret(0, 2));
    assert!(!editor.can_undo());
    editor.redo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["b"]);
}

#[test]
fn native_commands_forward_delete_removes_whole_emoji_zwj_sequence() {
    let mut editor = editor(&["a👩\u{200d}💻b"]);
    select(&mut editor, caret(0, 1));
    run(&mut editor, Command::DeleteForward {});
    assert_eq!(texts(&editor), ["ab"]);
    assert_eq!(editor.selection(), &caret(0, 1));
    editor.undo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["a👩\u{200d}💻b"]);
}

#[test]
fn native_commands_interior_deletion_never_leaves_an_orphan_mark() {
    for command in [Command::DeleteBackward {}, Command::DeleteForward {}] {
        let mut editor = editor(&["a\u{301}z"]);
        select(&mut editor, caret(0, 1));
        run(&mut editor, command);
        assert_eq!(texts(&editor), ["z"]);
        editor.undo(editor.session().revision()).unwrap();
        assert_eq!(texts(&editor), ["a\u{301}z"]);
        assert_eq!(editor.selection(), &caret(0, 1));
    }
}

#[test]
fn native_commands_paragraph_edge_navigation_and_delete_join_once() {
    let mut editor = editor(&["가😀", "나다"]);
    select(&mut editor, caret(0, 2));
    run(
        &mut editor,
        Command::MoveRight {
            extend_selection: false,
        },
    );
    assert_eq!(editor.selection(), &caret(1, 0));
    run(
        &mut editor,
        Command::MoveLeft {
            extend_selection: false,
        },
    );
    assert_eq!(editor.selection(), &caret(0, 2));
    run(&mut editor, Command::DeleteForward {});
    assert_eq!(texts(&editor), ["가😀나다"]);
    assert_eq!(editor.selection(), &caret(0, 2));
    editor.undo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["가😀", "나다"]);
    assert!(!editor.can_undo());
    select(&mut editor, caret(1, 0));
    run(&mut editor, Command::DeleteBackward {});
    assert_eq!(texts(&editor), ["가😀나다"]);
    editor.undo(editor.session().revision()).unwrap();
    assert_eq!(editor.selection(), &caret(1, 0));
}

#[test]
fn native_commands_split_replaces_range_and_undo_restores_formatting() {
    {
        let mut editor = editor(&["가😀나"]);
        select(&mut editor, caret(0, 2));
        run(&mut editor, Command::SplitParagraph {});
        assert_eq!(texts(&editor), ["가😀", "나"]);
        assert_eq!(editor.selection(), &caret(1, 0));
        editor.undo(editor.session().revision()).unwrap();
        assert_eq!(texts(&editor), ["가😀나"]);
        assert_eq!(editor.selection(), &caret(0, 2));
    }
    let mut core = core(&["가😀나", "다라마"]);
    core.apply_char_format_native(0, 0, 1, 2, r#"{"bold":true}"#)
        .unwrap();
    let before = serde_json::to_value(&core.document().sections[0].paragraphs).unwrap();
    let selection = selected(position(0, 1), position(1, 2));
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), selection.clone()).unwrap();
    let outcome = run(&mut editor, Command::SplitParagraph {}).outcome;
    assert_eq!(texts(&editor), ["가", "마"]);
    assert_eq!(outcome.selection, caret(1, 0));
    assert_eq!(outcome.revision, 1);
    editor.undo(outcome.revision).unwrap();
    assert_eq!(editor.selection(), &selection);
    assert_eq!(
        serde_json::to_value(&editor.session().document().document().sections[0].paragraphs)
            .unwrap(),
        before
    );
    assert!(!editor.can_undo());
    editor.redo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["가", "마"]);
}

#[test]
fn native_commands_explicit_join_restores_empty_paragraph_on_undo() {
    let mut editor = editor(&["가", "", "나"]);
    select(&mut editor, caret(1, 0));
    run(&mut editor, Command::JoinPreviousParagraph {});
    assert_eq!(texts(&editor), ["가", "나"]);
    assert_eq!(editor.selection(), &caret(0, 1));
    editor.undo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["가", "", "나"]);
    assert_eq!(editor.selection(), &caret(1, 0));
    select(&mut editor, selected(position(0, 0), position(0, 1)));
    let revision = editor.session().revision();
    assert!(matches!(
        editor.execute_command(revision, Command::JoinPreviousParagraph {}),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert_eq!(texts(&editor), ["가", "", "나"]);
    assert!(editor.can_redo());
}

#[test]
fn native_commands_boundary_noops_and_movement_preserve_redo() {
    let mut editor = editor(&[""]);
    editor.replace_selection(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    for command in [
        Command::DeleteBackward {},
        Command::DeleteForward {},
        Command::JoinPreviousParagraph {},
        Command::MoveLeft {
            extend_selection: false,
        },
        Command::MoveRight {
            extend_selection: false,
        },
        Command::SelectAll {},
    ] {
        let outcome = run(&mut editor, command).outcome;
        assert_eq!(outcome.revision, revision);
        assert!(!outcome.document_changed && !outcome.history_changed);
        assert!(editor.can_redo());
    }
    editor.redo(revision).unwrap();
    assert_eq!(texts(&editor), ["x"]);
}

#[test]
fn native_commands_stale_revision_and_preedit_reject_without_mutation() {
    let mut editor = editor(&["가"]);
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "각").unwrap();
    let before = format!("{:?}", editor.session().document().document());
    let selection = editor.selection().clone();
    for command in [
        Command::SplitParagraph {},
        Command::DeleteForward {},
        Command::MoveRight {
            extend_selection: true,
        },
    ] {
        assert!(matches!(
            editor.execute_command(0, command.clone()),
            Err(NativeHostError::StaleRevision { .. })
        ));
        assert!(matches!(
            editor.execute_command(editor.session().revision(), command),
            Err(NativeHostError::InvalidArgument(_))
        ));
    }
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
    assert_eq!(editor.selection(), &selection);
    assert_eq!(editor.composition_id(), Some(id));
    editor
        .cancel_composition(editor.session().revision(), id)
        .unwrap();
    assert_eq!(texts(&editor), ["가"]);
}

#[test]
fn native_commands_select_all_is_limited_to_the_current_body_section() {
    let mut core = core(&["가😀", "나다"]);
    let mut document = core.document().clone();
    document.sections.push(document.sections[0].clone());
    core.set_document(document);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), caret(0, 1)).unwrap();
    run(&mut editor, Command::SelectAll {});
    assert_eq!(
        editor.selection(),
        &selected(position(0, 0), position(1, 2))
    );
    run(&mut editor, Command::DeleteForward {});
    assert_eq!(texts(&editor), [""]);
    assert_eq!(
        editor.session().document().document().sections[1].paragraphs[0].text,
        "가😀"
    );
    editor.undo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["가😀", "나다"]);
}

#[test]
fn native_commands_unsupported_adjacent_inline_target_preserves_document_and_history() {
    let mut core = core(&["가", "나"]);
    let mut document = core.document().clone();
    document.sections[0].paragraphs[1]
        .controls
        .push(Control::CharOverlap(Default::default()));
    core.set_document(document);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), caret(0, 1)).unwrap();
    editor.replace_selection(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let before = format!("{:?}", editor.session().document().document());
    for command in [
        Command::DeleteForward {},
        Command::MoveRight {
            extend_selection: false,
        },
        Command::SelectAll {},
    ] {
        assert!(matches!(
            editor.execute_command(revision, command),
            Err(NativeHostError::Unsupported(_))
        ));
        assert_eq!(editor.selection(), &caret(0, 1));
        assert_eq!(editor.session().revision(), revision);
        assert!(editor.can_redo());
    }
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
}

#[test]
fn native_commands_floating_middle_object_rejects_all_body_mutations() {
    let mut core = core(&["A", "B", "C"]);
    let mut document = core.document().clone();
    let mut table = Table::default();
    table.common.treat_as_char = false;
    document.sections[0].paragraphs[1]
        .controls
        .push(Control::Table(Box::new(table)));
    core.set_document(document);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), caret(0, 0)).unwrap();
    editor.replace_selection(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let range = selected(position(0, 1), position(2, 0));
    select(&mut editor, range.clone());
    let before = format!("{:?}", editor.session().document().document());
    for command in [
        Command::DeleteForward {},
        Command::DeleteBackward {},
        Command::SplitParagraph {},
    ] {
        assert!(matches!(
            editor.execute_command(revision, command),
            Err(NativeHostError::Unsupported(_))
        ));
        assert_eq!(
            format!("{:?}", editor.session().document().document()),
            before
        );
        assert_eq!(editor.selection(), &range);
        assert_eq!(editor.session().revision(), revision);
        assert!(editor.can_redo());
        assert!(!editor.can_undo());
    }
    assert!(matches!(
        editor.replace_selection(revision, "x"),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), revision);
    assert!(editor.can_redo());
    assert!(!editor.can_undo());

    let id = editor
        .begin_composition(revision)
        .unwrap()
        .composition_id
        .unwrap();
    assert!(matches!(
        editor.update_composition(revision, id, "가"),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), revision);
    assert_eq!(editor.composition_id(), Some(id));
    // History actions are unavailable while preedit owns the transaction.
    assert!(!editor.can_redo());
    assert!(!editor.can_undo());
    editor.cancel_composition(revision, id).unwrap();
    assert!(editor.can_redo());
    assert!(!editor.can_undo());
}

#[test]
fn native_commands_join_checks_previous_paragraph_outside_collapsed_selection() {
    let mut core = core(&["A", "B"]);
    let mut document = core.document().clone();
    let mut table = Table::default();
    table.common.treat_as_char = false;
    document.sections[0].paragraphs[0]
        .controls
        .push(Control::Table(Box::new(table)));
    core.set_document(document);
    let range = caret(1, 0);
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), range.clone()).unwrap();
    editor.replace_selection(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let before = format!("{:?}", editor.session().document().document());
    for command in [
        Command::JoinPreviousParagraph {},
        Command::DeleteBackward {},
    ] {
        assert!(matches!(
            editor.execute_command(revision, command),
            Err(NativeHostError::Unsupported(_))
        ));
        assert_eq!(
            format!("{:?}", editor.session().document().document()),
            before
        );
        assert_eq!(editor.selection(), &range);
        assert_eq!(editor.session().revision(), revision);
        assert!(editor.can_redo());
        assert!(!editor.can_undo());
    }
}

#[test]
fn native_commands_semantic_control_paragraph_allows_selection_and_identical_text() {
    let mut core = core(&["A"]);
    let mut document = core.document().clone();
    document.sections[0].paragraphs[0]
        .controls
        .push(Control::Bookmark(Default::default()));
    core.set_document(document);
    let range = selected(position(0, 0), position(0, 1));
    let mut editor = NativeEditor::new(NativeDocumentSession::from_core(core), range).unwrap();
    let before = format!("{:?}", editor.session().document().document());
    assert!(!editor.replace_selection(0, "A").unwrap().document_changed);
    assert_eq!(editor.selection(), &caret(0, 1));
    run(
        &mut editor,
        Command::MoveLeft {
            extend_selection: false,
        },
    );
    assert_eq!(editor.selection(), &caret(0, 0));
    assert!(matches!(
        editor.replace_selection(0, "B"),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
    assert_eq!(editor.session().revision(), 0);
    assert!(!editor.can_undo());
}

#[test]
fn native_commands_vertical_navigation_consumes_core_query_and_returns_preferred_column() {
    let mut editor = editor(&["abc", "def"]);
    let expected = editor
        .session()
        .document()
        .native_move_vertical(&position(0, 0), NativeVerticalDirection::Down, None)
        .unwrap();
    let outcome = run(
        &mut editor,
        Command::MoveDown {
            extend_selection: true,
            preferred_x: None,
        },
    );
    assert_eq!(
        outcome.outcome.selection,
        selected(position(0, 0), expected.position)
    );
    assert_eq!(outcome.preferred_x, Some(expected.preferred_x));
    let selection = editor.selection().clone();
    assert!(matches!(
        editor.execute_command(
            0,
            Command::MoveUp {
                extend_selection: false,
                preferred_x: Some(f64::NAN),
            }
        ),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert_eq!(editor.selection(), &selection);
    let up = run(
        &mut editor,
        Command::MoveUp {
            extend_selection: false,
            preferred_x: outcome.preferred_x,
        },
    );
    assert_eq!(up.outcome.selection.focus.paragraph, 0);
    assert_eq!(up.outcome.revision, 0);
    assert!(!editor.can_undo());
}

#[test]
fn native_commands_json_contract_uses_explicit_camel_case_fields() {
    let command = Command::MoveDown {
        extend_selection: true,
        preferred_x: Some(12.5),
    };
    let json = serde_json::to_value(&command).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"command":"moveDown","extendSelection":true,"preferredX":12.5})
    );
    assert_eq!(serde_json::from_value::<Command>(json).unwrap(), command);
    assert_eq!(
        serde_json::from_str::<Command>(r#"{"command":"moveLeft"}"#).unwrap(),
        Command::MoveLeft {
            extend_selection: false
        }
    );
    for json in [
        r#"{"command":"moveLeft","utf16Offset":2}"#,
        r#"{"command":"moveLeft","extendSelection":"true"}"#,
        r#"{"command":"deleteForward","unexpected":true}"#,
    ] {
        assert!(serde_json::from_str::<Command>(json).is_err(), "{json}");
    }
}

#[test]
fn ime_context_is_read_only_bounded_and_scalar_based_during_preedit() {
    let mut editor = editor(&["A😀é나"]);
    select(&mut editor, caret(0, 2));
    let context = editor.ime_context(0).unwrap();
    assert_eq!(context.cursor_scalar, 2);
    assert_eq!(context.text, "A😀é나");
    assert_eq!(context.anchor_scalar, Some(2));
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "가").unwrap();
    let revision = editor.session().revision();
    assert!(editor.ime_context(revision).unwrap().text.contains('가'));
    assert_eq!(editor.session().revision(), revision);
    assert!(editor.ime_context(0).is_err());
    editor.cancel_composition(revision, id).unwrap();
    let long = "x".repeat(5000);
    let mut editor = NativeEditor::new(
        NativeDocumentSession::from_core(core(&[&long])),
        caret(0, 4000),
    )
    .unwrap();
    let context = editor.ime_context(0).unwrap();
    assert!(context.text.chars().count() <= 4096);
    assert!(context.text_truncated);
    assert_eq!(context.absolute_start_scalar + context.cursor_scalar, 4000);
}

#[test]
fn argument_free_commands_reject_unknown_fields() {
    for command in [
        "deleteBackward",
        "deleteForward",
        "splitParagraph",
        "joinPreviousParagraph",
        "selectAll",
    ] {
        let input = format!("{{\"command\":\"{command}\",\"unexpected\":true}}");
        assert!(serde_json::from_str::<Command>(&input).is_err());
    }
}
