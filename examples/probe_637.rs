// Quick probe for SVG footer page number format
use std::fs;
fn main() {
    let path = "samples/aift.hwp";
    let data = fs::read(path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    for p in [0u32, 1, 2, 5, 73] {
        let svg = core.render_page_svg_native(p).unwrap_or_default();
        println!("\n=== render_page_svg_native({}) ===", p);
        println!("  total len: {}", svg.len());
        // Find ALL <text> elements with font-size="10"
        for chunk in svg.split("<text").skip(1) {
            if !chunk.contains("font-size=\"10\"") { continue; }
            let header_end = chunk.find('>').unwrap_or(0);
            let body_start = header_end + 1;
            let body_end = chunk.find("</text>").unwrap_or(chunk.len());
            if body_start < body_end {
                let header = &chunk[..header_end];
                let body = &chunk[body_start..body_end];
                let y = header.split("y=\"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("?");
                let trans = header.split("translate(").nth(1).and_then(|s| s.split(')').next()).unwrap_or("");
                println!("  <text y={} trans=({})> body={:?}", y, trans, body);
            }
        }
    }
}
