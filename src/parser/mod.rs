//! HWP/HWPX 파일 파서 모듈
//!
//! HWP 5.0 바이너리 또는 HWPX(XML) 파일을 파싱하여 IR(Document Model)로 변환한다.
//!
//! ## HWP 바이너리 파싱 순서
//! 1. CFB 컨테이너 열기 (cfb_reader)
//! 2. FileHeader 파싱 (header)
//! 3. DocInfo 파싱 → 참조 테이블 구축 (doc_info)
//! 4. BodyText 파싱 → 섹션/문단 (body_text)
//! 5. 컨트롤 파싱 → 표/도형/그림 (control)
//!
//! ## HWPX 파싱 순서
//! 1. ZIP 컨테이너 열기 (hwpx/reader)
//! 2. content.hpf → 섹션 목록 (hwpx/content)
//! 3. header.xml → DocInfo (hwpx/header)
//! 4. section*.xml → Section (hwpx/section)

pub mod bin_data;
pub mod body_text;
pub mod byte_reader;
pub mod cfb_reader;
pub mod control;
pub mod crypto;
pub mod doc_info;
pub mod header;
pub mod hwp3;
pub mod hwpx;
pub mod ingest;
pub mod ole_container;
pub mod record;
pub mod tags;

use crate::model::bin_data::BinDataContent;
use crate::model::document::{
    Document, FileHeader as ModelFileHeader, HwpVersion as ModelHwpVersion, Preview, PreviewImage,
    PreviewImageFormat,
};

/// 파일 포맷 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileFormat {
    /// HWP 5.0 바이너리 (CFB/OLE 컨테이너)
    Hwp,
    /// HWPX (XML 기반, ZIP 컨테이너)
    Hwpx,
    /// HWP 3.0 바이너리
    Hwp3,
    /// Legacy raw HWPML XML (미지원 — 감지만, Issue #1053)
    LegacyHwpml,
    /// DRM/보안 컨테이너로 보호된 문서 (미지원 — 감지만, Issue #1982)
    /// Fasoo(`\x9b DRMONE`) / SoftCamp SCDSA(`SCDSA00x`) 등. 복호화는 범위 밖.
    DrmProtected,
    /// 빈 파일(0 바이트) (Issue #1982)
    Empty,
    /// 알 수 없는 포맷
    Unknown,
}

const FORMAT_PROBE_SCAN_LIMIT: usize = 4096;
const UNSUPPORTED_HWPML_CODE: &str = "UNSUPPORTED_HWPML";
const UNSUPPORTED_FILE_FORMAT_CODE: &str = "UNSUPPORTED_FILE_FORMAT";
const SUPPORTED_FORMATS_HINT: &str = "현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다.";
const UNSUPPORTED_HWPML_HINT: &str =
    "현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다. 한컴오피스에서 HWP 5.0 또는 HWPX로 다시 저장한 뒤 열어주세요.";
const DRM_PROTECTED_CODE: &str = "DRM_PROTECTED";
const DRM_PROTECTED_HINT: &str =
    "DRM/보안 컨테이너로 보호된 문서입니다. 한컴오피스 등 DRM 클라이언트에서 보호를 해제한 뒤 저장해 열어주세요.";
const EMPTY_FILE_CODE: &str = "EMPTY_FILE";
const EMPTY_FILE_HINT: &str = "빈 파일(0 바이트)입니다.";

// DRM/보안 컨테이너 시그니처 (Issue #1982 — 10k 서베이 검출).
// Fasoo: `\x9b DRMONE  This Document is encrypted and protected by Fasoo`.
const FASOO_DRM_SIG: &[u8] = b"\x9b DRMONE";
// SoftCamp SCDSA(Security Content Document Security Agent): `SCDSA002`/`SCDSA004`.
const SCDSA_SIG: &[u8] = b"SCDSA";

/// 파일 데이터의 매직 바이트로 포맷을 감지한다.
pub fn detect_format(data: &[u8]) -> FileFormat {
    if data.is_empty() {
        return FileFormat::Empty;
    }
    // DRM/보안 컨테이너(미지원 — 감지만, Issue #1982). 정상 매직보다 먼저 판별해
    // "알 수 없는 파일 형식" 대신 명확한 안내를 준다.
    if data.starts_with(FASOO_DRM_SIG) || data.starts_with(SCDSA_SIG) {
        return FileFormat::DrmProtected;
    }
    if data.len() >= 8 {
        // CFB/OLE 시그니처: D0 CF 11 E0 A1 B1 1A E1
        if data[0] == 0xD0 && data[1] == 0xCF && data[2] == 0x11 && data[3] == 0xE0 {
            return FileFormat::Hwp;
        }
        // ZIP 시그니처: 50 4B 03 04 ("PK\x03\x04")
        if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 {
            return FileFormat::Hwpx;
        }
    }
    // HWP 3.0 바이너리 (Issue #265): "HWP Document File" 프리픽스.
    // V3.00 ~ 2.x/초기 한컴 워디안까지 관대하게 포괄.
    if data.len() >= 17 && &data[0..17] == b"HWP Document File" {
        return FileFormat::Hwp3;
    }
    if detect_legacy_hwpml(data) {
        return FileFormat::LegacyHwpml;
    }
    FileFormat::Unknown
}

fn format_probe_prefix(data: &[u8]) -> &[u8] {
    &data[..data.len().min(FORMAT_PROBE_SCAN_LIMIT)]
}

fn trim_format_probe_prefix(data: &[u8]) -> &[u8] {
    let mut start = if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else {
        0
    };
    while start < data.len() && data[start].is_ascii_whitespace() {
        start += 1;
    }
    &data[start..]
}

fn ascii_lower(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + 32
    } else {
        byte
    }
}

fn ascii_eq_ignore_case(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(a, b)| ascii_lower(*a) == ascii_lower(*b))
}

fn starts_with_ascii_ignore_case(data: &[u8], needle: &[u8]) -> bool {
    data.get(..needle.len())
        .is_some_and(|head| ascii_eq_ignore_case(head, needle))
}

fn contains_ascii_ignore_case(data: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && data
            .windows(needle.len())
            .any(|window| ascii_eq_ignore_case(window, needle))
}

fn detect_legacy_hwpml(data: &[u8]) -> bool {
    let prefix = trim_format_probe_prefix(format_probe_prefix(data));
    let looks_like_xml = starts_with_ascii_ignore_case(prefix, b"<?xml")
        || starts_with_ascii_ignore_case(prefix, b"<hwpml");

    looks_like_xml && contains_ascii_ignore_case(prefix, b"<hwpml")
}

fn detect_legacy_hwpml_format_name(data: &[u8]) -> &'static str {
    let prefix = format_probe_prefix(data);
    let versions: &[(&[u8], &'static str)] = &[
        (b"version=\"2.1\"", "HWPML 2.1"),
        (b"version='2.1'", "HWPML 2.1"),
        (b"hwpml=\"2.1\"", "HWPML 2.1"),
        (b"hwpml='2.1'", "HWPML 2.1"),
        (b"version=\"2.0\"", "HWPML 2.0"),
        (b"version='2.0'", "HWPML 2.0"),
        (b"version=\"1.0\"", "HWPML 1.0"),
        (b"version='1.0'", "HWPML 1.0"),
    ];
    for (needle, format_name) in versions {
        if contains_ascii_ignore_case(prefix, needle) {
            return format_name;
        }
    }
    "HWPML"
}

