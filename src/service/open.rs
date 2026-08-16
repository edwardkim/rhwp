//! 문서 열기 — 형식 자동판별·크기 상한·비밀번호를 **한 곳에서** 처리한다.
//!
//! 지금은 이 일이 표면마다 다시 쓰여 있다. `src/main.rs` 는 `fs::read` → 오류 출력
//! 블록을 45번, `detect_format` 을 24번 되풀이하고, `src/mcp_serve.rs` 는 같은
//! 순서를 `session_open` 에서 한 번 더 쓰며, `src/wasm_api.rs` 는 비밀번호 유무
//! 분기를 세 번째로 쓴다. 셋의 결과가 다른 곳이 이미 있다 — 예를 들어 CLI 만
//! 비밀번호 오류를 갈래로 나누고, MCP 는 전부 "파싱 실패" 하나로 뭉갠다.
//!
//! [`DocumentService::open_bytes`] 는 그 순서를 한 번만 정의한다.
//!
//! 1. 크기 상한 — **파서를 부르기 전에** 끊는다.
//! 2. 형식 자동판별 — 열 수 없는 형식은 파서를 부르기 전에 [`ServiceError::UnsupportedFormat`].
//! 3. 비밀번호 유무 분기 → [`DocumentCore`] 로드.
//! 4. 실패는 [`ServiceError`] 로 이름을 붙여 올린다.
//!
//! 감지한 형식과 원본 크기는 **버리지 않고** [`OpenedDocument`] 가 들고 있는다.
//! 현행 소비자가 이미 파싱한 바이트를 `detect_format` 으로 한 번 더 훑는 이유가
//! 바로 이 두 값을 잃어버려서다(`DocumentCore::from_bytes_inner` 는 내부에서
//! `detect_format` 을 부르고도 결과를 폐기한다).

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::document_core::DocumentCore;
use crate::model::document::Document;
use crate::parser::FileFormat;
use crate::service::error::ServiceError;

/// 서비스가 열지 않는 입력을 파싱 전에 끊기 위한 기본 크기 상한(256 MiB).
///
/// 현행 표면에는 상한이 **아예 없다** — 신뢰할 수 없는 업로드를 그대로 파서에
/// 넣는다. 자동화·백엔드가 쓰는 표면에서 이건 상한이 아니라 구멍이라, 새 축은
/// 기본값을 켠 채로 시작한다. 무제한이 필요하면 명시적으로 끈다
/// ([`DocumentService::with_max_bytes`]).
pub const DEFAULT_MAX_BYTES: usize = 256 * 1024 * 1024;

/// 제목 추정이 훑는 앞쪽 페이지 수. `info --json` 의 `title` 규칙과 같은 "앞 3쪽".
const TITLE_SCAN_PAGES: u32 = 3;

/// 문서 열기·조회의 진입점. **읽기 전용·결정적**이다.
///
/// 같은 바이트와 같은 옵션은 언제나 같은 결과를 낸다. LLM·네트워크·전역 상태를
/// 쓰지 않으며, 전역 비밀번호를 `thread_local` 로 숨겨 나르지도 않는다
/// (현행 CLI 의 `CLI_PASSWORD` 가 그렇게 한다 — 그 값은 시그니처에 보이지 않아
/// 호출자가 무엇이 적용되는지 알 수 없다).
///
/// 값 타입이므로 복제해서 설정만 다른 서비스를 만들 수 있다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentService {
    max_bytes: Option<usize>,
    title_scan: bool,
}

impl Default for DocumentService {
    fn default() -> Self {
        DocumentService {
            max_bytes: Some(DEFAULT_MAX_BYTES),
            title_scan: true,
        }
    }
}

impl DocumentService {
    /// 기본 설정(상한 [`DEFAULT_MAX_BYTES`], 제목 추정 켬)으로 만든다.
    pub fn new() -> Self {
        DocumentService::default()
    }

