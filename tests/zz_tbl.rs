use rhwp::model::control::Control;
use rhwp::parser::parse_document;
#[test]
fn probe() {
    for p in std::env::var("ZPATHS").unwrap_or_default().split(';').filter(|s| !s.is_empty()) {
        let Ok(doc) = std::fs::read(p).map_err(|_| ()).and_then(|b| parse_document(&b).map_err(|_| ())) else {
            println!("{p}: 못 읽음");
            continue;
        };
        println!("== {}", p.rsplit('/').next().unwrap_or(p));
        for sec in &doc.sections {
            for para in &sec.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Table(t) = ctrl {
                        println!("  {}행 {}열 셀 {}", t.row_count, t.col_count, t.cells.len());
                        for c in t.cells.iter().take(8) {
                            println!("   ({},{}) span {}x{} size {}x{}",
                                c.row, c.col, c.row_span, c.col_span, c.width, c.height);
                        }
                        return;
                    }
                }
            }
        }
    }
}
