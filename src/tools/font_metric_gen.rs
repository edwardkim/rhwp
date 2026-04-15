//! 폰트 메트릭 DB 생성 도구
//!
//! TTF 파일에서 head/cmap/hmtx/maxp/name 테이블을 파싱하여
//! 글리프 폭 데이터를 추출하고, Rust 소스코드로 출력한다.
//!
//! 사용법:
//!   cargo run --bin font-metric-gen -- ttfs/windows/malgun.ttf
//!   cargo run --bin font-metric-gen -- --dir ttfs/windows/ --output src/renderer/font_metrics_data.rs

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

// ─── name 테이블: 폰트 패밀리명 ───

fn parse_name(data: &[u8], tables: &[TableEntry]) -> String {
    let Some(name_off) = table_offset(tables, "name") else {
        return String::new();
    };
    let count = read_u16_be(data, name_off + 2) as usize;
    let string_offset = name_off + read_u16_be(data, name_off + 4) as usize;

    // nameID=1 (Font Family), platformID=3 (Windows), encodingID=1 (Unicode BMP)
    for i in 0..count {
        let rec = name_off + 6 + i * 12;
        if rec + 12 > data.len() {
            break;
        }
        let platform_id = read_u16_be(data, rec);
        let encoding_id = read_u16_be(data, rec + 2);
        let name_id = read_u16_be(data, rec + 6);
        let length = read_u16_be(data, rec + 8) as usize;
        let offset = string_offset + read_u16_be(data, rec + 10) as usize;

        if name_id == 1 && platform_id == 3 && encoding_id == 1 && offset + length <= data.len() {
            // UTF-16 BE 디코딩
            let mut s = String::new();
            for j in (0..length).step_by(2) {
                let ch = read_u16_be(data, offset + j);
                if let Some(c) = char::from_u32(ch as u32) {
                    s.push(c);
                }
            }
            return s;
        }
    }

    // 폴백: platformID=1 (Mac), encodingID=0 (Roman)
    for i in 0..count {
        let rec = name_off + 6 + i * 12;
        if rec + 12 > data.len() {
            break;
        }
        let platform_id = read_u16_be(data, rec);
        let name_id = read_u16_be(data, rec + 6);
        let length = read_u16_be(data, rec + 8) as usize;
        let offset = string_offset + read_u16_be(data, rec + 10) as usize;

        if name_id == 1 && platform_id == 1 && offset + length <= data.len() {
            return String::from_utf8_lossy(&data[offset..offset + length]).to_string();
        }
    }

    String::new()
}

// ─── 폰트 메트릭 데이터 구조 ───

#[derive(Debug)]
struct FontMetric {
    family_name: String,
    file_name: String,
    em_size: u16,
    bold: bool,
    italic: bool,
    /// Unicode codepoint → advance width (em 단위)
    char_widths: HashMap<u32, u16>,
}