    /// 입력 크기 상한을 바꾼다. `None` 이면 무제한.
    ///
    /// [`OpenOptions::max_bytes`] 가 지정되면 호출 단위로 이 값을 덮어쓴다.
    pub fn with_max_bytes(mut self, max_bytes: Option<usize>) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    /// [`DocumentInfo::title`] 추정 여부를 바꾼다.
    ///
    /// 제목 추정은 앞 3쪽의 **텍스트 렌더**를 요구한다. 수백~수천 건을 훑는
    /// 대장화에서는 이 비용이 지배적이므로 끌 수 있어야 한다. 끄면 `title` 은
    /// 언제나 `None` 이다.
    pub fn with_title_scan(mut self, title_scan: bool) -> Self {
        self.title_scan = title_scan;
        self
    }

    /// 현재 적용 중인 기본 크기 상한.
    pub fn max_bytes(&self) -> Option<usize> {
        self.max_bytes
    }

    /// 바이트 버퍼에서 문서를 연다. 모든 표면이 결국 지나가는 유일한 문이다.
    ///
    /// WASM 을 포함한 모든 타깃에서 쓸 수 있다 — 파일 시스템을 건드리지 않는다.
    pub fn open_bytes(
        &self,
        bytes: &[u8],
        opts: &OpenOptions,
    ) -> Result<OpenedDocument, ServiceError> {
        self.open_inner(bytes, opts, DocumentSource::Bytes)
    }

    /// 경로에서 문서를 연다. 읽기 실패는 파싱 실패와 **다른 이름**으로 올라온다.
    ///
    /// 네이티브 타깃 전용이다. WASM 에는 열 파일 시스템이 없으므로
    /// [`DocumentService::open_bytes`] 를 쓴다.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_path(
        &self,
        path: &Path,
        opts: &OpenOptions,
    ) -> Result<OpenedDocument, ServiceError> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ServiceError::NotFound {
                    path: path.to_path_buf(),
                })
            }
            Err(error) => {
                return Err(ServiceError::Io {
                    path: path.to_path_buf(),
                    kind: error.kind(),
                })
            }
        };
        self.open_inner(&bytes, opts, DocumentSource::Path(path.to_path_buf()))
    }

    fn open_inner(
        &self,
        bytes: &[u8],
        opts: &OpenOptions,
        source: DocumentSource,
    ) -> Result<OpenedDocument, ServiceError> {
        // 1) 상한 — 파서를 부르기 전에 끊는다. 손상 입력이 파서를 통과할 기회를
        //    얻지 못하는 것이 상한의 요점이다.
        if let Some(limit) = opts.max_bytes.or(self.max_bytes) {
            if bytes.len() > limit {
                return Err(ServiceError::TooLarge {
                    size_bytes: bytes.len(),
                    limit_bytes: limit,
                });
            }
        }
        // 2) 형식 자동판별 — 열 수 없는 형식은 여기서 끝난다. 열 수 있는 형식이면
        //    이 값을 **보관**한다(소비자가 다시 훑지 않게).
        let format = crate::parser::detect_format(bytes);
        if !is_openable(format) {
            return Err(ServiceError::UnsupportedFormat { detected: format });
        }
        // 3) 비밀번호 유무 분기 — 세 표면이 각자 쓰던 그 분기, 여기 한 번만.
        let loaded = match opts.password.as_deref() {
            Some(password) => DocumentCore::from_bytes_with_password(bytes, password.as_bytes()),
            None => DocumentCore::from_bytes(bytes),
        };
        // 4) 실패에 이름을 붙인다.
        let core = loaded
            .map_err(|error| ServiceError::from_open_failure(&error, opts.password.is_some()))?;
        Ok(OpenedDocument {
            core,
            format,
            size_bytes: bytes.len(),
            source,
            title_scan: self.title_scan,
        })
    }
}

/// 열기 한 번에 적용할 옵션.
///
/// 비밀번호는 **인자로 전달한다**. 현행 CLI 처럼 전역(`thread_local`)에 숨기면
/// 호출 지점만 봐서는 어떤 인증이 적용되는지 알 수 없고, `batch` 처럼 워커
/// 스레드로 갈라지는 경로에서 조용히 사라진다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenOptions {
    /// 암호 문서 비밀번호. `None` 이면 평문 경로로 연다.
    ///
    /// 값은 [`OpenedDocument`] 에 보존되지 않는다 — 열기가 끝나면 잊는다.
    pub password: Option<String>,
    /// 이 호출에만 적용할 크기 상한. `None` 이면 서비스 기본값을 쓴다.
    ///
    /// 무제한으로 열려면 서비스 쪽을 [`DocumentService::with_max_bytes`]`(None)`
    /// 으로 만든다 — 호출 단위 `None` 은 "지정 안 함"이지 "무제한"이 아니다.
    pub max_bytes: Option<usize>,
}