/// 파싱 에러 (통합)
#[derive(Debug)]
pub enum ParseError {
    CfbError(cfb_reader::CfbError),
    HeaderError(header::HeaderError),
    DocInfoError(doc_info::DocInfoError),
    BodyTextError(body_text::BodyTextError),
    CryptoError(crypto::CryptoError),
    HwpxError(hwpx::HwpxError),
    Hwp3Error(hwp3::Hwp3Error),
    EncryptedDocument,
    /// 감지는 되었으나 지원하지 않는 포맷
    UnsupportedFormat {
        code: &'static str,
        format: &'static str,
        hint: &'static str,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::CfbError(e) => write!(f, "CFB 오류: {}", e),
            ParseError::HeaderError(e) => write!(f, "헤더 오류: {}", e),
            ParseError::DocInfoError(e) => write!(f, "DocInfo 오류: {}", e),
            ParseError::BodyTextError(e) => write!(f, "BodyText 오류: {}", e),
            ParseError::CryptoError(e) => write!(f, "암호 오류: {}", e),
            ParseError::HwpxError(e) => write!(f, "HWPX 오류: {}", e),
            ParseError::Hwp3Error(e) => write!(f, "HWP 3.0 오류: {}", e),
            ParseError::EncryptedDocument => write!(f, "암호화된 문서는 지원하지 않습니다"),
            ParseError::UnsupportedFormat { code, format, hint } => {
                write!(
                    f,
                    "지원하지 않는 포맷입니다: {format}. 오류코드: {code}. {hint}"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl From<hwpx::HwpxError> for ParseError {
    fn from(e: hwpx::HwpxError) -> Self {
        ParseError::HwpxError(e)
    }
}

impl From<hwp3::Hwp3Error> for ParseError {
    fn from(e: hwp3::Hwp3Error) -> Self {
        ParseError::Hwp3Error(e)
    }
}

/// HWP 파일 바이트 데이터를 파싱하여 Document IR로 변환
///
/// 파싱 순서:
/// 1. CFB 컨테이너 열기
/// 2. FileHeader 파싱 (버전, 플래그)
/// 3. DocInfo 파싱 (참조 테이블)
/// 4. BodyText 섹션별 파싱 (배포용 문서: ViewText 복호화)
pub fn parse_hwp(data: &[u8]) -> Result<Document, ParseError> {
    // 1. CFB 컨테이너 열기 (strict → lenient 폴백)
    match cfb_reader::CfbReader::open(data) {
        Ok(cfb) => parse_hwp_with_cfb(cfb, data),
        Err(strict_err) => {
            eprintln!(
                "표준 CFB 파서 실패: {}, lenient 파서로 재시도...",
                strict_err
            );
            let lenient = cfb_reader::LenientCfbReader::open(data)
                .map_err(|_| ParseError::CfbError(strict_err))?;
            parse_hwp_with_lenient(lenient, data)
        }
    }
}

/// 표준 CfbReader로 파싱
fn parse_hwp_with_cfb(
    mut cfb: cfb_reader::CfbReader,
    _raw_data: &[u8],
) -> Result<Document, ParseError> {
    // 2. FileHeader 파싱
    let header_data = cfb.read_file_header().map_err(ParseError::CfbError)?;
    let file_header = header::parse_file_header(&header_data).map_err(ParseError::HeaderError)?;

    if file_header.flags.encrypted {
        return Err(ParseError::EncryptedDocument);
    }

    let compressed = file_header.flags.compressed;
    let distribution = file_header.flags.distribution;

    // 3. DocInfo 파싱
    let doc_info_data = cfb
        .read_doc_info(compressed)
        .map_err(ParseError::CfbError)?;
    let (mut doc_info, doc_properties) =
        doc_info::parse_doc_info(&doc_info_data).map_err(ParseError::DocInfoError)?;
    doc_info.raw_stream = Some(doc_info_data);

    // 4. BodyText 섹션별 파싱
    let section_count = cfb.section_count();
    let sections = parse_sections_strict(&mut cfb, section_count, compressed, distribution)?;

    // 5-7. 미리보기, BinData, 추가 스트림
    let preview = extract_preview(&mut cfb);
    let bin_data_content = load_bin_data_content(&mut cfb, &doc_info.bin_data_list, compressed);
    let extra_streams = collect_extra_streams(&mut cfb, &doc_info.bin_data_list, &bin_data_content);

    // Document 조립
    let model_header = ModelFileHeader {
        version: ModelHwpVersion {
            major: file_header.version.major,
            minor: file_header.version.minor,
            build: file_header.version.build,
            revision: file_header.version.revision,
        },
        flags: file_header.flags.raw,
        compressed,
        encrypted: file_header.flags.encrypted,
        distribution,
        raw_data: Some(header_data),
    };

    // [Task #1001] HWP3 변환본 식별 — HwpSummary HWP3 시대 년 검출.
    // sample16-hwp5 같은 복잡한 변환본 (Task #554 의 PS<0.05 휴리스틱 미적용)
    // 도 식별. 단 false positive (예: HWP5 에 HWP3 시대 텍스트만 인용된 일반
    // 문서 — exam_eng) 차단 위해 PS/CS 비율도 추가 검증 (variant 는 작성자
    // 다양한 스타일 사용 안하므로 작은 비율).
    let summary_hwp3_era = cfb.detect_hwp3_variant();

    // [Issue #1770] rhwp HWPX→HWP 변환본 식별 — 마커 스트림 감지 (결정론).
    // 변환본 IR 은 HWPX LINE_SEG 시멘틱 그대로이므로 pagination/렌더의
    // is_hwpx_source 분기를 HWPX 로 해석해야 roundtrip 쪽수가 자기정합한다.
    let is_hwpx_variant = extra_streams
        .iter()
        .any(|(p, _)| p == crate::document_core::converters::hwpx_to_hwp::HWPX_ORIGIN_STREAM_PATH);

    let mut doc = Document {
        header: model_header,
        doc_properties,
        doc_info,
        sections,
        preview,
        bin_data_content,
        extra_streams,
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
        is_hwpx_variant,
    };

    // 자동 번호 할당 (문서 전체에서 순차적으로)
    assign_auto_numbers(&mut doc);

    // [Task #554] HWP3 → HWP5 변환본 식별 + page_def margin_bottom 보정
    apply_hwp3_origin_fixup(&mut doc);

    // [Task #1001] HwpSummary HWP3 시대 년 AND PS/CS 비율 작음 → 변환본 확정.
    // 두 신호 결합으로 false positive 차단 (exam_eng 등 일반 HWP5 가 본문에
    // HWP3 시대 텍스트만 인용한 경우).
    // [#1880 v2] rhwp HWPX→HWP 변환본 제외 — 원본 HWPX 가 HWP3-계보 요약정보를
    // 승계해도 rhwp 변환본 IR 은 HWPX 시멘틱이므로 spacing 반감 보정이 오발동
    // 하면 안 된다 (위 apply_hwp3_origin_fixup 게이트와 동일 근거).
    if summary_hwp3_era && !doc.is_hwpx_variant {
        let total_paras: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
        if total_paras > 50 {
            let ps_r = doc.doc_info.para_shapes.len() as f64 / total_paras as f64;
            let cs_r = doc.doc_info.char_shapes.len() as f64 / total_paras as f64;
            if ps_r < 0.20 && cs_r < 0.20 {
                doc.is_hwp3_variant = true;
                // [Task #1001 Stage 11] line_segs.vertical_pos /2 보정 revert —
                // 실제 raw vpos 비교 결과 HWP5 변환본 vpos 는 HWP3 의 2배가 아닌
                // ~1.15배 (15% 만 차이). /2 fix 시 HWP5 가 HWP3 보다 더 compact 되어
                // 한컴 정합 페이지 분할 회귀 (한컴은 section 2 가 새 페이지 vs rhwp
                // 는 같은 페이지에 packed). vpos 보정 없이 ParaShape /4 만으로 정합.

                // [Task #1037 → #1042 정정] ParaShape unit semantic normalize.
                // HWP3 vs HWP5 variant 비교 결과 (diag_1042_hwp3_vs_hwp5_paragraph):
                //   - margin_left/right: HWP5 raw = HWP3 raw 동일 → /2 적용은 wrong
                //   - spacing_before / spacing_after: HWP5 raw = HWP3 × 2 → /2 정합
                // margin_left/right /2 제거 — HWP3 정답 paragraph 분포 정합.
                //
                // [Task #1472] indent /2 제거 — IR 은 정답(full HWPUNIT, HWPX 일치)로 둔다.
                //   종전 indent /2 는 본문 내어쓰기를 절반으로 훼손(한컴/HWPX 와 어긋남)하면서,
                //   미주 TAC 수식 흐름의 available_width 계산이 indent_scale=2.0 으로 이를 되돌려
                //   "수식 effective indent = (indent/2)×2 = full" 로 페이지네이션을 한컴과 정합시켰다.
                //   재설계: IR indent 는 full 로 두고, 미주 수식 흐름의 indent_scale 을 변환본에서만
                //   절반(2.0→1.0)으로 낮춰 effective indent(=full) 를 불변 유지한다(아래 렌더러).
                for ps in &mut doc.doc_info.para_shapes {
                    ps.spacing_before /= 2;
                    ps.spacing_after /= 2;
                }
            }
        }
    }

    // [Task #873] BinData Link 타입 의 외부 file path 영역 Picture.external_path 전달.
    // 이후 model::document::populate_external_images_from_dir (Task #741) 가 같은
    // dir 영역 basename 매칭 영역 image 영역 자동 load.
    populate_link_image_paths(&mut doc);

    // [Task #1042 Stage 5] HWP5 variant 의 paragraph data raw vpos normalize —
    // HWP3 vs HWP5 variant 진단 결과 HWP5 의 raw vpos = HWP3 vpos + cumulative
    // spacing_before. paragraph 마다 +sb 누적 → paragraph_layout 의 외부 path
    // (예: pagination engine 의 vpos 보정) 에서 cascade 차이 야기. HWP3 정합 위해
    // paragraph 의 line_segs.vpos 에서 cumulative spacing_before 차감.
    if doc.is_hwp3_variant {
        normalize_variant_paragraph_vpos(&mut doc);
    }

    Ok(doc)
}

/// [Task #1042 Stage 5] HWP5 variant 의 paragraph data vpos 를 HWP3 형식으로 normalize.
///
/// HWP3 paragraph 사이 vpos diff = lh + ls (spacing_before 미포함)
/// HWP5 variant paragraph 사이 vpos diff = lh + ls + sb (spacing_before 포함)
///
/// HWP5 variant 의 line_segs.vpos 에서 cumulative spacing_before 차감하여 HWP3
/// 형식과 정합. paragraph_layout 의 spacing_before 적용 path 는 ParaShape 기반
/// 으로 처리되므로 vpos normalize 후에도 동일.
///
/// paragraph local reset detection: 현재 paragraph 의 first vpos 가 이전
/// paragraph 의 vpos 끝보다 작으면 reset 발생 (page boundary 등). cumulative_sb
/// reset.
fn normalize_variant_paragraph_vpos(doc: &mut crate::model::document::Document) {
    let para_shapes = doc.doc_info.para_shapes.clone();
    for section in doc.sections.iter_mut() {
        let mut cumulative_sb: i32 = 0;
        let mut prev_vpos_end: i32 = 0;
        for para in section.paragraphs.iter_mut() {
            if para.line_segs.is_empty() {
                continue;
            }
            let sb = para_shapes
                .get(para.para_shape_id as usize)
                .map(|p| p.spacing_before)
                .unwrap_or(0);
            let first_vpos = para.line_segs[0].vertical_pos;
            // paragraph local reset detection
            if first_vpos < prev_vpos_end.saturating_sub(cumulative_sb + sb) {
                cumulative_sb = 0;
            }
            cumulative_sb = cumulative_sb.saturating_add(sb);
            for ls in para.line_segs.iter_mut() {
                ls.vertical_pos = ls.vertical_pos.saturating_sub(cumulative_sb);
            }
            let last = para.line_segs.last().unwrap();
            prev_vpos_end = last.vertical_pos + last.line_height + last.line_spacing;
        }
    }
}

/// [Task #554] HWP3 → HWP5/HWPX 변환본 식별 휴리스틱 + 페이지 여백 보정
///
/// 한컴이 HWP3 → HWP5 로 변환할 때 한글97의 "마지막 줄 tolerance" (1600 HU)
/// 동작이 누락되어 페이지 수가 +1 ~ +4 증가한다. 변환본을 식별 후 모든
/// SectionDef.page_def.margin_bottom 을 1600 HU 줄여 한글97 페이지네이션과 정합.
///
/// ## 식별 휴리스틱 (Task #554 진단 결과)
///
/// 한컴은 HWP3 → HWP5 변환 시 ParaShape/CharShape 를 거의 재사용하지 않고 매우
/// 적은 수만 생성하여 paragraph 대비 비율이 극도로 낮다. 직접 작성본은 작성자가
/// 다양한 스타일을 사용하므로 비율이 paragraph 와 비슷하거나 더 높다.
///
/// - **`ParaShape/Paragraph < 0.05` AND `CharShape/Paragraph < 0.15`** → 변환본
/// - **`Paragraph > 50`** 가드: 매우 짧은 문서는 비율이 왜곡되므로 제외
///
/// 27 fixture 검증에서 100% 정확 분류 (Stage 1 보고서 §3.2 참조).
/// [Task #1001] 변환본의 line_segs 단위 보정.
/// vertical_pos 만 ParaShape spacing 누적 영향으로 변환본에서 2배 단위.
/// 나머지 필드 (line_height/text_height/baseline_distance/line_spacing/column_start/
/// segment_width) 는 단위 동일 (HWP3 와 같음) 이라 보정 불필요.
fn fixup_line_segs_for_variant(paragraphs: &mut [crate::model::paragraph::Paragraph]) {
    for para in paragraphs.iter_mut() {
        for ls in para.line_segs.iter_mut() {
            ls.vertical_pos /= 2;
        }
        // 표 셀 내부 paragraph 재귀
        for control in para.controls.iter_mut() {
            if let crate::model::control::Control::Table(table) = control {
                for cell in table.cells.iter_mut() {
                    fixup_line_segs_for_variant(&mut cell.paragraphs);
                }
            }
        }
    }
}

fn apply_hwp3_origin_fixup(doc: &mut Document) {
    // [#1880 v2] rhwp HWPX→HWP 변환본(is_hwpx_variant, #1886 마커)은 한컴
    // HWP3→HWP5 변환본이 아니다 — 결정론 마커가 비율 휴리스틱에 우선한다.
    // 미게이트 시 저-스타일 대형 문서(2959953)가 비율에 걸려 margin_bottom
    // -1600 이 오발동, HWPX 렌더와 페이지 기하가 21.3px 어긋나 PI_MOVED 유발
    // (HWPX 파스는 #1608 에서 동종 감지 제거됨).
    if doc.is_hwpx_variant {
        return;
    }
    let total_paragraphs: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
    if total_paragraphs <= 50 {
        return;
    }
    let ps_ratio = doc.doc_info.para_shapes.len() as f64 / total_paragraphs as f64;
    let cs_ratio = doc.doc_info.char_shapes.len() as f64 / total_paragraphs as f64;
    if ps_ratio < 0.05 && cs_ratio < 0.15 {
        // [Task #554] 변환본 의심 시 margin_bottom 보정 (한글97 의 마지막 줄
        // tolerance 모방). is_hwp3_variant 플래그 설정은 caller 가 별도 (HwpSummary
        // HWP3-era + 더 관대한 ratio AND 조건) 로 처리 — hwpspec.hwp 같은 spec 문서
        // false-positive 차단 위해 ratio 단독 변환본 확정 회피.
        for section in doc.sections.iter_mut() {
            section.section_def.page_def.margin_bottom = section
                .section_def
                .page_def
                .margin_bottom
                .saturating_sub(1600);
        }
    }
}

/// CfbReader로 섹션들 파싱
fn parse_sections_strict(
    cfb: &mut cfb_reader::CfbReader,
    section_count: u32,
    compressed: bool,
    distribution: bool,
) -> Result<Vec<crate::model::document::Section>, ParseError> {
    let mut sections = Vec::new();

    for i in 0..section_count {
        let section_data = if distribution {
            // 배포용 문서: ViewText 복호화
            let raw = cfb
                .read_body_text_section(i, compressed, true)
                .map_err(ParseError::CfbError)?;
            crypto::decrypt_viewtext_section(&raw, compressed).map_err(ParseError::CryptoError)?
        } else {
            cfb.read_body_text_section(i, compressed, false)
                .map_err(ParseError::CfbError)?
        };

        match body_text::parse_body_text_section(&section_data) {
            Ok(mut section) => {
                // 원본 BodyText 스트림 보존 (라운드트립용)
                section.raw_stream = Some(section_data);
                sections.push(section);
            }
            Err(e) => {
                // 개별 섹션 파싱 실패 시 빈 섹션으로 대체 (전체 실패 방지)
                eprintln!("경고: Section{} 파싱 실패: {}", i, e);
                sections.push(crate::model::document::Section::default());
            }
        }
    }

    Ok(sections)
}

/// LenientCfbReader로 파싱 (FAT 검증 무시)
fn parse_hwp_with_lenient(
    lenient: cfb_reader::LenientCfbReader,
    _raw_data: &[u8],
) -> Result<Document, ParseError> {
    // FileHeader 파싱
    let header_data = lenient.read_file_header().map_err(ParseError::CfbError)?;
    let file_header = header::parse_file_header(&header_data).map_err(ParseError::HeaderError)?;

    if file_header.flags.encrypted {
        return Err(ParseError::EncryptedDocument);
    }

    let compressed = file_header.flags.compressed;
    let distribution = file_header.flags.distribution;

    // DocInfo 파싱
    let doc_info_data = lenient
        .read_doc_info(compressed)
        .map_err(ParseError::CfbError)?;
    let (mut doc_info, doc_properties) =
        doc_info::parse_doc_info(&doc_info_data).map_err(ParseError::DocInfoError)?;
    doc_info.raw_stream = Some(doc_info_data);

    // BodyText 섹션별 파싱
    let section_count = lenient.section_count();
    let mut sections = Vec::new();

    for i in 0..section_count {
        let section_data = if distribution {
            let raw = lenient
                .read_body_text_section_full(i, compressed, true)
                .map_err(ParseError::CfbError)?;
            crypto::decrypt_viewtext_section(&raw, compressed).map_err(ParseError::CryptoError)?
        } else {
            lenient
                .read_body_text_section_full(i, compressed, false)
                .map_err(ParseError::CfbError)?
        };

        match body_text::parse_body_text_section(&section_data) {
            Ok(mut section) => {
                section.raw_stream = Some(section_data);
                sections.push(section);
            }
            Err(e) => {
                eprintln!("경고: Section{} 파싱 실패 (lenient): {}", i, e);
                sections.push(crate::model::document::Section::default());
            }
        }
    }

    // BinData 로드 시도
    let bin_data_content = load_bin_data_content_lenient(&lenient, &doc_info.bin_data_list);

    // Document 조립 (preview, extra_streams는 lenient에서 생략)
    let model_header = ModelFileHeader {
        version: ModelHwpVersion {
            major: file_header.version.major,
            minor: file_header.version.minor,
            build: file_header.version.build,
            revision: file_header.version.revision,
        },
        flags: file_header.flags.raw,
        compressed,
        encrypted: file_header.flags.encrypted,
        distribution,
        raw_data: Some(header_data),
    };

    let mut doc = Document {
        header: model_header,
        doc_properties,
        doc_info,
        sections,
        preview: None,
        bin_data_content,
        extra_streams: Vec::new(),
        is_hwpx_variant: false,
        hwpx_aux_entries: Vec::new(),
        is_hwp3_variant: false,
    };

    assign_auto_numbers(&mut doc);

    // [Task #554] HWP3 → HWP5 변환본 식별 + page_def margin_bottom 보정
    // [Task #1001] 변환본 식별 시 doc.is_hwp3_variant = true 설정
    apply_hwp3_origin_fixup(&mut doc);

    // [Task #873] BinData Link 타입 의 외부 file path 영역 Picture.external_path 전달.
    // 이후 model::document::populate_external_images_from_dir (Task #741) 가 같은
    // dir 영역 basename 매칭 영역 image 영역 자동 load.
    populate_link_image_paths(&mut doc);

    // [Task #1042 Stage 5] HWP5 variant 의 paragraph data raw vpos normalize —
    // HWP3 vs HWP5 variant 진단 결과 HWP5 의 raw vpos = HWP3 vpos + cumulative
    // spacing_before. paragraph 마다 +sb 누적 → paragraph_layout 의 외부 path
    // (예: pagination engine 의 vpos 보정) 에서 cascade 차이 야기. HWP3 정합 위해
    // paragraph 의 line_segs.vpos 에서 cumulative spacing_before 차감.
    if doc.is_hwp3_variant {
        normalize_variant_paragraph_vpos(&mut doc);
    }

    Ok(doc)
}

/// LenientCfbReader로 BinData 로드
fn load_bin_data_content_lenient(
    lenient: &cfb_reader::LenientCfbReader,
    bin_data_list: &[crate::model::bin_data::BinData],
) -> Vec<BinDataContent> {
    use crate::model::bin_data::BinDataType;

    let mut contents = Vec::new();

    for bd in bin_data_list.iter() {
        let is_storage = match bd.data_type {
            BinDataType::Embedding => false,
            BinDataType::Storage => true,
            BinDataType::Link => continue,
        };

        let ext = if is_storage {
            bd.extension.as_deref().unwrap_or("OLE")
        } else {
            bd.extension.as_deref().unwrap_or("dat")
        };
        let storage_name = format!("BIN{:04X}.{}", bd.storage_id, ext);

        match lenient.read_stream(&storage_name) {
            Ok(data) => {
                let mut decompressed = match cfb_reader::decompress_stream(&data) {
                    Ok(d) => d,
                    Err(_) => data,
                };

                // Task #195 단계 6: OLE Storage는 CFB 매직 바로 앞의 4-byte size prefix 스킵
                if is_storage && decompressed.len() > 8 {
                    let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
                    if decompressed[..8] != cfb_magic && decompressed[4..12] == cfb_magic {
                        decompressed.drain(..4);
                    }
                }

                contents.push(BinDataContent {
                    id: bd.storage_id,
                    data: decompressed,
                    extension: ext.to_string(),
                });
            }
            Err(e) => {
                eprintln!(
                    "경고: BinData '{}' 로드 실패 (lenient): {}",
                    storage_name, e
                );
            }
        }
    }

    contents
}

/// [Task #873] BinData Link 타입의 외부 file path 를 Picture.image_attr.external_path
/// 로 전달. 모든 포맷 (HWP5/HWPX) 공통 — HWP3 는 파서 내부에서 직접 설정 (Task #741).
///
/// HWP5 의 BinDataType::Link entry, HWPX 의 isEmbeded="0" item 이 abs_path/rel_path
/// 보유. 본 함수는 Picture.bin_data_id 로 BinData entry lookup → Link 인 경우
/// external_path 설정. 이후 populate_external_images_from_dir (model/document.rs) 가
/// HWP 파일 디렉토리에서 basename 매칭으로 실제 image 로드.
pub(crate) fn populate_link_image_paths(doc: &mut Document) {
    use crate::model::bin_data::BinDataType;
    use crate::model::control::Control;
    use crate::model::shape::ShapeObject;

    let bin_data = doc.doc_info.bin_data_list.clone();
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                let pic = match ctrl {
                    Control::Picture(p) => p,
                    Control::Shape(s) => match s.as_mut() {
                        ShapeObject::Picture(p) => p,
                        _ => continue,
                    },
                    _ => continue,
                };
                if pic.image_attr.external_path.is_some() {
                    continue;
                }
                let bin_idx = (pic.image_attr.bin_data_id as usize).saturating_sub(1);
                if let Some(bd) = bin_data.get(bin_idx) {
                    if matches!(bd.data_type, BinDataType::Link) {
                        let path = bd
                            .abs_path
                            .clone()
                            .filter(|p| !p.is_empty())
                            .or_else(|| bd.rel_path.clone().filter(|p| !p.is_empty()));
                        if let Some(p) = path {
                            pic.image_attr.external_path = Some(p);
                        }
                    }
                }
            }
        }
    }
}

/// 문서 내 모든 AutoNumber 컨트롤에 번호를 할당한다.
/// NewNumber 컨트롤을 만나면 해당 종류의 카운터를 리셋한다.
pub(crate) fn assign_auto_numbers(doc: &mut Document) {
    use crate::model::control::AutoNumberType;

    // 번호 종류별 카운터 — DocProperties 시작번호로 초기화
    let mut counters = [
        doc.doc_properties.page_start_num.saturating_sub(1),
        doc.doc_properties.footnote_start_num.saturating_sub(1),
        doc.doc_properties.endnote_start_num.saturating_sub(1),
        doc.doc_properties.picture_start_num.saturating_sub(1),
        doc.doc_properties.table_start_num.saturating_sub(1),
        doc.doc_properties.equation_start_num.saturating_sub(1),
    ];

    fn counter_index(t: AutoNumberType) -> usize {
        match t {
            AutoNumberType::Page => 0,
            AutoNumberType::Footnote => 1,
            AutoNumberType::Endnote => 2,
            AutoNumberType::Picture => 3,
            AutoNumberType::Table => 4,
            AutoNumberType::Equation => 5,
        }
    }

    // 모든 섹션, 문단, 컨트롤 순회
    for section in &mut doc.sections {
        // 구역별 시작번호 반영: 0이 아니면 해당 카운터를 리셋
        let sd = &section.section_def;
        if sd.picture_num > 0 {
            counters[3] = sd.picture_num.saturating_sub(1);
        }
        if sd.table_num > 0 {
            counters[4] = sd.table_num.saturating_sub(1);
        }
        if sd.equation_num > 0 {
            counters[5] = sd.equation_num.saturating_sub(1);
        }
        if sd.page_num > 0 {
            counters[0] = sd.page_num.saturating_sub(1);
        }

        // 본문 문단
        for para in &mut section.paragraphs {
            assign_auto_numbers_in_controls(&mut para.controls, &mut counters, counter_index);
        }
    }
}

fn assign_auto_numbers_in_controls(
    controls: &mut [crate::model::control::Control],
    counters: &mut [u16; 6],
    counter_index: fn(crate::model::control::AutoNumberType) -> usize,
) {
    use crate::model::control::Control;

    for ctrl in controls.iter_mut() {
        match ctrl {
            Control::AutoNumber(an) => {
                let idx = counter_index(an.number_type);
                counters[idx] += 1;
                an.assigned_number = counters[idx];
                an.number = counters[idx];
            }
            Control::Table(table) => {
                // 표 내부 셀의 문단도 처리
                for cell in &mut table.cells {
                    for para in &mut cell.paragraphs {
                        assign_auto_numbers_in_controls(
                            &mut para.controls,
                            counters,
                            counter_index,
                        );
                    }
                }
                // 표 캡션 처리
                if let Some(ref mut caption) = table.caption {
                    for para in &mut caption.paragraphs {
                        assign_auto_numbers_in_controls(
                            &mut para.controls,
                            counters,
                            counter_index,
                        );
                    }
                }
            }
            Control::Picture(pic) => {
                // 그림 캡션 처리
                if let Some(ref mut caption) = pic.caption {
                    for para in &mut caption.paragraphs {
                        assign_auto_numbers_in_controls(
                            &mut para.controls,
                            counters,
                            counter_index,
                        );
                    }
                }
            }
            Control::Shape(shape) => {
                // 묶음 개체(Group)의 캡션 처리
                if let crate::model::shape::ShapeObject::Group(ref mut group) = shape.as_mut() {
                    if let Some(ref mut caption) = group.caption {
                        for para in &mut caption.paragraphs {
                            assign_auto_numbers_in_controls(
                                &mut para.controls,
                                counters,
                                counter_index,
                            );
                        }
                    }
                }
                // 도형(글상자 등) 캡션 처리
                if let Some(ref mut drawing) = shape.drawing_mut() {
                    if let Some(ref mut caption) = drawing.caption {
                        for para in &mut caption.paragraphs {
                            assign_auto_numbers_in_controls(
                                &mut para.controls,
                                counters,
                                counter_index,
                            );
                        }
                    }
                    // 글상자 내부 문단의 자동 번호 처리
                    if let Some(ref mut text_box) = drawing.text_box {
                        for para in &mut text_box.paragraphs {
                            assign_auto_numbers_in_controls(
                                &mut para.controls,
                                counters,
                                counter_index,
                            );
                        }
                    }
                }
            }
            Control::Header(h) => {
                for para in &mut h.paragraphs {
                    assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
                }
            }
            Control::Footer(f) => {
                for para in &mut f.paragraphs {
                    assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
                }
            }
            Control::Footnote(fn_) => {
                for para in &mut fn_.paragraphs {
                    assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
                }
            }
            Control::Endnote(en) => {
                for para in &mut en.paragraphs {
                    assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
                }
            }
            Control::NewNumber(nn) => {
                let idx = counter_index(nn.number_type);
                counters[idx] = nn.number.saturating_sub(1);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Trait 추상화: DocumentParser
// ---------------------------------------------------------------------------

/// 문서 파서 trait — 바이트 데이터를 Document IR로 변환
pub trait DocumentParser {
    fn parse(&self, data: &[u8]) -> Result<Document, ParseError>;
}

/// HWP 5.0 바이너리 파서
pub struct HwpParser;

impl DocumentParser for HwpParser {
    fn parse(&self, data: &[u8]) -> Result<Document, ParseError> {
        parse_hwp(data)
    }
}

/// HWPX (XML/ZIP) 파서
pub struct HwpxParser;

impl DocumentParser for HwpxParser {
    fn parse(&self, data: &[u8]) -> Result<Document, ParseError> {
        hwpx::parse_hwpx(data).map_err(ParseError::from)
    }
}

/// HWP 3.0 파서
pub struct Hwp3Parser;

impl DocumentParser for Hwp3Parser {
    fn parse(&self, data: &[u8]) -> Result<Document, ParseError> {
        hwp3::parse_hwp3(data).map_err(ParseError::from)
    }
}

/// 포맷 자동 감지 후 적절한 파서로 파싱
pub fn parse_document(data: &[u8]) -> Result<Document, ParseError> {
    match detect_format(data) {
        FileFormat::Hwp => HwpParser.parse(data),
        FileFormat::Hwpx => HwpxParser.parse(data),
        FileFormat::Hwp3 => Hwp3Parser.parse(data),
        FileFormat::LegacyHwpml => Err(ParseError::UnsupportedFormat {
            code: UNSUPPORTED_HWPML_CODE,
            format: detect_legacy_hwpml_format_name(data),
            hint: UNSUPPORTED_HWPML_HINT,
        }),
        FileFormat::DrmProtected => Err(ParseError::UnsupportedFormat {
            code: DRM_PROTECTED_CODE,
            format: drm_format_name(data),
            hint: DRM_PROTECTED_HINT,
        }),
        FileFormat::Empty => Err(ParseError::UnsupportedFormat {
            code: EMPTY_FILE_CODE,
            format: "빈 파일",
            hint: EMPTY_FILE_HINT,
        }),
        FileFormat::Unknown => Err(ParseError::UnsupportedFormat {
            code: UNSUPPORTED_FILE_FORMAT_CODE,
            format: "알 수 없는 파일 형식",
            hint: SUPPORTED_FORMATS_HINT,
        }),
    }
}

/// DRM 벤더 시그니처로 사람이 읽을 이름을 고른다 (Issue #1982).
fn drm_format_name(data: &[u8]) -> &'static str {
    if data.starts_with(FASOO_DRM_SIG) {
        "DRM 보호 문서 (Fasoo)"
    } else if data.starts_with(SCDSA_SIG) {
        "DRM 보호 문서 (SoftCamp SCDSA)"
    } else {
        "DRM 보호 문서"
    }
}

/// 미리보기 데이터 추출 (PrvImage, PrvText)
fn extract_preview(cfb: &mut cfb_reader::CfbReader) -> Option<Preview> {
    let image_data = cfb.read_preview_image();
    let text = cfb.read_preview_text();

    // 둘 다 없으면 None 반환
    if image_data.is_none() && text.is_none() {
        return None;
    }

    let image = image_data.map(|data| {
        let format = detect_image_format(&data);
        PreviewImage { format, data }
    });

    Some(Preview { image, text })
}

/// HWP/HWPX 파일에서 썸네일 이미지만 경량 추출 (전체 파싱 없이)
///
/// - HWP (CFB): `/PrvImage` 스트림에서 추출
/// - HWPX (ZIP): `Preview/PrvImage.png` 엔트리에서 추출
pub fn extract_thumbnail_only(data: &[u8]) -> Option<ThumbnailResult> {
    let image_data = if detect_format(data) == FileFormat::Hwpx {
        // HWPX: ZIP 컨테이너에서 Preview/PrvImage.png 읽기
        extract_thumbnail_from_hwpx(data)?
    } else {
        // HWP: CFB 컨테이너에서 /PrvImage 스트림 읽기
        let mut cfb = cfb_reader::CfbReader::open(data).ok()?;
        cfb.read_preview_image()?
    };
    let format = detect_image_format(&image_data);

    // 이미지 크기 추출
    let (width, height) = match format {
        PreviewImageFormat::Png if image_data.len() >= 24 => {
            // PNG IHDR: offset 16 = width (u32 BE), offset 20 = height (u32 BE)
            let w = u32::from_be_bytes([
                image_data[16],
                image_data[17],
                image_data[18],
                image_data[19],
            ]);
            let h = u32::from_be_bytes([
                image_data[20],
                image_data[21],
                image_data[22],
                image_data[23],
            ]);
            (w, h)
        }
        PreviewImageFormat::Bmp if image_data.len() >= 26 => {
            // BMP 헤더: offset 18 = width (i32 LE), offset 22 = height (i32 LE)
            let w = i32::from_le_bytes([
                image_data[18],
                image_data[19],
                image_data[20],
                image_data[21],
            ]);
            let h = i32::from_le_bytes([
                image_data[22],
                image_data[23],
                image_data[24],
                image_data[25],
            ]);
            (w.unsigned_abs(), h.unsigned_abs())
        }
        PreviewImageFormat::Gif if image_data.len() >= 10 => {
            let w = u16::from_le_bytes([image_data[6], image_data[7]]) as u32;
            let h = u16::from_le_bytes([image_data[8], image_data[9]]) as u32;
            (w, h)
        }
        _ => (0, 0),
    };

    let output_format = match format {
        PreviewImageFormat::Png => "png",
        PreviewImageFormat::Bmp => "bmp",
        PreviewImageFormat::Gif => "gif",
        PreviewImageFormat::Unknown => "unknown",
    };

    Some(ThumbnailResult {
        format: output_format.to_string(),
        data: image_data,
        width,
        height,
    })
}

/// HWPX(ZIP)에서 Preview/PrvImage.png 추출
fn extract_thumbnail_from_hwpx(data: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;

    // Preview/PrvImage.png 또는 Preview/PrvImage.* 탐색
    let entry_name = (0..archive.len()).find_map(|i| {
        let file = archive.by_index(i).ok()?;
        let name = file.name().to_string();
        if name.starts_with("Preview/PrvImage") {
            Some(name)
        } else {
            None
        }
    })?;

    let mut file = archive.by_name(&entry_name).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;

    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

/// 썸네일 추출 결과
#[derive(Debug, Clone)]
pub struct ThumbnailResult {
    /// 출력 포맷 ("png", "gif", "unknown")
    pub format: String,
    /// 이미지 바이너리 데이터 (BMP는 PNG로 변환됨)
    pub data: Vec<u8>,
    /// 이미지 너비 (px)
    pub width: u32,
    /// 이미지 높이 (px)
    pub height: u32,
}

/// 이미지 포맷 감지 (BMP/GIF/PNG)
fn detect_image_format(data: &[u8]) -> PreviewImageFormat {
    if data.len() >= 8 && &data[..8] == b"\x89PNG\r\n\x1a\n" {
        PreviewImageFormat::Png
    } else if data.len() >= 2 && data[0] == 0x42 && data[1] == 0x4D {
        PreviewImageFormat::Bmp
    } else if data.len() >= 3 && &data[..3] == b"GIF" {
        PreviewImageFormat::Gif
    } else {
        PreviewImageFormat::Unknown
    }
}

/// 파서가 모델링하지 않는 CFB 스트림을 수집한다.
///
/// FileHeader, DocInfo, BodyText/Section*, BinData/*, PrvImage, PrvText는
/// 이미 별도로 파싱되므로 제외한다.
fn collect_extra_streams(
    cfb: &mut cfb_reader::CfbReader,
    bin_data_list: &[crate::model::bin_data::BinData],
    bin_data_content: &[BinDataContent],
) -> Vec<(String, Vec<u8>)> {
    let all_streams = cfb.list_streams();
    let mut extra = Vec::new();

    // [Task #1554] 직렬화기(`cfb_writer`)가 `bin_data_content` 로부터 재생성할
    // /BinData 스트림 경로 집합. 직렬화기와 동일한 명명 규칙(`find_bin_data_info_with_compress`)
    // 을 미러링하여 계산한다. 이 집합에 들어가지 않는 /BinData 스트림은 대응 BinData
    // 레코드가 없는 "고아 스트림"(예: img-start-001 의 20개 BIN, interview.hwp 의 BIN0001)
    // 이며, 그대로 두면 저장 시 통째 드롭된다. extra_streams 로 원본 바이트를 보존한다.
    let emitted_bin_paths: std::collections::HashSet<String> = bin_data_content
        .iter()
        .map(|c| {
            let (storage_id, ext) = serialized_bin_name(bin_data_list, c);
            format!("/BinData/BIN{:04X}.{}", storage_id, ext)
        })
        .collect();

    for path in &all_streams {
        // 이미 파싱된 스트림은 제외
        if path == "/FileHeader"
            || path == "/DocInfo"
            || path.starts_with("/BodyText/")
            || path.starts_with("/ViewText/")
            || path == "/PrvImage"
            || path == "/PrvText"
        {
            continue;
        }

        // /BinData 는 직렬화기가 재생성하는 스트림만 제외하고, 고아 스트림은 보존
        if path.starts_with("/BinData/") && emitted_bin_paths.contains(path) {
            continue;
        }

        // 나머지 스트림 보존
        if let Ok(data) = cfb.read_stream_raw(path) {
            extra.push((path.clone(), data));
        }
    }

    extra
}

/// 직렬화기가 `BinDataContent` 에 대해 생성할 스트림 이름의 (storage_id, ext) 계산.
///
/// `cfb_writer::find_bin_data_info_with_compress` 의 명명 규칙(매칭 레코드 우선,
/// 없으면 content 자체값)을 미러링한다. extra_streams 의 고아 /BinData 판별 전용.
fn serialized_bin_name<'a>(
    bin_data_list: &'a [crate::model::bin_data::BinData],
    content: &'a BinDataContent,
) -> (u16, &'a str) {
    use crate::model::bin_data::BinDataType;
    for bd in bin_data_list {
        if matches!(bd.data_type, BinDataType::Embedding | BinDataType::Storage)
            && bd.storage_id == content.id
        {
            return (bd.storage_id, bd.extension.as_deref().unwrap_or("dat"));
        }
    }
    (content.id, &content.extension)
}

/// BinData 스토리지에서 이미지 데이터 로드
///
/// bin_data_list의 각 항목에 대해 CFB 스토리지에서 바이너리 데이터를 읽어온다.
/// Embedding 타입인 경우에만 로드하며, 압축된 경우 해제한다.
fn load_bin_data_content(
    cfb: &mut cfb_reader::CfbReader,
    bin_data_list: &[crate::model::bin_data::BinData],
    _compressed: bool,
) -> Vec<BinDataContent> {
    use crate::model::bin_data::BinDataType;

    let mut contents = Vec::new();

    for bd in bin_data_list.iter() {
        // Embedding(이미지)과 Storage(OLE) 로드. Link는 외부 파일 참조이므로 제외
        let is_storage = match bd.data_type {
            BinDataType::Embedding => false,
            BinDataType::Storage => true,
            BinDataType::Link => continue,
        };

        // 스토리지 이름 생성: BIN0001.jpg (이미지) / BIN0001.OLE (OLE)
        // Storage 타입은 확장자 정보가 없을 수 있으므로 "OLE"로 기본 폴백
        let ext = if is_storage {
            bd.extension.as_deref().unwrap_or("OLE")
        } else {
            bd.extension.as_deref().unwrap_or("dat")
        };
        let storage_name = format!("BIN{:04X}.{}", bd.storage_id, ext);

        match cfb.read_bin_data(&storage_name) {
            Ok(data) => {
                // 압축된 BinData 해제 시도
                let mut decompressed = match cfb_reader::decompress_stream(&data) {
                    Ok(d) => d,
                    Err(_) => data, // 압축 해제 실패 시 원본 사용 (비압축 데이터)
                };

                // Task #195 단계 6: OLE Storage는 해제 후 선두 4바이트 size prefix를 스킵하여
                // 내부 CFB(`d0cf11e0...`) 시작 바이트부터 노출한다.
                if is_storage && decompressed.len() > 8 {
                    // CFB 매직이 바로 시작하면 prefix 없음
                    let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
                    if decompressed[..8] != cfb_magic && decompressed[4..12] == cfb_magic {
                        decompressed.drain(..4);
                    }
                }

                contents.push(BinDataContent {
                    id: bd.storage_id,
                    data: decompressed,
                    extension: ext.to_string(),
                });
            }
            Err(e) => {
                eprintln!("경고: BinData '{}' 로드 실패: {}", storage_name, e);
            }
        }
    }

    contents
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [#1880 v2] HWP3-origin 비율 휴리스틱 대상 문서(문단>50, 저-스타일 비율)
    /// 를 합성해, HWPX-변환본 마커(is_hwpx_variant) 유무에 따라 margin_bottom
    /// 보정(-1600)이 갈리는지 확인한다. 마커 있으면 보정 오발동 금지.
    fn hwp3_ratio_suspect_doc() -> Document {
        let mut doc = Document::default();
        doc.doc_info
            .para_shapes
            .push(crate::model::style::ParaShape::default()); // ps_ratio = 1/60
        doc.doc_info
            .char_shapes
            .push(crate::model::style::CharShape::default()); // cs_ratio = 1/60
        let mut section = crate::model::document::Section::default();
        section.section_def.page_def.margin_bottom = 4252;
        for _ in 0..60 {
            section
                .paragraphs
                .push(crate::model::paragraph::Paragraph::default());
        }
        doc.sections.push(section);
        doc
    }

    #[test]
    fn issue1880v2_hwp3_fixup_applies_to_native() {
        let mut doc = hwp3_ratio_suspect_doc();
        assert!(!doc.is_hwpx_variant);
        apply_hwp3_origin_fixup(&mut doc);
        assert_eq!(
            doc.sections[0].section_def.page_def.margin_bottom,
            4252 - 1600,
            "native HWP5 의심본은 종전대로 margin_bottom 보정"
        );
    }

    #[test]
    fn issue1880v2_hwp3_fixup_skipped_for_hwpx_variant() {
        let mut doc = hwp3_ratio_suspect_doc();
        doc.is_hwpx_variant = true;
        apply_hwp3_origin_fixup(&mut doc);
        assert_eq!(
            doc.sections[0].section_def.page_def.margin_bottom, 4252,
            "rhwp HWPX→HWP 변환본(마커)은 HWP3-origin 보정 오발동 금지 (#1880 v2, 2959953)"
        );
    }

    #[test]
    fn test_parse_hwp_too_small() {
        let result = parse_hwp(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hwp_invalid_cfb() {
        let result = parse_hwp(&[0u8; 512]);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_image_format_bmp() {
        let bmp_data = [0x42, 0x4D, 0x00, 0x00]; // BM header
        assert_eq!(detect_image_format(&bmp_data), PreviewImageFormat::Bmp);
    }

    #[test]
    fn test_detect_image_format_gif() {
        let gif_data = b"GIF89a";
        assert_eq!(detect_image_format(gif_data), PreviewImageFormat::Gif);
    }

    #[test]
    fn test_detect_image_format_unknown() {
        let unknown_data = [0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            detect_image_format(&unknown_data),
            PreviewImageFormat::Unknown
        );
    }

    #[test]
    fn test_detect_format_hwp() {
        let cfb_header = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        assert_eq!(detect_format(&cfb_header), FileFormat::Hwp);
    }

    #[test]
    fn test_detect_format_hwpx() {
        let zip_header = [0x50, 0x4B, 0x03, 0x04, 0x14, 0x00, 0x06, 0x00];
        assert_eq!(detect_format(&zip_header), FileFormat::Hwpx);
    }

    #[test]
    fn test_detect_format_unknown() {
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(detect_format(&data), FileFormat::Unknown);
    }

    #[test]
    fn test_detect_format_too_short() {
        assert_eq!(detect_format(&[0x50, 0x4B]), FileFormat::Unknown);
    }

    #[test]
    fn issue1982_detect_empty_file() {
        assert_eq!(detect_format(&[]), FileFormat::Empty);
        let err = parse_document(&[]).unwrap_err();
        assert!(
            matches!(&err, ParseError::UnsupportedFormat { code, .. } if *code == EMPTY_FILE_CODE),
            "empty file → EMPTY_FILE: {err}"
        );
    }

    #[test]
    fn issue1982_detect_drm_containers() {
        // Fasoo DRM
        let fasoo = b"\x9b DRMONE  This Document is encrypted and protected by Fasoo DRM";
        assert_eq!(detect_format(fasoo), FileFormat::DrmProtected);
        // SoftCamp SCDSA
        let scdsa = b"SCDSA002\x00\x00\xd0\x04";
        assert_eq!(detect_format(scdsa), FileFormat::DrmProtected);
        let err = parse_document(fasoo).unwrap_err();
        assert!(
            matches!(&err, ParseError::UnsupportedFormat { code, .. } if *code == DRM_PROTECTED_CODE),
            "DRM → DRM_PROTECTED: {err}"
        );
        assert_eq!(drm_format_name(fasoo), "DRM 보호 문서 (Fasoo)");
        assert_eq!(drm_format_name(scdsa), "DRM 보호 문서 (SoftCamp SCDSA)");
    }

    #[test]
    fn test_detect_format_hwp3() {
        // Issue #265: HWP 3.0 바이너리 시그니처
        let hwp3_header = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05\x00\x00";
        assert_eq!(detect_format(hwp3_header), FileFormat::Hwp3);
    }

    #[test]
    fn test_detect_format_hwp3_exact_17_bytes() {
        // 경계: 정확히 17바이트 "HWP Document File" 로 감지
        let exact = b"HWP Document File";
        assert_eq!(detect_format(exact), FileFormat::Hwp3);
    }

    #[test]
    fn test_detect_format_hwp3_too_short() {
        // 17바이트 미만이면 감지 불가 (Unknown)
        let short = b"HWP Document Fil"; // 16바이트
        assert_eq!(detect_format(short), FileFormat::Unknown);
    }

    #[test]
    fn test_detect_format_legacy_hwpml_21() {
        let hwpml = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.1"></HWPML>"#;
        assert_eq!(detect_format(hwpml), FileFormat::LegacyHwpml);
    }

    #[test]
    fn test_detect_format_legacy_hwpml_with_bom_and_space() {
        let hwpml = b"\xEF\xBB\xBF  \n<?xml version='1.0'?><hwpml version='2.1'></hwpml>";
        assert_eq!(detect_format(hwpml), FileFormat::LegacyHwpml);
    }

    #[test]
    fn test_parse_document_dispatches_hwp() {
        // CFB 시그니처 → HwpParser 경로로 디스패치
        let result = parse_document(&[0xD0, 0xCF, 0x11, 0xE0, 0x00, 0x00, 0x00, 0x00]);
        assert!(result.is_err()); // 유효하지 않은 CFB이므로 에러
    }

    #[test]
    fn test_parse_document_dispatches_hwpx() {
        // ZIP 시그니처 → HwpxParser 경로로 디스패치
        let result = parse_document(&[0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00]);
        assert!(result.is_err()); // 유효하지 않은 ZIP이므로 에러
    }

    #[test]
    fn test_parse_document_legacy_hwpml_returns_unsupported_code() {
        let hwpml = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.1"></HWPML>"#;
        let err = parse_document(hwpml).unwrap_err();
        match err {
            ParseError::UnsupportedFormat { code, format, hint } => {
                assert_eq!(code, "UNSUPPORTED_HWPML");
                assert_eq!(format, "HWPML 2.1");
                assert!(
                    hint.contains("HWP 5.0"),
                    "hint must explain support: {hint}"
                );
            }
            other => panic!("expected UnsupportedFormat, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_document_unknown_returns_unsupported_file_format() {
        let err = parse_document(b"not a document").unwrap_err();
        let msg = format!("{err}");
        match err {
            ParseError::UnsupportedFormat { code, format, .. } => {
                assert_eq!(code, "UNSUPPORTED_FILE_FORMAT");
                assert_eq!(format, "알 수 없는 파일 형식");
            }
            other => panic!("expected UnsupportedFormat, got {other:?}"),
        }
        assert!(msg.contains("UNSUPPORTED_FILE_FORMAT"));
        assert!(!msg.contains("CFB 오류"), "CFB detail leaked: {msg}");
    }

    #[test]
    fn test_parse_document_hwp3_too_short_errors() {
        // Issue #265 (updated): HWP 3.0 헤더 (now supported, but data is incomplete)
        let hwp3_header = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05";
        let err = parse_document(hwp3_header).unwrap_err();
        match err {
            ParseError::Hwp3Error(_) => {}
            other => panic!("expected Hwp3Error, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_document_issue_265_sample() {
        // Issue #265: 실제 제보 파일 samples/issue_265.hwp 가 HWP 3.0 으로
        // 감지되고 정상적으로 파싱되는지 확인.
        let data = std::fs::read("samples/issue_265.hwp")
            .expect("samples/issue_265.hwp should exist in repo");
        assert_eq!(detect_format(&data), FileFormat::Hwp3);
        let doc = parse_document(&data).expect("Should successfully parse HWP3 sample");
        assert!(
            !doc.sections.is_empty(),
            "Document should have at least one section"
        );
    }

    #[test]
    fn test_mock_parser() {
        struct MockParser;
        impl DocumentParser for MockParser {
            fn parse(&self, _data: &[u8]) -> Result<Document, ParseError> {
                Err(ParseError::EncryptedDocument)
            }
        }
        let result = MockParser.parse(&[]);
        assert!(result.is_err());
    }
}
