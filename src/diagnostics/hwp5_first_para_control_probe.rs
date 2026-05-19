//! Focused first-paragraph control probe for task M100-993 stage 12.
//!
//! The Stage 11 files fixed the diagonal cell border axis, but Hancom still
//! displayed a ghost line near the first paragraph controls. This diagnostic
//! compares the first BodyText paragraph bundle against a Hancom-produced
//! oracle and generates limited graft probes around paragraph text, char-shape
//! offsets, and top-level control/header records.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::ops::Range;
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
    section: u32,
    mode: ProbeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeMode {
    Stage12,
    TripletMatrix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeKind {
    Baseline,
    OracleParaHeader,
    OracleParaText,
    OracleParaCharShape,
    OracleParaHeaderText,
    OracleParaHeaderCharShape,
    OracleParaTextCharShape,
    OracleParaHeaderTextCharShape,
    OracleTopLevelCtrlHeaders,
    OracleHeaderSubtree,
    OracleFirstParaBundle,
}

#[derive(Debug)]
struct Variant {
    name: &'static str,
    description: &'static str,
    kind: ProbeKind,
}

#[derive(Debug, Default, Clone)]
struct PatchCounts {
    records_replaced: usize,
    byte_ranges_replaced: usize,
    generated_bytes_removed: usize,
    oracle_bytes_inserted: usize,
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
struct FirstParaView {
    range: Range<usize>,
    base_level: u16,
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
    let mut section = 0u32;
    let mut mode = ProbeMode::Stage12;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--triplet-matrix" => {
                mode = ProbeMode::TripletMatrix;
                i += 1;
            }
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
                section = value
                    .parse::<u32>()
                    .map_err(|_| format!("section 번호가 올바르지 않습니다: {value}"))?;
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
        mode,
    })
}

