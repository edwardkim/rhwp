//! Targeted CTRL_DATA contract tracer.
//!
//! This command does not generate HWP probes. It decodes CTRL_DATA payloads from
//! a Hancom oracle HWP and compares their presence against a generated HWP.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use crate::diagnostics::hwp5_inventory::{build_inventory, Hwp5InventoryItem};
use crate::parser::tags;

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out: PathBuf,
    section: Option<u32>,
    record_index: Option<usize>,
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
    let mut out = None;
    let mut section = None;
    let mut record_index = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" | "-o" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--out 뒤에 경로가 필요합니다".to_string())?;
                out = Some(PathBuf::from(value));
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
            "--record-index" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--record-index 뒤에 번호가 필요합니다".to_string())?;
                record_index = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("record-index가 올바르지 않습니다: {value}"))?,
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

    if inputs.len() != 2 {
        return Err("oracle HWP와 generated HWP 경로가 필요합니다".to_string());
    }

    Ok(Options {
        oracle: inputs[0].clone(),
        generated: inputs[1].clone(),
        out: out.ok_or_else(|| "--out 경로가 필요합니다".to_string())?,
        section,
        record_index,
    })
}

fn print_usage() {
    println!("사용법:");
    println!(
        "  rhwp hwp5-ctrl-data-trace <oracle.hwp> <generated.hwp> --out <path> [--section N] [--record-index N]"
    );
}

fn run_inner(options: Options) -> Result<(), String> {
    let oracle = build_inventory(&options.oracle, options.section)?;
    let generated = build_inventory(&options.generated, options.section)?;

    let oracle_ctrl_data = filter_ctrl_data(&oracle.items, options.record_index);
    let generated_ctrl_data = filter_ctrl_data(&generated.items, options.record_index);

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 CTRL_DATA Trace");
    let _ = writeln!(output);
    let _ = writeln!(output, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(output, "- generated: `{}`", options.generated.display());
    if let Some(section) = options.section {
        let _ = writeln!(output, "- section: `{section}`");
    }
    if let Some(record_index) = options.record_index {
        let _ = writeln!(output, "- record_index filter: `{record_index}`");
    }
    let _ = writeln!(output);

    let _ = writeln!(output, "## Summary");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| side | CTRL_DATA records | total bytes | payload hashes |"
    );
    let _ = writeln!(output, "|---|---:|---:|---|");
    render_summary_row(&mut output, "oracle", &oracle_ctrl_data);
    render_summary_row(&mut output, "generated", &generated_ctrl_data);
    let _ = writeln!(output);

    let _ = writeln!(output, "## Oracle CTRL_DATA Records");
    let _ = writeln!(output);
    if oracle_ctrl_data.is_empty() {
        let _ = writeln!(output, "oracle CTRL_DATA record가 없다.");
    } else {
        for item in &oracle_ctrl_data {
            render_ctrl_data_item(&mut output, item);
        }
    }

    let _ = writeln!(output);
    let _ = writeln!(output, "## Generated CTRL_DATA Records");
    let _ = writeln!(output);
    if generated_ctrl_data.is_empty() {
        let _ = writeln!(output, "generated CTRL_DATA record가 없다.");
    } else {
        for item in &generated_ctrl_data {
            render_ctrl_data_item(&mut output, item);
        }
    }

    if let Some(parent) = options.out.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("출력 폴더 생성 실패 - {}: {error}", parent.display()))?;
    }
    fs::write(&options.out, output)
        .map_err(|error| format!("출력 파일 쓰기 실패 - {}: {error}", options.out.display()))?;

    println!("generated: {}", options.out.display());
    Ok(())
}

fn filter_ctrl_data(
    items: &[Hwp5InventoryItem],
    record_index: Option<usize>,
) -> Vec<&Hwp5InventoryItem> {
    items
        .iter()
        .filter(|item| item.tag_id == tags::HWPTAG_CTRL_DATA)
        .filter(|item| record_index.is_none_or(|index| item.record_index == index))
        .collect()
}

