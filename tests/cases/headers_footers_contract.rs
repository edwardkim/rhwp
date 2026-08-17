//! [#5043] `headers-footers` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-hflist-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture_with_header_and_footer() -> PathBuf {
    let bytes = std::fs::read(sample()).unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let header = doc
        .create_header_footer_native(0, true, 0)
        .expect("머리말 생성");
    assert!(header.contains(r#""ok":true"#), "{header}");
    let footer = doc
        .create_header_footer_native(0, false, 2)
        .expect("꼬리말 생성");
    assert!(footer.contains(r#""ok":true"#), "{footer}");
    let out = temp("fx");
    let exported = doc.export_hwp().expect("export");
    std::fs::write(&out, exported).unwrap();
    out
}

#[test]
fn headers_footers_json_has_array() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["headers-footers", src.as_str(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["headersFooters"].as_array().is_some());
    assert_eq!(
        v["count"].as_u64().unwrap(),
        v["headersFooters"].as_array().unwrap().len() as u64
    );
}

#[test]
fn lists_created_header_and_footer() {
    let inserted = fixture_with_header_and_footer();
    let out = Command::new(rhwp_bin())
        .args(["headers-footers", inserted.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let items = v["headersFooters"].as_array().unwrap();
    assert!(items.len() >= 2, "{v}");
    assert!(
        items
            .iter()
            .any(|h| h["isHeader"] == true && h["applyTo"] == 0),
        "{v}"
    );
    assert!(
        items
            .iter()
            .any(|h| h["isHeader"] == false && h["applyTo"] == 2),
        "{v}"
    );
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args(["headers-footers", src.as_str(), "--nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mcp_declared() {
    let out = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_headers_footers"));
}
