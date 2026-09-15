//! #7168: column quantization is a cache key, not a cache repair tolerance.
//! Hancom 12.0.0.4605 stores 36000-unit rows for the 43202-unit page. A long
//! space run wraps even without a following word, and survives a forced break.
use rhwp::document_core::DocumentCore;
use serde_json::Value;

const IMPORTED: &[u8] = include_bytes!("../fixtures/stored_column_width_quantization/header.hwpx");
const HANCOM: &[u8] =
    include_bytes!("../fixtures/stored_column_width_quantization/header-hancom-2020.hwp");
const TRAILING: &[u8] =
    include_bytes!("../fixtures/stored_column_width_quantization/trailing-spaces-2020.hwp");
const FORCED: &[u8] =
    include_bytes!("../fixtures/stored_column_width_quantization/spaces-before-break-2020.hwp");

fn text_layout(core: &mut DocumentCore) -> Value {
    serde_json::from_str(&core.get_page_text_layout_native(0).expect("text layout")).unwrap()
}

fn right_position(layout: &Value) -> (f64, f64) {
    let run = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["text"].as_str().unwrap().contains("Right"))
        .expect("Right");
    let offset = run["text"].as_str().unwrap().find("Right").unwrap();
    (
        run["x"].as_f64().unwrap() + run["charX"][offset].as_f64().unwrap(),
        run["y"].as_f64().unwrap(),
    )
}

#[test]
fn unquantized_imported_cache_reflows_like_hancom() {
    let mut imported = DocumentCore::from_bytes(IMPORTED).unwrap();
    let mut reference = DocumentCore::from_bytes(HANCOM).unwrap();
    let expected = right_position(&text_layout(&mut reference));
    // Independent PDF: Right begins at x=180.121552pt, yMin=51.7152pt.
    // Layout y is the line top, not the PDF glyph bbox.
    assert!((expected.0 - 240.162).abs() < 2.0, "{expected:?}");
    let layout = text_layout(&mut imported);
    let actual = right_position(&layout);
    assert!((actual.0 - expected.0).abs() < 0.5, "{layout}");
    assert!((actual.1 - expected.1).abs() < 0.5, "{layout}");
    let first = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["text"].as_str().unwrap().contains("Left"))
        .unwrap();
    assert!(actual.1 > first["y"].as_f64().unwrap() + 15.0, "{layout}");
}

#[test]
fn quantized_hancom_rows_survive_remainders_and_edit_invalidation() {
    for fixture in [HANCOM, TRAILING, FORCED] {
        let source = DocumentCore::from_bytes(fixture).unwrap();
        let doc = source.document();
        assert!(doc.sections[0].paragraphs[0]
            .line_segs
            .iter()
            .all(|s| s.segment_width == 36000));
        let mut reference = DocumentCore::from_bytes(fixture).unwrap();
        let expected = text_layout(&mut reference);
        for remainder in 0..4 {
            for dirty in [false, true] {
                let mut document = doc.clone();
                document.sections[0].section_def.page_def.width = 43200 + remainder;
                if dirty {
                    document.sections[0].paragraphs[0].invalidate_layout_inputs();
                }
                let mut core = DocumentCore::new_empty();
                core.set_document(document);
                let actual = text_layout(&mut core);
                let signature = |v: &Value| -> Vec<(String, f64)> {
                    v["runs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|r| {
                            (
                                r["text"].as_str().unwrap().to_owned(),
                                r["y"].as_f64().unwrap(),
                            )
                        })
                        .collect()
                };
                assert_eq!(
                    signature(&actual),
                    signature(&expected),
                    "remainder={remainder}, dirty={dirty}: {actual}"
                );
            }
        }
    }
}
