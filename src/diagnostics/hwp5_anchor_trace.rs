//! Focused HWP5 record trace around a paragraph containing a target text.
//!
//! The inventory diff is intentionally broad. This command is a smaller tool
//! for cases where the visual failure is tied to one anchor paragraph and the
//! meaningful difference may be hidden in raw PARA_TEXT/CTRL_HEADER contracts.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use crate::parser::cfb_reader::CfbReader;
use crate::parser::header;
use crate::parser::record::Record;
use crate::parser::tags;

#[derive(Debug)]
struct Options {
    input: PathBuf,
    needle: String,
    section: u32,
    window: usize,
    out: Option<PathBuf>,
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

    match run_inner(&options) {
        Ok(output) => {
            if let Some(out) = &options.out {
                if let Some(parent) = out.parent() {
                    if let Err(error) = fs::create_dir_all(parent) {
                        eprintln!("오류: 출력 폴더 생성 실패 - {}: {error}", parent.display());
                        return 1;
                    }
                }
                if let Err(error) = fs::write(out, output) {
                    eprintln!("오류: 출력 파일 쓰기 실패 - {}: {error}", out.display());
                    return 1;
                }
            } else {
                print!("{output}");
            }
            0
        }
        Err(error) => {
            eprintln!("오류: {error}");
            1
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut input = None;
    let mut needle = None;
    let mut section = 0u32;
    let mut window = 8usize;
    let mut out = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--needle" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--needle 뒤에 값이 필요합니다".to_string())?;
                needle = Some(value.clone());
                i += 2;
            }
            "--section" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--section 뒤에 번호가 필요합니다".to_string())?;
                section = value
                    .parse::<u32>()
                    .map_err(|_| format!("section 번호가 올바르지 않습니다: {value}"))?;
                i += 2;
            }
            "--window" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--window 뒤에 번호가 필요합니다".to_string())?;
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
        needle: needle.ok_or_else(|| "--needle 값이 필요합니다".to_string())?,
        section,
        window,
        out,
    })
}

fn print_usage() {
    // 사용법 안내는 stderr 로 나간다 — stdout 은 trace 데이터 전용이라
    // 섞이면 파이프로 받는 소비자가 파싱에서 깨진다.
    eprintln!("사용법:");
    eprintln!("  rhwp hwp5-anchor-trace <파일.hwp> --needle <텍스트> [--section N] [--window N] [--out <path>]");
}

