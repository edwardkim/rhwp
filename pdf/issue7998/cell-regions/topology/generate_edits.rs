//! Run from the repository root; argument 1 is an empty output directory.
//! Public DocumentCore operations reproduce the stage27 inputs without editing XML.
use rhwp::{model::control::Control, parser::pdf::PdfImportOptions, DocumentCore};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};

fn import(name: &str) -> (DocumentCore, usize) {
    let source = if name == "retained" {
        "pdf/issue7998/retained-border-structure/retained-pen.pdf".into()
    } else {
        format!("pdf/issue7998/cell-regions/{name}.pdf")
    };
    let core =
        DocumentCore::from_pdf_bytes(&fs::read(source).unwrap(), &PdfImportOptions::default())
            .unwrap();
    let index = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|value| matches!(value, Control::Table(_)))
        .unwrap();
    (core, index)
}

fn save(core: &DocumentCore, directory: &Path, key: &str) {
    fs::write(
        directory.join(format!("{key}.hwp")),
        core.export_hwp_with_adapter_snapshot().unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join(format!("{key}.hwpx")),
        core.export_hwpx_native().unwrap(),
    )
    .unwrap();
}

fn main() {
    let output = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    assert!(
        !output.exists(),
        "Preserve existing files; choose a new output directory"
    );
    let edits = output.join("edited");
    let preserved = output.join("preserved");
    fs::create_dir_all(&edits).unwrap();
    fs::create_dir_all(&preserved).unwrap();
    for name in ["retained", "hidden-separate"] {
        let (core, _) = import(name);
        save(&core, &preserved, name);
    }
    for (key, source, operation) in [
        ("colspan-split", "colspan", 0),
        ("rowspan-split", "rowspan", 0),
        ("colspan-merge", "colspan", 1),
        ("rowspan-merge", "rowspan", 2),
        ("colspan-into", "colspan", 3),
        ("rowspan-range", "rowspan", 4),
    ] {
        let (mut core, index) = import(source);
        match operation {
            0 => core.split_table_cell_native(0, 0, index, 0, 0),
            1 => core.merge_table_cells_native(0, 0, index, 1, 0, 1, 1),
            2 => core.merge_table_cells_native(0, 0, index, 0, 1, 1, 1),
            3 => core.split_table_cell_into_native(0, 0, index, 0, 0, 1, 2, false, false),
            _ => core.split_table_cells_in_range_native(0, 0, index, 0, 1, 1, 1, 1, 2, false),
        }
        .unwrap();
        save(&core, &edits, key);
    }

    let (mut core, index) = import("rowspan");
    core.merge_table_cells_native(0, 0, index, 0, 1, 1, 1)
        .unwrap();
    let package = core.export_hwpx_native().unwrap();
    // A separate ordinary-format control: remove only rhwp's latent-grid entry.
    let mut source = zip::ZipArchive::new(Cursor::new(package)).unwrap();
    let mut standard = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..source.len() {
        let mut entry = source.by_index(i).unwrap();
        let name = entry.name().unwrap().into_owned();
        if name == "META-INF/rhwp-table-row-grids.json" {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        standard
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        standard.write_all(&bytes).unwrap();
    }
    fs::write(
        edits.join("rowspan-merge-standard.hwpx"),
        standard.finish().unwrap().into_inner(),
    )
    .unwrap();
    core.insert_table_row_native(0, 0, index, 1, true).unwrap();
    save(&core, &edits, "rowspan-merge-row");

    // Same parser/export API used by `rhwp export-hwpx`, with no reflow edits.
    let reopened =
        DocumentCore::from_bytes(&fs::read(edits.join("rowspan-merge.hwp")).unwrap()).unwrap();
    fs::write(
        edits.join("rowspan-merge-from-hwp.hwpx"),
        reopened.export_hwpx_native().unwrap(),
    )
    .unwrap();
}
