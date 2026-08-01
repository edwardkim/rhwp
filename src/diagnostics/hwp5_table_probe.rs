//! HWP5 TABLE field probe generator.
//!
//! This command creates focused HWP files by grafting selected TABLE-related
//! record payload fields from a Hancom-produced oracle HWP into a generated HWP.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::parser::cfb_reader::CfbReader;
use crate::parser::header;
use crate::parser::tags;
use crate::serializer::mini_cfb;
use crate::DocumentCore;

const TABLE_CTRL_ID: u32 = 0x7462_6c20;

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out_dir: PathBuf,
    section: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProbeAxis {
    CtrlOuterMargin,
    CtrlCommonAttr,
    TableAttr,
    TableTail,
}

impl ProbeAxis {
    fn as_str(self) -> &'static str {
        match self {
            Self::CtrlOuterMargin => "ctrl_outer_margin",
            Self::CtrlCommonAttr => "ctrl_common_attr",
            Self::TableAttr => "table_attr",
            Self::TableTail => "table_tail",
        }
    }
}

#[derive(Debug)]
struct Variant {
    name: &'static str,
    axes: &'static [ProbeAxis],
}

#[derive(Debug, Default, Clone)]
struct PatchCounts {
    ctrl_outer_margin: usize,
    ctrl_common_attr: usize,
    table_attr: usize,
    table_tail: usize,
    sections_touched: usize,
}

impl PatchCounts {
    fn add(&mut self, other: &Self) {
        self.ctrl_outer_margin += other.ctrl_outer_margin;
        self.ctrl_common_attr += other.ctrl_common_attr;
        self.table_attr += other.table_attr;
        self.table_tail += other.table_tail;
        self.sections_touched += other.sections_touched;
    }
}

#[derive(Debug)]
struct GeneratedProbe {
    file_name: String,
    bytes: usize,
    rhwp_pages: Option<u32>,
    hash: String,
    counts: PatchCounts,
}

#[derive(Debug, Clone)]
struct RecordMeta {
    tag: u16,
    level: u16,
    size: usize,
    offset: usize,
    data_offset: usize,
}

pub fn run(args: &[String]) -> i32 {
    // 종료 코드 계약(mydocs/manual/cli_commands.md): 명시적 `--help` 만 성공(0)이고,
    // 인자 누락·파싱 실패는 사용법 오류(2), 실행 실패는 런타임 오류(1)다. 종전에는
    // 반환형이 `()` 라 main 의 exit_with 를 통과하지 못해 무슨 일이 나든 0으로 끝났다.
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        print_usage();
        return super::EXIT_OK;
    }
    if args.is_empty() {
        print_usage();
        return super::EXIT_USAGE;
    }

    let options = match parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            return super::EXIT_USAGE;
        }
    };

    if let Err(error) = run_inner(options) {
        eprintln!("오류: {error}");
        return super::EXIT_RUNTIME;
    }
    super::EXIT_OK
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

    if inputs.len() != 2 {
        return Err("oracle HWP와 generated HWP 경로가 필요합니다".to_string());
    }

    Ok(Options {
        oracle: inputs[0].clone(),
        generated: inputs[1].clone(),
        out_dir: out_dir.ok_or_else(|| "--out-dir 경로가 필요합니다".to_string())?,
        section,
    })
}

fn print_usage() {
    // 안내는 stderr 로 나간다(계약). stdout 은 프로브 데이터 전용이라
    // 섞이면 JSONL 을 파이프로 받는 소비자가 파싱에서 깨진다.
    eprintln!("사용법:");
    eprintln!(
        "  rhwp hwp5-table-probe <oracle.hwp> <generated.hwp> --out-dir <폴더> [--section N]"
    );
}

