//! HWP5 CHAR_SHAPE provenance audit.
//!
//! Hancom HWP와 rhwp 생성 HWP를 비교해 CHAR_SHAPE의 sentinel 차이를 분류하고,
//! 생성 파일에서 그 style이 실제로 참조되는 문단을 추적한다. 이 명령은 진단 전용이다.
//! oracle record나 순번을 production serializer에 전달하거나 파일을 변경하지 않는다.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Read as _;
use std::path::PathBuf;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::diagnostics::{
    read_hwp5_body_text_section_limited, read_hwp5_doc_info_limited,
    MAX_HWP5_DIAGNOSTIC_STREAM_OUTPUT_BYTES,
};
use crate::parser::cfb_reader::CfbReader;
use crate::parser::header;
use crate::parser::record::Record;
use crate::parser::tags;

const CHAR_SHAPE_ATTR_OFFSET: usize = 46;
const CHAR_SHAPE_SHADOW_COLOR_OFFSET: usize = 64;
const CHAR_SHAPE_MIN_SIZE: usize = 74;
const DEFAULT_INACTIVE_SHADOW_COLOR: u32 = 0x00c0_c0c0;

#[derive(Debug)]
struct Options {
    oracle: PathBuf,
    generated: PathBuf,
    out: PathBuf,
    source_hwpx: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct CharShapeRecord {
    id: usize,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SentinelValues {
    attr: u32,
    shadow_color: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawMatch {
    Exact,
    UniqueDifferent,
    Ambiguous,
    Unmatched,
    Invalid,
}

impl RawMatch {
    fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::UniqueDifferent => "unique_different",
            Self::Ambiguous => "ambiguous",
            Self::Unmatched => "unmatched",
            Self::Invalid => "invalid_payload",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogicalMatch {
    Equivalent,
    Unmatched,
    Invalid,
}

impl LogicalMatch {
    fn as_str(self) -> &'static str {
        match self {
            Self::Equivalent => "equivalent",
            Self::Unmatched => "unmatched",
            Self::Invalid => "invalid_payload",
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ShapeUsage {
    runs: usize,
    paragraphs: BTreeSet<(u32, usize)>,
    stored_pages: BTreeSet<usize>,
    samples: Vec<String>,
}

#[derive(Debug, Default)]
struct ParagraphUsage {
    text: String,
    shape_runs: Vec<usize>,
    stored_pages: BTreeSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HwpxDecorationSignature {
    underline: Option<String>,
    strikeout: Option<String>,
    shadow: Option<String>,
}

impl HwpxDecorationSignature {
    fn render(&self) -> String {
        format!(
            "u:{}; s:{}; sh:{}",
            self.underline.as_deref().unwrap_or("<absent>"),
            self.strikeout.as_deref().unwrap_or("<absent>"),
            self.shadow.as_deref().unwrap_or("<absent>"),
        )
    }
}

#[derive(Debug)]
struct AuditRow {
    generated_id: usize,
    values: Option<SentinelValues>,
    raw_match: RawMatch,
    logical_match: LogicalMatch,
    logical_oracle_ids: usize,
    usage: ShapeUsage,
    source_signature: Option<HwpxDecorationSignature>,
}

pub fn run(args: &[String]) -> i32 {
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

    match run_inner(&options) {
        Ok(report) => {
            if let Some(parent) = options
                .out
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                if let Err(error) = fs::create_dir_all(parent) {
                    eprintln!("오류: 출력 폴더 생성 실패 - {}: {error}", parent.display());
                    return super::EXIT_RUNTIME;
                }
            }
            if let Err(error) = fs::write(&options.out, report) {
                eprintln!(
                    "오류: 보고서 쓰기 실패 - {}: {error}",
                    options.out.display()
                );
                return super::EXIT_RUNTIME;
            }
            println!("written: {}", options.out.display());
            super::EXIT_OK
        }
        Err(error) => {
            eprintln!("오류: {error}");
            super::EXIT_RUNTIME
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut inputs = Vec::new();
    let mut out = None;
    let mut source_hwpx = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--out" | "-o" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--out 뒤에 경로가 필요합니다".to_string())?;
                out = Some(PathBuf::from(value));
                index += 2;
            }
            "--source-hwpx" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--source-hwpx 뒤에 경로가 필요합니다".to_string())?;
                source_hwpx = Some(PathBuf::from(value));
                index += 2;
            }
            value if value.starts_with('-') => return Err(format!("알 수 없는 옵션: {value}")),
            value => {
                inputs.push(PathBuf::from(value));
                index += 1;
            }
        }
    }

    if inputs.len() != 2 {
        return Err("Hancom oracle HWP와 generated HWP 경로가 필요합니다".to_string());
    }

    Ok(Options {
        oracle: inputs[0].clone(),
        generated: inputs[1].clone(),
        out: out.ok_or_else(|| "--out 경로가 필요합니다".to_string())?,
        source_hwpx,
    })
}

fn print_usage() {
    eprintln!("사용법:");
    eprintln!("  rhwp hwp5-char-shape-audit <hancom-oracle.hwp> <generated.hwp> --out <보고서.md> [--source-hwpx <원본.hwpx>]");
}

fn run_inner(options: &Options) -> Result<String, String> {
    let oracle = read_char_shapes(&options.oracle)?;
    let (generated, usages, stored_page_count) = read_generated_audit(&options.generated)?;
    let source_signatures = options
        .source_hwpx
        .as_ref()
        .map(|path| read_hwpx_signatures(path))
        .transpose()?;
    Ok(render_report(
        options,
        &oracle,
        &generated,
        &usages,
        stored_page_count,
        source_signatures.as_ref(),
    ))
}

fn read_hwpx_signatures(
    path: &PathBuf,
) -> Result<BTreeMap<usize, HwpxDecorationSignature>, String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("HWPX 파일 열기 실패 - {}: {error}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| format!("HWPX ZIP 열기 실패 - {}: {error}", path.display()))?;
    let mut header = archive
        .by_name("Contents/header.xml")
        .map_err(|error| format!("HWPX Contents/header.xml 읽기 실패: {error}"))?;
    let mut xml = String::new();
    header
        .read_to_string(&mut xml)
        .map_err(|error| format!("HWPX header.xml UTF-8 읽기 실패: {error}"))?;
    parse_hwpx_signatures(&xml)
}

fn parse_hwpx_signatures(xml: &str) -> Result<BTreeMap<usize, HwpxDecorationSignature>, String> {
    let mut reader = Reader::from_str(xml);
    let mut buffer = Vec::new();
    let mut current: Option<(usize, HwpxDecorationSignature)> = None;
    let mut signatures = BTreeMap::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) => {
                let name = event.name();
                let local = xml_local_name(name.as_ref());
                if local == b"charPr" {
                    current = Some((
                        xml_required_id(&event)?,
                        HwpxDecorationSignature {
                            underline: None,
                            strikeout: None,
                            shadow: None,
                        },
                    ));
                } else if let Some((_, signature)) = current.as_mut() {
                    set_decoration_signature(signature, local, &event);
                }
            }
            Ok(Event::Empty(event)) => {
                if let Some((_, signature)) = current.as_mut() {
                    let name = event.name();
                    set_decoration_signature(signature, xml_local_name(name.as_ref()), &event);
                }
            }
            Ok(Event::End(event)) => {
                let name = event.name();
                if xml_local_name(name.as_ref()) == b"charPr" {
                    if let Some((id, signature)) = current.take() {
                        if signatures.insert(id, signature).is_some() {
                            return Err(format!("HWPX header에 중복 charPr id가 있습니다: {id}"));
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(format!("HWPX header.xml 파싱 실패: {error}")),
            _ => {}
        }
        buffer.clear();
    }

    if current.is_some() {
        return Err("HWPX header.xml의 charPr 닫는 태그가 없습니다".to_string());
    }
    Ok(signatures)
}

fn xml_required_id(event: &quick_xml::events::BytesStart<'_>) -> Result<usize, String> {
    for attribute in event.attributes().flatten() {
        if attribute.key.as_ref() == b"id" {
            let value = String::from_utf8_lossy(attribute.value.as_ref());
            return value
                .parse::<usize>()
                .map_err(|_| format!("HWPX charPr id가 올바르지 않습니다: {value}"));
        }
    }
    Err("HWPX charPr에 id 속성이 없습니다".to_string())
}

fn set_decoration_signature(
    signature: &mut HwpxDecorationSignature,
    local: &[u8],
    event: &quick_xml::events::BytesStart<'_>,
) {
    let slot = match local {
        b"underline" => &mut signature.underline,
        b"strikeout" => &mut signature.strikeout,
        b"shadow" => &mut signature.shadow,
        _ => return,
    };
    *slot = Some(xml_attribute_signature(event));
}

fn xml_attribute_signature(event: &quick_xml::events::BytesStart<'_>) -> String {
    let mut attributes = event
        .attributes()
        .flatten()
        .map(|attribute| {
            format!(
                "{}={}",
                String::from_utf8_lossy(attribute.key.as_ref()),
                String::from_utf8_lossy(attribute.value.as_ref())
            )
        })
        .collect::<Vec<_>>();
    attributes.sort();
    attributes.join(",")
}

fn xml_local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn read_char_shapes(path: &PathBuf) -> Result<Vec<CharShapeRecord>, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("파일 읽기 실패 - {}: {error}", path.display()))?;
    let mut cfb = CfbReader::open(&bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    let header_data = cfb
        .read_file_header()
        .map_err(|error| format!("FileHeader 읽기 실패: {error}"))?;
    let file_header = header::parse_file_header(&header_data)
        .map_err(|error| format!("FileHeader 파싱 실패: {error}"))?;
    if file_header.flags.encrypted || file_header.flags.distribution {
        return Err(format!(
            "암호화 또는 배포용 HWP는 audit 입력으로 지원하지 않습니다: {}",
            path.display()
        ));
    }
    let doc_info = read_hwp5_doc_info_limited(
        &mut cfb,
        file_header.flags.compressed,
        MAX_HWP5_DIAGNOSTIC_STREAM_OUTPUT_BYTES,
    )
    .map_err(|error| format!("DocInfo 읽기 실패: {error}"))?;
    extract_char_shapes(&doc_info)
}

fn read_generated_audit(
    path: &PathBuf,
) -> Result<(Vec<CharShapeRecord>, BTreeMap<usize, ShapeUsage>, usize), String> {
    let bytes =
        fs::read(path).map_err(|error| format!("파일 읽기 실패 - {}: {error}", path.display()))?;
    let mut cfb = CfbReader::open(&bytes).map_err(|error| format!("CFB 열기 실패: {error}"))?;
    let header_data = cfb
        .read_file_header()
        .map_err(|error| format!("FileHeader 읽기 실패: {error}"))?;
    let file_header = header::parse_file_header(&header_data)
        .map_err(|error| format!("FileHeader 파싱 실패: {error}"))?;
    if file_header.flags.encrypted || file_header.flags.distribution {
        return Err(format!(
            "암호화 또는 배포용 HWP는 audit 입력으로 지원하지 않습니다: {}",
            path.display()
        ));
    }

    let doc_info = read_hwp5_doc_info_limited(
        &mut cfb,
        file_header.flags.compressed,
        MAX_HWP5_DIAGNOSTIC_STREAM_OUTPUT_BYTES,
    )
    .map_err(|error| format!("DocInfo 읽기 실패: {error}"))?;
    let shapes = extract_char_shapes(&doc_info)?;
    let mut usages = BTreeMap::new();
    let mut stored_page_count = 0;

    for section in 0..cfb.section_count() {
        let data = read_hwp5_body_text_section_limited(
            &mut cfb,
            section,
            file_header.flags.compressed,
            file_header.flags.distribution,
            MAX_HWP5_DIAGNOSTIC_STREAM_OUTPUT_BYTES,
        )
        .map_err(|error| format!("BodyText Section{section} 읽기 실패: {error}"))?;
        let records = Record::read_all(&data)
            .map_err(|error| format!("BodyText Section{section} record 파싱 실패: {error}"))?;
        let paragraphs = collect_section_paragraphs(section, &records, &mut stored_page_count);
        append_shape_usages(&mut usages, section, paragraphs);
    }

    Ok((shapes, usages, stored_page_count))
}

fn extract_char_shapes(doc_info: &[u8]) -> Result<Vec<CharShapeRecord>, String> {
    let records =
        Record::read_all(doc_info).map_err(|error| format!("DocInfo record 파싱 실패: {error}"))?;
    Ok(records
        .into_iter()
        .filter(|record| record.tag_id == tags::HWPTAG_CHAR_SHAPE)
        .enumerate()
        .map(|(id, record)| CharShapeRecord {
            id,
            payload: record.data,
        })
        .collect())
}

fn collect_section_paragraphs(
    _section: u32,
    records: &[Record],
    stored_page_count: &mut usize,
) -> Vec<ParagraphUsage> {
    let mut paragraphs = Vec::new();
    let mut open_headers: Vec<(u16, usize)> = Vec::new();

    for record in records {
        open_headers.retain(|(level, _)| *level < record.level);

        if record.tag_id == tags::HWPTAG_PARA_HEADER {
            let paragraph_id = paragraphs.len();
            paragraphs.push(ParagraphUsage::default());
            open_headers.push((record.level, paragraph_id));
            continue;
        }

        let Some((_, paragraph_id)) = open_headers.last().copied() else {
            continue;
        };
        let paragraph = &mut paragraphs[paragraph_id];
        match record.tag_id {
            tags::HWPTAG_PARA_TEXT => paragraph.text.push_str(&decode_para_text(&record.data)),
            tags::HWPTAG_PARA_CHAR_SHAPE => {
                paragraph
                    .shape_runs
                    .extend(decode_char_shape_ids(&record.data));
            }
            tags::HWPTAG_PARA_LINE_SEG => {
                for tag in decode_line_seg_tags(&record.data) {
                    if tag & 1 != 0 {
                        *stored_page_count += 1;
                    }
                    if *stored_page_count > 0 {
                        paragraph.stored_pages.insert(*stored_page_count);
                    }
                }
            }
            _ => {}
        }
    }

    paragraphs
}

fn append_shape_usages(
    usages: &mut BTreeMap<usize, ShapeUsage>,
    section: u32,
    paragraphs: Vec<ParagraphUsage>,
) {
    for (paragraph_index, paragraph) in paragraphs.into_iter().enumerate() {
        let sample = text_sample(&paragraph.text);
        for shape_id in paragraph.shape_runs {
            let usage = usages.entry(shape_id).or_default();
            usage.runs += 1;
            usage.paragraphs.insert((section, paragraph_index));
            usage.stored_pages.extend(&paragraph.stored_pages);
            if !sample.is_empty() && usage.samples.len() < 2 && !usage.samples.contains(&sample) {
                usage.samples.push(sample.clone());
            }
        }
    }
}

fn decode_char_shape_ids(data: &[u8]) -> Vec<usize> {
    data.chunks_exact(8)
        .map(|entry| u32::from_le_bytes(entry[4..8].try_into().expect("8-byte entry")) as usize)
        .collect()
}

fn decode_line_seg_tags(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(36)
        .map(|entry| u32::from_le_bytes(entry[32..36].try_into().expect("36-byte entry")))
        .collect()
}

fn decode_para_text(data: &[u8]) -> String {
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let mut text = String::new();
    let mut index = 0;
    while index < units.len() {
        match units[index] {
            0x000d => break,
            0x0003 | 0x0004 | 0x0009 | 0x000b | 0x000f..=0x0012 | 0x0015..=0x0017 => {
                index += 8;
            }
            value if value < 0x0020 => index += 1,
            high if (0xd800..=0xdbff).contains(&high) && index + 1 < units.len() => {
                text.push_str(&String::from_utf16_lossy(&units[index..=index + 1]));
                index += 2;
            }
            _ => {
                text.push_str(&String::from_utf16_lossy(&units[index..=index]));
                index += 1;
            }
        }
    }
    text
}

fn text_sample(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::new();
    for character in compact.chars().take(72) {
        result.push(character);
    }
    result
}

fn raw_semantic_key(payload: &[u8]) -> Option<Vec<u8>> {
    if payload.len() < CHAR_SHAPE_MIN_SIZE {
        return None;
    }
    let mut key = Vec::with_capacity(payload.len() - 8);
    key.extend_from_slice(&payload[..CHAR_SHAPE_ATTR_OFFSET]);
    key.extend_from_slice(&payload[CHAR_SHAPE_ATTR_OFFSET + 4..CHAR_SHAPE_SHADOW_COLOR_OFFSET]);
    key.extend_from_slice(&payload[CHAR_SHAPE_SHADOW_COLOR_OFFSET + 4..]);
    Some(key)
}

fn sentinel_values(payload: &[u8]) -> Option<SentinelValues> {
    if payload.len() < CHAR_SHAPE_MIN_SIZE {
        return None;
    }
    Some(SentinelValues {
        attr: u32::from_le_bytes(payload[46..50].try_into().expect("validated attr slice")),
        shadow_color: u32::from_le_bytes(
            payload[64..68].try_into().expect("validated shadow slice"),
        ),
    })
}

fn logical_key(payload: &[u8]) -> Option<Vec<u8>> {
    let values = sentinel_values(payload)?;
    let mut key = payload.to_vec();
    let mut attr = values.attr;
    let underline_type = (attr >> 2) & 0x03;
    if !matches!(underline_type, 1 | 3) {
        attr &= !((0x03 << 2) | (0x0f << 4));
    }
    let strike_style = (attr >> 18) & 0x07;
    let strike_shape = (attr >> 26) & 0x0f;
    if strike_style == 0 || strike_shape > 12 {
        attr &= !((0x07 << 18) | (0x0f << 26));
    }
    let shadow_type = (attr >> 11) & 0x03;
    key[46..50].copy_from_slice(&attr.to_le_bytes());
    if shadow_type == 0 {
        key[64..68].copy_from_slice(&0u32.to_le_bytes());
    }
    Some(key)
}

fn render_report(
    options: &Options,
    oracle: &[CharShapeRecord],
    generated: &[CharShapeRecord],
    usages: &BTreeMap<usize, ShapeUsage>,
    stored_page_count: usize,
    source_signatures: Option<&BTreeMap<usize, HwpxDecorationSignature>>,
) -> String {
    let mut raw_oracle = BTreeMap::<Vec<u8>, BTreeSet<SentinelValues>>::new();
    let mut logical_oracle = BTreeMap::<Vec<u8>, BTreeSet<usize>>::new();
    for shape in oracle {
        if let (Some(raw), Some(values)) = (
            raw_semantic_key(&shape.payload),
            sentinel_values(&shape.payload),
        ) {
            raw_oracle.entry(raw).or_default().insert(values);
        }
        if let Some(logical) = logical_key(&shape.payload) {
            logical_oracle.entry(logical).or_default().insert(shape.id);
        }
    }

    let mut rows = Vec::new();
    let mut raw_counts = BTreeMap::<&str, usize>::new();
    let mut logical_counts = BTreeMap::<&str, usize>::new();
    for shape in generated {
        let values = sentinel_values(&shape.payload);
        let raw_match = match (raw_semantic_key(&shape.payload), values) {
            (Some(key), Some(actual)) => match raw_oracle.get(&key) {
                Some(expected) if expected.len() == 1 && expected.contains(&actual) => {
                    RawMatch::Exact
                }
                Some(expected) if expected.len() == 1 => RawMatch::UniqueDifferent,
                Some(_) => RawMatch::Ambiguous,
                None => RawMatch::Unmatched,
            },
            _ => RawMatch::Invalid,
        };
        let (logical_match, logical_oracle_ids) = match logical_key(&shape.payload) {
            Some(key) => match logical_oracle.get(&key) {
                Some(ids) => (LogicalMatch::Equivalent, ids.len()),
                None => (LogicalMatch::Unmatched, 0),
            },
            None => (LogicalMatch::Invalid, 0),
        };
        *raw_counts.entry(raw_match.as_str()).or_default() += 1;
        *logical_counts.entry(logical_match.as_str()).or_default() += 1;
        rows.push(AuditRow {
            generated_id: shape.id,
            values,
            raw_match,
            logical_match,
            logical_oracle_ids,
            usage: usages.get(&shape.id).cloned().unwrap_or_default(),
            source_signature: source_signatures
                .and_then(|signatures| signatures.get(&shape.id).cloned()),
        });
    }

    let mut output = String::new();
    let _ = writeln!(output, "# HWP5 CHAR_SHAPE provenance audit");
    let _ = writeln!(output);
    let _ = writeln!(output, "- Hancom 비교 입력: `{}`", options.oracle.display());
    let _ = writeln!(
        output,
        "- rhwp 생성 입력: `{}`",
        options.generated.display()
    );
    let _ = writeln!(output, "- Hancom CHAR_SHAPE: `{}`", oracle.len());
    let _ = writeln!(output, "- rhwp CHAR_SHAPE: `{}`", generated.len());
    if let Some(path) = &options.source_hwpx {
        let source_count = source_signatures.map_or(0, BTreeMap::len);
        let _ = writeln!(
            output,
            "- source HWPX: `{}` (`{source_count}` charPr signatures)",
            path.display()
        );
    }
    let _ = writeln!(
        output,
        "- 저장된 PARA_LINE_SEG 첫 줄 표식 기준 누적 쪽수: `{stored_page_count}`"
    );
    let _ = writeln!(output);
    let _ = writeln!(output, "## 판정 범위");
    let _ = writeln!(output);
    let _ = writeln!(output, "이 보고서는 한컴 HWP를 **진단 비교 대상**으로만 사용한다. `unique_different`는 Stage 4와 같은 fail-closed probe 후보를 뜻할 뿐이며, runtime serializer가 Hancom record ID 또는 값을 요구한다는 뜻이 아니다.");
    let _ = writeln!(output, "`logical_equivalent`는 inactive underline/strike/shadow sentinel을 제거한 뒤 payload가 한컴 style 하나 이상과 논리적으로 같다는 뜻이다. 이는 source-derived production 변경의 충분 조건이 아니다.");
    let _ = writeln!(output);
    write_count_table(&mut output, "raw semantic key", &raw_counts);
    write_count_table(&mut output, "logical normalized payload", &logical_counts);
    if source_signatures.is_some() {
        write_source_signature_summary(&mut output, &rows);
    }
    let _ = writeln!(output, "## Style별 사용 위치");
    let _ = writeln!(output);
    let _ = writeln!(output, "아래 표는 raw sentinel가 exact가 아닌 style만 표시한다. `stored pages`는 HWP5 `PARA_LINE_SEG` bit 0의 누적 추정이며 한컴 PDF 페이지 번호와 동일하다고 가정하지 않는다.");
    let _ = writeln!(output);
    let _ = writeln!(output, "| generated id | raw sentinel | logical | oracle logical ids | attr | shadow color | source decoration | runs | paragraphs | stored pages | text samples |");
    let _ = writeln!(output, "|---:|---|---|---:|---|---|---|---:|---:|---|---|");
    for row in rows.iter().filter(|row| row.raw_match != RawMatch::Exact) {
        let (attr, shadow_color) = row
            .values
            .map(|values| {
                (
                    format!("`0x{:08x}`", values.attr),
                    format!("`0x{:08x}`", values.shadow_color),
                )
            })
            .unwrap_or_else(|| ("-".to_string(), "-".to_string()));
        let pages = render_number_set(&row.usage.stored_pages);
        let samples = if row.usage.samples.is_empty() {
            "-".to_string()
        } else {
            row.usage
                .samples
                .iter()
                .map(|sample| escape_cell(sample))
                .collect::<Vec<_>>()
                .join("<br>")
        };
        let source_signature = row
            .source_signature
            .as_ref()
            .map(HwpxDecorationSignature::render)
            .map(|value| escape_cell(&value))
            .unwrap_or_else(|| "-".to_string());
        let _ = writeln!(
            output,
            "| {} | `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.generated_id,
            row.raw_match.as_str(),
            row.logical_match.as_str(),
            row.logical_oracle_ids,
            attr,
            shadow_color,
            source_signature,
            row.usage.runs,
            row.usage.paragraphs.len(),
            pages,
            samples,
        );
    }
    output
}

fn write_source_signature_summary(output: &mut String, rows: &[AuditRow]) {
    let mut grouped = BTreeMap::<(&str, String), usize>::new();
    for row in rows {
        let signature = row
            .source_signature
            .as_ref()
            .map(HwpxDecorationSignature::render)
            .unwrap_or_else(|| "<missing source charPr>".to_string());
        *grouped
            .entry((row.raw_match.as_str(), signature))
            .or_default() += 1;
    }

    let _ = writeln!(output, "## HWPX decoration signature 요약");
    let _ = writeln!(output);
    let _ = writeln!(output, "동일 source signature가 raw unique_different와 ambiguous/unmatched에 함께 나타나면, 이 signature는 production canonicalization 선택 기준으로 사용할 수 없다.");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| raw 상태 | HWPX underline/strikeout/shadow signature | CHAR_SHAPE 수 |"
    );
    let _ = writeln!(output, "|---|---|---:|");
    for ((raw_match, signature), count) in grouped {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {} |",
            raw_match,
            escape_cell(&signature),
            count
        );
    }
    let _ = writeln!(output);
}

fn write_count_table(output: &mut String, heading: &str, counts: &BTreeMap<&str, usize>) {
    let _ = writeln!(output, "## {heading} 요약");
    let _ = writeln!(output);
    let _ = writeln!(output, "| 상태 | CHAR_SHAPE 수 |");
    let _ = writeln!(output, "|---|---:|");
    for (state, count) in counts {
        let _ = writeln!(output, "| `{state}` | {count} |");
    }
    let _ = writeln!(output);
}

fn render_number_set(values: &BTreeSet<usize>) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    let mut parts = values
        .iter()
        .take(12)
        .map(usize::to_string)
        .collect::<Vec<_>>();
    if values.len() > parts.len() {
        parts.push(format!("…(+{})", values.len() - parts.len()));
    }
    parts.join(",")
}

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "\\`")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(attr: u32, shadow_color: u32, marker: u8) -> Vec<u8> {
        let mut payload = vec![0u8; CHAR_SHAPE_MIN_SIZE];
        payload[0] = marker;
        payload[46..50].copy_from_slice(&attr.to_le_bytes());
        payload[64..68].copy_from_slice(&shadow_color.to_le_bytes());
        payload
    }

    #[test]
    fn logical_key_ignores_inactive_sentinels() {
        let plain = payload(0, 0x00b2_b2b2, 1);
        let hancom_default = payload(
            (2 << 2) | (15 << 4) | (1 << 18) | (15 << 26),
            DEFAULT_INACTIVE_SHADOW_COLOR,
            1,
        );
        assert_eq!(logical_key(&plain), logical_key(&hancom_default));
        assert_ne!(raw_semantic_key(&plain), None);
        assert_ne!(sentinel_values(&plain), sentinel_values(&hancom_default));
    }

    #[test]
    fn logical_key_preserves_active_decorations() {
        let underline = payload((1 << 2) | (3 << 4), 0, 1);
        let plain = payload(0, 0, 1);
        assert_ne!(logical_key(&underline), logical_key(&plain));

        let shadow = payload(1 << 11, 0x0012_3456, 1);
        assert_ne!(logical_key(&shadow), logical_key(&plain));
    }

    #[test]
    fn para_char_shape_ids_use_second_u32() {
        let mut data = Vec::new();
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&7u32.to_le_bytes());
        data.extend_from_slice(&12u32.to_le_bytes());
        data.extend_from_slice(&3u32.to_le_bytes());
        assert_eq!(decode_char_shape_ids(&data), vec![7, 3]);
    }

    #[test]
    fn line_seg_page_flag_uses_last_u32() {
        let mut data = vec![0u8; 72];
        data[32..36].copy_from_slice(&1u32.to_le_bytes());
        data[68..72].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(decode_line_seg_tags(&data), vec![1, 0]);
    }

    #[test]
    fn hwpx_signature_keeps_explicit_none_children() {
        let xml = r##"<hh:head xmlns:hh="urn:test"><hh:charPr id="3"><hh:underline type="NONE" shape="SOLID"/><hh:strikeout shape="NONE"/><hh:shadow type="NONE" color="#C0C0C0" offsetX="10" offsetY="10"/></hh:charPr></hh:head>"##;
        let signatures = parse_hwpx_signatures(xml).unwrap();
        assert_eq!(
            signatures.get(&3).unwrap().render(),
            "u:shape=SOLID,type=NONE; s:shape=NONE; sh:color=#C0C0C0,offsetX=10,offsetY=10,type=NONE"
        );
    }
}
