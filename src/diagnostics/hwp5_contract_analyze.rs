//! HWPX -> HWP5 contract graph analyzer.
//!
//! Unlike the probe generators, this command does not write mutated HWP files.
//! It compares a Hancom-produced oracle HWP, a generated HWP, and the HWPX
//! source to describe where the HWP5 record/control contract diverges.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use crate::diagnostics::hwp5_inventory::{build_inventory, Hwp5InventoryItem};
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::shape::ShapeObject;
use crate::parser::hwpx::parse_hwpx;
use crate::parser::tags;

#[derive(Debug)]
struct Options {
    hwpx: PathBuf,
    oracle: PathBuf,
    generated: PathBuf,
    out_dir: PathBuf,
    section: Option<u32>,
}

#[derive(Debug, Default)]
struct IrSummary {
    sections: usize,
    paragraphs: usize,
    line_segs: usize,
    controls: BTreeMap<String, usize>,
    table_cells: usize,
    bin_data_items: usize,
    bin_data_payloads: usize,
    docinfo_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Default)]
struct HwpxXmlSummary {
    xml_files: usize,
    bindata_files: usize,
    section_files: usize,
    counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
struct ControlSummary {
    key: String,
    stream_path: String,
    record_index: usize,
    level: u16,
    control_name: String,
    control_id: String,
    child_count: usize,
    child_tag_counts: BTreeMap<String, usize>,
    child_role_counts: BTreeMap<String, usize>,
    ctrl_header_size: u32,
    ctrl_header_hash: String,
    scope_path: String,
}

pub fn run(args: &[String]) {
    if args.is_empty() || matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        print_usage();
        return;
    }

    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            return;
        }
    };

    if let Err(error) = run_inner(options) {
        eprintln!("오류: {error}");
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut inputs = Vec::new();
    let mut out_dir = None;
    let mut section = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--out-dir 뒤에 경로가 필요합니다".to_string())?;
                out_dir = Some(PathBuf::from(value));
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
            value if value.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {value}"));
            }
            value => {
                inputs.push(PathBuf::from(value));
                i += 1;
            }
        }
    }

    if inputs.len() != 3 {
        return Err("HWPX source, oracle HWP, generated HWP 경로가 필요합니다".to_string());
    }

    Ok(Options {
        hwpx: inputs[0].clone(),
        oracle: inputs[1].clone(),
        generated: inputs[2].clone(),
        out_dir: out_dir.ok_or_else(|| "--out-dir 경로가 필요합니다".to_string())?,
        section,
    })
}

fn print_usage() {
    println!("사용법:");
    println!(
        "  rhwp hwp5-contract-analyze <source.hwpx> <oracle.hwp> <generated.hwp> --out-dir <폴더> [--section N]"
    );
}

