//! Cell metadata wire compatibility, including preserved malformed UTF-16.
use rhwp::model::{
    control::Control,
    document::Section,
    paragraph::Paragraph,
    table::{Cell, Table},
};

#[test]
fn table_cell_field_names_keep_lossy_surface_and_original_units_until_modified() {
    let cases: &[&[u16]] = &[
        &[],
        &[0x0041],
        &[0xd800],
        &[0xdc00],
        &[0xd800, 0x0041],
        &[0xd83d, 0xde00],
        &[0xd800, 0xdc00, 0xdfff],
        &[0xd55c, 0xae00, 0x0000],
    ];
    for units in cases {
        let mut raw = 1000u32.to_le_bytes().to_vec();
        raw.extend_from_slice(&[0xff, 0x1b, 0x02, 0x01, 0, 0, 0, 0, 0x40, 0x01, 0]);
        raw.extend_from_slice(&(units.len() as u16).to_le_bytes());
        for unit in *units {
            raw.extend_from_slice(&unit.to_le_bytes());
        }
        raw.extend_from_slice(&[0; 8]);
        for modified in [false, true] {
            // Independent standard-library UTF-16 decoding defines the logical name.
            let name = if modified {
                Some("새필드😀".to_owned())
            } else {
                (!units.is_empty()).then(|| String::from_utf16_lossy(units))
            };
            let table = Table {
                row_count: 1,
                col_count: 1,
                row_sizes: vec![1],
                cells: vec![Cell {
                    row_span: 1,
                    col_span: 1,
                    width: 1000,
                    height: 1000,
                    field_name: name.clone(),
                    raw_list_extra: raw.clone(),
                    paragraphs: vec![Paragraph::new_empty()],
                    ..Default::default()
                }],
                ..Default::default()
            };
            let section = Section {
                paragraphs: vec![Paragraph {
                    controls: vec![Control::Table(Box::new(table))],
                    ..Paragraph::new_empty()
                }],
                ..Default::default()
            };
            let bytes = rhwp::serializer::body_text::serialize_section(&section);
            let reopened = rhwp::parser::body_text::parse_body_text_section(&bytes).unwrap();
            let cell = reopened
                .paragraphs
                .iter()
                .flat_map(|para| &para.controls)
                .find_map(|ctrl| {
                    if let Control::Table(table) = ctrl {
                        table.cells.first()
                    } else {
                        None
                    }
                })
                .expect("table cell was lost from the actual record stream");
            assert_eq!(cell.field_name, name);
            if modified {
                assert_ne!(cell.raw_list_extra, raw);
            } else {
                assert_eq!(
                    cell.raw_list_extra, raw,
                    "raw UTF-16 must stay intact: {units:x?}"
                );
            }
        }
    }
}
