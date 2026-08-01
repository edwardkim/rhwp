//! HWP5 inventory diff extraction.
//!
//! This compares two HWP files after converting both into the same
//! record-level inventory representation.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::hwp5_inventory::{build_inventory, Hwp5InventoryItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Jsonl,
    Markdown,
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "jsonl" | "json" => Ok(Self::Jsonl),
            "md" | "markdown" => Ok(Self::Markdown),
            other => Err(format!("지원하지 않는 format: {other} (jsonl|md)")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignMode {
    Index,
    Lcs,
}

impl AlignMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "index" => Ok(Self::Index),
            "lcs" => Ok(Self::Lcs),
            other => Err(format!("지원하지 않는 align mode: {other} (index|lcs)")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::Lcs => "lcs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportMode {
    Diff,
    Hints,
    Bundles,
    TableFields,
    TableProbePlan,
}

impl ReportMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "diff" => Ok(Self::Diff),
            "hints" => Ok(Self::Hints),
            "bundles" => Ok(Self::Bundles),
            "table-fields" | "table_fields" => Ok(Self::TableFields),
            "table-probe-plan" | "table_probe_plan" => Ok(Self::TableProbePlan),
            other => Err(format!(
                "지원하지 않는 report mode: {other} (diff|hints|bundles|table-fields|table-probe-plan)"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Diff => "diff",
            Self::Hints => "hints",
            Self::Bundles => "bundles",
            Self::TableFields => "table-fields",
            Self::TableProbePlan => "table-probe-plan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusMode {
    All,
    Table,
    Shape,
    Ctrl,
    Missing,
    DocInfo,
}

impl FocusMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "all" => Ok(Self::All),
            "table" => Ok(Self::Table),
            "shape" | "picture" | "pic" => Ok(Self::Shape),
            "ctrl" | "ctrl-header" => Ok(Self::Ctrl),
            "missing" => Ok(Self::Missing),
            "docinfo" => Ok(Self::DocInfo),
            other => Err(format!(
                "지원하지 않는 focus mode: {other} (all|table|shape|ctrl|missing|docinfo)"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Table => "table",
            Self::Shape => "shape",
            Self::Ctrl => "ctrl",
            Self::Missing => "missing",
            Self::DocInfo => "docinfo",
        }
    }
}

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    format: OutputFormat,
    align_mode: AlignMode,
    report_mode: ReportMode,
    focus_mode: FocusMode,
    window: usize,
    section: Option<u32>,
    out: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct Hwp5InventoryDiffRecord {
    record_uid: String,
    tag_id: u16,
    tag_name: String,
    size: u32,
    tuple_role: String,
    tuple_index: usize,
    control_id: Option<String>,
    control_name: Option<String>,
    scope_path: String,
    payload_hash: String,
}

#[derive(Debug, Serialize)]
struct Hwp5InventoryDiffItem {
    align_mode: String,
    alignment_status: String,
    diff_kind: String,
    key: String,
    stream_path: String,
    section: Option<u32>,
    record_index: usize,
    oracle_record_index: Option<usize>,
    generated_record_index: Option<usize>,
    oracle_record_uid: Option<String>,
    generated_record_uid: Option<String>,
    signature: String,
    changed_fields: Vec<String>,
    oracle: Option<Hwp5InventoryDiffRecord>,
    generated: Option<Hwp5InventoryDiffRecord>,
    note: Option<String>,
}

#[derive(Debug)]
struct Hwp5InventoryDiff {
    align_mode: AlignMode,
    oracle_path: String,
    generated_path: String,
    stats: AlignmentStats,
    items: Vec<Hwp5InventoryDiffItem>,
    oracle_items: Vec<Hwp5InventoryItem>,
    generated_items: Vec<Hwp5InventoryItem>,
}

#[derive(Debug, Default)]
struct AlignmentStats {
    matched: usize,
    changed: usize,
    missing: usize,
    extra: usize,
}

pub fn run(args: &[String]) -> i32 {
    // 종료 코드 계약(mydocs/manual/cli_commands.md): 명시적 `--help`/`-h` 만 성공(0)이고
    // 인자 누락·파싱 실패는 사용법 오류(2), 실행 실패는 런타임 오류(1)다.
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        print_usage();
        return 0;
    }
    if args.is_empty() {
        print_usage();
        return 2;
    }

    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            return 2;
        }
    };

    let diff = match build_diff(
        &options.oracle,
        &options.generated,
        options.section,
        options.align_mode,
    ) {
        Ok(diff) => diff,
        Err(error) => {
            eprintln!("오류: {error}");
            return 1;
        }
    };

    let rendered = match options.report_mode {
        ReportMode::Diff => match options.format {
            OutputFormat::Jsonl => render_jsonl(&diff),
            OutputFormat::Markdown => render_markdown(&diff),
        },
        ReportMode::Hints => render_hints_markdown(&diff),
        ReportMode::Bundles => render_bundles_markdown(&diff, options.focus_mode, options.window),
        ReportMode::TableFields => render_table_fields_markdown(&diff),
        ReportMode::TableProbePlan => render_table_probe_plan_markdown(&diff),
    };

    if let Some(out) = options.out {
        if let Some(parent) = out.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("오류: 출력 폴더 생성 실패 - {}: {error}", parent.display());
                return 1;
            }
        }
        if let Err(error) = fs::write(&out, rendered) {
            eprintln!("오류: 출력 파일 쓰기 실패 - {}: {error}", out.display());
            return 1;
        }
    } else {
        print!("{rendered}");
    }
    0
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut inputs: Vec<PathBuf> = Vec::new();
    let mut format = OutputFormat::Markdown;
    let mut align_mode = AlignMode::Index;
    let mut report_mode = ReportMode::Diff;
    let mut focus_mode = FocusMode::All;
    let mut window = 2usize;
    let mut section = None;
    let mut out = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--format 뒤에 값이 필요합니다".to_string())?;
                format = OutputFormat::parse(value)?;
                i += 2;
            }
            "--section" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--section 뒤에 번호가 필요합니다".to_string())?;
                section = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("section 번호가 올바르지 않습니다: {value}"))?,
                );
                i += 2;
            }
            "--align" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--align 뒤에 값이 필요합니다".to_string())?;
                align_mode = AlignMode::parse(value)?;
                i += 2;
            }
            "--report" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--report 뒤에 값이 필요합니다".to_string())?;
                report_mode = ReportMode::parse(value)?;
                i += 2;
            }
            "--focus" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--focus 뒤에 값이 필요합니다".to_string())?;
                focus_mode = FocusMode::parse(value)?;
                i += 2;
            }
            "--window" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--window 뒤에 숫자가 필요합니다".to_string())?;
                window = value
                    .parse::<usize>()
                    .map_err(|_| format!("window 값이 올바르지 않습니다: {value}"))?;
                i += 2;
            }
            "--out" | "-o" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--out 뒤에 경로가 필요합니다".to_string())?;
                out = Some(PathBuf::from(value));
                i += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {value}"));
            }
            value => {
                inputs.push(PathBuf::from(value));
                i += 1;
            }
        }
    }

    if inputs.len() != 2 {
        return Err("oracle HWP와 generated HWP 경로가 필요합니다".to_string());
    }

    Ok(Options {
        oracle: inputs[0].clone(),
        generated: inputs[1].clone(),
        format,
        align_mode,
        report_mode,
        focus_mode,
        window,
        section,
        out,
    })
}

fn print_usage() {
    // 사용법 안내는 stderr 로 나간다 — stdout 은 diff 데이터(jsonl/md) 전용이라
    // 섞이면 파이프로 받는 소비자가 파싱에서 깨진다.
    eprintln!("사용법:");
    eprintln!(
        "  rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> [--align index|lcs] [--report diff|hints|bundles|table-fields|table-probe-plan] [--focus all|table|shape|ctrl|missing|docinfo] [--window N] [--format jsonl|md] [--section N] [--out <path>]"
    );
}

