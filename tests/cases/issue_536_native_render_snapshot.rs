//! 관계형: 불변 렌더 입력의 공유·세대 분리·원 session 종료 이후 소유와 한도를 검사한다.
//! 주석 갱신: 2026-11-07T16:41:01+09:00; 직전 파일 커밋: 2026-11-07T03:08:35+09:00 (bde016c5af9579883707717a3831801dfd92fdb9).
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
//! 한컴 시각 일치율·실물 페이지 좌표·GPU 가속을 판정하는 검사가 아니다.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::render_snapshot::{
    NativeRenderSnapshot, NativeRenderSnapshotLimits, MAX_SNAPSHOT_NODES,
};
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::paint::RenderProfile;
use rhwp::DocumentCore;

fn core(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    core
}

fn selection() -> NativeTextSelection {
    let position = NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: 0,
        cell_path: Vec::new(),
    };
    NativeTextSelection {
        anchor: position.clone(),
        focus: position,
    }
}

fn source_text(snapshot: &NativeRenderSnapshot) -> String {
    snapshot
        .tree()
        .text_sources
        .entries
        .iter()
        .map(|entry| entry.text.as_str())
        .collect()
}

#[test]
fn native_render_snapshots_are_send_sync_and_share_cached_profile_inputs() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<NativeRenderSnapshot>();

    let session = NativeDocumentSession::from_core(core("old text"));
    let screen = session.render_snapshot(0, RenderProfile::Screen).unwrap();
    let again = session.render_snapshot(0, RenderProfile::Screen).unwrap();
    let print = session.render_snapshot(0, RenderProfile::Print).unwrap();
    let core_tree = session
        .document()
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    assert!(Arc::ptr_eq(screen.tree(), again.tree()));
    assert!(Arc::ptr_eq(screen.tree(), &core_tree));
    assert!(!Arc::ptr_eq(screen.tree(), print.tree()));
    assert_eq!(screen.profile(), RenderProfile::Screen);
    assert_eq!(print.profile(), RenderProfile::Print);
    assert_eq!(screen.page(), 0);
    assert_eq!(screen.revision(), session.revision());
    assert_eq!(screen.font_generation(), session.font_generation());
    assert!(screen.usage().nodes > 0);
    assert!(screen.usage().operations > 0);
    assert!(screen.usage().text_bytes >= "old text".len());
    assert_eq!(screen.explicit_font_count(), 0);
}

#[test]
fn editor_changes_do_not_mutate_a_previous_detached_snapshot() {
    let session = NativeDocumentSession::from_core(core("old text"));
    let old = session.render_snapshot(0, RenderProfile::Screen).unwrap();
    let original = source_text(&old);
    let mut editor = NativeEditor::new(session, selection()).unwrap();
    let outcome = editor.replace_selection(0, "new ").unwrap();
    let current = editor
        .session()
        .render_snapshot(0, RenderProfile::Screen)
        .unwrap();
    assert_eq!(old.revision(), 0);
    assert_eq!(current.revision(), outcome.revision);
    assert_ne!(current.revision(), old.revision());
    assert!(!Arc::ptr_eq(old.tree(), current.tree()));
    assert_ne!(source_text(&current), original);
    drop(editor);
    assert_eq!(source_text(&old), original);
    std::thread::spawn(move || assert_eq!(source_text(&old), original))
        .join()
        .unwrap();
}

#[test]
fn snapshot_limits_reject_inputs_without_displacing_a_valid_shared_tree() {
    let session = NativeDocumentSession::from_core(core("bounded text"));
    let valid = session.render_snapshot(0, RenderProfile::Screen).unwrap();
    for limits in [
        NativeRenderSnapshotLimits {
            max_nodes: 0,
            ..Default::default()
        },
        NativeRenderSnapshotLimits {
            max_operations: 0,
            ..Default::default()
        },
        NativeRenderSnapshotLimits {
            max_payload_bytes: 0,
            ..Default::default()
        },
        NativeRenderSnapshotLimits {
            max_depth: 0,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            session.render_snapshot_with_limits(0, RenderProfile::Screen, limits),
            Err(NativeHostError::LimitExceeded(_))
        ));
    }
    for max_nodes in [MAX_SNAPSHOT_NODES + 1, usize::MAX] {
        assert!(matches!(
            session.render_snapshot_with_limits(
                0,
                RenderProfile::Screen,
                NativeRenderSnapshotLimits {
                    max_nodes,
                    ..Default::default()
                }
            ),
            Err(NativeHostError::InvalidArgument(_))
        ));
    }
    assert!(session
        .render_snapshot(u32::MAX, RenderProfile::Screen)
        .is_err());
    assert!(Arc::ptr_eq(
        valid.tree(),
        session
            .render_snapshot(0, RenderProfile::Screen)
            .unwrap()
            .tree()
    ));
}

#[cfg(feature = "native-skia")]
mod native {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::raster_pixels::RasterPixels;
    use rhwp::renderer::skia::SkiaSurfaceRenderOptions;
    use skia_safe::{surfaces, AlphaType, Color, ColorType, ImageInfo};

    const REGULAR: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    const ITALIC: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Italic.ttf");
    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Directory(PathBuf);