fn print_usage() {
    println!("사용법:");
    println!(
        "  rhwp hwp5-first-para-control-probe <oracle.hwp> <generated.hwp> --out-dir <폴더> [--section N]"
    );
    println!(
        "  옵션: --triplet-matrix  # Stage 14 PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE 조합 probe"
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
    let oracle_section =
        read_required_section(&oracle_bytes, options.section, oracle_compressed, "oracle")?;
    let generated_section = read_required_section(
        &generated_bytes,
        options.section,
        generated_compressed,
        "generated",
    )?;
    let oracle_records = parse_records(&oracle_section);
    let generated_records = parse_records(&generated_section);
    let oracle_first = first_para_view(&oracle_records)
        .ok_or_else(|| "oracle first paragraph bundle을 찾지 못했습니다".to_string())?;
    let generated_first = first_para_view(&generated_records)
        .ok_or_else(|| "generated first paragraph bundle을 찾지 못했습니다".to_string())?;

    let mut results = Vec::new();
    let variants = variants_for_mode(options.mode);
    for variant in variants {
        let (bytes, counts) = if variant.kind == ProbeKind::Baseline {
            (generated_bytes.clone(), PatchCounts::default())
        } else {
            build_probe_file(
                &oracle_bytes,
                oracle_compressed,
                &generated_bytes,
                generated_compressed,
                options.section,
                variant.kind,
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

    let report = render_trace_report(
        &options,
        oracle_compressed,
        generated_compressed,
        &oracle_section,
        &oracle_records,
        &oracle_first,
        &generated_section,
        &generated_records,
        &generated_first,
        &results,
        variants,
    );
    let report_path = options.out_dir.join("first_para_control_trace.md");
    fs::write(&report_path, report).map_err(|error| {
        format!(
            "first paragraph trace 쓰기 실패 - {}: {error}",
            report_path.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn variants_for_mode(mode: ProbeMode) -> &'static [Variant] {
    match mode {
        ProbeMode::Stage12 => stage12_variants(),
        ProbeMode::TripletMatrix => triplet_matrix_variants(),
    }
}

fn stage12_variants() -> &'static [Variant] {
    use ProbeKind::*;
    &[
        Variant {
            name: "01_stage11_positive",
            description: "Stage 11 positive 기준 파일",
            kind: Baseline,
        },
        Variant {
            name: "02_oracle_first_para_char_shape",
            description: "첫 문단 PARA_CHAR_SHAPE만 정답지로 교체",
            kind: OracleParaCharShape,
        },
        Variant {
            name: "03_oracle_first_para_head_text_char_shape",
            description: "첫 문단 PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE 교체",
            kind: OracleParaHeaderTextCharShape,
        },
        Variant {
            name: "04_oracle_top_level_ctrl_headers",
            description: "첫 문단 상위 CTRL_HEADER payload만 교체",
            kind: OracleTopLevelCtrlHeaders,
        },
        Variant {
            name: "05_oracle_header_subtree",
            description: "첫 문단 머리말 control subtree 교체",
            kind: OracleHeaderSubtree,
        },
        Variant {
            name: "06_oracle_first_para_bundle",
            description: "첫 문단 record bundle 전체 교체",
            kind: OracleFirstParaBundle,
        },
    ]
}

fn triplet_matrix_variants() -> &'static [Variant] {
    use ProbeKind::*;
    &[
        Variant {
            name: "01_stage11_positive",
            description: "Stage 11 positive 기준 파일",
            kind: Baseline,
        },
        Variant {
            name: "02_oracle_para_header",
            description: "첫 문단 PARA_HEADER만 정답지로 교체",
            kind: OracleParaHeader,
        },
        Variant {
            name: "03_oracle_para_text",
            description: "첫 문단 PARA_TEXT만 정답지로 교체",
            kind: OracleParaText,
        },
        Variant {
            name: "04_oracle_para_char_shape",
            description: "첫 문단 PARA_CHAR_SHAPE만 정답지로 교체",
            kind: OracleParaCharShape,
        },
        Variant {
            name: "05_oracle_para_header_text",
            description: "첫 문단 PARA_HEADER/PARA_TEXT 교체",
            kind: OracleParaHeaderText,
        },
        Variant {
            name: "06_oracle_para_header_char_shape",
            description: "첫 문단 PARA_HEADER/PARA_CHAR_SHAPE 교체",
            kind: OracleParaHeaderCharShape,
        },
        Variant {
            name: "07_oracle_para_text_char_shape",
            description: "첫 문단 PARA_TEXT/PARA_CHAR_SHAPE 교체",
            kind: OracleParaTextCharShape,
        },
        Variant {
            name: "08_oracle_para_header_text_char_shape",
            description: "첫 문단 PARA_HEADER/PARA_TEXT/PARA_CHAR_SHAPE 교체",
            kind: OracleParaHeaderTextCharShape,
        },
    ]
}

fn build_probe_file(
    oracle_bytes: &[u8],
    oracle_compressed: bool,
    generated_bytes: &[u8],
    generated_compressed: bool,
    section: u32,
    kind: ProbeKind,
) -> Result<(Vec<u8>, PatchCounts), String> {
    let mut stream_map = read_all_streams(generated_bytes)?;
    let oracle_section = read_required_section(oracle_bytes, section, oracle_compressed, "oracle")?;
    let generated_section =
        read_required_section(generated_bytes, section, generated_compressed, "generated")?;
    let (patched_section, counts) =
        patch_first_para_section(&oracle_section, &generated_section, kind)?;
    let stream_bytes = if generated_compressed {
        compress_stream(&patched_section)?
    } else {
        patched_section
    };
    stream_map.insert(format!("/BodyText/Section{section}"), stream_bytes);

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

fn patch_first_para_section(
    oracle_raw: &[u8],
    generated_raw: &[u8],
    kind: ProbeKind,
) -> Result<(Vec<u8>, PatchCounts), String> {
    let oracle_records = parse_records(oracle_raw);
    let generated_records = parse_records(generated_raw);
    let oracle_first = first_para_view(&oracle_records)
        .ok_or_else(|| "oracle first paragraph bundle을 찾지 못했습니다".to_string())?;
    let generated_first = first_para_view(&generated_records)
        .ok_or_else(|| "generated first paragraph bundle을 찾지 못했습니다".to_string())?;

    match kind {
        ProbeKind::Baseline => Ok((generated_raw.to_vec(), PatchCounts::default())),
        ProbeKind::OracleFirstParaBundle => replace_record_range(
            oracle_raw,
            &oracle_records,
            oracle_first.range,
            generated_raw,
            &generated_records,
            generated_first.range,
        ),
        ProbeKind::OracleHeaderSubtree => {
            let oracle_header = find_control_subtree(
                &oracle_records,
                &oracle_first,
                oracle_raw,
                tags::CTRL_HEADER,
            )
            .ok_or_else(|| "oracle first paragraph Header subtree를 찾지 못했습니다".to_string())?;
            let generated_header = find_control_subtree(
                &generated_records,
                &generated_first,
                generated_raw,
                tags::CTRL_HEADER,
            )
            .ok_or_else(|| {
                "generated first paragraph Header subtree를 찾지 못했습니다".to_string()
            })?;
            replace_record_range(
                oracle_raw,
                &oracle_records,
                oracle_header,
                generated_raw,
                &generated_records,
                generated_header,
            )
        }
        ProbeKind::OracleParaCharShape
        | ProbeKind::OracleParaHeader
        | ProbeKind::OracleParaText
        | ProbeKind::OracleParaHeaderText
        | ProbeKind::OracleParaHeaderCharShape
        | ProbeKind::OracleParaTextCharShape
        | ProbeKind::OracleParaHeaderTextCharShape
        | ProbeKind::OracleTopLevelCtrlHeaders => {
            let patches = record_patches_for_kind(
                oracle_raw,
                &oracle_records,
                &oracle_first,
                generated_raw,
                &generated_records,
                &generated_first,
                kind,
            )?;
            let counts = PatchCounts {
                records_replaced: patches.len(),
                ..PatchCounts::default()
            };
            Ok((
                apply_record_patches(generated_raw, &generated_records, patches),
                counts,
            ))
        }
    }
}

fn record_patches_for_kind(
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
    kind: ProbeKind,
) -> Result<BTreeMap<usize, Vec<u8>>, String> {
    let mut patches = BTreeMap::new();

    match kind {
        ProbeKind::OracleParaHeader => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_HEADER,
                &mut patches,
            )?;
        }
        ProbeKind::OracleParaText => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_TEXT,
                &mut patches,
            )?;
        }
        ProbeKind::OracleParaCharShape => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_CHAR_SHAPE,
                &mut patches,
            )?;
        }
        ProbeKind::OracleParaHeaderText => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_HEADER,
                &mut patches,
            )?;
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_TEXT,
                &mut patches,
            )?;
        }
        ProbeKind::OracleParaHeaderCharShape => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_HEADER,
                &mut patches,
            )?;
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_CHAR_SHAPE,
                &mut patches,
            )?;
        }
        ProbeKind::OracleParaTextCharShape => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_TEXT,
                &mut patches,
            )?;
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_CHAR_SHAPE,
                &mut patches,
            )?;
        }
        ProbeKind::OracleParaHeaderTextCharShape => {
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_HEADER,
                &mut patches,
            )?;
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_TEXT,
                &mut patches,
            )?;
            patch_matching_first_tag(
                oracle_raw,
                oracle_records,
                oracle_first,
                generated_records,
                generated_first,
                tags::HWPTAG_PARA_CHAR_SHAPE,
                &mut patches,
            )?;
        }
        ProbeKind::OracleTopLevelCtrlHeaders => {
            let oracle_by_ctrl =
                top_level_ctrl_headers_by_id(oracle_raw, oracle_records, oracle_first);
            for generated_index in generated_first.range.clone() {
                let generated_record = &generated_records[generated_index];
                if generated_record.tag != tags::HWPTAG_CTRL_HEADER
                    || generated_record.level != generated_first.base_level + 1
                {
                    continue;
                }
                let generated_payload = record_payload(generated_raw, generated_record);
                let Some(ctrl_id) = read_u32(generated_payload, 0) else {
                    continue;
                };
                let Some(&oracle_index) = oracle_by_ctrl.get(&ctrl_id) else {
                    continue;
                };
                let oracle_payload = record_payload(oracle_raw, &oracle_records[oracle_index]);
                patches.insert(
                    generated_index,
                    build_record(generated_record.tag, generated_record.level, oracle_payload),
                );
            }
        }
        ProbeKind::Baseline | ProbeKind::OracleHeaderSubtree | ProbeKind::OracleFirstParaBundle => {
        }
    }

    if patches.is_empty() {
        return Err(format!("patch 대상 record를 찾지 못했습니다: {kind:?}"));
    }

    Ok(patches)
}

