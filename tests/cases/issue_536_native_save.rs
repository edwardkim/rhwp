//! Host persistence checkpoints, byte/report ownership and saved-caret contracts.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read};
use std::sync::Arc;

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::model::bin_data::{BinData, BinDataBytes, BinDataContent, BinDataResolver, BinDataType};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::save::{NativeSaveCaretPolicy, NativeSaveFormat, NativeSaveToken};
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::serializer::SerializedFormat;
use rhwp::DocumentCore;

fn core(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    core
}

fn position(offset: u32) -> NativeTextPosition {
    NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: offset,
        cell_path: Vec::new(),
    }
}

fn selection(offset: u32) -> NativeTextSelection {
    NativeTextSelection {
        anchor: position(offset),
        focus: position(offset),
    }
}

fn editor() -> NativeEditor {
    NativeEditor::new(NativeDocumentSession::new_blank().unwrap(), selection(0)).unwrap()
}

#[test]
fn new_and_imported_core_are_dirty_but_opened_bytes_start_clean() {
    assert!(NativeDocumentSession::new_blank().unwrap().is_dirty());
    assert!(NativeDocumentSession::from_core(core("A")).is_dirty());
    let bytes = core("A").export_hwpx_native().unwrap();
    let opened = NativeDocumentSession::open_bytes(&bytes, None).unwrap();
    assert_eq!(opened.revision(), 0);
    assert!(!opened.is_dirty());
}

#[test]
fn prepared_formats_own_reopenable_bytes_and_matching_reports_without_mutation() {
    for (format, expected) in [
        (NativeSaveFormat::Hwp, SerializedFormat::Hwp),
        (NativeSaveFormat::Hwpx, SerializedFormat::Hwpx),
    ] {
        let mut session = NativeDocumentSession::from_core(core("가😀나"));
        let before = format!("{:?}", session.document().document());
        let prepared = session.prepare_save(format).unwrap();
        assert_eq!(prepared.revision(), 0);
        assert_eq!(prepared.content_loss().output_format(), expected);
        assert!(prepared.content_loss().is_empty());
        assert!(session.is_dirty());
        assert_eq!(format!("{:?}", session.document().document()), before);
        assert_eq!(session.revision(), 0);
        drop(session);
        let (bytes, report, revision, token) = prepared.into_parts();
        assert_eq!(report.output_format(), expected);
        assert_eq!(revision, 0);
        assert_ne!(token.as_u64(), 0);
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, "가😀나");
    }
}

#[test]
fn save_acknowledgement_is_explicit_and_single_use() {
    let mut session = NativeDocumentSession::new_blank().unwrap();
    let prepared = session.prepare_save(NativeSaveFormat::Hwp).unwrap();
    assert!(session.is_dirty());
    assert!(session
        .acknowledge_save(NativeSaveToken::from_raw(0))
        .is_err());
    assert!(session.is_dirty());
    let status = session.acknowledge_save(prepared.token()).unwrap();
    assert_eq!(status.saved_revision, Some(0));
    assert_eq!(status.revision, 0);
    assert!(!status.dirty);
    assert!(!session.is_dirty());
    assert!(session.acknowledge_save(prepared.token()).is_err());
    assert!(session.cancel_save(prepared.token()).is_err());
}

#[test]
fn superseded_foreign_and_cancelled_tokens_cannot_consume_the_latest_checkpoint() {
    let mut session = NativeDocumentSession::new_blank().unwrap();
    let mut other = NativeDocumentSession::new_blank().unwrap();
    let old = session.prepare_save(NativeSaveFormat::Hwp).unwrap();
    let current = session.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    let foreign = other.prepare_save(NativeSaveFormat::Hwp).unwrap();
    assert_ne!(old.token(), current.token());
    assert_ne!(current.token(), foreign.token());
    for invalid in [old.token(), foreign.token(), NativeSaveToken::from_raw(0)] {
        assert!(session.acknowledge_save(invalid).is_err());
        assert!(session.cancel_save(invalid).is_err());
    }
    let status = session.cancel_save(current.token()).unwrap();
    assert!(status.dirty);
    assert_eq!(status.saved_revision, None);
    assert!(session.acknowledge_save(current.token()).is_err());
    let retry = session.prepare_save(NativeSaveFormat::Hwp).unwrap();
    assert!(!session.acknowledge_save(retry.token()).unwrap().dirty);
    assert!(!other.acknowledge_save(foreign.token()).unwrap().dirty);
}

