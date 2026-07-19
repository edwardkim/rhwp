//! Issue #2193 Stage 2 repeated native input/pagination baseline.
//!
//! 진단 전용 ignored probe다. fresh HWP/HWPX 문서에서 Studio와 같은 sequential single-key
//! 입력을 준비한 뒤 stable 입력과 첫 cell-flow boundary 입력의 mutation, tree/cursor query,
//! explicit full-pagination flush를 같은 순서로 반복 측정한다.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Map, Value};

const HWP_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const HWPX_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

const SECTION: usize = 0;
const PARENT_PARAGRAPH: usize = 0;
const TABLE_CONTROL: usize = 2;
const CELL: usize = 2;
const TARGET_PARAGRAPH: usize = 5;
const INSERT_OFFSET: usize = 130;
const CELL_PATH: &str = r#"[{"controlIndex":2,"cellIndex":2,"cellParaIndex":5}]"#;

const DEFAULT_WARMUPS: usize = 1;
const DEFAULT_REPEATS: usize = 10;
const MAX_REPEATS: usize = 100;

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    target_input: usize,
    warm_before_target: bool,
}

const CASES: [Case; 4] = [
    Case {
        name: "cold_stable_28",
        target_input: 28,
        warm_before_target: false,
    },
    Case {
        name: "warm_stable_28",
        target_input: 28,
        warm_before_target: true,
    },
    Case {
        name: "cold_boundary_44",
        target_input: 44,
        warm_before_target: false,
    },
    Case {
        name: "warm_boundary_44",
        target_input: 44,
        warm_before_target: true,
    },
];

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn load_sample(relative: &str) -> HwpDocument {
    let bytes = fs::read(manifest_path(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {relative}: {error}"))
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

fn line_starts(doc: &HwpDocument) -> Vec<usize> {
    target_paragraph(doc)
        .line_segs
        .iter()
        .map(|segment| segment.text_start as usize)
        .collect()
}

fn insert_one(doc: &mut HwpDocument, inserted: usize) -> Value {
    let raw = doc
        .insert_text_in_cell_native_deferred_pagination(
            SECTION,
            PARENT_PARAGRAPH,
            TABLE_CONTROL,
            CELL,
            TARGET_PARAGRAPH,
            INSERT_OFFSET + inserted,
            "1",
        )
        .expect("sequential deferred insert");
    serde_json::from_str(&raw).expect("cell edit result json")
}

fn target_tree_end(node: &RenderNode, end: &mut usize) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let (Some(start), Some(context)) = (run.char_start, run.cell_context.as_ref()) {
            let target = context.parent_para_index == PARENT_PARAGRAPH
                && context.path.len() == 1
                && context.path.first().is_some_and(|entry| {
                    entry.control_index == TABLE_CONTROL
                        && entry.cell_index == CELL
                        && entry.cell_para_index == TARGET_PARAGRAPH
                });
            if target {
                *end = (*end).max(start + run.text.encode_utf16().count());
            }
        }
    }
    for child in &node.children {
        target_tree_end(child, end);
    }
}

fn measure_tree(doc: &HwpDocument, expected_end: usize) -> f64 {
    let started = Instant::now();
    let tree = doc
        .build_page_render_tree(0)
        .expect("build page zero render tree");
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut actual_end = 0;
    target_tree_end(&tree.root, &mut actual_end);
    assert_eq!(actual_end, expected_end, "target render tree must be exact");
    elapsed_ms
}

fn measure_cursor(doc: &HwpDocument, offset: usize) -> f64 {
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
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    let value: Value = serde_json::from_str(&raw).expect("cursor rect json");
    assert_eq!(value["pageIndex"], 0, "target cursor page");
    assert_eq!(value["cellOverflowed"], false, "target cell overflow");
    elapsed_ms
}

fn run_sample(format: &str, sample_path: &str, case: Case, run: usize) -> Value {
    let load_started = Instant::now();
    let mut doc = load_sample(sample_path);
    let load_ms = load_started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(
        doc.page_count(),
        115,
        "{format}/{} initial pages",
        case.name
    );
    assert_eq!(
        target_paragraph(&doc).text.encode_utf16().count(),
        INSERT_OFFSET,
        "{format}/{} initial target length",
        case.name
    );
    let initial_next_vpos = target_next_vpos(&doc);

    let prepare_started = Instant::now();
    for inserted in 0..case.target_input - 1 {
        let result = insert_one(&mut doc, inserted);
        assert_eq!(result["cellFlowChanged"], false);
    }
    let prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.0;

    let warmup_ms = if case.warm_before_target {
        let started = Instant::now();
        let expected_end = INSERT_OFFSET + case.target_input - 1;
        measure_tree(&doc, expected_end);
        measure_cursor(&doc, expected_end);
        started.elapsed().as_secs_f64() * 1000.0
    } else {
        0.0
    };

    let mutation_started = Instant::now();
    let result = insert_one(&mut doc, case.target_input - 1);
    let mutation_ms = mutation_started.elapsed().as_secs_f64() * 1000.0;
    let expected_flow_changed = case.target_input == 44;
    assert_eq!(
        result["cellFlowChanged"], expected_flow_changed,
        "{format}/{} target flow signal",
        case.name
    );
    let expected_end = INSERT_OFFSET + case.target_input;
    assert_eq!(result["charOffset"], expected_end);
    assert_eq!(
        target_paragraph(&doc).text.encode_utf16().count(),
        expected_end
    );

    let pre_flush_tree_ms = measure_tree(&doc, expected_end);
    let pre_flush_cursor_ms = measure_cursor(&doc, expected_end);

    let flush_started = Instant::now();
    doc.flush_deferred_pagination()
        .expect("explicit deferred pagination flush");
    let flush_ms = flush_started.elapsed().as_secs_f64() * 1000.0;

    let post_flush_tree_ms = measure_tree(&doc, expected_end);
    let post_flush_cursor_ms = measure_cursor(&doc, expected_end);
    let expected_lines = if expected_flow_changed {
        vec![0, 44, 84, 122, 129]
    } else {
        vec![0, 44, 84, 122]
    };
    let expected_next_vpos = initial_next_vpos + if expected_flow_changed { 1920 } else { 0 };
    assert_eq!(
        line_starts(&doc),
        expected_lines,
        "{format}/{} lines",
        case.name
    );
    assert_eq!(
        target_next_vpos(&doc),
        expected_next_vpos,
        "{format}/{} next paragraph vpos",
        case.name
    );
    assert_eq!(doc.page_count(), 115, "{format}/{} final pages", case.name);

    json!({
        "run": run,
        "loadMs": load_ms,
        "prepareMs": prepare_ms,
        "warmupMs": warmup_ms,
        "mutationMs": mutation_ms,
        "preFlushTreeMs": pre_flush_tree_ms,
        "preFlushCursorMs": pre_flush_cursor_ms,
        "flushMs": flush_ms,
        "postFlushTreeMs": post_flush_tree_ms,
        "postFlushCursorMs": post_flush_cursor_ms,
        "accuracy": {
            "cellFlowChanged": expected_flow_changed,
            "targetLength": expected_end,
            "lineStarts": line_starts(&doc),
            "nextParagraphVpos": target_next_vpos(&doc),
            "pageCount": doc.page_count(),
        }
    })
}

fn percentile(values: &[f64], quantile: f64) -> f64 {
    assert!(!values.is_empty());
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = ((sorted.len() as f64 * quantile).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[index]
}

fn summarize(samples: &[Value]) -> Value {
    let phases = [
        "loadMs",
        "prepareMs",
        "warmupMs",
        "mutationMs",
        "preFlushTreeMs",
        "preFlushCursorMs",
        "flushMs",
        "postFlushTreeMs",
        "postFlushCursorMs",
    ];
    let mut summary = Map::new();
    for phase in phases {
        let values = samples
            .iter()
            .map(|sample| sample[phase].as_f64().expect("timing number"))
            .collect::<Vec<_>>();
        summary.insert(
            phase.to_string(),
            json!({
                "count": values.len(),
                "p50Ms": percentile(&values, 0.50),
                "p95Ms": percentile(&values, 0.95),
                "maxMs": percentile(&values, 1.0),
            }),
        );
    }
    Value::Object(summary)
}

fn env_count(name: &str, default: usize) -> usize {
    let value = std::env::var(name)
        .ok()
        .map(|raw| {
            raw.parse::<usize>()
                .unwrap_or_else(|_| panic!("{name} must be an integer"))
        })
        .unwrap_or(default);
    assert!(
        (1..=MAX_REPEATS).contains(&value),
        "{name} must be 1..={MAX_REPEATS}"
    );
    value
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn sha256(path: &Path) -> Option<String> {
    if let Some(output) = command_output("shasum", &["-a", "256", path.to_str()?]) {
        return output.split_whitespace().next().map(str::to_string);
    }
    command_output("sha256sum", &[path.to_str()?])?
        .split_whitespace()
        .next()
        .map(str::to_string)
}

fn fixture_metadata(relative: &str) -> Value {
    let path = manifest_path(relative);
    json!({
        "path": relative,
        "size": fs::metadata(&path).expect("fixture metadata").len(),
        "sha256": sha256(&path),
    })
}

#[test]
#[ignore = "performance diagnostic; run explicitly with --ignored --nocapture"]
fn issue_2193_repeated_input_pagination_baseline() {
    let warmups = env_count("RHWP_2193_WARMUPS", DEFAULT_WARMUPS);
    let repeats = env_count("RHWP_2193_REPEATS", DEFAULT_REPEATS);
    let format_filter = std::env::var("RHWP_2193_FORMAT").ok();
    let case_filter = std::env::var("RHWP_2193_CASE").ok();
    let formats = [("hwp", HWP_SAMPLE), ("hwpx", HWPX_SAMPLE)];
    let mut cases = Vec::new();

    for (format, sample_path) in formats {
        if format_filter
            .as_deref()
            .is_some_and(|filter| filter != format)
        {
            continue;
        }
        for case in CASES {
            if case_filter
                .as_deref()
                .is_some_and(|filter| filter != case.name)
            {
                continue;
            }
            eprintln!("#2193 warmup: {format}/{} x{warmups}", case.name);
            for run in 1..=warmups {
                let _ = run_sample(format, sample_path, case, run);
            }

            eprintln!("#2193 measure: {format}/{} x{repeats}", case.name);
            let samples = (1..=repeats)
                .map(|run| run_sample(format, sample_path, case, run))
                .collect::<Vec<_>>();
            let summary = summarize(&samples);
            eprintln!(
                "  mutation p50={:.3}ms p95={:.3}ms; flush p50={:.3}ms p95={:.3}ms",
                summary["mutationMs"]["p50Ms"]
                    .as_f64()
                    .expect("mutation p50"),
                summary["mutationMs"]["p95Ms"]
                    .as_f64()
                    .expect("mutation p95"),
                summary["flushMs"]["p50Ms"].as_f64().expect("flush p50"),
                summary["flushMs"]["p95Ms"].as_f64().expect("flush p95"),
            );
            cases.push(json!({
                "format": format,
                "case": case.name,
                "targetInput": case.target_input,
                "warmBeforeTarget": case.warm_before_target,
                "summary": summary,
                "samples": samples,
            }));
        }
    }
    assert!(
        !cases.is_empty(),
        "RHWP_2193_FORMAT/RHWP_2193_CASE filters selected no cases"
    );

    let git_head = command_output("git", &["rev-parse", "HEAD"]);
    let git_dirty =
        command_output("git", &["status", "--porcelain"]).map(|status| !status.is_empty());
    let output = json!({
        "issue": 2193,
        "stage": 2,
        "kind": "native-repeated-baseline",
        "percentileMethod": "nearest-rank: ceil(count * quantile) - 1",
        "environment": {
            "gitHead": git_head,
            "gitDirty": git_dirty,
            "os": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "rustc": command_output("rustc", &["--version"]),
            "cargo": command_output("cargo", &["--version"]),
            "profile": "release-test",
            "warmups": warmups,
            "repeats": repeats,
            "formatFilter": format_filter,
            "caseFilter": case_filter,
        },
        "fixtures": [fixture_metadata(HWP_SAMPLE), fixture_metadata(HWPX_SAMPLE)],
        "cases": cases,
    });
    let output_name =
        std::env::var("RHWP_2193_OUTPUT_NAME").unwrap_or_else(|_| "native-baseline".to_string());
    assert!(
        !output_name.is_empty()
            && output_name.chars().all(
                |character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            ),
        "RHWP_2193_OUTPUT_NAME must contain only ASCII letters, digits, '-' or '_'"
    );
    let output_path = manifest_path(&format!("output/poc/task2193/stage2/{output_name}.json"));
    fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create output directory");
    fs::write(
        &output_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&output).expect("serialize output")
        ),
    )
    .expect("write native baseline");
    eprintln!("#2193 native baseline written: {}", output_path.display());
}