impl OpenOptions {
    /// 비밀번호 없이, 서비스 기본 상한으로 여는 옵션.
    pub fn new() -> Self {
        OpenOptions::default()
    }

    /// 비밀번호를 붙인다.
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// 이 호출에만 적용할 크기 상한을 붙인다.
    pub fn with_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }
}

/// 문서가 어디서 왔는지. 봉투의 `source` 자리에 그대로 쓸 수 있다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentSource {
    /// 경로에서 읽었다.
    Path(PathBuf),
    /// 바이트 버퍼에서 열었다(WASM 업로드·MCP 인메모리 등). 이름은 소비자가 붙인다 —
    /// MCP 는 `docId`, WASM 은 브라우저가 아는 파일명이 그 자리에 온다.
    Bytes,
}

/// 열린 문서 하나. 파싱된 IR 과 **열 때 알아낸 사실**(형식·원본 크기·출처)을 함께 든다.
///
/// 읽기 전용이다. 편집이 필요하면 [`OpenedDocument::into_core`] 로 소유권을 가져가
/// `DocumentCore` 를 직접 쓴다 — 편집·저장은 이 축의 범위가 아니다.
pub struct OpenedDocument {
    pub(crate) core: DocumentCore,
    format: FileFormat,
    size_bytes: usize,
    source: DocumentSource,
    title_scan: bool,
}

impl std::fmt::Debug for OpenedDocument {
    /// `DocumentCore` 는 문서 IR 전체라 `Debug` 출력이 사실상 문서 덤프다.
    /// 로그에 문서 내용이 통째로 흘러나가지 않도록 메타만 찍는다.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenedDocument")
            .field("format", &self.format)
            .field("size_bytes", &self.size_bytes)
            .field("source", &self.source)
            .field("page_count", &self.core.page_count())
            .finish()
    }
}

impl OpenedDocument {
    /// 매직 바이트로 감지한 원본 형식.
    ///
    /// 소비자가 `detect_format` 을 다시 부를 이유가 없다 — 그 재호출이 지금
    /// `main.rs` 에 24곳, `mcp_serve.rs` 에 1곳 있다.
    pub fn format(&self) -> FileFormat {
        self.format
    }

    /// 원본 바이트 크기.
    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    /// 문서 출처.
    pub fn source(&self) -> &DocumentSource {
        &self.source
    }

    /// 조판까지 끝난 문서 코어(읽기 전용).
    pub fn core(&self) -> &DocumentCore {
        &self.core
    }

    /// 문서 IR(읽기 전용).
    pub fn document(&self) -> &Document {
        &self.core.document
    }

    /// 총 페이지 수.
    pub fn page_count(&self) -> u32 {
        self.core.page_count()
    }

    /// 코어의 소유권을 가져간다 — 편집·저장으로 넘어가는 탈출구.
    ///
    /// 이 축은 읽기 전용이므로 변이 API 를 제공하지 않는다. 편집이 필요한
    /// 소비자(MCP 세션 도구 등)는 여기서 코어를 받아 기존 경로로 계속한다.
    pub fn into_core(self) -> DocumentCore {
        self.core
    }