fn run_inner(options: Options) -> Result<(), String> {
    fs::create_dir_all(&options.out_dir).map_err(|error| {
        format!(
            "출력 폴더 생성 실패 - {}: {error}",
            options.out_dir.display()
        )
    })?;

    let oracle = build_inventory(&options.oracle, options.section)?;
    let generated = build_inventory(&options.generated, options.section)?;
    let hwpx_bytes = fs::read(&options.hwpx)
        .map_err(|error| format!("HWPX 파일 읽기 실패 - {}: {error}", options.hwpx.display()))?;
    let hwpx_xml = summarize_hwpx_xml(&hwpx_bytes)?;
    let ir_summary = parse_hwpx(&hwpx_bytes)
        .map(|doc| summarize_ir(&doc))
        .map_err(|error| format!("HWPX IR 파싱 실패: {error}"))?;

    let docinfo_report = render_docinfo_contract(&options, &oracle.items, &generated.items);
    fs::write(options.out_dir.join("docinfo_contract.md"), docinfo_report).map_err(|error| {
        format!(
            "docinfo_contract.md 쓰기 실패 - {}: {error}",
            options.out_dir.display()
        )
    })?;

    let bodytext_report = render_bodytext_control_graph(&options, &oracle.items, &generated.items);
    fs::write(
        options.out_dir.join("bodytext_control_graph.md"),
        bodytext_report,
    )
    .map_err(|error| {
        format!(
            "bodytext_control_graph.md 쓰기 실패 - {}: {error}",
            options.out_dir.display()
        )
    })?;

    let trace_report = render_hwpx_ir_serializer_trace(&options, &hwpx_xml, &ir_summary);
    fs::write(
        options.out_dir.join("hwpx_ir_serializer_trace.md"),
        trace_report,
    )
    .map_err(|error| {
        format!(
            "hwpx_ir_serializer_trace.md 쓰기 실패 - {}: {error}",
            options.out_dir.display()
        )
    })?;

    let index_report = render_index(&options);
    fs::write(options.out_dir.join("stage12_index.md"), index_report).map_err(|error| {
        format!(
            "stage12_index.md 쓰기 실패 - {}: {error}",
            options.out_dir.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn render_index(options: &Options) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "# Stage 12 Contract Graph Analysis");
    let _ = writeln!(output);
    let _ = writeln!(output, "- hwpx: `{}`", options.hwpx.display());
    let _ = writeln!(output, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(output, "- generated: `{}`", options.generated.display());
    let _ = writeln!(output);
    let _ = writeln!(output, "## Reports");
    let _ = writeln!(output);
    let _ = writeln!(output, "- `docinfo_contract.md`");
    let _ = writeln!(output, "- `bodytext_control_graph.md`");
    let _ = writeln!(output, "- `hwpx_ir_serializer_trace.md`");
    output
}

fn render_docinfo_contract(
    options: &Options,
    oracle_items: &[Hwp5InventoryItem],
    generated_items: &[Hwp5InventoryItem],
) -> String {
    let oracle_doc = docinfo_items(oracle_items);
    let generated_doc = docinfo_items(generated_items);
    let oracle_counts = count_tags(&oracle_doc);
    let generated_counts = count_tags(&generated_doc);

    let mut output = String::new();
    let _ = writeln!(output, "# DocInfo Contract");
    let _ = writeln!(output);
    let _ = writeln!(output, "- hwpx: `{}`", options.hwpx.display());
    let _ = writeln!(output, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(output, "- generated: `{}`", options.generated.display());
    let _ = writeln!(output);

    let _ = writeln!(output, "## Tag Counts");
    let _ = writeln!(output);
    let _ = writeln!(output, "| tag | oracle | generated | delta |");
    let _ = writeln!(output, "|---|---:|---:|---:|");
    for tag in union_keys(&oracle_counts, &generated_counts) {
        let oracle_count = *oracle_counts.get(&tag).unwrap_or(&0);
        let generated_count = *generated_counts.get(&tag).unwrap_or(&0);
        let delta = generated_count as isize - oracle_count as isize;
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} |",
            tag, oracle_count, generated_count, delta
        );
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## ID_MAPPINGS Payload");
    let _ = writeln!(output);
    let oracle_id = first_item_by_tag(&oracle_doc, tags::HWPTAG_ID_MAPPINGS);
    let generated_id = first_item_by_tag(&generated_doc, tags::HWPTAG_ID_MAPPINGS);
    if let (Some(oracle_id), Some(generated_id)) = (oracle_id, generated_id) {
        let _ = writeln!(output, "- oracle size: `{}`", oracle_id.size);
        let _ = writeln!(output, "- generated size: `{}`", generated_id.size);
        let _ = writeln!(output);
        let oracle_values = u32_values(&oracle_id.payload_bytes);
        let generated_values = u32_values(&generated_id.payload_bytes);
        let max_len = oracle_values.len().max(generated_values.len());
        let _ = writeln!(
            output,
            "| index | field | oracle | generated | delta | status |"
        );
        let _ = writeln!(output, "|---:|---|---:|---:|---:|---|");
        for index in 0..max_len {
            let oracle_value = oracle_values.get(index).copied();
            let generated_value = generated_values.get(index).copied();
            let status = if oracle_value == generated_value {
                "same"
            } else {
                "diff"
            };
            let delta = match (oracle_value, generated_value) {
                (Some(o), Some(g)) => (g as i64 - o as i64).to_string(),
                (Some(_), None) => "missing-field".to_string(),
                (None, Some(_)) => "extra-field".to_string(),
                (None, None) => "-".to_string(),
            };
            let _ = writeln!(
                output,
                "| {} | `{}` | {} | {} | {} | `{}` |",
                index,
                id_mapping_field_name(index),
                format_opt_u32(oracle_value),
                format_opt_u32(generated_value),
                delta,
                status
            );
        }
    } else {
        let _ = writeln!(output, "ID_MAPPINGS record를 찾지 못했다.");
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Payload Difference Summary");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| tag | paired records | changed payload | size differences | first oracle idx | first generated idx |"
    );
    let _ = writeln!(output, "|---|---:|---:|---:|---:|---:|");
    for row in docinfo_payload_summaries(&oracle_doc, &generated_doc) {
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} | {} | {} |",
            row.tag_name,
            row.paired,
            row.changed,
            row.size_diffs,
            row.first_oracle_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            row.first_generated_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Contract Findings");
    let _ = writeln!(output);
    write_docinfo_findings(&mut output, &oracle_doc, &generated_doc);
    output
}

fn write_docinfo_findings(
    output: &mut String,
    oracle_doc: &[&Hwp5InventoryItem],
    generated_doc: &[&Hwp5InventoryItem],
) {
    let oracle_memo = count_tag(oracle_doc, tags::HWPTAG_MEMO_SHAPE);
    let generated_memo = count_tag(generated_doc, tags::HWPTAG_MEMO_SHAPE);
    let _ = writeln!(
        output,
        "- MEMO_SHAPE count: oracle=`{}`, generated=`{}`",
        oracle_memo, generated_memo
    );

    let oracle_id = first_item_by_tag(oracle_doc, tags::HWPTAG_ID_MAPPINGS);
    let generated_id = first_item_by_tag(generated_doc, tags::HWPTAG_ID_MAPPINGS);
    if let (Some(oracle_id), Some(generated_id)) = (oracle_id, generated_id) {
        let oracle_values = u32_values(&oracle_id.payload_bytes);
        let generated_values = u32_values(&generated_id.payload_bytes);
        let memo_index = 15usize;
        let _ = writeln!(
            output,
            "- ID_MAPPINGS memo_shape_count(index 15): oracle=`{}`, generated=`{}`",
            format_opt_u32(oracle_values.get(memo_index).copied()),
            format_opt_u32(generated_values.get(memo_index).copied())
        );
        if oracle_id.size != generated_id.size {
            let _ = writeln!(
                output,
                "- ID_MAPPINGS payload size differs: oracle=`{}`, generated=`{}`",
                oracle_id.size, generated_id.size
            );
        }
    }

    let _ = writeln!(output);
    let _ = writeln!(output, "해석:");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "```text\nDocInfo 차이는 record 존재 여부만 보지 않고 ID_MAPPINGS count와 함께 해석해야 한다.\nMEMO_SHAPE가 oracle에만 존재하거나 ID_MAPPINGS size/count가 다르면, 한컴 에디터는\nDocInfo 참조 테이블 계약 위반으로 판단할 수 있다.\n```"
    );
}

#[derive(Debug, Default)]
struct PayloadSummaryRow {
    tag_name: String,
    paired: usize,
    changed: usize,
    size_diffs: usize,
    first_oracle_index: Option<usize>,
    first_generated_index: Option<usize>,
}

fn docinfo_payload_summaries(
    oracle_items: &[&Hwp5InventoryItem],
    generated_items: &[&Hwp5InventoryItem],
) -> Vec<PayloadSummaryRow> {
    let mut oracle_seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut generated_seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut oracle_by_key: BTreeMap<String, &Hwp5InventoryItem> = BTreeMap::new();
    let mut generated_by_key: BTreeMap<String, &Hwp5InventoryItem> = BTreeMap::new();

    for item in oracle_items {
        let seen = oracle_seen.entry(item.tag_name.clone()).or_insert(0);
        let key = format!("{}#{}", item.tag_name, *seen);
        *seen += 1;
        oracle_by_key.insert(key, *item);
    }

    for item in generated_items {
        let seen = generated_seen.entry(item.tag_name.clone()).or_insert(0);
        let key = format!("{}#{}", item.tag_name, *seen);
        *seen += 1;
        generated_by_key.insert(key, *item);
    }

    let mut summaries: BTreeMap<String, PayloadSummaryRow> = BTreeMap::new();
    for key in union_ref_keys(&oracle_by_key, &generated_by_key) {
        if let (Some(oracle), Some(generated)) =
            (oracle_by_key.get(&key), generated_by_key.get(&key))
        {
            let row =
                summaries
                    .entry(oracle.tag_name.clone())
                    .or_insert_with(|| PayloadSummaryRow {
                        tag_name: oracle.tag_name.clone(),
                        ..PayloadSummaryRow::default()
                    });
            row.paired += 1;
            if oracle.payload_hash != generated.payload_hash {
                row.changed += 1;
                row.first_oracle_index.get_or_insert(oracle.record_index);
                row.first_generated_index
                    .get_or_insert(generated.record_index);
            }
            if oracle.size != generated.size {
                row.size_diffs += 1;
                row.first_oracle_index.get_or_insert(oracle.record_index);
                row.first_generated_index
                    .get_or_insert(generated.record_index);
            }
        }
    }
    summaries.into_values().collect()
}

fn render_bodytext_control_graph(
    options: &Options,
    oracle_items: &[Hwp5InventoryItem],
    generated_items: &[Hwp5InventoryItem],
) -> String {
    let oracle_controls = control_summaries(oracle_items);
    let generated_controls = control_summaries(generated_items);
    let oracle_by_key = controls_by_key(&oracle_controls);
    let generated_by_key = controls_by_key(&generated_controls);

    let oracle_body = bodytext_items(oracle_items);
    let generated_body = bodytext_items(generated_items);
    let oracle_counts = count_tags(&oracle_body);
    let generated_counts = count_tags(&generated_body);

    let mut output = String::new();
    let _ = writeln!(output, "# BodyText Control Graph");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(output, "- generated: `{}`", options.generated.display());
    let _ = writeln!(output);

    let _ = writeln!(output, "## BodyText Tag Counts");
    let _ = writeln!(output);
    let _ = writeln!(output, "| tag | oracle | generated | delta |");
    let _ = writeln!(output, "|---|---:|---:|---:|");
    for tag in union_keys(&oracle_counts, &generated_counts) {
        let oracle_count = *oracle_counts.get(&tag).unwrap_or(&0);
        let generated_count = *generated_counts.get(&tag).unwrap_or(&0);
        let delta = generated_count as isize - oracle_count as isize;
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} |",
            tag, oracle_count, generated_count, delta
        );
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Control Contract Differences");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| key | status | oracle idx | generated idx | ctrl | oracle children | generated children | finding |"
    );
    let _ = writeln!(output, "|---|---|---:|---:|---|---|---|---|");

    for key in union_ref_keys(&oracle_by_key, &generated_by_key) {
        match (oracle_by_key.get(&key), generated_by_key.get(&key)) {
            (Some(oracle), Some(generated)) => {
                let child_diff = oracle.child_tag_counts != generated.child_tag_counts
                    || oracle.child_role_counts != generated.child_role_counts
                    || oracle.ctrl_header_hash != generated.ctrl_header_hash
                    || oracle.ctrl_header_size != generated.ctrl_header_size;
                if child_diff {
                    let _ = writeln!(
                        output,
                        "| `{}` | `changed` | {} | {} | `{}` | `{}` | `{}` | `{}` |",
                        key,
                        oracle.record_index,
                        generated.record_index,
                        escape_md(&oracle.control_name),
                        escape_md(&format_counts(&oracle.child_tag_counts)),
                        escape_md(&format_counts(&generated.child_tag_counts)),
                        escape_md(&control_finding(oracle, generated))
                    );
                }
            }
            (Some(oracle), None) => {
                let _ = writeln!(
                    output,
                    "| `{}` | `missing-control` | {} | - | `{}` | `{}` | `-` | `oracle에만 존재하는 control` |",
                    key,
                    oracle.record_index,
                    escape_md(&oracle.control_name),
                    escape_md(&format_counts(&oracle.child_tag_counts))
                );
            }
            (None, Some(generated)) => {
                let _ = writeln!(
                    output,
                    "| `{}` | `extra-control` | - | {} | `{}` | `-` | `{}` | `generated에만 존재하는 control` |",
                    key,
                    generated.record_index,
                    escape_md(&generated.control_name),
                    escape_md(&format_counts(&generated.child_tag_counts))
                );
            }
            (None, None) => {}
        }
    }
    let _ = writeln!(output);

    let oracle_type_counts = count_control_types(&oracle_controls);
    let generated_type_counts = count_control_types(&generated_controls);
    let _ = writeln!(output, "## Control Type Counts");
    let _ = writeln!(output);
    let _ = writeln!(output, "| control | oracle | generated | delta |");
    let _ = writeln!(output, "|---|---:|---:|---:|");
    for key in union_keys(&oracle_type_counts, &generated_type_counts) {
        let oracle_count = *oracle_type_counts.get(&key).unwrap_or(&0);
        let generated_count = *generated_type_counts.get(&key).unwrap_or(&0);
        let delta = generated_count as isize - oracle_count as isize;
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} |",
            key, oracle_count, generated_count, delta
        );
    }
    output
}