fn build_diff(
    oracle_path: &Path,
    generated_path: &Path,
    section_filter: Option<u32>,
    align_mode: AlignMode,
) -> Result<Hwp5InventoryDiff, String> {
    let oracle = build_inventory(oracle_path, section_filter)?;
    let generated = build_inventory(generated_path, section_filter)?;

    let (items, stats) = match align_mode {
        AlignMode::Index => build_index_diff(&oracle.items, &generated.items),
        AlignMode::Lcs => build_lcs_diff(&oracle.items, &generated.items),
    };

    Ok(Hwp5InventoryDiff {
        align_mode,
        oracle_path: oracle.source_path,
        generated_path: generated.source_path,
        stats,
        items,
        oracle_items: oracle.items,
        generated_items: generated.items,
    })
}

fn build_index_diff(
    oracle_items: &[Hwp5InventoryItem],
    generated_items: &[Hwp5InventoryItem],
) -> (Vec<Hwp5InventoryDiffItem>, AlignmentStats) {
    let oracle_by_key = inventory_map(oracle_items);
    let generated_by_key = inventory_map(generated_items);
    let mut keys = BTreeSet::new();
    keys.extend(oracle_by_key.keys().cloned());
    keys.extend(generated_by_key.keys().cloned());

    let mut items = Vec::new();
    let mut stats = AlignmentStats::default();
    for key in keys {
        match (oracle_by_key.get(&key), generated_by_key.get(&key)) {
            (Some(oracle_item), Some(generated_item)) => {
                let before = items.len();
                append_index_changed_items(&mut items, &key, oracle_item, generated_item);
                if items.len() == before {
                    stats.matched += 1;
                } else {
                    stats.changed += items.len() - before;
                }
            }
            (Some(oracle_item), None) => {
                items.push(diff_item(
                    AlignMode::Index,
                    "missing",
                    "missing",
                    &key,
                    Some(oracle_item),
                    None,
                    Vec::new(),
                    Some("oracle에만 존재".to_string()),
                ));
                stats.missing += 1;
            }
            (None, Some(generated_item)) => {
                items.push(diff_item(
                    AlignMode::Index,
                    "extra",
                    "extra",
                    &key,
                    None,
                    Some(generated_item),
                    Vec::new(),
                    Some("generated에만 존재".to_string()),
                ));
                stats.extra += 1;
            }
            (None, None) => {}
        }
    }

    (items, stats)
}

fn build_lcs_diff(
    oracle_items: &[Hwp5InventoryItem],
    generated_items: &[Hwp5InventoryItem],
) -> (Vec<Hwp5InventoryDiffItem>, AlignmentStats) {
    let oracle_by_stream = group_by_stream(oracle_items);
    let generated_by_stream = group_by_stream(generated_items);
    let mut streams = BTreeSet::new();
    streams.extend(oracle_by_stream.keys().cloned());
    streams.extend(generated_by_stream.keys().cloned());

    let mut items = Vec::new();
    let mut stats = AlignmentStats::default();

    for stream in streams {
        let oracle_stream = oracle_by_stream.get(&stream).cloned().unwrap_or_default();
        let generated_stream = generated_by_stream
            .get(&stream)
            .cloned()
            .unwrap_or_default();
        let ops = lcs_alignment_ops(&oracle_stream, &generated_stream);

        for op in ops {
            match op {
                AlignmentOp::Pair(oracle_index, generated_index) => {
                    let oracle_item = oracle_stream[oracle_index];
                    let generated_item = generated_stream[generated_index];
                    let changed_fields = changed_fields_lcs(oracle_item, generated_item);
                    if changed_fields.is_empty() {
                        stats.matched += 1;
                    } else {
                        stats.changed += 1;
                        items.push(diff_item(
                            AlignMode::Lcs,
                            "changed",
                            "changed",
                            &lcs_pair_key(oracle_item, generated_item),
                            Some(oracle_item),
                            Some(generated_item),
                            changed_fields,
                            None,
                        ));
                    }
                }
                AlignmentOp::Missing(oracle_index) => {
                    let oracle_item = oracle_stream[oracle_index];
                    stats.missing += 1;
                    items.push(diff_item(
                        AlignMode::Lcs,
                        "missing",
                        "missing",
                        &oracle_item.record_uid,
                        Some(oracle_item),
                        None,
                        Vec::new(),
                        Some("LCS alignment에서 oracle에만 존재".to_string()),
                    ));
                }
                AlignmentOp::Extra(generated_index) => {
                    let generated_item = generated_stream[generated_index];
                    stats.extra += 1;
                    items.push(diff_item(
                        AlignMode::Lcs,
                        "extra",
                        "extra",
                        &generated_item.record_uid,
                        None,
                        Some(generated_item),
                        Vec::new(),
                        Some("LCS alignment에서 generated에만 존재".to_string()),
                    ));
                }
            }
        }
    }

    (items, stats)
}

fn group_by_stream(items: &[Hwp5InventoryItem]) -> BTreeMap<String, Vec<&Hwp5InventoryItem>> {
    let mut grouped: BTreeMap<String, Vec<&Hwp5InventoryItem>> = BTreeMap::new();
    for item in items {
        grouped
            .entry(item.stream_path.clone())
            .or_default()
            .push(item);
    }
    grouped
}

#[derive(Debug, Clone, Copy)]
enum AlignmentOp {
    Pair(usize, usize),
    Missing(usize),
    Extra(usize),
}

fn lcs_alignment_ops(
    oracle: &[&Hwp5InventoryItem],
    generated: &[&Hwp5InventoryItem],
) -> Vec<AlignmentOp> {
    if oracle.is_empty() {
        return (0..generated.len()).map(AlignmentOp::Extra).collect();
    }
    if generated.is_empty() {
        return (0..oracle.len()).map(AlignmentOp::Missing).collect();
    }

    let oracle_signatures: Vec<String> = oracle
        .iter()
        .map(|item| structural_signature(item))
        .collect();
    let generated_signatures: Vec<String> = generated
        .iter()
        .map(|item| structural_signature(item))
        .collect();
    let rows = oracle.len() + 1;
    let cols = generated.len() + 1;
    let mut dp = vec![0u16; rows * cols];

    for i in 0..oracle.len() {
        for j in 0..generated.len() {
            let value = if oracle_signatures[i] == generated_signatures[j] {
                dp[i * cols + j].saturating_add(1)
            } else {
                dp[i * cols + j + 1].max(dp[(i + 1) * cols + j])
            };
            dp[(i + 1) * cols + j + 1] = value;
        }
    }

    let mut ops = Vec::new();
    let mut i = oracle.len();
    let mut j = generated.len();
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && oracle_signatures[i - 1] == generated_signatures[j - 1] {
            ops.push(AlignmentOp::Pair(i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i * cols + j - 1] >= dp[(i - 1) * cols + j]) {
            ops.push(AlignmentOp::Extra(j - 1));
            j -= 1;
        } else {
            ops.push(AlignmentOp::Missing(i - 1));
            i -= 1;
        }
    }

    ops.reverse();
    ops
}

fn structural_signature(item: &Hwp5InventoryItem) -> String {
    format!(
        "tag={}|role={}|ctrl={}",
        item.tag_id,
        item.tuple_role,
        item.control_id.as_deref().unwrap_or("-")
    )
}

fn lcs_pair_key(oracle: &Hwp5InventoryItem, generated: &Hwp5InventoryItem) -> String {
    if oracle.record_uid == generated.record_uid {
        oracle.record_uid.clone()
    } else {
        format!("{}~{}", oracle.record_uid, generated.record_uid)
    }
}

fn changed_fields_lcs(oracle: &Hwp5InventoryItem, generated: &Hwp5InventoryItem) -> Vec<String> {
    let mut fields = Vec::new();
    if oracle.size != generated.size {
        fields.push("size".to_string());
    }
    if oracle.payload_hash != generated.payload_hash {
        fields.push("payload_hash".to_string());
    }
    if oracle.control_name != generated.control_name {
        fields.push("control_name".to_string());
    }
    fields
}

