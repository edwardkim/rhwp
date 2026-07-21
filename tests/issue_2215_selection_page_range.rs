use std::collections::BTreeSet;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

const CELL_CONTEXT: (u32, u32, u32, u32) = (0, 0, 2, 2);

#[derive(Clone, Copy, Debug)]
struct SelectionCase {
    start_para: u32,
    start_offset: u32,
    end_para: u32,
    end_offset: u32,
    start_page_hint: u32,
    end_page_hint: u32,
}

fn load_sample(file_name: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(file_name);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| {
        panic!("read {}: {error}", path.display());
    });
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|error| {
        panic!("parse {}: {error}", path.display());
    })
}

fn selection_rects_with_hints(doc: &HwpDocument, case: SelectionCase) -> String {
    let (section, parent_para, control, cell) = CELL_CONTEXT;
    doc.get_selection_rects_in_cell_ex(
        &json!({
            "sectionIdx": section,
            "parentParaIdx": parent_para,
            "controlIdx": control,
            "cellIdx": cell,
            "startCellParaIdx": case.start_para,
            "startCharOffset": case.start_offset,
            "endCellParaIdx": case.end_para,
            "endCharOffset": case.end_offset,
            "startPageHint": case.start_page_hint,
            "endPageHint": case.end_page_hint,
        })
        .to_string(),
    )
    .expect("hinted cell selection rects")
}

fn copied_text(doc: &mut HwpDocument, case: SelectionCase) -> String {
    let (section, parent_para, control, cell) = CELL_CONTEXT;
    let copied = doc
        .copy_selection_in_cell(
            section,
            parent_para,
            control,
            cell,
            case.start_para,
            case.start_offset,
            case.end_para,
            case.end_offset,
        )
        .expect("copy cell selection");
    serde_json::from_str::<Value>(&copied).expect("copy selection JSON")["text"]
        .as_str()
        .expect("copy selection text")
        .to_string()
}

fn rect_values(json: &str) -> Vec<Value> {
    serde_json::from_str(json).expect("selection rect JSON")
}

