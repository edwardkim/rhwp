use std::fs;
fn main() {
    let path = "/Users/planet/rhwp/samples/21_언어_기출_편집가능본.hwp";
    let data = fs::read(path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    for (i, ps) in doc.doc_info.para_shapes.iter().enumerate() {
        if i >= 15 && i <= 25 {
            println!("ps_id={}: margin_left={} margin_right={} indent={} border_fill_id={} alignment={:?}",
                i, ps.margin_left, ps.margin_right, ps.indent, ps.border_fill_id, ps.alignment);
        }
    }
}
