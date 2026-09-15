//! 저장 줄 끝의 표와 다음 줄 첫 글자가 같은 가시 위치로 투영되는 경우.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

const FIXTURE: &[u8] = include_bytes!("../fixtures/stored_inline_table_suffix/two_digits.hwpx");

#[test]
fn stored_inline_tables_keep_their_side_of_a_text_line_boundary() {
    // raw stream: table(0..8), "x y "(8..12), table(12..20), h(20).
    // Both 12 and 20 project to visible position 4, but own different tables.
    for (second_start, expected_same_row) in [(20, true), (12, false)] {
        let source = DocumentCore::from_bytes(FIXTURE).expect("synthetic HWPX");
        let mut document = source.document().clone();
        let Control::Table(outer) = &mut document.sections[0].paragraphs[1].controls[0] else {
            panic!("outer cell");
        };
        outer.cells[0].paragraphs[0].line_segs[1].text_start = second_start;
        let mut core = DocumentCore::new_empty();
        core.set_document(document);
        let layout: Value = serde_json::from_str(
            &core
                .get_page_control_layout_native(0)
                .expect("control layout"),
        )
        .unwrap();
        let tables: Vec<_> = layout["controls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["type"] == "table" && c["stableIndex"].as_array().unwrap().len() > 3)
            .collect();
        assert_eq!(tables.len(), 2, "each table is emitted exactly once");
        let outer = layout["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["type"] == "table" && c["stableIndex"].as_array().unwrap().len() == 3)
            .expect("outer table");
        let left = outer["cells"][0]["x"].as_f64().unwrap();
        let right = left + 28800.0 * 96.0 / 7200.0;
        for table in &tables {
            let cell = &table["cells"][0];
            let x = cell["x"].as_f64().unwrap();
            let width = cell["w"].as_f64().unwrap();
            assert!(
                x >= left - 0.5
                    && x + width <= right + 0.5
                    && width >= table["w"].as_f64().unwrap() - 0.5,
                "stored break {second_start}: inline table must fit without clipping: {table}"
            );
        }
        let y = |i: usize| tables[i]["y"].as_f64().unwrap();
        assert_eq!(
            (y(0) - y(1)).abs() < 0.2,
            expected_same_row,
            "stored break {second_start}: table y={},{}",
            y(0),
            y(1)
        );
        if expected_same_row {
            let text: Value =
                serde_json::from_str(&core.get_page_text_layout_native(0).expect("text layout"))
                    .unwrap();
            let suffix = text["runs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["text"] == "h")
                .expect("suffix");
            assert!(
                suffix["y"].as_f64().unwrap() >= y(0) + tables[0]["h"].as_f64().unwrap() - 1.0,
                "h remains on the next stored line"
            );
        }
    }
}
