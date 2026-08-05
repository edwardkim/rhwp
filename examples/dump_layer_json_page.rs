// 페이지 layer JSON 원문 덤프 — 스윕 anomaly 조사용.
// 사용: dump_layer_json_page <file> <page> [profile]
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("file");
    let page: u32 = args.next().expect("page").parse().expect("page num");
    let profile = args.next().unwrap_or_else(|| "screen".to_string());
    let bytes = std::fs::read(&path).expect("read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let json = doc
        .get_page_layer_tree_with_profile(page, &profile, Some(true))
        .expect("layer json");
    println!("{json}");
}
