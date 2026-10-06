//! Synthetic inline-image Markdown regression; no user documents.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
use rhwp::wasm_api::HwpDocument;
use std::io::Read;
use std::process::Command;

// Generated from a blank scaffold with "Before image", a solid blue PNG,
// and "After image". The picture is treated as a character in the same line.
fn synthetic_document() -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(SYNTHETIC_HWPX.split_whitespace().collect::<String>())
        .expect("valid synthetic HWPX")
}

#[test]
fn inline_image_keeps_text_order_and_is_not_duplicated() {
    let doc = HwpDocument::from_bytes(&synthetic_document()).expect("parse synthetic document");
    let (markdown, images) = doc
        .extract_page_markdown_with_images_native(0)
        .expect("extract Markdown");
    assert_eq!(images.len(), 1, "inline picture must be collected once");
    assert_eq!(markdown.matches("[[RHWP_IMAGE:1]]").count(), 1);
    let before = markdown.find("Before image").expect("leading text");
    let image = markdown.find("[[RHWP_IMAGE:1]]").expect("image token");
    let after = markdown.find("After image").expect("trailing text");
    assert!(before < image && image < after, "{markdown}");
    assert_eq!(
        doc.extract_page_text_native(0).expect("plain text").trim(),
        "Before imageAfter image"
    );
}

#[test]
fn cli_exports_original_inline_image_bytes() {
    let bytes = synthetic_document();
    let output_dir = std::env::temp_dir().join(format!(
        "rhwp-inline-markdown-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&output_dir).expect("temporary directory");
    let input = output_dir.join("inline-image.hwpx");
    std::fs::write(&input, &bytes).expect("write synthetic input");
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let result = Command::new(binary)
        .arg("export-markdown")
        .arg(&input)
        .arg("-o")
        .arg(&output_dir)
        .arg("--json")
        .output()
        .expect("run CLI");
    assert!(result.status.success(), "{result:?}");
    let manifest: serde_json::Value =
        serde_json::from_slice(&result.stdout).expect("JSON manifest");
    assert_eq!(manifest["imageCount"], 1);
    let markdown =
        std::fs::read_to_string(output_dir.join("inline-image.md")).expect("exported Markdown");
    assert_eq!(markdown.matches("![image 1]").count(), 1);
    let image_path = output_dir
        .join("inline-image_assets")
        .join("inline-image_p001_img001.png");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("fixture ZIP");
    let mut original = Vec::new();
    archive
        .by_name("BinData/image1.png")
        .expect("original image")
        .read_to_end(&mut original)
        .expect("read image");
    assert_eq!(std::fs::read(image_path).expect("exported image"), original);
    std::fs::remove_dir_all(&output_dir).expect("clean test outputs");
}

const SYNTHETIC_HWPX: &str = "
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
BgAAAAAMAAwA/QIAAJwVAAAAAA==";
