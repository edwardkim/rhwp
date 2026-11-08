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
    fs::create_dir_all(b.join("print-source-inputs")).unwrap();
    let source =
        fs::read("pdf/issue7998/native-borders/print-source-inputs/solid-parent.hwpx").unwrap();
    for key in [
        "local-columns",
        "local-outer-only",
        "local-l-shape",
        "merged-blank",
        "empty-merged-row",
        "linear-columns",
        "hidden-blank",
        "local-blank",
        "linear-rows",
    ] {
        if std::env::args().nth(2).is_some_and(|only| only != key) {
            continue;
        }
        let mut c = DocumentCore::from_bytes(&source).unwrap();
        let mut d = c.document().clone();
        d.sections.truncate(1);
        d.sections[0].paragraphs = vec![Paragraph::new_empty()];
        d.sections[0].raw_stream = None;
        for fonts in &mut d.doc_info.font_faces {
            for f in fonts {
                f.name = "Arial".into();
            }
        }
        d.doc_info.char_shapes[0].base_size = 1000;
        c.set_document(d);
        let rows = if key == "empty-merged-row" {
            3
        } else if key == "linear-columns" {
            1
        } else {
            2
        };
        let cols = if key == "linear-rows" { 1 } else { 2 };
        c.create_table_native(0, 0, 0, rows, cols).unwrap();
        let ci = ci(&c);
        for r in 0..rows as usize {
            for col in 0..cols as usize {
                let i = r * cols as usize + col;
                let text = match key {
                    "merged-blank" => ["MERGED HEADER", "", "", "LOWER RIGHT"][i].to_owned(),
                    "empty-merged-row" if r == 1 => String::new(),
                    "hidden-blank" if i == 1 => String::new(),
                    "local-blank" if i == 2 => String::new(),
                    _ => format!("CELL {} {}", r + 1, col + 1),
                };
                if !text.is_empty() {
                    c.insert_text_in_cell_native(0, 0, ci, i, 0, 0, &text)
                        .unwrap();
                }
                let hide_v = match key {
                    "local-columns" | "local-outer-only" | "local-blank" | "linear-columns" => true,
                    "local-l-shape" | "hidden-blank" => r == 0,
                    _ => false,
                };
                let hide_h = key == "linear-rows"
                    || key == "local-outer-only"
                    || (key == "local-l-shape" && col == 0);
                let sides = [
                    ("Left", col == 0 || !hide_v),
                    ("Right", col + 1 == cols as usize || !hide_v),
                    ("Top", r == 0 || !hide_h),
                    ("Bottom", r + 1 == rows as usize || !hide_h),
                ];
                let json = format!(
                    "{{{}}}",
                    sides
                        .iter()
                        .map(|(side, on)| format!(
                            "\"border{}\":{{\"type\":{},\"width\":3,\"color\":\"#000000\"}}",
                            side,
                            if *on { 1 } else { 0 }
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                c.set_cell_properties_native(0, 0, ci, i, &json).unwrap();
            }
        }
        if key == "merged-blank" {
            c.merge_table_cells_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        }
        if key == "empty-merged-row" {
            c.merge_table_cells_native(0, 0, ci, 1, 0, 1, 1).unwrap();
        }
        fs::write(
            b.join(format!("print-source-inputs/{key}.hwpx")),
            c.export_hwpx_native().unwrap(),
        )
        .unwrap();
        fs::write(
            b.join(format!("print-source-inputs/{key}.hwp")),
            c.export_hwp_with_adapter().unwrap(),
        )
        .unwrap();
        println!("{key}: normal create/insert/merge/border APIs -> HWPX/HWP");
    }
}
