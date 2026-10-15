//! Native clipboard content/history contracts; no OS clipboard or visual claim.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::native_host::clipboard::{
    NativeClipboardPayload, MAX_CLIPBOARD_BYTES, MAX_CLIPBOARD_PARAGRAPHS,
};
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

fn selection(from: (u32, u32), to: (u32, u32)) -> NativeTextSelection {
    NativeTextSelection {
        anchor: position(from.0, from.1),
        focus: position(to.0, to.1),
    }
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

fn editor(paragraphs: &[&str], range: NativeTextSelection) -> NativeEditor {
    NativeEditor::new(NativeDocumentSession::from_core(core(paragraphs)), range).unwrap()
}

fn texts(editor: &NativeEditor) -> Vec<&str> {
    editor.session().document().document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect()
}

fn document(editor: &NativeEditor) -> String {
    format!("{:?}", editor.session().document().document())
}

#[test]
fn native_clipboard_copy_is_read_only_and_uses_scalar_ranges() {
    let range = selection((0, 4), (0, 1));
    let editor = editor(&["가😀a\u{301}나"], range.clone());
    let before = document(&editor);
    let payload = editor.copy_selection(0).unwrap();
    assert_eq!(payload.plain_text, "😀a\u{301}");
    assert!(payload.html.as_ref().unwrap().contains("😀a\u{301}"));
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), 0);
    assert_eq!(document(&editor), before);
    assert!(!editor.can_undo());
}

#[test]
fn native_clipboard_copy_retains_supported_html_style_and_strict_payload_schema() {
    let mut core = core(&["가😀나"]);
    core.apply_char_format_native(0, 0, 1, 2, r#"{"bold":true}"#)
        .unwrap();
    let editor = NativeEditor::new(
        NativeDocumentSession::from_core(core),
        selection((0, 1), (0, 2)),
    )
    .unwrap();
    let payload = editor.copy_selection(0).unwrap();
    assert_eq!(payload.plain_text, "😀");
    assert!(payload.html.as_ref().unwrap().contains("font-weight:bold"));
    let encoded = serde_json::to_value(&payload).unwrap();
    assert!(encoded.get("plainText").is_some());
    assert_eq!(
        serde_json::from_value::<NativeClipboardPayload>(encoded).unwrap(),
        payload
    );
    assert!(serde_json::from_str::<NativeClipboardPayload>(
        r#"{"plainText":"a","html":null,"extra":true}"#
    )
    .is_err());
}

#[test]
fn native_clipboard_empty_fragments_use_lossless_plain_text_fallback() {
    let editor = editor(&["가", "", "나"], selection((0, 0), (2, 1)));
    let payload = editor.copy_selection(0).unwrap();
    assert_eq!(payload.plain_text, "가\n\n나");
    assert!(
        payload.html.is_none(),
        "legacy exporter omits empty selected paragraphs"
    );
    let mut empty = NativeEditor::new(
        NativeDocumentSession::from_core(core(&["abc"])),
        selection((0, 1), (0, 1)),
    )
    .unwrap();
    let (payload, outcome) = empty.cut_selection(0).unwrap();
    assert_eq!(
        payload,
        NativeClipboardPayload {
            plain_text: String::new(),
            html: None
        }
    );
    assert!(!outcome.document_changed && !outcome.history_changed);
    assert_eq!(texts(&empty), ["abc"]);
}

#[test]
fn native_clipboard_cut_is_one_undo_and_restores_selection_and_formatting() {
    let mut core = core(&["가😀나", "다라마"]);
    core.apply_char_format_native(0, 0, 1, 2, r#"{"italic":true}"#)
        .unwrap();
    let range = selection((1, 2), (0, 1));
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), range.clone()).unwrap();
    let before = document(&editor);
    let (payload, outcome) = editor.cut_selection(0).unwrap();
    assert_eq!(payload.plain_text, "😀나\n다라");
    assert_eq!(texts(&editor), ["가마"]);
    assert_eq!(editor.selection(), &selection((0, 1), (0, 1)));
    assert_eq!(outcome.revision, 1);
    editor.undo(1).unwrap();
    assert_eq!(document(&editor), before);
    assert_eq!(editor.selection(), &range);
    assert!(!editor.can_undo());
    editor.redo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["가마"]);
}

