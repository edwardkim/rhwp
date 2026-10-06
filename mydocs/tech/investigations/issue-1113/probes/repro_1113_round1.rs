// Task #1113 — round1 정정 적용본 산출 (HeaderFooter autonum placeholder 제거).
fn main() {
    let src = "samples/hwpx/exam_social.hwpx";
    let dst = "output/poc/issue_1113/exam_social-round1.hwp";
    let bytes = std::fs::read(src).expect("read source");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse hwpx");
    std::fs::create_dir_all("output/poc/issue_1113").ok();
    let out_bytes = doc.export_hwp_with_adapter().expect("export");
    std::fs::write(dst, &out_bytes).expect("write output");
    println!("저장: {} ({} bytes)", dst, out_bytes.len());
}
