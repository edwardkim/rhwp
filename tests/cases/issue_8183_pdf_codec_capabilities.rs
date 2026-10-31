//! Compile-time feature disclosure, independent of rendering output.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{json, Value};
use std::process::Command;

#[test]
fn codec_capabilities_match_selected_features_and_schema() {
    let support = rhwp::pdf_support::pdf_import_support();
    assert_eq!(support.enabled, cfg!(feature = "pdf-import"));
    #[cfg(feature = "pdf-jbig2")]
    assert!(support.jbig2);
    #[cfg(feature = "pdf-jpeg2000")]
    assert!(support.jpeg2000);
    let expected = json!({
        "enabled": support.enabled,
        "jbig2": support.jbig2,
        "jpeg2000": support.jpeg2000,
        "icc": support.icc,
        "boundedXrefRecovery": support.bounded_xref_recovery,
    });
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    let output = Command::new(binary).arg("capabilities").output().unwrap();
    assert!(output.status.success(), "{:?}", output);
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["pdfImportSupport"], expected);
    assert_eq!(envelope["untrustedContent"], false);
    let import = envelope["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "import-pdf")
        .unwrap();
    assert_eq!(import["available"], support.enabled);
    assert_eq!(import["requiresFeature"], "pdf-import");
    let schema = rhwp::capabilities_schema::capabilities_schema();
    let shape = &schema["$defs"]["Capabilities"]["properties"]["pdfImportSupport"];
    for field in ["enabled", "jbig2", "jpeg2000"] {
        assert_eq!(shape["properties"][field]["type"], "boolean");
    }
}
