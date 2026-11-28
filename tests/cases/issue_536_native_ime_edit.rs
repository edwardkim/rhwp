//! 관계형: UTF-16 원문·선택·히스토리와 원자적 rollback·presentation 무효화를 검사한다.
//! 주석 갱신: 2026-11-07T16:41:01+09:00; 직전 파일 커밋: 2026-11-07T07:41:53+09:00 (0f629e936bd6d3f96a05c36a1fe18c9dcac663dc).
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
//! Atomic bounded IME content/history contracts; OS input methods are not run here.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{
    NativeCellPathEntry, NativeTextPosition, NativeTextSelection,
};
use rhwp::model::control::Control;
use rhwp::native_host::commands::NativeEditCommand;
use rhwp::native_host::editor::ime_edit::{
    NativeImeEdit, NativeImeEditContext, NativeImePreedit, NativeImeReplacement,
    NativeImeUtf16Range, NativeImeUtf16Selection, MAX_IME_EDIT_BYTES, MAX_IME_EDIT_SCALARS,
};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::save::NativeSaveFormat;
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::DocumentCore;

fn position(offset: u32) -> NativeTextPosition {
    NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: offset,
        cell_path: vec![],
    }
}

fn selected(anchor: u32, focus: u32) -> NativeTextSelection {
    NativeTextSelection {
        anchor: position(anchor),
        focus: position(focus),
    }
}

fn core(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    core
}

fn editor(text: &str) -> NativeEditor {
    NativeEditor::new(NativeDocumentSession::from_core(core(text)), selected(0, 0)).unwrap()
}

fn revision(editor: &NativeEditor) -> u64 {
    editor.session().revision()
}

fn text(editor: &NativeEditor) -> &str {
    &editor.session().document().document().sections[0].paragraphs[0].text
}

fn replace(start: u32, end: u32, text: &str) -> NativeImeReplacement {
    NativeImeReplacement {
        range: NativeImeUtf16Range { start, end },
        text: text.into(),
    }
}

fn commit(replacement: NativeImeReplacement) -> NativeImeEdit {
    NativeImeEdit::Commit {
        replacement,
        selection: None,
        preedit: None,
    }
}

fn apply(editor: &mut NativeEditor, edit: NativeImeEdit) {
    let revision = revision(editor);
    let context = editor.ime_edit_context(revision).unwrap();
    editor.apply_ime_edit(revision, &context, edit).unwrap();
}

#[test]
fn native_ime_utf16_commit_and_reversed_selection_are_one_undo() {
    let mut editor = editor("가😀나다");
    editor.set_selection(0, selected(3, 1)).unwrap();
    apply(
        &mut editor,
        NativeImeEdit::Commit {
            replacement: replace(1, 4, "한😀"),
            selection: Some(NativeImeUtf16Selection {
                anchor: 4,
                focus: 1,
            }),
            preedit: None,
        },
    );
    assert_eq!(text(&editor), "가한😀다");
    assert_eq!(editor.selection(), &selected(3, 1));
    assert_eq!(revision(&editor), 1);
    editor.undo(1).unwrap();
    assert_eq!(text(&editor), "가😀나다");
    assert_eq!(editor.selection(), &selected(3, 1));
    assert!(!editor.can_undo());
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "가한😀다");
}

