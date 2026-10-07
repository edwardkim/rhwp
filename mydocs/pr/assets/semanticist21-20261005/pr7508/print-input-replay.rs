use rhwp::document_core::DocumentCore;
fn main() {
    let root=std::path::Path::new("output/pr-review/semanticist21-20261005/pr7508-print-inputs");
    std::fs::create_dir_all(root).unwrap();
    let mut d=DocumentCore::new_empty();d.create_blank_document_native().unwrap();
    d.insert_text_native(0,0,0,"가나다라마바사아").unwrap();d.split_paragraph_native(0,0,4,None).unwrap();
    d.apply_char_format_native(0,0,0,2,r#"{"bold":true}"#).unwrap();
    d.copy_selection_native(0,0,0,1,4).unwrap();d.paste_internal_native(0,1,4).unwrap();
    std::fs::write(root.join("pr7508-internal-paste.hwp"),d.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    let fragment=r#"{"ro":{"hp":"p0","p0":{"id":0,"np":"p1","ru":[{"cp":"","ch":[{"cc":2,"ci":1936024420,"co":"s0"},{"cc":2,"ci":1668246628,"co":"c0"},{"t":"가나다라"}]}]},"p1":{"id":1,"ru":[{"cp":"","ch":[{"t":"마바사아"}]}]}},"cs":{"s0":{},"c0":{}}}"#;
    let mut d=DocumentCore::new_empty();d.create_blank_document_native().unwrap();d.insert_text_native(0,0,0,"XY").unwrap();
    d.paste_hwp_json_native(0,0,1,fragment).unwrap();
    std::fs::write(root.join("pr7508-foreign-paste.hwp"),d.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
}