fn patch_matching_first_tag(
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
    tag: u16,
    patches: &mut BTreeMap<usize, Vec<u8>>,
) -> Result<(), String> {
    let oracle_index =
        find_first_para_record(oracle_records, oracle_first, tag).ok_or_else(|| {
            format!(
                "oracle first paragraph에서 {} record를 찾지 못했습니다",
                tags::tag_name(tag)
            )
        })?;
    let generated_index = find_first_para_record(generated_records, generated_first, tag)
        .ok_or_else(|| {
            format!(
                "generated first paragraph에서 {} record를 찾지 못했습니다",
                tags::tag_name(tag)
            )
        })?;
    let oracle_payload = record_payload(oracle_raw, &oracle_records[oracle_index]);
    let generated_record = &generated_records[generated_index];
    patches.insert(
        generated_index,
        build_record(generated_record.tag, generated_record.level, oracle_payload),
    );
    Ok(())
}

fn find_first_para_record(
    records: &[RecordMeta],
    first: &FirstParaView,
    tag: u16,
) -> Option<usize> {
    first.range.clone().find(|&index| records[index].tag == tag)
}

fn top_level_ctrl_headers_by_id<'a>(
    raw: &'a [u8],
    records: &'a [RecordMeta],
    first: &'a FirstParaView,
) -> BTreeMap<u32, usize> {
    let mut by_ctrl = BTreeMap::new();
    for index in first.range.clone() {
        let record = &records[index];
        if record.tag != tags::HWPTAG_CTRL_HEADER || record.level != first.base_level + 1 {
            continue;
        }
        if let Some(ctrl_id) = read_u32(record_payload(raw, record), 0) {
            by_ctrl.entry(ctrl_id).or_insert(index);
        }
    }
    by_ctrl
}