fn render_summary_row(output: &mut String, side: &str, items: &[&Hwp5InventoryItem]) {
    let total_bytes: usize = items.iter().map(|item| item.payload_bytes.len()).sum();
    let hashes = if items.is_empty() {
        "-".to_string()
    } else {
        items
            .iter()
            .map(|item| short_hash(&item.payload_hash).to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let _ = writeln!(
        output,
        "| {side} | {} | {} | `{}` |",
        items.len(),
        total_bytes,
        escape_md(&hashes)
    );
}

fn render_ctrl_data_item(output: &mut String, item: &Hwp5InventoryItem) {
    let _ = writeln!(output, "### `{}`", item.record_uid);
    let _ = writeln!(output);
    let _ = writeln!(output, "| field | value |");
    let _ = writeln!(output, "|---|---|");
    let _ = writeln!(output, "| stream | `{}` |", escape_md(&item.stream_path));
    let _ = writeln!(output, "| record_index | `{}` |", item.record_index);
    let _ = writeln!(output, "| level | `{}` |", item.level);
    let _ = writeln!(output, "| size | `{}` |", item.size);
    let _ = writeln!(
        output,
        "| parent | `{}` |",
        escape_md(item.parent_scope.as_deref().unwrap_or("-"))
    );
    let _ = writeln!(output, "| scope | `{}` |", escape_md(&item.scope_path));
    let _ = writeln!(
        output,
        "| payload_head | `{}` |",
        escape_md(&item.payload_head_hex)
    );
    let _ = writeln!(output, "| hash | `{}` |", short_hash(&item.payload_hash));
    let _ = writeln!(output);
    let _ = writeln!(output, "#### ParameterSet decode");
    let _ = writeln!(output);
    let mut lines = Vec::new();
    match decode_parameter_set(&item.payload_bytes, 0, 0, &mut lines) {
        Ok(consumed) => {
            for line in lines {
                let _ = writeln!(output, "- {}", escape_md(&line));
            }
            if consumed < item.payload_bytes.len() {
                let _ = writeln!(
                    output,
                    "- trailing bytes: offset={} len={} `{}`",
                    consumed,
                    item.payload_bytes.len() - consumed,
                    hex_bytes(&item.payload_bytes[consumed..])
                );
            }
        }
        Err(error) => {
            let _ = writeln!(output, "- decode error: {}", escape_md(&error));
            let _ = writeln!(output, "- raw: `{}`", hex_bytes(&item.payload_bytes));
        }
    }
    let _ = writeln!(output);
}

fn decode_parameter_set(
    data: &[u8],
    offset: usize,
    depth: usize,
    lines: &mut Vec<String>,
) -> Result<usize, String> {
    if offset + 6 > data.len() {
        return Err(format!("ParameterSet header 부족: offset={offset}"));
    }

    let ps_id = read_u16(data, offset)?;
    let count = read_u16(data, offset + 2)?;
    let dummy = read_u16(data, offset + 4)?;
    lines.push(format!(
        "{}ParameterSet ps_id=0x{ps_id:04x} count={count} dummy=0x{dummy:04x}",
        indent(depth)
    ));

    let mut cursor = offset + 6;
    for index in 0..count {
        if cursor + 4 > data.len() {
            return Err(format!(
                "ParameterItem header 부족: index={index} offset={cursor}"
            ));
        }
        let item_id = read_u16(data, cursor)?;
        let item_type = read_u16(data, cursor + 2)?;
        cursor += 4;
        lines.push(format!(
            "{}item#{index} id=0x{item_id:04x} type=0x{item_type:04x}({})",
            indent(depth + 1),
            param_type_name(item_type)
        ));

        match item_type {
            0x0000 => {}
            0x0001 => {
                let len = read_u16(data, cursor)? as usize;
                cursor += 2;
                let byte_len = len.saturating_mul(2);
                if cursor + byte_len > data.len() {
                    return Err(format!("String value 부족: len={len} offset={cursor}"));
                }
                let value = decode_utf16le(&data[cursor..cursor + byte_len]);
                lines.push(format!(
                    "{}string len={len} value=\"{value}\"",
                    indent(depth + 2)
                ));
                cursor += byte_len;
            }
            0x0002..=0x0009 => {
                if cursor + 4 > data.len() {
                    return Err(format!("integer value 부족: offset={cursor}"));
                }
                let value = read_u32(data, cursor)?;
                lines.push(format!("{}u32=0x{value:08x} ({value})", indent(depth + 2)));
                cursor += 4;
            }
            0x8000 => {
                cursor = decode_parameter_set(data, cursor, depth + 2, lines)?;
            }
            0x8001 => {
                if cursor + 4 > data.len() {
                    return Err(format!("array header 부족: offset={cursor}"));
                }
                let array_count = read_u16(data, cursor)?;
                let array_item_id = read_u16(data, cursor + 2)?;
                lines.push(format!(
                    "{}array count={array_count} item_id=0x{array_item_id:04x} (raw tail not expanded)",
                    indent(depth + 2)
                ));
                cursor += 4;
            }
            0x8002 => {
                let value = read_u16(data, cursor)?;
                lines.push(format!("{}bindata_id={value}", indent(depth + 2)));
                cursor += 2;
            }
            _ => {
                return Err(format!(
                    "지원하지 않는 ParameterItem type=0x{item_type:04x} offset={cursor}"
                ));
            }
        }
    }

    Ok(cursor)
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    if offset + 2 > data.len() {
        return Err(format!("u16 읽기 범위 초과: offset={offset}"));
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    if offset + 4 > data.len() {
        return Err(format!("u32 읽기 범위 초과: offset={offset}"));
    }
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

fn decode_utf16le(data: &[u8]) -> String {
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

fn param_type_name(item_type: u16) -> &'static str {
    match item_type {
        0x0000 => "NULL",
        0x0001 => "String",
        0x0002 => "Integer1",
        0x0003 => "Integer2",
        0x0004 => "Integer4",
        0x0005 => "Integer",
        0x0006 => "UnsignedInteger1",
        0x0007 => "UnsignedInteger2",
        0x0008 => "UnsignedInteger4",
        0x0009 => "UnsignedInteger",
        0x8000 => "ParameterSet",
        0x8001 => "Array",
        0x8002 => "BINDataID",
        _ => "Unknown",
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut result = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            result.push(' ');
        }
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn short_hash(hash: &str) -> &str {
    let stripped = hash.strip_prefix("blake3:").unwrap_or(hash);
    stripped.get(..16).unwrap_or(stripped)
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