fn control_finding(oracle: &ControlSummary, generated: &ControlSummary) -> String {
    let mut findings = Vec::new();
    for tag in union_keys(&oracle.child_tag_counts, &generated.child_tag_counts) {
        let o = *oracle.child_tag_counts.get(&tag).unwrap_or(&0);
        let g = *generated.child_tag_counts.get(&tag).unwrap_or(&0);
        if o != g {
            findings.push(format!("{tag}: oracle={o}, generated={g}"));
        }
    }
    if oracle.ctrl_header_size != generated.ctrl_header_size {
        findings.push(format!(
            "CTRL_HEADER size: oracle={}, generated={}",
            oracle.ctrl_header_size, generated.ctrl_header_size
        ));
    }
    if oracle.ctrl_header_hash != generated.ctrl_header_hash {
        findings.push("CTRL_HEADER payload differs".to_string());
    }
    if findings.is_empty() {
        "payload/role differs".to_string()
    } else {
        findings.join("; ")
    }
}

fn control_summaries(items: &[Hwp5InventoryItem]) -> Vec<ControlSummary> {
    let body_items: Vec<&Hwp5InventoryItem> = items
        .iter()
        .filter(|item| item.owner == "BodyText")
        .collect();
    let mut occurrence: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut summaries = Vec::new();

    for (pos, item) in body_items.iter().enumerate() {
        if item.tag_id != tags::HWPTAG_CTRL_HEADER {
            continue;
        }

        let control_name = item
            .control_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let control_id = item.control_id.clone().unwrap_or_else(|| "-".to_string());
        let occurrence_key = (item.stream_path.clone(), control_name.clone());
        let seen = occurrence.entry(occurrence_key).or_insert(0);
        let key = format!("{}:{}#{}", item.stream_path, control_name, *seen);
        *seen += 1;

        let mut child_tag_counts = BTreeMap::new();
        let mut child_role_counts = BTreeMap::new();
        let mut child_count = 0usize;
        for child in body_items.iter().skip(pos + 1) {
            if child.level <= item.level {
                break;
            }
            child_count += 1;
            *child_tag_counts.entry(child.tag_name.clone()).or_insert(0) += 1;
            *child_role_counts
                .entry(child.tuple_role.clone())
                .or_insert(0) += 1;
        }

        summaries.push(ControlSummary {
            key,
            stream_path: item.stream_path.clone(),
            record_index: item.record_index,
            level: item.level,
            control_name,
            control_id,
            child_count,
            child_tag_counts,
            child_role_counts,
            ctrl_header_size: item.size,
            ctrl_header_hash: short_hash(&item.payload_hash),
            scope_path: item.scope_path.clone(),
        });
    }

    summaries
}