#[test]
fn native_clipboard_plain_text_normalizes_newlines_and_preserves_unicode_order() {
    let mut editor = editor(&["앞뒤"], selection((0, 1), (0, 1)));
    let outcome = editor.paste_text(0, "가😀\r\n나\r다\n").unwrap();
    assert_eq!(texts(&editor), ["앞가😀", "나", "다", "뒤"]);
    assert_eq!(outcome.selection, selection((3, 0), (3, 0)));
    assert_eq!(outcome.revision, 1);
    editor.undo(1).unwrap();
    assert_eq!(texts(&editor), ["앞뒤"]);
    assert_eq!(editor.selection(), &selection((0, 1), (0, 1)));
    assert!(!editor.can_undo());
    editor.redo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["앞가😀", "나", "다", "뒤"]);
}

#[test]
fn native_clipboard_identical_and_empty_plain_paste_preserve_redo() {
    let range = selection((0, 0), (1, 1));
    let mut editor = editor(&["가😀", "나"], range.clone());
    editor.paste_text(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let empty = editor.paste_text(revision, "").unwrap();
    assert_eq!(empty.selection, range);
    assert!(!empty.document_changed && !empty.history_changed);
    let identical = editor.paste_text(revision, "가😀\r\n나").unwrap();
    assert_eq!(identical.revision, revision);
    assert_eq!(identical.selection, selection((1, 1), (1, 1)));
    assert!(editor.can_redo());
    editor.redo(revision).unwrap();
    assert_eq!(texts(&editor), ["x"]);
}

#[test]
fn native_clipboard_html_preserves_core_formatting_and_scalar_caret_in_one_undo() {
    let range = selection((0, 1), (0, 3));
    let mut editor = editor(&["앞가나뒤"], range.clone());
    let before = document(&editor);
    let outcome = editor
        .paste_html(0, "<p><b>😀한</b></p><p><i>글</i></p>")
        .unwrap();
    assert_eq!(texts(&editor), ["앞😀한", "글뒤"]);
    assert_eq!(outcome.selection, selection((1, 1), (1, 1)));
    editor.set_selection(1, selection((0, 1), (0, 3))).unwrap();
    assert!(editor
        .copy_selection(1)
        .unwrap()
        .html
        .unwrap()
        .contains("font-weight:bold"));
    editor.undo(1).unwrap();
    assert_eq!(document(&editor), before);
    assert_eq!(editor.selection(), &range);
    assert!(!editor.can_undo());
    editor.redo(editor.session().revision()).unwrap();
    assert_eq!(texts(&editor), ["앞😀한", "글뒤"]);
}

#[test]
fn native_clipboard_empty_html_restores_parser_metadata_and_preserves_redo() {
    let range = selection((0, 0), (0, 1));
    let mut editor = editor(&["가나"], range.clone());
    editor.paste_text(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let before = document(&editor);
    for html in [
        "",
        " \n ",
        "<html><body></body></html>",
        "<p></p>",
        "<p style=\"font-size:43pt\"></p>",
    ] {
        let outcome = editor.paste_html(revision, html).unwrap();
        assert_eq!(outcome.revision, revision, "{html}");
        assert!(
            !outcome.document_changed && !outcome.history_changed,
            "{html}"
        );
        assert_eq!(editor.selection(), &range, "{html}");
        assert_eq!(document(&editor), before, "{html}");
        assert!(editor.can_redo());
    }
}

#[test]
fn native_clipboard_unsupported_middle_table_rolls_back_entire_html_import() {
    let range = selection((0, 1), (0, 3));
    let mut editor = editor(&["앞가나뒤"], range.clone());
    editor.paste_text(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let before = document(&editor);
    let result = editor.paste_html(
        revision,
        "<p><b>first</b></p><table><tr><td>cell</td></tr></table><p>last</p>",
    );
    assert!(matches!(result, Err(NativeHostError::Unsupported(_))));
    assert_eq!(
        document(&editor),
        before,
        "styles, paragraphs and table must all roll back"
    );
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), revision);
    assert!(editor.can_redo());
    assert!(!editor.can_undo());
}

#[test]
fn native_clipboard_floating_object_in_source_span_is_not_silently_discarded() {
    let mut core = core(&["A", "B", "C"]);
    let mut model = core.document().clone();
    let mut table = Table::default();
    table.common.treat_as_char = false;
    model.sections[0].paragraphs[1]
        .controls
        .push(Control::Table(Box::new(table)));
    core.set_document(model);
    let mut editor = NativeEditor::new(
        NativeDocumentSession::from_core(core),
        selection((0, 0), (0, 0)),
    )
    .unwrap();
    // Editing another paragraph stays supported, and rejection must preserve redo.
    editor.replace_selection(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let range = selection((0, 1), (2, 0));
    editor.set_selection(revision, range.clone()).unwrap();
    let before = document(&editor);
    for operation in ["copy", "cut", "plain", "multiline", "html"] {
        let result = match operation {
            "copy" => editor.copy_selection(revision).map(|_| ()),
            "cut" => editor.cut_selection(revision).map(|_| ()),
            "plain" => editor.paste_text(revision, "replacement").map(|_| ()),
            "multiline" => editor.paste_text(revision, "one\ntwo").map(|_| ()),
            "html" => editor
                .paste_html(revision, "<p>replacement</p>")
                .map(|_| ()),
            _ => unreachable!(),
        };
        assert!(
            matches!(result, Err(NativeHostError::Unsupported(_))),
            "{operation}"
        );
        assert_eq!(document(&editor), before, "{operation}");
        assert_eq!(editor.selection(), &range, "{operation}");
        assert_eq!(editor.session().revision(), revision, "{operation}");
        assert!(editor.can_redo(), "{operation}");
        assert!(!editor.can_undo(), "{operation}");
    }
}

#[test]
fn native_clipboard_html_source_objects_reject_before_lossy_import() {
    let range = selection((0, 0), (0, 2));
    let mut editor = editor(&["원문"], range.clone());
    editor.replace_selection(0, "x").unwrap();
    editor.undo(1).unwrap();
    let revision = editor.session().revision();
    let before = document(&editor);
    for html in [
        "<p>앞<img src=\"missing.png\">뒤</p>",
        "<p>앞<IMG alt=\">\" SRC='missing.png'>뒤</p>",
        "<p>앞<img src='data:image/png;base64,invalid'>뒤</p>",
        "<p>앞<svg><text>object</text></svg>뒤</p>",
        "<p>앞<object data='missing.bin'></object>뒤</p>",
        "<p>앞<iframe src='missing.html'></iframe>뒤</p>",
    ] {
        assert!(
            matches!(
                editor.paste_html(revision, html),
                Err(NativeHostError::Unsupported(_))
            ),
            "{html}"
        );
        assert_eq!(document(&editor), before, "{html}");
        assert_eq!(editor.selection(), &range, "{html}");
        assert_eq!(editor.session().revision(), revision, "{html}");
        assert!(editor.can_redo(), "{html}");
        assert!(!editor.can_undo(), "{html}");
    }
}

#[test]
fn native_clipboard_bookmark_in_middle_paragraph_is_not_plain_text() {
    let mut core = core(&["A", "B", "C"]);
    let mut model = core.document().clone();
    model.sections[0].paragraphs[1]
        .controls
        .push(Control::Bookmark(Default::default()));
    core.set_document(model);
    let range = selection((0, 1), (2, 0));
    let mut editor =
        NativeEditor::new(NativeDocumentSession::from_core(core), range.clone()).unwrap();
    let before = document(&editor);
    assert!(matches!(
        editor.copy_selection(0),
        Err(NativeHostError::Unsupported(_))
    ));
    assert!(matches!(
        editor.cut_selection(0),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(document(&editor), before);
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), 0);
    assert!(!editor.can_undo());
}

#[test]
fn native_clipboard_html_escaped_object_text_and_comments_remain_text() {
    let mut editor = editor(&["원문"], selection((0, 0), (0, 2)));
    editor
        .paste_html(
            0,
            "<html><body><!--StartFragment--><p><b>앞</b>&lt;img&gt;<i>뒤</i></p><!-- <img src='ignored'> --><!--EndFragment--></body></html>",
        )
        .unwrap();
    assert_eq!(texts(&editor), ["앞<img>뒤"]);
    editor.undo(1).unwrap();
    assert_eq!(texts(&editor), ["원문"]);
}

#[test]
fn native_clipboard_payload_limits_and_controls_reject_before_editing() {
    let range = selection((0, 0), (0, 1));
    let mut editor = editor(&["가나"], range.clone());
    let before = document(&editor);
    for text in ["a\tb", "a\0b", "a\u{0085}b"] {
        assert!(matches!(
            editor.paste_text(0, text),
            Err(NativeHostError::Unsupported(_))
        ));
    }
    let oversized = "x".repeat(MAX_CLIPBOARD_BYTES + 1);
    assert!(matches!(
        editor.paste_text(0, &oversized),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert!(matches!(
        editor.paste_html(0, &oversized),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert!(matches!(
        editor.paste_text(0, &"\n".repeat(MAX_CLIPBOARD_PARAGRAPHS)),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(document(&editor), before);
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), 0);
    assert!(!editor.can_undo());
}

#[test]
fn native_clipboard_failed_copy_never_cuts_unsupported_source_content() {
    let range = selection((0, 0), (0, 3));
    let mut editor = editor(&["a\u{0085}b"], range.clone());
    let before = document(&editor);
    assert!(matches!(
        editor.copy_selection(0),
        Err(NativeHostError::Unsupported(_))
    ));
    assert!(matches!(
        editor.cut_selection(0),
        Err(NativeHostError::Unsupported(_))
    ));
    assert_eq!(document(&editor), before);
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.session().revision(), 0);
    assert!(!editor.can_undo());
}

#[test]
fn native_clipboard_all_entry_points_reject_stale_revision_and_active_composition() {
    let mut editor = editor(&["가나"], selection((0, 0), (0, 1)));
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    editor.update_composition(0, id, "각").unwrap();
    let revision = editor.session().revision();
    let before = document(&editor);
    let range = editor.selection().clone();
    assert!(matches!(
        editor.copy_selection(0),
        Err(NativeHostError::StaleRevision { .. })
    ));
    assert!(matches!(
        editor.cut_selection(0),
        Err(NativeHostError::StaleRevision { .. })
    ));
    assert!(matches!(
        editor.paste_text(0, "x"),
        Err(NativeHostError::StaleRevision { .. })
    ));
    assert!(matches!(
        editor.paste_html(0, "<p>x</p>"),
        Err(NativeHostError::StaleRevision { .. })
    ));
    assert!(matches!(
        editor.copy_selection(revision),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert!(matches!(
        editor.cut_selection(revision),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert!(matches!(
        editor.paste_text(revision, "x"),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert!(matches!(
        editor.paste_html(revision, "<p>x</p>"),
        Err(NativeHostError::InvalidArgument(_))
    ));
    assert_eq!(document(&editor), before);
    assert_eq!(editor.selection(), &range);
    assert_eq!(editor.composition_id(), Some(id));
}

#[test]
fn native_clipboard_runtime_schema_rejects_unknown_fields_and_redacts_payloads() {
    use rhwp::native_host::runtime::NativeRuntimeCommand;

    for mut value in [
        serde_json::json!({"op":"copySelection","target":{"documentId":1,"revision":0}}),
        serde_json::json!({"op":"cutSelection","target":{"documentId":1,"revision":0}}),
        serde_json::json!({"op":"pasteText","target":{"documentId":1,"revision":0},"text":"PRIVATE"}),
        serde_json::json!({"op":"pasteHtml","target":{"documentId":1,"revision":0},"html":"<p>PRIVATE</p>"}),
    ] {
        let command: NativeRuntimeCommand = serde_json::from_value(value.clone()).unwrap();
        assert!(!format!("{command:?}").contains("PRIVATE"));
        value["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<NativeRuntimeCommand>(value.clone()).is_err());
        value.as_object_mut().unwrap().remove("unexpected");
        value.as_object_mut().unwrap().remove("target");
        assert!(serde_json::from_value::<NativeRuntimeCommand>(value).is_err());
    }
}

#[test]
fn native_clipboard_runtime_reserves_output_ceiling_and_owned_input_capacity() {
    use rhwp::native_host::runtime::{
        NativeRequestId, NativeRuntime, NativeRuntimeCommand as Command, NativeRuntimeConfig,
        NativeRuntimeError, NativeRuntimeTarget,
    };

    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
        max_pending_bytes: MAX_CLIPBOARD_BYTES - 1,
        ..Default::default()
    })
    .unwrap();
    let target = NativeRuntimeTarget {
        document_id: NativeRequestId::from_raw(1),
        revision: 0,
    };
    for command in [
        Command::CopySelection { target },
        Command::CutSelection { target },
    ] {
        assert_eq!(
            runtime.try_submit(command).unwrap_err().error,
            NativeRuntimeError::ByteCapacity,
            "copy/cut must reserve their possible output before acceptance"
        );
    }
    for html in [false, true] {
        let mut text = String::with_capacity(MAX_CLIPBOARD_BYTES);
        text.push('x');
        let rejected = runtime
            .try_submit(if html {
                Command::PasteHtml { target, html: text }
            } else {
                Command::PasteText { target, text }
            })
            .unwrap_err();
        assert_eq!(rejected.error, NativeRuntimeError::ByteCapacity);
        match *rejected.command {
            Command::PasteText { text, .. } | Command::PasteHtml { html: text, .. } => {
                assert_eq!(text, "x");
                assert!(text.capacity() >= MAX_CLIPBOARD_BYTES);
            }
            other => panic!("rejected input ownership changed: {other:?}"),
        }
    }
    runtime.close();
}
