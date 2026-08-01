//! HWP5 record/control inventory extraction.
//!
//! This module is intentionally record-level. It does not try to prove a
//! lowering contract yet; it produces stable inventory rows that can be diffed
//! in later stages.

use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::cfb_reader::CfbReader;
use crate::parser::header;
use crate::parser::record::Record;
use crate::parser::tags;

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

#[derive(Debug)]
struct Options {
    input: PathBuf,
    format: OutputFormat,
    section: Option<u32>,
    out: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hwp5InventoryItem {
    pub sample: String,
    pub source_path: String,
    pub stream_path: String,
    pub section: Option<u32>,
    pub record_index: usize,
    pub record_uid: String,
    pub level: u16,
    pub tag_id: u16,
    pub tag_name: String,
    pub size: u32,
    pub owner: String,
    pub parent_uid: Option<String>,
    pub parent_scope: Option<String>,
    pub scope_path: String,
    pub body_order: Option<usize>,
    pub control_id: Option<String>,
    pub control_name: Option<String>,
    pub tuple_role: String,
    pub tuple_index: usize,
    pub payload_head_hex: String,
    #[serde(skip_serializing)]
    pub payload_bytes: Vec<u8>,
    pub key_payload: String,
    pub payload_hash: String,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct Hwp5Inventory {
    pub source_path: String,
    pub sample: String,
    pub compressed: bool,
    pub encrypted: bool,
    pub distribution: bool,
    pub section_count: u32,
    pub stream_paths: Vec<String>,
    pub items: Vec<Hwp5InventoryItem>,
}

pub fn run(args: &[String]) -> i32 {
    // 종료 코드 계약(mydocs/manual/cli_commands.md): 명시적 `--help`/`-h` 만 성공(0)이고
    // 인자 누락·파싱 실패는 사용법 오류(2), 실행 실패는 런타임 오류(1)다. 인자 없음을
    // help 와 같은 취급으로 0을 돌려주면 스크립트가 "경로를 안 줬는데 성공"으로 읽는다.
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

    let inventory = match build_inventory(&options.input, options.section) {
        Ok(inventory) => inventory,
        Err(error) => {
            eprintln!("오류: {error}");
            return 1;
        }
    };

    let rendered = match options.format {
        OutputFormat::Jsonl => render_jsonl(&inventory),
        OutputFormat::Markdown => render_markdown(&inventory),
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
    let mut input: Option<PathBuf> = None;
    let mut format = OutputFormat::Markdown;
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
                if input.is_some() {
                    return Err(format!("입력 파일은 하나만 지정할 수 있습니다: {value}"));
                }
                input = Some(PathBuf::from(value));
                i += 1;
            }
        }
    }

    Ok(Options {
        input: input.ok_or_else(|| "입력 HWP 파일 경로가 필요합니다".to_string())?,
        format,
        section,
        out,
    })
}

fn print_usage() {
    // 사용법 안내는 stderr 로 나간다 — stdout 은 인벤토리 데이터(jsonl/md) 전용이라
    // 섞이면 파이프로 받는 소비자가 파싱에서 깨진다.
    eprintln!("사용법:");
    eprintln!("  rhwp hwp5-inventory <파일.hwp> [--format jsonl|md] [--section N] [--out <path>]");
}