fn controls_by_key(controls: &[ControlSummary]) -> BTreeMap<String, &ControlSummary> {
    controls
        .iter()
        .map(|control| (control.key.clone(), control))
        .collect()
}

fn count_control_types(controls: &[ControlSummary]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for control in controls {
        *counts.entry(control.control_name.clone()).or_insert(0) += 1;
    }
    counts
}

fn render_hwpx_ir_serializer_trace(
    options: &Options,
    hwpx_xml: &HwpxXmlSummary,
    ir_summary: &IrSummary,
) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "# HWPX / IR / Serializer Trace");
    let _ = writeln!(output);
    let _ = writeln!(output, "- hwpx: `{}`", options.hwpx.display());
    let _ = writeln!(output);

    let _ = writeln!(output, "## HWPX XML Surface Counts");
    let _ = writeln!(output);
    let _ = writeln!(output, "- xml files: `{}`", hwpx_xml.xml_files);
    let _ = writeln!(output, "- bindata files: `{}`", hwpx_xml.bindata_files);
    let _ = writeln!(output, "- section files: `{}`", hwpx_xml.section_files);
    let _ = writeln!(output);
    let _ = writeln!(output, "| marker | count |");
    let _ = writeln!(output, "|---|---:|");
    for (key, value) in &hwpx_xml.counts {
        let _ = writeln!(output, "| `{}` | {} |", key, value);
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## IR Counts");
    let _ = writeln!(output);
    let _ = writeln!(output, "- sections: `{}`", ir_summary.sections);
    let _ = writeln!(output, "- paragraphs: `{}`", ir_summary.paragraphs);
    let _ = writeln!(output, "- line_segs: `{}`", ir_summary.line_segs);
    let _ = writeln!(output, "- table_cells: `{}`", ir_summary.table_cells);
    let _ = writeln!(output, "- bin_data_items: `{}`", ir_summary.bin_data_items);
    let _ = writeln!(
        output,
        "- bin_data_payloads: `{}`",
        ir_summary.bin_data_payloads
    );
    let _ = writeln!(output);
    let _ = writeln!(output, "### IR Controls");
    let _ = writeln!(output);
    let _ = writeln!(output, "| control | count |");
    let _ = writeln!(output, "|---|---:|");
    for (key, value) in &ir_summary.controls {
        let _ = writeln!(output, "| `{}` | {} |", key, value);
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "### IR DocInfo Counts");
    let _ = writeln!(output);
    let _ = writeln!(output, "| item | count |");
    let _ = writeln!(output, "|---|---:|");
    for (key, value) in &ir_summary.docinfo_counts {
        let _ = writeln!(output, "| `{}` | {} |", key, value);
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Serializer Responsibility Map");
    let _ = writeln!(output);
    let _ = writeln!(output, "| contract area | source path | responsibility |");
    let _ = writeln!(output, "|---|---|---|");
    let rows = [
        (
            "DocInfo ID_MAPPINGS",
            "src/serializer/doc_info.rs",
            "DocInfo model counts and extra preserved counts must serialize into the HWP5 count table.",
        ),
        (
            "DocInfo MEMO_SHAPE",
            "src/parser/doc_info.rs, src/serializer/doc_info.rs",
            "MEMO_SHAPE must be parsed/preserved or reconstructed with the matching ID_MAPPINGS count.",
        ),
        (
            "TABLE control tuple",
            "src/serializer/control.rs",
            "CTRL_HEADER(Table), optional CTRL_DATA, LIST_HEADER cells, TABLE payload, and child paragraphs must be emitted as one valid tuple.",
        ),
        (
            "Shape/Picture tuple",
            "src/serializer/control.rs",
            "CTRL_HEADER(GenShape), SHAPE_COMPONENT, optional CTRL_DATA, and shape-specific child records must stay in a valid parent/child graph.",
        ),
        (
            "HWPX parsing to IR",
            "src/parser/hwpx/header.rs, src/parser/hwpx/section.rs",
            "HWPX semantic controls must survive into Document IR before HWP serialization.",
        ),
    ];
    for (area, path, responsibility) in rows {
        let _ = writeln!(output, "| `{}` | `{}` | {} |", area, path, responsibility);
    }
    output
}

fn summarize_hwpx_xml(bytes: &[u8]) -> Result<HwpxXmlSummary, String> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|error| format!("HWPX ZIP 열기 실패: {error}"))?;
    let mut summary = HwpxXmlSummary::default();

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("HWPX entry 열기 실패: {error}"))?;
        let name = file.name().to_string();
        if name.starts_with("BinData/") {
            summary.bindata_files += 1;
        }
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            summary.section_files += 1;
        }
        if !(name.ends_with(".xml") || name.ends_with(".hpf")) {
            continue;
        }
        summary.xml_files += 1;
        let mut text = String::new();
        if file.read_to_string(&mut text).is_err() {
            continue;
        }
        add_marker_count(&mut summary.counts, "hp:tbl", &text, "<hp:tbl");
        add_marker_count(&mut summary.counts, "hp:pic", &text, "<hp:pic");
        add_marker_count(&mut summary.counts, "hp:container", &text, "<hp:container");
        add_marker_count(&mut summary.counts, "hp:ole", &text, "<hp:ole");
        add_marker_count(&mut summary.counts, "hp:chart", &text, "<hp:chart");
        add_marker_count(
            &mut summary.counts,
            "hp:lineSegArray",
            &text,
            "lineSegArray",
        );
        add_marker_count(&mut summary.counts, "hp:lineSeg", &text, "<hp:lineSeg");
        add_marker_count(&mut summary.counts, "hp:p", &text, "<hp:p");
        add_marker_count(&mut summary.counts, "hp:run", &text, "<hp:run");
        add_marker_count(&mut summary.counts, "bindata-ref", &text, "binaryItem");
    }

    Ok(summary)
}

