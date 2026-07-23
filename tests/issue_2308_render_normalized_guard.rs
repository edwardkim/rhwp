//! Issue #2308 architecture guard.
//!
//! 렌더 정규화는 editable IR의 전체 section clone이나 경로별 mutable mirror에
//! 의존하지 않아야 한다. 이 테스트는 기능 회귀 테스트와 별도로 금지 구조의
//! 재도입을 빠르게 차단한다.

use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn issue_2308_uses_revision_overlay_without_clone_mirror() {
    let core = source("src/document_core/mod.rs");
    let rendering = source("src/document_core/queries/rendering.rs");
    let editing = source("src/document_core/commands/text_editing.rs");

    assert!(
        core.contains("RenderNormalizationState"),
        "DocumentCore must own an explicit render normalization derived-state cache"
    );
    assert!(
        rendering.contains("RenderPathEntry"),
        "normalization mapping must use an explicit logical path contract"
    );
    assert!(
        !rendering.contains("section.paragraphs.clone()"),
        "render normalization must not clone an entire source section"
    );
    assert!(
        !rendering.contains("refresh_render_normalized_cell_paragraph_after_edit"),
        "deferred edits must invalidate/rederive an overlay entry, not mirror a Paragraph clone"
    );
    assert!(
        !editing.contains("refresh_render_normalized_cell_paragraph_after_edit"),
        "text editing must not call the legacy clone mirror"
    );
    assert!(
        !rendering.contains("cell_idx == 65534"),
        "normalization path mapping must not use the table-caption sentinel"
    );
}
