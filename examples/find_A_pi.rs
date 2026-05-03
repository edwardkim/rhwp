use std::fs;
use rhwp::model::control::Control;

fn main() {
    let path = "/Users/planet/rhwp/samples/21_언어_기출_편집가능본.hwp";
    let data = fs::read(path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();

    // Find all paragraphs containing a tiny table (1697 HU width cell with cs=2)
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    for (ci, cell) in t.cells.iter().enumerate() {
                        if cell.col_span >= 2 && cell.width == 1697 {
                            // Check inside cell paragraphs for "[" "A" "]"
                            let mut has_bracket = false;
                            let mut has_close = false;
                            for cp in &cell.paragraphs {
                                if cp.text.contains('[') { has_bracket = true; }
                                if cp.text.contains(']') { has_close = true; }
                            }
                            if has_bracket && has_close {
                                println!("section={} pi={} ci={} cell.aim={} pad(l={} t={} r={} b={}) tab(l={} t={} r={} b={})",
                                    si, pi, ci, cell.apply_inner_margin,
                                    cell.padding.left, cell.padding.top, cell.padding.right, cell.padding.bottom,
                                    t.padding.left, t.padding.top, t.padding.right, t.padding.bottom);
                            }
                        }
                    }
                }
            }
        }
    }
}
