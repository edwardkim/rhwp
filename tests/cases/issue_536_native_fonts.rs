//! Native host font supply contracts; these fixtures do not establish Hancom
//! layout fidelity or coverage beyond the observed synthetic characters.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::native_host::{NativeDocumentSession, NativeHostError};

#[cfg(not(feature = "native-skia"))]
#[test]
fn font_supply_without_skia_reports_feature_disabled_before_filesystem_access() {
    let mut session = NativeDocumentSession::new_blank().unwrap();
    assert!(matches!(
        session.set_font_sources(&["/missing/rhwp-font.ttf".into()], None),
        Err(NativeHostError::FeatureDisabled)
    ));
    assert_eq!(session.font_generation(), 0);
    assert!(session.font_state().sources.is_empty());
}

#[cfg(feature = "native-skia")]
mod native {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use rhwp::native_host::fonts::{MAX_FONT_FILES, MAX_FONT_FILE_BYTES};
    use rhwp::renderer::font_environment::FontEnvironment;
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use rhwp::DocumentCore;

    const REGULAR: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    const ITALIC: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Italic.ttf");
    const COLLECTION: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture.ttc");
    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Directory(PathBuf);

    impl Directory {
        fn new() -> Self {
            loop {
                let serial = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("rhwp-native-fonts-{}-{serial}", std::process::id()));
                match std::fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create isolated font directory: {error}"),
                }
            }
        }

        fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, bytes).unwrap();
            path
        }
    }

    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn environment(id: &str) -> FontEnvironment {
        FontEnvironment::from_json(&format!(
            r#"{{"id":"{id}","substitutions":{{"RHWP Declared Face":"RHWP Host Fixture"}}}}"#
        ))
        .unwrap()
    }

    fn document() -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "A가😀").unwrap();
        let mut document = core.document().clone();
        for language in &mut document.doc_info.font_faces {
            for font in language {
                font.name = "RHWP Host Fixture".into();
            }
        }
        core.set_document(document);
        core
    }

    #[test]
    fn same_contents_are_a_noop_but_same_path_new_contents_advance_only_font_generation() {
        let directory = Directory::new();
        let first = directory.file("one.ttf", REGULAR);
        let copy = directory.file("same.ttf", REGULAR);
        let mut session = NativeDocumentSession::new_blank().unwrap();
        let initial = session.font_state().clone();
        assert!(!session.set_font_sources(&[], None).unwrap());
        assert_eq!(session.font_state(), &initial);
        assert!(session
            .set_font_sources(std::slice::from_ref(&first), None)
            .unwrap());
        assert_eq!(session.font_generation(), 1);
        let state = session.font_state().clone();
        assert_eq!(
            state.sources[0].blake3,
            blake3::hash(REGULAR).to_hex().to_string()
        );
        assert_eq!(state.sources[0].byte_length, REGULAR.len());
        assert_eq!(state.sources[0].face_count, 1);
        assert!(!session.set_font_sources(&[copy], None).unwrap());
        assert_eq!(session.font_state(), &state);
        assert_eq!(session.font_generation(), 1);

        std::fs::write(&first, ITALIC).unwrap();
        assert!(session.set_font_sources(&[first], None).unwrap());
        assert_ne!(session.font_state().fingerprint, state.fingerprint);
        assert_eq!(session.font_generation(), 2);
        assert_eq!(session.revision(), 0);
        assert!(session.document().font_environment().is_none());
    }

    #[test]
    fn source_order_and_environment_are_observable_but_declarations_are_not_supply() {
        let directory = Directory::new();
        let regular = directory.file("regular.ttf", REGULAR);
        let italic = directory.file("italic.ttf", ITALIC);
        let mut session = NativeDocumentSession::new_blank().unwrap();
        let declared = environment("declaration-only");
        assert!(session
            .set_font_sources(&[], Some(declared.clone()))
            .unwrap());
        assert!(session.font_state().sources.is_empty());
        assert_eq!(session.font_state().environment.as_ref(), Some(&declared));
        assert_eq!(session.document().font_environment(), Some(&declared));
        assert!(!session
            .set_font_sources(&[], Some(declared.clone()))
            .unwrap());
        assert_eq!(session.font_generation(), 1);
        session
            .set_font_sources(&[regular.clone(), italic.clone()], Some(declared.clone()))
            .unwrap();
        let first_order = session.font_state().fingerprint.clone();
        session
            .set_font_sources(&[italic, regular], Some(declared))
            .unwrap();
        assert_ne!(session.font_state().fingerprint, first_order);
        assert_eq!(session.font_generation(), 3);
        assert_eq!(session.revision(), 0);

        let mut core = document();
        let inherited = environment("inherited");
        core.set_font_environment(Some(inherited.clone())).unwrap();
        let mut inherited_session = NativeDocumentSession::from_core(core);
        assert_eq!(
            inherited_session.font_state().environment.as_ref(),
            Some(&inherited)
        );
        assert!(!inherited_session
            .set_font_sources(&[], Some(inherited))
            .unwrap());
        assert_eq!(inherited_session.font_generation(), 0);
        assert!(inherited_session.set_font_sources(&[], None).unwrap());
        assert!(inherited_session.document().font_environment().is_none());
    }

    #[test]
    fn rejected_replacement_preserves_state_environment_and_actual_rendered_frame() {
        let directory = Directory::new();
        let valid = directory.file("valid.ttf", REGULAR);
        let malformed = directory.file("invalid.ttf", b"this is not a font");
        let mut session = NativeDocumentSession::from_core(document());
        let installed_environment = environment("installed");
        session
            .set_font_sources(
                std::slice::from_ref(&valid),
                Some(installed_environment.clone()),
            )
            .unwrap();
        let state = session.font_state().clone();
        let frame = session.render_page(0, 0.5).unwrap();
        assert!(frame
            .pixels
            .bytes
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]));
        assert!(session
            .set_font_sources(&[valid.clone(), malformed], Some(environment("rejected")))
            .is_err());
        let invalid_environment: FontEnvironment =
            serde_json::from_str(r#"{"id":"invalid;id","substitutions":{}}"#).unwrap();
        assert!(session
            .set_font_sources(std::slice::from_ref(&valid), Some(invalid_environment))
            .is_err());
        assert_eq!(session.font_state(), &state);
        assert_eq!(session.font_generation(), 1);
        assert_eq!(
            session.document().font_environment(),
            Some(&installed_environment)
        );
        assert_eq!(session.revision(), 0);
        assert_eq!(session.render_page(0, 0.5).unwrap().pixels, frame.pixels);

        // Successful snapshots keep owning bytes after the caller removes files.
        std::fs::remove_file(&valid).unwrap();
        assert_eq!(session.render_page(0, 0.5).unwrap().pixels, frame.pixels);
        assert_eq!(session.font_state(), &state);

        std::fs::write(&valid, ITALIC).unwrap();
        assert!(session
            .set_font_sources(&[valid], Some(installed_environment))
            .unwrap());
        assert_ne!(session.render_page(0, 0.5).unwrap().pixels, frame.pixels);
        assert_eq!(session.font_generation(), 2);
        assert_eq!(session.revision(), 0);
    }

    #[test]
    fn core_environment_rejection_does_not_commit_prepared_renderer_or_generation() {
        let directory = Directory::new();
        let valid = directory.file("valid.ttf", REGULAR);
        let mut core = document();
        let declared = environment("before-batch");
        core.set_font_environment(Some(declared.clone())).unwrap();
        core.begin_batch_native().unwrap();
        let mut session = NativeDocumentSession::from_core(core);
        let state = session.font_state().clone();
        assert!(session
            .set_font_sources(&[valid], Some(environment("during-batch")))
            .is_err());
        assert_eq!(session.font_state(), &state);
        assert_eq!(session.font_generation(), 0);
        assert_eq!(session.document().font_environment(), Some(&declared));
    }

    #[test]
    fn all_collection_faces_must_load_and_broken_later_face_rejects_whole_replacement() {
        let directory = Directory::new();
        let collection = directory.file("faces.ttc", COLLECTION);
        let mut session = NativeDocumentSession::new_blank().unwrap();
        session
            .set_font_sources(std::slice::from_ref(&collection), None)
            .unwrap();
        assert_eq!(session.font_state().sources[0].face_count, 3);
        let state = session.font_state().clone();
        let mut broken = COLLECTION.to_vec();
        broken[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
        std::fs::write(&collection, &broken).unwrap();
        assert!(session.set_font_sources(&[collection], None).is_err());
        assert_eq!(session.font_state(), &state);
        assert_eq!(session.font_generation(), 1);
    }

    #[test]
    fn input_count_regular_file_size_collection_and_retained_copy_limits_are_explicit() {
        let directory = Directory::new();
        let mut session = NativeDocumentSession::new_blank().unwrap();
        let valid = directory.file("font.ttf", REGULAR);
        let too_many = vec![valid.clone(); MAX_FONT_FILES + 1];
        assert!(matches!(
            session.set_font_sources(&too_many, None),
            Err(NativeHostError::LimitExceeded(_))
        ));
        for invalid in [
            directory.0.clone(),
            directory.0.join("missing.ttf"),
            directory.file("empty.ttf", &[]),
        ] {
            assert!(matches!(
                session.set_font_sources(&[invalid], None),
                Err(NativeHostError::InvalidArgument(_))
            ));
        }
        let oversized = directory.file("large.ttf", REGULAR);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&oversized)
            .unwrap()
            .set_len(MAX_FONT_FILE_BYTES as u64 + 1)
            .unwrap();
        assert!(matches!(
            session.set_font_sources(&[oversized], None),
            Err(NativeHostError::LimitExceeded(_))
        ));
        let mut collection = COLLECTION.to_vec();
        collection[8..12].copy_from_slice(&65u32.to_be_bytes());
        let too_many_faces = directory.file("many-faces.ttc", &collection);
        assert!(matches!(
            session.set_font_sources(&[too_many_faces], None),
            Err(NativeHostError::LimitExceeded(_))
        ));
        collection[8..12].copy_from_slice(&64u32.to_be_bytes());
        collection.resize(1024 * 1024 + 1, 0);
        let excessive_copies = directory.file("copies.ttc", &collection);
        assert!(matches!(
            session.set_font_sources(&[excessive_copies], None),
            Err(NativeHostError::LimitExceeded(_))
        ));
        assert_eq!(session.font_generation(), 0);
        assert!(session.font_state().sources.is_empty());
    }

    #[test]
    fn immutable_data_loader_supplies_observed_custom_glyph_coverage() {
        let core = document();
        let mut snapshots = vec![REGULAR.to_vec()];
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&snapshots)
            .unwrap();
        snapshots[0].fill(0);
        let trace: serde_json::Value = serde_json::from_str(
            &renderer
                .get_font_decision_trace(&core, 0, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        let records = trace["records"].as_array().unwrap();
        assert!(!records.is_empty());
        for character in ["A", "가", "😀"] {
            let record = records
                .iter()
                .find(|record| record["source"]["character"] == character)
                .unwrap();
            assert_eq!(record["paint"]["native"]["source"], "custom");
            assert!(record["paint"]["native"]["capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|capability| capability == "nativeGlyphCoverageObserved"));
        }
        assert!(SkiaLayerRenderer::new()
            .try_with_font_data(&snapshots)
            .is_err());
    }
}
