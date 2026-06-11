// A cell paragraph that soft-wraps AND contains an embedded line-break char
// must produce a selection rect for EVERY rendered line. The line whose range
// ends on the break char used to vanish: the trailing cursor lookup at
// range_end (the break char's offset, == the next line's start) succeeded
// with the NEXT line's start hit instead of falling back one char, so the
// rect width came out 0 and was dropped (observed live as a missing
// highlight line inside a worksheet table cell).
//
// samples/line_break_cell.hwp: a 1x2 table whose first cell holds ONE
// paragraph: "And it is also important for your memory.\nNEXT LINE TAIL" —
// the English soft-wraps in the ~7cm cell, then the break starts a third
// rendered line.

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn json_number(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !matches!(c, '-' | '+' | '.' | '0'..='9' | 'e' | 'E'))
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

#[test]
fn embedded_line_break_cell_selection_covers_every_rendered_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/line_break_cell.hwp");
    let bytes = fs::read(&path).expect("read line_break_cell.hwp");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse line_break_cell.hwp");

    let (sec, ppara, ctrl, cell, cpara) = (0u32, 1u32, 0u32, 0u32, 0u32);
    let len = doc
        .get_cell_paragraph_length(sec, ppara, ctrl, cell, cpara)
        .expect("cell paragraph length");

    let line_info = doc
        .get_line_info_in_cell(sec, ppara, ctrl, cell, cpara, 0)
        .expect("line info");
    let line_count = json_number(&line_info, "lineCount").expect("lineCount") as usize;
    assert!(
        line_count >= 3,
        "fixture must wrap + break into >= 3 lines, got {line_info}"
    );

    let rects_json = doc
        .get_selection_rects_in_cell(sec, ppara, ctrl, cell, cpara, 0, cpara, len)
        .expect("full-range selection rects");

    // One rect per rendered line — count DISTINCT y values.
    let mut ys: Vec<f64> = rects_json
        .split("\"y\":")
        .skip(1)
        .filter_map(|chunk| {
            let end = chunk
                .find(|c: char| !matches!(c, '-' | '+' | '.' | '0'..='9' | 'e' | 'E'))
                .unwrap_or(chunk.len());
            chunk[..end].parse::<f64>().ok()
        })
        .collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);

    assert_eq!(
        ys.len(),
        line_count,
        "every rendered line needs a selection rect; lines={line_count} rects={rects_json}"
    );
}
