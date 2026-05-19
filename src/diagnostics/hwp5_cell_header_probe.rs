//! HWP5 table cell header probe generator.
//!
//! This command creates focused HWP files by grafting selected table-cell
//! LIST_HEADER/PARA_HEADER record payload tails from a Hancom-produced oracle
//! HWP into a generated HWP.

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

const TABLE_CTRL_ID: u32 = 0x7462_6c20;
const ORG_TABLE_ROWS: u16 = 10;
const ORG_TABLE_COLS: u16 = 65;
const TARGET_ORG_CELLS: &[CellKey] = &[
    CellKey::new(4, 24, 2, 4),
    CellKey::new(4, 27, 3, 4),
    CellKey::new(4, 51, 3, 4),
    CellKey::new(4, 55, 3, 4),
    CellKey::new(6, 30, 4, 2),
];

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out_dir: PathBuf,
    section: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProbeAxis {
    CellListWidthRef,
    CellListExtra,
    CellParaHeaderTail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeScope {
    GeneratedBaseline,
    TargetOrgCells,
    OrgTable,
}

#[derive(Debug)]
struct Variant {
    name: &'static str,
    scope: ProbeScope,
    axes: &'static [ProbeAxis],
}

#[derive(Debug, Default, Clone)]
struct PatchCounts {
    cell_list_width_ref: usize,
    cell_list_extra: usize,
    cell_para_header_tail: usize,
    target_org_cells: usize,
    org_table_blocks: usize,
    all_table_blocks: usize,
    sections_touched: usize,
}

impl PatchCounts {
    fn add(&mut self, other: &Self) {
        self.cell_list_width_ref += other.cell_list_width_ref;
        self.cell_list_extra += other.cell_list_extra;
        self.cell_para_header_tail += other.cell_para_header_tail;
        self.target_org_cells += other.target_org_cells;
        self.org_table_blocks += other.org_table_blocks;
        self.all_table_blocks += other.all_table_blocks;
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

#[derive(Debug, Clone)]
struct TableBlock {
    table_index: usize,
    host_para_index: Option<usize>,
    start: usize,
    end: usize,
    rows: u16,
    cols: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellKey {
    row: u16,
    col: u16,
    col_span: u16,
    row_span: u16,
}

impl CellKey {
    const fn new(row: u16, col: u16, col_span: u16, row_span: u16) -> Self {
        Self {
            row,
            col,
            col_span,
            row_span,
        }
    }
}

#[derive(Debug, Default)]
struct ScopeIndices {
    list_headers: BTreeSet<usize>,
    para_headers: BTreeSet<usize>,
    target_org_cells: usize,
    org_table_blocks: usize,
    all_table_blocks: usize,
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
        "  rhwp hwp5-cell-header-probe <oracle.hwp> <generated.hwp> --out-dir <폴더> [--section N]"
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
    let variants = cell_header_probe_variants();
    let mut results = Vec::new();

    for variant in variants {
        let (bytes, counts) = if variant.scope == ProbeScope::GeneratedBaseline {
            (generated_bytes.clone(), PatchCounts::default())
        } else {
            build_probe_file(
                &oracle_bytes,
                oracle_compressed,
                &generated_bytes,
                generated_compressed,
                variant.scope,
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
    let report_path = options.out_dir.join("stage8_generation.md");
    fs::write(&report_path, report).map_err(|error| {
        format!(
            "generation report 쓰기 실패 - {}: {error}",
            report_path.display()
        )
    })?;

    println!("generated: {}", options.out_dir.display());
    Ok(())
}

fn cell_header_probe_variants() -> &'static [Variant] {
    use ProbeAxis::*;
    use ProbeScope::*;
    &[
        Variant {
            name: "01_current",
            scope: GeneratedBaseline,
            axes: &[],
        },
        Variant {
            name: "02_target_org_cell_list_width_ref_only",
            scope: TargetOrgCells,
            axes: &[CellListWidthRef],
        },
        Variant {
            name: "03_target_org_cell_list_extra_only",
            scope: TargetOrgCells,
            axes: &[CellListExtra],
        },
        Variant {
            name: "04_target_org_cell_list_width_ref_plus_extra",
            scope: TargetOrgCells,
            axes: &[CellListWidthRef, CellListExtra],
        },
        Variant {
            name: "05_target_org_cell_para_header_tail_only",
            scope: TargetOrgCells,
            axes: &[CellParaHeaderTail],
        },
        Variant {
            name: "06_target_org_cell_list_bundle_plus_para_header_tail",
            scope: TargetOrgCells,
            axes: &[CellListWidthRef, CellListExtra, CellParaHeaderTail],
        },
        Variant {
            name: "07_all_org_cells_list_width_ref_plus_extra",
            scope: OrgTable,
            axes: &[CellListWidthRef, CellListExtra],
        },
        Variant {
            name: "08_all_org_cells_list_bundle_plus_para_header_tail",
            scope: OrgTable,
            axes: &[CellListWidthRef, CellListExtra, CellParaHeaderTail],
        },
    ]
}

fn build_probe_file(
    oracle_bytes: &[u8],
    oracle_compressed: bool,
    generated_bytes: &[u8],
    generated_compressed: bool,
    scope: ProbeScope,
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
            patch_section(&oracle_section, &generated_section, scope, axes);
        if section_counts.cell_list_width_ref
            + section_counts.cell_list_extra
            + section_counts.cell_para_header_tail
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

fn patch_section(
    oracle_raw: &[u8],
    generated_raw: &[u8],
    scope: ProbeScope,
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
    let scope_indices = scoped_indices(&generated_records, generated_raw, scope);
    let mut patches: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut counts = PatchCounts {
        target_org_cells: scope_indices.target_org_cells,
        org_table_blocks: scope_indices.org_table_blocks,
        all_table_blocks: scope_indices.all_table_blocks,
        ..PatchCounts::default()
    };

    for (oracle_index, generated_index) in pairs {
        let oracle_record = &oracle_records[oracle_index];
        let generated_record = &generated_records[generated_index];
        let oracle_payload = record_payload(oracle_raw, oracle_record);
        let generated_payload = record_payload(generated_raw, generated_record);
        let mut patched_payload = generated_payload.to_vec();
        let mut changed = false;

        if axes.contains(&ProbeAxis::CellListWidthRef)
            && generated_record.tag == tags::HWPTAG_LIST_HEADER
            && oracle_record.tag == tags::HWPTAG_LIST_HEADER
            && scope_indices.list_headers.contains(&generated_index)
            && graft_list_width_ref(&mut patched_payload, oracle_payload)
        {
            counts.cell_list_width_ref += 1;
            changed = true;
        }

        if axes.contains(&ProbeAxis::CellListExtra)
            && generated_record.tag == tags::HWPTAG_LIST_HEADER
            && oracle_record.tag == tags::HWPTAG_LIST_HEADER
            && scope_indices.list_headers.contains(&generated_index)
            && graft_tail_from_offset(&mut patched_payload, oracle_payload, 34)
        {
            counts.cell_list_extra += 1;
            changed = true;
        }

        if axes.contains(&ProbeAxis::CellParaHeaderTail)
            && generated_record.tag == tags::HWPTAG_PARA_HEADER
            && oracle_record.tag == tags::HWPTAG_PARA_HEADER
            && scope_indices.para_headers.contains(&generated_index)
            && graft_tail_from_offset(&mut patched_payload, oracle_payload, 20)
        {
            counts.cell_para_header_tail += 1;
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

fn scoped_indices(records: &[RecordMeta], raw: &[u8], scope: ProbeScope) -> ScopeIndices {
    let table_blocks = table_blocks(records, raw);
    let selected_blocks: Vec<&TableBlock> = match scope {
        ProbeScope::GeneratedBaseline => Vec::new(),
        ProbeScope::TargetOrgCells => table_blocks
            .iter()
            .filter(|block| block.rows == ORG_TABLE_ROWS && block.cols == ORG_TABLE_COLS)
            .collect(),
        ProbeScope::OrgTable => table_blocks
            .iter()
            .filter(|block| block.rows == ORG_TABLE_ROWS && block.cols == ORG_TABLE_COLS)
            .collect(),
    };

    let mut indices = ScopeIndices {
        org_table_blocks: table_blocks
            .iter()
            .filter(|block| block.rows == ORG_TABLE_ROWS && block.cols == ORG_TABLE_COLS)
            .count(),
        all_table_blocks: table_blocks.len(),
        ..ScopeIndices::default()
    };

    for block in selected_blocks {
        match scope {
            ProbeScope::GeneratedBaseline => {}
            ProbeScope::TargetOrgCells => {
                for index in block.start..block.end {
                    if records[index].tag != tags::HWPTAG_LIST_HEADER {
                        continue;
                    }
                    let payload = record_payload(raw, &records[index]);
                    if !is_target_org_cell(payload) {
                        continue;
                    }
                    indices.list_headers.insert(index);
                    indices.target_org_cells += 1;
                    collect_cell_para_headers(records, block, index, &mut indices.para_headers);
                }
            }
            ProbeScope::OrgTable => {
                for index in block.start..block.end {
                    match records[index].tag {
                        tags::HWPTAG_LIST_HEADER => {
                            indices.list_headers.insert(index);
                        }
                        tags::HWPTAG_PARA_HEADER => {
                            indices.para_headers.insert(index);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    indices
}

fn collect_cell_para_headers(
    records: &[RecordMeta],
    block: &TableBlock,
    list_header_index: usize,
    out: &mut BTreeSet<usize>,
) {
    let cell_level = records[list_header_index].level;
    for index in list_header_index + 1..block.end {
        let record = &records[index];
        if record.level < cell_level {
            break;
        }
        if record.level == cell_level && record.tag == tags::HWPTAG_LIST_HEADER {
            break;
        }
        if record.level == cell_level && record.tag == tags::HWPTAG_PARA_HEADER {
            out.insert(index);
        }
    }
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
            host_para_index: find_host_para_index(records, index),
            start: index,
            end,
            rows,
            cols,
        });
    }
    blocks
}

fn find_host_para_index(records: &[RecordMeta], table_index: usize) -> Option<usize> {
    let table_level = records.get(table_index)?.level;
    let ctrl_level = table_level.checked_sub(1)?;
    let mut ctrl_index = None;
    for index in (0..table_index).rev() {
        let record = &records[index];
        if record.level < ctrl_level {
            break;
        }
        if record.level == ctrl_level && record.tag == tags::HWPTAG_CTRL_HEADER {
            ctrl_index = Some(index);
            break;
        }
    }
    let ctrl_index = ctrl_index?;
    let host_level = ctrl_level.checked_sub(1)?;
    for index in (0..ctrl_index).rev() {
        let record = &records[index];
        if record.level < host_level {
            break;
        }
        if record.level == host_level && record.tag == tags::HWPTAG_PARA_HEADER {
            return Some(index);
        }
    }
    None
}

fn table_rows_cols(payload: &[u8]) -> Option<(u16, u16)> {
    let rows = u16::from_le_bytes(payload.get(4..6)?.try_into().ok()?);
    let cols = u16::from_le_bytes(payload.get(6..8)?.try_into().ok()?);
    Some((rows, cols))
}

fn cell_key(payload: &[u8]) -> Option<CellKey> {
    Some(CellKey {
        col: u16::from_le_bytes(payload.get(8..10)?.try_into().ok()?),
        row: u16::from_le_bytes(payload.get(10..12)?.try_into().ok()?),
        col_span: u16::from_le_bytes(payload.get(12..14)?.try_into().ok()?),
        row_span: u16::from_le_bytes(payload.get(14..16)?.try_into().ok()?),
    })
}

fn is_target_org_cell(payload: &[u8]) -> bool {
    cell_key(payload)
        .map(|key| TARGET_ORG_CELLS.contains(&key))
        .unwrap_or(false)
}

fn graft_list_width_ref(target: &mut [u8], source: &[u8]) -> bool {
    let Some(source_width_ref) = source.get(6..8) else {
        return false;
    };
    let Some(target_width_ref) = target.get_mut(6..8) else {
        return false;
    };
    if target_width_ref == source_width_ref {
        return false;
    }
    target_width_ref.copy_from_slice(source_width_ref);
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
        "# Task M100-993 Stage 8 Cell Linebreak Contract Probe Files"
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
        "- org table dimension: `{ORG_TABLE_ROWS}x{ORG_TABLE_COLS}`"
    );
    if let Some(section) = options.section {
        let _ = writeln!(report, "- section filter: `{}`", section);
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 2. 생성 파일");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| file | bytes | hash | rhwp reload | list width_ref | list extra | para header tail | target cells | org blocks | table blocks | sections |"
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
            result.counts.cell_list_width_ref,
            result.counts.cell_list_extra,
            result.counts.cell_para_header_tail,
            result.counts.target_org_cells,
            result.counts.org_table_blocks,
            result.counts.all_table_blocks,
            result.counts.sections_touched
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "## 3. 작업지시자 판정표");
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "| variant | 한컴 판정 유형 | 조직도 셀 줄나눔 | 표 높이 | rhwp-studio 판정 | 비고 |"
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
        "이 파일들은 HWPX 저장기 구현 결과가 아니라, generated HWP의 BodyText 표 셀 LIST_HEADER/PARA_HEADER record 일부를 oracle 값으로 graft한 판정용 probe다."
    );
    report
}

fn short_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().chars().take(16).collect()
}
