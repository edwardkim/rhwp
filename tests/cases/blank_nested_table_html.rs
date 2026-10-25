use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

fn tables(paragraphs: &[Paragraph]) -> usize {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .map(|control| match control {
            Control::Table(table) => {
                1 + table
                    .cells
                    .iter()
                    .map(|cell| tables(&cell.paragraphs))
                    .sum::<usize>()
            }
            _ => 0,
        })
        .sum()
}

fn first_table(paragraphs: &[Paragraph]) -> &rhwp::model::table::Table {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .unwrap()
}
fn assert_sizes(core: &DocumentCore) {
    let outer = first_table(&core.document().sections[0].paragraphs);
    assert_eq!((outer.common.width, outer.common.height), (30000, 20000));
    assert_eq!(outer.row_count, 1);
    assert_eq!(outer.cells[0].height, 20000);
    let inner = first_table(&outer.cells[0].paragraphs);
    assert_eq!((inner.common.width, inner.common.height), (15000, 7000));
}

#[test]
fn blank_nested_table_survives_html_paste_and_both_save_formats() {
    let html = "<table style=\"width:300pt;height:200pt\"><tr><td style=\"width:300pt;height:200pt\"><table style=\"width:150pt;height:70pt\"><tr><td style=\"width:150pt;height:70pt\"><p></p></td></tr></table></td></tr></table>";
    let multiple_rows = html.replace("height:70pt\"><p></p></td></tr>", "height:30pt\"><p></p></td></tr><tr><td style=\"width:150pt;height:40pt\"><p></p></td></tr>");
    assert_ne!(multiple_rows, html);
    for html in [html.to_string(), multiple_rows] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.paste_html_native(0, 0, 0, &html).unwrap();
        assert_eq!(tables(&core.document().sections[0].paragraphs), 2);
        assert_sizes(&core);
        for bytes in [
            core.export_hwp_with_adapter().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(tables(&reopened.document().sections[0].paragraphs), 2);
            assert_sizes(&reopened);
        }
    }
}

#[test]
fn nested_table_recursion_keeps_the_existing_html_depth_limit() {
    let mut html = "바닥 글".to_string();
    for _ in 0..40 {
        html = format!("<table><tr><td>{html}</td></tr></table>");
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.paste_html_native(0, 0, 0, &html).unwrap();
    assert!(tables(&core.document().sections[0].paragraphs) <= 16);
    assert!(format!("{:?}", core.document()).contains("바닥 글"));
}
