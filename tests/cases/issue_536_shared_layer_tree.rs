//! 관계형: 같은 profile의 immutable snapshot 공유와 문서·revision 교체 후 세대 분리를 검사한다.
//! 주석 갱신: 2026-11-07T16:41:01+09:00; 직전 파일 커밋: 2026-11-06T12:43:16+09:00 (ca1f63c157401fbc8a95f53a4c2582bf2c1e87a6).
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
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