#[test]
fn native_ime_batch_preserves_unmodified_formatting_across_save_and_history() {
    let mut editor = editor("abcdefghijklmno");
    editor.set_selection(0, selected(6, 9)).unwrap();
    editor
        .execute_command(
            0,
            NativeEditCommand::FormatCharacters {
                bold: Some(true),
                font_size: Some(2000),
            },
        )
        .unwrap();
    let before = revision(&editor);
    let context = editor.ime_edit_context(before).unwrap();
    let wire = serde_json::json!({
        "kind": "commitBatch",
        "replacements": [replace(0, 1, "X"), replace(14, 15, "Y")],
        "selection": { "anchor": 15, "focus": 15 }
    });
    let outcome = editor
        .apply_ime_edit(before, &context, serde_json::from_value(wire).unwrap())
        .unwrap();
    assert!(outcome.document_changed && outcome.history_changed);
    assert_eq!(revision(&editor), before + 1);
    assert_eq!(editor.selection(), &selected(15, 15));

    let check = |core: &DocumentCore, expected: &str| {
        let document = core.document();
        let paragraph = &document.sections[0].paragraphs[0];
        assert_eq!(paragraph.text, expected);
        for offset in 0..15 {
            let id = paragraph.char_shape_id_at(offset).unwrap() as usize;
            let shape = &document.doc_info.char_shapes[id];
            assert_eq!(shape.bold, (6..9).contains(&offset), "offset {offset}");
            if shape.bold {
                assert_eq!(shape.base_size, 2000);
            }
        }
    };
    check(editor.session().document(), "XbcdefghijklmnY");
    for format in [NativeSaveFormat::Hwp, NativeSaveFormat::Hwpx] {
        let prepared = editor.prepare_save(format).unwrap();
        let reopened = DocumentCore::from_bytes(prepared.bytes()).unwrap();
        check(&reopened, "XbcdefghijklmnY");
        editor.cancel_save(prepared.token()).unwrap();
    }
    editor.undo(revision(&editor)).unwrap();
    check(editor.session().document(), "abcdefghijklmno");
    assert_eq!(editor.selection(), &selected(6, 9));
    editor.redo(revision(&editor)).unwrap();
    check(editor.session().document(), "XbcdefghijklmnY");
    assert_eq!(editor.selection(), &selected(15, 15));
}

#[test]
fn native_ime_batch_rejects_an_invalid_later_range_without_changing_history() {
    let mut editor = editor("가나다😀");
    editor.replace_selection(0, "x").unwrap();
    editor.undo(revision(&editor)).unwrap();
    let before = revision(&editor);
    let context = editor.ime_edit_context(before).unwrap();
    for replacements in [
        vec![replace(0, 1, "한"), replace(4, 5, "x")],
        vec![replace(0, 3, "한"), replace(2, 3, "다")],
        vec![replace(2, 3, "한"), replace(0, 1, "x")],
    ] {
        assert!(editor
            .apply_ime_edit(
                before,
                &context,
                NativeImeEdit::CommitBatch {
                    replacements,
                    selection: None
                },
            )
            .is_err());
        assert_eq!(text(&editor), "가나다😀");
        assert_eq!(revision(&editor), before);
        assert_eq!(editor.selection(), &context.selection);
        assert!(editor.can_redo() && !editor.can_undo());
    }
}

#[test]
fn native_ime_invalid_surrogates_offsets_text_and_selection_preserve_redo() {
    let mut editor = editor("가😀나");
    editor.replace_selection(0, "x").unwrap();
    editor.undo(revision(&editor)).unwrap();
    let context = editor.ime_edit_context(revision(&editor)).unwrap();
    let before = format!("{:?}", editor.session().document().document());
    let edits = [
        commit(replace(2, 3, "x")),
        commit(replace(3, 2, "x")),
        commit(replace(0, 99, "x")),
        commit(replace(0, 0, "\n")),
        NativeImeEdit::Commit {
            replacement: replace(0, 0, ""),
            selection: Some(NativeImeUtf16Selection {
                anchor: 2,
                focus: 3,
            }),
            preedit: None,
        },
    ];
    for edit in edits {
        assert!(editor
            .apply_ime_edit(context.revision, &context, edit)
            .is_err());
        assert_eq!(
            format!("{:?}", editor.session().document().document()),
            before
        );
        assert_eq!(editor.selection(), &context.selection);
        assert_eq!(revision(&editor), context.revision);
        assert!(editor.can_redo());
        assert!(!editor.can_undo());
    }
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "x가😀나");
}