#[test]
fn failed_preparation_preserves_live_state_and_the_previous_checkpoint() {
    let mut invalid = core("text");
    // A missing embedded-font BinData is rejected by the HWPX header writer;
    // HWP retains this reference field. A dangling paragraph style is repaired
    // by canonical export, so it is not a serializer failure fixture.
    invalid.document_mut().doc_info.font_faces[0].push(rhwp::model::style::Font {
        name: "Missing Embedded Face".into(),
        is_embedded: true,
        resolved_bin_data_id: Some(9),
        ..Default::default()
    });
    let mut session = NativeDocumentSession::from_core(invalid);
    let before = format!("{:?}", session.document().document());
    let prior = session.prepare_save(NativeSaveFormat::Hwp).unwrap();
    assert!(session.prepare_save(NativeSaveFormat::Hwpx).is_err());
    assert!(session
        .prepare_save_with_caret(NativeSaveFormat::Hwp, &position(u32::MAX))
        .is_err());
    assert_eq!(format!("{:?}", session.document().document()), before);
    assert_eq!(session.revision(), 0);
    assert!(session.is_dirty());
    assert!(!session.acknowledge_save(prior.token()).unwrap().dirty);
}

#[test]
fn late_acknowledgement_keeps_new_edits_dirty_and_does_not_clear_history() {
    let mut editor = editor();
    let first = editor.replace_selection(0, "가").unwrap();
    let old = editor.prepare_save(NativeSaveFormat::Hwp).unwrap();
    let second = editor.replace_selection(first.revision, "나").unwrap();
    let status = editor.acknowledge_save(old.token()).unwrap();
    assert_eq!(status.saved_revision, Some(first.revision));
    assert_eq!(status.revision, second.revision);
    assert!(status.dirty);
    assert!(editor.can_undo());
    let current = editor.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    assert!(!current.caret_saved());
    assert!(!current.caret_preserved_from_source());
    assert!(!editor.acknowledge_save(current.token()).unwrap().dirty);
    let undo = editor.undo(second.revision).unwrap();
    assert!(editor.session().is_dirty());
    assert!(editor.can_redo());
    let redo = editor.redo(undo.revision).unwrap();
    assert!(redo.revision > second.revision);
    assert!(
        editor.session().is_dirty(),
        "dirty is revision-based, not text equality"
    );
}

#[test]
fn active_composition_blocks_editor_prepare_without_losing_a_pending_checkpoint() {
    let mut editor = editor();
    let prior = editor.prepare_save(NativeSaveFormat::Hwp).unwrap();
    let composition = editor.begin_composition(0).unwrap();
    for format in [NativeSaveFormat::Hwp, NativeSaveFormat::Hwpx] {
        assert!(editor.prepare_save(format).is_err());
    }
    editor
        .cancel_composition(0, composition.composition_id.unwrap())
        .unwrap();
    assert!(!editor.acknowledge_save(prior.token()).unwrap().dirty);
}

#[test]
fn hwp_focus_is_stamped_on_the_snapshot_with_the_same_utf16_and_raw_docinfo_contract() {
    let mut session = NativeDocumentSession::from_core(core("A😀B"));
    let before = format!("{:?}", session.document().document());
    let expected_utf16 = session.document().document().sections[0].paragraphs[0].char_offsets[2];
    let prepared = session
        .prepare_save_with_caret(NativeSaveFormat::Hwp, &position(2))
        .unwrap();
    assert!(prepared.caret_saved());
    assert!(!prepared.caret_preserved_from_source());
    assert_eq!(prepared.caret_policy(), NativeSaveCaretPolicy::CurrentFocus);
    assert_eq!(format!("{:?}", session.document().document()), before);
    let reopened = DocumentCore::from_bytes(prepared.bytes()).unwrap();
    let properties = &reopened.document().doc_properties;
    assert_eq!(properties.caret_list_id, 0);
    assert_eq!(properties.caret_para_id, 0);
    assert_eq!(properties.caret_char_pos, expected_utf16);
    assert!(session
        .prepare_save_with_caret(NativeSaveFormat::Hwpx, &position(2))
        .is_err());
    let hwpx = session.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    assert_eq!(
        hwpx.caret_policy(),
        NativeSaveCaretPolicy::HwpxSourceSettings
    );
    assert!(!hwpx.caret_saved());
    assert!(!hwpx.caret_preserved_from_source());
}