fn find_control_subtree(
    records: &[RecordMeta],
    first: &FirstParaView,
    raw: &[u8],
    ctrl_id: u32,
) -> Option<Range<usize>> {
    for start in first.range.clone() {
        let record = &records[start];
        if record.tag != tags::HWPTAG_CTRL_HEADER || record.level != first.base_level + 1 {
            continue;
        }
        if read_u32(record_payload(raw, record), 0)? != ctrl_id {
            continue;
        }
        let mut end = first.range.end;
        for next in start + 1..first.range.end {
            if records[next].level <= record.level {
                end = next;
                break;
            }
        }
        return Some(start..end);
    }
    None
}

fn replace_record_range(
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_range: Range<usize>,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_range: Range<usize>,
) -> Result<(Vec<u8>, PatchCounts), String> {
    if oracle_range.is_empty() || generated_range.is_empty() {
        return Err("빈 record range는 교체할 수 없습니다".to_string());
    }
    let oracle_bytes = records_byte_range(oracle_records, oracle_range.clone());
    let generated_bytes = records_byte_range(generated_records, generated_range.clone());
    let mut out =
        Vec::with_capacity(generated_raw.len() - generated_bytes.len() + oracle_bytes.len());
    out.extend_from_slice(&generated_raw[..generated_bytes.start]);
    out.extend_from_slice(&oracle_raw[oracle_bytes.clone()]);
    out.extend_from_slice(&generated_raw[generated_bytes.end..]);
    Ok((
        out,
        PatchCounts {
            records_replaced: generated_range.len(),
            byte_ranges_replaced: 1,
            generated_bytes_removed: generated_bytes.len(),
            oracle_bytes_inserted: oracle_bytes.len(),
        },
    ))
}