fn editor_with_matching_cells() -> NativeEditor {
    let mut core = core("");
    core.paste_html_native(
        0,
        0,
        0,
        "<table><tr><td><p>가😀나</p></td><td><p>가😀나</p></td></tr></table>",
    )
    .unwrap();
    let (paragraph, control) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(paragraph, host)| {
            host.controls
                .iter()
                .position(|item| matches!(item, Control::Table(_)))
                .map(|control| (paragraph as u32, control as u32))
        })
        .expect("table in fixture");
    let mut selection = selected(2, 1);
    for position in [&mut selection.anchor, &mut selection.focus] {
        position.paragraph = paragraph;
        position.cell_path.push(NativeCellPathEntry {
            control,
            cell: 0,
            paragraph: 0,
        });
    }
    NativeEditor::new(NativeDocumentSession::from_core(core), selection).unwrap()
}

fn cell_texts(editor: &NativeEditor) -> [&str; 2] {
    let pos = &editor.selection().focus;
    let para =
        &editor.session().document().document().sections[0].paragraphs[pos.paragraph as usize];
    let Control::Table(table) = &para.controls[pos.cell_path[0].control as usize] else {
        panic!("missing fixture table");
    };
    [
        &table.cells[0].paragraphs[0].text,
        &table.cells[1].paragraphs[0].text,
    ]
}

#[test]
fn native_ime_cell_uses_leaf_text_and_cancels_only_new_preedit() {
    let mut editor = editor_with_matching_cells();
    let original = editor.selection().clone();
    let context = editor.ime_edit_context(0).unwrap();
    assert_eq!(context.context.text, "가😀나");
    assert_eq!(context.context.selected_text, "😀");
    assert_eq!(context.context.anchor_scalar, Some(2));
    assert!(editor
        .apply_ime_edit(0, &context, commit(replace(1, 2, "한")))
        .is_err());
    assert_eq!(editor.ime_edit_context(0).unwrap(), context);

    let first = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, first, "ㅎ").unwrap();
    apply(
        &mut editor,
        NativeImeEdit::Continue {
            replacement: replace(1, 2, "하"),
            selection: None,
        },
    );
    assert_eq!(cell_texts(&editor), ["가하나", "가😀나"]);
    apply(
        &mut editor,
        NativeImeEdit::Commit {
            replacement: replace(1, 2, "한"),
            selection: None,
            preedit: Some(NativeImePreedit {
                replacement: replace(2, 2, "ㄱ"),
                selection: None,
            }),
        },
    );
    assert_eq!(cell_texts(&editor), ["가한ㄱ나", "가😀나"]);
    let second = editor.composition_id().unwrap();
    assert_ne!(first, second);
    editor
        .cancel_composition(revision(&editor), second)
        .unwrap();
    assert_eq!(cell_texts(&editor), ["가한나", "가😀나"]);
    let committed_selection = editor.selection().clone();
    editor.undo(revision(&editor)).unwrap();
    assert_eq!(editor.selection(), &original);
    assert_eq!(cell_texts(&editor), ["가😀나", "가😀나"]);
    assert!(!editor.can_undo());
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(editor.selection(), &committed_selection);
    assert_eq!(cell_texts(&editor), ["가한나", "가😀나"]);
}