fn inventory_map(items: &[Hwp5InventoryItem]) -> BTreeMap<String, &Hwp5InventoryItem> {
    items
        .iter()
        .map(|item| (item.record_uid.clone(), item))
        .collect()
}

fn append_index_changed_items(
    items: &mut Vec<Hwp5InventoryDiffItem>,
    key: &str,
    oracle: &Hwp5InventoryItem,
    generated: &Hwp5InventoryItem,
) {
    if oracle.tag_id != generated.tag_id {
        items.push(diff_item(
            AlignMode::Index,
            "tag_changed",
            "changed",
            key,
            Some(oracle),
            Some(generated),
            vec!["tag".to_string()],
            Some(format!("{} -> {}", oracle.tag_name, generated.tag_name)),
        ));
    }

    if oracle.size != generated.size {
        items.push(diff_item(
            AlignMode::Index,
            "size_changed",
            "changed",
            key,
            Some(oracle),
            Some(generated),
            vec!["size".to_string()],
            Some(format!("{} -> {}", oracle.size, generated.size)),
        ));
    }

    if oracle.payload_hash != generated.payload_hash {
        items.push(diff_item(
            AlignMode::Index,
            "payload_changed",
            "changed",
            key,
            Some(oracle),
            Some(generated),
            vec!["payload_hash".to_string()],
            None,
        ));
    }

    if oracle.scope_path != generated.scope_path {
        items.push(diff_item(
            AlignMode::Index,
            "scope_changed",
            "changed",
            key,
            Some(oracle),
            Some(generated),
            vec!["scope_path".to_string()],
            None,
        ));
    }

    if oracle.control_id != generated.control_id || oracle.control_name != generated.control_name {
        items.push(diff_item(
            AlignMode::Index,
            "control_changed",
            "changed",
            key,
            Some(oracle),
            Some(generated),
            vec!["control".to_string()],
            None,
        ));
    }
}

fn diff_item(
    align_mode: AlignMode,
    diff_kind: &str,
    alignment_status: &str,
    key: &str,
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
    changed_fields: Vec<String>,
    note: Option<String>,
) -> Hwp5InventoryDiffItem {
    let anchor = oracle.or(generated).expect("diff item must have a side");
    Hwp5InventoryDiffItem {
        align_mode: align_mode.as_str().to_string(),
        alignment_status: alignment_status.to_string(),
        diff_kind: diff_kind.to_string(),
        key: key.to_string(),
        stream_path: anchor.stream_path.clone(),
        section: anchor.section,
        record_index: anchor.record_index,
        oracle_record_index: oracle.map(|item| item.record_index),
        generated_record_index: generated.map(|item| item.record_index),
        oracle_record_uid: oracle.map(|item| item.record_uid.clone()),
        generated_record_uid: generated.map(|item| item.record_uid.clone()),
        signature: structural_signature(anchor),
        changed_fields,
        oracle: oracle.map(record_summary),
        generated: generated.map(record_summary),
        note,
    }
}

fn record_summary(item: &Hwp5InventoryItem) -> Hwp5InventoryDiffRecord {
    Hwp5InventoryDiffRecord {
        record_uid: item.record_uid.clone(),
        tag_id: item.tag_id,
        tag_name: item.tag_name.clone(),
        size: item.size,
        tuple_role: item.tuple_role.clone(),
        tuple_index: item.tuple_index,
        control_id: item.control_id.clone(),
        control_name: item.control_name.clone(),
        scope_path: item.scope_path.clone(),
        payload_hash: item.payload_hash.clone(),
    }
}

fn render_jsonl(diff: &Hwp5InventoryDiff) -> String {
    let mut output = String::new();
    for item in &diff.items {
        if let Ok(line) = serde_json::to_string(item) {
            output.push_str(&line);
            output.push('\n');
        }
    }
    output
}

fn render_markdown(diff: &Hwp5InventoryDiff) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Inventory Diff");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", diff.oracle_path);
    let _ = writeln!(output, "- generated: `{}`", diff.generated_path);
    let _ = writeln!(output, "- align_mode: `{}`", diff.align_mode.as_str());
    let _ = writeln!(output, "- diff_count: `{}`", diff.items.len());
    let _ = writeln!(output);

    let _ = writeln!(output, "## Alignment Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "| status | count |");
    let _ = writeln!(output, "|---|---:|");
    let _ = writeln!(output, "| `matched` | {} |", diff.stats.matched);
    let _ = writeln!(output, "| `changed` | {} |", diff.stats.changed);
    let _ = writeln!(output, "| `missing` | {} |", diff.stats.missing);
    let _ = writeln!(output, "| `extra` | {} |", diff.stats.extra);
    let _ = writeln!(output);

    let _ = writeln!(output, "## Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "| kind | count |");
    let _ = writeln!(output, "|---|---:|");
    for (kind, count) in summary_counts(&diff.items) {
        let _ = writeln!(output, "| `{}` | {} |", kind, count);
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Tuple Anchor Summary");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| role | control | missing | extra | changed | total |"
    );
    let _ = writeln!(output, "|---|---|---:|---:|---:|---:|");
    for ((role, control), counts) in tuple_anchor_summary(&diff.items) {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {} | {} | {} | {} |",
            escape_md(&role),
            escape_md(&control),
            counts.missing,
            counts.extra,
            counts.changed,
            counts.total()
        );
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Divergence Blocks");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| stream | rows | kinds | oracle range | generated range |"
    );
    let _ = writeln!(output, "|---|---:|---|---|---|");
    for block in divergence_blocks(&diff.items, 50) {
        let _ = writeln!(
            output,
            "| `{}` | {} | `{}` | `{}` | `{}` |",
            escape_md(&block.stream_path),
            block.rows,
            escape_md(&block.kinds),
            escape_md(&block.oracle_range),
            escape_md(&block.generated_range)
        );
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Diff Rows");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| status | kind | key | fields | oracle | generated | note |"
    );
    let _ = writeln!(output, "|---|---|---|---|---|---|---|");
    for item in &diff.items {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | `{}` | `{}` | {} | {} | {} |",
            escape_md(&item.alignment_status),
            escape_md(&item.diff_kind),
            escape_md(&item.key),
            escape_md(&item.changed_fields.join(",")),
            render_side(item.oracle.as_ref()),
            render_side(item.generated.as_ref()),
            item.note
                .as_deref()
                .map(escape_md)
                .unwrap_or_else(|| "-".to_string())
        );
    }

    output
}