fn run_inner(options: &Options) -> Result<String, String> {
    let bytes = fs::read(&options.input)
        .map_err(|error| format!("파일 읽기 실패 - {}: {error}", options.input.display()))?;
    let mut cfb = CfbReader::open(&bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    let header_data = cfb
        .read_file_header()
        .map_err(|error| format!("FileHeader 읽기 실패: {error}"))?;
    let file_header = header::parse_file_header(&header_data)
        .map_err(|error| format!("FileHeader 파싱 실패: {error}"))?;
    let section_data = cfb
        .read_body_text_section(
            options.section,
            file_header.flags.compressed,
            file_header.flags.distribution,
        )
        .map_err(|error| format!("BodyText Section{} 읽기 실패: {error}", options.section))?;
    let records = Record::read_all(&section_data).map_err(|error| {
        format!(
            "BodyText Section{} record 파싱 실패: {error}",
            options.section
        )
    })?;

    let mut hits = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if record.tag_id != tags::HWPTAG_PARA_TEXT {
            continue;
        }
        let decoded = decode_para_text(&record.data);
        if decoded.plain.contains(&options.needle) {
            hits.push((index, decoded));
        }
    }

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 Anchor Trace");
    let _ = writeln!(output);
    let _ = writeln!(output, "- source: `{}`", options.input.display());
    let _ = writeln!(output, "- section: `{}`", options.section);
    let _ = writeln!(output, "- needle: `{}`", options.needle);
    let _ = writeln!(output, "- hits: `{}`", hits.len());
    let _ = writeln!(output);

    for (hit_no, (text_index, decoded)) in hits.iter().enumerate() {
        let para_start = paragraph_start(&records, *text_index);
        let para_end = paragraph_end(&records, para_start);
        let from = para_start.saturating_sub(options.window);
        let to = (para_end + options.window).min(records.len());

        let _ = writeln!(output, "## Hit {}", hit_no + 1);
        let _ = writeln!(output);
        let _ = writeln!(output, "- para_start_record: `{para_start}`");
        let _ = writeln!(output, "- para_end_record_exclusive: `{para_end}`");
        let _ = writeln!(output, "- para_text_record: `{text_index}`");
        let _ = writeln!(output, "- plain: `{}`", escape_inline(&decoded.plain));
        let _ = writeln!(
            output,
            "- annotated: `{}`",
            escape_inline(&decoded.annotated)
        );
        let _ = writeln!(output);
        let _ = writeln!(
            output,
            "| idx | rel | level | tag | size | hash | decoded/key | payload |"
        );
        let _ = writeln!(output, "|---:|---:|---:|---|---:|---|---|---|");

        for index in from..to {
            let record = &records[index];
            let rel = index as isize - para_start as isize;
            let hash = blake3::hash(&record.data).to_hex().to_string();
            let hash_short = hash.get(..16).unwrap_or(&hash);
            let key = record_key(record);
            let payload = hex_bytes(&record.data);
            let _ = writeln!(
                output,
                "| {} | {} | {} | `{}` | {} | `{}` | `{}` | `{}` |",
                index,
                rel,
                record.level,
                record.tag_name(),
                record.size,
                hash_short,
                escape_inline(&key),
                payload
            );
        }
        let _ = writeln!(output);
    }

    Ok(output)
}

fn paragraph_start(records: &[Record], text_index: usize) -> usize {
    let text_level = records[text_index].level;
    records[..=text_index]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, record)| record.tag_id == tags::HWPTAG_PARA_HEADER && record.level < text_level)
        .map(|(index, _)| index)
        .unwrap_or(text_index)
}

fn paragraph_end(records: &[Record], para_start: usize) -> usize {
    let base_level = records[para_start].level;
    records[para_start + 1..]
        .iter()
        .enumerate()
        .find(|(_, record)| record.level <= base_level)
        .map(|(offset, _)| para_start + 1 + offset)
        .unwrap_or(records.len())
}

fn record_key(record: &Record) -> String {
    match record.tag_id {
        tags::HWPTAG_PARA_TEXT => decode_para_text(&record.data).annotated,
        tags::HWPTAG_CTRL_HEADER => ctrl_header_key(&record.data),
        _ => String::new(),
    }
}

fn ctrl_header_key(data: &[u8]) -> String {
    if data.len() < 4 {
        return String::new();
    }
    let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let name = tags::ctrl_name(ctrl_id);
    if data.len() < 11 {
        return format!("ctrl_id=0x{ctrl_id:08x}({name})");
    }

    let command_len = u16::from_le_bytes([data[9], data[10]]) as usize;
    let command_start = 11usize;
    let command_end = command_start + command_len * 2;
    let command = if command_end <= data.len() {
        let chars: Vec<u16> = data[command_start..command_end]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16_lossy(&chars)
    } else {
        String::new()
    };
    let field_id = if command_end + 4 <= data.len() {
        u32::from_le_bytes([
            data[command_end],
            data[command_end + 1],
            data[command_end + 2],
            data[command_end + 3],
        ])
    } else {
        0
    };
    let memo_index = if command_end + 8 <= data.len() {
        u32::from_le_bytes([
            data[command_end + 4],
            data[command_end + 5],
            data[command_end + 6],
            data[command_end + 7],
        ])
    } else {
        0
    };
    let common_attr = if matches!(ctrl_id, tags::CTRL_GEN_SHAPE | tags::CTRL_TABLE) {
        common_obj_attr_summary(u32::from_le_bytes([data[4], data[5], data[6], data[7]]))
    } else {
        String::new()
    };
    format!(
        "ctrl_id=0x{ctrl_id:08x}({name}); properties=0x{:08x}{common_attr}; extra=0x{:02x}; command={command}; field_id={field_id}; memo_index={memo_index}",
        u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        data[8],
    )
}

