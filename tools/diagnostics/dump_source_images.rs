// 문서의 source image op 바이트를 파일로 덤프 — 변환 실패 건 조사용.
// 사용: dump_source_images <file> <outdir>
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("file");
    let outdir = args.next().expect("outdir");
    std::fs::create_dir_all(&outdir).expect("mkdir");
    let bytes = std::fs::read(&path).expect("read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let mut seen = std::collections::BTreeSet::new();
    for page in 0..doc.page_count() {
        let Ok(keys_json) = doc.get_page_source_image_keys(page) else {
            continue;
        };
        for key in keys_json.split('"').skip(1).step_by(2) {
            if key == "cacheable" || key == "keys" || !seen.insert(key.to_string()) {
                continue;
            }
            if let Ok(data) = doc.get_source_image_bytes(key) {
                let name = key.replace([':', '/'], "_");
                std::fs::write(format!("{outdir}/{name}.bin"), &data).expect("write");
            }
        }
    }
    println!("dumped {} images", seen.len());
}