fn render_hints_markdown(diff: &Hwp5InventoryDiff) -> String {
    const LIMIT: usize = 80;

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Contract Violation Hints");
    let _ = writeln!(output);
    let _ = writeln!(output, "## Input");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", diff.oracle_path);
    let _ = writeln!(output, "- generated: `{}`", diff.generated_path);
    let _ = writeln!(output, "- align_mode: `{}`", diff.align_mode.as_str());
    let _ = writeln!(output, "- report_mode: `{}`", ReportMode::Hints.as_str());
    let _ = writeln!(output, "- diff_count: `{}`", diff.items.len());
    let _ = writeln!(output);

    let _ = writeln!(output, "## Alignment Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "| status | count |");
    let _ = writeln!(output, "|---|---:|");
    let _ = writeln!(output, "| `matched` | {} |", diff.stats.matched);
    let _ = writeln!(output, "| `changed` | {} |", diff.stats.changed);
    let _ = writeln!(output, "| `missing` | {} |", diff.stats.missing);
    let _ = writeln!(output, "| `extra` | {} |", diff.stats.extra);
    let _ = writeln!(output);

    render_top_role_control_buckets(&mut output, diff);
    render_missing_records(&mut output, diff, LIMIT);
    render_candidate_table(
        &mut output,
        "Table Candidates",
        &diff.items,
        is_table_candidate,
        LIMIT,
    );
    render_candidate_table(
        &mut output,
        "Picture/Shape Candidates",
        &diff.items,
        is_picture_shape_candidate,
        LIMIT,
    );
    render_candidate_table(
        &mut output,
        "CtrlHeader Candidates",
        &diff.items,
        is_ctrl_header_candidate,
        LIMIT,
    );
    render_payload_hotspots(&mut output, diff, LIMIT);
    render_next_probe_suggestions(&mut output, diff);

    output
}

fn render_bundles_markdown(
    diff: &Hwp5InventoryDiff,
    focus_mode: FocusMode,
    window: usize,
) -> String {
    const LIMIT: usize = 80;

    let candidates: Vec<&Hwp5InventoryDiffItem> = diff
        .items
        .iter()
        .filter(|item| matches_focus(item, focus_mode))
        .collect();

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Candidate Bundles");
    let _ = writeln!(output);
    let _ = writeln!(output, "## Input");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", diff.oracle_path);
    let _ = writeln!(output, "- generated: `{}`", diff.generated_path);
    let _ = writeln!(output, "- align_mode: `{}`", diff.align_mode.as_str());
    let _ = writeln!(output, "- report_mode: `{}`", ReportMode::Bundles.as_str());
    let _ = writeln!(output, "- focus: `{}`", focus_mode.as_str());
    let _ = writeln!(output, "- window: `{}`", window);
    let _ = writeln!(output, "- candidate_count: `{}`", candidates.len());
    let _ = writeln!(output);

    let _ = writeln!(output, "## Alignment Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "| status | count |");
    let _ = writeln!(output, "|---|---:|");
    let _ = writeln!(output, "| `matched` | {} |", diff.stats.matched);
    let _ = writeln!(output, "| `changed` | {} |", diff.stats.changed);
    let _ = writeln!(output, "| `missing` | {} |", diff.stats.missing);
    let _ = writeln!(output, "| `extra` | {} |", diff.stats.extra);
    let _ = writeln!(output);

    render_bundle_candidate_summary(&mut output, &candidates);

    let _ = writeln!(output, "## Bundles");
    let _ = writeln!(output);
    for (index, item) in candidates.into_iter().take(LIMIT).enumerate() {
        render_one_bundle(&mut output, index + 1, item, diff, window);
    }

    if diff
        .items
        .iter()
        .filter(|item| matches_focus(item, focus_mode))
        .count()
        > LIMIT
    {
        let omitted = diff
            .items
            .iter()
            .filter(|item| matches_focus(item, focus_mode))
            .count()
            - LIMIT;
        let _ = writeln!(output, "후보 `{omitted}`개는 생략했다.");
        let _ = writeln!(output);
    }

    output
}

fn render_table_fields_markdown(diff: &Hwp5InventoryDiff) -> String {
    let candidates: Vec<&Hwp5InventoryDiffItem> = diff
        .items
        .iter()
        .filter(|item| is_table_candidate(item))
        .collect();

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Table Field Diff");
    let _ = writeln!(output);
    let _ = writeln!(output, "## Input");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", diff.oracle_path);
    let _ = writeln!(output, "- generated: `{}`", diff.generated_path);
    let _ = writeln!(output, "- align_mode: `{}`", diff.align_mode.as_str());
    let _ = writeln!(
        output,
        "- report_mode: `{}`",
        ReportMode::TableFields.as_str()
    );
    let _ = writeln!(output, "- candidate_count: `{}`", candidates.len());
    let _ = writeln!(output);

    render_table_field_summary(&mut output, &candidates, diff);

    let _ = writeln!(output, "## Table Field Rows");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| key | tag | field | offset | oracle | generated | status |"
    );
    let _ = writeln!(output, "|---|---|---|---:|---|---|---|");

    for item in candidates {
        let oracle = inventory_side(
            &diff.oracle_items,
            &item.stream_path,
            item.oracle_record_index,
        );
        let generated = inventory_side(
            &diff.generated_items,
            &item.stream_path,
            item.generated_record_index,
        );
        for row in table_field_rows(oracle, generated) {
            let _ = writeln!(
                output,
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                escape_md(&item.key),
                escape_md(&row.tag_name),
                escape_md(&row.field_name),
                escape_md(&row.offset),
                escape_md(&row.oracle),
                escape_md(&row.generated),
                row.status
            );
        }
    }

    output
}

fn render_table_probe_plan_markdown(diff: &Hwp5InventoryDiff) -> String {
    let candidates: Vec<&Hwp5InventoryDiffItem> = diff
        .items
        .iter()
        .filter(|item| is_table_candidate(item))
        .collect();
    let axes = table_probe_axes(&candidates, diff);

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Table Probe Plan");
    let _ = writeln!(output);
    let _ = writeln!(output, "## Input");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", diff.oracle_path);
    let _ = writeln!(output, "- generated: `{}`", diff.generated_path);
    let _ = writeln!(output, "- align_mode: `{}`", diff.align_mode.as_str());
    let _ = writeln!(
        output,
        "- report_mode: `{}`",
        ReportMode::TableProbePlan.as_str()
    );
    let _ = writeln!(output, "- candidate_count: `{}`", candidates.len());
    let _ = writeln!(output);

    let _ = writeln!(output, "## Probe Axes");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| axis | record kind | affected records | meaning |"
    );
    let _ = writeln!(output, "|---|---|---:|---|");
    for axis in &axes {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {} | {} |",
            axis.name,
            axis.record_kind,
            axis.rows.len(),
            axis.description
        );
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Recommended Probe Matrix");
    let _ = writeln!(output);
    let _ = writeln!(output, "| variant | graft axes | purpose |");
    let _ = writeln!(output, "|---|---|---|");
    let _ = writeln!(
        output,
        "| `01_ctrl_outer_margin_only` | `ctrl_outer_margin` | TABLE CTRL_HEADER outer margin 단독 효과 확인 |"
    );
    let _ = writeln!(
        output,
        "| `02_table_attr_only` | `table_attr` | TABLE attr 단독 효과 확인 |"
    );
    let _ = writeln!(
        output,
        "| `03_table_tail_only` | `table_tail` | TABLE payload tail 2 bytes 계열 단독 효과 확인 |"
    );
    let _ = writeln!(
        output,
        "| `04_ctrl_common_attr_only` | `ctrl_common_attr` | TABLE CTRL_HEADER common attr 차이의 영향 확인 |"
    );
    let _ = writeln!(
        output,
        "| `05_outer_margin_table_attr` | `ctrl_outer_margin`, `table_attr` | 위치/속성 축 결합 확인 |"
    );
    let _ = writeln!(
        output,
        "| `06_outer_margin_table_tail` | `ctrl_outer_margin`, `table_tail` | 위치/tail 축 결합 확인 |"
    );
    let _ = writeln!(
        output,
        "| `07_table_attr_tail` | `table_attr`, `table_tail` | TABLE record 내부 축 결합 확인 |"
    );
    let _ = writeln!(
        output,
        "| `08_all_table_axes` | `ctrl_outer_margin`, `ctrl_common_attr`, `table_attr`, `table_tail` | TABLE 관련 관찰 축 전체 positive guard |"
    );
    let _ = writeln!(output);

    let _ = writeln!(output, "## Axis Details");
    let _ = writeln!(output);
    for axis in &axes {
        let _ = writeln!(output, "### `{}`", axis.name);
        let _ = writeln!(output);
        let _ = writeln!(output, "- record_kind: `{}`", axis.record_kind);
        let _ = writeln!(output, "- affected_records: `{}`", axis.rows.len());
        let _ = writeln!(output, "- description: {}", axis.description);
        let _ = writeln!(output);
        let _ = writeln!(
            output,
            "| key | oracle record | generated record | fields | oracle values | generated values |"
        );
        let _ = writeln!(output, "|---|---|---|---|---|---|");
        for row in &axis.rows {
            let _ = writeln!(
                output,
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                escape_md(&row.key),
                escape_md(&row.oracle_record),
                escape_md(&row.generated_record),
                escape_md(&row.fields.join(", ")),
                escape_md(&row.oracle_values.join("; ")),
                escape_md(&row.generated_values.join("; "))
            );
        }
        let _ = writeln!(output);
    }

    let _ = writeln!(output, "## Notes");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "- 이 문서는 판정용 HWP를 직접 생성하지 않는다. 다음 단계에서 바이너리 graft 또는 저장기 projection을 만들 때 사용할 작업 지시서다."
    );
    let _ = writeln!(
        output,
        "- 필드명은 현재 P0 decoder의 관찰명이다. `tail_after_0x16`, `z_order_or_instance` 등은 contract 이름으로 확정하지 않는다."
    );
    let _ = writeln!(
        output,
        "- probe 생성 시에는 한 번에 한 축만 바꾸는 파일과 전체 positive guard 파일을 함께 만들어야 한다."
    );

    output
}

