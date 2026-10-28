use rhwp::model::{control::Control, document::Document};
use rhwp_pdf_oxide_poc::{compile_page_painted_rules, compile_page_tables};

fn input(relative: &str) -> Vec<u8> {
    let repo = std::env::var("RHWP_POC_REPO").expect("set RHWP_POC_REPO");
    std::fs::read(std::path::Path::new(&repo).join(relative)).unwrap()
}
fn table(doc: &Document) -> &rhwp::model::table::Table {
    let ts: Vec<_> = doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| {
            if let Control::Table(t) = c {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(ts.len(), 1);
    ts[0]
}
#[test]
fn actual_table_cells_paragraphs_and_images_survive_hwpx() {
    let c = compile_page_tables(
        &input("pdf/issue5875/nested_table_text_caption-2020.pdf"),
        0,
    )
    .unwrap();
    let loaded = rhwp::parse_document(&c.hwpx).unwrap();
    let (a, b) = (table(&c.document), table(&loaded));
    assert_eq!((b.row_count, b.col_count, b.cells.len()), (5, 3, 15));
    assert_eq!(c.report["lowering"]["textBoxes"], 1);
    assert_eq!(c.report["lowering"]["paintedRuleRectangles"], 0);
    assert_eq!(a.cell_grid, b.cell_grid);
    for (x, y) in a.cells.iter().zip(&b.cells) {
        assert_eq!(
            (x.row, x.col, x.row_span, x.col_span, x.width, x.height),
            (y.row, y.col, y.row_span, y.col_span, y.width, y.height)
        );
        assert_eq!(x.paragraphs.len(), y.paragraphs.len());
        for (p, q) in x.paragraphs.iter().zip(&y.paragraphs) {
            assert_eq!(p.text, q.text);
            assert_eq!(p.char_offsets, q.char_offsets);
            assert_eq!(p.char_shapes.len(), q.char_shapes.len());
            for (x, y) in p.char_shapes.iter().zip(&q.char_shapes) {
                assert_eq!(
                    (x.start_pos, x.char_shape_id),
                    (y.start_pos, y.char_shape_id)
                );
                assert!(y.start_pos < q.char_count);
            }
            assert_eq!(p.line_segs.len(), q.line_segs.len());
            assert_eq!(p.controls.len(), q.controls.len());
        }
    }
    let picture_cell = b
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 2)
        .unwrap();
    let pictures: Vec<_> = picture_cell
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|c| {
            if let Control::Picture(p) = c {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(pictures.len(), 2);
    assert!(pictures.iter().all(|p| p.common.treat_as_char));
    assert_eq!(loaded.bin_data_content.len(), 2);
    for (x, y) in c
        .document
        .bin_data_content
        .iter()
        .zip(&loaded.bin_data_content)
    {
        assert_eq!(x.data.load(), y.data.load());
    }
    let extracted: String = c.report["extraction"]["spans"]
        .as_array()
        .unwrap()
        .iter()
        .skip(1)
        .map(|s| s["text"].as_str().unwrap())
        .collect();
    let restored: String = b
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|p| p.text.as_str())
        .collect();
    let chars = |s: &str| {
        let mut cs: Vec<_> = s.chars().filter(|c| !c.is_whitespace()).collect();
        cs.sort_unstable();
        cs
    };
    assert_eq!(chars(&extracted), chars(&restored));
}
#[test]
fn page_without_grid_keeps_the_positional_document() {
    let bytes = input("pdf/re-align-center-hancom-hwp-2020.pdf");
    let a = compile_page_painted_rules(&bytes, 0).unwrap();
    let b = compile_page_tables(&bytes, 0).unwrap();
    assert_eq!(b.report["lowering"]["semanticTables"], 0);
    assert_eq!(a.hwpx, b.hwpx);
}
