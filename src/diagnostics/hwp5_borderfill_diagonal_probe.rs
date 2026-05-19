//! Focused probe generator for task M100-993 stage 11.
//!
//! The Stage 10 positive files improved the personnel table height by grafting
//! cell LIST_HEADER materialization, but their diagonal cell borders were still
//! missing. This diagnostic only patches DocInfo BORDER_FILL diagonal contract
//! bytes so the visual judgment can separate diagonal handling from table/cell
//! layout axes.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::parser::cfb_reader::CfbReader;
use crate::parser::header;
use crate::parser::tags;
use crate::serializer::mini_cfb;
use crate::DocumentCore;

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeAxis {
    DiagonalAttr,
    DiagonalPayload,
}

#[derive(Debug)]
struct Variant {
    name: &'static str,
    axes: &'static [ProbeAxis],
}

#[derive(Debug, Default, Clone)]
struct PatchCounts {
    border_fills_seen: usize,
    target_border_fills: usize,
    attr_patches: usize,
    payload_patches: usize,
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
struct BorderFillView {
    id: usize,
    record_index: usize,
    level: u16,
    size: usize,
    attr: u16,
    slash_bits: u16,
    backslash_bits: u16,
    centerline: bool,
    diagonal_type: u8,
    diagonal_width: u8,
    diagonal_color: u32,
    hash: String,
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
    })
}