#[derive(Debug)]
struct TableProbeAxis {
    name: &'static str,
    record_kind: &'static str,
    description: &'static str,
    rows: Vec<TableProbeAxisRow>,
}

#[derive(Debug)]
struct TableProbeAxisRow {
    key: String,
    oracle_record: String,
    generated_record: String,
    fields: Vec<String>,
    oracle_values: Vec<String>,
    generated_values: Vec<String>,
}

fn table_probe_axes(
    candidates: &[&Hwp5InventoryDiffItem],
    diff: &Hwp5InventoryDiff,
) -> Vec<TableProbeAxis> {
    let mut ctrl_outer_margin = TableProbeAxis {
        name: "ctrl_outer_margin",
        record_kind: "CTRL_HEADER(Table)",
        description: "TABLE control wrapper의 바깥 여백 4필드",
        rows: Vec::new(),
    };
    let mut ctrl_common_attr = TableProbeAxis {
        name: "ctrl_common_attr",
        record_kind: "CTRL_HEADER(Table)",
        description: "TABLE control wrapper 공통 속성 비트",
        rows: Vec::new(),
    };
    let mut table_attr = TableProbeAxis {
        name: "table_attr",
        record_kind: "TABLE",
        description: "TABLE record 첫 4바이트 속성 비트",
        rows: Vec::new(),
    };
    let mut table_tail = TableProbeAxis {
        name: "table_tail",
        record_kind: "TABLE",
        description: "TABLE record 0x16 이후 tail payload",
        rows: Vec::new(),
    };

    for item in candidates {
        let oracle = inventory_side(
            &diff.oracle_items,
            &item.stream_path,
            item.oracle_record_index,
        );
        let generated = inventory_side(
            &diff.generated_items,
            &item.stream_path,
            item.generated_record_index,
        );
        let rows = table_field_rows(oracle, generated);
        let tag_name = oracle
            .or(generated)
            .map(|item| item.tag_name.as_str())
            .unwrap_or("-");

        if tag_name == "CTRL_HEADER" {
            if let Some(row) = table_probe_row(
                item,
                oracle,
                generated,
                &rows,
                &[
                    "out_margin_left",
                    "out_margin_right",
                    "out_margin_top",
                    "out_margin_bottom",
                ],
            ) {
                ctrl_outer_margin.rows.push(row);
            }
            if let Some(row) = table_probe_row(item, oracle, generated, &rows, &["common_attr"]) {
                ctrl_common_attr.rows.push(row);
            }
        } else if tag_name == "TABLE" {
            if let Some(row) = table_probe_row(item, oracle, generated, &rows, &["table_attr"]) {
                table_attr.rows.push(row);
            }
            if let Some(row) = table_tail_probe_row(item, oracle, generated) {
                table_tail.rows.push(row);
            }
        }
    }

    vec![ctrl_outer_margin, ctrl_common_attr, table_attr, table_tail]
}

fn table_tail_probe_row(
    item: &Hwp5InventoryDiffItem,
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
) -> Option<TableProbeAxisRow> {
    let oracle_payload = oracle
        .map(|item| item.payload_bytes.as_slice())
        .unwrap_or(&[]);
    let generated_payload = generated
        .map(|item| item.payload_bytes.as_slice())
        .unwrap_or(&[]);
    if oracle_payload.len() < 0x16 || generated_payload.len() < 0x16 {
        return None;
    }

    let oracle_tail = &oracle_payload[0x16..];
    let generated_tail = &generated_payload[0x16..];
    if oracle_tail == generated_tail {
        return None;
    }

    Some(TableProbeAxisRow {
        key: item.key.clone(),
        oracle_record: oracle
            .map(|item| item.record_uid.clone())
            .unwrap_or_else(|| "<missing>".to_string()),
        generated_record: generated
            .map(|item| item.record_uid.clone())
            .unwrap_or_else(|| "<missing>".to_string()),
        fields: vec!["table_tail_full".to_string()],
        oracle_values: vec![format!(
            "table_tail_full={} bytes: {}",
            oracle_tail.len(),
            hex_slice(oracle_tail)
        )],
        generated_values: vec![format!(
            "table_tail_full={} bytes: {}",
            generated_tail.len(),
            hex_slice(generated_tail)
        )],
    })
}

fn table_probe_row(
    item: &Hwp5InventoryDiffItem,
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
    rows: &[TableFieldRow],
    wanted_fields: &[&str],
) -> Option<TableProbeAxisRow> {
    let mut fields = Vec::new();
    let mut oracle_values = Vec::new();
    let mut generated_values = Vec::new();

    for wanted in wanted_fields {
        if let Some(row) = rows
            .iter()
            .find(|row| row.field_name == *wanted && row.status == "diff")
        {
            fields.push(row.field_name.clone());
            oracle_values.push(format!("{}={}", row.field_name, row.oracle));
            generated_values.push(format!("{}={}", row.field_name, row.generated));
        }
    }

    if fields.is_empty() {
        return None;
    }

    Some(TableProbeAxisRow {
        key: item.key.clone(),
        oracle_record: oracle
            .map(|item| item.record_uid.clone())
            .unwrap_or_else(|| "<missing>".to_string()),
        generated_record: generated
            .map(|item| item.record_uid.clone())
            .unwrap_or_else(|| "<missing>".to_string()),
        fields,
        oracle_values,
        generated_values,
    })
}

