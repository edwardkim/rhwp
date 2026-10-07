use rhwp::wasm_api::HwpDocument;
use std::{fs, path::Path};
const TEXT: &str = "1) 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하";
fn open(p: &str) -> HwpDocument {
    HwpDocument::from_bytes(&fs::read(p).unwrap()).unwrap()
}
fn save(doc: &HwpDocument, name: &str, out: &Path) {
    let p = out.join(name);
    fs::create_dir_all(&p).unwrap();
    fs::write(
        p.join(format!("{name}.hwp")),
        doc.export_hwp_native().unwrap(),
    )
    .unwrap();
    for page in 0..doc.page_count() {
        fs::write(
            p.join(format!("page_{:03}.svg", page + 1)),
            doc.render_page_svg_native(page as u32).unwrap(),
        )
        .unwrap();
        fs::write(
            p.join(format!("page_{:03}.json", page + 1)),
            doc.get_page_layer_tree_native(page as u32).unwrap(),
        )
        .unwrap();
    }
    println!("{name}: {} pages", doc.page_count());
}
fn main() {
    let out = std::env::args().nth(1).unwrap();
    let out = Path::new(&out);
    for (name, indent) in [
        ("typed-flat", 0),
        ("typed-first", 3000),
        ("typed-hanging", -3000),
    ] {
        let mut d = HwpDocument::create_empty();
        d.create_blank_document_native().unwrap();
        d.insert_text_native(0, 0, 0, TEXT).unwrap();
        d.apply_para_format_native(0, 0, &format!("{{\"indent\":{indent}}}"))
            .unwrap();
        save(&d, name, out);
    }
    let mut d = open("samples/issue6190/center_align_first_line_indent.hwp");
    d.insert_text_native(0, 4, 7, "가").unwrap();
    d.insert_text_native(0, 7, 0, "가").unwrap();
    save(&d, "6190-edited", out);
    // The original has no ColumnDef: Hancom opens that HWP with default
    // 30mm margins rather than its stored 25mm. The serializer now publishes
    // a default ColumnDef; also exercise the actual one-column command.
    // No LineSeg, page margin, paragraph style or object geometry is patched.
    d.set_column_def_native(0, 1, 0, true, 0).unwrap();
    save(&d, "6190-edited-one-column", out);
    let mut d = open("samples/issue6190/center_align_first_line_indent.hwp");
    d.insert_text_native(0, 7, 0, "가\n").unwrap();
    d.set_column_def_native(0, 1, 0, true, 0).unwrap();
    save(&d, "6190-explicit-break-one-column", out);
    for (name, prefix) in [
        ("prefix-spaces", "  "),
        ("prefix-blank-break", "\n"),
        ("prefix-punctuation-break", ".\n"),
    ] {
        let mut d = open("samples/issue6190/center_align_first_line_indent.hwp");
        d.insert_text_native(0, 7, 0, prefix).unwrap();
        save(&d, name, out);
    }
    let mut d = open("samples/biz_plan.hwp");
    d.insert_text_native(0, 51, 67, "가").unwrap();
    save(&d, "biz-edited", out);
    // Independent cell-growth page-boundary references. These saved inputs
    // are for Hancom comparison; rhwp saved-cell height fidelity is not claimed.
    for count in [8, 20] {
        let mut d = open("samples/issue6882/synth_cell_enter_table_growth.hwp");
        let rhwp::model::control::Control::Table(table) =
            &d.document().sections[0].paragraphs[1].controls[0]
        else {
            panic!("table input");
        };
        let last = table.cells[31].paragraphs.len() - 1;
        let len = table.cells[31].paragraphs[last].char_offsets.len();
        for i in 0..count {
            d.split_paragraph_in_cell_native(
                0,
                1,
                0,
                31,
                last + i,
                if i == 0 { len } else { 0 },
                None,
            )
            .unwrap();
        }
        save(&d, &format!("growth-{count}"), out);
    }
}