#[test]
fn native_ime_rejects_equal_sibling_context_and_stale_cell_composition() {
    let mut editor = editor_with_matching_cells();
    let original = editor.ime_edit_context(0).unwrap();
    let first = editor.begin_composition(0).unwrap().composition_id.unwrap();
    let old_composition = editor.ime_edit_context(0).unwrap();
    editor.cancel_composition(0, first).unwrap();
    let mut sibling = original.selection.clone();
    sibling.anchor.cell_path[0].cell = 1;
    sibling.focus.cell_path[0].cell = 1;
    editor.set_selection(0, sibling).unwrap();
    let current = editor.ime_edit_context(0).unwrap();
    assert_eq!(current.context.text, original.context.text);
    assert_eq!(
        current.context.selected_text,
        original.context.selected_text
    );
    assert!(editor
        .apply_ime_edit(0, &original, commit(replace(1, 3, "X")))
        .is_err());
    assert_eq!(editor.ime_edit_context(0).unwrap(), current);

    let second = editor.begin_composition(0).unwrap().composition_id.unwrap();
    assert_ne!(first, second);
    let current = editor.ime_edit_context(0).unwrap();
    assert!(editor
        .apply_ime_edit(
            0,
            &old_composition,
            NativeImeEdit::Continue {
                replacement: replace(1, 3, "X"),
                selection: None,
            }
        )
        .is_err());
    assert_eq!(editor.ime_edit_context(0).unwrap(), current);
    editor.cancel_composition(0, second).unwrap();
    assert_eq!(revision(&editor), 0);
    assert_eq!(cell_texts(&editor), ["가😀나", "가😀나"]);
    assert!(!editor.can_undo() && !editor.can_redo());
}

#[test]
fn native_ime_context_checks_selection_and_composition_identity_without_revision_changes() {
    let mut editor = editor("AB");
    let old = editor.ime_edit_context(0).unwrap();
    editor.set_selection(0, selected(1, 1)).unwrap();
    assert!(editor
        .apply_ime_edit(0, &old, commit(replace(0, 0, "x")))
        .is_err());
    let first = editor.begin_composition(0).unwrap().composition_id.unwrap();
    let old = editor.ime_edit_context(0).unwrap();
    editor.cancel_composition(0, first).unwrap();
    let second = editor.begin_composition(0).unwrap().composition_id.unwrap();
    assert_ne!(first, second);
    assert!(editor
        .apply_ime_edit(0, &old, commit(replace(0, 0, "x")))
        .is_err());
    assert_eq!(editor.composition_id(), Some(second));
    assert_eq!(text(&editor), "AB");
}

#[test]
fn native_ime_continues_preview_in_utf16_and_keeps_original_cancel_and_undo() {
    let mut editor = editor("AB");
    editor.set_selection(0, selected(1, 1)).unwrap();
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "ㄱ").unwrap();
    apply(
        &mut editor,
        NativeImeEdit::Continue {
            replacement: replace(1, 2, "가😀"),
            selection: Some(NativeImeUtf16Selection {
                anchor: 2,
                focus: 2,
            }),
        },
    );
    assert_eq!(text(&editor), "A가😀B");
    assert_eq!(editor.composition_id(), Some(id));
    assert_eq!(editor.selection(), &selected(2, 2));
    let context = editor.ime_edit_context(revision(&editor)).unwrap();
    assert_eq!(
        context.preedit,
        Some(NativeImeUtf16Range { start: 1, end: 4 })
    );
    assert!(editor
        .apply_ime_edit(
            context.revision,
            &context,
            NativeImeEdit::Continue {
                replacement: replace(0, 1, "X"),
                selection: None
            }
        )
        .is_err());
    assert_eq!(text(&editor), "A가😀B");
    apply(&mut editor, commit(replace(4, 4, "")));
    editor.undo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "AB");
    assert_eq!(editor.selection(), &selected(1, 1));
    assert!(!editor.can_undo());
}

#[test]
fn native_ime_commits_surrounding_replacement_then_cancels_only_new_preedit() {
    let mut editor = editor("AB");
    editor.set_selection(0, selected(1, 1)).unwrap();
    let old = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, old, "ㄱ").unwrap();
    apply(
        &mut editor,
        NativeImeEdit::Commit {
            replacement: replace(0, 2, "X가"),
            selection: None,
            preedit: Some(NativeImePreedit {
                replacement: replace(2, 2, "나"),
                selection: None,
            }),
        },
    );
    assert_eq!(text(&editor), "X가나B");
    assert_eq!(
        revision(&editor),
        2,
        "commit and new preview publish one revision"
    );
    let new = editor.composition_id().unwrap();
    assert_ne!(old, new);
    editor.cancel_composition(revision(&editor), new).unwrap();
    assert_eq!(text(&editor), "X가B");
    assert_eq!(editor.selection(), &selected(2, 2));
    editor.undo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "AB");
    assert_eq!(editor.selection(), &selected(1, 1));
    assert!(!editor.can_undo());
    editor.redo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "X가B");
}