fn add_marker_count(counts: &mut BTreeMap<String, usize>, key: &str, haystack: &str, needle: &str) {
    *counts.entry(key.to_string()).or_insert(0) += haystack.matches(needle).count();
}

fn summarize_ir(doc: &Document) -> IrSummary {
    let mut summary = IrSummary::default();
    summary.sections = doc.sections.len();
    summary.bin_data_items = doc.doc_info.bin_data_list.len();
    summary.bin_data_payloads = doc.bin_data_content.len();
    summary
        .docinfo_counts
        .insert("bin_data".to_string(), doc.doc_info.bin_data_list.len());
    summary
        .docinfo_counts
        .insert("border_fill".to_string(), doc.doc_info.border_fills.len());
    summary
        .docinfo_counts
        .insert("char_shape".to_string(), doc.doc_info.char_shapes.len());
    summary
        .docinfo_counts
        .insert("para_shape".to_string(), doc.doc_info.para_shapes.len());
    summary
        .docinfo_counts
        .insert("style".to_string(), doc.doc_info.styles.len());
    summary.docinfo_counts.insert(
        "extra_records".to_string(),
        doc.doc_info.extra_records.len(),
    );
    summary.docinfo_counts.insert(
        "memo_shape_count".to_string(),
        doc.doc_info.memo_shape_count as usize,
    );

    for section in &doc.sections {
        summarize_paragraphs(&section.paragraphs, &mut summary);
    }
    summary
}