/// 단일 TTF 또는 TTC의 모든 폰트를 파싱
fn parse_ttf(path: &Path) -> Result<FontMetric, String> {
    parse_ttf_all(path).and_then(|v| v.into_iter().next().ok_or_else(|| "폰트 없음".to_string()))
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

    for &font_off in &offsets {
        let tables = parse_table_directory_at(&data, font_off);
        if tables.is_empty() {
            continue;
        }

        let head = parse_head(&data, &tables);
        let num_glyphs = parse_maxp(&data, &tables);
        let cmap = parse_cmap(&data, &tables);
        let hmtx = parse_hmtx(&data, &tables, num_glyphs);
        let family_name = parse_name(&data, &tables);

        let mut char_widths = HashMap::new();
        for (&codepoint, &glyph_id) in &cmap {
            if (glyph_id as usize) < hmtx.len() {
                char_widths.insert(codepoint, hmtx[glyph_id as usize]);
            }
        }

        results.push(FontMetric {
            family_name,
            file_name: file_name.clone(),
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

    out.push_str("//! 폰트 메트릭 데이터 (자동 생성)\n");
    out.push_str("//!\n");
    out.push_str("//! font-metric-gen 도구로 TTF 파일에서 추출.\n");
    out.push_str("//! 수동 편집 금지.\n\n");

    // 한글 음절분해 구조체
    out.push_str("#[derive(Debug)]\n");
    out.push_str("pub struct HangulMetric {\n");
    out.push_str("    pub cho_groups: u8,\n");
    out.push_str("    pub jung_groups: u8,\n");
    out.push_str("    pub jong_groups: u8,\n");
    out.push_str("    pub cho_map: &'static [u8],\n");
    out.push_str("    pub jung_map: &'static [u8],\n");
    out.push_str("    pub jong_map: &'static [u8],\n");
    out.push_str("    pub widths: &'static [u16],\n");
    out.push_str("}\n\n");

    // 폰트 메트릭 구조체
    out.push_str("#[derive(Debug)]\n");
    out.push_str("pub struct FontMetric {\n");
    out.push_str("    pub name: &'static str,\n");
    out.push_str("    pub bold: bool,\n");
    out.push_str("    pub italic: bool,\n");
    out.push_str("    pub em_size: u16,\n");
    out.push_str("    pub latin_ranges: &'static [LatinRange],\n");
    out.push_str("    pub hangul: Option<&'static HangulMetric>,\n");
    out.push_str("}\n\n");

    out.push_str("#[derive(Debug)]\n");
    out.push_str("pub struct LatinRange {\n");
    out.push_str("    pub start: u32,\n");
    out.push_str("    pub end: u32,\n");
    out.push_str("    pub widths: &'static [u16],\n");
    out.push_str("}\n\n");

    // 조회 함수
    out.push_str("impl FontMetric {\n");
    out.push_str("    pub fn get_width(&self, ch: char) -> Option<u16> {\n");
    out.push_str("        let code = ch as u32;\n");
    out.push_str("        // 한글 음절 (U+AC00~U+D7A3)\n");
    out.push_str("        if code >= 0xAC00 && code <= 0xD7A3 {\n");
    out.push_str("            if let Some(h) = self.hangul {\n");
    out.push_str("                let idx = code - 0xAC00;\n");
    out.push_str("                let cho = (idx / (21 * 28)) as usize;\n");
    out.push_str("                let jung = ((idx % (21 * 28)) / 28) as usize;\n");
    out.push_str("                let jong = (idx % 28) as usize;\n");
    out.push_str("                let gi = h.cho_map[cho] as usize\n");
    out.push_str("                    * h.jung_groups as usize * h.jong_groups as usize\n");
    out.push_str("                    + h.jung_map[jung] as usize * h.jong_groups as usize\n");
    out.push_str("                    + h.jong_map[jong] as usize;\n");
    out.push_str("                return h.widths.get(gi).copied();\n");
    out.push_str("            }\n");
    out.push_str("            return None;\n");
    out.push_str("        }\n");
    out.push_str("        // Latin 및 기타 범위\n");
    out.push_str("        for range in self.latin_ranges {\n");
    out.push_str("            if code >= range.start && code <= range.end {\n");
    out.push_str("                let w = range.widths[(code - range.start) as usize];\n");
    out.push_str("                return if w > 0 { Some(w) } else { None };\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        None\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // 이름+스타일로 검색 (bold/italic 일치 → fallback to Regular)
    out.push_str("pub fn find_metric(name: &str, bold: bool, italic: bool) -> Option<&'static FontMetric> {\n");
    out.push_str("    // 정확한 매칭 (name + bold + italic)\n");
    out.push_str("    if let Some(m) = FONT_METRICS.iter().find(|m| m.name == name && m.bold == bold && m.italic == italic) {\n");
    out.push_str("        return Some(m);\n");
    out.push_str("    }\n");
    out.push_str("    // bold만 매칭 (italic 무시)\n");
    out.push_str("    if let Some(m) = FONT_METRICS.iter().find(|m| m.name == name && m.bold == bold && !m.italic) {\n");
    out.push_str("        return Some(m);\n");
    out.push_str("    }\n");
    out.push_str("    // Regular 폴백\n");
    out.push_str("    FONT_METRICS.iter().find(|m| m.name == name)\n");
    out.push_str("}\n\n");

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

    // FONT_METRICS 배열
    out.push_str(&format!(
        "pub static FONT_METRICS: [FontMetric; {}] = [\n",
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

// ─── main ───

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("사용법:");
        eprintln!("  font-metric-gen <파일.ttf>              # 단일 파일 진단");
        eprintln!("  font-metric-gen --dir <폴더> [--output <출력.rs>]  # 폴더 일괄 처리");
        eprintln!("  font-metric-gen --dir <폴더> --list      # 폴더 내 폰트 목록");
        std::process::exit(1);
    }

    if args[1] == "--dir" {
        let dir = PathBuf::from(&args[2]);
        let list_mode = args.iter().any(|a| a == "--list");
        let output_path = args
            .iter()
            .position(|a| a == "--output")
            .map(|i| PathBuf::from(&args[i + 1]));

        let mut entries: Vec<_> = fs::read_dir(&dir)
            .expect("디렉토리 열기 실패")
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc")
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        if list_mode {
            println!("폰트 목록 ({} 파일):", entries.len());
            for entry in &entries {
                match parse_ttf(&entry.path()) {
                    Ok(m) => {
                        let style = match (m.bold, m.italic) {
                            (false, false) => "",
                            (true, false) => " [B]",
                            (false, true) => " [I]",
                            (true, true) => " [BI]",
                        };
                        println!(
                            "  {} → \"{}\"{} (em={}, 글리프={})",
                            m.file_name,
                            m.family_name,
                            style,
                            m.em_size,
                            m.char_widths.len()
                        );
                    }
                    Err(e) => println!("  {} → 오류: {}", entry.file_name().to_string_lossy(), e),
                }
            }
            return;
        }

        let mut metrics = Vec::new();
        let mut errors = Vec::new();

        println!("TTF 파싱 중... ({} 파일)", entries.len());
        for entry in &entries {
            match parse_ttf_all(&entry.path()) {
                Ok(fonts) => {
                    for m in fonts {
                        if !list_mode {
                            print_diagnostic(&m);
                            println!();
                        }
                        metrics.push(m);
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        println!("파싱 성공: {} / 실패: {}", metrics.len(), errors.len());
        if !errors.is_empty() {
            println!("실패 목록:");
            for e in &errors {
                println!("  {}", e);
            }
        }

        // 중복 제거: (family_name, bold, italic) 동일 시 글리프 수 최대인 것만 유지
        let before_dedup = metrics.len();
        let mut deduped: Vec<FontMetric> = Vec::new();
        for m in metrics {
            if let Some(existing) = deduped.iter_mut().find(|e| {
                e.family_name == m.family_name && e.bold == m.bold && e.italic == m.italic
            }) {
                if m.char_widths.len() > existing.char_widths.len() {
                    *existing = m;
                }
            } else {
                deduped.push(m);
            }
        }
        // 우선 폰트를 앞쪽에 배치 (HWP 기본 폰트 → 기타)
        let priority_fonts: Vec<&str> = vec![
            "함초롬돋움",
            "함초롬바탕",
            "HCR Batang",
            "HCR Dotum",
            "Malgun Gothic",
            "맑은 고딕",
            "Haansoft Batang",
            "Haansoft Dotum",
            "NanumGothic",
            "NanumMyeongjo",
            "NanumBarunGothic",
            "Noto Sans KR",
            "Noto Serif KR",
            "Arial",
            "Times New Roman",
            "Calibri",
            "Verdana",
            "Tahoma",
            "Batang",
            "Dotum",
            "Gulim",
            "Gungsuh",
        ];
        deduped.sort_by(|a, b| {
            let pa = priority_fonts.iter().position(|&p| p == a.family_name);
            let pb = priority_fonts.iter().position(|&p| p == b.family_name);
            match (pa, pb) {
                (Some(ia), Some(ib)) => ia
                    .cmp(&ib)
                    .then(a.bold.cmp(&b.bold))
                    .then(a.italic.cmp(&b.italic)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a
                    .family_name
                    .cmp(&b.family_name)
                    .then(a.bold.cmp(&b.bold))
                    .then(a.italic.cmp(&b.italic)),
            }
        });
        println!("중복 제거: {} → {} 엔트리", before_dedup, deduped.len());

        if let Some(output) = output_path {
            println!("\nRust 소스코드 생성 중...");
            let source = generate_rust_source(&deduped);
            fs::write(&output, &source).expect("출력 파일 쓰기 실패");
            println!(
                "출력: {} ({} 바이트, {} 폰트)",
                output.display(),
                source.len(),
                deduped.len()
            );
        }
    } else {
        // 단일 파일 진단
        let path = PathBuf::from(&args[1]);
        match parse_ttf(&path) {
            Ok(m) => print_diagnostic(&m),
            Err(e) => {
                eprintln!("오류: {}", e);
                std::process::exit(1);
            }
        }
    }
}
