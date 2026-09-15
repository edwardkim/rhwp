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
                .find(|r| r["text"].as_str().unwrap().contains("Left"))
                .expect("Left");
            let right = runs
                .iter()
                .find(|r| r["text"].as_str().unwrap().contains("Right"))
                .expect("Right");
            assert_eq!(
                left["y"] == right["y"],
                same_line,
                "remainder={remainder}, width_change={width_change}, dirty={dirty}: {layout}"
            );
        }
    }
}