fn summarize_paragraphs(paragraphs: &[Paragraph], summary: &mut IrSummary) {
    summary.paragraphs += paragraphs.len();
    for paragraph in paragraphs {
        summary.line_segs += paragraph.line_segs.len();
        for control in &paragraph.controls {
            summarize_control(control, summary);
        }
    }
}

fn summarize_control(control: &Control, summary: &mut IrSummary) {
    let key = control_kind(control).to_string();
    *summary.controls.entry(key).or_insert(0) += 1;

    match control {
        Control::Table(table) => {
            summary.table_cells += table.cells.len();
            for cell in &table.cells {
                summarize_paragraphs(&cell.paragraphs, summary);
            }
        }
        Control::Header(header) => summarize_paragraphs(&header.paragraphs, summary),
        Control::Footer(footer) => summarize_paragraphs(&footer.paragraphs, summary),
        Control::Footnote(footnote) => summarize_paragraphs(&footnote.paragraphs, summary),
        Control::Endnote(endnote) => summarize_paragraphs(&endnote.paragraphs, summary),
        Control::HiddenComment(comment) => summarize_paragraphs(&comment.paragraphs, summary),
        Control::Shape(shape) => summarize_shape(shape, summary),
        _ => {}
    }
}

fn summarize_shape(shape: &ShapeObject, summary: &mut IrSummary) {
    if let Some(drawing) = shape.drawing() {
        if let Some(text_box) = &drawing.text_box {
            summarize_paragraphs(&text_box.paragraphs, summary);
        }
    }
    if let Some(caption) = shape_caption(shape) {
        summarize_paragraphs(&caption.paragraphs, summary);
    }
    if let ShapeObject::Group(group) = shape {
        *summary
            .controls
            .entry("GroupChild".to_string())
            .or_insert(0) += group.children.len();
        for child in &group.children {
            summarize_shape(child, summary);
        }
    }
}