fn apply_record_patches(
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    patches: BTreeMap<usize, Vec<u8>>,
) -> Vec<u8> {
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
    out
}

fn first_para_view(records: &[RecordMeta]) -> Option<FirstParaView> {
    let start = records
        .iter()
        .position(|record| record.tag == tags::HWPTAG_PARA_HEADER)?;
    let base_level = records[start].level;
    let mut end = records.len();
    for (index, record) in records.iter().enumerate().skip(start + 1) {
        if record.tag == tags::HWPTAG_PARA_HEADER && record.level == base_level {
            end = index;
            break;
        }
    }
    Some(FirstParaView {
        range: start..end,
        base_level,
    })
}

fn records_byte_range(records: &[RecordMeta], range: Range<usize>) -> Range<usize> {
    let start = records[range.start].offset;
    let end = record_end(&records[range.end - 1]);
    start..end
}

fn render_trace_report(
    options: &Options,
    oracle_compressed: bool,
    generated_compressed: bool,
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
    results: &[GeneratedProbe],
    variants: &[Variant],
) -> String {
    let mut report = String::new();
    let title = match options.mode {
        ProbeMode::Stage12 => "Task M100-993 Stage 12 First Paragraph Control Trace",
        ProbeMode::TripletMatrix => "Task M100-993 Stage 14 First Paragraph Triplet Matrix",
    };
    let _ = writeln!(report, "# {title}");
    let _ = writeln!(report);
    let _ = writeln!(report, "## 1. 입력");
    let _ = writeln!(report);
    let _ = writeln!(report, "- oracle: `{}`", options.oracle.display());
    let _ = writeln!(report, "- generated: `{}`", options.generated.display());
    let _ = writeln!(report, "- section: `{}`", options.section);
    let _ = writeln!(report, "- oracle compressed: `{}`", oracle_compressed);
    let _ = writeln!(report, "- generated compressed: `{}`", generated_compressed);
    let _ = writeln!(report);

    let _ = writeln!(report, "## 2. 생성 파일");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| file | bytes | hash | rhwp reload | records | ranges | generated bytes removed | oracle bytes inserted |"
    );
    let _ = writeln!(report, "|---|---:|---|---|---:|---:|---:|---:|");
    for result in results {
        let _ = writeln!(
            report,
            "| `{}` | {} | `{}` | {} | {} | {} | {} | {} |",
            result.file_name,
            result.bytes,
            result.hash,
            result
                .rhwp_pages
                .map(|pages| format!("ok, pages={pages}"))
                .unwrap_or_else(|| "failed".to_string()),
            result.counts.records_replaced,
            result.counts.byte_ranges_replaced,
            result.counts.generated_bytes_removed,
            result.counts.oracle_bytes_inserted
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 3. 판정표");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| file | 한컴 판정 유형 | 1페이지 고스트 선 | 표/셀 배치 | rhwp-studio 판정 | 비고 |"
    );
    let _ = writeln!(report, "|---|---|---|---|---|---|");
    for variant in variants {
        let _ = writeln!(
            report,
            "| `{}.hwp` |  |  |  |  | {} |",
            variant.name, variant.description
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 4. 첫 문단 record bundle 비교");
    let _ = writeln!(report);
    render_record_table(
        &mut report,
        "oracle",
        oracle_raw,
        oracle_records,
        oracle_first,
    );
    render_record_table(
        &mut report,
        "generated",
        generated_raw,
        generated_records,
        generated_first,
    );

    let _ = writeln!(report, "## 5. 핵심 차이");
    let _ = writeln!(report);
    render_para_header_summary(
        &mut report,
        oracle_raw,
        oracle_records,
        oracle_first,
        generated_raw,
        generated_records,
        generated_first,
    );
    render_para_text_markers(
        &mut report,
        oracle_raw,
        oracle_records,
        oracle_first,
        generated_raw,
        generated_records,
        generated_first,
    );
    render_char_shape_summary(
        &mut report,
        oracle_raw,
        oracle_records,
        oracle_first,
        generated_raw,
        generated_records,
        generated_first,
    );
    render_top_level_ctrl_sequence(
        &mut report,
        oracle_raw,
        oracle_records,
        oracle_first,
        generated_raw,
        generated_records,
        generated_first,
    );

    let _ = writeln!(report, "## 6. 1차 해석");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "첫 문단의 상위 CTRL_HEADER 순서는 record stream 기준으로 oracle/generated 모두 동일하다."
    );
    let _ = writeln!(
        report,
        "따라서 한컴 에디터 조판부호 표시에서 관찰되는 순서 차이는 단순한 `para.controls` 순서 문제가 아니라, 첫 문단 `PARA_TEXT` control char stream, `PARA_CHAR_SHAPE` offset, 또는 머리말/list 하위 record materialization 차이로 분리해 판정해야 한다."
    );
    let _ = writeln!(
        report,
        "특히 generated는 첫 문단 `PARA_CHAR_SHAPE`에 oracle에는 없는 중간 run이 들어가므로, `02_oracle_first_para_char_shape.hwp`가 우선 판정 대상이다."
    );
    report
}

fn render_record_table(
    report: &mut String,
    label: &str,
    raw: &[u8],
    records: &[RecordMeta],
    first: &FirstParaView,
) {
    let byte_range = records_byte_range(records, first.range.clone());
    let _ = writeln!(report, "### {label}");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "- record range: `{}..{}`",
        first.range.start, first.range.end
    );
    let _ = writeln!(
        report,
        "- byte range: `{}..{}`",
        byte_range.start, byte_range.end
    );
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| local | record | level | tag | ctrl | size | hash | head |"
    );
    let _ = writeln!(report, "|---:|---:|---:|---|---|---:|---|---|");
    for (local, index) in first.range.clone().enumerate() {
        let record = &records[index];
        let payload = record_payload(raw, record);
        let ctrl = if record.tag == tags::HWPTAG_CTRL_HEADER {
            read_u32(payload, 0)
                .map(format_ctrl_id)
                .unwrap_or_else(|| "-".to_string())
        } else {
            "-".to_string()
        };
        let _ = writeln!(
            report,
            "| {} | {} | {} | `{}` | {} | {} | `{}` | `{}` |",
            local,
            index,
            record.level,
            tags::tag_name(record.tag),
            ctrl,
            record.size,
            short_hash(payload),
            hex_head(payload, 24)
        );
    }
    let _ = writeln!(report);
}

