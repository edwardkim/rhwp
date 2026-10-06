//! Read-only real textbox diagnostics; never strips unsupported data for import.
use rhwp::document_core::DocumentCore;

#[test]
#[ignore = "diagnostic: set RHWP_3587_TEXTBOX_OUTPUT"]
fn dump_real_textbox_import_source() {
    let source =
        DocumentCore::from_bytes(&std::fs::read("samples/table-in-tbox.hwp").unwrap()).unwrap();
    let paragraph = &source.document().sections[0].paragraphs[4];
    let path = std::path::PathBuf::from(std::env::var("RHWP_3587_TEXTBOX_OUTPUT").unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(paragraph).unwrap()).unwrap();
}