#[derive(Debug)]
struct UnreadableResource;

impl BinDataResolver for UnreadableResource {
    fn resolve(&self, _key: &str) -> Vec<u8> {
        panic!("save must use bounded resource loading")
    }
    fn resolved_len(&self, _key: &str) -> usize {
        1
    }
    fn resolved_is_empty(&self, _key: &str) -> bool {
        false
    }
    // Default bounded/raw resolution returns None without allocating bytes.
}

#[test]
fn content_loss_describes_the_actual_empty_entry_in_the_same_prepared_bytes() {
    for format in [NativeSaveFormat::Hwp, NativeSaveFormat::Hwpx] {
        let mut source = core("text");
        source.document_mut().doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Embedding,
            storage_id: 1,
            extension: Some("png".into()),
            ..Default::default()
        });
        source.document_mut().bin_data_content.push(BinDataContent {
            id: 1,
            extension: "png".into(),
            data: BinDataBytes::Lazy {
                resolver: Arc::new(UnreadableResource),
                key: "missing".into(),
            },
        });
        let mut session = NativeDocumentSession::from_core(source);
        let prepared = session.prepare_save(format).unwrap();
        assert_eq!(prepared.content_loss().len(), 1);
        let loss = &prepared.content_loss().losses()[0];
        assert_eq!(loss.resource_id, Some(1));
        let mut actual = Vec::new();
        match format {
            NativeSaveFormat::Hwp => {
                let mut file = cfb::CompoundFile::open(Cursor::new(prepared.bytes())).unwrap();
                file.open_stream(&loss.path)
                    .unwrap()
                    .read_to_end(&mut actual)
                    .unwrap();
            }
            NativeSaveFormat::Hwpx => {
                let mut file = zip::ZipArchive::new(Cursor::new(prepared.bytes())).unwrap();
                file.by_name(&loss.path)
                    .unwrap()
                    .read_to_end(&mut actual)
                    .unwrap();
            }
        }
        assert!(actual.is_empty());
        assert!(
            session.is_dirty(),
            "producing lossy bytes is not a host save acknowledgement"
        );
    }
}

#[test]
fn save_formats_and_tokens_have_strict_transport_types() {
    assert!(serde_json::from_str::<NativeSaveFormat>("\"pdf\"").is_err());
    for invalid in ["-1", "1.5", "null", "\"1\""] {
        assert!(serde_json::from_str::<NativeSaveToken>(invalid).is_err());
    }
    let mut session = NativeDocumentSession::new_blank().unwrap();
    assert!(matches!(
        session.acknowledge_save(NativeSaveToken::from_raw(u64::MAX)),
        Err(NativeHostError::InvalidArgument(_))
    ));
}

#[test]
fn hwpx_caret_source_flag_matches_preserved_settings_bytes() {
    use std::io::Read;
    let mut original = NativeDocumentSession::new_blank().unwrap();
    let first = original.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    assert!(!first.caret_preserved_from_source());
    let mut reopened = NativeDocumentSession::open_bytes(first.bytes(), None).unwrap();
    let expected = reopened
        .document()
        .document()
        .hwpx_aux_entry("settings.xml")
        .unwrap()
        .to_vec();
    let prepared = reopened.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    assert!(prepared.caret_preserved_from_source());
    assert!(!prepared.caret_saved());
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(prepared.bytes())).unwrap();
    let mut actual = Vec::new();
    archive
        .by_name("settings.xml")
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(actual, expected);
}
