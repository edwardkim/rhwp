fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: probe_program_install <hwp-path>");
    let bytes = std::fs::read(&path).expect("read hwp");
    let core = rhwp::DocumentCore::from_bytes(&bytes).expect("parse");
    let section = &core.document().sections[0];

    for para_idx in 80..90 {
        let Some(para) = section.paragraphs.get(para_idx) else {
            continue;
        };
        println!(
            "para[{para_idx}] chars={} text={:?}",
            para.char_count, para.text
        );
        for (line_idx, seg) in para.line_segs.iter().enumerate() {
            println!(
                "  ls[{line_idx}] start={} vpos={} h={} sp={} col={} width={} tag=0x{:08x}",
                seg.text_start,
                seg.vertical_pos,
                seg.line_height,
                seg.line_spacing,
                seg.column_start,
                seg.segment_width,
                seg.tag
            );
        }
    }

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("wasm wrapper");
    let layout = doc
        .get_page_text_layout_native(7)
        .expect("page text layout");
    println!("{layout}");
}
