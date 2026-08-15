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
pub mod hml;
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
    /// Standalone HWPML XML
    Hml,
    /// DRM/보안 컨테이너로 보호된 문서 (미지원 — 감지만, Issue #1982)
    /// Fasoo(`\x9b DRMONE`) / SoftCamp SCDSA(`SCDSA00x`) 등. 복호화는 범위 밖.
    DrmProtected,
    /// 빈 파일(0 바이트) (Issue #1982)
    Empty,
    /// 알 수 없는 포맷
    Unknown,
}

const UNSUPPORTED_FILE_FORMAT_CODE: &str = "UNSUPPORTED_FILE_FORMAT";
const SUPPORTED_FORMATS_HINT: &str =
    "현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0, HWPML 2.9 문서를 지원합니다.";
const DRM_PROTECTED_CODE: &str = "DRM_PROTECTED";
const DRM_PROTECTED_HINT: &str =
    "DRM/보안 컨테이너로 보호된 문서입니다. 한컴오피스 등 DRM 클라이언트에서 보호를 해제한 뒤 저장해 열어주세요.";
const EMPTY_FILE_CODE: &str = "EMPTY_FILE";
const EMPTY_FILE_HINT: &str = "빈 파일(0 바이트)입니다.";

/// HWP5 완전 문서 열기가 선택하는 핵심 스트림 출력 예산.
///
/// 이 정책은 `parse_hwp*`와 자동 포맷 감지 진입점에서만 만들고, CFB/암호 계층에는
/// 남은 바이트 상한만 명시적으로 전달한다.
#[derive(Clone, Copy, Debug)]
struct Hwp5DocumentOpenDecompressionLimits {
    max_stream_input_bytes: usize,
    max_stream_output_bytes: usize,
    max_total_output_bytes: usize,
}

impl Hwp5DocumentOpenDecompressionLimits {
    const DEFAULT: Self = Self {
        // 암호문/압축 원본도 materialize 전에 제한한다. 압축 불가 데이터와 암호화
        // 블록 패딩을 위해 출력 상한보다 1 MiB 여유를 둔다.
        max_stream_input_bytes: 257 * 1024 * 1024,
        // 단일 DocInfo/BodyText 압축 해제 결과 상한. HWPX XML 엔트리와 HWP3 본문에
        // 적용된 256 MiB 문서 열기 계약과 맞춘다.
        max_stream_output_bytes: 256 * 1024 * 1024,
        // 여러 정상 섹션을 허용하되 단일 문서가 상한을 반복하지 못하게 하는 누적 상한.
        max_total_output_bytes: 512 * 1024 * 1024,
    };
}

/// 자동 포맷 감지를 포함한 완전 문서 열기의 자원 정책.
///
/// HWP3은 그 자체로 완전 문서 진입점인 `hwp3::parse_hwp3`에도 같은 기본 정책을
/// 적용하지만, 자동 감지 경로에서는 이 객체가 명시적으로 전달한다.
#[derive(Clone, Copy, Debug)]
struct DocumentOpenDecompressionPolicy {
    hwp5: Hwp5DocumentOpenDecompressionLimits,
    hwp3_max_body_output_bytes: usize,
}

impl DocumentOpenDecompressionPolicy {
    const DEFAULT: Self = Self {
        hwp5: Hwp5DocumentOpenDecompressionLimits::DEFAULT,
        hwp3_max_body_output_bytes: hwp3::DEFAULT_HWP3_DOCUMENT_OPEN_BODY_OUTPUT_BYTES,
    };
}

/// 비밀번호 보호 HWP5의 BinData를 parser가 materialize할 때 적용하는 별도 출력 상한.
///
/// BinData는 DocInfo/BodyText 문서 열기 누적 예산에는 포함하지 않지만, 해당 소비 경로가
/// crypto helper에 명시적으로 전달하는 parser 소유 정책이다.
const MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES: usize = 512 * 1024 * 1024;

#[cfg(test)]
thread_local! {
    /// E2E 회귀에서 공개 문서 열기 API를 작은 예산으로 실행하는 scoped test seam.
    /// 릴리스 빌드에는 존재하지 않는다.
    static TEST_DOCUMENT_OPEN_DECOMPRESSION_POLICY:
        std::cell::Cell<Option<DocumentOpenDecompressionPolicy>> = const { std::cell::Cell::new(None) };
}

fn document_open_decompression_policy() -> DocumentOpenDecompressionPolicy {
    #[cfg(test)]
    {
        TEST_DOCUMENT_OPEN_DECOMPRESSION_POLICY.with(|slot| {
            slot.get()
                .unwrap_or(DocumentOpenDecompressionPolicy::DEFAULT)
        })
    }

    #[cfg(not(test))]
    {
        DocumentOpenDecompressionPolicy::DEFAULT
    }
}

#[cfg(test)]
struct TestDocumentOpenDecompressionPolicyGuard(Option<DocumentOpenDecompressionPolicy>);

#[cfg(test)]
impl Drop for TestDocumentOpenDecompressionPolicyGuard {
    fn drop(&mut self) {
        TEST_DOCUMENT_OPEN_DECOMPRESSION_POLICY.with(|slot| slot.set(self.0));
    }
}

#[cfg(test)]
fn with_document_open_decompression_policy_for_test<T>(
    policy: DocumentOpenDecompressionPolicy,
    operation: impl FnOnce() -> T,
) -> T {
    let previous = TEST_DOCUMENT_OPEN_DECOMPRESSION_POLICY.with(|slot| slot.replace(Some(policy)));
    let _restore = TestDocumentOpenDecompressionPolicyGuard(previous);
    operation()
}

#[derive(Debug)]
struct Hwp5DocumentOpenDecompressionBudget {
    remaining: usize,
    max_stream_input_bytes: usize,
    max_stream_output_bytes: usize,
}

impl Hwp5DocumentOpenDecompressionBudget {
    fn new(limits: Hwp5DocumentOpenDecompressionLimits) -> Self {
        Self {
            remaining: limits.max_total_output_bytes,
            max_stream_input_bytes: limits.max_stream_input_bytes,
            max_stream_output_bytes: limits.max_stream_output_bytes,
        }
    }

    fn stream_limit(&self) -> usize {
        self.remaining.min(self.max_stream_output_bytes)
    }

    /// 암호문과 압축 원본을 `Vec`로 읽기 전에 적용하는 별도 상한이다.
    ///
    /// 압축 해제 출력 상한만으로는 원시 스트림이 과도하게 큰 입력을 막지 못하므로,
    /// strict/lenient CFB 경로 모두 이 값을 먼저 적용해야 한다.
    fn raw_stream_limit(&self) -> usize {
        self.max_stream_input_bytes
    }

    fn consume(&mut self, bytes: usize) -> Result<(), cfb_reader::CfbError> {
        if bytes > self.stream_limit() {
            return Err(cfb_reader::CfbError::LimitExceeded(self.stream_limit()));
        }
        self.remaining -= bytes;
        Ok(())
    }
}

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
    if hml::detect_hml_signature(data) {
        return FileFormat::Hml;
    }
    FileFormat::Unknown
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
    HmlError(hml::HmlError),
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
            ParseError::HmlError(e) => write!(f, "HML 오류: {}", e),
            ParseError::EncryptedDocument => write!(
                f,
                "비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)"
            ),
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
        match e {
            hwpx::HwpxError::Encrypted(_) => ParseError::EncryptedDocument,
            other => ParseError::HwpxError(other),
        }
    }
}

impl From<hwp3::Hwp3Error> for ParseError {
    fn from(e: hwp3::Hwp3Error) -> Self {
        match e {
            hwp3::Hwp3Error::PasswordRequired => ParseError::EncryptedDocument,
            other => ParseError::Hwp3Error(other),
        }
    }
}