pub fn build_inventory(path: &Path, section_filter: Option<u32>) -> Result<Hwp5Inventory, String> {
    let source_path = path.display().to_string();
    let sample = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_string();
    let bytes =
        fs::read(path).map_err(|error| format!("파일 읽기 실패 - {source_path}: {error}"))?;

    let mut cfb = CfbReader::open(&bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    let stream_paths = cfb.list_streams();
    let header_data = cfb
        .read_file_header()
        .map_err(|error| format!("FileHeader 읽기 실패: {error}"))?;
    let file_header = header::parse_file_header(&header_data)
        .map_err(|error| format!("FileHeader 파싱 실패: {error}"))?;

    let compressed = file_header.flags.compressed;
    let encrypted = file_header.flags.encrypted;
    let distribution = file_header.flags.distribution;
    let section_count = cfb.section_count();

    let mut items = Vec::new();
    let doc_info = cfb
        .read_doc_info(compressed)
        .map_err(|error| format!("DocInfo 읽기 실패: {error}"))?;
    append_record_items(
        &mut items,
        &sample,
        &source_path,
        "/DocInfo",
        None,
        "DocInfo",
        &doc_info,
    )?;

    for section in 0..section_count {
        if let Some(filter) = section_filter {
            if section != filter {
                continue;
            }
        }
        let stream_path = body_text_stream_path(&cfb, section, distribution);
        let section_data = cfb
            .read_body_text_section(section, compressed, distribution)
            .map_err(|error| format!("BodyText Section{section} 읽기 실패: {error}"))?;
        append_record_items(
            &mut items,
            &sample,
            &source_path,
            &stream_path,
            Some(section),
            "BodyText",
            &section_data,
        )?;
    }

    Ok(Hwp5Inventory {
        source_path,
        sample,
        compressed,
        encrypted,
        distribution,
        section_count,
        stream_paths,
        items,
    })
}

fn body_text_stream_path(cfb: &CfbReader, section: u32, distribution: bool) -> String {
    let body = format!("/BodyText/Section{section}");
    let view = format!("/ViewText/Section{section}");
    let root = format!("/Section{section}");

    if distribution && cfb.has_stream(&view) {
        view
    } else if cfb.has_stream(&body) {
        body
    } else {
        root
    }
}

fn append_record_items(
    items: &mut Vec<Hwp5InventoryItem>,
    sample: &str,
    source_path: &str,
    stream_path: &str,
    section: Option<u32>,
    owner: &str,
    data: &[u8],
) -> Result<(), String> {
    let records = Record::read_all(data)
        .map_err(|error| format!("{stream_path} record 파싱 실패: {error}"))?;
    let mut scope_stack: Vec<ScopeStackEntry> = Vec::new();
    let mut role_counts: HashMap<&'static str, usize> = HashMap::new();
    let mut body_order_count = 0usize;
    let mut current_body_order = None;

    for (record_index, record) in records.iter().enumerate() {
        scope_stack.retain(|entry| entry.level < record.level);
        if owner == "BodyText" && record.level == 0 {
            current_body_order = Some(body_order_count);
            body_order_count += 1;
        }

        let record_uid = record_uid(stream_path, record_index);
        let parent_uid = scope_stack.last().map(|entry| entry.uid.clone());
        let parent_scope = scope_stack.last().map(|entry| {
            format!(
                "{}#{}@lv{}",
                entry.tag_name, entry.record_index, entry.level
            )
        });
        let scope_path = scope_path(stream_path, &scope_stack, record_index, record);
        let payload_head_hex = payload_head_hex(record);
        let (control_id, control_name) = control_info(record);
        let role = tuple_role(record);
        let tuple_role = role.to_string();
        let tuple_index = {
            let count = role_counts.entry(role).or_insert(0);
            let index = *count;
            *count += 1;
            index
        };

        items.push(Hwp5InventoryItem {
            sample: sample.to_string(),
            source_path: source_path.to_string(),
            stream_path: stream_path.to_string(),
            section,
            record_index,
            record_uid: record_uid.clone(),
            level: record.level,
            tag_id: record.tag_id,
            tag_name: record.tag_name().to_string(),
            size: record.size,
            owner: owner.to_string(),
            parent_uid,
            parent_scope,
            scope_path,
            body_order: if owner == "BodyText" {
                current_body_order
            } else {
                None
            },
            control_id,
            control_name,
            tuple_role,
            tuple_index,
            payload_head_hex,
            payload_bytes: record.data.clone(),
            key_payload: key_payload(record),
            payload_hash: format!("blake3:{}", blake3::hash(&record.data).to_hex()),
            note: None,
        });

        scope_stack.push(ScopeStackEntry {
            level: record.level,
            record_index,
            uid: record_uid,
            tag_name: record.tag_name().to_string(),
        });
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct ScopeStackEntry {
    level: u16,
    record_index: usize,
    uid: String,
    tag_name: String,
}

fn record_uid(stream_path: &str, record_index: usize) -> String {
    let stream = stream_path.trim_start_matches('/').replace('/', ".");
    format!("{stream}#{record_index}")
}

fn scope_path(
    stream_path: &str,
    scope_stack: &[ScopeStackEntry],
    record_index: usize,
    record: &Record,
) -> String {
    let mut parts = vec![stream_path.trim_start_matches('/').to_string()];
    for entry in scope_stack {
        parts.push(format!("{}#{}", entry.tag_name, entry.record_index));
    }
    parts.push(format!("{}#{}", record.tag_name(), record_index));
    parts.join("/")
}

fn tuple_role(record: &Record) -> &'static str {
    match record.tag_id {
        tags::HWPTAG_DOCUMENT_PROPERTIES
        | tags::HWPTAG_ID_MAPPINGS
        | tags::HWPTAG_BIN_DATA
        | tags::HWPTAG_FACE_NAME
        | tags::HWPTAG_BORDER_FILL
        | tags::HWPTAG_CHAR_SHAPE
        | tags::HWPTAG_TAB_DEF
        | tags::HWPTAG_NUMBERING
        | tags::HWPTAG_BULLET
        | tags::HWPTAG_PARA_SHAPE
        | tags::HWPTAG_STYLE
        | tags::HWPTAG_DOC_DATA
        | tags::HWPTAG_DISTRIBUTE_DOC_DATA
        | tags::HWPTAG_COMPATIBLE_DOCUMENT
        | tags::HWPTAG_LAYOUT_COMPATIBILITY
        | tags::HWPTAG_TRACKCHANGE => "docinfo",
        tags::HWPTAG_PARA_HEADER => "para_header",
        tags::HWPTAG_PARA_TEXT => "para_text",
        tags::HWPTAG_PARA_CHAR_SHAPE => "para_char_shape",
        tags::HWPTAG_PARA_LINE_SEG => "para_line_seg",
        tags::HWPTAG_PARA_RANGE_TAG => "para_range_tag",
        tags::HWPTAG_CTRL_HEADER => "ctrl_header",
        tags::HWPTAG_LIST_HEADER => "list_header",
        tags::HWPTAG_PAGE_DEF | tags::HWPTAG_FOOTNOTE_SHAPE | tags::HWPTAG_PAGE_BORDER_FILL => {
            "page_control"
        }
        tags::HWPTAG_TABLE => "table",
        tags::HWPTAG_SHAPE_COMPONENT
        | tags::HWPTAG_SHAPE_COMPONENT_LINE
        | tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE
        | tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE
        | tags::HWPTAG_SHAPE_COMPONENT_ARC
        | tags::HWPTAG_SHAPE_COMPONENT_POLYGON
        | tags::HWPTAG_SHAPE_COMPONENT_CURVE
        | tags::HWPTAG_SHAPE_COMPONENT_OLE
        | tags::HWPTAG_SHAPE_COMPONENT_CONTAINER
        | tags::HWPTAG_SHAPE_COMPONENT_TEXTART => "shape_component",
        tags::HWPTAG_SHAPE_COMPONENT_PICTURE => "pic",
        tags::HWPTAG_CTRL_DATA => "ctrl_data",
        tags::HWPTAG_EQEDIT => "equation",
        tags::HWPTAG_FORM_OBJECT => "form_object",
        tags::HWPTAG_MEMO_SHAPE | tags::HWPTAG_MEMO_LIST => "memo",
        tags::HWPTAG_FORBIDDEN_CHAR => "forbidden_char",
        tags::HWPTAG_CHART_DATA => "chart_data",
        _ => "unknown",
    }
}

fn control_info(record: &Record) -> (Option<String>, Option<String>) {
    if record.tag_id != tags::HWPTAG_CTRL_HEADER || record.data.len() < 4 {
        return (None, None);
    }

    let ctrl_id = u32::from_le_bytes([
        record.data[0],
        record.data[1],
        record.data[2],
        record.data[3],
    ]);
    (
        Some(format!("0x{ctrl_id:08x}")),
        Some(tags::ctrl_name(ctrl_id).to_string()),
    )
}

fn key_payload(record: &Record) -> String {
    let head_len = if record.tag_id == tags::HWPTAG_CTRL_HEADER && record.data.len() <= 128 {
        record.data.len()
    } else {
        record.data.len().min(32)
    };
    let head = hex_bytes(&record.data[..head_len]);

    if record.tag_id == tags::HWPTAG_CTRL_HEADER && record.data.len() >= 4 {
        let ctrl_id = u32::from_le_bytes([
            record.data[0],
            record.data[1],
            record.data[2],
            record.data[3],
        ]);
        return format!(
            "ctrl_id=0x{ctrl_id:08x}({}); head{}={}",
            tags::ctrl_name(ctrl_id),
            head_len,
            head
        );
    }

    format!("head{}={}", head_len, head)
}

fn payload_head_hex(record: &Record) -> String {
    let head_len = record.data.len().min(32);
    hex_bytes(&record.data[..head_len])
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

fn render_jsonl(inventory: &Hwp5Inventory) -> String {
    let mut output = String::new();
    for item in &inventory.items {
        if let Ok(line) = serde_json::to_string(item) {
            output.push_str(&line);
            output.push('\n');
        }
    }
    output
}

fn render_markdown(inventory: &Hwp5Inventory) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Inventory");
    let _ = writeln!(output);
    let _ = writeln!(output, "- source: `{}`", inventory.source_path);
    let _ = writeln!(output, "- sample: `{}`", inventory.sample);
    let _ = writeln!(output, "- compressed: `{}`", inventory.compressed);
    let _ = writeln!(output, "- encrypted: `{}`", inventory.encrypted);
    let _ = writeln!(output, "- distribution: `{}`", inventory.distribution);
    let _ = writeln!(output, "- section_count: `{}`", inventory.section_count);
    let _ = writeln!(output);
    let _ = writeln!(output, "## Streams");
    let _ = writeln!(output);
    for stream in &inventory.stream_paths {
        let _ = writeln!(output, "- `{stream}`");
    }
    let _ = writeln!(output);
    let _ = writeln!(output, "## Records");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| stream | section | idx | uid | level | tag | role | tuple | ctrl | size | owner | parent | scope | key payload | hash |"
    );
    let _ = writeln!(
        output,
        "|---|---:|---:|---|---:|---|---|---:|---|---:|---|---|---|---|---|"
    );

    for item in &inventory.items {
        let section = item
            .section
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let parent = item.parent_scope.as_deref().unwrap_or("-");
        let ctrl = match (&item.control_id, &item.control_name) {
            (Some(id), Some(name)) => format!("{id}/{name}"),
            (Some(id), None) => id.clone(),
            _ => "-".to_string(),
        };
        let hash = item
            .payload_hash
            .strip_prefix("blake3:")
            .unwrap_or(&item.payload_hash);
        let hash_short = hash.get(..16).unwrap_or(hash);
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | `{}` | {} | `{}` | `{}` | {} | `{}` | {} | `{}` | `{}` | `{}` | `{}` | `{}` |",
            escape_md(&item.stream_path),
            section,
            item.record_index,
            escape_md(&item.record_uid),
            item.level,
            item.tag_name,
            item.tuple_role,
            item.tuple_index,
            escape_md(&ctrl),
            item.size,
            escape_md(&item.owner),
            escape_md(parent),
            escape_md(&item.scope_path),
            escape_md(&item.key_payload),
            hash_short
        );
    }

    output
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|")
}
