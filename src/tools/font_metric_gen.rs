//! 폰트 메트릭 DB 생성 도구
//!
//! TTF 파일에서 head/cmap/hmtx/maxp/name 테이블을 파싱하여
//! 글리프 폭 데이터를 추출하고, Rust 소스코드로 출력한다.
//!
//! 사용법:
//!   cargo run --bin font-metric-gen -- ttfs/windows/malgun.ttf --face-index 0
//!   cargo run --bin font-metric-gen -- --plan plan.json \
//!     --generated-output generated.rs --metadata-output provenance.json
//!
//! 생성 모드는 입력 순서와 TTC face를 계획 파일에 명시해야 한다. core lookup과
//! measured/manual overlay는 이 도구의 소유가 아니므로 출력 대상으로 지정할 수 없다.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const GENERATOR_SOURCE_BYTES: &[u8] = include_bytes!("font_metric_gen.rs");

// ─── TTF 바이너리 파싱 헬퍼 ───

fn read_u16_be(data: &[u8], off: usize) -> u16 {
    ((data[off] as u16) << 8) | (data[off + 1] as u16)
}

fn read_u32_be(data: &[u8], off: usize) -> u32 {
    ((data[off] as u32) << 24)
        | ((data[off + 1] as u32) << 16)
        | ((data[off + 2] as u32) << 8)
        | (data[off + 3] as u32)
}

fn read_i16_be(data: &[u8], off: usize) -> i16 {
    read_u16_be(data, off) as i16
}

fn tag_str(data: &[u8], off: usize) -> String {
    String::from_utf8_lossy(&data[off..off + 4]).to_string()
}

fn sha256_bytes(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

fn sha256_file(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|data| sha256_bytes(&data))
        .map_err(|error| format!("{}: {}", path.display(), error))
}

// ─── TTF 테이블 디렉토리 ───

#[derive(Debug)]
struct TableEntry {
    tag: String,
    offset: u32,
    length: u32,
}

/// TTC 내 모든 폰트의 오프셋 배열 반환 (단일 TTF면 [0])
fn get_font_offsets(data: &[u8]) -> Vec<usize> {
    if data.len() >= 12 && &data[0..4] == b"ttcf" {
        let num_fonts = read_u32_be(data, 8) as usize;
        (0..num_fonts)
            .map(|i| read_u32_be(data, 12 + i * 4) as usize)
            .collect()
    } else {
        vec![0]
    }
}

fn parse_table_directory_at(data: &[u8], header_off: usize) -> Vec<TableEntry> {
    let num_tables = read_u16_be(data, header_off + 4);

    let mut tables = Vec::new();
    for i in 0..num_tables as usize {
        let entry_off = header_off + 12 + i * 16;
        if entry_off + 16 > data.len() {
            break;
        }
        tables.push(TableEntry {
            tag: tag_str(data, entry_off),
            offset: read_u32_be(data, entry_off + 8),
            length: read_u32_be(data, entry_off + 12),
        });
    }
    tables
}

fn table_offset(tables: &[TableEntry], tag: &str) -> Option<usize> {
    tables
        .iter()
        .find(|t| t.tag == tag)
        .map(|t| t.offset as usize)
}

// ─── head 테이블: unitsPerEm, macStyle ───

struct HeadInfo {
    units_per_em: u16,
    mac_style: u16, // bit0=Bold, bit1=Italic
}

fn parse_head(data: &[u8], tables: &[TableEntry]) -> HeadInfo {
    let off = table_offset(tables, "head").expect("head 테이블 없음");
    HeadInfo {
        units_per_em: read_u16_be(data, off + 18),
        mac_style: read_u16_be(data, off + 44),
    }
}

// ─── maxp 테이블: numGlyphs ───

fn parse_maxp(data: &[u8], tables: &[TableEntry]) -> u16 {
    let off = table_offset(tables, "maxp").expect("maxp 테이블 없음");
    read_u16_be(data, off + 4) // numGlyphs at offset 4
}

// ─── cmap 테이블: Unicode → Glyph ID ───

