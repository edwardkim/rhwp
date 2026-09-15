//! A stored row may use the exact column width before the solver's ÷4×4 step.
//! Synthetic layout contract; no supplied form or Hancom-version claim.
use rhwp::document_core::DocumentCore;
use serde_json::Value;

const FIXTURE: &[u8] = include_bytes!("../fixtures/stored_column_width_quantization/header.hwpx");

#[test]
fn stored_header_survives_column_quantization_but_not_real_width_changes() {
    for remainder in 0..4 {
        for (width_change, dirty, same_line) in
            [(0, false, true), (-1, false, false), (0, true, false)]
        {
            let source = DocumentCore::from_bytes(FIXTURE).expect("synthetic header");
            let mut document = source.document().clone();
            document.sections[0].section_def.page_def.width = 43200 + remainder + width_change;
            let para = &mut document.sections[0].paragraphs[0];
            para.line_segs[0].segment_width = 36000 + remainder;
            if dirty {
                para.invalidate_layout_inputs();
            }
            let mut core = DocumentCore::new_empty();
            core.set_document(document);
            let layout: Value = serde_json::from_str(
                &core
                    .get_page_text_layout_native(0)
                    .expect("header text layout"),
            )
            .unwrap();
            let runs = layout["runs"].as_array().unwrap();
            let left = runs
                .iter()
                .find(|r| r["text"].as_str().unwrap().contains("Left 10"))
                .expect("Left 10");
            let right = runs
                .iter()
                .find(|r| r["text"].as_str().unwrap().contains("Right"))
                .expect("Right");
            assert_eq!(
                left["y"] == right["y"],
                same_line,
                "remainder={remainder}, width_change={width_change}, dirty={dirty}: {layout}"
            );
            if same_line {
                // The synthetic contract puts the compact label in the left quarter
                // and all of "Right" in the right quarter of the physical column.
                // charX measures substrings even when both labels share one run.
                let column_left = 3600.0 / 75.0; // HWPUNIT -> px at 96 dpi.
                let column_width = f64::from(36000 + remainder) / 75.0;
                let left_x = left["x"].as_f64().unwrap();
                let left_width = left["charX"]["Left 10".len()].as_f64().unwrap();
                let right_start = right["text"].as_str().unwrap().find("Right").unwrap();
                let right_x =
                    right["x"].as_f64().unwrap() + right["charX"][right_start].as_f64().unwrap();
                let right_end = right["x"].as_f64().unwrap()
                    + right["charX"][right_start + "Right".len()]
                        .as_f64()
                        .unwrap();
                // x and charX are each serialized to one decimal place.
                assert!((left_x - column_left).abs() <= 0.2, "{layout}");
                assert!(
                    left_width > 0.0 && left_width <= column_width / 4.0,
                    "left label must remain compact: {layout}"
                );
                assert!(
                    right_x >= column_left + column_width * 0.75
                        && right_end > right_x
                        && right_end <= column_left + column_width + 0.2,
                    "right label must stay inside the right quarter: {layout}"
                );
            }
        }
    }
}