fn common_obj_attr_summary(attr: u32) -> String {
    let wrap_bits = (attr >> 21) & 0x07;
    let wrap = match wrap_bits {
        0 => "Square/어울림",
        1 => "TopAndBottom/자리차지",
        2 => "BehindText/글뒤로",
        3 => "InFrontOfText/글앞으로",
        _ => "reserved",
    };
    let vrel_bits = (attr >> 3) & 0x03;
    let vrel = match vrel_bits {
        1 => "Page",
        2 => "Para",
        _ => "Paper",
    };
    let valign_bits = (attr >> 5) & 0x07;
    let valign = match valign_bits {
        0 => "Top",
        1 => "Center",
        2 => "Bottom",
        3 => "Inside",
        4 => "Outside",
        _ => "Top",
    };
    let hrel_bits = (attr >> 8) & 0x03;
    let hrel = match hrel_bits {
        1 => "Page",
        2 => "Column",
        3 => "Para",
        _ => "Paper",
    };
    let halign_bits = (attr >> 10) & 0x07;
    let halign = match halign_bits {
        0 => "Left",
        1 => "Center",
        2 => "Right",
        3 => "Inside",
        4 => "Outside",
        _ => "Left",
    };
    format!(
        "; common=tac={}, wrap={}({}), vrel={}({}), valign={}({}), hrel={}({}), halign={}({}), flowWithText={}, allowOverlap={}, bit26={}",
        attr & 0x01 != 0,
        wrap_bits,
        wrap,
        vrel_bits,
        vrel,
        valign_bits,
        valign,
        hrel_bits,
        hrel,
        halign_bits,
        halign,
        attr & (1 << 13) != 0,
        attr & (1 << 14) != 0,
        attr & (1 << 26) != 0
    )
}

#[derive(Debug)]
struct DecodedParaText {
    plain: String,
    annotated: String,
}

fn decode_para_text(data: &[u8]) -> DecodedParaText {
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let mut plain = String::new();
    let mut annotated = String::new();
    let mut i = 0usize;

    while i < units.len() {
        let ch = units[i];
        match ch {
            0x000D => {
                annotated.push_str("<PARA_END>");
                break;
            }
            0x0003 | 0x0004 | 0x000B | 0x000F | 0x0010 | 0x0011 | 0x0012 | 0x0015 | 0x0016
            | 0x0017 => {
                let ctrl_id = if i + 2 < units.len() {
                    let lo = units[i + 1].to_le_bytes();
                    let hi = units[i + 2].to_le_bytes();
                    u32::from_le_bytes([lo[0], lo[1], hi[0], hi[1]])
                } else {
                    0
                };
                let name = if ctrl_id == 0 {
                    "-"
                } else {
                    tags::ctrl_name(ctrl_id)
                };
                let _ = write!(annotated, "<0x{ch:04x}:{name}:0x{ctrl_id:08x}>");
                i += 8;
            }
            0x0009 => {
                plain.push('\t');
                annotated.push_str("<TAB>");
                i += 8;
            }
            0x000A => {
                plain.push('\n');
                annotated.push_str("<LINE_BREAK>");
                i += 1;
            }
            value if value < 0x0020 => {
                let _ = write!(annotated, "<0x{value:04x}>");
                i += 1;
            }
            _ => {
                let start = i;
                i += 1;
                if (0xD800..=0xDBFF).contains(&ch) && i < units.len() {
                    i += 1;
                }
                let text = String::from_utf16_lossy(&units[start..i]);
                plain.push_str(&text);
                annotated.push_str(&text);
            }
        }
    }

    DecodedParaText { plain, annotated }
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

fn escape_inline(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace('\n', "\\n")
        .replace('`', "\\`")
}
