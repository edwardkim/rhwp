//! Issue #2214 Stage 2 latest-devel diagnostic matrix.
//!
//! 진단 전용 ignored probe. production 계약으로 승격하기 전에 cold/warm layout-cache,
//! batch/sequential edit, direct/path cursor 관찰 순서를 fresh document별로 분리한다.

use std::fs;
use std::path::Path;
use std::time::Instant;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

const HWP_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const HWPX_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

const SECTION: usize = 0;
const PARENT_PARAGRAPH: usize = 0;
const TABLE_CONTROL: usize = 2;
const CELL: usize = 2;
const TARGET_PARAGRAPH: usize = 5;
const INSERT_OFFSET: usize = 130;
const CELL_PATH: &str = r#"[{"controlIndex":2,"cellIndex":2,"cellParaIndex":5}]"#;

#[derive(Clone, Copy, Debug)]
enum EditMode {
    Batch,
    Sequential,
}

#[derive(Clone, Copy, Debug)]
enum WarmMode {
    Cold,
    BeforeEdit,
    After(usize),
    EveryEdit,
}

#[derive(Clone, Copy, Debug)]
enum QueryMode {
    Tree,
    Direct,
    PathNear,
}

#[derive(Clone, Copy, Debug)]
struct Case {
    name: &'static str,
    count: usize,
    edit: EditMode,
    warm: WarmMode,
    query: QueryMode,
}

fn load_sample(relative_path: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {relative_path}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {relative_path}: {e}"))
}

fn target_paragraph(core: &DocumentCore) -> &rhwp::model::paragraph::Paragraph {
    match &core.document().sections[SECTION].paragraphs[PARENT_PARAGRAPH].controls[TABLE_CONTROL] {
        Control::Table(table) => &table.cells[CELL].paragraphs[TARGET_PARAGRAPH],
        other => panic!("target control is not a table: {other:?}"),
    }
}

fn target_next_vpos(core: &DocumentCore) -> i32 {
    match &core.document().sections[SECTION].paragraphs[PARENT_PARAGRAPH].controls[TABLE_CONTROL] {
        Control::Table(table) => {
            table.cells[CELL].paragraphs[TARGET_PARAGRAPH + 1]
                .line_segs
                .first()
                .expect("next target paragraph line seg")
                .vertical_pos
        }
        other => panic!("target control is not a table: {other:?}"),
    }
}

fn model_state(doc: &HwpDocument) -> Value {
    let para = target_paragraph(doc);
    json!({
        "length": para.text.chars().count(),
        "tail": para.text.chars().rev().take(60).collect::<Vec<_>>().into_iter().rev().collect::<String>(),
        "lineStarts": para.line_segs.iter().map(|seg| seg.text_start).collect::<Vec<_>>(),
        "lineVpos": para.line_segs.iter().map(|seg| seg.vertical_pos).collect::<Vec<_>>(),
        "nextParagraphVpos": target_next_vpos(doc),
        "pageCount": doc.page_count(),
    })
}

fn matches_target_run(node: &RenderNode) -> Option<(usize, usize, String, f64)> {
    let RenderNodeType::TextRun(run) = &node.node_type else {
        return None;
    };
    let ctx = run.cell_context.as_ref()?;
    let entry = ctx.path.first()?;
    if ctx.parent_para_index != PARENT_PARAGRAPH
        || entry.control_index != TABLE_CONTROL
        || entry.cell_index != CELL
        || entry.cell_para_index != TARGET_PARAGRAPH
    {
        return None;
    }
    let start = run.char_start?;
    let end = start + run.text.chars().count();
    Some((start, end, run.text.clone(), node.bbox.y))
}

fn collect_target_runs(node: &RenderNode, out: &mut Vec<(usize, usize, String, f64)>) {
    if let Some(run) = matches_target_run(node) {
        out.push(run);
    }
    for child in &node.children {
        collect_target_runs(child, out);
    }
}

fn tree_state(doc: &HwpDocument) -> Value {
    let started = Instant::now();
    let tree = doc
        .build_page_render_tree(0)
        .expect("build page zero render tree");
    let elapsed = started.elapsed();
    let mut runs = Vec::new();
    collect_target_runs(&tree.root, &mut runs);
    let max_char = runs.iter().map(|(_, end, _, _)| *end).max().unwrap_or(0);
    json!({
        "elapsedMs": elapsed.as_secs_f64() * 1000.0,
        "maxChar": max_char,
        "exact": max_char >= target_paragraph(doc).text.chars().count(),
        "runs": runs.iter().map(|(start, end, text, y)| json!({
            "start": start,
            "end": end,
            "text": text,
            "y": y,
        })).collect::<Vec<_>>(),
    })
}

