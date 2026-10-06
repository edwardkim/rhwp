// Task #1113 — exam_social.hwpx 저장본 reproduce (baseline).
//
// 현재 코드에서 HWPX → HWP 저장 → 한컴 에디터 시각 판정 + 정답지 record diff 용 산출물.
fn main() {
    let src = "samples/hwpx/exam_social.hwpx";
    let dst = "output/poc/issue_1113/exam_social-current.hwp";

    let bytes = std::fs::read(src).expect("read source");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse hwpx");

    std::fs::create_dir_all("output/poc/issue_1113").ok();
    let out_bytes = doc.export_hwp_with_adapter().expect("export");
    std::fs::write(dst, &out_bytes).expect("write output");
    println!("저장: {} ({} bytes)", dst, out_bytes.len());
    println!();
    println!("비교 자료:");
    println!("  정답지: samples/exam_social.hwp");
    println!("  현재본: {}", dst);
}
