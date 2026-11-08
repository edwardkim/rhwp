use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    DocumentCore,
};
use std::{fs, path::PathBuf};
fn ci(c: &DocumentCore) -> usize {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|v| matches!(v, Control::Table(_)))
        .unwrap()
}
fn main() {
    let b = PathBuf::from(std::env::args().nth(1).unwrap());
    let source =
        fs::read("pdf/issue7998/native-borders/print-source-inputs/solid-parent.hwpx").unwrap();
    for key in ["colspan", "rowspan", "hidden-separate"] {
        let mut c = DocumentCore::from_bytes(&source).unwrap();
        let mut d = c.document().clone();
        d.sections.truncate(1);
        d.sections[0].paragraphs = vec![Paragraph::new_empty()];
        d.sections[0].raw_stream = None;
        for fonts in &mut d.doc_info.font_faces {
            for f in fonts {
                f.name = "Arial".into()
            }
        }
        d.doc_info.char_shapes[0].base_size = 1000;
        c.set_document(d);
        c.create_table_native(0, 0, 0, 2, 2).unwrap();
        let ci = ci(&c);
        for (i, text) in if key == "rowspan" {
            ["MERGED LEFT", "UPPER RIGHT", "", "LOWER RIGHT"]
        } else if key == "colspan" {
            ["MERGED HEADER", "", "LOWER LEFT", "LOWER RIGHT"]
        } else {
            ["UPPER LEFT", "UPPER RIGHT", "LOWER LEFT", "LOWER RIGHT"]
        }
        .iter()
        .enumerate()
        {
            if !text.is_empty() {
                c.insert_text_in_cell_native(0, 0, ci, i, 0, 0, text)
                    .unwrap();
            }
        }
        if key == "colspan" {
            c.merge_table_cells_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        } else if key == "rowspan" {
            c.merge_table_cells_native(0, 0, ci, 0, 0, 1, 0).unwrap();
        } else {
            c.set_cell_properties_native(0,0,ci,0,r##"{"borderLeft":{"type":1,"width":3,"color":"#000000"},"borderRight":{"type":0,"width":3,"color":"#000000"},"borderTop":{"type":1,"width":3,"color":"#000000"},"borderBottom":{"type":1,"width":3,"color":"#000000"}}"##).unwrap();
            c.set_cell_properties_native(0,0,ci,1,r##"{"borderLeft":{"type":0,"width":3,"color":"#000000"},"borderRight":{"type":1,"width":3,"color":"#000000"},"borderTop":{"type":1,"width":3,"color":"#000000"},"borderBottom":{"type":1,"width":3,"color":"#000000"}}"##).unwrap();
        }
        fs::write(
            b.join(format!("input/{key}.hwpx")),
            c.export_hwpx_native().unwrap(),
        )
        .unwrap();
        fs::write(
            b.join(format!("input/{key}.hwp")),
            c.export_hwp_native().unwrap(),
        )
        .unwrap();
        println!("{key}: normal create/insert/merge/border APIs -> HWPX/HWP");
    }
}