fn render_table_field_summary(
    output: &mut String,
    candidates: &[&Hwp5InventoryDiffItem],
    diff: &Hwp5InventoryDiff,
) {
    let mut field_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut record_counts: BTreeMap<String, usize> = BTreeMap::new();

    for item in candidates {
        let oracle = inventory_side(
            &diff.oracle_items,
            &item.stream_path,
            item.oracle_record_index,
        );
        let generated = inventory_side(
            &diff.generated_items,
            &item.stream_path,
            item.generated_record_index,
        );
        let tag_name = oracle
            .or(generated)
            .map(|item| item.tag_name.clone())
            .unwrap_or_else(|| "-".to_string());
        *record_counts.entry(tag_name).or_insert(0) += 1;

        for row in table_field_rows(oracle, generated) {
            if row.status == "diff" {
                *field_counts.entry(row.field_name).or_insert(0) += 1;
            }
        }
    }

    let mut fields: Vec<_> = field_counts.into_iter().collect();
    fields.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let _ = writeln!(output, "## Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "### Candidate Tags");
    let _ = writeln!(output);
    let _ = writeln!(output, "| tag | count |");
    let _ = writeln!(output, "|---|---:|");
    for (tag, count) in record_counts {
        let _ = writeln!(output, "| `{}` | {} |", escape_md(&tag), count);
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "### Diff Fields");
    let _ = writeln!(output);
    let _ = writeln!(output, "| field | diff count |");
    let _ = writeln!(output, "|---|---:|");
    if fields.is_empty() {
        let _ = writeln!(output, "| - | 0 |");
    } else {
        for (field, count) in fields {
            let _ = writeln!(output, "| `{}` | {} |", escape_md(&field), count);
        }
    }
    let _ = writeln!(output);
}

fn render_bundle_candidate_summary(output: &mut String, candidates: &[&Hwp5InventoryDiffItem]) {
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for item in candidates {
        let (role, control) = role_control(item);
        *counts.entry((role, control)).or_insert(0) += 1;
    }

    let mut rows: Vec<_> = counts.into_iter().collect();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let _ = writeln!(output, "## Candidate Summary");
    let _ = writeln!(output);
    let _ = writeln!(output, "| role | control | count |");
    let _ = writeln!(output, "|---|---|---:|");
    if rows.is_empty() {
        let _ = writeln!(output, "| - | - | 0 |");
    } else {
        for ((role, control), count) in rows {
            let _ = writeln!(
                output,
                "| `{}` | `{}` | {} |",
                escape_md(&role),
                escape_md(&control),
                count
            );
        }
    }
    let _ = writeln!(output);
}

fn render_one_bundle(
    output: &mut String,
    ordinal: usize,
    item: &Hwp5InventoryDiffItem,
    diff: &Hwp5InventoryDiff,
    window: usize,
) {
    let _ = writeln!(output, "### Bundle {ordinal}: `{}`", escape_md(&item.key));
    let _ = writeln!(output);
    let _ = writeln!(output, "- status: `{}`", item.alignment_status);
    let _ = writeln!(output, "- fields: `{}`", item.changed_fields.join(","));
    let _ = writeln!(output, "- stream: `{}`", item.stream_path);
    let _ = writeln!(output, "- oracle: {}", render_side(item.oracle.as_ref()));
    let _ = writeln!(
        output,
        "- generated: {}",
        render_side(item.generated.as_ref())
    );
    if let Some(note) = item.note.as_deref() {
        let _ = writeln!(output, "- note: `{}`", escape_md(note));
    }
    let _ = writeln!(output);

    render_inventory_window(
        output,
        "Oracle Window",
        &diff.oracle_items,
        &item.stream_path,
        item.oracle_record_index,
        window,
    );
    render_inventory_window(
        output,
        "Generated Window",
        &diff.generated_items,
        &item.stream_path,
        item.generated_record_index,
        window,
    );
}

fn render_inventory_window(
    output: &mut String,
    title: &str,
    items: &[Hwp5InventoryItem],
    stream_path: &str,
    center_index: Option<usize>,
    window: usize,
) {
    let _ = writeln!(output, "#### {title}");
    let _ = writeln!(output);
    let Some(center_index) = center_index else {
        let _ = writeln!(output, "해당 side record 없음");
        let _ = writeln!(output);
        return;
    };

    let start = center_index.saturating_sub(window);
    let end = center_index.saturating_add(window);
    let rows: Vec<&Hwp5InventoryItem> = items
        .iter()
        .filter(|item| {
            item.stream_path == stream_path
                && item.record_index >= start
                && item.record_index <= end
        })
        .collect();

    let _ = writeln!(
        output,
        "| focus | idx | level | tag | role | tuple | ctrl | size | key payload | head | hash | scope |"
    );
    let _ = writeln!(
        output,
        "|---|---:|---:|---|---|---:|---|---:|---|---|---|---|"
    );
    for row in rows {
        let focus = if row.record_index == center_index {
            ">"
        } else {
            ""
        };
        let ctrl = match (&row.control_id, &row.control_name) {
            (Some(id), Some(name)) => format!("{id}/{name}"),
            (Some(id), None) => id.clone(),
            _ => "-".to_string(),
        };
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | `{}` | `{}` | {} | `{}` | {} | `{}` | `{}` | `{}` | `{}` |",
            focus,
            row.record_index,
            row.level,
            escape_md(&row.tag_name),
            escape_md(&row.tuple_role),
            row.tuple_index,
            escape_md(&ctrl),
            row.size,
            escape_md(&row.key_payload),
            escape_md(&row.payload_head_hex),
            short_hash(&row.payload_hash),
            escape_md(&row.scope_path)
        );
    }
    let _ = writeln!(output);
}

fn matches_focus(item: &Hwp5InventoryDiffItem, focus_mode: FocusMode) -> bool {
    match focus_mode {
        FocusMode::All => true,
        FocusMode::Table => is_table_candidate(item),
        FocusMode::Shape => is_picture_shape_candidate(item),
        FocusMode::Ctrl => is_ctrl_header_candidate(item),
        FocusMode::Missing => item.alignment_status == "missing",
        FocusMode::DocInfo => role_control(item).0 == "docinfo",
    }
}

#[derive(Debug)]
struct TableFieldRow {
    tag_name: String,
    field_name: String,
    offset: String,
    oracle: String,
    generated: String,
    status: &'static str,
}

fn inventory_side<'a>(
    items: &'a [Hwp5InventoryItem],
    stream_path: &str,
    record_index: Option<usize>,
) -> Option<&'a Hwp5InventoryItem> {
    let record_index = record_index?;
    items
        .iter()
        .find(|item| item.stream_path == stream_path && item.record_index == record_index)
}

fn table_field_rows(
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
) -> Vec<TableFieldRow> {
    let tag_name = oracle
        .or(generated)
        .map(|item| item.tag_name.as_str())
        .unwrap_or("-");

    let mut rows = Vec::new();
    rows.push(compare_meta_field(
        "record_size",
        "-",
        oracle,
        generated,
        |item| item.size.to_string(),
    ));
    rows.push(compare_meta_field(
        "payload_len",
        "-",
        oracle,
        generated,
        |item| item.payload_bytes.len().to_string(),
    ));

    if tag_name == "CTRL_HEADER" {
        rows.extend(compare_ctrl_header_table_fields(oracle, generated));
    } else if tag_name == "TABLE" {
        rows.extend(compare_table_record_fields(oracle, generated));
    } else {
        rows.push(compare_meta_field(
            "payload_head",
            "0x00",
            oracle,
            generated,
            |item| item.payload_head_hex.clone(),
        ));
    }

    rows
}

fn compare_ctrl_header_table_fields(
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
) -> Vec<TableFieldRow> {
    let specs = [
        FieldSpec::u32("ctrl_id", 0x00),
        FieldSpec::u32_hex("common_attr", 0x04),
        FieldSpec::i32("x", 0x08),
        FieldSpec::i32("y", 0x0c),
        FieldSpec::i32("width", 0x10),
        FieldSpec::i32("height", 0x14),
        FieldSpec::u32("z_order_or_instance", 0x18),
        FieldSpec::u16("out_margin_left", 0x1c),
        FieldSpec::u16("out_margin_right", 0x1e),
        FieldSpec::u16("out_margin_top", 0x20),
        FieldSpec::u16("out_margin_bottom", 0x22),
        FieldSpec::hex("tail_after_0x24", 0x24, 16),
    ];
    specs
        .iter()
        .map(|spec| compare_payload_field("CTRL_HEADER", spec, oracle, generated))
        .collect()
}

fn compare_table_record_fields(
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
) -> Vec<TableFieldRow> {
    let specs = [
        FieldSpec::u32_hex("table_attr", 0x00),
        FieldSpec::u16("rows", 0x04),
        FieldSpec::u16("cols", 0x06),
        FieldSpec::u16("cell_spacing", 0x08),
        FieldSpec::u16("in_margin_left", 0x0a),
        FieldSpec::u16("in_margin_right", 0x0c),
        FieldSpec::u16("in_margin_top", 0x0e),
        FieldSpec::u16("in_margin_bottom", 0x10),
        FieldSpec::u16("row_count_hint", 0x12),
        FieldSpec::u16("col_count_hint", 0x14),
        FieldSpec::hex("tail_after_0x16", 0x16, 16),
    ];
    specs
        .iter()
        .map(|spec| compare_payload_field("TABLE", spec, oracle, generated))
        .collect()
}

fn compare_meta_field(
    field_name: &str,
    offset: &str,
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
    value: fn(&Hwp5InventoryItem) -> String,
) -> TableFieldRow {
    let tag_name = oracle
        .or(generated)
        .map(|item| item.tag_name.clone())
        .unwrap_or_else(|| "-".to_string());
    let oracle_value = oracle.map(value).unwrap_or_else(|| "<missing>".to_string());
    let generated_value = generated
        .map(value)
        .unwrap_or_else(|| "<missing>".to_string());
    TableFieldRow {
        tag_name,
        field_name: field_name.to_string(),
        offset: offset.to_string(),
        status: value_status(&oracle_value, &generated_value),
        oracle: oracle_value,
        generated: generated_value,
    }
}