fn print_usage() {
    println!("사용법:");
    println!("  rhwp hwp5-borderfill-diagonal-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
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
    let oracle_doc_info = read_decompressed_doc_info(&oracle_bytes, oracle_compressed)?;
    let generated_doc_info = read_decompressed_doc_info(&generated_bytes, generated_compressed)?;
    let findings = analyze_border_fills(&oracle_doc_info, &generated_doc_info);

    let mut results = Vec::new();
    for variant in stage11_variants() {
        let (bytes, counts) = if variant.axes.is_empty() {
            (generated_bytes.clone(), PatchCounts::default())
        } else {
            build_probe_file(
                &oracle_bytes,
                oracle_compressed,
                &generated_bytes,
                generated_compressed,
                variant.axes,
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

    let report = render_diff_report(
        &options,
        oracle_compressed,
        generated_compressed,
        &findings,
        &results,
    );
    let report_path = options.out_dir.join("diagonal_borderfill_diff.md");
    fs::write(&report_path, report).map_err(|error| {
        format!(
            "diagonal report 쓰기 실패 - {}: {error}",
            report_path.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn stage11_variants() -> &'static [Variant] {
    use ProbeAxis::*;
    &[
        Variant {
            name: "01_stage10_positive",
            axes: &[],
        },
        Variant {
            name: "02_diagonal_attr_only",
            axes: &[DiagonalAttr],
        },
        Variant {
            name: "03_diagonal_payload_only",
            axes: &[DiagonalPayload],
        },
        Variant {
            name: "04_diagonal_attr_payload",
            axes: &[DiagonalAttr, DiagonalPayload],
        },
        Variant {
            name: "05_diagonal_attr_payload_plus_pi22_list",
            axes: &[DiagonalAttr, DiagonalPayload],
        },
    ]
}

fn build_probe_file(
    oracle_bytes: &[u8],
    oracle_compressed: bool,
    generated_bytes: &[u8],
    generated_compressed: bool,
    axes: &[ProbeAxis],
) -> Result<(Vec<u8>, PatchCounts), String> {
    let mut stream_map = read_all_streams(generated_bytes)?;
    let oracle_doc_info = read_decompressed_doc_info(oracle_bytes, oracle_compressed)?;
    let generated_doc_info = read_decompressed_doc_info(generated_bytes, generated_compressed)?;
    let (patched_doc_info, counts) = patch_doc_info(&oracle_doc_info, &generated_doc_info, axes);

    let stream_bytes = if generated_compressed {
        compress_stream(&patched_doc_info)?
    } else {
        patched_doc_info
    };
    stream_map.insert("/DocInfo".to_string(), stream_bytes);

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

fn patch_doc_info(
    oracle_raw: &[u8],
    generated_raw: &[u8],
    axes: &[ProbeAxis],
) -> (Vec<u8>, PatchCounts) {
    let oracle_records = parse_records(oracle_raw);
    let generated_records = parse_records(generated_raw);
    let oracle_border_fills = border_fill_record_indices(&oracle_records);
    let generated_border_fills = border_fill_record_indices(&generated_records);
    let mut out = generated_raw.to_vec();
    let mut counts = PatchCounts {
        border_fills_seen: generated_border_fills.len(),
        ..PatchCounts::default()
    };

    for (position, (oracle_index, generated_index)) in oracle_border_fills
        .iter()
        .zip(generated_border_fills.iter())
        .enumerate()
    {
        let oracle_record = &oracle_records[*oracle_index];
        let generated_record = &generated_records[*generated_index];
        let oracle_payload = record_payload(oracle_raw, oracle_record);
        let generated_payload = record_payload(generated_raw, generated_record);
        if !is_diagonal_target(oracle_payload, generated_payload) {
            continue;
        }
        counts.target_border_fills += 1;

        let generated_payload_start = generated_record.data_offset;
        if axes.contains(&ProbeAxis::DiagonalAttr)
            && oracle_payload.len() >= 2
            && generated_payload.len() >= 2
            && oracle_payload[0..2] != generated_payload[0..2]
        {
            out[generated_payload_start..generated_payload_start + 2]
                .copy_from_slice(&oracle_payload[0..2]);
            counts.attr_patches += 1;
        }

        if axes.contains(&ProbeAxis::DiagonalPayload)
            && oracle_payload.len() >= 32
            && generated_payload.len() >= 32
            && oracle_payload[26..32] != generated_payload[26..32]
        {
            out[generated_payload_start + 26..generated_payload_start + 32]
                .copy_from_slice(&oracle_payload[26..32]);
            counts.payload_patches += 1;
        }

        let _ = position;
    }

    (out, counts)
}

fn analyze_border_fills(
    oracle_raw: &[u8],
    generated_raw: &[u8],
) -> Vec<(Option<BorderFillView>, Option<BorderFillView>, bool)> {
    let oracle_records = parse_records(oracle_raw);
    let generated_records = parse_records(generated_raw);
    let oracle_views = border_fill_views(&oracle_records, oracle_raw);
    let generated_views = border_fill_views(&generated_records, generated_raw);
    let count = oracle_views.len().max(generated_views.len());
    let mut findings = Vec::new();

    for index in 0..count {
        let oracle = oracle_views.get(index).cloned();
        let generated = generated_views.get(index).cloned();
        let target = match (&oracle, &generated) {
            (Some(o), Some(g)) => {
                diagonal_bits_present(o)
                    || diagonal_bits_present(g)
                    || o.slash_bits != g.slash_bits
                    || o.backslash_bits != g.backslash_bits
                    || o.centerline != g.centerline
            }
            (Some(o), None) => diagonal_bits_present(o),
            (None, Some(g)) => diagonal_bits_present(g),
            (None, None) => false,
        };
        if target {
            findings.push((oracle, generated, target));
        }
    }

    findings
}

fn border_fill_views(records: &[RecordMeta], raw: &[u8]) -> Vec<BorderFillView> {
    let mut views = Vec::new();
    for (position, record_index) in border_fill_record_indices(records).into_iter().enumerate() {
        let record = &records[record_index];
        let payload = record_payload(raw, record);
        views.push(BorderFillView {
            id: position + 1,
            record_index,
            level: record.level,
            size: record.size,
            attr: read_u16(payload, 0).unwrap_or(0),
            slash_bits: read_u16(payload, 0)
                .map(|attr| (attr >> 2) & 0x07)
                .unwrap_or(0),
            backslash_bits: read_u16(payload, 0)
                .map(|attr| (attr >> 5) & 0x07)
                .unwrap_or(0),
            centerline: read_u16(payload, 0)
                .map(|attr| (attr & (1 << 13)) != 0)
                .unwrap_or(false),
            diagonal_type: payload.get(26).copied().unwrap_or(0),
            diagonal_width: payload.get(27).copied().unwrap_or(0),
            diagonal_color: read_u32(payload, 28).unwrap_or(0),
            hash: short_hash(payload),
        });
    }
    views
}

fn border_fill_record_indices(records: &[RecordMeta]) -> Vec<usize> {
    records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| (record.tag == tags::HWPTAG_BORDER_FILL).then_some(index))
        .collect()
}

fn is_diagonal_target(oracle_payload: &[u8], generated_payload: &[u8]) -> bool {
    let oracle_attr = read_u16(oracle_payload, 0).unwrap_or(0);
    let generated_attr = read_u16(generated_payload, 0).unwrap_or(0);
    let oracle_bits = diagonal_attr_mask(oracle_attr);
    let generated_bits = diagonal_attr_mask(generated_attr);
    oracle_bits != 0 || generated_bits != 0 || oracle_bits != generated_bits
}

fn diagonal_bits_present(view: &BorderFillView) -> bool {
    view.slash_bits != 0 || view.backslash_bits != 0 || view.centerline
}

fn diagonal_attr_mask(attr: u16) -> u16 {
    attr & 0x20fc
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

fn read_decompressed_doc_info(bytes: &[u8], compressed: bool) -> Result<Vec<u8>, String> {
    let mut cfb = CfbReader::open(bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    cfb.read_doc_info(compressed)
        .map_err(|error| format!("DocInfo 읽기 실패: {error}"))
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

fn record_payload<'a>(raw: &'a [u8], record: &RecordMeta) -> &'a [u8] {
    &raw[record.data_offset..record.data_offset + record.size]
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

fn render_diff_report(
    options: &Options,
    oracle_compressed: bool,
    generated_compressed: bool,
    findings: &[(Option<BorderFillView>, Option<BorderFillView>, bool)],
    results: &[GeneratedProbe],
) -> String {
    let mut report = String::new();
    let _ = writeln!(report, "# Task M100-993 Stage 11 BorderFill Diagonal Probe");
    let _ = writeln!(report);
    let _ = writeln!(report, "## 1. 입력");
    let _ = writeln!(report);
    let _ = writeln!(report, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(report, "- generated: `{}`", options.generated.display());
    let _ = writeln!(report, "- oracle compressed: `{}`", oracle_compressed);
    let _ = writeln!(report, "- generated compressed: `{}`", generated_compressed);
    let _ = writeln!(
        report,
        "- generated base note: Stage 10 positive already contains the pi22 LIST_HEADER axis."
    );
    let _ = writeln!(report);

    let _ = writeln!(report, "## 2. 대각선 후보 BorderFill");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| id | oracle attr | generated attr | oracle slash/back | generated slash/back | oracle diag | generated diag | oracle hash | generated hash |"
    );
    let _ = writeln!(report, "|---:|---:|---:|---|---|---|---|---|---|");
    if findings.is_empty() {
        let _ = writeln!(report, "| - | - | - | - | - | - | - | - | - |");
    }
    for (oracle, generated, _) in findings {
        let id = oracle
            .as_ref()
            .map(|view| view.id)
            .or_else(|| generated.as_ref().map(|view| view.id))
            .unwrap_or(0);
        let _ = writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} | {} | `{}` | `{}` |",
            id,
            oracle
                .as_ref()
                .map(|view| format!("0x{:04x}", view.attr))
                .unwrap_or_else(|| "-".to_string()),
            generated
                .as_ref()
                .map(|view| format!("0x{:04x}", view.attr))
                .unwrap_or_else(|| "-".to_string()),
            oracle
                .as_ref()
                .map(format_slash_back)
                .unwrap_or_else(|| "-".to_string()),
            generated
                .as_ref()
                .map(format_slash_back)
                .unwrap_or_else(|| "-".to_string()),
            oracle
                .as_ref()
                .map(format_diagonal)
                .unwrap_or_else(|| "-".to_string()),
            generated
                .as_ref()
                .map(format_diagonal)
                .unwrap_or_else(|| "-".to_string()),
            oracle
                .as_ref()
                .map(|view| view.hash.clone())
                .unwrap_or_else(|| "-".to_string()),
            generated
                .as_ref()
                .map(|view| view.hash.clone())
                .unwrap_or_else(|| "-".to_string())
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 3. 생성 파일");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| file | bytes | hash | rhwp reload | target BF | attr patches | payload patches |"
    );
    let _ = writeln!(report, "|---|---:|---|---|---:|---:|---:|");
    for result in results {
        let _ = writeln!(
            report,
            "| `{}` | {} | `{}` | {} | {} | {} | {} |",
            result.file_name,
            result.bytes,
            result.hash,
            result
                .rhwp_pages
                .map(|pages| format!("ok, pages={pages}"))
                .unwrap_or_else(|| "failed".to_string()),
            result.counts.target_border_fills,
            result.counts.attr_patches,
            result.counts.payload_patches
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 4. 작업지시자 판정표");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| variant | 한컴 판정 유형 | 2페이지 인원현황 표 높이 | 셀 대각선 | 1페이지 첫 부분 선 | 1x1 표 배경 | rhwp-studio 판정 | 비고 |"
    );
    let _ = writeln!(report, "|---|---|---|---|---|---|---|---|");
    for result in results {
        let variant = result.file_name.trim_end_matches(".hwp");
        let _ = writeln!(report, "| `{}` |  |  |  |  |  |  |  |", variant);
    }
    report
}

fn format_slash_back(view: &BorderFillView) -> String {
    format!(
        "slash={}, back={}, center={}",
        view.slash_bits, view.backslash_bits, view.centerline
    )
}

fn format_diagonal(view: &BorderFillView) -> String {
    format!(
        "type={}, width={}, color=0x{:08x}, size={}, rec={}, lv={}",
        view.diagonal_type,
        view.diagonal_width,
        view.diagonal_color,
        view.size,
        view.record_index,
        view.level
    )
}

fn short_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().chars().take(16).collect()
}