fn shape_caption(shape: &ShapeObject) -> Option<&crate::model::shape::Caption> {
    match shape {
        ShapeObject::Line(s) => s.drawing.caption.as_ref(),
        ShapeObject::Rectangle(s) => s.drawing.caption.as_ref(),
        ShapeObject::Ellipse(s) => s.drawing.caption.as_ref(),
        ShapeObject::Arc(s) => s.drawing.caption.as_ref(),
        ShapeObject::Polygon(s) => s.drawing.caption.as_ref(),
        ShapeObject::Curve(s) => s.drawing.caption.as_ref(),
        ShapeObject::Group(g) => g.caption.as_ref(),
        ShapeObject::Picture(p) => p.caption.as_ref(),
        ShapeObject::Chart(s) => s.drawing.caption.as_ref(),
        ShapeObject::Ole(s) => s.drawing.caption.as_ref(),
    }
}

fn control_kind(control: &Control) -> &'static str {
    match control {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Table(_) => "Table",
        Control::Shape(_) => "Shape",
        Control::Picture(_) => "Picture",
        Control::Header(_) => "Header",
        Control::Footer(_) => "Footer",
        Control::Footnote(_) => "Footnote",
        Control::Endnote(_) => "Endnote",
        Control::AutoNumber(_) => "AutoNumber",
        Control::NewNumber(_) => "NewNumber",
        Control::PageNumberPos(_) => "PageNumberPos",
        Control::Bookmark(_) => "Bookmark",
        Control::Hyperlink(_) => "Hyperlink",
        Control::Ruby(_) => "Ruby",
        Control::CharOverlap(_) => "CharOverlap",
        Control::PageHide(_) => "PageHide",
        Control::HiddenComment(_) => "HiddenComment",
        Control::Equation(_) => "Equation",
        Control::Field(_) => "Field",
        Control::Form(_) => "Form",
        Control::Unknown(_) => "Unknown",
    }
}