impl From<hml::HmlError> for ParseError {
    fn from(error: hml::HmlError) -> Self {
        ParseError::HmlError(error)
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
    parse_hwp_inner(data, None)
}

/// 비밀번호로 보호된 HWP 파일을 비밀번호와 함께 파싱한다.
///
/// `encrypted` 플래그와 EncryptVersion 4가 설정된 문서를 연다. 비밀번호가 틀리면
/// `ParseError::CryptoError(CryptoError::WrongPassword)` 가 반환된다. 비밀번호가
/// 필요 없는 일반/배포용 문서에 비밀번호를 전달해도 결과는 동일하다(무시됨).
pub fn parse_hwp_with_password(data: &[u8], password: &[u8]) -> Result<Document, ParseError> {
    parse_hwp_inner(data, Some(password))
}

fn validate_password_encryption(file_header: &header::FileHeader) -> Result<(), ParseError> {
    if file_header.flags.encrypted
        && file_header.encrypt_version != crypto::SUPPORTED_PASSWORD_ENCRYPT_VERSION
    {
        return Err(ParseError::CryptoError(
            crypto::CryptoError::UnsupportedScheme {
                encrypt_version: file_header.encrypt_version,
            },
        ));
    }
    Ok(())
}

fn parse_doc_info_stream(
    data: &[u8],
    encrypted: bool,
) -> Result<
    (
        crate::model::document::DocInfo,
        crate::model::document::DocProperties,
    ),
    ParseError,
> {
    if encrypted {
        // 압축 문서는 deflate 오류로 오답 비밀번호를 검출할 수 있지만 비압축 문서는
        // 레코드 구조를 직접 확인해야 한다. DocInfo의 필수 선두 레코드 두 종류를
        // 인증 표식으로 사용해 무작위 복호화 결과가 빈/부분 문서로 통과하지 않게 한다.
        let records = record::Record::read_all(data)
            .map_err(|_| ParseError::CryptoError(crypto::CryptoError::WrongPassword))?;
        let has_required_prefix = records
            .first()
            .is_some_and(|record| record.tag_id == tags::HWPTAG_DOCUMENT_PROPERTIES)
            && records
                .iter()
                .any(|record| record.tag_id == tags::HWPTAG_ID_MAPPINGS);
        if !has_required_prefix {
            return Err(ParseError::CryptoError(crypto::CryptoError::WrongPassword));
        }
    }

    doc_info::parse_doc_info(data).map_err(|error| {
        if encrypted {
            ParseError::CryptoError(crypto::CryptoError::WrongPassword)
        } else {
            ParseError::DocInfoError(error)
        }
    })
}

fn parse_hwp_inner(data: &[u8], password: Option<&[u8]>) -> Result<Document, ParseError> {
    parse_hwp_inner_with_limits(data, password, document_open_decompression_policy().hwp5)
}

fn parse_hwp_inner_with_limits(
    data: &[u8],
    password: Option<&[u8]>,
    limits: Hwp5DocumentOpenDecompressionLimits,
) -> Result<Document, ParseError> {
    // 1. CFB 컨테이너 열기 (strict → lenient 폴백)
    match cfb_reader::CfbReader::open(data) {
        Ok(cfb) => parse_hwp_with_cfb(cfb, data, password, limits),
        Err(strict_err) => {
            eprintln!(
                "표준 CFB 파서 실패: {}, lenient 파서로 재시도...",
                strict_err
            );
            let lenient = cfb_reader::LenientCfbReader::open(data)
                .map_err(|_| ParseError::CfbError(strict_err))?;
            parse_hwp_with_lenient(lenient, data, password, limits)
        }
    }
}

/// 표준 CfbReader로 파싱
fn parse_hwp_with_cfb(
    mut cfb: cfb_reader::CfbReader,
    raw_data: &[u8],
    password: Option<&[u8]>,
    limits: Hwp5DocumentOpenDecompressionLimits,
) -> Result<Document, ParseError> {
    // 2. FileHeader 파싱
    let header_data = cfb.read_file_header().map_err(ParseError::CfbError)?;
    let file_header = header::parse_file_header(&header_data).map_err(ParseError::HeaderError)?;
    validate_password_encryption(&file_header)?;

    let encrypted = file_header.flags.encrypted;
    // 비밀번호 암호 문서인데 비밀번호가 없으면 열 수 없다.
    // 비밀번호가 제공되지 않은 경우 기존과 동일하게 EncryptedDocument 에러로
    // 호출자가 비밀번호 입력을 유도할 수 있게 한다.
    if encrypted && password.is_none() {
        return Err(ParseError::EncryptedDocument);
    }

    let compressed = file_header.flags.compressed;
    let distribution = file_header.flags.distribution;
    let mut decompression_budget = Hwp5DocumentOpenDecompressionBudget::new(limits);

    // 3. DocInfo 파싱 (비밀번호 암호 문서: raw 읽기 → 복호화)
    //
    // EncryptVersion 4(한글 7.0 이후)만 지원한다. 구버전 1~3을 같은 알고리즘으로
    // 해석하면 WrongPassword로 오진하므로 FileHeader 단계에서 명시적으로 거부한다.
    let doc_info_output_limit = decompression_budget.stream_limit();
    let doc_info_raw_limit = decompression_budget.raw_stream_limit();
    let doc_info_data = if encrypted {
        let raw = cfb
            .read_stream_raw_limited("/DocInfo", doc_info_raw_limit)
            .map_err(ParseError::CfbError)?;
        crypto::decrypt_password_protected_limited(
            &raw,
            password.unwrap(),
            compressed,
            doc_info_output_limit,
        )
        .map_err(ParseError::CryptoError)?
    } else {
        let raw = cfb
            .read_stream_raw_limited("/DocInfo", doc_info_raw_limit)
            .map_err(ParseError::CfbError)?;
        cfb_reader::decode_stream_limited(raw, compressed, doc_info_output_limit)
            .map_err(ParseError::CfbError)?
    };
    decompression_budget
        .consume(doc_info_data.len())
        .map_err(ParseError::CfbError)?;
    let (mut doc_info, doc_properties) = parse_doc_info_stream(&doc_info_data, encrypted)?;
    doc_info.raw_stream = Some(doc_info_data);

    // 4. BodyText 섹션별 파싱
    let section_count = cfb.section_count();
    let sections = parse_sections_strict(
        &mut cfb,
        section_count,
        compressed,
        distribution,
        encrypted,
        password,
        &mut decompression_budget,
    )?;

    // 5-7. 미리보기, BinData, 추가 스트림
    // Preview is optional document metadata.  A malformed or disproportionately
    // large thumbnail must not prevent the actual document from opening.
    let preview = extract_preview(&mut cfb, MAX_THUMBNAIL_BYTES);
    let bin_data_content = load_bin_data_content(
        &mut cfb,
        raw_data,
        &doc_info.bin_data_list,
        compressed,
        encrypted,
        password,
    );
    let extra_streams = collect_extra_streams(
        &mut cfb,
        &doc_info.bin_data_list,
        &bin_data_content,
        encrypted,
        password,
    );

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
        .any(|(p, _)| p == crate::model::document::HWPX_ORIGIN_STREAM_PATH);

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
        provenance: crate::model::provenance::SourceProvenance {
            format: crate::model::provenance::SourceFormat::Hwp5,
            hwp3_lineage: false,
            hwpx_lineage: is_hwpx_variant,
        },
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
                doc.provenance.hwp3_lineage = true;
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
            // 주변(saturating_sub/add)과 달리 여기만 raw 덧셈이라 손상 입력의 거대
            // vpos/height 로 i32 오버플로 패닉하던 것을 saturating 으로 맞춘다.
            prev_vpos_end = last
                .vertical_pos
                .saturating_add(last.line_height)
                .saturating_add(last.line_spacing);
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
    // [#3707] rhwp 가 만든 HWP3→HWP5 변환본에 쪽나눔 허용치를 되돌린다.
    //
    // HWP3 파서는 `pagination_bottom_tolerance = 1600 HU`(21.3px)를 세운다 — 한글97 의
    // 마지막 줄 tolerance 를 흉내 내 **페이지네이터에게만** 여유를 주는 렌더러 내부
    // 값이고 파일 포맷 필드가 아니다. HWP5 로 저장·재파싱하면 0 이 되어 본문 가용이
    // 21.3px 짧아진다.
    //
    // 그만큼 미주 단 가용이 줄어 단 전환이 일찍 걸리고, 2단 미주의 왼쪽 단이 조기에
    // 닫혀 미주가 다음 쪽으로 밀린다(SO-SUEOP 44쪽: 미주 128·129 가 45쪽으로). 한컴은
    // 원본·왕복본 모두 44쪽에 실으므로 허용치가 살아 있는 쪽이 정답지와 맞는다.
    //
    // 파일에 실리는 여백(`margin_bottom`)은 건드리지 않는다 — 줄이면 한컴이 보는 쪽
    // 기하가 원본과 달라지고 `convert --verify` 의 IR 비교에도 잡힌다. 이 값은 그
    // 비교에서 제외되는 렌더러 내부 값이라 왕복 정합을 깨지 않는다.
    if doc
        .extra_streams
        .iter()
        .any(|(p, _)| p == crate::model::document::HWP3_ORIGIN_STREAM_PATH)
    {
        // 마커의 존재 이유가 출처 복원이다 — lineage 를 함께 세워야
        // `native_hwp5_layout` 이 변환본을 원본 HWP5 로 오판하지 않는다.
        // (종전에는 휴리스틱(문단>50·shape 비율)만 lineage 를 세워, 마커가
        // 있는 소형 서식 변환본이 native 로 새어 저장-lineseg 전용 분기가
        // 발화했다 — issue_1892 자기일관 실측.)
        doc.provenance.hwp3_lineage = true;
        for section in doc.sections.iter_mut() {
            let pd = &mut section.section_def.page_def;
            if pd.pagination_bottom_tolerance == 0 {
                pd.pagination_bottom_tolerance = 1600.min(pd.margin_bottom);
            }
        }
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
#[allow(clippy::too_many_arguments)]
fn parse_sections_strict(
    cfb: &mut cfb_reader::CfbReader,
    section_count: u32,
    compressed: bool,
    distribution: bool,
    encrypted: bool,
    password: Option<&[u8]>,
    decompression_budget: &mut Hwp5DocumentOpenDecompressionBudget,
) -> Result<Vec<crate::model::document::Section>, ParseError> {
    let mut sections = Vec::new();

    for i in 0..section_count {
        let section_output_limit = decompression_budget.stream_limit();
        let section_raw_limit = decompression_budget.raw_stream_limit();
        let section_data = if distribution {
            // 배포용 문서: ViewText 복호화
            let raw = cfb
                .read_viewtext_section_raw_limited(i, section_raw_limit)
                .map_err(ParseError::CfbError)?;
            crypto::decrypt_viewtext_section_limited(&raw, compressed, section_output_limit)
                .map_err(ParseError::CryptoError)?
        } else if encrypted {
            // 비밀번호 암호 문서: BodyText raw → 비밀번호 복호화.
            // read_body_text_section_raw() 가 스트림 경로 탐색
            // (BodyText/Section{i} → /Section{i}) 을 담당하므로 raw 만 얻어
            // 복호화+압축해제는 crypto 로 위임한다.
            let raw = cfb
                .read_body_text_section_raw_limited(i, section_raw_limit)
                .map_err(ParseError::CfbError)?;
            crypto::decrypt_password_protected_limited(
                &raw,
                password.unwrap(),
                compressed,
                section_output_limit,
            )
            .map_err(ParseError::CryptoError)?
        } else {
            let raw = cfb
                .read_body_text_section_raw_limited(i, section_raw_limit)
                .map_err(ParseError::CfbError)?;
            cfb_reader::decode_stream_limited(raw, compressed, section_output_limit)
                .map_err(ParseError::CfbError)?
        };
        decompression_budget
            .consume(section_data.len())
            .map_err(ParseError::CfbError)?;

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
    password: Option<&[u8]>,
    limits: Hwp5DocumentOpenDecompressionLimits,
) -> Result<Document, ParseError> {
    // FileHeader 파싱
    let header_data = lenient.read_file_header().map_err(ParseError::CfbError)?;
    let file_header = header::parse_file_header(&header_data).map_err(ParseError::HeaderError)?;
    validate_password_encryption(&file_header)?;

    let encrypted = file_header.flags.encrypted;
    if encrypted && password.is_none() {
        return Err(ParseError::EncryptedDocument);
    }

    let compressed = file_header.flags.compressed;
    let distribution = file_header.flags.distribution;
    let mut decompression_budget = Hwp5DocumentOpenDecompressionBudget::new(limits);

    // DocInfo 파싱 (비밀번호 암호 문서: 제한된 raw 읽기 → 복호화)
    let doc_info_output_limit = decompression_budget.stream_limit();
    let doc_info_raw_limit = decompression_budget.raw_stream_limit();
    let doc_info_data = if encrypted {
        let raw = lenient
            .read_stream_raw_limited("DocInfo", doc_info_raw_limit)
            .map_err(ParseError::CfbError)?;
        crypto::decrypt_password_protected_limited(
            &raw,
            password.unwrap(),
            compressed,
            doc_info_output_limit,
        )
        .map_err(ParseError::CryptoError)?
    } else {
        let raw = lenient
            .read_stream_raw_limited("DocInfo", doc_info_raw_limit)
            .map_err(ParseError::CfbError)?;
        cfb_reader::decode_stream_limited(raw, compressed, doc_info_output_limit)
            .map_err(ParseError::CfbError)?
    };
    decompression_budget
        .consume(doc_info_data.len())
        .map_err(ParseError::CfbError)?;
    let (mut doc_info, doc_properties) = parse_doc_info_stream(&doc_info_data, encrypted)?;
    doc_info.raw_stream = Some(doc_info_data);

    // BodyText 섹션별 파싱
    let section_count = lenient.section_count();
    let mut sections = Vec::new();

    for i in 0..section_count {
        let section_output_limit = decompression_budget.stream_limit();
        let section_raw_limit = decompression_budget.raw_stream_limit();
        let section_data = if distribution {
            let raw = lenient
                .read_viewtext_section_raw_limited(i, section_raw_limit)
                .map_err(ParseError::CfbError)?;
            crypto::decrypt_viewtext_section_limited(&raw, compressed, section_output_limit)
                .map_err(ParseError::CryptoError)?
        } else if encrypted {
            // 비밀번호 암호 문서: lenient reader 로 raw 섹션 바이트를 얻어 복호화.
            let raw = lenient
                .read_stream_raw_limited(&format!("Section{}", i), section_raw_limit)
                .map_err(ParseError::CfbError)?;
            crypto::decrypt_password_protected_limited(
                &raw,
                password.unwrap(),
                compressed,
                section_output_limit,
            )
            .map_err(ParseError::CryptoError)?
        } else {
            let raw = lenient
                .read_stream_raw_limited(&format!("Section{}", i), section_raw_limit)
                .map_err(ParseError::CfbError)?;
            cfb_reader::decode_stream_limited(raw, compressed, section_output_limit)
                .map_err(ParseError::CfbError)?
        };
        decompression_budget
            .consume(section_data.len())
            .map_err(ParseError::CfbError)?;

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
    let bin_data_content = load_bin_data_content_lenient(
        &lenient,
        &doc_info.bin_data_list,
        encrypted,
        compressed,
        password,
    );

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
        provenance: crate::model::provenance::SourceProvenance {
            format: crate::model::provenance::SourceFormat::Hwp5,
            hwp3_lineage: false,
            hwpx_lineage: false,
        },
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

fn bin_data_should_compress(
    compression: crate::model::bin_data::BinDataCompression,
    document_compressed: bool,
) -> bool {
    use crate::model::bin_data::BinDataCompression;
    match compression {
        BinDataCompression::Default => document_compressed,
        BinDataCompression::Compress => true,
        BinDataCompression::NoCompress => false,
    }
}

/// BinData 스트림의 실제 압축 여부를 판정한다.
///
/// 일반 HWP는 `HWPTAG_BIN_DATA`의 개별 압축 속성을 따른다. 반면 한컴이
/// EncryptVersion 4로 저장한 암호 문서는 `NoCompress` BinData도 문서 전역
/// `compressed` 플래그에 따라 압축한 뒤 암호화한다. 따라서 암호 문서에서 개별
/// 속성을 따르면 복호화까지만 된 raw-deflate 바이트를 이미지로 오인하게 된다.
fn bin_data_stream_is_compressed(
    compression: crate::model::bin_data::BinDataCompression,
    document_compressed: bool,
    encrypted: bool,
) -> bool {
    if encrypted {
        document_compressed
    } else {
        bin_data_should_compress(compression, document_compressed)
    }
}

fn decode_encrypted_stream_limited(
    raw: &[u8],
    password: &[u8],
    compressed: bool,
    max_bytes: usize,
) -> Option<Vec<u8>> {
    crypto::decrypt_password_protected_limited(raw, password, compressed, max_bytes).ok()
}

/// LenientCfbReader로 BinData 로드
fn load_bin_data_content_lenient(
    lenient: &cfb_reader::LenientCfbReader,
    bin_data_list: &[crate::model::bin_data::BinData],
    encrypted: bool,
    compressed: bool,
    password: Option<&[u8]>,
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
        let stream_compressed =
            bin_data_stream_is_compressed(bd.compression, compressed, encrypted);

        match lenient.read_stream(&storage_name) {
            Ok(data) => {
                let mut decompressed = if encrypted {
                    let pwd = password.unwrap();
                    match crypto::decrypt_password_protected_limited(
                        &data,
                        pwd,
                        stream_compressed,
                        MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES,
                    ) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!(
                                "경고: BinData '{}' 복호화 실패 (lenient): {}",
                                storage_name, e
                            );
                            continue;
                        }
                    }
                } else {
                    match cfb_reader::decompress_stream(&data) {
                        Ok(d) => d,
                        Err(_) => data,
                    }
                };

                // Task #195 단계 6: OLE Storage는 CFB 매직 바로 앞의 4-byte size prefix 스킵
                if is_storage && decompressed.len() >= 12 {
                    let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
                    if decompressed[..8] != cfb_magic && decompressed[4..12] == cfb_magic {
                        decompressed.drain(..4);
                    }
                }

                contents.push(BinDataContent {
                    id: bd.storage_id,
                    data: decompressed.into(),
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

    fn counter_index(t: AutoNumberType) -> Option<usize> {
        match t {
            AutoNumberType::Page => Some(0),
            AutoNumberType::Footnote => Some(1),
            AutoNumberType::Endnote => Some(2),
            AutoNumberType::Picture => Some(3),
            AutoNumberType::Table => Some(4),
            AutoNumberType::Equation => Some(5),
            // 총 쪽수는 페이지네이션이 끝난 뒤 결정되는 표시값이다. 일반 자동번호
            // 카운터를 증가시키거나 NewNumber로 재설정할 대상이 아니다.
            AutoNumberType::TotalPage => None,
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
    counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
) {
    use crate::model::control::Control;

    fn assign_caption_auto_numbers(
        caption: &mut Option<crate::model::shape::Caption>,
        counters: &mut [u16; 6],
        counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
    ) {
        if let Some(caption) = caption {
            for para in &mut caption.paragraphs {
                assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
            }
        }
    }

    fn assign_text_box_auto_numbers(
        text_box: &mut Option<crate::model::shape::TextBox>,
        counters: &mut [u16; 6],
        counter_index: fn(crate::model::control::AutoNumberType) -> Option<usize>,
    ) {
        if let Some(text_box) = text_box {
            for para in &mut text_box.paragraphs {
                assign_auto_numbers_in_controls(&mut para.controls, counters, counter_index);
            }
        }
    }

    for ctrl in controls.iter_mut() {
        match ctrl {
            Control::AutoNumber(an) => {
                if let Some(idx) = counter_index(an.number_type) {
                    counters[idx] += 1;
                    an.assigned_number = counters[idx];
                    an.number = counters[idx];
                }
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
                assign_caption_auto_numbers(&mut table.caption, counters, counter_index);
            }
            Control::Picture(pic) => {
                // 그림 캡션 처리
                assign_caption_auto_numbers(&mut pic.caption, counters, counter_index);
            }
            Control::Shape(shape) => {
                use crate::model::shape::ShapeObject;

                match shape.as_mut() {
                    ShapeObject::Line(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Rectangle(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Ellipse(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Arc(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Polygon(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Curve(s) => {
                        assign_caption_auto_numbers(
                            &mut s.drawing.caption,
                            counters,
                            counter_index,
                        );
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Group(s) => {
                        assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                    }
                    ShapeObject::Picture(s) => {
                        assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                    }
                    ShapeObject::Chart(s) => {
                        if s.caption.is_some() {
                            assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                        } else {
                            assign_caption_auto_numbers(
                                &mut s.drawing.caption,
                                counters,
                                counter_index,
                            );
                        }
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
                    }
                    ShapeObject::Ole(s) => {
                        if s.caption.is_some() {
                            assign_caption_auto_numbers(&mut s.caption, counters, counter_index);
                        } else {
                            assign_caption_auto_numbers(
                                &mut s.drawing.caption,
                                counters,
                                counter_index,
                            );
                        }
                        assign_text_box_auto_numbers(
                            &mut s.drawing.text_box,
                            counters,
                            counter_index,
                        );
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
                if let Some(idx) = counter_index(nn.number_type) {
                    counters[idx] = nn.number.saturating_sub(1);
                }
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

/// Standalone HWPML XML parser.
pub struct HmlParser;

impl DocumentParser for HmlParser {
    fn parse(&self, data: &[u8]) -> Result<Document, ParseError> {
        hml::parse_hml(data)
            .map(|result| result.document)
            .map_err(ParseError::from)
    }
}

/// HML 입력에서만 제공되는 열기 메타데이터와 손실 진단.
pub struct HmlImportMetadata {
    pub hwpml_version: Option<String>,
    pub sub_version: Option<String>,
    pub style: Option<String>,
    pub encoding: hml::HmlEncoding,
    pub resource_count: usize,
    pub warnings: Vec<hml::HmlWarning>,
    pub preserved_fragments: Vec<hml::PreservedFragment>,
}

/// 공통 IR과 입력 포맷 전용 열기 메타데이터를 분리해 전달한다.
pub struct ParsedDocument {
    pub document: Document,
    pub hml_metadata: Option<HmlImportMetadata>,
}

/// 포맷 자동 감지 후 공통 IR과 입력 메타데이터를 파싱한다.
pub fn parse_document_with_metadata(data: &[u8]) -> Result<ParsedDocument, ParseError> {
    parse_document_inner(data, None)
}

/// 포맷 자동 감지 후 비밀번호와 함께 파싱한다.
///
/// HWP5 EncryptVersion 4, 압축 HWP3, ODF AES-256-CBC HWPX 비밀번호 암호 문서를 연다.
/// 비밀번호가 틀리면 암호 불일치/손상 오류를 반환한다. 암호화되지 않은 HWPX는 기존
/// 파서 결과를 그대로 반환한다.
pub fn parse_document_with_metadata_password(
    data: &[u8],
    password: &[u8],
) -> Result<ParsedDocument, ParseError> {
    parse_document_inner(data, Some(password))
}

fn parse_document_inner(
    data: &[u8],
    password: Option<&[u8]>,
) -> Result<ParsedDocument, ParseError> {
    let open_policy = document_open_decompression_policy();
    match detect_format(data) {
        FileFormat::Hwp => {
            parse_hwp_inner_with_limits(data, password, open_policy.hwp5).map(without_hml_metadata)
        }
        FileFormat::Hwpx => match password {
            Some(password) => hwpx::parse_hwpx_with_password(data, password),
            None => hwpx::parse_hwpx(data),
        }
        .map_err(ParseError::from)
        .map(without_hml_metadata),
        FileFormat::Hwp3 => match password {
            Some(password) => hwp3::parse_hwp3_with_open_body_limit_and_password(
                data,
                password,
                open_policy.hwp3_max_body_output_bytes,
            ),
            None => {
                hwp3::parse_hwp3_with_open_body_limit(data, open_policy.hwp3_max_body_output_bytes)
            }
        }
        .map_err(ParseError::from)
        .map(without_hml_metadata),
        FileFormat::Hml => {
            let result = hml::parse_hml(data).map_err(ParseError::from)?;
            Ok(ParsedDocument {
                document: result.document,
                hml_metadata: Some(HmlImportMetadata {
                    hwpml_version: result.metadata.hwpml_version,
                    sub_version: result.metadata.sub_version,
                    style: result.metadata.style,
                    encoding: result.metadata.encoding,
                    resource_count: result.metadata.resource_count,
                    warnings: result.warnings,
                    preserved_fragments: result.preserved_fragments,
                }),
            })
        }
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

fn without_hml_metadata(document: Document) -> ParsedDocument {
    ParsedDocument {
        document,
        hml_metadata: None,
    }
}

/// 포맷 자동 감지 후 공통 IR만 반환하는 호환 진입점.
pub fn parse_document(data: &[u8]) -> Result<Document, ParseError> {
    parse_document_with_metadata(data).map(|parsed| parsed.document)
}

/// 포맷 자동 감지 후 비밀번호와 함께 공통 IR만 반환한다.
///
/// 비암호 문서와 다른 지원 포맷에서는 비밀번호를 무시한다. 암호화된 HWP5의
/// EncryptVersion이 4가 아니면 `UnsupportedScheme`을 반환하며, HWP3는 압축
/// 암호 본문, HWPX는 ODF AES-256-CBC/PBKDF2 패키지를 지원한다.
pub fn parse_document_with_password(data: &[u8], password: &[u8]) -> Result<Document, ParseError> {
    parse_document_with_metadata_password(data, password).map(|parsed| parsed.document)
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
fn extract_preview(cfb: &mut cfb_reader::CfbReader, max_thumbnail_bytes: usize) -> Option<Preview> {
    let image_data = cfb.read_preview_image_limited(max_thumbnail_bytes);
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
        extract_thumbnail_from_hwpx(data, MAX_THUMBNAIL_BYTES)?
    } else {
        // HWP: CFB 컨테이너에서 /PrvImage 스트림 읽기
        let mut cfb = cfb_reader::CfbReader::open(data).ok()?;
        cfb.read_preview_image_limited(MAX_THUMBNAIL_BYTES)?
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

/// 미리보기 이미지 하나의 최대 압축 해제 크기.
///
/// HWP/HWPX 문서 열기와 브라우저 썸네일 소비자가 선택하는 10 MiB 상한이다.
pub const MAX_THUMBNAIL_BYTES: usize = 10 * 1024 * 1024;

fn read_thumbnail_limited<R: std::io::Read>(
    reader: &mut R,
    expected_bytes: usize,
    max_bytes: usize,
) -> Option<Vec<u8>> {
    if expected_bytes == 0 || expected_bytes > max_bytes {
        return None;
    }

    let mut buf = Vec::new();
    let mut limited = std::io::Read::take(&mut *reader, (expected_bytes as u64).saturating_add(1));
    std::io::Read::read_to_end(&mut limited, &mut buf).ok()?;
    (buf.len() == expected_bytes).then_some(buf)
}

/// HWPX(ZIP)에서 Preview/PrvImage.png 추출
fn extract_thumbnail_from_hwpx(data: &[u8], max_bytes: usize) -> Option<Vec<u8>> {
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
    let declared_size = usize::try_from(file.size()).ok()?;
    read_thumbnail_limited(&mut file, declared_size, max_bytes)
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
    encrypted: bool,
    password: Option<&[u8]>,
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

        // 비밀번호 암호화 대상 중 모델링하지 않는 스트림은 평문 raw 상태로 보존한다.
        // 직렬화기는 암호화 플래그를 제거하므로 암호문을 그대로 보존하면 Scripts와
        // 고아 BinData가 읽을 수 없는 상태로 저장된다.
        if let Ok(mut data) = cfb.read_stream_raw(path) {
            if encrypted && (path.starts_with("/Scripts/") || path.starts_with("/BinData/")) {
                if let Some(password) = password {
                    data = crypto::decrypt_password_stream(&data, password);
                }
            }
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
/// [Task #2263] HWP5 CFB 원본을 보유하고 요청 시점에 BinData 스트림을 압축 해제한다.
///
/// 파싱 시점에 모든 내장 이미지를 풀어 IR 에 상주시키면 원본 파일 크기의
/// 수십 배 메모리를 쓰게 된다. CFB 안의 BinData 스트림은 zlib 압축 상태이므로,
/// 원본 컨테이너만 들고 있다가 실제로 렌더·직렬화되는 항목만 그때 푼다.
struct Hwp5BinResolver {
    cfb: std::sync::Mutex<cfb_reader::CfbReader>,
    /// 선두 4-byte size prefix 정규화가 필요한 OLE Storage 스트림명
    ole_streams: std::collections::HashSet<String>,
    /// 스트림별 압축 여부. HWPTAG_BIN_DATA 속성이 문서 전역 플래그보다 우선한다.
    compressed_streams: std::collections::HashMap<String, bool>,
    /// 비밀번호 암호 문서 여부 (FileHeader encrypted 플래그)
    encrypted: bool,
    /// 비밀번호 바이트. 지연 로딩이 파싱 이후 렌더 시점에 스트림을 읽으므로
    /// 리졸버가 바이트를 소유해야 한다.
    password: Option<Vec<u8>>,
}

impl std::fmt::Debug for Hwp5BinResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hwp5BinResolver")
            .field("ole_streams", &self.ole_streams.len())
            .field("compressed_streams", &self.compressed_streams.len())
            .field("encrypted", &self.encrypted)
            .finish()
    }
}

impl Hwp5BinResolver {
    /// raw BinData 바이트를 복호화(암호 문서)+압축 해제한다.
    /// 비암호 문서에서 압축 해제 실패 시 원본을 그대로 반환한다(기존 동작).
    fn try_decode(&self, key: &str, raw: &[u8]) -> Option<Vec<u8>> {
        if self.encrypted {
            let pwd = self.password.as_deref().unwrap_or(&[]);
            let compressed = self.compressed_streams.get(key).copied().unwrap_or(false);
            crypto::decrypt_password_protected_limited(
                raw,
                pwd,
                compressed,
                MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES,
            )
            .ok()
        } else {
            match cfb_reader::decompress_stream(raw) {
                Ok(d) => Some(d),
                Err(_) => Some(raw.to_vec()),
            }
        }
    }
}

impl crate::model::bin_data::BinDataResolver for Hwp5BinResolver {
    fn resolve(&self, key: &str) -> Vec<u8> {
        let mut cfb = match self.cfb.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };
        let raw = match cfb.read_bin_data(key) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("경고: BinData '{}' 로드 실패: {}", key, e);
                return Vec::new();
            }
        };

        let mut decompressed = match self.try_decode(key, &raw) {
            Some(d) => d,
            None => {
                eprintln!(
                    "경고: BinData '{}' 복호화 실패 (비밀번호 불일치 또는 손상)",
                    key
                );
                return Vec::new();
            }
        };

        // Task #195 단계 6: OLE Storage는 해제 후 선두 4바이트 size prefix를 스킵하여
        // 내부 CFB(`d0cf11e0...`) 시작 바이트부터 노출한다.
        if self.ole_streams.contains(key) && decompressed.len() >= 12 {
            let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
            if decompressed[..8] != cfb_magic && decompressed[4..12] == cfb_magic {
                decompressed.drain(..4);
            }
        }

        decompressed
    }

    fn resolve_limited(&self, key: &str, max_bytes: usize) -> Option<Vec<u8>> {
        let mut cfb = match self.cfb.lock() {
            Ok(cfb) => cfb,
            Err(poisoned) => poisoned.into_inner(),
        };
        // 암호 문서는 블록 단위 복호화가 전체 스트림을 요구하므로 제한 없이 읽는다.
        let raw = if self.encrypted {
            match cfb.read_bin_data(key) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("경고: BinData '{}' bounded 로드 실패: {}", key, error);
                    return None;
                }
            }
        } else {
            match cfb.read_bin_data_limited(key, max_bytes) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("경고: BinData '{}' bounded 로드 실패: {}", key, error);
                    return None;
                }
            }
        };

        let mut bytes = if self.encrypted {
            let password = self.password.as_deref().unwrap_or(&[]);
            let compressed = self.compressed_streams.get(key).copied().unwrap_or(false);
            decode_encrypted_stream_limited(&raw, password, compressed, max_bytes)?
        } else {
            match cfb_reader::decompress_stream_limited(&raw, max_bytes) {
                Ok(data) => data,
                Err(cfb_reader::CfbError::LimitExceeded(_)) => return None,
                Err(_) => raw,
            }
        };
        if self.ole_streams.contains(key) && bytes.len() >= 12 {
            let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
            if bytes[..8] != cfb_magic && bytes[4..12] == cfb_magic {
                bytes.drain(..4);
            }
        }
        (bytes.len() <= max_bytes).then_some(bytes)
    }

    /// [#2550] 저장된 형태 그대로의 바이트 — 압축 해제 없이 반환한다.
    ///
    /// 복호화는 크기 1:1 변환이라 deflate bomb 위험이 없으므로 수행한다. 반환
    /// 바이트는 원본 스트림과 같은 압축 상태이며, OLE size prefix 도 (압축 안에)
    /// 그대로 들어 있다 — 저장 경로가 이 바이트를 그대로 기록하면 왕복 무손실이다.
    fn resolve_raw(&self, key: &str) -> Option<crate::model::bin_data::StoredBinData> {
        let raw = {
            let mut cfb = match self.cfb.lock() {
                Ok(cfb) => cfb,
                Err(poisoned) => poisoned.into_inner(),
            };
            match cfb.read_bin_data(key) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("경고: BinData '{}' 원본 로드 실패: {}", key, error);
                    return None;
                }
            }
        };
        let bytes = if self.encrypted {
            let password = self.password.as_deref().unwrap_or(&[]);
            // `decrypt_hwp5_stream` 은 입력 길이로 truncate 하므로 크기가 보존된다.
            crypto::decrypt_password_stream(&raw, password)
        } else {
            raw
        };
        Some(crate::model::bin_data::StoredBinData {
            compressed: self.compressed_streams.get(key).copied().unwrap_or(false),
            bytes,
        })
    }

    /// [#2550] `resolve().len()` 의 bounded 미러 — 출력을 materialize 하지 않는다.
    ///
    /// 상한(비암호 [`MAX_BIN_DATA_BYTES`](crate::model::bin_data::MAX_BIN_DATA_BYTES),
    /// 암호 `MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES`) 초과 항목은 bounded 로드 경로가
    /// placeholder 로 접으므로 길이도 0 으로 보고한다.
    fn resolved_len(&self, key: &str) -> usize {
        let raw = {
            let mut cfb = match self.cfb.lock() {
                Ok(cfb) => cfb,
                Err(poisoned) => poisoned.into_inner(),
            };
            match cfb.read_bin_data(key) {
                Ok(data) => data,
                Err(_) => return 0,
            }
        };

        let stream_compressed = self.compressed_streams.get(key).copied().unwrap_or(false);
        let (plain, len) = if self.encrypted {
            // 복호화(1:1) 후 압축 여부에 따라 길이만 센다. resolve() 는 오류·상한
            // 초과 시 빈 값을 반환하므로 길이도 0 이다.
            let password = self.password.as_deref().unwrap_or(&[]);
            let plain = crypto::decrypt_password_stream(&raw, password);
            let len = if stream_compressed {
                match cfb_reader::decompressed_len_capped(
                    &plain,
                    MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES,
                ) {
                    Ok(len) => len,
                    Err(_) => return 0,
                }
            } else if plain.len() > MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES {
                return 0;
            } else {
                plain.len()
            };
            (plain, len)
        } else {
            let len = match cfb_reader::decompressed_len_capped(
                &raw,
                crate::model::bin_data::MAX_BIN_DATA_BYTES,
            ) {
                Ok(len) => len,
                Err(cfb_reader::CfbError::LimitExceeded(_)) => return 0,
                // 해제 실패 시 원본 바이트를 그대로 노출하는 resolve() 를 미러링.
                Err(_) => raw.len(),
            };
            (raw, len)
        };

        // resolve() 의 OLE size prefix strip 을 선두 12바이트만 해제해 미러링한다.
        if len >= 12 && self.ole_streams.contains(key) {
            let head = if self.encrypted && !stream_compressed {
                plain.get(..12).map(<[u8]>::to_vec).unwrap_or_default()
            } else {
                cfb_reader::decompress_stream_prefix(&plain, 12)
                    .unwrap_or_else(|_| plain.get(..12).map(<[u8]>::to_vec).unwrap_or_default())
            };
            let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
            if head.len() >= 12 && head[..8] != cfb_magic && head[4..12] == cfb_magic {
                return len - 4;
            }
        }
        len
    }

    /// [#2550] 상한 초과 항목은 placeholder(빈 값) 의미이므로 길이 0 판정을 공유한다.
    fn resolved_is_empty(&self, key: &str) -> bool {
        self.resolved_len(key) == 0
    }
}

fn load_bin_data_content(
    cfb: &mut cfb_reader::CfbReader,
    data: &[u8],
    bin_data_list: &[crate::model::bin_data::BinData],
    compressed: bool,
    encrypted: bool,
    password: Option<&[u8]>,
) -> Vec<BinDataContent> {
    use crate::model::bin_data::BinDataType;

    // 지연 로딩 리졸버가 참조할 OLE Storage 스트림 집합을 먼저 구성한다.
    let mut ole_streams = std::collections::HashSet::new();
    let mut compressed_streams = std::collections::HashMap::new();
    for bd in bin_data_list.iter() {
        if matches!(bd.data_type, BinDataType::Embedding | BinDataType::Storage) {
            let ext = if bd.data_type == BinDataType::Storage {
                bd.extension.as_deref().unwrap_or("OLE")
            } else {
                bd.extension.as_deref().unwrap_or("dat")
            };
            let storage_name = format!("BIN{:04X}.{}", bd.storage_id, ext);
            if bd.data_type == BinDataType::Storage {
                ole_streams.insert(storage_name.clone());
            }
            compressed_streams.insert(
                storage_name,
                bin_data_stream_is_compressed(bd.compression, compressed, encrypted),
            );
        }
    }

    let resolver: Option<std::sync::Arc<dyn crate::model::bin_data::BinDataResolver>> =
        match cfb_reader::CfbReader::open(data) {
            Ok(reader) => Some(std::sync::Arc::new(Hwp5BinResolver {
                cfb: std::sync::Mutex::new(reader),
                ole_streams,
                compressed_streams,
                encrypted,
                password: password.map(|p| p.to_vec()),
            })),
            Err(e) => {
                // 리졸버를 못 열면 지연 로딩 불가 — 기존처럼 즉시 로드로 폴백한다.
                eprintln!(
                    "경고: BinData 지연 로딩 리졸버 생성 실패: {} — 즉시 로드로 폴백",
                    e
                );
                None
            }
        };

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
        let stream_compressed =
            bin_data_stream_is_compressed(bd.compression, compressed, encrypted);

        // [Task #2263] 스트림 존재만 확인하고(압축 해제 없이) 지연 등록한다.
        //
        // 기존 동작은 읽기 실패 시 항목을 배열에 넣지 않고 건너뛴다. 이 의미를
        // 보존해야 위치 기반 조회(`find_bin_data` 의 `get(id-1)`)와 왕복 길이
        // 비교가 깨지지 않으므로, `has_stream` 으로 존재 여부만 미리 확인한다.
        if let Some(resolver) = resolver.as_ref() {
            if !cfb.has_stream(&format!("/BinData/{}", storage_name)) {
                eprintln!("경고: BinData '{}' 스트림 없음", storage_name);
                continue;
            }
            contents.push(BinDataContent {
                id: bd.storage_id,
                data: crate::model::bin_data::BinDataBytes::Lazy {
                    resolver: resolver.clone(),
                    key: storage_name.clone(),
                },
                extension: ext.to_string(),
            });
            continue;
        }

        match cfb.read_bin_data(&storage_name) {
            Ok(data) => {
                // 암호 문서: 복호화+압축해제. 그 외: 압축 해제 시도 (실패 시 원본).
                let mut decompressed = if encrypted {
                    let pwd = password.unwrap();
                    match crypto::decrypt_password_protected_limited(
                        &data,
                        pwd,
                        stream_compressed,
                        MAX_PASSWORD_PROTECTED_BIN_DATA_OUTPUT_BYTES,
                    ) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!(
                                "경고: BinData '{}' 복호화 실패 (비밀번호 불일치?): {}",
                                storage_name, e
                            );
                            continue;
                        }
                    }
                } else {
                    match cfb_reader::decompress_stream(&data) {
                        Ok(d) => d,
                        Err(_) => data, // 압축 해제 실패 시 원본 사용 (비압축 데이터)
                    }
                };

                // Task #195 단계 6: OLE Storage는 해제 후 선두 4바이트 size prefix를 스킵하여
                // 내부 CFB(`d0cf11e0...`) 시작 바이트부터 노출한다.
                if is_storage && decompressed.len() >= 12 {
                    // CFB 매직이 바로 시작하면 prefix 없음
                    let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
                    if decompressed[..8] != cfb_magic && decompressed[4..12] == cfb_magic {
                        decompressed.drain(..4);
                    }
                }

                contents.push(BinDataContent {
                    id: bd.storage_id,
                    data: decompressed.into(),
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

    /// [robustness] 초인적 퍼징(7479 손상)이 잡은 `normalize_variant_paragraph_vpos`
    /// 의 i32 덧셈 오버플로 패닉 회귀(mod.rs:628). 손상 입력의 거대 vpos/height 로
    /// `vertical_pos + line_height + line_spacing` 이 오버플로해 패닉하던 것을
    /// saturating 으로 막았다 — 이제 파싱이 패닉 없이 Ok/Err 로 끝난다.
    #[test]
    fn corrupt_variant_vpos_does_not_overflow_panic() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/hwp3-sample11-hwp5.hwp"
        );
        let data = std::fs::read(path).expect("샘플 읽기");
        let mut corrupt = data.clone();
        let pos = corrupt.len() * 90 / 100; // 감사기가 패닉을 재현한 결정적 손상
        corrupt[pos] ^= 0xFF;
        let _ = parse_document(&corrupt); // 결과값 무관 — 패닉만 안 하면 통과
    }

    fn png_preview() -> Vec<u8> {
        let mut png = vec![0; 24];
        png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png[16..20].copy_from_slice(&1u32.to_be_bytes());
        png[20..24].copy_from_slice(&1u32.to_be_bytes());
        png
    }

    fn hwp_with_preview(image: Vec<u8>) -> Vec<u8> {
        let mut document = Document::default();
        document.preview = Some(Preview {
            image: Some(PreviewImage {
                format: PreviewImageFormat::Png,
                data: image,
            }),
            text: None,
        });
        crate::serializer::serialize_document(&document).expect("serialize HWP preview fixture")
    }

    fn hwpx_with_preview(image: Vec<u8>) -> Vec<u8> {
        let mut document = Document::default();
        document
            .hwpx_aux_entries
            .push(("Preview/PrvImage.png".to_string(), image));
        crate::serializer::hwpx::serialize_hwpx(&document).expect("serialize HWPX preview fixture")
    }

    #[test]
    fn thumbnail_limited_read_rejects_streamed_overflow() {
        let mut input = std::io::Cursor::new(vec![0u8; 9]);
        assert!(read_thumbnail_limited(&mut input, 8, 8).is_none());
    }

    #[test]
    fn thumbnail_limited_read_rejects_truncated_stream() {
        let mut input = std::io::Cursor::new(vec![0u8; 7]);
        assert!(read_thumbnail_limited(&mut input, 8, 8).is_none());
    }

    #[test]
    fn document_open_preserves_small_hwp_thumbnail() {
        let image = png_preview();
        let document = parse_document(&hwp_with_preview(image.clone()))
            .expect("document opens with a valid thumbnail");

        assert_eq!(
            document
                .preview
                .as_ref()
                .and_then(|preview| preview.image.as_ref())
                .map(|preview| preview.data.as_slice()),
            Some(image.as_slice())
        );
    }

    #[test]
    fn document_open_omits_oversized_hwp_thumbnail() {
        let document = parse_document(&hwp_with_preview(vec![0; MAX_THUMBNAIL_BYTES + 1]))
            .expect("document still opens when its optional thumbnail is oversized");

        assert!(document
            .preview
            .as_ref()
            .and_then(|preview| preview.image.as_ref())
            .is_none());
    }

    #[test]
    fn document_open_preserves_small_hwpx_thumbnail() {
        let image = png_preview();
        let document = parse_document(&hwpx_with_preview(image.clone()))
            .expect("document opens with a valid HWPX thumbnail");

        assert_eq!(
            document.hwpx_aux_entry("Preview/PrvImage.png"),
            Some(image.as_slice())
        );
        assert_eq!(
            document
                .extra_streams
                .iter()
                .find(|(path, _)| path == "/PrvImage")
                .map(|(_, data)| data.as_slice()),
            Some(image.as_slice())
        );
    }

    #[test]
    fn document_open_omits_oversized_hwpx_thumbnail() {
        let document = parse_document(&hwpx_with_preview(vec![0; MAX_THUMBNAIL_BYTES + 1]))
            .expect("document still opens when its optional HWPX thumbnail is oversized");

        assert!(document.hwpx_aux_entry("Preview/PrvImage.png").is_none());
        assert!(document
            .extra_streams
            .iter()
            .find(|(path, _)| path == "/PrvImage")
            .is_some_and(|(_, data)| data.len() < MAX_THUMBNAIL_BYTES));
    }

    #[test]
    fn cfb_thumbnail_rejects_declared_output_above_limit() {
        use std::io::Write;

        let mut compound = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new())).unwrap();
        {
            let mut image = compound.create_stream("/PrvImage").unwrap();
            image
                .write_all(&vec![0u8; MAX_THUMBNAIL_BYTES + 1])
                .unwrap();
        }

        assert!(extract_thumbnail_only(&compound.into_inner().into_inner()).is_none());
    }

    #[test]
    fn hwpx_thumbnail_rejects_declared_output_above_limit() {
        use std::io::Write;

        let mut archive = std::io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut archive);
            writer
                .start_file(
                    "Preview/PrvImage.png",
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated),
                )
                .unwrap();
            writer
                .write_all(&vec![0u8; MAX_THUMBNAIL_BYTES + 1])
                .unwrap();
            writer.finish().unwrap();
        }

        assert!(extract_thumbnail_only(&archive.into_inner()).is_none());
    }

    #[test]
    fn hwpx_thumbnail_rejects_declared_actual_size_mismatch() {
        use std::io::Write;

        let mut archive = std::io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut archive);
            writer
                .start_file(
                    "Preview/PrvImage.png",
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored),
                )
                .unwrap();
            writer.write_all(&[1, 2, 3]).unwrap();
            writer.finish().unwrap();
        }

        let mut bytes = archive.into_inner();
        for (signature, size_offset) in [
            ([0x50, 0x4B, 0x03, 0x04], 22),
            ([0x50, 0x4B, 0x01, 0x02], 24),
        ] {
            let header = bytes
                .windows(signature.len())
                .position(|window| window == signature)
                .unwrap();
            bytes[header + size_offset..header + size_offset + 4]
                .copy_from_slice(&4u32.to_le_bytes());
        }

        assert!(extract_thumbnail_only(&bytes).is_none());
    }

    #[test]
    fn open_decompression_budget_rejects_cumulative_overflow() {
        let mut budget =
            Hwp5DocumentOpenDecompressionBudget::new(Hwp5DocumentOpenDecompressionLimits {
                max_stream_input_bytes: 1024,
                max_stream_output_bytes: 1024,
                max_total_output_bytes: 1024,
            });
        budget.consume(700).unwrap();
        assert_eq!(budget.stream_limit(), 324);
        assert!(matches!(
            budget.consume(325),
            Err(cfb_reader::CfbError::LimitExceeded(324))
        ));
        budget.consume(324).unwrap();
        assert_eq!(budget.stream_limit(), 0);
    }

    fn document_with_number_controls(controls: Vec<crate::model::control::Control>) -> Document {
        let mut paragraph = crate::model::paragraph::Paragraph::default();
        paragraph.controls = controls;

        let mut section = crate::model::document::Section::default();
        section.paragraphs.push(paragraph);

        let mut document = Document::default();
        document.doc_properties.page_start_num = 1;
        document.sections.push(section);
        document
    }

    fn auto_number_values(
        document: &Document,
    ) -> Vec<(crate::model::control::AutoNumberType, u16, u16)> {
        use crate::model::control::Control;

        document.sections[0].paragraphs[0]
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::AutoNumber(auto_number) => Some((
                    auto_number.number_type,
                    auto_number.number,
                    auto_number.assigned_number,
                )),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn total_page_auto_number_does_not_advance_page_counter() {
        use crate::model::control::{AutoNumber, AutoNumberType, Control};

        let mut document = document_with_number_controls(vec![
            Control::AutoNumber(AutoNumber {
                number_type: AutoNumberType::Page,
                ..Default::default()
            }),
            Control::AutoNumber(AutoNumber {
                number_type: AutoNumberType::TotalPage,
                number: 8,
                assigned_number: 8,
                ..Default::default()
            }),
            Control::AutoNumber(AutoNumber {
                number_type: AutoNumberType::Page,
                ..Default::default()
            }),
        ]);

        assign_auto_numbers(&mut document);

        assert_eq!(
            auto_number_values(&document),
            vec![
                (AutoNumberType::Page, 1, 1),
                (AutoNumberType::TotalPage, 8, 8),
                (AutoNumberType::Page, 2, 2),
            ],
            "TotalPage 표시값은 보존되고 뒤 Page 번호는 연속이어야 한다"
        );
    }

    #[test]
    fn total_page_new_number_does_not_reset_page_counter() {
        use crate::model::control::{AutoNumber, AutoNumberType, Control, NewNumber};

        let mut document = document_with_number_controls(vec![
            Control::AutoNumber(AutoNumber {
                number_type: AutoNumberType::Page,
                ..Default::default()
            }),
            Control::NewNumber(NewNumber {
                number_type: AutoNumberType::TotalPage,
                number: 99,
            }),
            Control::AutoNumber(AutoNumber {
                number_type: AutoNumberType::Page,
                ..Default::default()
            }),
            Control::NewNumber(NewNumber {
                number_type: AutoNumberType::Page,
                number: 10,
            }),
            Control::AutoNumber(AutoNumber {
                number_type: AutoNumberType::Page,
                ..Default::default()
            }),
        ]);

        assign_auto_numbers(&mut document);

        assert_eq!(
            auto_number_values(&document),
            vec![
                (AutoNumberType::Page, 1, 1),
                (AutoNumberType::Page, 2, 2),
                (AutoNumberType::Page, 10, 10),
            ],
            "NewNumber(TotalPage)는 Page 카운터에 영향이 없고 NewNumber(Page)는 유지돼야 한다"
        );
    }

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
        assert_eq!(detect_format(hwpml), FileFormat::Hml);
    }

    #[test]
    fn test_detect_format_rejects_lowercase_generic_xml_root() {
        let hwpml = b"\xEF\xBB\xBF  \n<?xml version='1.0'?><hwpml version='2.1'></hwpml>";
        assert_eq!(detect_format(hwpml), FileFormat::Unknown);
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
    fn test_parse_document_reports_unsupported_hwpml_version() {
        let hwpml = br#"<?xml version="1.0" encoding="UTF-8"?>
<HWPML Version="2.1"></HWPML>"#;
        let err = parse_document(hwpml).unwrap_err();
        match err {
            ParseError::HmlError(hml::HmlError::UnsupportedVersion(version)) => {
                assert_eq!(version, "2.1");
            }
            other => panic!("expected HML unsupported version, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_document_with_metadata_preserves_hml_import_diagnostics() {
        let parsed =
            parse_document_with_metadata(include_bytes!("../../samples/hml/formatting_table.hml"))
                .expect("real HML fixture should parse");
        let metadata = parsed
            .hml_metadata
            .expect("HML import metadata should be retained");

        assert_eq!(metadata.hwpml_version.as_deref(), Some("2.91"));
        assert_eq!(metadata.encoding, hml::HmlEncoding::Utf8);
        assert_eq!(metadata.resource_count, 0);
        assert!(metadata
            .warnings
            .iter()
            .any(|warning| warning.xml_path == "/HWPML/TAIL/SCRIPTCODE"));
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

    fn set_password_header(data: &[u8], encrypt_version: u32) -> Vec<u8> {
        use std::io::{Read, Seek, SeekFrom, Write};

        let mut compound =
            cfb::CompoundFile::open(std::io::Cursor::new(data.to_vec())).expect("cfb open");
        let mut header = Vec::new();
        {
            let mut stream = compound
                .open_stream("/FileHeader")
                .expect("FileHeader 스트림");
            stream.read_to_end(&mut header).unwrap();
        }
        let flags = u32::from_le_bytes(header[36..40].try_into().unwrap()) | 0x02;
        header[36..40].copy_from_slice(&flags.to_le_bytes());
        header[44..48].copy_from_slice(&encrypt_version.to_le_bytes());
        {
            let mut stream = compound
                .create_stream("/FileHeader")
                .expect("FileHeader 갱신");
            stream.seek(SeekFrom::Start(0)).unwrap();
            stream.write_all(&header).unwrap();
        }
        compound.into_inner().into_inner()
    }

    fn set_distribution_header(data: &[u8]) -> Vec<u8> {
        use std::io::{Read, Seek, SeekFrom, Write};

        let mut compound =
            cfb::CompoundFile::open(std::io::Cursor::new(data.to_vec())).expect("cfb open");
        let mut header = Vec::new();
        {
            let mut stream = compound
                .open_stream("/FileHeader")
                .expect("FileHeader 스트림");
            stream.read_to_end(&mut header).unwrap();
        }
        let flags = u32::from_le_bytes(header[36..40].try_into().unwrap()) | 0x04;
        header[36..40].copy_from_slice(&flags.to_le_bytes());
        {
            let mut stream = compound
                .create_stream("/FileHeader")
                .expect("FileHeader 갱신");
            stream.seek(SeekFrom::Start(0)).unwrap();
            stream.write_all(&header).unwrap();
        }
        compound.into_inner().into_inner()
    }

    fn add_raw_streams(data: &[u8], streams: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;

        let mut compound =
            cfb::CompoundFile::open(std::io::Cursor::new(data.to_vec())).expect("cfb open");
        for (path, payload) in streams {
            if let Some(parent) = std::path::Path::new(path).parent() {
                compound.create_storage_all(parent).unwrap();
            }
            let mut stream = compound.create_stream(path).expect("테스트 스트림 생성");
            stream.write_all(payload).unwrap();
        }
        compound.into_inner().into_inner()
    }

    fn raw_deflate(data: &[u8]) -> Vec<u8> {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn hwp5_document_open_policy_for_test(
        max_stream_output_bytes: usize,
        max_total_output_bytes: usize,
    ) -> DocumentOpenDecompressionPolicy {
        hwp5_document_open_policy_with_input_for_test(
            max_stream_output_bytes,
            max_stream_output_bytes,
            max_total_output_bytes,
        )
    }

    fn hwp5_document_open_policy_with_input_for_test(
        max_stream_input_bytes: usize,
        max_stream_output_bytes: usize,
        max_total_output_bytes: usize,
    ) -> DocumentOpenDecompressionPolicy {
        DocumentOpenDecompressionPolicy {
            hwp5: Hwp5DocumentOpenDecompressionLimits {
                max_stream_input_bytes,
                max_stream_output_bytes,
                max_total_output_bytes,
            },
            ..DocumentOpenDecompressionPolicy::DEFAULT
        }
    }

    fn hwp3_document_open_policy_for_test(
        max_body_output_bytes: usize,
    ) -> DocumentOpenDecompressionPolicy {
        DocumentOpenDecompressionPolicy {
            hwp3_max_body_output_bytes: max_body_output_bytes,
            ..DocumentOpenDecompressionPolicy::DEFAULT
        }
    }

    fn hwp5_doc_info_and_first_section(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut cfb = cfb_reader::CfbReader::open(data).expect("HWP5 CFB fixture");
        let header = header::parse_file_header(&cfb.read_file_header().unwrap())
            .expect("HWP5 FileHeader fixture");
        assert!(
            header.flags.compressed,
            "fixture must exercise decompression"
        );
        let doc_info = cfb.read_doc_info(true).expect("compressed DocInfo");
        let section = cfb
            .read_body_text_section(0, true, false)
            .expect("compressed BodyText/Section0");
        (doc_info, section)
    }

    fn hwp5_raw_doc_info(data: &[u8]) -> Vec<u8> {
        let mut cfb = cfb_reader::CfbReader::open(data).expect("HWP5 CFB fixture");
        cfb.read_stream_raw("/DocInfo")
            .expect("raw DocInfo fixture")
    }

    fn replace_raw_stream(data: &[u8], path: &str, payload: &[u8]) -> Vec<u8> {
        use std::io::Write;

        let mut compound =
            cfb::CompoundFile::open(std::io::Cursor::new(data.to_vec())).expect("cfb open");
        let mut stream = compound.create_stream(path).expect("테스트 스트림 교체");
        stream.write_all(payload).unwrap();
        compound.into_inner().into_inner()
    }

    /// 기본 CFB reader가 열기 단계에서 거부하지만 LenientCfbReader는 계속 읽을 수 있는
    /// FAT 메타데이터 불일치를 만든다. 실제 디렉터리/스트림 chain은 건드리지 않으므로
    /// 아래 공개 parse 경로가 strict → lenient fallback을 실제로 지난다.
    fn force_lenient_cfb_fallback(data: &[u8]) -> Vec<u8> {
        const CFB_HEADER_BYTES: usize = 512;

        let mut corrupted = data.to_vec();
        assert!(
            corrupted.len() >= CFB_HEADER_BYTES,
            "CFB fixture must include its header"
        );
        let sector_shift =
            u16::from_le_bytes(corrupted[30..32].try_into().expect("sector shift bytes"));
        assert!(
            (9..=12).contains(&sector_shift),
            "fixture must use a supported CFB sector size"
        );
        let sector_size = 1usize << sector_shift;
        let fat_sectors_count =
            u32::from_le_bytes(corrupted[44..48].try_into().expect("FAT count bytes")) as usize;
        assert!(
            (1..=109).contains(&fat_sectors_count),
            "fixture must keep its FAT sector ids in the CFB header"
        );
        let fat_sector_ids: Vec<u32> = (0..fat_sectors_count)
            .map(|index| {
                let offset = 76 + index * 4;
                u32::from_le_bytes(
                    corrupted[offset..offset + 4]
                        .try_into()
                        .expect("FAT sector id bytes"),
                )
            })
            .collect();
        let entries_per_fat_sector = sector_size / 4;
        let physical_sector_count = (corrupted.len() - CFB_HEADER_BYTES) / sector_size;
        let fat_entry_offset = |sector_id: usize| {
            let owner_index = sector_id / entries_per_fat_sector;
            let owner_sector_id = *fat_sector_ids
                .get(owner_index)
                .expect("fixture FAT must cover each physical sector")
                as usize;
            CFB_HEADER_BYTES
                + owner_sector_id * sector_size
                + (sector_id % entries_per_fat_sector) * 4
        };
        let target = (0..physical_sector_count)
            .filter_map(|sector_id| {
                let offset = fat_entry_offset(sector_id);
                let value = u32::from_le_bytes(
                    corrupted[offset..offset + 4]
                        .try_into()
                        .expect("FAT entry bytes"),
                );
                (value < physical_sector_count as u32).then_some(value)
            })
            .next()
            .expect("fixture must contain a live FAT pointee");
        let padding_offset = fat_entry_offset(physical_sector_count);
        assert!(
            padding_offset + 4 <= corrupted.len(),
            "fixture FAT padding entry must be in range"
        );
        // The padding slot is beyond the file's physical sectors and hence cannot
        // be reached by the intact document. Reusing a live target makes the
        // permissive cfb allocator reject duplicate pointees; LenientCfbReader
        // ignores this unused tail entry and retains the real chains.
        corrupted[padding_offset..padding_offset + 4].copy_from_slice(&target.to_le_bytes());

        assert!(
            cfb_reader::CfbReader::open(&corrupted).is_err(),
            "mutation must make the standard CFB reader fail"
        );
        assert!(
            cfb_reader::LenientCfbReader::open(&corrupted).is_ok(),
            "mutation must retain a usable lenient CFB path"
        );
        corrupted
    }

    /// 공개 `parse_hwp` 진입점이 low-level CFB 기본값이 아니라 문서 열기 정책을
    /// 선택하고, DocInfo의 압축 해제 출력을 그 정책으로 차단하는지 검증한다.
    #[test]
    fn hwp5_open_decompression_rejects_doc_info_past_e2e_limit() {
        const LIMIT: usize = 1024;

        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let _ = hwp5_doc_info_and_first_section(&source);
        let oversized_doc_info = raw_deflate(&vec![0; LIMIT + 1]);
        let mutated = replace_raw_stream(&source, "/DocInfo", &oversized_doc_info);

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_for_test(LIMIT, LIMIT * 2),
            || parse_hwp(&mutated),
        );

        assert!(matches!(
            result,
            Err(ParseError::CfbError(cfb_reader::CfbError::LimitExceeded(
                LIMIT
            )))
        ));
    }

    /// 공개 HWP5 문서 열기는 압축 해제 전에 DocInfo 원시 스트림 자체를 제한해야 한다.
    /// trailing bytes는 raw deflate decoder가 본문을 정상 해제할 수 있어도, 이전 구현에서는
    /// 불필요하게 모두 materialize했으므로 원시 입력 제한의 회귀 fixture로 사용한다.
    #[test]
    fn hwp5_open_rejects_doc_info_raw_input_before_decode() {
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let mut oversized_raw_doc_info = hwp5_raw_doc_info(&source);
        let raw_limit = oversized_raw_doc_info.len();
        oversized_raw_doc_info.push(0);
        let mutated = replace_raw_stream(&source, "/DocInfo", &oversized_raw_doc_info);

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_with_input_for_test(raw_limit, 1024 * 1024, 2 * 1024 * 1024),
            || parse_document(&mutated),
        );

        assert!(matches!(
            result,
            Err(ParseError::CfbError(cfb_reader::CfbError::LimitExceeded(limit)))
                if limit == raw_limit
        ));
    }

    /// strict CFB가 거부되어 lenient fallback을 타는 경우도 같은 raw 입력 상한을
    /// 디코딩 전에 적용해야 한다.
    #[test]
    fn hwp5_open_lenient_rejects_doc_info_raw_input_before_decode() {
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let mut oversized_raw_doc_info = hwp5_raw_doc_info(&source);
        let raw_limit = oversized_raw_doc_info.len();
        oversized_raw_doc_info.push(0);
        let strict_rejected = replace_raw_stream(&source, "/DocInfo", &oversized_raw_doc_info);
        let mutated = force_lenient_cfb_fallback(&strict_rejected);

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_with_input_for_test(raw_limit, 1024 * 1024, 2 * 1024 * 1024),
            || parse_document(&mutated),
        );

        assert!(matches!(
            result,
            Err(ParseError::CfbError(cfb_reader::CfbError::LimitExceeded(limit)))
                if limit == raw_limit
        ));
    }

    /// 배포 플래그가 있지만 ViewText가 없는 손상 HWP5는 BodyText로 fallback해
    /// 무제한 해제하지 않고, 공개 문서 열기 경계에서 missing ViewText 오류로 끝나야 한다.
    #[test]
    fn hwp5_open_decompression_rejects_missing_viewtext_without_bodytext_fallback() {
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let (doc_info, _) = hwp5_doc_info_and_first_section(&source);
        let limit = doc_info.len().max(1024);
        let oversized_body_text = raw_deflate(&vec![0; limit + 1]);
        let mutated = set_distribution_header(&replace_raw_stream(
            &source,
            "/BodyText/Section0",
            &oversized_body_text,
        ));

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_for_test(limit, limit * 2),
            || parse_document(&mutated),
        );

        assert!(matches!(
            result,
            Err(ParseError::CfbError(cfb_reader::CfbError::StreamNotFound(path)))
                if path == "/ViewText/Section0"
        ));
    }

    /// strict CFB 검증을 통과하지 못해 lenient fallback으로 연 문서도 공개 자동 감지
    /// 경로에서 같은 DocInfo 출력 상한을 적용해야 한다.
    #[test]
    fn hwp5_open_decompression_lenient_path_rejects_doc_info_past_e2e_limit() {
        const LIMIT: usize = 1024;

        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let oversized_doc_info = raw_deflate(&vec![0; LIMIT + 1]);
        let strict_rejected = replace_raw_stream(&source, "/DocInfo", &oversized_doc_info);
        let mutated = force_lenient_cfb_fallback(&strict_rejected);

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_for_test(LIMIT, LIMIT * 2),
            || parse_document(&mutated),
        );

        assert!(matches!(
            result,
            Err(ParseError::CfbError(cfb_reader::CfbError::LimitExceeded(
                LIMIT
            )))
        ));
    }

    /// Lenient fallback에서도 배포 플래그가 켜졌다면 정확한 ViewText hierarchy만
    /// 읽어야 한다. 손상된 BodyText를 fallback으로 해제하면 안 된다.
    #[test]
    fn hwp5_open_decompression_lenient_distribution_rejects_missing_viewtext() {
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let (doc_info, _) = hwp5_doc_info_and_first_section(&source);
        let limit = doc_info.len().max(1024);
        let oversized_body_text = raw_deflate(&vec![0; limit + 1]);
        let strict_rejected = set_distribution_header(&replace_raw_stream(
            &source,
            "/BodyText/Section0",
            &oversized_body_text,
        ));
        let mutated = force_lenient_cfb_fallback(&strict_rejected);

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_for_test(limit, limit * 2),
            || parse_document(&mutated),
        );

        match &result {
            Err(ParseError::CfbError(cfb_reader::CfbError::StreamNotFound(path)))
                if path == "ViewText/Section0" => {}
            other => panic!("expected missing exact lenient ViewText path, got {other:?}"),
        }
    }

    /// 자동 감지 공개 진입점에서 DocInfo와 첫 BodyText가 각각의 단일 스트림 상한은
    /// 넘지 않아도 같은 문서 열기 누적 예산을 공유하는지 검증한다.
    #[test]
    fn hwp5_open_decompression_rejects_cumulative_output_through_parse_document() {
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let (doc_info, section) = hwp5_doc_info_and_first_section(&source);
        assert!(!doc_info.is_empty(), "DocInfo fixture must have output");
        assert!(
            section.len() > 1,
            "BodyText fixture must have enough output for a one-byte deficit"
        );

        let max_total_output_bytes = doc_info
            .len()
            .checked_add(section.len())
            .expect("fixture output length fits usize")
            - 1;
        let expected_remaining_limit = section.len() - 1;
        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_for_test(
                doc_info.len().max(section.len()),
                max_total_output_bytes,
            ),
            || parse_document(&source),
        );

        assert!(matches!(
            result,
            Err(ParseError::CfbError(cfb_reader::CfbError::LimitExceeded(limit)))
                if limit == expected_remaining_limit
        ));
    }

    /// 비밀번호 보호 HWP5도 공개 `parse_document_with_password` 경로에서 같은
    /// 문서 열기 정책의 명시적 상한을 crypto helper에 전달하는지 검증한다.
    #[test]
    fn hwp5_open_decompression_encrypted_doc_info_uses_e2e_limit() {
        const PASSWORD: &[u8] = b"e2e-open-budget";

        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let (doc_info, _) = hwp5_doc_info_and_first_section(&source);
        assert!(doc_info.len() > 1, "DocInfo fixture must have output");
        let encrypted = encrypt_hwp_streams_for_test(&source, PASSWORD);
        assert!(
            parse_document_with_password(&encrypted, PASSWORD).is_ok(),
            "encrypted fixture must be valid before constraining its public open path"
        );

        let limit = doc_info.len() - 1;
        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_for_test(limit, doc_info.len()),
            || parse_document_with_password(&encrypted, PASSWORD),
        );

        assert!(matches!(
            result,
            Err(ParseError::CryptoError(
                crypto::CryptoError::DecompressedStreamLimitExceeded { max_bytes }
            )) if max_bytes == limit
        ));
    }

    /// 비밀번호 문서도 복호화 helper에 전달하기 전에 암호문 raw 입력 크기를 차단해야 한다.
    #[test]
    fn hwp5_open_encrypted_doc_info_rejects_raw_input_before_decrypt() {
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let password = hwp5_raw_doc_info(&source)
            .into_iter()
            .take(16)
            .collect::<Vec<_>>();
        assert!(
            !password.is_empty(),
            "DocInfo fixture must provide test bytes"
        );

        let encrypted = encrypt_hwp_streams_for_test(&source, &password);
        let ciphertext_len = hwp5_raw_doc_info(&encrypted).len();
        assert!(
            ciphertext_len > 1,
            "encrypted DocInfo fixture must be nonempty"
        );
        let raw_limit = ciphertext_len - 1;

        let result = with_document_open_decompression_policy_for_test(
            hwp5_document_open_policy_with_input_for_test(raw_limit, 1024 * 1024, 2 * 1024 * 1024),
            || parse_document_with_password(&encrypted, &password),
        );

        assert!(
            matches!(
                &result,
                Err(ParseError::CfbError(cfb_reader::CfbError::LimitExceeded(limit)))
                    if *limit == raw_limit
            ),
            "expected raw input limit {raw_limit}"
        );
    }

    /// HWP3도 public `parse_document` 자동 감지 경로에서 작은 문서 열기 상한을
    /// HWP3 본문 압축 해제 helper로 전달하는지 실제 압축 fixture로 검증한다.
    #[test]
    fn hwp3_open_decompression_rejects_compressed_body_through_parse_document() {
        let source = std::fs::read("samples/issue_265.hwp").expect("HWP3 sample 존재");
        assert_eq!(detect_format(&source), FileFormat::Hwp3);
        assert_eq!(
            source.get(154),
            Some(&1),
            "fixture must mark its HWP3 body as compressed"
        );
        assert!(
            parse_document(&source).is_ok(),
            "HWP3 fixture must be valid before constraining its public open path"
        );

        let result = with_document_open_decompression_policy_for_test(
            hwp3_document_open_policy_for_test(1),
            || parse_document(&source),
        );

        assert!(matches!(
            result,
            Err(ParseError::Hwp3Error(hwp3::Hwp3Error::ParseError { message }))
                if message.contains("1 바이트 상한")
        ));
    }

    fn encrypt_hwp_streams_for_test(data: &[u8], password: &[u8]) -> Vec<u8> {
        use std::io::{Read, Write};

        let mut compound =
            cfb::CompoundFile::open(std::io::Cursor::new(data.to_vec())).expect("cfb open");
        let encrypted_paths: Vec<String> = compound
            .walk()
            .filter(|entry| entry.is_stream())
            .map(|entry| entry.path().to_string_lossy().replace('\\', "/"))
            .filter(|path| {
                path == "/DocInfo"
                    || path.starts_with("/BodyText/")
                    || path.starts_with("/ViewText/")
                    || path.starts_with("/BinData/")
                    || path.starts_with("/Scripts/")
            })
            .collect();

        for path in encrypted_paths {
            let mut raw = Vec::new();
            {
                let mut stream = compound.open_stream(&path).expect("암호화 대상 스트림");
                stream.read_to_end(&mut raw).unwrap();
            }
            let encrypted = crypto::encrypt_password_stream_for_test(&raw, password);
            let mut stream = compound.create_stream(&path).expect("암호문 스트림 갱신");
            stream.write_all(&encrypted).unwrap();
        }

        let bytes = compound.into_inner().into_inner();
        set_password_header(&bytes, crypto::SUPPORTED_PASSWORD_ENCRYPT_VERSION)
    }

    fn extra_stream_payload<'a>(doc: &'a Document, path: &str) -> &'a [u8] {
        doc.extra_streams
            .iter()
            .find(|(candidate, _)| candidate == path)
            .map(|(_, data)| data.as_slice())
            .unwrap_or_else(|| panic!("추가 스트림 없음: {path}"))
    }

    fn total_paragraphs(doc: &Document) -> usize {
        doc.sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum()
    }

    /// 비밀번호 암호 라우팅 검증: 일반 샘플의 FileHeader 에서 encrypted 비트와
    /// EncryptVersion만 켠 변형을 만들어 (스트림 자체는 암호화하지 않음)
    /// 파서의 분기를 확인한다.
    ///
    /// - 비밀번호 없음 → ParseError::EncryptedDocument
    /// - 비밀번호 제공(틀림) → 실제 복호화 시도 후 WrongPassword (압축 해제 실패)
    /// - 원본은 여전히 정상 파싱 (회귀 없음)
    #[test]
    fn test_encrypted_flag_routes_password_paths() {
        let original = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        // 원본 정상 파싱 (회귀 가드)
        assert!(parse_document(&original).is_ok(), "원본은 파싱되어야 함");

        let modified = set_password_header(&original, crypto::SUPPORTED_PASSWORD_ENCRYPT_VERSION);

        // 1) 비밀번호 없음 → EncryptedDocument
        match parse_document(&modified) {
            Err(ParseError::EncryptedDocument) => {}
            Err(e) => panic!("비밀번호 없는 암호 문서는 EncryptedDocument 이어야 함: {e}"),
            Ok(_) => panic!("encrypted 플래그가 켜졌으면 파싱이 성공하면 안 됨"),
        }

        // 2) 비밀번호 제공 (스트림이 실제로 암호화되지 않았으므로 복호화 결과는 쓰레기
        //    → 압축 해제 실패 → CryptoError::WrongPassword)
        match parse_document_with_metadata_password(&modified, b"any-password") {
            Err(ParseError::CryptoError(crypto::CryptoError::WrongPassword)) => {}
            Err(e) => panic!("틀린 비밀번호는 WrongPassword 여야 함 (다른 에러): {e}"),
            Ok(_) => panic!("틀린 비밀번호인데 파싱이 성공하면 안 됨"),
        }
    }

    #[test]
    fn test_encrypted_document_rejects_unsupported_encrypt_version() {
        let original = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let modified = set_password_header(&original, 3);

        assert!(matches!(
            parse_document_with_password(&modified, b"password"),
            Err(ParseError::CryptoError(
                crypto::CryptoError::UnsupportedScheme { encrypt_version: 3 }
            ))
        ));
    }

    #[test]
    fn test_encrypted_stream_limited_decode_enforces_decompression_bound() {
        const PASSWORD: &[u8] = b"bounded-password";
        let plaintext = vec![b'A'; 4096];
        let compressed = raw_deflate(&plaintext);
        let encrypted = crypto::encrypt_password_stream_for_test(&compressed, PASSWORD);

        assert!(
            decode_encrypted_stream_limited(&encrypted, PASSWORD, true, 1024).is_none(),
            "압축 해제 결과가 상한을 넘으면 materialize하지 않아야 함"
        );
        assert_eq!(
            decode_encrypted_stream_limited(&encrypted, PASSWORD, true, plaintext.len()).unwrap(),
            plaintext
        );
    }

    #[test]
    fn test_password_encrypted_uncompressed_hwp_detects_wrong_password() {
        const PASSWORD: &[u8] = b"uncompressed-password";

        let mut source_doc = Document::default();
        source_doc.header.version = crate::model::document::HwpVersion {
            major: 5,
            minor: 0,
            build: 1,
            revision: 7,
        };
        source_doc.doc_properties.section_count = 1;
        source_doc.sections.push(crate::model::document::Section {
            paragraphs: vec![crate::model::paragraph::Paragraph {
                text: "비압축".to_string(),
                char_count: 4,
                ..Default::default()
            }],
            ..Default::default()
        });

        let uncompressed =
            crate::serializer::serialize_document(&source_doc).expect("비압축 기준 HWP");
        let baseline = parse_document(&uncompressed).expect("비압축 기준 재파싱");
        let encrypted = encrypt_hwp_streams_for_test(&uncompressed, PASSWORD);

        assert!(matches!(
            parse_document_with_password(&encrypted, b"wrong-password"),
            Err(ParseError::CryptoError(crypto::CryptoError::WrongPassword))
        ));
        let decrypted =
            parse_document_with_password(&encrypted, PASSWORD).expect("비압축 암호 HWP 파싱");
        assert_eq!(decrypted.sections.len(), baseline.sections.len());
        assert_eq!(total_paragraphs(&decrypted), total_paragraphs(&baseline));
    }

    #[test]
    fn test_password_encrypted_hwp_full_stream_roundtrip() {
        use crate::model::bin_data::{
            BinData, BinDataCompression, BinDataContent, BinDataStatus, BinDataType,
        };

        const PASSWORD: &[u8] = "한글-password".as_bytes();
        const SCRIPT_TEXT: &[u8] = b"function OnDocumentOpen() { return 7; }";
        const ORPHAN_TEXT: &[u8] = b"orphan BinData preserved after password decryption";
        const BIN_PAYLOAD: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

        // 권위 샘플을 한 번 정상 파싱한 뒤, 문서 전역 compressed=true와 반대인
        // NoCompress BinData를 추가해 스트림별 압축 속성도 함께 검증한다.
        let source = std::fs::read("samples/2010-01-06.hwp").expect("sample 존재");
        let mut source_doc = parse_document(&source).expect("원본 파싱");
        let storage_id = source_doc
            .doc_info
            .bin_data_list
            .iter()
            .map(|item| item.storage_id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        source_doc.doc_info.bin_data_list.push(BinData {
            raw_data: None,
            attr: 0x0021, // Embedding + NoCompress
            data_type: BinDataType::Embedding,
            compression: BinDataCompression::NoCompress,
            status: BinDataStatus::NotAccessed,
            abs_path: None,
            rel_path: None,
            storage_id,
            extension: Some("pwdtest".to_string()),
        });
        source_doc.bin_data_content.push(BinDataContent {
            id: storage_id,
            data: BIN_PAYLOAD.to_vec().into(),
            extension: "pwdtest".to_string(),
        });
        source_doc.doc_info.raw_stream_dirty = true;

        let serialized = crate::serializer::serialize_document(&source_doc).expect("기준 HWP 생성");
        let script_raw = raw_deflate(SCRIPT_TEXT);
        let orphan_raw = raw_deflate(ORPHAN_TEXT);
        let with_extras = add_raw_streams(
            &serialized,
            &[
                ("/Scripts/PasswordTest", &script_raw),
                ("/BinData/ORPHAN.pwd", &orphan_raw),
            ],
        );
        let baseline = parse_document(&with_extras).expect("기준 HWP 재파싱");

        // 실제 한컴 EncryptVersion 4 저장 동작은 BIN_DATA attr이 NoCompress여도
        // 문서 전역 compressed=true이면 BinData를 raw-deflate한 뒤 암호화한다.
        // 일반 HWP 기준본은 NoCompress를 유지하고, 암호화 입력만 이 on-disk
        // 형태로 바꿔 실제 파일의 압축 계약을 재현한다.
        let password_bin_path = format!("/BinData/BIN{:04X}.pwdtest", storage_id);
        let password_bin_raw = raw_deflate(BIN_PAYLOAD);
        let password_layout = add_raw_streams(
            &with_extras,
            &[(password_bin_path.as_str(), password_bin_raw.as_slice())],
        );
        let encrypted = encrypt_hwp_streams_for_test(&password_layout, PASSWORD);

        assert!(matches!(
            parse_document(&encrypted),
            Err(ParseError::EncryptedDocument)
        ));
        assert!(matches!(
            parse_document_with_password(&encrypted, b"wrong-password"),
            Err(ParseError::CryptoError(crypto::CryptoError::WrongPassword))
        ));

        let decrypted =
            parse_document_with_password(&encrypted, PASSWORD).expect("암호 HWP 전체 파싱");
        assert_eq!(decrypted.sections.len(), baseline.sections.len());
        assert_eq!(total_paragraphs(&decrypted), total_paragraphs(&baseline));
        assert_eq!(
            decrypted
                .bin_data_content
                .iter()
                .find(|content| content.id == storage_id)
                .expect("NoCompress BinData")
                .data
                .load(),
            BIN_PAYLOAD
        );
        assert_eq!(
            cfb_reader::decompress_stream(extra_stream_payload(
                &decrypted,
                "/Scripts/PasswordTest"
            ))
            .expect("Scripts 평문 raw-deflate"),
            SCRIPT_TEXT
        );
        assert_eq!(
            cfb_reader::decompress_stream(extra_stream_payload(&decrypted, "/BinData/ORPHAN.pwd"))
                .expect("고아 BinData 평문 raw-deflate"),
            ORPHAN_TEXT
        );

        // 암호화 쓰기는 지원하지 않으므로 저장 결과는 일반 HWP로 강하한다.
        // 이때 플래그뿐 아니라 EncryptVersion과 보존 스트림 암호문도 정리돼야 한다.
        let saved = crate::serializer::serialize_document(&decrypted).expect("복호 문서 저장");
        let mut saved_cfb = cfb_reader::CfbReader::open(&saved).expect("저장 CFB");
        let saved_header =
            header::parse_file_header(&saved_cfb.read_file_header().unwrap()).unwrap();
        assert!(!saved_header.flags.encrypted);
        assert_eq!(saved_header.encrypt_version, 0);

        let reparsed = parse_document(&saved).expect("저장 결과 비밀번호 없이 재파싱");
        assert_eq!(
            reparsed
                .bin_data_content
                .iter()
                .find(|content| content.id == storage_id)
                .expect("저장 NoCompress BinData")
                .data
                .load(),
            BIN_PAYLOAD
        );
        assert_eq!(
            cfb_reader::decompress_stream(extra_stream_payload(&reparsed, "/Scripts/PasswordTest"))
                .expect("저장 Scripts"),
            SCRIPT_TEXT
        );
        assert_eq!(
            cfb_reader::decompress_stream(extra_stream_payload(&reparsed, "/BinData/ORPHAN.pwd"))
                .expect("저장 고아 BinData"),
            ORPHAN_TEXT
        );
    }
}