#[test]
fn native_ime_explicit_new_preedit_range_is_separate_from_cancel_selection() {
    let mut editor = editor("ABC");
    apply(
        &mut editor,
        NativeImeEdit::Commit {
            replacement: replace(0, 0, ""),
            selection: None,
            preedit: Some(NativeImePreedit {
                replacement: replace(1, 2, "가"),
                selection: None,
            }),
        },
    );
    let id = editor.composition_id().unwrap();
    assert_eq!(text(&editor), "A가C");
    editor
        .update_composition(revision(&editor), id, "나")
        .unwrap();
    assert_eq!(text(&editor), "A나C");
    editor.cancel_composition(revision(&editor), id).unwrap();
    assert_eq!(text(&editor), "ABC");
    assert_eq!(editor.selection(), &selected(0, 0));
    assert!(!editor.can_undo());
}

#[test]
fn native_ime_invalid_new_preedit_is_rejected_before_committing_old_composition() {
    let mut editor = editor("AB");
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "가").unwrap();
    let context = editor.ime_edit_context(revision(&editor)).unwrap();
    let before = format!("{:?}", editor.session().document().document());
    let edit = NativeImeEdit::Commit {
        replacement: replace(0, 1, "각"),
        selection: None,
        preedit: Some(NativeImePreedit {
            replacement: replace(1, 1, "😀"),
            selection: Some(NativeImeUtf16Selection {
                anchor: 2,
                focus: 2,
            }),
        }),
    };
    assert!(editor
        .apply_ime_edit(context.revision, &context, edit)
        .is_err());
    assert_eq!(
        format!("{:?}", editor.session().document().document()),
        before
    );
    assert_eq!(editor.composition_id(), Some(id));
    assert_eq!(editor.selection(), &context.selection);
    assert_eq!(revision(&editor), context.revision);
    editor.cancel_composition(revision(&editor), id).unwrap();
    assert_eq!(text(&editor), "AB");
}

#[test]
fn native_ime_commit_reverting_preview_restores_original_formatting_without_history() {
    let mut core = core("ABC");
    core.apply_char_format_native(0, 0, 1, 2, r#"{"bold":true}"#)
        .unwrap();
    let before = serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap();
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), selected(1, 2)).unwrap();
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "가").unwrap();
    apply(&mut editor, commit(replace(1, 2, "B")));
    assert_eq!(
        serde_json::to_value(&editor.session().document().document().sections[0].paragraphs[0])
            .unwrap(),
        before
    );
    assert_eq!(editor.composition_id(), None);
    assert!(!editor.can_undo());
}

#[test]
fn native_ime_pending_save_bytes_and_checkpoint_survive_failure_and_later_edit() {
    let mut editor = editor("before");
    let prepared = editor.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    let context = editor.ime_edit_context(0).unwrap();
    assert!(editor
        .apply_ime_edit(0, &context, commit(replace(0, 99, "bad")))
        .is_err());
    let clean = editor.acknowledge_save(prepared.token()).unwrap();
    assert!(!clean.dirty);
    let prepared = editor.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    apply(&mut editor, commit(replace(0, 6, "after")));
    let saved = editor.acknowledge_save(prepared.token()).unwrap();
    assert!(saved.dirty);
    assert_eq!(saved.saved_revision, Some(0));
    let reopened = DocumentCore::from_bytes(prepared.bytes()).unwrap();
    assert_eq!(reopened.document().sections[0].paragraphs[0].text, "before");
    assert_eq!(text(&editor), "after");
}

