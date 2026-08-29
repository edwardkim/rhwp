// Task #1067 — HWP 파일의 첫 페이지 layer JSON 에서 polygon path 의 transform 추출 확인.
fn main() {
    let path = std::env::args().nth(1).expect("hwp path");
    let bytes = std::fs::read(&path).expect("read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    // 첫 페이지 layer JSON
    let json = doc.get_page_layer_tree(0).expect("layer json");
    // path 항목 추출
    let mut path_count = 0;
    let mut idx = 0;
    while let Some(p) = json[idx..].find("\"type\":\"path\"") {
        path_count += 1;
        let start = idx + p;
        // 다음 path 또는 root까지의 일부 추출 (간단히 200 글자)
        let snippet_end = (start + 800).min(json.len());
        println!("=== path #{} @ {} ===", path_count, start);
        println!("{}", &json[start..snippet_end]);
        println!();
        idx = start + 1;
        if path_count >= 6 {
            break;
        }
    }
    println!("\nTotal paths in page 0: {} (showing first 6)", path_count);
}