fn render_para_header_summary(
    report: &mut String,
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
) {
    let _ = writeln!(report, "### PARA_HEADER");
    let _ = writeln!(report);
    render_tag_compare(
        report,
        "PARA_HEADER",
        tags::HWPTAG_PARA_HEADER,
        oracle_raw,
        oracle_records,
        oracle_first,
        generated_raw,
        generated_records,
        generated_first,
    );
}

fn render_para_text_markers(
    report: &mut String,
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
) {
    let _ = writeln!(report, "### PARA_TEXT control marker 후보");
    let _ = writeln!(report);
    let oracle_payload = first_payload(
        oracle_raw,
        oracle_records,
        oracle_first,
        tags::HWPTAG_PARA_TEXT,
    );
    let generated_payload = first_payload(
        generated_raw,
        generated_records,
        generated_first,
        tags::HWPTAG_PARA_TEXT,
    );
    let _ = writeln!(report, "#### oracle");
    render_text_markers(report, oracle_payload);
    let _ = writeln!(report, "#### generated");
    render_text_markers(report, generated_payload);
}

fn render_char_shape_summary(
    report: &mut String,
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
) {
    let _ = writeln!(report, "### PARA_CHAR_SHAPE entries");
    let _ = writeln!(report);
    let oracle_payload = first_payload(
        oracle_raw,
        oracle_records,
        oracle_first,
        tags::HWPTAG_PARA_CHAR_SHAPE,
    );
    let generated_payload = first_payload(
        generated_raw,
        generated_records,
        generated_first,
        tags::HWPTAG_PARA_CHAR_SHAPE,
    );
    let _ = writeln!(
        report,
        "- oracle: `{}`",
        format_char_shape_entries(oracle_payload)
    );
    let _ = writeln!(
        report,
        "- generated: `{}`",
        format_char_shape_entries(generated_payload)
    );
    let _ = writeln!(report);
}

