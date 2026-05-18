//! HWP5 contract probe generator.
//!
//! This command creates focused HWP files by grafting selected HWP5
//! contract records from a Hancom-produced oracle HWP into a generated HWP.
//! It is intentionally narrow: Stage #949 uses it to separate DocInfo
//! MEMO_SHAPE/ID_MAPPINGS issues from missing BodyText CTRL_DATA issues.

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

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out_dir: PathBuf,
    section: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeAxis {
    IdMappings,
    MemoShape,
    ShapeCtrlData,
}

impl ProbeAxis {
    fn as_str(self) -> &'static str {
        match self {
            Self::IdMappings => "id_mappings",
            Self::MemoShape => "memo_shape",
            Self::ShapeCtrlData => "shape_ctrl_data",
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
    id_mappings: usize,
    memo_shape: usize,
    shape_ctrl_data: usize,
    docinfo_touched: usize,
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
        "  rhwp hwp5-contract-probe <oracle.hwp> <generated.hwp> --out-dir <폴더> [--section N]"
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
    for variant in contract_probe_variants() {
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
    let report_path = options.out_dir.join("stage11_generation.md");
    fs::write(&report_path, report).map_err(|error| {
        format!(
            "generation report 쓰기 실패 - {}: {error}",
            report_path.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn contract_probe_variants() -> &'static [Variant] {
    use ProbeAxis::*;
    &[
        Variant {
            name: "01_id_mappings_only",
            axes: &[IdMappings],
        },
        Variant {
            name: "02_memo_shape_only",
            axes: &[MemoShape],
        },
        Variant {
            name: "03_id_mappings_memo_shape",
            axes: &[IdMappings, MemoShape],
        },
        Variant {
            name: "04_shape_ctrl_data_only",
            axes: &[ShapeCtrlData],
        },
        Variant {
            name: "05_id_mappings_ctrl_data",
            axes: &[IdMappings, ShapeCtrlData],
        },
        Variant {
            name: "06_memo_shape_ctrl_data",
            axes: &[MemoShape, ShapeCtrlData],
        },
        Variant {
            name: "07_id_mappings_memo_shape_ctrl_data",
            axes: &[IdMappings, MemoShape, ShapeCtrlData],
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

    if axes.contains(&ProbeAxis::IdMappings) || axes.contains(&ProbeAxis::MemoShape) {
        let oracle_doc_info = read_decompressed_doc_info(oracle_bytes, oracle_compressed)?;
        let generated_doc_info = read_decompressed_doc_info(generated_bytes, generated_compressed)?;
        let (patched_doc_info, doc_counts) =
            patch_doc_info(&oracle_doc_info, &generated_doc_info, axes);
        if doc_counts.id_mappings + doc_counts.memo_shape > 0 {
            let stream_bytes = if generated_compressed {
                compress_stream(&patched_doc_info)?
            } else {
                patched_doc_info
            };
            stream_map.insert("/DocInfo".to_string(), stream_bytes);
            counts.id_mappings += doc_counts.id_mappings;
            counts.memo_shape += doc_counts.memo_shape;
            counts.docinfo_touched = 1;
        }
    }

    if axes.contains(&ProbeAxis::ShapeCtrlData) {
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

            let (patched_section, inserted) =
                patch_missing_ctrl_data(&oracle_section, &generated_section);
            if inserted > 0 {
                let stream_bytes = if generated_compressed {
                    compress_stream(&patched_section)?
                } else {
                    patched_section
                };
                stream_map.insert(path, stream_bytes);
                counts.shape_ctrl_data += inserted;
                counts.sections_touched += 1;
            }
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

fn patch_doc_info(
    oracle_raw: &[u8],
    generated_raw: &[u8],
    axes: &[ProbeAxis],
) -> (Vec<u8>, PatchCounts) {
    let oracle_records = parse_records(oracle_raw);
    let generated_records = parse_records(generated_raw);
    let oracle_id_mappings = oracle_records
        .iter()
        .find(|record| record.tag == tags::HWPTAG_ID_MAPPINGS);
    let oracle_memo_shapes: Vec<&RecordMeta> = oracle_records
        .iter()
        .filter(|record| record.tag == tags::HWPTAG_MEMO_SHAPE)
        .collect();
    let generated_has_memo_shape = generated_records
        .iter()
        .any(|record| record.tag == tags::HWPTAG_MEMO_SHAPE);
    let insert_memo_after = generated_records
        .iter()
        .rposition(|record| record.tag == tags::HWPTAG_STYLE);

    let mut counts = PatchCounts::default();
    let mut out = Vec::with_capacity(generated_raw.len() + oracle_memo_shapes.len() * 64);
    let mut cursor = 0usize;

    for (index, record) in generated_records.iter().enumerate() {
        if cursor < record.offset {
            out.extend_from_slice(&generated_raw[cursor..record.offset]);
        }

        let id_mappings_replacement =
            if axes.contains(&ProbeAxis::IdMappings) && record.tag == tags::HWPTAG_ID_MAPPINGS {
                oracle_id_mappings
            } else {
                None
            };

        if let Some(oracle_record) = id_mappings_replacement {
            out.extend_from_slice(&build_record(
                oracle_record.tag,
                oracle_record.level,
                record_payload(oracle_raw, oracle_record),
            ));
            counts.id_mappings += 1;
        } else {
            out.extend_from_slice(&generated_raw[record.offset..record_end(record)]);
        }

        if axes.contains(&ProbeAxis::MemoShape)
            && !generated_has_memo_shape
            && insert_memo_after == Some(index)
        {
            for oracle_memo_shape in &oracle_memo_shapes {
                out.extend_from_slice(&build_record(
                    oracle_memo_shape.tag,
                    oracle_memo_shape.level,
                    record_payload(oracle_raw, oracle_memo_shape),
                ));
                counts.memo_shape += 1;
            }
        }

        cursor = record_end(record);
    }

    if cursor < generated_raw.len() {
        out.extend_from_slice(&generated_raw[cursor..]);
    }

    (out, counts)
}

fn patch_missing_ctrl_data(oracle_raw: &[u8], generated_raw: &[u8]) -> (Vec<u8>, usize) {
    let oracle_records = parse_records(oracle_raw);
    let generated_records = parse_records(generated_raw);
    let pairs = lcs_pairs(
        &oracle_records,
        oracle_raw,
        &generated_records,
        generated_raw,
    );
    let mut inserts: BTreeMap<usize, Vec<Vec<u8>>> = BTreeMap::new();
    let mut inserted = 0usize;

    for (oracle_index, generated_index) in pairs {
        let Some(oracle_next) = oracle_records.get(oracle_index + 1) else {
            continue;
        };
        if oracle_next.tag != tags::HWPTAG_CTRL_DATA {
            continue;
        }
        if generated_records
            .get(generated_index + 1)
            .is_some_and(|record| record.tag == tags::HWPTAG_CTRL_DATA)
        {
            continue;
        }

        inserts
            .entry(generated_index)
            .or_default()
            .push(build_record(
                oracle_next.tag,
                oracle_next.level,
                record_payload(oracle_raw, oracle_next),
            ));
        inserted += 1;
    }

    if inserts.is_empty() {
        return (generated_raw.to_vec(), 0);
    }

    let mut out = Vec::with_capacity(generated_raw.len() + inserts.len() * 128);
    let mut cursor = 0usize;
    for (index, record) in generated_records.iter().enumerate() {
        if cursor < record.offset {
            out.extend_from_slice(&generated_raw[cursor..record.offset]);
        }
        out.extend_from_slice(&generated_raw[record.offset..record_end(record)]);
        if let Some(records) = inserts.get(&index) {
            for record_bytes in records {
                out.extend_from_slice(record_bytes);
            }
        }
        cursor = record_end(record);
    }
    if cursor < generated_raw.len() {
        out.extend_from_slice(&generated_raw[cursor..]);
    }

    (out, inserted)
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

fn read_decompressed_doc_info(bytes: &[u8], compressed: bool) -> Result<Vec<u8>, String> {
    let mut cfb = CfbReader::open(bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    cfb.read_doc_info(compressed)
        .map_err(|error| format!("DocInfo 읽기 실패: {error}"))
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
    let _ = writeln!(report, "# Task m100 #949 Stage 11 Contract Probe Files");
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
        "| file | bytes | hash | rhwp reload | ID_MAPPINGS | MEMO_SHAPE | CTRL_DATA | DocInfo | sections | axes |"
    );
    let _ = writeln!(report, "|---|---:|---|---|---:|---:|---:|---:|---:|---|");
    for result in results {
        let axes = display_axes_for_file(&result.file_name);
        let _ = writeln!(
            report,
            "| `{}` | {} | `{}` | {} | {} | {} | {} | {} | {} | `{}` |",
            result.file_name,
            result.bytes,
            result.hash,
            result
                .rhwp_pages
                .map(|pages| format!("ok, pages={pages}"))
                .unwrap_or_else(|| "failed".to_string()),
            result.counts.id_mappings,
            result.counts.memo_shape,
            result.counts.shape_ctrl_data,
            result.counts.docinfo_touched,
            result.counts.sections_touched,
            axes
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 3. 작업지시자 판정표");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| variant | 한컴 판정 유형 | 한컴 출력 위치 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |"
    );
    let _ = writeln!(report, "|---|---|---|---|---|---|---|");
    for result in results {
        let variant = result.file_name.trim_end_matches(".hwp");
        let _ = writeln!(report, "| `{}` |  |  |  |  |  |  |", variant);
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 4. 주의");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "이 파일들은 HWPX 저장기 구현 결과가 아니라, generated HWP에 oracle의 HWP5 contract record 일부를 graft한 판정용 probe다."
    );
    report
}

fn short_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().chars().take(16).collect()
}

fn display_axes_for_file(file_name: &str) -> &'static str {
    match file_name.trim_end_matches(".hwp") {
        "01_id_mappings_only" => "ID_MAPPINGS",
        "02_memo_shape_only" => "MEMO_SHAPE",
        "03_id_mappings_memo_shape" => "ID_MAPPINGS + MEMO_SHAPE",
        "04_shape_ctrl_data_only" => "CTRL_DATA",
        "05_id_mappings_ctrl_data" => "ID_MAPPINGS + CTRL_DATA",
        "06_memo_shape_ctrl_data" => "MEMO_SHAPE + CTRL_DATA",
        "07_id_mappings_memo_shape_ctrl_data" => "ID_MAPPINGS + MEMO_SHAPE + CTRL_DATA",
        _ => "-",
    }
}
