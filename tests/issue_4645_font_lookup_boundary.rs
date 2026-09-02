//! [#4645] 문서에서 온 face 이름이 SVG full-font embed의 파일 탐색 루트를
//! 벗어나지 않는지 확인한다.
//!
//! 이 테스트는 private lookup helper가 아니라 실제 HML 문서를
//! `DocumentCore::from_bytes`로 열고, public
//! `render_page_svg_with_fonts(..., FontEmbedMode::Full, ...)`로 렌더한다.
//! 따라서 parser → style resolver → SVG font planning → filesystem read 경로를
//! 함께 고정한다.

use base64::Engine;
use rhwp::document_core::DocumentCore;
use rhwp::renderer::svg::FontEmbedMode;
use std::path::{Path, PathBuf};

const HML_TEMPLATE: &str = include_str!("../samples/hml/formatting_table.hml");

fn document_with_face_name(face_name: &str) -> Vec<u8> {
    let replacement = format!("Name=\"{face_name}\"");
    HML_TEMPLATE
        .replace("Name=\"함초롬돋움\"", &replacement)
        .replace("Name=\"함초롬바탕\"", &replacement)
        .into_bytes()
}

fn render_document_with_face_name(face_name: &str, font_root: &Path) -> String {
    let core = DocumentCore::from_bytes(&document_with_face_name(face_name))
        .expect("HML fixture with document-derived font name should parse");
    core.render_page_svg_with_fonts(0, FontEmbedMode::Full, &[font_root.to_path_buf()])
        .expect("public SVG font embedding path should render")
}

fn temporary_font_root() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-issue-4645-font-root-{}-{nonce}",
        std::process::id()
    ))
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn public_svg_embedding_rejects_nested_document_font_candidate_but_embeds_direct_child() {
    let root = temporary_font_root();
    let nested_dir = root.join("nested");
    std::fs::create_dir_all(&nested_dir).expect("create temporary nested font directory");

    let font_stem = format!("RhwpIssue4645Font{}", std::process::id());
    let nested_bytes = b"nested-font-sentinel";
    let direct_bytes = b"direct-font-sentinel";
    std::fs::write(nested_dir.join(format!("{font_stem}.ttf")), nested_bytes)
        .expect("write nested sentinel font");
    std::fs::write(root.join(format!("{font_stem}.ttf")), direct_bytes)
        .expect("write direct-child sentinel font");

    let nested_face = format!("nested/{font_stem}");
    let nested_svg = render_document_with_face_name(&nested_face, &root);
    let nested_b64 = base64::engine::general_purpose::STANDARD.encode(nested_bytes);
    let direct_b64 = base64::engine::general_purpose::STANDARD.encode(direct_bytes);
    assert!(
        !nested_svg.contains(&nested_b64),
        "a document-derived nested candidate must not reach a nested font file"
    );
    assert!(
        !nested_svg.contains(&direct_b64),
        "rejecting a nested candidate must not reinterpret it as a direct-child file"
    );

    let direct_svg = render_document_with_face_name(&font_stem, &root);
    assert!(
        direct_svg.contains(&direct_b64),
        "a valid direct-child document font filename must still be embedded"
    );
    assert!(
        !direct_svg.contains(&nested_b64),
        "the direct-child lookup must not descend into nested directories"
    );

    std::fs::remove_dir_all(root).expect("remove temporary font directory");
}