fn compare_payload_field(
    tag_name: &str,
    spec: &FieldSpec,
    oracle: Option<&Hwp5InventoryItem>,
    generated: Option<&Hwp5InventoryItem>,
) -> TableFieldRow {
    let oracle_value = oracle
        .map(|item| spec.read(&item.payload_bytes))
        .unwrap_or_else(|| "<missing>".to_string());
    let generated_value = generated
        .map(|item| spec.read(&item.payload_bytes))
        .unwrap_or_else(|| "<missing>".to_string());
    TableFieldRow {
        tag_name: tag_name.to_string(),
        field_name: spec.name.to_string(),
        offset: format!("0x{:02x}", spec.offset),
        status: value_status(&oracle_value, &generated_value),
        oracle: oracle_value,
        generated: generated_value,
    }
}

fn value_status(oracle: &str, generated: &str) -> &'static str {
    if oracle == generated {
        "same"
    } else {
        "diff"
    }
}

#[derive(Debug, Clone, Copy)]
struct FieldSpec {
    name: &'static str,
    offset: usize,
    kind: FieldKind,
}

impl FieldSpec {
    fn u16(name: &'static str, offset: usize) -> Self {
        Self {
            name,
            offset,
            kind: FieldKind::U16,
        }
    }

    fn u32(name: &'static str, offset: usize) -> Self {
        Self {
            name,
            offset,
            kind: FieldKind::U32,
        }
    }

    fn u32_hex(name: &'static str, offset: usize) -> Self {
        Self {
            name,
            offset,
            kind: FieldKind::U32Hex,
        }
    }

    fn i32(name: &'static str, offset: usize) -> Self {
        Self {
            name,
            offset,
            kind: FieldKind::I32,
        }
    }

    fn hex(name: &'static str, offset: usize, len: usize) -> Self {
        Self {
            name,
            offset,
            kind: FieldKind::Hex(len),
        }
    }