fn render_top_level_ctrl_sequence(
    report: &mut String,
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
) {
    let _ = writeln!(report, "### Top-level CTRL_HEADER sequence");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "- oracle: `{}`",
        format_ctrl_sequence(oracle_raw, oracle_records, oracle_first)
    );
    let _ = writeln!(
        report,
        "- generated: `{}`",
        format_ctrl_sequence(generated_raw, generated_records, generated_first)
    );
    let _ = writeln!(report);
}

fn render_tag_compare(
    report: &mut String,
    label: &str,
    tag: u16,
    oracle_raw: &[u8],
    oracle_records: &[RecordMeta],
    oracle_first: &FirstParaView,
    generated_raw: &[u8],
    generated_records: &[RecordMeta],
    generated_first: &FirstParaView,
) {
    let oracle_payload = first_payload(oracle_raw, oracle_records, oracle_first, tag);
    let generated_payload = first_payload(generated_raw, generated_records, generated_first, tag);
    let _ = writeln!(report, "| source | {} size | hash | head |", label);
    let _ = writeln!(report, "|---|---:|---|---|");
    let _ = writeln!(
        report,
        "| oracle | {} | `{}` | `{}` |",
        payload_len(oracle_payload),
        payload_hash(oracle_payload),
        payload_head(oracle_payload, 32)
    );
    let _ = writeln!(
        report,
        "| generated | {} | `{}` | `{}` |",
        payload_len(generated_payload),
        payload_hash(generated_payload),
        payload_head(generated_payload, 32)
    );
    let _ = writeln!(report);
}

