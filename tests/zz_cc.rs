use rhwp::model::control::Control;
use rhwp::parser::parse_document;
#[test]
fn probe() {
    let doc = parse_document(&std::fs::read(std::env::var("ZPATH").unwrap()).unwrap()).unwrap();
    for sec in &doc.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    for c in t.cells.iter().take(6) {
                        let text: String = c.paragraphs.iter().map(|p| p.text.clone()).collect();
                        println!("({},{}) 문단 {} 글 {:?}", c.row, c.col, c.paragraphs.len(), text);
                    }
                    return;
                }
            }
        }
    }
}