    /// 문서 메타 — 쪽수·형식·암호화 등.
    ///
    /// **왜 이 함수가 필요한가**: 같은 질문에 지금 두 개의 답이 있다.
    /// `main.rs::info_json_value` 는 글꼴을 선언 순서대로 중복까지 보존해서 주고,
    /// `DocumentCore::get_document_info`(WASM 이 쓰는 쪽)는 `BTreeSet` 으로
    /// 정렬·중복 제거하고 대체 글꼴까지 해소해서 준다. 어느 표면에 물었느냐로
    /// 답이 달라지는 필드는 계약이 아니다.
    ///
    /// 이 축은 **`info --json` 의 어휘로 통일한다** — 그쪽이 이미 문서화된 봉투
    /// 계약(`schemaVersion` 이 걸린)이고, 문서 인벤토리라는 목적상 "선언된 순서와
    /// 중복"이 정보이기 때문이다.
    pub fn info(&self) -> DocumentInfo {
        let document = self.document();
        let version = if self.format == FileFormat::Hml {
            // HML 에는 HWP 바이너리 버전이 없다. 0.0.0.0 을 지어내지 않는다.
            None
        } else {
            Some(format!(
                "{}.{}.{}.{}",
                document.header.version.major,
                document.header.version.minor,
                document.header.version.build,
                document.header.version.revision,
            ))
        };
        // 선언된 모든 글꼴군(한글·영어·한자·일어·기타·기호·사용자)을 문서 순서대로
        // 평탄화한다. 중복은 남긴다 — 어느 군에서 왔는지가 소비자에게 정보다.
        let fonts: Vec<String> = document
            .doc_info
            .font_faces
            .iter()
            .flatten()
            .map(|face| face.name.clone())
            .collect();
        let para_count: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
        DocumentInfo {
            format: format_token(self.format),
            size_bytes: self.size_bytes,
            version,
            sections: document.sections.len(),
            page_count: self.core.page_count(),
            para_count,
            encrypted: document.header.encrypted,
            fonts,
            title: if self.title_scan {
                self.guess_title()
            } else {
                None
            },
        }
    }

    /// 앞쪽 몇 쪽의 첫 의미 줄을 제목으로 추정한다. best-effort 이며 계약이 아니다.
    fn guess_title(&self) -> Option<String> {
        for page in 0..self.core.page_count().min(TITLE_SCAN_PAGES) {
            let Ok(text) = self.core.extract_page_text_native(page) else {
                // 표지가 깨진 문서 하나 때문에 메타 조회 전체가 실패하면 안 된다.
                continue;
            };
            if let Some(line) = text.lines().map(str::trim).find(|l| !l.is_empty()) {
                return Some(line.to_string());
            }
        }
        None
    }
}

/// 문서 메타 한 벌. 봉투에 그대로 실을 수 있도록 camelCase 로 직렬화한다.
///
/// `schemaVersion`·`source` 는 **일부러 빼 두었다** — 스키마 버전은 봉투를 만드는
/// 표면의 몫이고, `source` 는 표면마다 다른 이름(경로·`docId`·업로드 파일명)이
/// 들어가야 하기 때문이다.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    /// 형식 토큰 — `hwp5`·`hwpx`·`hwp3`·`hml`.
    pub format: &'static str,
    /// 원본 바이트 크기.
    pub size_bytes: usize,
    /// HWP 버전 문자열(`major.minor.build.revision`). HML 은 `None`.
    pub version: Option<String>,
    /// 구역 수.
    pub sections: usize,
    /// 총 페이지 수.
    pub page_count: u32,
    /// 총 문단 수(본문 기준).
    pub para_count: usize,
    /// 파일 헤더가 밝힌 암호화 여부.
    pub encrypted: bool,
    /// 선언된 글꼴 이름(문서 순서, 중복 보존).
    pub fonts: Vec<String>,
    /// 추정 제목. 없거나 추정을 끄면 `None`.
    pub title: Option<String>,
}

/// 형식 → 봉투 토큰. `info --json` 의 `format` 어휘와 같은 문자열이다.
pub fn format_token(format: FileFormat) -> &'static str {
    match format {
        FileFormat::Hwp => "hwp5",
        FileFormat::Hwpx => "hwpx",
        FileFormat::Hwp3 => "hwp3",
        FileFormat::Hml => "hml",
        FileFormat::DrmProtected => "drm-protected",
        FileFormat::Empty => "empty",
        FileFormat::Unknown => "unknown",
    }
}

/// rhwp 가 문서로 여는 형식인가.
///
/// DRM 컨테이너·빈 파일·미상 바이트를 파서에 넘기지 않는 게 요점이다. 파서도
/// 결국 거절하지만, 거절의 **이름**이 형식 판정에서 나와야 소비자가 "지원하지
/// 않는 형식"과 "손상된 문서"를 가를 수 있다.
fn is_openable(format: FileFormat) -> bool {
    matches!(
        format,
        FileFormat::Hwp | FileFormat::Hwpx | FileFormat::Hwp3 | FileFormat::Hml
    )
}
