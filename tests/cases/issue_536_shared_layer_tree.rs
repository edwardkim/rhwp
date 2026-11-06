//! 관계형 계약: 같은 profile의 immutable snapshot 재사용과 문서 교체 후 세대 분리.
//! 실물 페이지의 조판 좌표나 한컴 시각 일치율을 고정하는 검사가 아니다.

use std::sync::Arc;

use rhwp::paint::RenderProfile;
use rhwp::DocumentCore;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

#[test]
fn warm_shared_queries_reuse_the_complete_immutable_snapshot() {
    let core = blank();
    let first = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    let second = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert!(first.page_width > 0.0 && first.page_height > 0.0);
}

#[test]
fn owned_compatibility_result_cannot_mutate_the_cached_snapshot() {
    let core = blank();
    let shared = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    let width = shared.page_width;
    let root_width = shared.root.bounds.width;
    let mut owned = core.build_page_layer_tree(0).unwrap();
    owned.page_width += 100.0;
    owned.root.bounds.width = 0.0;
    let cached = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    assert!(Arc::ptr_eq(&shared, &cached));
    assert_eq!(cached.page_width, width);
    assert_eq!(cached.root.bounds.width, root_width);
    assert_ne!(owned.page_width, cached.page_width);
}

#[test]
fn print_and_screen_profiles_keep_separate_snapshots() {
    let core = blank();
    let screen = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    let print = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Print)
        .unwrap();
    assert!(!Arc::ptr_eq(&screen, &print));
    assert_eq!(screen.profile, RenderProfile::Screen);
    assert_eq!(print.profile, RenderProfile::Print);
    assert!(Arc::ptr_eq(
        &screen,
        &core
            .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
            .unwrap()
    ));
}

#[test]
fn document_replacement_preserves_old_snapshot_and_rebuilds_new_geometry() {
    let mut core = blank();
    let old = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    let old_width = old.page_width;
    let mut document = core.document().clone();
    document.sections[0].section_def.page_def.width += 7500;
    core.set_document(document);
    let current = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    assert!(!Arc::ptr_eq(&old, &current));
    assert!(current.page_width > old_width);
    assert_eq!(old.page_width, old_width);
    assert_eq!(
        core.build_page_layer_tree(0).unwrap().page_width,
        current.page_width
    );
}

#[test]
fn invalid_page_does_not_displace_a_valid_cached_snapshot() {
    let core = blank();
    let valid = core
        .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
        .unwrap();
    assert!(core
        .build_page_layer_tree_shared_with_profile(u32::MAX, RenderProfile::Screen)
        .is_err());
    assert!(Arc::ptr_eq(
        &valid,
        &core
            .build_page_layer_tree_shared_with_profile(0, RenderProfile::Screen)
            .unwrap()
    ));
}
