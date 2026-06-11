// Repro inspector for the missing-selection-line bug (ecrits task #21):
// a cell paragraph that soft-wraps and ALSO contains an embedded line-break
// character loses the wrapped line before the break from
// get_selection_rects_in_cell (observed: 4 rendered lines, 3 rects — the
// "memory." line vanished). Dumps the cell paragraph's text, line info, the
// full-range rects, and the rendered line of every char offset.
//
//   cargo run --example inspect_line_skip -- /tmp/l4_repro.hwp

use std::env;
use std::fs;

use rhwp::wasm_api::HwpDocument;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "/tmp/l4_repro.hwp".to_string());
    let bytes = fs::read(&path).expect("read repro file");
    let doc = HwpDocument::from_bytes(&bytes).expect("document load failed");

    // The L4 cell: section 0, parent paragraph 1, control 0, cellIdx 7, para 0.
    let (sec, ppara, ctrl, cell, cpara) = (0u32, 1u32, 0u32, 7u32, 0u32);

    let len = doc
        .get_cell_paragraph_length(sec, ppara, ctrl, cell, cpara)
        .expect("len");
    let text = doc
        .get_text_in_cell(sec, ppara, ctrl, cell, cpara, 0, len)
        .unwrap_or_default();
    println!("cell para len={len}");
    println!("cell para text={:?}", text);

    println!(
        "line_info l0 = {}",
        doc.get_line_info_in_cell(sec, ppara, ctrl, cell, cpara, 0)
            .unwrap_or_default()
    );

    println!(
        "full-range selection rects = {}",
        doc.get_selection_rects_in_cell(sec, ppara, ctrl, cell, cpara, 0, cpara, len)
            .unwrap_or_default()
    );

    // Rendered line of every char offset (cursor rect y): exposes where the
    // model's line ranges and the painted lines disagree.
    let mut last_y = f64::MIN;
    for off in 0..=len {
        let raw = doc
            .get_cursor_rect_in_cell(sec, ppara, ctrl, cell, cpara, off)
            .unwrap_or_default();
        if let (Some(xpos), Some(y)) = (extract(&raw, "\"x\":"), extract(&raw, "\"y\":")) {
            if (y - last_y).abs() > 0.5 {
                println!("offset {off:>3}: NEW LINE at y={y:.1} (x={xpos:.1})");
                last_y = y;
            }
        }
    }

    // Model-side line ranges (line_segs): get_line_info_in_cell keys on a char
    // OFFSET and returns that offset's line — sweep to print distinct ranges.
    let mut last = String::new();
    for off in 0..=len {
        let info = doc
            .get_line_info_in_cell(sec, ppara, ctrl, cell, cpara, off)
            .unwrap_or_default();
        if info != last {
            println!("offset {off:>3}: line_info {info}");
            last = info;
        }
    }

    // Render-tree text runs of the cell (page 2): text + charStart, the data
    // find_cell_cursor matches against.
    let tree = doc.get_page_render_tree(1).unwrap_or_default();
    for chunk in tree.split("{\"type\":\"TextRun\"").skip(1) {
        let head = &chunk[..chunk.len().min(400)];
        if head.contains("memory")
            || head.contains("important")
            || head.contains("\\uf0e8")
            || head.contains("그리고")
            || head.contains("합니다")
        {
            let text = head.split("\"text\":\"").nth(1).and_then(|s| s.split('"').next());
            let cs = head.split("\"charStart\":").nth(1).and_then(|s| {
                s.split(|c: char| !c.is_ascii_digit()).next()
            });
            let ctx = head.contains("cellContext") || head.contains("cell_context");
            println!("run text={:?} charStart={:?} hasCellCtx={}", text, cs, ctx);
        }
    }
}

fn extract(json: &str, key: &str) -> Option<f64> {
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !matches!(c, '-' | '+' | '.' | '0'..='9' | 'e' | 'E'))
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}