fn partial_table_cut(doc: &HwpDocument) -> String {
    doc.dump_page_items(Some(0))
        .lines()
        .find(|line| line.contains("PartialTable") && line.contains("pi=0 ci=2"))
        .expect("target PartialTable pi=0 ci=2")
        .trim()
        .to_string()
}

fn direct_rect(doc: &HwpDocument, offset: usize) -> Value {
    let started = Instant::now();
    let raw = doc
        .get_cursor_rect_in_cell_native(
            SECTION,
            PARENT_PARAGRAPH,
            TABLE_CONTROL,
            CELL,
            TARGET_PARAGRAPH,
            offset,
        )
        .expect("direct cursor rect");
    let elapsed = started.elapsed();
    json!({
        "elapsedMs": elapsed.as_secs_f64() * 1000.0,
        "value": serde_json::from_str::<Value>(&raw).expect("direct rect json"),
    })
}

fn path_rect(doc: &HwpDocument, offset: usize) -> Value {
    let started = Instant::now();
    let raw = doc
        .get_cursor_rect_by_path_near(
            SECTION as u32,
            PARENT_PARAGRAPH as u32,
            CELL_PATH,
            offset as u32,
            0,
        )
        .expect("path-near cursor rect");
    let elapsed = started.elapsed();
    json!({
        "elapsedMs": elapsed.as_secs_f64() * 1000.0,
        "value": serde_json::from_str::<Value>(&raw).expect("path rect json"),
    })
}

fn observe_for_warm(doc: &HwpDocument, offset: usize) {
    let _ = doc.build_page_render_tree(0).expect("warm page zero tree");
    let _ = doc
        .get_cursor_rect_in_cell_native(
            SECTION,
            PARENT_PARAGRAPH,
            TABLE_CONTROL,
            CELL,
            TARGET_PARAGRAPH,
            offset,
        )
        .expect("warm direct cursor");
}

fn insert_batch(doc: &mut HwpDocument, count: usize) {
    let text = "1".repeat(count);
    doc.insert_text_in_cell_native_deferred_pagination(
        SECTION,
        PARENT_PARAGRAPH,
        TABLE_CONTROL,
        CELL,
        TARGET_PARAGRAPH,
        INSERT_OFFSET,
        &text,
    )
    .expect("batch deferred insert");
}

fn insert_one(doc: &mut HwpDocument, index: usize) {
    doc.insert_text_in_cell_native_deferred_pagination(
        SECTION,
        PARENT_PARAGRAPH,
        TABLE_CONTROL,
        CELL,
        TARGET_PARAGRAPH,
        INSERT_OFFSET + index,
        "1",
    )
    .expect("sequential deferred insert");
}

fn apply_edits(doc: &mut HwpDocument, case: Case) {
    if matches!(case.warm, WarmMode::BeforeEdit) {
        observe_for_warm(doc, INSERT_OFFSET);
    }
    match case.edit {
        EditMode::Batch => insert_batch(doc, case.count),
        EditMode::Sequential => {
            for i in 0..case.count {
                insert_one(doc, i);
                let offset = INSERT_OFFSET + i + 1;
                match case.warm {
                    WarmMode::After(after) if i + 1 == after => observe_for_warm(doc, offset),
                    WarmMode::EveryEdit => observe_for_warm(doc, offset),
                    _ => {}
                }
            }
        }
    }
}

