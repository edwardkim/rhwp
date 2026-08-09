//! [#4132] Issue #1144 native Skia PNG filename cache 계약.
//!
//! 파일 전체를 native-skia로 게이트해 Native Skia job·classifier의 파일 게이트
//! 규약이 이 target의 배선을 자동으로 강제한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

#[path = "support/issue_1144_support.rs"]
mod issue_1144_support;

use issue_1144_support::{document_with_filename_footer, layer_tree_texts};

#[test]
fn issue_1144_skia_png_export_entrypoint_does_not_freeze_filename_context() {
    let mut doc = document_with_filename_footer();
    doc.set_file_name("skia-old.hwp");

    let png = doc
        .render_page_png_native(0)
        .expect("Skia PNG export should build through PageLayerTree");
    assert!(!png.is_empty(), "Skia PNG export should produce bytes");

    doc.set_file_name("skia-new.hwp");
    let texts = layer_tree_texts(&doc);

    assert!(
        texts.iter().any(|text| text.contains("skia-new.hwp")),
        "Skia export entrypoint should not leave stale cached filename. texts={texts:?}"
    );
    assert!(
        texts.iter().all(|text| !text.contains("skia-old.hwp")),
        "old filename from Skia export should not remain cached. texts={texts:?}"
    );
}