fn payload_len(payload: Option<&[u8]>) -> String {
    payload
        .map(|payload| payload.len().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn payload_hash(payload: Option<&[u8]>) -> String {
    payload.map(short_hash).unwrap_or_else(|| "-".to_string())
}

fn payload_head(payload: Option<&[u8]>, limit: usize) -> String {
    payload
        .map(|payload| hex_head(payload, limit))
        .unwrap_or_else(|| "-".to_string())
}

fn render_text_markers(report: &mut String, payload: Option<&[u8]>) {
    let Some(payload) = payload else {
        let _ = writeln!(report, "- PARA_TEXT 없음");
        let _ = writeln!(report);
        return;
    };
    let markers = para_text_markers(payload);
    if markers.is_empty() {
        let _ = writeln!(report, "- special marker 없음");
        let _ = writeln!(report);
        return;
    }
    let _ = writeln!(report);
    let _ = writeln!(report, "| byte | code | meaning | possible ctrl | head |");
    let _ = writeln!(report, "|---:|---:|---|---|---|");
    for marker in markers {
        let _ = writeln!(
            report,
            "| {} | `0x{:04x}` | {} | {} | `{}` |",
            marker.offset,
            marker.code,
            marker.meaning,
            marker
                .ctrl
                .map(format_ctrl_id)
                .unwrap_or_else(|| "-".to_string()),
            hex_slice(payload, marker.offset, 16)
        );
    }
    let _ = writeln!(report);
}

#[derive(Debug)]
struct TextMarker {
    offset: usize,
    code: u16,
    meaning: &'static str,
    ctrl: Option<u32>,
}

fn para_text_markers(payload: &[u8]) -> Vec<TextMarker> {
    let mut markers = Vec::new();
    let mut offset = 0usize;
    while offset + 2 <= payload.len() {
        let code = u16::from_le_bytes([payload[offset], payload[offset + 1]]);
        let meaning = match code {
            0x0002 => "section/column control char",
            0x0003 => "field begin",
            0x0004 => "field end",
            0x0008 => "inline non-text control",
            0x0009 => "tab",
            0x000b => "extended control char",
            0x0010 => "header/footer control char 후보",
            0x0015 => "page control char 후보",
            _ => "",
        };
        if !meaning.is_empty() {
            let ctrl = read_u32(payload, offset + 2);
            markers.push(TextMarker {
                offset,
                code,
                meaning,
                ctrl,
            });
        }
        offset += 2;
    }
    markers
}

fn first_payload<'a>(
    raw: &'a [u8],
    records: &'a [RecordMeta],
    first: &FirstParaView,
    tag: u16,
) -> Option<&'a [u8]> {
    first
        .range
        .clone()
        .find(|&index| records[index].tag == tag)
        .map(|index| record_payload(raw, &records[index]))
}

fn format_char_shape_entries(payload: Option<&[u8]>) -> String {
    let Some(payload) = payload else {
        return "없음".to_string();
    };
    let entries: Vec<String> = payload
        .chunks_exact(8)
        .map(|chunk| {
            let pos = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let shape = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
            format!("pos={pos}:shape={shape}")
        })
        .collect();
    if entries.is_empty() {
        "empty".to_string()
    } else {
        entries.join(", ")
    }
}

fn format_ctrl_sequence(raw: &[u8], records: &[RecordMeta], first: &FirstParaView) -> String {
    let entries: Vec<String> = first
        .range
        .clone()
        .filter_map(|index| {
            let record = &records[index];
            if record.tag != tags::HWPTAG_CTRL_HEADER || record.level != first.base_level + 1 {
                return None;
            }
            read_u32(record_payload(raw, record), 0)
                .map(|ctrl_id| format!("{}#{}@lv{}", format_ctrl_id(ctrl_id), index, record.level))
        })
        .collect();
    entries.join(" -> ")
}

fn read_required_section(
    bytes: &[u8],
    section: u32,
    compressed: bool,
    label: &str,
) -> Result<Vec<u8>, String> {
    read_decompressed_section(bytes, section, compressed)?
        .ok_or_else(|| format!("{label} BodyText/Section{section} 스트림을 읽지 못했습니다"))
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

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn format_ctrl_id(ctrl_id: u32) -> String {
    let bytes = ctrl_id.to_be_bytes();
    let ascii = bytes
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            }
        })
        .collect::<String>();
    format!("{}({ascii}/0x{ctrl_id:08x})", tags::ctrl_name(ctrl_id))
}

fn hex_head(data: &[u8], limit: usize) -> String {
    hex_slice(data, 0, limit)
}

fn hex_slice(data: &[u8], offset: usize, limit: usize) -> String {
    let end = data.len().min(offset.saturating_add(limit));
    data.get(offset..end)
        .unwrap_or(&[])
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn short_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().chars().take(16).collect()
}