#[test]
fn native_ime_snapshot_budget_rejection_preserves_live_composition() {
    let mut core = core("AB");
    for _ in 0..DocumentCore::MAX_SNAPSHOTS - 4 {
        core.save_snapshot_native();
    }
    let mut editor = NativeEditor::with_history_capacity(
        NativeDocumentSession::from_core(core),
        selected(0, 0),
        1,
    )
    .unwrap();
    editor.replace_selection(0, "x").unwrap();
    let id = editor
        .begin_composition(revision(&editor))
        .unwrap()
        .composition_id
        .unwrap();
    editor
        .update_composition(revision(&editor), id, "가")
        .unwrap();
    let context = editor.ime_edit_context(revision(&editor)).unwrap();
    let edit = NativeImeEdit::Commit {
        replacement: replace(0, 1, "각"),
        selection: None,
        preedit: Some(NativeImePreedit {
            replacement: replace(1, 1, "나"),
            selection: None,
        }),
    };
    assert!(matches!(
        editor.apply_ime_edit(context.revision, &context, edit),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(editor.ime_edit_context(context.revision).unwrap(), context);
    editor.cancel_composition(revision(&editor), id).unwrap();
    assert_eq!(text(&editor), "xAB");
    editor.undo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), "AB");
}

#[test]
fn native_ime_context_rejects_semantic_paragraphs_and_mixed_selection_scopes() {
    let mut core = core("AB");
    let mut document = core.document().clone();
    document.sections[0].paragraphs[0]
        .controls
        .push(Control::Bookmark(Default::default()));
    core.set_document(document);
    let editor = NativeEditor::new(NativeDocumentSession::from_core(core), selected(0, 0)).unwrap();
    assert!(matches!(
        editor.ime_edit_context(0),
        Err(NativeHostError::Unsupported(_))
    ));
    let mut editor = self::editor("AB");
    let mut context = editor.ime_edit_context(0).unwrap();
    context.selection.focus.cell_path.push(NativeCellPathEntry {
        control: 0,
        cell: 0,
        paragraph: 0,
    });
    assert!(matches!(
        editor.apply_ime_edit(0, &context, commit(replace(0, 0, "x"))),
        Err(NativeHostError::Unsupported(_))
    ));
    let mut core = self::core("AB");
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    let mut range = selected(0, 0);
    range.focus.paragraph = 1;
    let editor = NativeEditor::new(NativeDocumentSession::from_core(core), range).unwrap();
    assert!(matches!(
        editor.ime_edit_context(0),
        Err(NativeHostError::Unsupported(_))
    ));
}