    impl Directory {
        fn new() -> Self {
            loop {
                let serial = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "rhwp-native-render-snapshot-{}-{serial}",
                    std::process::id()
                ));
                match std::fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create isolated snapshot directory: {error}"),
                }
            }
        }
    }

    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn replay(snapshot: &NativeRenderSnapshot) -> Vec<u8> {
        let mut target = surfaces::raster_n32_premul((256, 256)).unwrap();
        target.canvas().clear(Color::WHITE);
        snapshot
            .create_renderer()
            .unwrap()
            .render_to_surface_strict(
                &mut target,
                snapshot.tree(),
                SkiaSurfaceRenderOptions::default(),
            )
            .unwrap();
        let info = ImageInfo::new((256, 256), ColorType::RGBA8888, AlphaType::Premul, None);
        let mut bytes = vec![0; info.min_row_bytes() * info.height() as usize];
        assert!(target.read_pixels(&info, &mut bytes, info.min_row_bytes(), (0, 0)));
        bytes
    }

    fn raster(snapshot: &NativeRenderSnapshot) -> RasterPixels {
        snapshot
            .create_renderer()
            .unwrap()
            .render_pixels_with_options(
                snapshot.tree(),
                RasterRenderOptions {
                    scale: 0.5,
                    max_pixels: rhwp::native_host::MAX_FRAME_PIXELS,
                    transparent: false,
                    ..Default::default()
                },
            )
            .unwrap()
    }

    #[test]
    fn detached_font_snapshot_replays_on_a_new_thread_after_sources_change_and_disappear() {
        let directory = Directory::new();
        let path = directory.0.join("host.ttf");
        std::fs::write(&path, REGULAR).unwrap();
        let mut document = core("A가😀");
        let mut model = document.document().clone();
        for language in &mut model.doc_info.font_faces {
            for font in language {
                font.name = "RHWP Host Fixture".into();
            }
        }
        document.set_document(model);
        let mut session = NativeDocumentSession::from_core(document);
        session
            .set_font_sources(std::slice::from_ref(&path), None)
            .unwrap();
        let snapshot = session.render_snapshot(0, RenderProfile::Screen).unwrap();
        let expected = session.render_page(0, 0.5).unwrap().pixels;
        assert_eq!(raster(&snapshot), expected);
        let expected_borrowed = replay(&snapshot);
        assert_eq!(snapshot.usage().explicit_font_bytes, REGULAR.len());
        assert_eq!(snapshot.explicit_font_count(), 1);
        assert_eq!(
            snapshot.font_state().sources[0].blake3,
            blake3::hash(REGULAR).to_hex().to_string()
        );
        std::fs::write(&path, ITALIC).unwrap();
        session
            .set_font_sources(std::slice::from_ref(&path), None)
            .unwrap();
        assert_ne!(snapshot.font_generation(), session.font_generation());
        let current = session.render_snapshot(0, RenderProfile::Screen).unwrap();
        let current_pixels = session.render_page(0, 0.5).unwrap().pixels;
        assert_eq!(current.usage().explicit_font_bytes, ITALIC.len());
        assert_ne!(
            current.font_state().fingerprint,
            snapshot.font_state().fingerprint
        );
        let current_state = session.font_state().clone();
        std::fs::write(&path, b"invalid font").unwrap();
        assert!(session.set_font_sources(&[path], None).is_err());
        assert_eq!(session.font_state(), &current_state);
        let after_rejection = session.render_snapshot(0, RenderProfile::Screen).unwrap();
        assert_eq!(raster(&after_rejection), current_pixels);
        assert_eq!(
            current.font_state(),
            session
                .render_snapshot(0, RenderProfile::Screen)
                .unwrap()
                .font_state()
        );
        drop(session);
        drop(directory);
        let actual = std::thread::spawn(move || (raster(&snapshot), replay(&snapshot)))
            .join()
            .unwrap();
        assert_eq!(actual.0, expected);
        assert_eq!(actual.1, expected_borrowed);
    }

    #[test]
    fn image_payload_is_bounded_and_owned_by_the_detached_snapshot() {
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(2, 2)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let png = png.into_inner();
        let mut document = core("");
        document
            .insert_picture_native(
                0,
                0,
                0,
                &[],
                &png,
                1500,
                1500,
                2,
                2,
                "png",
                "snapshot",
                None,
                None,
            )
            .unwrap();
        let session = NativeDocumentSession::from_core(document);
        let snapshot = session.render_snapshot(0, RenderProfile::Screen).unwrap();
        assert!(snapshot.usage().resource_payload_bytes >= png.len());
        assert!(snapshot.usage().resources > 0);
        assert!(matches!(
            session.render_snapshot_with_limits(
                0,
                RenderProfile::Screen,
                NativeRenderSnapshotLimits {
                    max_resources: 0,
                    ..Default::default()
                }
            ),
            Err(NativeHostError::LimitExceeded(_))
        ));
        let expected = replay(&snapshot);
        assert!(expected.iter().any(|channel| *channel != 255));
        drop(session);
        drop(png);
        assert_eq!(replay(&snapshot), expected);
    }
}