fn parse_cmap(data: &[u8], tables: &[TableEntry]) -> HashMap<u32, u16> {
    let cmap_off = table_offset(tables, "cmap").expect("cmap 테이블 없음");
    let num_subtables = read_u16_be(data, cmap_off + 2) as usize;

    let mut map = HashMap::new();

    // 우선순위: platformID=3(Windows) encodingID=10(UCS-4, Format 12) > encodingID=1(BMP, Format 4)
    let mut format4_off = None;
    let mut format12_off = None;

    for i in 0..num_subtables {
        let rec = cmap_off + 4 + i * 8;
        if rec + 8 > data.len() {
            break;
        }
        let platform_id = read_u16_be(data, rec);
        let encoding_id = read_u16_be(data, rec + 2);
        let subtable_off = cmap_off + read_u32_be(data, rec + 4) as usize;

        if subtable_off >= data.len() {
            continue;
        }
        let format = read_u16_be(data, subtable_off);

        if platform_id == 3 {
            if encoding_id == 1 && format == 4 {
                format4_off = Some(subtable_off);
            }
            if encoding_id == 10 && format == 12 {
                format12_off = Some(subtable_off);
            }
        }
        // platformID=0 (Unicode)도 폴백으로
        if platform_id == 0 {
            if format == 4 && format4_off.is_none() {
                format4_off = Some(subtable_off);
            }
            if format == 12 && format12_off.is_none() {
                format12_off = Some(subtable_off);
            }
        }
    }

    // Format 12 파싱 (전체 유니코드)
    if let Some(off) = format12_off {
        let n_groups = read_u32_be(data, off + 12) as usize;
        for g in 0..n_groups {
            let rec = off + 16 + g * 12;
            if rec + 12 > data.len() {
                break;
            }
            let start_char = read_u32_be(data, rec);
            let end_char = read_u32_be(data, rec + 4);
            let start_glyph = read_u32_be(data, rec + 8);
            for c in start_char..=end_char {
                let gid = start_glyph + (c - start_char);
                map.insert(c, gid as u16);
            }
        }
        return map;
    }

    // Format 4 파싱 (BMP only)
    if let Some(off) = format4_off {
        let seg_count = read_u16_be(data, off + 6) as usize / 2;
        let end_codes_off = off + 14;
        let start_codes_off = end_codes_off + seg_count * 2 + 2; // +2 for reservedPad
        let id_delta_off = start_codes_off + seg_count * 2;
        let id_range_off = id_delta_off + seg_count * 2;

        for seg in 0..seg_count {
            let end_code = read_u16_be(data, end_codes_off + seg * 2) as u32;
            let start_code = read_u16_be(data, start_codes_off + seg * 2) as u32;
            let id_delta = read_i16_be(data, id_delta_off + seg * 2) as i32;
            let id_range_offset = read_u16_be(data, id_range_off + seg * 2) as usize;

            if start_code == 0xFFFF {
                break;
            }

            for c in start_code..=end_code {
                let gid = if id_range_offset == 0 {
                    ((c as i32 + id_delta) & 0xFFFF) as u16
                } else {
                    let glyph_idx_off =
                        id_range_off + seg * 2 + id_range_offset + ((c - start_code) as usize) * 2;
                    if glyph_idx_off + 2 <= data.len() {
                        let gid = read_u16_be(data, glyph_idx_off);
                        if gid != 0 {
                            ((gid as i32 + id_delta) & 0xFFFF) as u16
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                };
                if gid != 0 {
                    map.insert(c, gid);
                }
            }
        }
    }

    map
}

// ─── hmtx 테이블: Glyph ID → advance width ───

fn parse_hmtx(data: &[u8], tables: &[TableEntry], num_glyphs: u16) -> Vec<u16> {
    let hmtx_off = table_offset(tables, "hmtx").expect("hmtx 테이블 없음");
    // hhea 테이블에서 numberOfHMetrics 읽기
    let hhea_off = table_offset(tables, "hhea").expect("hhea 테이블 없음");
    let num_h_metrics = read_u16_be(data, hhea_off + 34) as usize;

    let mut widths = Vec::with_capacity(num_glyphs as usize);

    // longHorMetric[numberOfHMetrics]: advanceWidth(u16) + lsb(i16) = 4바이트씩
    let mut last_width = 0u16;
    for i in 0..num_h_metrics.min(num_glyphs as usize) {
        let w = read_u16_be(data, hmtx_off + i * 4);
        widths.push(w);
        last_width = w;
    }

    // 나머지 글리프는 마지막 width 반복 (leftSideBearing만 다름)
    for _ in num_h_metrics..num_glyphs as usize {
        widths.push(last_width);
    }

    widths
}

// ─── name 테이블 ───

const REQUIRED_NAME_IDS: [u16; 8] = [1, 2, 3, 4, 6, 16, 17, 25];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NamingRecord {
    record_index: usize,
    platform_id: u16,
    encoding_id: u16,
    language_id: u16,
    name_id: u16,
    value: String,
}

fn decode_name_record(platform_id: u16, bytes: &[u8]) -> Option<String> {
    if platform_id == 0 || platform_id == 3 {
        if !bytes.len().is_multiple_of(2) {
            return None;
        }
        let units = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
        let value: String = char::decode_utf16(units)
            .map(|item| item.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        return Some(value);
    }
    if platform_id == 1 {
        return Some(String::from_utf8_lossy(bytes).into_owned());
    }
    None
}

fn parse_naming_records(data: &[u8], tables: &[TableEntry]) -> Vec<NamingRecord> {
    let Some(name_off) = table_offset(tables, "name") else {
        return Vec::new();
    };
    let count = read_u16_be(data, name_off + 2) as usize;
    let string_offset = name_off + read_u16_be(data, name_off + 4) as usize;
    let mut records = Vec::new();

    for i in 0..count {
        let rec = name_off + 6 + i * 12;
        if rec + 12 > data.len() {
            break;
        }
        let platform_id = read_u16_be(data, rec);
        let encoding_id = read_u16_be(data, rec + 2);
        let language_id = read_u16_be(data, rec + 4);
        let name_id = read_u16_be(data, rec + 6);
        let length = read_u16_be(data, rec + 8) as usize;
        let offset = string_offset + read_u16_be(data, rec + 10) as usize;

        if !REQUIRED_NAME_IDS.contains(&name_id) || offset + length > data.len() {
            continue;
        }
        if let Some(value) = decode_name_record(platform_id, &data[offset..offset + length]) {
            records.push(NamingRecord {
                record_index: i,
                platform_id,
                encoding_id,
                language_id,
                name_id,
                value,
            });
        }
    }
    records
}

fn select_family_name(records: &[NamingRecord]) -> String {
    for name_id in [16, 1] {
        if let Some(record) = records
            .iter()
            .find(|record| record.name_id == name_id && record.platform_id == 3)
        {
            return record.value.clone();
        }
        if let Some(record) = records.iter().find(|record| record.name_id == name_id) {
            return record.value.clone();
        }
    }

    String::new()
}

// ─── 폰트 메트릭 데이터 구조 ───

#[derive(Debug)]
struct FontMetric {
    family_name: String,
    file_name: String,
    source_sha256: String,
    face_index: u32,
    naming_records: Vec<NamingRecord>,
    em_size: u16,
    bold: bool,
    italic: bool,
    /// Unicode codepoint → advance width (em 단위)
    char_widths: HashMap<u32, u16>,
}

fn parse_ttf_all(path: &Path) -> Result<Vec<FontMetric>, String> {
    let data = fs::read(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    if data.len() < 12 {
        return Err(format!("{}: 파일이 너무 작음", path.display()));
    }

    let offsets = get_font_offsets(&data);
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut results = Vec::new();

    let source_sha256 = sha256_bytes(&data);
    for (face_index, &font_off) in offsets.iter().enumerate() {
        let tables = parse_table_directory_at(&data, font_off);
        if tables.is_empty() {
            continue;
        }

        let head = parse_head(&data, &tables);
        let num_glyphs = parse_maxp(&data, &tables);
        let cmap = parse_cmap(&data, &tables);
        let hmtx = parse_hmtx(&data, &tables, num_glyphs);
        let naming_records = parse_naming_records(&data, &tables);
        let family_name = select_family_name(&naming_records);

        let mut char_widths = HashMap::new();
        for (&codepoint, &glyph_id) in &cmap {
            if (glyph_id as usize) < hmtx.len() {
                char_widths.insert(codepoint, hmtx[glyph_id as usize]);
            }
        }

        results.push(FontMetric {
            family_name,
            file_name: file_name.clone(),
            source_sha256: source_sha256.clone(),
            face_index: face_index as u32,
            naming_records,
            em_size: head.units_per_em,
            bold: (head.mac_style & 0x01) != 0,
            italic: (head.mac_style & 0x02) != 0,
            char_widths,
        });
    }

    if results.is_empty() {
        Err(format!("{}: 폰트 없음", path.display()))
    } else {
        Ok(results)
    }
}

fn parse_ttf_face(path: &Path, face_index: u32) -> Result<FontMetric, String> {
    let metrics = parse_ttf_all(path)?;
    metrics
        .into_iter()
        .find(|metric| metric.face_index == face_index)
        .ok_or_else(|| format!("{}: TTC face index {}가 없음", path.display(), face_index))
}

// ─── 한글 음절 분해 압축 ───

const HANGUL_BASE: u32 = 0xAC00;
const HANGUL_END: u32 = 0xD7A3;
const CHO_COUNT: u32 = 19;
const JUNG_COUNT: u32 = 21;
const JONG_COUNT: u32 = 28;

fn decompose_hangul(code: u32) -> (u32, u32, u32) {
    let idx = code - HANGUL_BASE;
    let cho = idx / (JUNG_COUNT * JONG_COUNT);
    let jung = (idx % (JUNG_COUNT * JONG_COUNT)) / JONG_COUNT;
    let jong = idx % JONG_COUNT;
    (cho, jung, jong)
}

/// 한글 음절 폭을 초/중/종성 그룹으로 압축한다.
/// 그룹 수가 적을수록 압축률이 높지만 오차가 증가한다.
fn compress_hangul(
    char_widths: &HashMap<u32, u16>,
    max_cho_groups: u8,
    max_jung_groups: u8,
    max_jong_groups: u8,
) -> Option<HangulCompressed> {
    // 음절별 폭 수집
    let mut syllable_widths = Vec::new();
    for code in HANGUL_BASE..=HANGUL_END {
        if let Some(&w) = char_widths.get(&code) {
            syllable_widths.push((code, w));
        }
    }
    if syllable_widths.is_empty() {
        return None;
    }

    // 모든 음절이 동일한 폭인지 확인 (type:0)
    let first_w = syllable_widths[0].1;
    if syllable_widths.iter().all(|&(_, w)| w == first_w) {
        return Some(HangulCompressed {
            cho_groups: 1,
            jung_groups: 1,
            jong_groups: 1,
            cho_map: [0; 19],
            jung_map: [0; 21],
            jong_map: [0; 28],
            widths: vec![first_w],
            max_error: 0,
            avg_error: 0.0,
        });
    }

    // 초/중/종성별 폭 패턴 분석
    // 각 자모가 참여하는 음절들의 평균 폭으로 그룹핑
    let best = find_best_grouping(
        &syllable_widths,
        max_cho_groups,
        max_jung_groups,
        max_jong_groups,
    );

    Some(best)
}

fn find_best_grouping(
    syllable_widths: &[(u32, u16)],
    max_cho: u8,
    max_jung: u8,
    max_jong: u8,
) -> HangulCompressed {
    // 음절 폭을 3D 배열로 변환
    let mut widths_3d =
        vec![vec![vec![0u16; JONG_COUNT as usize]; JUNG_COUNT as usize]; CHO_COUNT as usize];
    let mut has_data =
        vec![vec![vec![false; JONG_COUNT as usize]; JUNG_COUNT as usize]; CHO_COUNT as usize];

    for &(code, w) in syllable_widths {
        let (cho, jung, jong) = decompose_hangul(code);
        widths_3d[cho as usize][jung as usize][jong as usize] = w;
        has_data[cho as usize][jung as usize][jong as usize] = true;
    }

    // 초성별 평균 폭 계산
    let cho_avgs = compute_axis_averages(&widths_3d, &has_data, 0);
    let jung_avgs = compute_axis_averages(&widths_3d, &has_data, 1);
    let jong_avgs = compute_axis_averages(&widths_3d, &has_data, 2);

    // K-means 클러스터링으로 그룹 할당
    let cho_map = kmeans_group(&cho_avgs, max_cho as usize);
    let jung_map = kmeans_group(&jung_avgs, max_jung as usize);
    let jong_map = kmeans_group(&jong_avgs, max_jong as usize);

    let cho_groups = *cho_map.iter().max().unwrap_or(&0) + 1;
    let jung_groups = *jung_map.iter().max().unwrap_or(&0) + 1;
    let jong_groups = *jong_map.iter().max().unwrap_or(&0) + 1;

    // 그룹 조합별 대표 폭 계산 (평균)
    let total_groups = cho_groups as usize * jung_groups as usize * jong_groups as usize;
    let mut group_sums = vec![0u64; total_groups];
    let mut group_counts = vec![0u32; total_groups];

    for &(code, w) in syllable_widths {
        let (cho, jung, jong) = decompose_hangul(code);
        let gi = cho_map[cho as usize] as usize * jung_groups as usize * jong_groups as usize
            + jung_map[jung as usize] as usize * jong_groups as usize
            + jong_map[jong as usize] as usize;
        group_sums[gi] += w as u64;
        group_counts[gi] += 1;
    }

    let group_widths: Vec<u16> = group_sums
        .iter()
        .zip(group_counts.iter())
        .map(|(&sum, &cnt)| {
            if cnt > 0 {
                (sum / cnt as u64) as u16
            } else {
                0
            }
        })
        .collect();

    // 오차 측정
    let mut max_error = 0u16;
    let mut total_error = 0u64;
    let mut count = 0u64;

    for &(code, w) in syllable_widths {
        let (cho, jung, jong) = decompose_hangul(code);
        let gi = cho_map[cho as usize] as usize * jung_groups as usize * jong_groups as usize
            + jung_map[jung as usize] as usize * jong_groups as usize
            + jong_map[jong as usize] as usize;
        let approx = group_widths[gi];
        let err = (w as i32 - approx as i32).unsigned_abs() as u16;
        max_error = max_error.max(err);
        total_error += err as u64;
        count += 1;
    }

    let mut cho_map_arr = [0u8; 19];
    for (i, &g) in cho_map.iter().enumerate() {
        cho_map_arr[i] = g;
    }
    let mut jung_map_arr = [0u8; 21];
    for (i, &g) in jung_map.iter().enumerate() {
        jung_map_arr[i] = g;
    }
    let mut jong_map_arr = [0u8; 28];
    for (i, &g) in jong_map.iter().enumerate() {
        jong_map_arr[i] = g;
    }

    HangulCompressed {
        cho_groups,
        jung_groups,
        jong_groups,
        cho_map: cho_map_arr,
        jung_map: jung_map_arr,
        jong_map: jong_map_arr,
        widths: group_widths,
        max_error,
        avg_error: if count > 0 {
            total_error as f64 / count as f64
        } else {
            0.0
        },
    }
}

/// 축(0=초성, 1=중성, 2=종성)별 평균 폭 계산
fn compute_axis_averages(
    widths: &[Vec<Vec<u16>>],
    has_data: &[Vec<Vec<bool>>],
    axis: usize,
) -> Vec<f64> {
    let count = match axis {
        0 => CHO_COUNT as usize,
        1 => JUNG_COUNT as usize,
        2 => JONG_COUNT as usize,
        _ => unreachable!(),
    };

    let mut avgs = vec![0.0; count];
    for idx in 0..count {
        let mut sum = 0u64;
        let mut cnt = 0u32;
        for cho in 0..CHO_COUNT as usize {
            for jung in 0..JUNG_COUNT as usize {
                for jong in 0..JONG_COUNT as usize {
                    let matches = match axis {
                        0 => cho == idx,
                        1 => jung == idx,
                        2 => jong == idx,
                        _ => false,
                    };
                    if matches && has_data[cho][jung][jong] {
                        sum += widths[cho][jung][jong] as u64;
                        cnt += 1;
                    }
                }
            }
        }
        avgs[idx] = if cnt > 0 {
            sum as f64 / cnt as f64
        } else {
            0.0
        };
    }
    avgs
}

/// 1D K-means 클러스터링 (간단한 정렬 기반 분할)
fn kmeans_group(values: &[f64], k: usize) -> Vec<u8> {
    let n = values.len();
    if n == 0 || k == 0 {
        return vec![0; n];
    }
    if k >= n {
        return (0..n).map(|i| i as u8).collect();
    }

    // 값-인덱스 쌍을 정렬
    let mut indexed: Vec<(f64, usize)> = values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // 정렬된 순서에서 k등분
    let mut groups = vec![0u8; n];
    for (rank, &(_, orig_idx)) in indexed.iter().enumerate() {
        groups[orig_idx] = (rank * k / n).min(k - 1) as u8;
    }
    groups
}

#[derive(Debug)]
struct HangulCompressed {
    cho_groups: u8,
    jung_groups: u8,
    jong_groups: u8,
    cho_map: [u8; 19],
    jung_map: [u8; 21],
    jong_map: [u8; 28],
    widths: Vec<u16>,
    max_error: u16,
    avg_error: f64,
}

// ─── Latin 범위 폭 추출 ───

struct LatinRange {
    start: char,
    end: char,
    widths: Vec<u16>,
}

fn extract_latin_ranges(char_widths: &HashMap<u32, u16>) -> Vec<LatinRange> {
    let ranges: Vec<(u32, u32)> = vec![
        (0x0020, 0x007E), // Basic Latin (space ~ tilde)
        (0x00A0, 0x00FF), // Latin-1 Supplement
        (0x2000, 0x206F), // General Punctuation
        (0x2200, 0x22FF), // Mathematical Operators
        (0x3000, 0x303F), // CJK Symbols and Punctuation
        (0x3130, 0x318F), // Hangul Compatibility Jamo
        (0xFF00, 0xFF5E), // Fullwidth Latin
    ];

    let mut result = Vec::new();
    for (start, end) in ranges {
        let mut widths = Vec::new();
        let mut has_any = false;
        for c in start..=end {
            if let Some(&w) = char_widths.get(&c) {
                widths.push(w);
                has_any = true;
            } else {
                widths.push(0); // 미등록 글리프
            }
        }
        if has_any {
            result.push(LatinRange {
                start: char::from_u32(start).unwrap_or(' '),
                end: char::from_u32(end).unwrap_or(' '),
                widths,
            });
        }
    }
    result
}

// ─── Rust 소스코드 생성 ───

fn generate_rust_source(metrics: &[FontMetric]) -> String {
    let mut out = String::new();

    out.push_str("// 폰트 메트릭 generated data fragment (자동 생성)\n");
    out.push_str("//\n");
    out.push_str("// font-metric-gen이 명시적 plan 순서로 추출했다.\n");
    out.push_str("// core lookup과 measured/manual overlay를 포함하지 않는다.\n");
    out.push_str("// sort 또는 dedupe 금지.\n\n");

    // 각 폰트별 데이터 생성
    for (idx, m) in metrics.iter().enumerate() {
        let var_prefix = format!("FONT_{}", idx);

        // Latin 범위 데이터
        let latin_ranges = extract_latin_ranges(&m.char_widths);
        for (ri, range) in latin_ranges.iter().enumerate() {
            out.push_str(&format!(
                "static {}_LATIN_{}: [u16; {}] = {:?};\n",
                var_prefix,
                ri,
                range.widths.len(),
                range.widths
            ));
        }

        // Latin 범위 배열
        out.push_str(&format!(
            "static {}_LATIN_RANGES: [LatinRange; {}] = [\n",
            var_prefix,
            latin_ranges.len()
        ));
        for (ri, range) in latin_ranges.iter().enumerate() {
            out.push_str(&format!(
                "    LatinRange {{ start: 0x{:04X}, end: 0x{:04X}, widths: &{}_LATIN_{} }},\n",
                range.start as u32, range.end as u32, var_prefix, ri
            ));
        }
        out.push_str("];\n");

        // 한글 메트릭
        let hangul = compress_hangul(&m.char_widths, 4, 6, 3);
        if let Some(ref h) = hangul {
            out.push_str(&format!(
                "static {}_HANGUL_CHO: [u8; 19] = {:?};\n",
                var_prefix, h.cho_map
            ));
            out.push_str(&format!(
                "static {}_HANGUL_JUNG: [u8; 21] = {:?};\n",
                var_prefix, h.jung_map
            ));
            out.push_str(&format!(
                "static {}_HANGUL_JONG: [u8; 28] = {:?};\n",
                var_prefix, h.jong_map
            ));
            out.push_str(&format!(
                "static {}_HANGUL_WIDTHS: [u16; {}] = {:?};\n",
                var_prefix,
                h.widths.len(),
                h.widths
            ));
            out.push_str(&format!(
                "static {}_HANGUL: HangulMetric = HangulMetric {{\n",
                var_prefix
            ));
            out.push_str(&format!("    cho_groups: {},\n", h.cho_groups));
            out.push_str(&format!("    jung_groups: {},\n", h.jung_groups));
            out.push_str(&format!("    jong_groups: {},\n", h.jong_groups));
            out.push_str(&format!("    cho_map: &{}_HANGUL_CHO,\n", var_prefix));
            out.push_str(&format!("    jung_map: &{}_HANGUL_JUNG,\n", var_prefix));
            out.push_str(&format!("    jong_map: &{}_HANGUL_JONG,\n", var_prefix));
            out.push_str(&format!("    widths: &{}_HANGUL_WIDTHS,\n", var_prefix));
            out.push_str("};\n");
        }

        out.push('\n');
    }

    // generated 영역 배열. core facade가 overlay 뒤가 아니라 이 배열 뒤에 overlay를 잇는다.
    out.push_str(&format!(
        "static GENERATED_FONT_METRICS: [FontMetric; {}] = [\n",
        metrics.len()
    ));
    for (idx, m) in metrics.iter().enumerate() {
        let var_prefix = format!("FONT_{}", idx);
        let hangul = compress_hangul(&m.char_widths, 4, 6, 3);
        let hangul_ref = if hangul.is_some() {
            format!("Some(&{}_HANGUL)", var_prefix)
        } else {
            "None".to_string()
        };

        out.push_str(&format!(
            "    FontMetric {{ name: \"{}\", bold: {}, italic: {}, em_size: {}, latin_ranges: &{}_LATIN_RANGES, hangul: {} }},\n",
            m.family_name.replace('"', "\\\""),
            m.bold,
            m.italic,
            m.em_size,
            var_prefix,
            hangul_ref
        ));
    }
    out.push_str("];\n");

    out
}

// ─── 명시적 생성 계획과 provenance metadata ───

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GenerationPlan {
    schema_version: u32,
    target_region: String,
    expected_entry_count: usize,
    inputs: Vec<GenerationInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GenerationInput {
    order: usize,
    path: String,
    face_index: u32,
    expected_identity: ExpectedIdentity,
    license: EvidenceDeclaration,
    provenance: EvidenceDeclaration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExpectedIdentity {
    family_name: String,
    bold: bool,
    italic: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EvidenceDeclaration {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    spdx: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratorManifest {
    schema_version: u32,
    generator: &'static str,
    generator_version: &'static str,
    generator_contract: &'static str,
    generator_source_sha256: String,
    target_region: String,
    expected_entry_count: usize,
    input_plan_sha256: String,
    generated_source_sha256: String,
    entries: Vec<GeneratedEntryMetadata>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedEntryMetadata {
    order: usize,
    source_path: String,
    source_sha256: String,
    face_index: u32,
    family_name: String,
    bold: bool,
    italic: bool,
    units_per_em: u16,
    naming_records: Vec<NamingRecord>,
    license: EvidenceMetadata,
    provenance: EvidenceMetadata,
    hangul_compression: HangulCompressionMetadata,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceMetadata {
    declaration: EvidenceDeclaration,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence_sha256: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HangulCompressionMetadata {
    status: &'static str,
    sample_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    cho_groups: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jung_groups: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jong_groups: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_error: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_error: Option<f64>,
}

fn validate_relative_path(path: &str, field: &str) -> Result<PathBuf, String> {
    let parsed = PathBuf::from(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "{}는 checkout 상대 경로여야 하고 '..'를 포함할 수 없음: {}",
            field, path
        ));
    }
    Ok(parsed)
}

fn validate_evidence(declaration: &EvidenceDeclaration, field: &str) -> Result<(), String> {
    match declaration.status.as_str() {
        "verified" => {
            if declaration.source.as_deref().unwrap_or_default().is_empty() {
                return Err(format!("{}.source는 verified 상태에서 필수", field));
            }
            if declaration
                .evidence_path
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            {
                return Err(format!("{}.evidencePath는 verified 상태에서 필수", field));
            }
            if field.ends_with(".license")
                && declaration.spdx.as_deref().unwrap_or_default().is_empty()
            {
                return Err(format!("{}.spdx는 verified 상태에서 필수", field));
            }
        }
        "unknown" | "local-only" => {
            if declaration.reason.as_deref().unwrap_or_default().is_empty() {
                return Err(format!(
                    "{}.reason은 {} 상태에서 필수",
                    field, declaration.status
                ));
            }
        }
        other => return Err(format!("{}.status 미지원 값: {}", field, other)),
    }
    if let Some(path) = declaration.evidence_path.as_deref() {
        validate_relative_path(path, &format!("{}.evidencePath", field))?;
    }
    Ok(())
}

fn load_generation_plan(path: &Path) -> Result<(GenerationPlan, String), String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {}", path.display(), error))?;
    let plan: GenerationPlan =
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {}", path.display(), error))?;
    if plan.schema_version != 1 {
        return Err(format!(
            "지원하지 않는 plan schemaVersion: {}",
            plan.schema_version
        ));
    }
    if plan.inputs.is_empty() {
        return Err("plan.inputs가 비어 있음".to_string());
    }
    if plan.expected_entry_count != plan.inputs.len() {
        return Err(format!(
            "expectedEntryCount={}와 inputs 길이={}가 다름",
            plan.expected_entry_count,
            plan.inputs.len()
        ));
    }
    match plan.target_region.as_str() {
        "canary" => {}
        "historical-generated-0-594" if plan.expected_entry_count == 595 => {}
        "historical-generated-0-594" => {
            return Err(
                "canonical historical generated plan은 정확히 595개 입력이어야 함".to_string(),
            );
        }
        other => return Err(format!("지원하지 않는 targetRegion: {}", other)),
    }
    let mut seen = std::collections::HashSet::new();
    for (index, input) in plan.inputs.iter().enumerate() {
        if input.order != index {
            return Err(format!(
                "입력 순서가 명시적 연속값이 아님: inputs[{}].order={} (expected {})",
                index, input.order, index
            ));
        }
        validate_relative_path(&input.path, &format!("inputs[{}].path", index))?;
        validate_evidence(&input.license, &format!("inputs[{}].license", index))?;
        validate_evidence(&input.provenance, &format!("inputs[{}].provenance", index))?;
        if !seen.insert((input.path.as_str(), input.face_index)) {
            return Err(format!(
                "동일 source/face가 plan에 중복됨: {}#{}; 자동 dedupe하지 않음",
                input.path, input.face_index
            ));
        }
    }
    Ok((plan, sha256_bytes(&bytes)))
}

fn evidence_metadata(declaration: &EvidenceDeclaration) -> Result<EvidenceMetadata, String> {
    let evidence_sha256 = declaration
        .evidence_path
        .as_deref()
        .map(Path::new)
        .map(sha256_file)
        .transpose()?;
    Ok(EvidenceMetadata {
        declaration: declaration.clone(),
        evidence_sha256,
    })
}

fn hangul_compression_metadata(metric: &FontMetric) -> HangulCompressionMetadata {
    let sample_count = metric
        .char_widths
        .keys()
        .filter(|&&codepoint| (HANGUL_BASE..=HANGUL_END).contains(&codepoint))
        .count();
    match compress_hangul(&metric.char_widths, 4, 6, 3) {
        Some(compressed) => HangulCompressionMetadata {
            status: "verified",
            sample_count,
            cho_groups: Some(compressed.cho_groups),
            jung_groups: Some(compressed.jung_groups),
            jong_groups: Some(compressed.jong_groups),
            max_error: Some(compressed.max_error),
            avg_error: Some(compressed.avg_error),
        },
        None => HangulCompressionMetadata {
            status: "not-applicable",
            sample_count,
            cho_groups: None,
            jung_groups: None,
            jong_groups: None,
            max_error: None,
            avg_error: None,
        },
    }
}

fn protected_output_name(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("font_metrics_data.rs" | "font_metrics_overlays.rs")
    )
}

fn resolved_output_path(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|error| format!("{}: {}", path.display(), error));
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("출력 파일명이 없음: {}", path.display()))?;
    parent
        .canonicalize()
        .map(|canonical_parent| canonical_parent.join(file_name))
        .map_err(|error| format!("{}: {}", parent.display(), error))
}

fn validate_output_paths(generated: &Path, metadata: &Path) -> Result<(), String> {
    let generated_resolved = resolved_output_path(generated)?;
    let metadata_resolved = resolved_output_path(metadata)?;
    let repository_root = env::current_dir()
        .and_then(|path| path.canonicalize())
        .map_err(|error| format!("현재 checkout 경로 확인 실패: {}", error))?;
    let protected_paths = [
        repository_root.join("src/renderer/font_metrics_data.rs"),
        repository_root.join("src/renderer/font_metrics_overlays.rs"),
    ];
    if protected_output_name(generated)
        || protected_output_name(metadata)
        || protected_paths.contains(&generated_resolved)
        || protected_paths.contains(&metadata_resolved)
    {
        return Err(
            "generator ownership 위반: core lookup 또는 measured/manual overlay는 출력 대상이 아님"
                .to_string(),
        );
    }
    if generated_resolved == metadata_resolved {
        return Err("generated output과 metadata output은 서로 달라야 함".to_string());
    }
    if generated.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Err("generated output 확장자는 .rs여야 함".to_string());
    }
    if metadata.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err("metadata output 확장자는 .json이어야 함".to_string());
    }
    Ok(())
}

fn is_canonical_generated_output(path: &Path) -> Result<bool, String> {
    let repository_root = env::current_dir()
        .and_then(|current| current.canonicalize())
        .map_err(|error| format!("현재 checkout 경로 확인 실패: {}", error))?;
    Ok(resolved_output_path(path)?
        == repository_root.join("src/renderer/font_metrics_generated.rs"))
}

fn generate_from_plan(
    plan_path: &Path,
    generated_output: &Path,
    metadata_output: &Path,
) -> Result<(), String> {
    validate_output_paths(generated_output, metadata_output)?;
    let (plan, input_plan_sha256) = load_generation_plan(plan_path)?;
    if is_canonical_generated_output(generated_output)?
        && plan.target_region != "historical-generated-0-594"
    {
        return Err(
            "canonical generated DB 출력에는 595-entry historical-generated-0-594 plan이 필요"
                .to_string(),
        );
    }
    let mut metrics = Vec::with_capacity(plan.inputs.len());
    for input in &plan.inputs {
        let metric = parse_ttf_face(Path::new(&input.path), input.face_index)?;
        if metric.family_name != input.expected_identity.family_name
            || metric.bold != input.expected_identity.bold
            || metric.italic != input.expected_identity.italic
        {
            return Err(format!(
                "{}#{} identity drift: 실제=({}, bold={}, italic={}), 기대=({}, bold={}, italic={})",
                input.path,
                input.face_index,
                metric.family_name,
                metric.bold,
                metric.italic,
                input.expected_identity.family_name,
                input.expected_identity.bold,
                input.expected_identity.italic
            ));
        }
        metrics.push(metric);
    }

    let generated_source = generate_rust_source(&metrics);
    let generated_source_sha256 = sha256_bytes(generated_source.as_bytes());
    let entries = plan
        .inputs
        .iter()
        .zip(metrics.iter())
        .map(|(input, metric)| {
            Ok(GeneratedEntryMetadata {
                order: input.order,
                source_path: input.path.clone(),
                source_sha256: metric.source_sha256.clone(),
                face_index: metric.face_index,
                family_name: metric.family_name.clone(),
                bold: metric.bold,
                italic: metric.italic,
                units_per_em: metric.em_size,
                naming_records: metric.naming_records.clone(),
                license: evidence_metadata(&input.license)?,
                provenance: evidence_metadata(&input.provenance)?,
                hangul_compression: hangul_compression_metadata(metric),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let manifest = GeneratorManifest {
        schema_version: 1,
        generator: "font-metric-gen",
        generator_version: env!("CARGO_PKG_VERSION"),
        generator_contract: "generated-data-and-provenance-only-v1",
        generator_source_sha256: sha256_bytes(GENERATOR_SOURCE_BYTES),
        target_region: plan.target_region,
        expected_entry_count: plan.expected_entry_count,
        input_plan_sha256,
        generated_source_sha256,
        entries,
    };
    let metadata = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("metadata 직렬화 실패: {}", error))?
        + "\n";

    // 모든 입력 파싱과 산출물 직렬화를 끝낸 뒤에만 쓰기 시작한다.
    fs::write(generated_output, generated_source)
        .map_err(|error| format!("{}: {}", generated_output.display(), error))?;
    fs::write(metadata_output, metadata)
        .map_err(|error| format!("{}: {}", metadata_output.display(), error))?;
    Ok(())
}

// ─── 진단 출력 ───

fn print_diagnostic(metric: &FontMetric) {
    let total_chars = metric.char_widths.len();
    let hangul_count = metric
        .char_widths
        .keys()
        .filter(|&&c| (HANGUL_BASE..=HANGUL_END).contains(&c))
        .count();
    let latin_count = metric
        .char_widths
        .keys()
        .filter(|&&c| (0x20..=0x7E).contains(&c))
        .count();

    let style = match (metric.bold, metric.italic) {
        (false, false) => "Regular",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (true, true) => "Bold Italic",
    };
    println!("  패밀리: {} [{}]", metric.family_name, style);
    println!("  파일: {}", metric.file_name);
    println!("  source SHA-256: {}", metric.source_sha256);
    println!("  TTC face index: {}", metric.face_index);
    println!("  name records: {}", metric.naming_records.len());
    println!("  em크기: {}", metric.em_size);
    println!("  총 글리프: {}", total_chars);
    println!("  한글 음절: {} / 11172", hangul_count);
    println!("  Basic Latin: {} / 95", latin_count);

    // 한글 압축 진단
    if hangul_count > 0 {
        if let Some(h) = compress_hangul(&metric.char_widths, 4, 6, 3) {
            println!(
                "  한글 압축: {}×{}×{} = {} 그룹 (최대오차: {} em단위, 평균오차: {:.1})",
                h.cho_groups,
                h.jung_groups,
                h.jong_groups,
                h.widths.len(),
                h.max_error,
                h.avg_error
            );
        }
    }

    // 샘플 폭
    let sample_chars = ['A', 'a', 'W', 'i', ' ', '가', '한', '글'];
    print!("  샘플 폭:");
    for ch in sample_chars {
        if let Some(&w) = metric.char_widths.get(&(ch as u32)) {
            print!(" {}={}", ch, w);
        }
    }
    println!();
}

fn argument_value(args: &[String], flag: &str) -> Result<Option<String>, String> {
    let Some(index) = args.iter().position(|argument| argument == flag) else {
        return Ok(None);
    };
    args.get(index + 1)
        .cloned()
        .map(Some)
        .ok_or_else(|| format!("{} 값이 없음", flag))
}

fn print_usage() {
    eprintln!("사용법:");
    eprintln!("  font-metric-gen <파일.ttf> [--face-index N]  # 단일 face 진단");
    eprintln!("  font-metric-gen --dir <폴더> --list          # 폴더 내 폰트 목록");
    eprintln!("  font-metric-gen --plan <plan.json> --generated-output <출력.rs> --metadata-output <출력.json>");
    eprintln!();
    eprintln!(
        "생성 모드는 plan의 명시적 order와 faceIndex를 그대로 보존하며 sort/dedupe하지 않습니다."
    );
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Err("인자가 없음".to_string());
    }

    if args[1] == "--plan" {
        let plan = argument_value(&args, "--plan")?
            .map(PathBuf::from)
            .ok_or_else(|| "--plan 값이 없음".to_string())?;
        let generated_output = argument_value(&args, "--generated-output")?
            .map(PathBuf::from)
            .ok_or_else(|| "--generated-output이 필수".to_string())?;
        let metadata_output = argument_value(&args, "--metadata-output")?
            .map(PathBuf::from)
            .ok_or_else(|| "--metadata-output이 필수".to_string())?;
        generate_from_plan(&plan, &generated_output, &metadata_output)?;
        println!("generated data: {}", generated_output.display());
        println!("provenance metadata: {}", metadata_output.display());
        return Ok(());
    }

    if args[1] == "--dir" {
        if !args.iter().any(|argument| argument == "--list") {
            return Err(
                "--dir 생성은 암묵적 sort/dedupe 때문에 폐기됨; --plan 생성 모드를 사용하세요"
                    .to_string(),
            );
        }
        let dir = argument_value(&args, "--dir")?
            .map(PathBuf::from)
            .ok_or_else(|| "--dir 값이 없음".to_string())?;

        let mut entries: Vec<_> = fs::read_dir(&dir)
            .map_err(|error| format!("{}: {}", dir.display(), error))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc")
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        println!("폰트 목록 ({} 파일):", entries.len());
        for entry in &entries {
            match parse_ttf_all(&entry.path()) {
                Ok(metrics) => {
                    for metric in metrics {
                        let style = match (metric.bold, metric.italic) {
                            (false, false) => "",
                            (true, false) => " [B]",
                            (false, true) => " [I]",
                            (true, true) => " [BI]",
                        };
                        println!(
                            "  {}#{} → \"{}\"{} (em={}, 글리프={})",
                            metric.file_name,
                            metric.face_index,
                            metric.family_name,
                            style,
                            metric.em_size,
                            metric.char_widths.len()
                        );
                    }
                }
                Err(error) => println!(
                    "  {} → 오류: {}",
                    entry.file_name().to_string_lossy(),
                    error
                ),
            }
        }
        return Ok(());
    } else {
        // 단일 파일 진단
        let path = PathBuf::from(&args[1]);
        let face_index = argument_value(&args, "--face-index")?
            .map(|value| {
                value
                    .parse::<u32>()
                    .map_err(|error| format!("--face-index: {}", error))
            })
            .transpose()?
            .unwrap_or(0);
        let metric = parse_ttf_face(&path, face_index)?;
        print_diagnostic(&metric);
    }
    Ok(())
}

// ─── main ───

fn main() {
    if let Err(error) = run() {
        eprintln!("오류: {}", error);
        std::process::exit(1);
    }
}
