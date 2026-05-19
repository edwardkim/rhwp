//! Focused probe generator for task M100-993 stage 10.
//!
//! It targets the `mel-001` personnel table (`8x12`) after the Stage 9 cell
//! LIST_HEADER materialization. The probes help determine whether Hancom's
//! oversized merged-cell rendering is caused by TABLE tail, cell LIST_HEADER
//! extra data, or cell PARA_HEADER tail contract differences.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::parser::cfb_reader::CfbReader;
use crate::parser::header;
use crate::parser::tags;
use crate::serializer::mini_cfb;
use crate::DocumentCore;

const TARGET_ROWS: u16 = 8;
const TARGET_COLS: u16 = 12;

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out_dir: PathBuf,
    section: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeAxis {
    RemoveListExtra,
    OracleListExtra,
    OracleTableTail,
    OracleParaHeaderTail,
    OracleParaText,
}

#[derive(Debug)]
struct Variant {
    name: &'static str,
    axes: &'static [ProbeAxis],
}

#[derive(Debug, Default, Clone)]
struct PatchCounts {
    table_tail: usize,
    list_extra_removed: usize,
    list_extra_oracle: usize,
    para_header_tail: usize,
    para_text: usize,
    target_blocks: usize,
    sections_touched: usize,
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

#[derive(Debug, Clone)]
struct TableBlock {
    table_index: usize,
    start: usize,
    end: usize,
    rows: u16,
    cols: u16,
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
    println!("사용법:");
    println!(
        "  rhwp hwp5-mel-personnel-probe <oracle.hwp> <generated.hwp> --out-dir <폴더> [--section N]"
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
    let mut results = Vec::new();

    for variant in stage10_variants() {
        let (bytes, counts) = if variant.axes.is_empty() {
            (generated_bytes.clone(), PatchCounts::default())
        } else {
            build_probe_file(
                &oracle_bytes,
                oracle_compressed,
                &generated_bytes,
                generated_compressed,
                variant.axes,
                options.section,
            )?
        };
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
    let report_path = options.out_dir.join("stage10_generation.md");
    fs::write(&report_path, report).map_err(|error| {
        format!(
            "generation report 쓰기 실패 - {}: {error}",
            report_path.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn stage10_variants() -> &'static [Variant] {
    use ProbeAxis::*;
    &[
        Variant {
            name: "01_stage9_current",
            axes: &[],
        },
        Variant {
            name: "02_pi22_without_list_extra",
            axes: &[RemoveListExtra],
        },
        Variant {
            name: "03_pi22_oracle_list_extra",
            axes: &[OracleListExtra],
        },
        Variant {
            name: "04_pi22_table_tail_only",
            axes: &[OracleTableTail],
        },
        Variant {
            name: "05_pi22_para_header_tail",
            axes: &[OracleParaHeaderTail],
        },
        Variant {
            name: "06_pi22_table_tail_plus_oracle_list",
            axes: &[OracleTableTail, OracleListExtra],
        },
        Variant {
            name: "07_pi22_table_list_para",
            axes: &[OracleTableTail, OracleListExtra, OracleParaHeaderTail],
        },
        Variant {
            name: "08_pi22_para_text_only",
            axes: &[OracleParaText],
        },
        Variant {
            name: "09_pi22_text_table_list_para",
            axes: &[
                OracleParaText,
                OracleTableTail,
                OracleListExtra,
                OracleParaHeaderTail,
            ],
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
        if section_counts.table_tail
            + section_counts.list_extra_removed
            + section_counts.list_extra_oracle
            + section_counts.para_header_tail
            + section_counts.para_text
            > 0
        {
            let stream_bytes = if generated_compressed {
                compress_stream(&patched_section)?
            } else {
                patched_section
            };
            stream_map.insert(path, stream_bytes);
            counts.table_tail += section_counts.table_tail;
            counts.list_extra_removed += section_counts.list_extra_removed;
            counts.list_extra_oracle += section_counts.list_extra_oracle;
            counts.para_header_tail += section_counts.para_header_tail;
            counts.para_text += section_counts.para_text;
            counts.target_blocks += section_counts.target_blocks;
            counts.sections_touched += 1;
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
    let scope = target_indices(&generated_records, generated_raw);
    let mut patches: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut counts = PatchCounts {
        target_blocks: scope.target_blocks,
        ..PatchCounts::default()
    };

    if axes.contains(&ProbeAxis::RemoveListExtra) {
        for index in &scope.list_headers {
            let generated_record = &generated_records[*index];
            let generated_payload = record_payload(generated_raw, generated_record);
            let mut patched_payload = generated_payload.to_vec();
            if clear_cell_list_extra(&mut patched_payload) {
                patches.insert(
                    *index,
                    build_record(
                        generated_record.tag,
                        generated_record.level,
                        &patched_payload,
                    ),
                );
                counts.list_extra_removed += 1;
            }
        }
    }

    for (oracle_index, generated_index) in pairs {
        let oracle_record = &oracle_records[oracle_index];
        let generated_record = &generated_records[generated_index];
        let oracle_payload = record_payload(oracle_raw, oracle_record);
        let generated_payload = record_payload(generated_raw, generated_record);
        let mut patched_payload = patches
            .remove(&generated_index)
            .and_then(|record| parse_single_record_payload(&record))
            .unwrap_or_else(|| generated_payload.to_vec());
        let mut changed = false;

        if axes.contains(&ProbeAxis::OracleTableTail)
            && generated_record.tag == tags::HWPTAG_TABLE
            && oracle_record.tag == tags::HWPTAG_TABLE
            && scope.table_records.contains(&generated_index)
            && graft_tail_from_offset(&mut patched_payload, oracle_payload, 0x16)
        {
            counts.table_tail += 1;
            changed = true;
        }

        if axes.contains(&ProbeAxis::OracleListExtra)
            && generated_record.tag == tags::HWPTAG_LIST_HEADER
            && oracle_record.tag == tags::HWPTAG_LIST_HEADER
            && scope.list_headers.contains(&generated_index)
        {
            let width_ref_changed = graft_range(&mut patched_payload, oracle_payload, 6, 8);
            let tail_changed = graft_tail_from_offset(&mut patched_payload, oracle_payload, 34);
            if width_ref_changed || tail_changed {
                counts.list_extra_oracle += 1;
                changed = true;
            }
        }

        if axes.contains(&ProbeAxis::OracleParaHeaderTail)
            && generated_record.tag == tags::HWPTAG_PARA_HEADER
            && oracle_record.tag == tags::HWPTAG_PARA_HEADER
            && scope.para_headers.contains(&generated_index)
            && graft_tail_from_offset(&mut patched_payload, oracle_payload, 20)
        {
            counts.para_header_tail += 1;
            changed = true;
        }

        if axes.contains(&ProbeAxis::OracleParaText)
            && generated_record.tag == tags::HWPTAG_PARA_TEXT
            && oracle_record.tag == tags::HWPTAG_PARA_TEXT
            && scope.para_texts.contains(&generated_index)
            && patched_payload != oracle_payload
        {
            patched_payload.clear();
            patched_payload.extend_from_slice(oracle_payload);
            counts.para_text += 1;
            changed = true;
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

#[derive(Debug, Default)]
struct ScopeIndices {
    table_records: BTreeSet<usize>,
    list_headers: BTreeSet<usize>,
    para_headers: BTreeSet<usize>,
    para_texts: BTreeSet<usize>,
    target_blocks: usize,
}

fn target_indices(records: &[RecordMeta], raw: &[u8]) -> ScopeIndices {
    let mut indices = ScopeIndices::default();
    for block in table_blocks(records, raw)
        .into_iter()
        .filter(|block| block.rows == TARGET_ROWS && block.cols == TARGET_COLS)
    {
        indices.target_blocks += 1;
        indices.table_records.insert(block.table_index);
        for index in block.start..block.end {
            match records[index].tag {
                tags::HWPTAG_LIST_HEADER => {
                    indices.list_headers.insert(index);
                }
                tags::HWPTAG_PARA_HEADER => {
                    indices.para_headers.insert(index);
                }
                tags::HWPTAG_PARA_TEXT => {
                    indices.para_texts.insert(index);
                }
                _ => {}
            }
        }
    }
    indices
}

fn table_blocks(records: &[RecordMeta], raw: &[u8]) -> Vec<TableBlock> {
    let mut blocks = Vec::new();
    for (index, record) in records.iter().enumerate() {
        if record.tag != tags::HWPTAG_TABLE {
            continue;
        }
        let payload = record_payload(raw, record);
        let Some((rows, cols)) = table_rows_cols(payload) else {
            continue;
        };

        let mut end = records.len();
        for (next_index, next_record) in records.iter().enumerate().skip(index + 1) {
            if next_record.level < record.level {
                end = next_index;
                break;
            }
        }

        blocks.push(TableBlock {
            table_index: index,
            start: index,
            end,
            rows,
            cols,
        });
    }
    blocks
}

fn table_rows_cols(payload: &[u8]) -> Option<(u16, u16)> {
    let rows = u16::from_le_bytes(payload.get(4..6)?.try_into().ok()?);
    let cols = u16::from_le_bytes(payload.get(6..8)?.try_into().ok()?);
    Some((rows, cols))
}

fn clear_cell_list_extra(target: &mut Vec<u8>) -> bool {
    let mut changed = false;
    if let Some(width_ref) = target.get_mut(6..8) {
        if width_ref != [0, 0] {
            width_ref.copy_from_slice(&[0, 0]);
            changed = true;
        }
    }
    if target.len() > 34 {
        target.truncate(34);
        changed = true;
    }
    changed
}

fn graft_range(target: &mut [u8], source: &[u8], start: usize, end: usize) -> bool {
    let Some(source_range) = source.get(start..end) else {
        return false;
    };
    let Some(target_range) = target.get_mut(start..end) else {
        return false;
    };
    if target_range == source_range {
        return false;
    }
    target_range.copy_from_slice(source_range);
    true
}

fn graft_tail_from_offset(target: &mut Vec<u8>, source: &[u8], offset: usize) -> bool {
    if source.len() <= offset || target.len() < offset {
        return false;
    }
    if target.get(offset..) == Some(&source[offset..]) {
        return false;
    }
    target.truncate(offset);
    target.extend_from_slice(&source[offset..]);
    true
}

fn parse_single_record_payload(record_bytes: &[u8]) -> Option<Vec<u8>> {
    let record = parse_records(record_bytes).into_iter().next()?;
    Some(record_payload(record_bytes, &record).to_vec())
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
    let _ = writeln!(
        report,
        "# Task M100-993 Stage 10 Personnel Table Probe Files"
    );
    let _ = writeln!(report);
    let _ = writeln!(report, "## 1. 입력");
    let _ = writeln!(report);
    let _ = writeln!(report, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(report, "- generated: `{}`", options.generated.display());
    let _ = writeln!(report, "- oracle compressed: `{}`", oracle_compressed);
    let _ = writeln!(report, "- generated compressed: `{}`", generated_compressed);
    let _ = writeln!(
        report,
        "- target table dimension: `{TARGET_ROWS}x{TARGET_COLS}`"
    );
    if let Some(section) = options.section {
        let _ = writeln!(report, "- section filter: `{}`", section);
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 2. 생성 파일");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| file | bytes | hash | rhwp reload | table tail | list extra removed | list extra oracle | para header tail | para text | target blocks | sections |"
    );
    let _ = writeln!(
        report,
        "|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|"
    );
    for result in results {
        let _ = writeln!(
            report,
            "| `{}` | {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            result.file_name,
            result.bytes,
            result.hash,
            result
                .rhwp_pages
                .map(|pages| format!("ok, pages={pages}"))
                .unwrap_or_else(|| "failed".to_string()),
            result.counts.table_tail,
            result.counts.list_extra_removed,
            result.counts.list_extra_oracle,
            result.counts.para_header_tail,
            result.counts.para_text,
            result.counts.target_blocks,
            result.counts.sections_touched
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 3. 작업지시자 판정표");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| variant | 한컴 판정 유형 | 2페이지 인원현황 표 높이 | 다음 표 페이지 분리 | 1x1 표 배경 | rhwp-studio 판정 | 비고 |"
    );
    let _ = writeln!(report, "|---|---|---|---|---|---|---|");
    for result in results {
        let variant = result.file_name.trim_end_matches(".hwp");
        let _ = writeln!(report, "| `{}` |  |  |  |  |  |  |", variant);
    }
    report
}

fn short_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().chars().take(16).collect()
}
