//! B2/B3 native block contracts. Roundtrip success is not Hancom acceptance.
use rhwp::{
    document_core::{DocumentCore, ParagraphBlockLimits, RepeatParagraphBlockRequest},
    model::{
        control::Control,
        paragraph::{ColumnBreakType, Paragraph},
    },
};
use serde_json::{json, Value};
use std::{collections::BTreeSet, time::Instant};

fn request(pi: usize, count: usize) -> RepeatParagraphBlockRequest {
    RepeatParagraphBlockRequest {
        section_index: 0,
        source_start: pi,
        source_end: pi + 1,
        insert_before: pi + 1,
        count,
        limits: ParagraphBlockLimits::default(),
    }
}
fn load(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture")).unwrap()
}

// Explicit save contract: paragraph text/order (including empty paragraphs),
// table/cell topology, and stored paragraph/run style references. Not IrDiff-0.
fn content(value: &Value, ledger: &mut Vec<Value>) {
    match value {
        Value::Object(o) => {
            if o.contains_key("char_count") && o.contains_key("line_segs") {
                ledger.push(
                    json!({"paragraph": o["text"], "paraShape": o["para_shape_id"],
                    "style": o["style_id"], "runs": o["char_shapes"]}),
                );
            }
            if o.contains_key("row_count") && o.contains_key("cells") {
                ledger.push(
                    json!({"tableRows":o["row_count"], "tableCols":o["col_count"],
                    "cells":o["cells"].as_array().unwrap().iter().map(|c| json!([
                        c["row"].as_u64().unwrap(), c["col"].as_u64().unwrap(),
                        c["row_span"].as_u64().unwrap(), c["col_span"].as_u64().unwrap()
                    ])).collect::<Vec<_>>()}),
                );
            }
            for child in o.values() {
                content(child, ledger);
            }
        }
        Value::Array(a) => {
            for child in a {
                content(child, ledger);
            }
        }
        _ => {}
    }
}
fn paragraph_content(p: &Paragraph) -> Vec<Value> {
    let mut result = Vec::new();
    content(&serde_json::to_value(p).unwrap(), &mut result);
    result
}
fn table_ids(c: &DocumentCore) -> Vec<u32> {
    c.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|x| {
            if let Control::Table(t) = x {
                Some(t.common.instance_id)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn real_blocks_save_both_formats_without_losing_paragraphs_or_table_ids() {
    for (path, pi) in [
        ("samples/hwp_table_test.hwp", 3),
        ("samples/rnote/labnote-001.hwp", 12),
    ] {
        for at in [
            0,
            pi,
            pi + 1,
            load(path).document().sections[0].paragraphs.len(),
        ] {
            let mut c = load(path);
            let mut r = request(pi, 2);
            r.insert_before = at;
            c.repeat_paragraph_block_native(&r).unwrap();
            let expected: Vec<_> = c.document().sections[0]
                .paragraphs
                .iter()
                .map(paragraph_content)
                .collect();
            let expected_ids = table_ids(&c);
            for (format, output) in [
                ("hwp", c.export_hwp_native().unwrap()),
                ("hwpx", c.export_hwpx_native().unwrap()),
            ] {
                let reopened = DocumentCore::from_bytes(&output).unwrap();
                let actual: Vec<_> = reopened.document().sections[0]
                    .paragraphs
                    .iter()
                    .map(paragraph_content)
                    .collect();
                assert_eq!(
                    actual.len(),
                    expected.len(),
                    "{path} at={at} format={format}"
                );
                for (pi, (a, b)) in actual.iter().zip(&expected).enumerate() {
                    assert_eq!(a, b, "{path} at={at} format={format} pi={pi}");
                }
                assert_eq!(table_ids(&reopened), expected_ids, "{path} {format}");
            }
        }
    }
}

#[test]
fn parsed_hwpx_is_a_derived_format_probe_not_an_independent_oracle() {
    let original = load("samples/hwp_table_test.hwp");
    let mut c = DocumentCore::from_bytes(&original.export_hwpx_native().unwrap()).unwrap();
    c.repeat_paragraph_block_native(&request(3, 2)).unwrap();
    let ids = table_ids(&c);
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(table_ids(&reopened), ids);
        assert_eq!(
            reopened.document().sections[0].paragraphs.len(),
            original.document().sections[0].paragraphs.len() + 2
        );
    }
}

#[test]
fn empty_paragraphs_explicit_breaks_and_deleting_one_copy_preserve_neighbors() {
    for empties in 0..=2 {
        for break_type in [
            ColumnBreakType::None,
            ColumnBreakType::Page,
            ColumnBreakType::Column,
        ] {
            let mut c = load("samples/hwp_table_test.hwp");
            let mut block = vec![Paragraph {
                text: "head".into(),
                ..Default::default()
            }];
            block.extend((0..empties).map(|_| Paragraph::default()));
            block.push(Paragraph {
                text: "tail".into(),
                column_type: break_type,
                ..Default::default()
            });
            c.document_mut().sections[0]
                .paragraphs
                .extend(block.clone());
            let len = c.document().sections[0].paragraphs.len();
            let r = RepeatParagraphBlockRequest {
                source_start: len - block.len(),
                source_end: len,
                insert_before: len,
                ..request(0, 2)
            };
            c.repeat_paragraph_block_native(&r).unwrap();
            for index in 0..3 {
                let start = r.source_start + index * block.len();
                for (a, b) in c.document().sections[0].paragraphs[start..start + block.len()]
                    .iter()
                    .zip(&block)
                {
                    assert_eq!(paragraph_content(a), paragraph_content(b));
                    assert_eq!(a.column_type, b.column_type);
                }
            }
            for _ in 0..block.len() {
                c.delete_paragraph_native(0, len).unwrap();
            }
            assert_eq!(c.document().sections[0].paragraphs.len(), len + block.len());
            assert_eq!(c.document().sections[0].paragraphs[len].text, "head");
        }
    }
}

fn memory_kib(key: &str) -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix(key)?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

/// One fixture/count per process. Outputs are opt-in, outside ordinary CI.
#[test]
#[ignore = "manual cost/artifact probe; see Stage 6 for exact invocation"]
fn block_cost_and_artifacts() {
    let fixture = std::env::var("RHWP_3587_FIXTURE").expect("table, labnote or textbox");
    let (path, pi) = match fixture.as_str() {
        "table" => ("samples/hwp_table_test.hwp", 3),
        "labnote" => ("samples/rnote/labnote-001.hwp", 12),
        "textbox" => ("samples/table-in-tbox.hwp", 0),
        _ => panic!("unknown fixture"),
    };
    let count: usize = std::env::var("RHWP_3587_COUNT").unwrap().parse().unwrap();
    assert!([1, 10, 100].contains(&count));
    let mut c = load(path);
    let r = request(pi, count);
    let rss_before = memory_kib("VmRSS:");
    let hwm_before = memory_kib("VmHWM:");
    let began = Instant::now();
    let budget = c.validate_paragraph_block_native(&r);
    let preflight_us = began.elapsed().as_micros();
    let began = Instant::now();
    let result = c.repeat_paragraph_block_native(&r);
    let repeat_us = began.elapsed().as_micros();
    let measurement = json!({"fixture":fixture,"path":path,"request":r,"budget":budget,
        "repeatOk":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),
        "preflightUs":preflight_us,"repeatUsIncludingPreflightAndPagination":repeat_us,
        "rssBeforeKiB":rss_before,"rssAfterKiB":memory_kib("VmRSS:"),
        "processHwmBeforeKiB":hwm_before,"processHwmAfterKiB":memory_kib("VmHWM:"),
        "bodyParagraphs":c.document().sections[0].paragraphs.len(),"pages":c.page_count()});
    println!("BLOCK_COST {measurement}");
    if fixture == "textbox" {
        assert!(result.is_err());
        return;
    }
    let result = result.unwrap();
    assert_eq!(result.copies.len(), count);
    let new_ids: Vec<_> = c.document().sections[0].paragraphs[result.inserted.clone()]
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|x| {
            if let Control::Table(t) = x {
                Some(t.common.instance_id)
            } else {
                None
            }
        })
        .collect();
    assert!(!new_ids.is_empty());
    assert_eq!(new_ids.iter().collect::<BTreeSet<_>>().len(), new_ids.len());
    if let Ok(directory) = std::env::var("RHWP_3587_OUTPUT") {
        assert_eq!(count, 1, "artifact export restricted to one-copy probe");
        let output = std::path::PathBuf::from(directory);
        std::fs::create_dir(&output).expect("new output directory required");
        std::fs::write(
            output.join("measurement.json"),
            serde_json::to_vec_pretty(&measurement).unwrap(),
        )
        .unwrap();
        std::fs::write(
            output.join("mapping.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        std::fs::copy(path, output.join("original.hwp")).unwrap();
        for (name, bytes) in [
            ("repeated.hwp", c.export_hwp_native().unwrap()),
            ("repeated.hwpx", c.export_hwpx_native().unwrap()),
        ] {
            std::fs::write(output.join(name), bytes).unwrap();
        }
        for (prefix, core) in [("original", load(path)), ("repeated", c)] {
            for page in 0..core.page_count() {
                std::fs::write(
                    output.join(format!("{prefix}-{:03}.svg", page + 1)),
                    core.render_page_svg_native(page).unwrap(),
                )
                .unwrap();
            }
        }
    }
}