fn run_case(format: &str, sample: &str, case: Case) -> Value {
    let mut doc = load_sample(sample);
    assert_eq!(
        doc.page_count(),
        115,
        "{format}/{} initial pages",
        case.name
    );
    apply_edits(&mut doc, case);

    let model_before = model_state(&doc);
    let cut_before = partial_table_cut(&doc);
    let final_offset = INSERT_OFFSET + case.count;
    let first_query = match case.query {
        QueryMode::Tree => tree_state(&doc),
        QueryMode::Direct => direct_rect(&doc, final_offset),
        QueryMode::PathNear => path_rect(&doc, final_offset),
    };
    let tree_before = tree_state(&doc);

    let flush_started = Instant::now();
    doc.flush_deferred_pagination()
        .expect("explicit deferred pagination flush");
    let flush_ms = flush_started.elapsed().as_secs_f64() * 1000.0;
    let model_after = model_state(&doc);
    let cut_after = partial_table_cut(&doc);
    let oracle_query = match case.query {
        QueryMode::Tree => tree_state(&doc),
        QueryMode::Direct => direct_rect(&doc, final_offset),
        QueryMode::PathNear => path_rect(&doc, final_offset),
    };
    let tree_after = tree_state(&doc);

    json!({
        "format": format,
        "case": case.name,
        "count": case.count,
        "edit": format!("{:?}", case.edit),
        "warm": format!("{:?}", case.warm),
        "query": format!("{:?}", case.query),
        "beforeFlush": {
            "model": model_before,
            "partialTable": cut_before,
            "firstQuery": first_query,
            "tree": tree_before,
        },
        "flushMs": flush_ms,
        "afterFlush": {
            "model": model_after,
            "partialTable": cut_after,
            "oracleQuery": oracle_query,
            "tree": tree_after,
        }
    })
}

#[test]
#[ignore = "diagnostic matrix; run explicitly with --ignored --nocapture"]
fn issue_2214_latest_devel_cache_matrix_probe() {
    let cases = [
        Case {
            name: "cold_batch28_tree",
            count: 28,
            edit: EditMode::Batch,
            warm: WarmMode::Cold,
            query: QueryMode::Tree,
        },
        Case {
            name: "cold_batch44_tree",
            count: 44,
            edit: EditMode::Batch,
            warm: WarmMode::Cold,
            query: QueryMode::Tree,
        },
        Case {
            name: "cold_seq44_tree",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::Cold,
            query: QueryMode::Tree,
        },
        Case {
            name: "prewarm_batch44_tree",
            count: 44,
            edit: EditMode::Batch,
            warm: WarmMode::BeforeEdit,
            query: QueryMode::Tree,
        },
        Case {
            name: "prewarm_seq44_tree",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::BeforeEdit,
            query: QueryMode::Tree,
        },
        Case {
            name: "mid30_seq44_tree",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::After(30),
            query: QueryMode::Tree,
        },
        Case {
            name: "each_seq44_tree",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::EveryEdit,
            query: QueryMode::Tree,
        },
        Case {
            name: "cold_seq44_direct",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::Cold,
            query: QueryMode::Direct,
        },
        Case {
            name: "prewarm_seq44_direct",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::BeforeEdit,
            query: QueryMode::Direct,
        },
        Case {
            name: "cold_seq44_path",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::Cold,
            query: QueryMode::PathNear,
        },
        Case {
            name: "prewarm_seq44_path",
            count: 44,
            edit: EditMode::Sequential,
            warm: WarmMode::BeforeEdit,
            query: QueryMode::PathNear,
        },
        Case {
            name: "cold_seq50_direct",
            count: 50,
            edit: EditMode::Sequential,
            warm: WarmMode::Cold,
            query: QueryMode::Direct,
        },
        Case {
            name: "mid30_seq50_direct",
            count: 50,
            edit: EditMode::Sequential,
            warm: WarmMode::After(30),
            query: QueryMode::Direct,
        },
        Case {
            name: "cold_seq50_path",
            count: 50,
            edit: EditMode::Sequential,
            warm: WarmMode::Cold,
            query: QueryMode::PathNear,
        },
        Case {
            name: "mid30_seq50_path",
            count: 50,
            edit: EditMode::Sequential,
            warm: WarmMode::After(30),
            query: QueryMode::PathNear,
        },
    ];

    let formats = [("hwp", HWP_SAMPLE), ("hwpx", HWPX_SAMPLE)];
    let mut records = Vec::new();
    for (format, sample) in formats {
        for case in cases {
            eprintln!("issue2214 matrix: {format}/{}", case.name);
            records.push(run_case(format, sample, case));
        }
    }

    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("output/poc/task2214/stage2/native-matrix.json");
    fs::create_dir_all(output.parent().expect("output parent")).expect("create output directory");
    fs::write(
        &output,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&records).expect("serialize matrix")
        ),
    )
    .expect("write native matrix");
    eprintln!("issue2214 matrix written: {}", output.display());
}
