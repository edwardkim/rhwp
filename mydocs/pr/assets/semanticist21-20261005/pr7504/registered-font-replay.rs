use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::svg::FontEmbedMode;

// 기존 원본에 실제 글꼴을 등록해 Print SVG·render tree를 출력한다.
// 입력 문서와 source 글꼴의 바이트는 변경하지 않는다.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "입력 HWP, 글꼴 TTF, 출력 디렉터리");
    let input = std::fs::read(&args[1]).unwrap();
    let font = std::fs::read(&args[2]).unwrap();
    let out = std::path::Path::new(&args[3]);
    std::fs::create_dir_all(out).unwrap();
    let mut doc = DocumentCore::from_bytes(&input).unwrap();
    for id in 0..doc.document().doc_info.char_shapes.len() as u32 {
        doc.register_exact_font_source_native(id, 1, &font, 0).unwrap();
    }
    for page in 0..doc.page_count() {
        let svg = doc.render_page_svg_with_fonts_and_profile(
            page, FontEmbedMode::Full, &[], RenderProfile::Print,
        ).unwrap();
        std::fs::write(out.join(format!("native_{}.svg", page + 1)), svg).unwrap();
        let tree = doc.build_page_render_tree(page).unwrap();
        std::fs::write(out.join(format!("native-tree_{}.json", page + 1)), tree.root.to_json()).unwrap();
    }
    println!("등록 글꼴 Print {}쪽", doc.page_count());
}