fn rect_pages(rects: &[Value]) -> Vec<u64> {
    rects
        .iter()
        .filter_map(|rect| rect["pageIndex"].as_u64())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn page_width(doc: &HwpDocument, page: u32) -> f64 {
    let info = doc.get_page_info(page).expect("page info");
    serde_json::from_str::<Value>(&info).expect("page info JSON")["width"]
        .as_f64()
        .expect("page width")
}

fn assert_single_rect_near(
    file_name: &str,
    label: &str,
    json: &str,
    expected_page: u64,
    expected_x: f64,
    expected_y: f64,
    expected_width: f64,
) {
    let rects = rect_values(json);
    assert_eq!(rects.len(), 1, "{file_name}: {label} rect count");
    let rect = &rects[0];
    assert_eq!(
        rect["pageIndex"].as_u64(),
        Some(expected_page),
        "{file_name}: {label} page"
    );
    for (field, expected) in [
        ("x", expected_x),
        ("y", expected_y),
        ("width", expected_width),
        ("height", 16.0),
    ] {
        let actual = rect[field].as_f64().expect("rect number");
        assert!(
            (actual - expected).abs() <= 0.5,
            "{file_name}: {label} {field} expected {expected}±0.5, got {actual}"
        );
    }
}

#[test]
fn issue_2215_hwp_and_hwpx_preserve_normal_selection_oracles() {
    let first = SelectionCase {
        start_para: 5,
        start_offset: 0,
        end_para: 5,
        end_offset: 10,
        start_page_hint: 0,
        end_page_hint: 0,
    };
    let middle = SelectionCase {
        start_para: 1250,
        start_offset: 0,
        end_para: 1250,
        end_offset: 1,
        start_page_hint: 54,
        end_page_hint: 54,
    };
    let cross = SelectionCase {
        start_para: 1250,
        start_offset: 0,
        end_para: 1275,
        end_offset: 1,
        start_page_hint: 54,
        end_page_hint: 55,
    };

    let mut format_oracles = Vec::new();
    for file_name in [
        "issue1949_giant_cell_nested_tables_perf.hwp",
        "issue1949_giant_cell_nested_tables_perf.hwpx",
    ] {
        let mut doc = load_sample(file_name);

        let first_rects = selection_rects_with_hints(&doc, first);
        assert_single_rect_near(file_name, "first", &first_rects, 0, 84.1, 242.4, 105.0); // [#2430] 메트릭 교정: 111.7→105.0 실측
        let first_text = copied_text(&mut doc, first);
        assert_eq!(first_text, "1.1.1 수면비행");

        let middle_rects = selection_rects_with_hints(&doc, middle);
        assert_single_rect_near(file_name, "middle", &middle_rects, 54, 92.1, 588.5, 7.9); // [#2430] 10.0→7.9 실측
        let middle_text = copied_text(&mut doc, middle);
        assert_eq!(middle_text, "8");

        if file_name.ends_with(".hwp") {
            let stale_but_valid_hints = SelectionCase {
                start_page_hint: 0,
                end_page_hint: 0,
                ..middle
            };
            assert_eq!(
                selection_rects_with_hints(&doc, stale_but_valid_hints),
                middle_rects,
                "valid host-page hints that miss the endpoint must retry the full host range"
            );
        }

        let cross_rects = selection_rects_with_hints(&doc, cross);
        let cross_values = rect_values(&cross_rects);
        assert_eq!(cross_values.len(), 45, "{file_name}: cross rect count");
        assert_eq!(rect_pages(&cross_values), vec![54, 55]);
        let cross_text = copied_text(&mut doc, cross);
        assert_eq!(cross_text.chars().count(), 1517, "{file_name}: copy length");
        assert!(cross_text.starts_with("8.3.2.4 거주구역"));
        assert!(cross_text
            .ends_with("나. 방화문은 어느 쪽에서도 한 사람이 충분히 개폐할 수 있어야 한다.\n다"));

        format_oracles.push((
            blake3::hash(first_rects.as_bytes()),
            blake3::hash(first_text.as_bytes()),
            blake3::hash(middle_rects.as_bytes()),
            blake3::hash(middle_text.as_bytes()),
            blake3::hash(cross_rects.as_bytes()),
            blake3::hash(cross_text.as_bytes()),
        ));
    }

    assert_eq!(
        format_oracles[0], format_oracles[1],
        "HWP/HWPX normal rect and copy bytes must match"
    );
}

#[test]
fn issue_2215_missing_or_invalid_hints_match_the_positional_fallback() {
    let mut doc = load_sample("exam_social.hwp");
    let positional = doc
        .get_selection_rects_in_cell(1, 16, 0, 0, 0, 0, 6, 469)
        .expect("positional fallback");
    let base = json!({
        "sectionIdx": 1,
        "parentParaIdx": 16,
        "controlIdx": 0,
        "cellIdx": 0,
        "startCellParaIdx": 0,
        "startCharOffset": 0,
        "endCellParaIdx": 6,
        "endCharOffset": 469,
    });

    let missing = doc
        .get_selection_rects_in_cell_ex(&base.to_string())
        .expect("missing hint fallback");
    assert_eq!(missing, positional);

    let mut one_sided = base.clone();
    one_sided["startPageHint"] = json!(1);
    let one_sided = doc
        .get_selection_rects_in_cell_ex(&one_sided.to_string())
        .expect("one-sided hint fallback");
    assert_eq!(one_sided, positional);

    let mut invalid = base;
    invalid["startPageHint"] = json!(999);
    invalid["endPageHint"] = json!(999);
    let invalid = doc
        .get_selection_rects_in_cell_ex(&invalid.to_string())
        .expect("invalid hint fallback");
    assert_eq!(invalid, positional);

    let copied = doc
        .copy_selection_in_cell(1, 16, 0, 0, 0, 0, 6, 469)
        .expect("fallback copy remains available");
    assert!(copied.contains("\"ok\":true"));
}

#[test]
fn issue_2215_split_paragraph_same_page_hints_select_the_pointer_fragment() {
    let split_cases = [
        SelectionCase {
            start_para: 17,
            start_offset: 166,
            end_para: 17,
            end_offset: 170,
            start_page_hint: 1,
            end_page_hint: 1,
        },
        SelectionCase {
            start_para: 1277,
            start_offset: 78,
            end_para: 1277,
            end_offset: 82,
            start_page_hint: 56,
            end_page_hint: 56,
        },
        SelectionCase {
            start_para: 2499,
            start_offset: 114,
            end_para: 2499,
            end_offset: 118,
            start_page_hint: 114,
            end_page_hint: 114,
        },
    ];

    let mut failures = Vec::new();
    for file_name in [
        "issue1949_giant_cell_nested_tables_perf.hwp",
        "issue1949_giant_cell_nested_tables_perf.hwpx",
    ] {
        let doc = load_sample(file_name);
        for case in split_cases {
            let json = selection_rects_with_hints(&doc, case);
            let rects = rect_values(&json);
            let expected_page = u64::from(case.start_page_hint);
            let width = page_width(&doc, case.start_page_hint);
            let valid = !rects.is_empty()
                && rects.iter().all(|rect| {
                    let page_matches = rect["pageIndex"].as_u64() == Some(expected_page);
                    let x = rect["x"].as_f64().unwrap_or(f64::INFINITY);
                    let rect_width = rect["width"].as_f64().unwrap_or(f64::INFINITY);
                    page_matches && x >= -0.5 && x + rect_width <= width + 0.5
                });
            if !valid {
                failures.push(format!(
                    "{file_name}: para {} expected page {} within width {width}, got {json}",
                    case.start_para, case.start_page_hint
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "#2215 RED — getSelectionRectsInCellEx still ignores page hints:\n{}",
        failures.join("\n")
    );
}

#[test]
fn issue_2215_split_paragraph_cross_page_hints_keep_cursor_pairs_on_each_page() {
    let split_cross_cases = [
        SelectionCase {
            start_para: 17,
            start_offset: 162,
            end_para: 17,
            end_offset: 170,
            start_page_hint: 0,
            end_page_hint: 1,
        },
        SelectionCase {
            start_para: 1277,
            start_offset: 74,
            end_para: 1277,
            end_offset: 82,
            start_page_hint: 55,
            end_page_hint: 56,
        },
        SelectionCase {
            start_para: 2499,
            start_offset: 110,
            end_para: 2499,
            end_offset: 118,
            start_page_hint: 113,
            end_page_hint: 114,
        },
    ];

    let mut format_oracles = Vec::new();
    for file_name in [
        "issue1949_giant_cell_nested_tables_perf.hwp",
        "issue1949_giant_cell_nested_tables_perf.hwpx",
    ] {
        let mut doc = load_sample(file_name);
        let mut case_oracles = Vec::new();
        for case in split_cross_cases {
            let json = selection_rects_with_hints(&doc, case);
            let rects = rect_values(&json);
            assert_eq!(
                rect_pages(&rects),
                vec![
                    u64::from(case.start_page_hint),
                    u64::from(case.end_page_hint),
                ],
                "{file_name}: split cross-page selection must render on both endpoint pages; got {json}"
            );
            for rect in &rects {
                let page = rect["pageIndex"].as_u64().expect("rect page") as u32;
                let x = rect["x"].as_f64().expect("rect x");
                let width = rect["width"].as_f64().expect("rect width");
                let page_width = page_width(&doc, page);
                assert!(
                    x >= -0.5 && x + width <= page_width + 0.5,
                    "{file_name}: split cross-page rect must stay within page {page} width {page_width}; got {json}"
                );
            }
            let copied = copied_text(&mut doc, case);
            case_oracles.push((json, copied));
        }
        format_oracles.push(case_oracles);
    }

    assert_eq!(
        format_oracles[0], format_oracles[1],
        "HWP/HWPX split cross-page rect and copy bytes must match"
    );
}