    fn read(&self, bytes: &[u8]) -> String {
        match self.kind {
            FieldKind::U16 => read_u16(bytes, self.offset)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<short>".to_string()),
            FieldKind::U32 => read_u32(bytes, self.offset)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<short>".to_string()),
            FieldKind::U32Hex => read_u32(bytes, self.offset)
                .map(|value| format!("0x{value:08x} ({value})"))
                .unwrap_or_else(|| "<short>".to_string()),
            FieldKind::I32 => read_i32(bytes, self.offset)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<short>".to_string()),
            FieldKind::Hex(len) => {
                if self.offset >= bytes.len() {
                    return "<none>".to_string();
                }
                let end = self.offset.saturating_add(len).min(bytes.len());
                hex_slice(&bytes[self.offset..end])
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FieldKind {
    U16,
    U32,
    U32Hex,
    I32,
    Hex(usize),
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    Some(i32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn hex_slice(bytes: &[u8]) -> String {
    let mut result = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            result.push(' ');
        }
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn render_top_role_control_buckets(output: &mut String, diff: &Hwp5InventoryDiff) {
    let _ = writeln!(output, "## Top Role/Control Buckets");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| role | control | missing | extra | changed | total |"
    );
    let _ = writeln!(output, "|---|---|---:|---:|---:|---:|");

    let mut rows: Vec<_> = tuple_anchor_summary(&diff.items).into_iter().collect();
    rows.sort_by(|left, right| {
        right
            .1
            .total()
            .cmp(&left.1.total())
            .then_with(|| left.0.cmp(&right.0))
    });

    for ((role, control), counts) in rows.into_iter().take(50) {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {} | {} | {} | {} |",
            escape_md(&role),
            escape_md(&control),
            counts.missing,
            counts.extra,
            counts.changed,
            counts.total()
        );
    }
    let _ = writeln!(output);
}

fn render_missing_records(output: &mut String, diff: &Hwp5InventoryDiff, limit: usize) {
    let _ = writeln!(output, "## Missing Records");
    let _ = writeln!(output);
    let _ = writeln!(output, "| key | stream | oracle | note |");
    let _ = writeln!(output, "|---|---|---|---|");

    let mut count = 0usize;
    for item in diff
        .items
        .iter()
        .filter(|item| item.alignment_status == "missing")
    {
        count += 1;
        if count <= limit {
            let _ = writeln!(
                output,
                "| `{}` | `{}` | {} | {} |",
                escape_md(&item.key),
                escape_md(&item.stream_path),
                render_side(item.oracle.as_ref()),
                item.note
                    .as_deref()
                    .map(escape_md)
                    .unwrap_or_else(|| "-".to_string())
            );
        }
    }

    if count > limit {
        let _ = writeln!(
            output,
            "| ... | ... | ... | `{}`개 추가 생략 |",
            count - limit
        );
    } else if count == 0 {
        let _ = writeln!(output, "| - | - | - | missing record 없음 |");
    }
    let _ = writeln!(output);
}

fn render_candidate_table(
    output: &mut String,
    title: &str,
    items: &[Hwp5InventoryDiffItem],
    predicate: fn(&Hwp5InventoryDiffItem) -> bool,
    limit: usize,
) {
    let _ = writeln!(output, "## {title}");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| status | fields | key | oracle | generated | note |"
    );
    let _ = writeln!(output, "|---|---|---|---|---|---|");

    let mut count = 0usize;
    for item in items.iter().filter(|item| predicate(item)) {
        count += 1;
        if count <= limit {
            let _ = writeln!(
                output,
                "| `{}` | `{}` | `{}` | {} | {} | {} |",
                escape_md(&item.alignment_status),
                escape_md(&item.changed_fields.join(",")),
                escape_md(&item.key),
                render_side(item.oracle.as_ref()),
                render_side(item.generated.as_ref()),
                item.note
                    .as_deref()
                    .map(escape_md)
                    .unwrap_or_else(|| "-".to_string())
            );
        }
    }

    if count > limit {
        let _ = writeln!(
            output,
            "| ... | ... | ... | ... | ... | `{}`개 추가 생략 |",
            count - limit
        );
    } else if count == 0 {
        let _ = writeln!(output, "| - | - | - | - | - | 후보 없음 |");
    }
    let _ = writeln!(output);
}

fn render_payload_hotspots(output: &mut String, diff: &Hwp5InventoryDiff, limit: usize) {
    let _ = writeln!(output, "## Payload Changed Hotspots");
    let _ = writeln!(output);
    let _ = writeln!(output, "| role | control | count |");
    let _ = writeln!(output, "|---|---|---:|");

    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for item in &diff.items {
        if !item
            .changed_fields
            .iter()
            .any(|field| field == "payload_hash")
        {
            continue;
        }
        let (role, control) = role_control(item);
        *counts.entry((role, control)).or_insert(0) += 1;
    }

    let mut rows: Vec<_> = counts.into_iter().collect();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    if rows.is_empty() {
        let _ = writeln!(output, "| - | - | 0 |");
    } else {
        for ((role, control), count) in rows.into_iter().take(limit) {
            let _ = writeln!(
                output,
                "| `{}` | `{}` | {} |",
                escape_md(&role),
                escape_md(&control),
                count
            );
        }
    }
    let _ = writeln!(output);
}

fn render_next_probe_suggestions(output: &mut String, diff: &Hwp5InventoryDiff) {
    let has_missing = diff.stats.missing > 0;
    let table_count = diff
        .items
        .iter()
        .filter(|item| is_table_candidate(item))
        .count();
    let picture_count = diff
        .items
        .iter()
        .filter(|item| is_picture_shape_candidate(item))
        .count();
    let ctrl_count = diff
        .items
        .iter()
        .filter(|item| is_ctrl_header_candidate(item))
        .count();
    let docinfo_count = diff
        .items
        .iter()
        .filter(|item| role_control(item).0 == "docinfo")
        .count();

    let _ = writeln!(output, "## Next Probe Suggestions");
    let _ = writeln!(output);
    if has_missing {
        let _ = writeln!(
            output,
            "- missing record는 oracle의 record 단위 graft 후보로 먼저 분리한다."
        );
    }
    if table_count > 0 {
        let _ = writeln!(
            output,
            "- TABLE 후보 `{table_count}`건은 CTRL_HEADER(Table) + TABLE payload + child header의 tuple contract로 묶어 검증한다."
        );
    }
    if picture_count > 0 {
        let _ = writeln!(
            output,
            "- 그림/도형 후보 `{picture_count}`건은 DocInfo BinData reference와 PIC/SHAPE_COMPONENT payload를 함께 본다."
        );
    }
    if ctrl_count > 0 {
        let _ = writeln!(
            output,
            "- CTRL_HEADER 후보 `{ctrl_count}`건은 다음 record tag와 묶어 control별 필수 payload를 비교한다."
        );
    }
    if docinfo_count > 0 {
        let _ = writeln!(
            output,
            "- DocInfo 후보 `{docinfo_count}`건은 body record보다 먼저 count/reference table 계약 위반 가능성을 확인한다."
        );
    }
    if !(has_missing || table_count > 0 || picture_count > 0 || ctrl_count > 0 || docinfo_count > 0)
    {
        let _ = writeln!(
            output,
            "- 후보가 작다. index alignment report로 scope_path 수준 차이를 재확인한다."
        );
    }
    let _ = writeln!(output);
}

fn is_table_candidate(item: &Hwp5InventoryDiffItem) -> bool {
    let (role, control) = role_control(item);
    role == "table" || control == "Table"
}

fn is_picture_shape_candidate(item: &Hwp5InventoryDiffItem) -> bool {
    let (role, control) = role_control(item);
    role == "pic" || role == "shape_component" || control == "GenShape"
}

fn is_ctrl_header_candidate(item: &Hwp5InventoryDiffItem) -> bool {
    role_control(item).0 == "ctrl_header"
}

fn role_control(item: &Hwp5InventoryDiffItem) -> (String, String) {
    let record = item.oracle.as_ref().or(item.generated.as_ref());
    let Some(record) = record else {
        return ("-".to_string(), "-".to_string());
    };
    let role = record.tuple_role.clone();
    let control = record
        .control_name
        .clone()
        .or_else(|| record.control_id.clone())
        .unwrap_or_else(|| "-".to_string());
    (role, control)
}

#[derive(Debug, Default, Clone)]
struct TupleCounts {
    missing: usize,
    extra: usize,
    changed: usize,
}

impl TupleCounts {
    fn total(&self) -> usize {
        self.missing + self.extra + self.changed
    }
}

fn tuple_anchor_summary(
    items: &[Hwp5InventoryDiffItem],
) -> BTreeMap<(String, String), TupleCounts> {
    let mut summary: BTreeMap<(String, String), TupleCounts> = BTreeMap::new();
    for item in items {
        let side = item.oracle.as_ref().or(item.generated.as_ref());
        let Some(record) = side else {
            continue;
        };
        let role = record.tuple_role.clone();
        let control = record
            .control_name
            .clone()
            .or_else(|| record.control_id.clone())
            .unwrap_or_else(|| "-".to_string());
        let counts = summary.entry((role, control)).or_default();
        match item.alignment_status.as_str() {
            "missing" => counts.missing += 1,
            "extra" => counts.extra += 1,
            _ => counts.changed += 1,
        }
    }
    summary
}

#[derive(Debug)]
struct DivergenceBlock {
    stream_path: String,
    rows: usize,
    kinds: String,
    oracle_range: String,
    generated_range: String,
}

fn divergence_blocks(items: &[Hwp5InventoryDiffItem], limit: usize) -> Vec<DivergenceBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<DivergenceBlockBuilder> = None;

    for item in items {
        let should_extend = current
            .as_ref()
            .map(|block| block.can_extend(item))
            .unwrap_or(false);

        if !should_extend {
            if let Some(block) = current.take() {
                blocks.push(block.finish());
                if blocks.len() >= limit {
                    return blocks;
                }
            }
            current = Some(DivergenceBlockBuilder::new(item));
        } else if let Some(block) = current.as_mut() {
            block.push(item);
        }
    }

    if let Some(block) = current {
        blocks.push(block.finish());
    }

    blocks.truncate(limit);
    blocks
}

#[derive(Debug)]
struct DivergenceBlockBuilder {
    stream_path: String,
    rows: usize,
    kinds: BTreeSet<String>,
    oracle_min: Option<usize>,
    oracle_max: Option<usize>,
    generated_min: Option<usize>,
    generated_max: Option<usize>,
}

impl DivergenceBlockBuilder {
    fn new(item: &Hwp5InventoryDiffItem) -> Self {
        let mut block = Self {
            stream_path: item.stream_path.clone(),
            rows: 0,
            kinds: BTreeSet::new(),
            oracle_min: None,
            oracle_max: None,
            generated_min: None,
            generated_max: None,
        };
        block.push(item);
        block
    }

    fn can_extend(&self, item: &Hwp5InventoryDiffItem) -> bool {
        if self.stream_path != item.stream_path {
            return false;
        }
        let next_oracle = item.oracle_record_index.or(item.generated_record_index);
        let current_oracle = self.oracle_max.or(self.generated_max);
        match (current_oracle, next_oracle) {
            (Some(current), Some(next)) => next <= current.saturating_add(3),
            _ => true,
        }
    }

    fn push(&mut self, item: &Hwp5InventoryDiffItem) {
        self.rows += 1;
        self.kinds.insert(item.diff_kind.clone());
        if let Some(index) = item.oracle_record_index {
            update_range(&mut self.oracle_min, &mut self.oracle_max, index);
        }
        if let Some(index) = item.generated_record_index {
            update_range(&mut self.generated_min, &mut self.generated_max, index);
        }
    }

    fn finish(self) -> DivergenceBlock {
        DivergenceBlock {
            stream_path: self.stream_path,
            rows: self.rows,
            kinds: self.kinds.into_iter().collect::<Vec<_>>().join(","),
            oracle_range: format_range(self.oracle_min, self.oracle_max),
            generated_range: format_range(self.generated_min, self.generated_max),
        }
    }
}

fn update_range(min: &mut Option<usize>, max: &mut Option<usize>, value: usize) {
    *min = Some(min.map(|current| current.min(value)).unwrap_or(value));
    *max = Some(max.map(|current| current.max(value)).unwrap_or(value));
}

fn format_range(min: Option<usize>, max: Option<usize>) -> String {
    match (min, max) {
        (Some(min), Some(max)) if min == max => min.to_string(),
        (Some(min), Some(max)) => format!("{min}..{max}"),
        _ => "-".to_string(),
    }
}

fn summary_counts(items: &[Hwp5InventoryDiffItem]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts.entry(item.diff_kind.as_str()).or_insert(0) += 1;
    }
    counts
}

fn render_side(record: Option<&Hwp5InventoryDiffRecord>) -> String {
    let Some(record) = record else {
        return "-".to_string();
    };
    let ctrl = match (&record.control_id, &record.control_name) {
        (Some(id), Some(name)) => format!("{id}/{name}"),
        (Some(id), None) => id.clone(),
        _ => "-".to_string(),
    };
    format!(
        "`{} role={} tuple={} ctrl={} size={} hash={}`",
        escape_md(&record.tag_name),
        escape_md(&record.tuple_role),
        record.tuple_index,
        escape_md(&ctrl),
        record.size,
        short_hash(&record.payload_hash)
    )
}

fn short_hash(hash: &str) -> &str {
    let hash = hash.strip_prefix("blake3:").unwrap_or(hash);
    hash.get(..16).unwrap_or(hash)
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|")
}