fn docinfo_items(items: &[Hwp5InventoryItem]) -> Vec<&Hwp5InventoryItem> {
    items
        .iter()
        .filter(|item| item.owner == "DocInfo")
        .collect()
}

fn bodytext_items(items: &[Hwp5InventoryItem]) -> Vec<&Hwp5InventoryItem> {
    items
        .iter()
        .filter(|item| item.owner == "BodyText")
        .collect()
}

fn count_tags(items: &[&Hwp5InventoryItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        *counts.entry(item.tag_name.clone()).or_insert(0) += 1;
    }
    counts
}

fn count_tag(items: &[&Hwp5InventoryItem], tag_id: u16) -> usize {
    items.iter().filter(|item| item.tag_id == tag_id).count()
}

fn first_item_by_tag<'a>(
    items: &'a [&Hwp5InventoryItem],
    tag_id: u16,
) -> Option<&'a Hwp5InventoryItem> {
    items.iter().copied().find(|item| item.tag_id == tag_id)
}

fn u32_values(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn id_mapping_field_name(index: usize) -> &'static str {
    match index {
        0 => "bin_data_count",
        1 => "font_hangul_count",
        2 => "font_latin_count",
        3 => "font_hanja_count",
        4 => "font_japanese_count",
        5 => "font_other_count",
        6 => "font_symbol_count",
        7 => "font_user_count",
        8 => "border_fill_count",
        9 => "char_shape_count",
        10 => "tab_def_count",
        11 => "numbering_count",
        12 => "bullet_count",
        13 => "para_shape_count",
        14 => "style_count",
        15 => "memo_shape_count",
        16 => "unknown_count_16",
        17 => "unknown_count_17",
        _ => "unknown_count",
    }
}

fn union_keys<T>(left: &BTreeMap<String, T>, right: &BTreeMap<String, T>) -> Vec<String> {
    let mut keys = BTreeSet::new();
    keys.extend(left.keys().cloned());
    keys.extend(right.keys().cloned());
    keys.into_iter().collect()
}

fn union_ref_keys<T, U>(left: &BTreeMap<String, T>, right: &BTreeMap<String, U>) -> Vec<String> {
    let mut keys = BTreeSet::new();
    keys.extend(left.keys().cloned());
    keys.extend(right.keys().cloned());
    keys.into_iter().collect()
}

fn format_counts(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "-".to_string();
    }
    counts
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_opt_u32(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn short_hash(value: &str) -> String {
    value
        .strip_prefix("blake3:")
        .unwrap_or(value)
        .chars()
        .take(16)
        .collect()
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|")
}
