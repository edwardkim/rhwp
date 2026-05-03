use std::fs;
use rhwp::model::control::Control;
fn main() {
    let path = "/Users/planet/rhwp/samples/21_언어_기출_편집가능본.hwp";
    let data = fs::read(path).expect("read");
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[299];
    if let Some(Control::Table(t)) = para.controls.get(0) {
        println!("attr: tac={} text_wrap={:?} hrel={:?} vrel={:?}",
            t.common.treat_as_char,
            t.common.text_wrap, t.common.horz_rel_to, t.common.vert_rel_to);
        println!("hofs={} vofs={} w={} h={}",
            t.common.horizontal_offset, t.common.vertical_offset,
            t.common.width, t.common.height);
        println!("margin: l={} t={} r={} b={}",
            t.common.margin.left, t.common.margin.top,
            t.common.margin.right, t.common.margin.bottom);
        println!("table padding: l={} t={} r={} b={}",
            t.padding.left, t.padding.top, t.padding.right, t.padding.bottom);
    }
}