fn run_inner(options: Options) -> Result<(), String> {
    fs::create_dir_all(&options.out_dir).map_err(|error| {
        format!(
            "출력 폴더 생성 실패 - {}: {error}",
            options.out_dir.display()
        )
    })?;

    let oracle_bytes = fs::read(&options.oracle).map_err(|error| {
        format!(
            "oracle 파일 읽기 실패 - {}: {error}",
            options.oracle.display()
        )
    })?;
    let generated_bytes = fs::read(&options.generated).map_err(|error| {
        format!(
            "generated 파일 읽기 실패 - {}: {error}",
            options.generated.display()
        )
    })?;

    let oracle_compressed = hwp_is_compressed(&oracle_bytes)?;
    let generated_compressed = hwp_is_compressed(&generated_bytes)?;
    let variants = table_probe_variants();
    let mut results = Vec::new();

    for variant in variants {
        let (bytes, counts) = build_probe_file(
            &oracle_bytes,
            oracle_compressed,
            &generated_bytes,
            generated_compressed,
            variant.axes,
            options.section,
        )?;
        let file_name = format!("{}.hwp", variant.name);
        let out_path = options.out_dir.join(&file_name);
        fs::write(&out_path, &bytes)
            .map_err(|error| format!("probe 파일 쓰기 실패 - {}: {error}", out_path.display()))?;
        let rhwp_pages = DocumentCore::from_bytes(&bytes)
            .map(|core| core.page_count())
            .ok();
        results.push(GeneratedProbe {
            file_name,
            bytes: bytes.len(),
            rhwp_pages,
            hash: short_hash(&bytes),
            counts,
        });
    }

    let report =
        render_generation_report(&options, oracle_compressed, generated_compressed, &results);
    let report_path = options.out_dir.join("stage9_generation.md");
    fs::write(&report_path, report).map_err(|error| {
        format!(
            "generation report 쓰기 실패 - {}: {error}",
            report_path.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn table_probe_variants() -> &'static [Variant] {
    use ProbeAxis::*;
    &[
        Variant {
            name: "01_ctrl_outer_margin_only",
            axes: &[CtrlOuterMargin],
        },
        Variant {
            name: "02_table_attr_only",
            axes: &[TableAttr],
        },
        Variant {
            name: "03_table_tail_only",
            axes: &[TableTail],
        },
        Variant {
            name: "04_ctrl_common_attr_only",
            axes: &[CtrlCommonAttr],
        },
        Variant {
            name: "05_outer_margin_table_attr",
            axes: &[CtrlOuterMargin, TableAttr],
        },
        Variant {
            name: "06_outer_margin_table_tail",
            axes: &[CtrlOuterMargin, TableTail],
        },
        Variant {
            name: "07_table_attr_tail",
            axes: &[TableAttr, TableTail],
        },
        Variant {
            name: "08_all_table_axes",
            axes: &[CtrlOuterMargin, CtrlCommonAttr, TableAttr, TableTail],
        },
    ]
}

fn build_probe_file(
    oracle_bytes: &[u8],
    oracle_compressed: bool,
    generated_bytes: &[u8],
    generated_compressed: bool,
    axes: &[ProbeAxis],
    section_filter: Option<u32>,
) -> Result<(Vec<u8>, PatchCounts), String> {
    let mut stream_map = read_all_streams(generated_bytes)?;
    let section_count = generated_section_count(generated_bytes)?;
    let mut counts = PatchCounts::default();

    for section in 0..section_count {
        if let Some(filter) = section_filter {
            if section != filter {
                continue;
            }
        }

        let path = format!("/BodyText/Section{section}");
        if !stream_map.contains_key(&path) {
            continue;
        }

        let Some(oracle_section) =
            read_decompressed_section(oracle_bytes, section, oracle_compressed)?
        else {
            continue;
        };
        let Some(generated_section) =
            read_decompressed_section(generated_bytes, section, generated_compressed)?
        else {
            continue;
        };

        let (patched_section, section_counts) =
            patch_section(&oracle_section, &generated_section, axes);
        if section_counts.ctrl_outer_margin
            + section_counts.ctrl_common_attr
            + section_counts.table_attr
            + section_counts.table_tail
            > 0
        {
            let stream_bytes = if generated_compressed {
                compress_stream(&patched_section)?
            } else {
                patched_section
            };
            stream_map.insert(path, stream_bytes);
            let mut section_counts = section_counts;
            section_counts.sections_touched = 1;
            counts.add(&section_counts);
        }
    }

    let mut streams: Vec<(String, Vec<u8>)> = stream_map.into_iter().collect();
    streams.sort_by(|left, right| left.0.cmp(&right.0));
    let named_streams: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(path, data)| (path.as_str(), data.as_slice()))
        .collect();
    let bytes = mini_cfb::build_cfb(&named_streams)
        .map_err(|error| format!("CFB probe 생성 실패: {error}"))?;
    Ok((bytes, counts))
}

fn hwp_is_compressed(bytes: &[u8]) -> Result<bool, String> {
    let mut cfb = CfbReader::open(bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    let header_bytes = cfb
        .read_file_header()
        .map_err(|error| format!("FileHeader 읽기 실패: {error}"))?;
    let file_header = header::parse_file_header(&header_bytes)
        .map_err(|error| format!("FileHeader 파싱 실패: {error}"))?;
    Ok(file_header.flags.compressed)
}

fn generated_section_count(bytes: &[u8]) -> Result<u32, String> {
    let cfb = CfbReader::open(bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    Ok(cfb.section_count())
}

fn read_all_streams(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut cfb = CfbReader::open(bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    let paths = cfb.list_streams();
    let mut streams = BTreeMap::new();
    for path in paths {
        let data = cfb
            .read_stream_raw(&path)
            .map_err(|error| format!("스트림 읽기 실패 - {path}: {error}"))?;
        streams.insert(path, data);
    }
    Ok(streams)
}

fn read_decompressed_section(
    bytes: &[u8],
    section: u32,
    compressed: bool,
) -> Result<Option<Vec<u8>>, String> {
    let mut cfb = CfbReader::open(bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    match cfb.read_body_text_section(section, compressed, false) {
        Ok(data) => Ok(Some(data)),
        Err(_) => Ok(None),
    }
}

fn compress_stream(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|error| format!("deflate 쓰기 실패: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("deflate 종료 실패: {error}"))
}

fn patch_section(
    oracle_raw: &[u8],
    generated_raw: &[u8],
    axes: &[ProbeAxis],
) -> (Vec<u8>, PatchCounts) {
    let oracle_records = parse_records(oracle_raw);
    let generated_records = parse_records(generated_raw);
    let pairs = lcs_pairs(
        &oracle_records,
        oracle_raw,
        &generated_records,
        generated_raw,
    );
    let mut patches: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut counts = PatchCounts::default();

    for (oracle_index, generated_index) in pairs {
        let oracle_record = &oracle_records[oracle_index];
        let generated_record = &generated_records[generated_index];
        let oracle_payload = record_payload(oracle_raw, oracle_record);
        let generated_payload = record_payload(generated_raw, generated_record);
        let mut patched_payload = generated_payload.to_vec();
        let mut changed = false;

        if is_table_ctrl_header(oracle_record, oracle_payload)
            && is_table_ctrl_header(generated_record, generated_payload)
        {
            if axes.contains(&ProbeAxis::CtrlOuterMargin)
                && copy_range(&mut patched_payload, oracle_payload, 0x1c, 0x24)
            {
                counts.ctrl_outer_margin += 1;
                changed = true;
            }
            if axes.contains(&ProbeAxis::CtrlCommonAttr)
                && copy_range(&mut patched_payload, oracle_payload, 0x04, 0x08)
            {
                counts.ctrl_common_attr += 1;
                changed = true;
            }
        } else if oracle_record.tag == tags::HWPTAG_TABLE
            && generated_record.tag == tags::HWPTAG_TABLE
        {
            if axes.contains(&ProbeAxis::TableAttr)
                && copy_range(&mut patched_payload, oracle_payload, 0x00, 0x04)
            {
                counts.table_attr += 1;
                changed = true;
            }
            if axes.contains(&ProbeAxis::TableTail)
                && oracle_payload.len() >= 0x16
                && patched_payload.len() >= 0x16
            {
                let current_tail = patched_payload.get(0x16..).unwrap_or(&[]);
                let oracle_tail = &oracle_payload[0x16..];
                if current_tail != oracle_tail {
                    patched_payload.truncate(0x16);
                    patched_payload.extend_from_slice(oracle_tail);
                    counts.table_tail += 1;
                    changed = true;
                }
            }
        }

        if changed {
            patches.insert(
                generated_index,
                build_record(
                    generated_record.tag,
                    generated_record.level,
                    &patched_payload,
                ),
            );
        }
    }

    let mut out = Vec::with_capacity(generated_raw.len());
    let mut cursor = 0usize;
    for (index, record) in generated_records.iter().enumerate() {
        if cursor < record.offset {
            out.extend_from_slice(&generated_raw[cursor..record.offset]);
        }
        if let Some(patched) = patches.get(&index) {
            out.extend_from_slice(patched);
        } else {
            out.extend_from_slice(&generated_raw[record.offset..record_end(record)]);
        }
        cursor = record_end(record);
    }
    if cursor < generated_raw.len() {
        out.extend_from_slice(&generated_raw[cursor..]);
    }

    (out, counts)
}

fn copy_range(target: &mut [u8], source: &[u8], start: usize, end: usize) -> bool {
    if target.len() < end || source.len() < end {
        return false;
    }
    if target[start..end] == source[start..end] {
        return false;
    }
    target[start..end].copy_from_slice(&source[start..end]);
    true
}

fn parse_records(data: &[u8]) -> Vec<RecordMeta> {
    let mut pos = 0usize;
    let mut records = Vec::new();
    while pos + 4 <= data.len() {
        let header = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = (header & 0x3ff) as u16;
        let level = ((header >> 10) & 0x3ff) as u16;
        let size_field = (header >> 20) & 0xfff;
        let mut size = size_field as usize;
        let mut data_offset = pos + 4;
        if size_field == 0xfff {
            if pos + 8 > data.len() {
                break;
            }
            size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
            data_offset = pos + 8;
        }
        if data_offset + size > data.len() {
            break;
        }
        records.push(RecordMeta {
            tag,
            level,
            size,
            offset: pos,
            data_offset,
        });
        pos = data_offset + size;
    }
    records
}

fn record_end(record: &RecordMeta) -> usize {
    record.data_offset + record.size
}

fn record_payload<'a>(raw: &'a [u8], record: &RecordMeta) -> &'a [u8] {
    &raw[record.data_offset..record_end(record)]
}

fn build_record(tag: u16, level: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 8);
    let size = payload.len() as u32;
    if size < 0xfff {
        let header = tag as u32 | ((level as u32) << 10) | (size << 20);
        out.extend_from_slice(&header.to_le_bytes());
    } else {
        let header = tag as u32 | ((level as u32) << 10) | (0xfff << 20);
        out.extend_from_slice(&header.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
    }
    out.extend_from_slice(payload);
    out
}

fn is_table_ctrl_header(record: &RecordMeta, payload: &[u8]) -> bool {
    record.tag == tags::HWPTAG_CTRL_HEADER && read_u32(payload, 0) == Some(TABLE_CTRL_ID)
}

fn lcs_pairs(
    oracle: &[RecordMeta],
    oracle_raw: &[u8],
    generated: &[RecordMeta],
    generated_raw: &[u8],
) -> Vec<(usize, usize)> {
    if oracle.is_empty() || generated.is_empty() {
        return Vec::new();
    }

    let oracle_signatures: Vec<String> = oracle
        .iter()
        .map(|record| record_signature(record, record_payload(oracle_raw, record)))
        .collect();
    let generated_signatures: Vec<String> = generated
        .iter()
        .map(|record| record_signature(record, record_payload(generated_raw, record)))
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

    let mut pairs = Vec::new();
    let mut i = oracle.len();
    let mut j = generated.len();
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && oracle_signatures[i - 1] == generated_signatures[j - 1] {
            pairs.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i * cols + j - 1] >= dp[(i - 1) * cols + j]) {
            j -= 1;
        } else {
            i -= 1;
        }
    }

    pairs.reverse();
    pairs
}

fn record_signature(record: &RecordMeta, payload: &[u8]) -> String {
    let ctrl = if record.tag == tags::HWPTAG_CTRL_HEADER {
        read_u32(payload, 0)
            .map(|value| format!("{value:08x}"))
            .unwrap_or_else(|| "-".to_string())
    } else {
        "-".to_string()
    };
    format!("tag={}|level={}|ctrl={}", record.tag, record.level, ctrl)
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn render_generation_report(
    options: &Options,
    oracle_compressed: bool,
    generated_compressed: bool,
    results: &[GeneratedProbe],
) -> String {
    let mut report = String::new();
    let _ = writeln!(report, "# Task m100 #949 Stage 9 TABLE Probe Files");
    let _ = writeln!(report);
    let _ = writeln!(report, "## 1. 입력");
    let _ = writeln!(report);
    let _ = writeln!(report, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(report, "- generated: `{}`", options.generated.display());
    let _ = writeln!(report, "- oracle compressed: `{}`", oracle_compressed);
    let _ = writeln!(report, "- generated compressed: `{}`", generated_compressed);
    if let Some(section) = options.section {
        let _ = writeln!(report, "- section filter: `{}`", section);
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 2. 생성 파일");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| file | bytes | hash | rhwp reload | outer margin | ctrl attr | table attr | table tail | sections |"
    );
    let _ = writeln!(report, "|---|---:|---|---|---:|---:|---:|---:|---:|");
    for result in results {
        let _ = writeln!(
            report,
            "| `{}` | {} | `{}` | {} | {} | {} | {} | {} | {} |",
            result.file_name,
            result.bytes,
            result.hash,
            result
                .rhwp_pages
                .map(|pages| format!("ok, pages={pages}"))
                .unwrap_or_else(|| "failed".to_string()),
            result.counts.ctrl_outer_margin,
            result.counts.ctrl_common_attr,
            result.counts.table_attr,
            result.counts.table_tail,
            result.counts.sections_touched
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 3. 작업지시자 판정표");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| variant | 한컴 판정 유형 | 한컴 출력 위치 | 표/셀 배치 | rhwp-studio 판정 | 비고 |"
    );
    let _ = writeln!(report, "|---|---|---|---|---|---|");
    for result in results {
        let variant = result.file_name.trim_end_matches(".hwp");
        let _ = writeln!(report, "| `{}` |  |  |  |  |  |", variant);
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 4. 주의");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "이 파일들은 HWPX 저장기 구현 결과가 아니라, generated HWP의 BodyText TABLE 관련 record payload 일부를 oracle 값으로 graft한 판정용 probe다."
    );
    report
}

fn short_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().chars().take(16).collect()
}