#[test]
fn native_ime_large_text_and_invented_context_are_rejected_without_editing() {
    let mut editor = editor("AB");
    let mut context = editor.ime_edit_context(0).unwrap();
    context.context.text = "x".repeat(MAX_IME_EDIT_BYTES + 1);
    assert!(matches!(
        editor.apply_ime_edit(0, &context, commit(replace(0, 0, "x"))),
        Err(NativeHostError::LimitExceeded(_))
    ));
    let context = editor.ime_edit_context(0).unwrap();
    assert!(matches!(
        editor.apply_ime_edit(
            0,
            &context,
            commit(replace(0, 0, &"x".repeat(MAX_IME_EDIT_BYTES + 1)))
        ),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(text(&editor), "AB");
    assert_eq!(revision(&editor), 0);
    assert!(!editor.can_undo());
}

#[test]
fn native_ime_long_result_must_remain_representable_in_the_next_context() {
    let mut editor = editor(&"a".repeat(3000));
    editor.set_selection(0, selected(2500, 2500)).unwrap();
    let context = editor.ime_edit_context(0).unwrap();
    let cursor = context.context.cursor_scalar;
    let edit = NativeImeEdit::Commit {
        replacement: replace(cursor, cursor, ""),
        selection: None,
        preedit: Some(NativeImePreedit {
            replacement: replace(cursor, cursor, &"가".repeat(3000)),
            selection: None,
        }),
    };
    assert!(matches!(
        editor.apply_ime_edit(0, &context, edit),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(text(&editor), "a".repeat(3000));
    assert_eq!(editor.selection(), &selected(2500, 2500));
    assert_eq!(editor.composition_id(), None);
    assert!(!editor.can_undo());
}

#[test]
fn native_ime_new_preedit_must_keep_its_cancel_baseline_queryable() {
    for (old, new) in [("a", "b"), ("😀", "😁")] {
        let original = old.repeat(MAX_IME_EDIT_SCALARS);
        let inserted = new.repeat(MAX_IME_EDIT_SCALARS);
        let original_units = original.encode_utf16().count() as u32;
        let inserted_units = inserted.encode_utf16().count() as u32;
        let mut editor = editor(&original);
        editor.replace_selection(0, "x").unwrap();
        editor.undo(revision(&editor)).unwrap();
        editor
            .set_selection(revision(&editor), selected(7, 3))
            .unwrap();
        let id = editor
            .begin_composition(revision(&editor))
            .unwrap()
            .composition_id
            .unwrap();
        let context = editor.ime_edit_context(revision(&editor)).unwrap();

        // Both incoming strings and final preview fit the contract. The new
        // preedit would still retain an oversized committed cancellation span,
        // making the next context query fail after an otherwise accepted edit.
        let edit = NativeImeEdit::Commit {
            replacement: replace(0, 0, &inserted),
            selection: None,
            preedit: Some(NativeImePreedit {
                replacement: replace(0, inserted_units + original_units, ""),
                selection: None,
            }),
        };
        assert!(matches!(
            editor.apply_ime_edit(context.revision, &context, edit),
            Err(NativeHostError::LimitExceeded(_))
        ));
        assert_eq!(text(&editor), original);
        assert_eq!(editor.ime_edit_context(revision(&editor)).unwrap(), context);
        editor.cancel_composition(revision(&editor), id).unwrap();
        assert_eq!(editor.selection(), &selected(7, 3));
        assert!(editor.can_redo());
        assert!(!editor.can_undo());

        // A bounded cancellation span remains valid even when the committed
        // paragraph exceeds one context. Cancel preserves the commit and one
        // undo restores the original reversed selection and text.
        apply(
            &mut editor,
            NativeImeEdit::Commit {
                replacement: replace(0, 0, &inserted),
                selection: None,
                preedit: Some(NativeImePreedit {
                    replacement: replace(inserted_units, inserted_units + original_units, ""),
                    selection: None,
                }),
            },
        );
        assert_eq!(text(&editor), inserted);
        let context = editor.ime_edit_context(revision(&editor)).unwrap();
        editor
            .cancel_composition(context.revision, context.composition_id.unwrap())
            .unwrap();
        assert_eq!(text(&editor), format!("{inserted}{original}"));
        editor.undo(revision(&editor)).unwrap();
        assert_eq!(text(&editor), original);
        assert_eq!(editor.selection(), &selected(7, 3));
    }
}

#[test]
fn native_ime_long_paragraph_deltas_use_the_exact_bounded_window_origin() {
    let original = "a".repeat(5000);
    let mut editor = editor(&original);
    editor.set_selection(0, selected(4500, 4500)).unwrap();
    let context = editor.ime_edit_context(0).unwrap();
    assert_eq!(context.context.absolute_start_scalar, 2452);
    let cursor = context.context.cursor_scalar;
    editor
        .apply_ime_edit(0, &context, commit(replace(cursor - 1, cursor, "가")))
        .unwrap();
    assert_eq!(
        text(&editor),
        format!("{}가{}", "a".repeat(4499), "a".repeat(500))
    );
    editor.undo(revision(&editor)).unwrap();
    assert_eq!(text(&editor), original);
    assert_eq!(editor.selection(), &selected(4500, 4500));
}

#[test]
fn native_ime_typed_context_roundtrips_and_rejects_unknown_nested_fields() {
    let mut editor = editor("AB");
    let context = editor.ime_edit_context(0).unwrap();
    let value = serde_json::to_value(&context).unwrap();
    assert_eq!(
        serde_json::from_value::<NativeImeEditContext>(value.clone()).unwrap(),
        context
    );
    let mut unknown = value;
    unknown["context"]["guessedOffset"] = serde_json::json!(2);
    assert!(serde_json::from_value::<NativeImeEditContext>(unknown).is_err());
    let mut stale = context.clone();
    stale.font_generation += 1;
    assert!(editor
        .apply_ime_edit(0, &stale, commit(replace(0, 0, "x")))
        .is_err());
    assert_eq!(editor.session().font_generation(), context.font_generation);
    assert_eq!(editor.selection(), &context.selection);
    assert_eq!(text(&editor), "AB");
}

#[test]
fn native_ime_selection_only_delta_changes_no_document_or_history() {
    let mut editor = editor("AB");
    let context = editor.ime_edit_context(0).unwrap();
    let outcome = editor
        .apply_ime_edit(
            0,
            &context,
            NativeImeEdit::Commit {
                replacement: replace(0, 0, ""),
                selection: Some(NativeImeUtf16Selection {
                    anchor: 2,
                    focus: 1,
                }),
                preedit: None,
            },
        )
        .unwrap();
    assert!(!outcome.document_changed && !outcome.history_changed);
    assert_eq!(outcome.revision, 0);
    assert_eq!(editor.selection(), &selected(2, 1));
    assert_eq!(text(&editor), "AB");
    assert!(!editor.can_undo());
    assert!(editor
        .apply_ime_edit(0, &context, commit(replace(0, 0, "x")))
        .is_err());
}

#[test]
fn atomic_ime_worker_rejects_retained_input_capacity_before_execution() {
    use rhwp::native_host::runtime::{
        NativeRequestId, NativeRuntime, NativeRuntimeCommand, NativeRuntimeConfig,
        NativeRuntimeError, NativeRuntimeTarget,
    };
    let editor =
        NativeEditor::new(NativeDocumentSession::new_blank().unwrap(), selected(0, 0)).unwrap();
    let mut expected = editor.ime_edit_context(0).unwrap();
    expected.context.text = String::with_capacity(8192);
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
        max_pending_bytes: 4096,
        ..Default::default()
    })
    .unwrap();
    let command = NativeRuntimeCommand::ApplyImeEdit {
        target: NativeRuntimeTarget {
            document_id: NativeRequestId::from_raw(1),
            revision: 0,
        },
        expected: Box::new(expected),
        edit: NativeImeEdit::Commit {
            replacement: NativeImeReplacement {
                range: NativeImeUtf16Range { start: 0, end: 0 },
                text: String::new(),
            },
            selection: None,
            preedit: None,
        },
    };
    let rejected = runtime.try_submit(command).unwrap_err();
    assert_eq!(rejected.error, NativeRuntimeError::ByteCapacity);
    let command = NativeRuntimeCommand::ApplyImeEdit {
        target: NativeRuntimeTarget {
            document_id: NativeRequestId::from_raw(1),
            revision: 0,
        },
        expected: Box::new(editor.ime_edit_context(0).unwrap()),
        edit: NativeImeEdit::CommitBatch {
            replacements: Vec::with_capacity(8192),
            selection: None,
        },
    };
    let rejected = runtime.try_submit(command).unwrap_err();
    assert_eq!(rejected.error, NativeRuntimeError::ByteCapacity);
    assert!(runtime
        .try_submit(NativeRuntimeCommand::New { replace: None })
        .is_ok());
}
