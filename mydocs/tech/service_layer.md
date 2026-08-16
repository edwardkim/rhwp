---
kind: reference
status: active
canonical: mydocs/tech/service_layer.md
last_verified: 2026-08-16
---

# 서비스 연계 표면 — `src/service`

CLI·MCP·WASM 이 **함께 서는** 문서 열기·조회 축의 계약 문서다. 구현 정본은
`src/service/`(`mod.rs`·`error.rs`·`open.rs`·`query.rs`)이고, 이 문서는 그 계약과
채택 경로를 서술한다.

## 1. 왜 필요한가

`ROADMAP.md` 의 "rhwp가 공식적으로 맡는 범위" 표는 **서비스 연계 표면**을 업스트림
책임으로 명시한다.

> 서비스 연계 표면 | 자동화와 백엔드 서비스가 사용할 CLI의 기계 판독 출력, MCP 서버와
> 공개 API 계약

그런데 그 계약을 담을 공통 모듈이 없었다. 그래서 계약의 첫 네 걸음(**열기 · 메타 조회 ·
검색 · 텍스트 내보내기**)이 표면마다 다시 쓰여 있다. 아래는 devel(441254611) 기준 실측이다.

| 하는 일 | CLI `src/main.rs` | MCP `src/mcp_serve.rs` | WASM `src/wasm_api.rs` / 코어 |
|---|---|---|---|
| 바이트 읽고 실패 보고 | `fs::read` + `"파일을 읽을 수 없습니다"` 블록 **45곳** (첫 곳 `main.rs:4714`) | `session_open` `mcp_serve.rs:1421` | 호출자(JS) 몫 |
| 비밀번호 유무 분기 | `load_document` `main.rs:115`, `load_document_core` `main.rs:124` | `session_open` `mcp_serve.rs:1429` | `HwpDocument::from_bytes*` `wasm_api.rs:433`·`437` |
| 실패를 갈래로 나누기 | `classify_hwp_error` `main.rs:104` — **한국어 문장 부분일치** | 없음. 전부 `"{path} 파싱 실패"` `mcp_serve.rs:1435` | 없음. `HwpError` 를 `JsValue` 로 통째 전달 |
| 형식 재판별 | `rhwp::parser::detect_format` **24곳** (예 `main.rs:4722`·`4963`·`5472`) | `mcp_serve.rs:1438` | `DocumentCore::from_bytes_inner` `document.rs:75` 가 계산하고 **폐기** |
| 메타 산출 | `info_json_value` `main.rs:10314` | 같은 함수 재사용 `mcp_serve.rs:1680` | `DocumentCore::get_document_info` `document_core/mod.rs:258` — **필드·글꼴 규칙이 다름** |
| 검색 엔진 | `grep` `main.rs:9936`, `grep_with_context` `main.rs:13323` | `grep` `mcp_serve.rs:1791` | `search_all_text_native` `wasm_api.rs:5115` — **엔진 자체가 다름** |
| 쪽 텍스트 추출 루프 | `main.rs:6618`·`10569`·`10651`·`10700` | `mcp_serve.rs:215`·`1598` | `extract_page_text_native` 직접 호출 |

### 1.1 중복의 대가 — 같은 질문, 다른 답

숫자보다 중요한 것은 마지막 세 줄이다.

- **"이 문서가 쓰는 글꼴은?"** `info --json`(`main.rs:10346`)은 선언된 글꼴군을 문서
  순서대로, 중복까지 보존해 준다. WASM 이 쓰는 `get_document_info`
  (`document_core/mod.rs:261`)는 `BTreeSet` 으로 정렬·중복 제거하고 대체 글꼴까지
  해소해서 준다. 같은 문서에 두 답이 있다.
- **"이 단어가 어디 있는가?"** CLI·MCP 는 좌표(구역·문단·**쪽**·문자 오프셋)가 붙은
  `GrepMatch` 를 준다. WASM 은 좌표 어휘가 다른 `search_all_text_native` 의 산출을 준다.
- **"왜 못 열었는가?"** CLI 만 갈래를 나눈다. 그 갈래조차 한국어 문장 부분일치다.

  ```rust
  // src/main.rs:104 classify_hwp_error
  if msg.contains("비밀번호가 일치하지 않") { LoadError::WrongPassword }
  else if msg.contains("비밀번호가 필요한 암호 문서") { LoadError::NeedPassword }
  ```

  이 판정이 exit code 를 정한다. 문장 한 글자가 바뀌면 종료 코드가 조용히 바뀌고,
  그 판정은 CLI 안에만 있어 MCP·WASM 은 갈래를 아예 잃는다.

**어느 표면에 물었느냐로 답이 달라지는 값은 계약이 아니다.** `src/service` 는 그
답을 한 번만 정의하기 위한 축이다.

### 1.2 이 PR 의 경계

이 PR 은 **축을 세우기만 한다**. `main.rs`·`mcp_serve.rs`·`wasm_api.rs` 는 한 줄도
고치지 않았다. 세 소비자의 이관은 4장의 시나리오대로 후속 PR 에서 표면별로 진행한다.
`src/lib.rs` 변경은 `pub mod service;` **한 줄**이다.

## 2. 공개 API

```rust
// 진입점 — 설정을 든 값 타입. 복제해서 설정만 다른 서비스를 만든다.
pub struct DocumentService { /* max_bytes, title_scan */ }
impl DocumentService {
    pub fn new() -> Self;
    pub fn with_max_bytes(self, max_bytes: Option<usize>) -> Self;
    pub fn with_title_scan(self, title_scan: bool) -> Self;
    pub fn max_bytes(&self) -> Option<usize>;

    pub fn open_bytes(&self, bytes: &[u8], opts: &OpenOptions)
        -> Result<OpenedDocument, ServiceError>;
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_path(&self, path: &Path, opts: &OpenOptions)
        -> Result<OpenedDocument, ServiceError>;
}

pub const DEFAULT_MAX_BYTES: usize = 256 * 1024 * 1024;

pub struct OpenOptions { pub password: Option<String>, pub max_bytes: Option<usize> }

pub struct OpenedDocument { /* DocumentCore + FileFormat + size_bytes + DocumentSource */ }
impl OpenedDocument {
    pub fn format(&self) -> FileFormat;
    pub fn size_bytes(&self) -> usize;
    pub fn source(&self) -> &DocumentSource;
    pub fn core(&self) -> &DocumentCore;
    pub fn document(&self) -> &Document;
    pub fn page_count(&self) -> u32;
    pub fn into_core(self) -> DocumentCore;          // 편집으로 넘어가는 탈출구

    pub fn info(&self) -> DocumentInfo;
    pub fn search(&self, query: &str, opts: &SearchOptions) -> SearchOutcome;
    pub fn export_text(&self, opts: &TextExportOptions) -> TextExport;
}

pub fn format_token(format: FileFormat) -> &'static str;  // "hwp5"|"hwpx"|"hwp3"|"hml"|…
```

산출 타입은 전부 `Serialize`(camelCase)라 봉투에 그대로 실린다.

| 타입 | 담는 것 |
|---|---|
| `DocumentInfo` | `format`·`sizeBytes`·`version`·`sections`·`pageCount`·`paraCount`·`encrypted`·`fonts`·`title` |
| `SearchOutcome` | `query`·`caseSensitive`·`matchCount`·`totalMatchCount`·`truncated`·`omittedCount`·`offset`·`nextOffset`·`matches[]`(좌표 포함) |
| `TextExport` | `pageCount`·`truncated`·`omittedCount`·`charOffset`·`nextOffset`·`outOfRange[]`·`pages[]` |
| `PageText` | `page`·`text`·`truncated`·`omittedCount`·`extractError` |

`schemaVersion` 과 `source` 는 **일부러 뺐다**. 스키마 버전은 봉투를 만드는 표면의
몫이고, `source` 자리에는 표면마다 다른 이름(경로·`docId`·업로드 파일명)이 와야 한다.

### 2.1 스케치에서 바꾼 것과 그 이유

| 스케치 | 채택형 | 이유 |
|---|---|---|
| `ServiceError::Encrypted` 하나 | `PasswordRequired` / `PasswordMismatch` 둘 | 현행 CLI 가 이미 두 갈래를 서로 다른 exit code(USAGE/RUNTIME)로 매핑한다. 하나로 뭉치면 소비자가 다시 문자열을 갈라 읽어야 한다. |
| `NotFound` (payload 없음) | `NotFound { path }` + `Io { path, kind }` 신설 | "파일이 없다"(경로를 고치면 됨)와 "읽을 수 없다"(권한·환경)는 조치가 다르다. |
| `UnsupportedFormat` (payload 없음) | `UnsupportedFormat { detected }` | DRM 컨테이너·빈 파일·미상 바이트는 사용자가 할 수 있는 조치가 다르다. 감지 결과를 버리지 않는다. |
| — | `DocumentService::with_title_scan` | `title` 추정은 앞 3쪽 **텍스트 렌더**를 요구한다. 수천 건 대장화에서 비용이 지배적이라 끌 수 있어야 한다. |
| — | `DEFAULT_MAX_BYTES` 기본 켬 | 현행 세 표면에 크기 상한이 **하나도 없다**. 자동화·백엔드가 쓰는 표면에서 그건 구멍이다. |

## 3. 오류 타입 계약

```rust
pub enum ServiceError {
    NotFound { path: PathBuf },
    Io { path: PathBuf, kind: std::io::ErrorKind },
    UnsupportedFormat { detected: FileFormat },
    PasswordRequired,
    PasswordMismatch,
    TooLarge { size_bytes: usize, limit_bytes: usize },
    Parse(String),
}
```

| 변형 | `code()` | `is_usage_error()` | `needs_password()` | 권장 exit code |
|---|---|---|---|---|
| `NotFound` | `NOT_FOUND` | true | false | 2 (USAGE) |
| `Io` | `IO` | false | false | 1 (RUNTIME) |
| `UnsupportedFormat` | `UNSUPPORTED_FORMAT` | true | false | 2 (USAGE) |
| `PasswordRequired` | `PASSWORD_REQUIRED` | true | true | 2 (USAGE) |
| `PasswordMismatch` | `PASSWORD_MISMATCH` | false | true | 1 (RUNTIME) |
| `TooLarge` | `TOO_LARGE` | true | false | 2 (USAGE) |
| `Parse` | `PARSE` | false | false | 1 (RUNTIME) |

`code()` 문자열이 **계약**이다. 토큰 추가는 허용, 변경·삭제는 소비자를 깨뜨린다.
`Display` 는 사람에게 보일 한국어 문장이며 **판정의 근거가 아니다**.

### 3.1 판정은 오류가 아니다

`Ok` 안에 담기고 `Err` 로 올라오지 **않는** 것들이다.

| 상황 | 표현 |
|---|---|
| 매치 0건 | `SearchOutcome { total_match_count: 0, .. }` · `is_empty() == true` |
| 빈 검색어 | 위와 같음(0건). 빈 검색어 거절은 인자 검증이고, 그건 표면의 몫이다. |
| 범위 밖 쪽 요청 | `TextExport::out_of_range: Vec<u32>` |
| 쪽 텍스트 추출 실패 | `PageText::extract_error: Option<String>` — 그 쪽도 목록에서 **빼지 않는다** |
| 절단 | `truncated`·`omitted_count`·`next_offset` |

쪽 항목을 빼지 않는 규칙은 이 저장소가 이미 지키는 것이다(#3787 S7·#4854). 빼면
`pageCount` 가 줄어 문서가 실제보다 짧아 보이고, "빈 쪽"과 "못 읽은 쪽"이 같은
모습이 된다.

### 3.2 타입 복원 경계 — 알려진 이음매

`DocumentCore::from_bytes` 는 타입 있는 `ParseError` 를
`HwpError::InvalidFile(String)` 으로 **평탄화해서** 돌려준다
(`document_core/commands/document.rs:80`). 이 PR 은 기존 파일을 고치지 않으므로 그
평탄화를 되돌릴 수 없고, 타입 복원은 `ServiceError::from_open_failure` 한 곳에서만
일어난다.

다만 `main.rs` 처럼 한국어 문장을 **상수로 박지 않는다**. 대조할 바늘을 그 자리에서
타입으로부터 만든다.

```rust
if inner.contains(&CryptoError::WrongPassword.to_string())        { PasswordMismatch }
if inner.contains(&ParseError::EncryptedDocument.to_string())     { PasswordRequired }
```

업스트림이 문구를 고치면 바늘도 같이 따라가므로, 문구 변경이 exit code 를 조용히
뒤집는 사고가 나지 않는다. HWP5·HWPX·HWP3 의 비밀번호 불일치는 세 형식 모두 같은
문장을 감싸 내보내므로(`암호 오류: …` / `HWPX 오류: …` / `HWP 3.0 오류: …`) 부분
일치 하나로 셋을 덮는다.

또 하나 바로잡은 갈래: **비밀번호를 주었는데도** "암호 문서" 오류가 오면 그건
"비밀번호를 주세요"가 아니라 "그 비밀번호가 틀렸다"이다. 현행 CLI 는 이 경우에도
`--password <pw> 로 전달` 을 출력해 이미 준 것을 다시 요구한다.

**후속**: `DocumentCore::from_bytes` 계열이 타입 있는 오류를 돌려주게 되면 이 이음매는
사라진다. 그 변경은 기존 소비자 전부를 건드리므로 이 PR 의 범위가 아니다.

## 4. 채택 시나리오

### 4.1 CLI (`src/main.rs`)

현행 명령 하나의 앞머리는 이렇게 생겼다(45곳 반복).

```rust
let data = match fs::read(file_path) {
    Ok(d) => d,
    Err(e) => { eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e); return EXIT_RUNTIME; }
};
let source_format = rhwp::parser::detect_format(&data);   // 코어가 이미 계산했다가 버린 값
let doc = match load_document(&data) { Ok(d) => d, Err(e) => return e.report() };
```

이관 후.

```rust
let opened = match SERVICE.open_path(Path::new(file_path), &open_opts) {
    Ok(opened) => opened,
    Err(error) => {
        eprintln!("오류: {error}");
        return if error.is_usage_error() { EXIT_USAGE } else { EXIT_RUNTIME };
    }
};
let source_format = opened.format();     // 재판별 없음
```

- `classify_hwp_error`·`LoadError`·`CLI_PASSWORD` thread-local 전역이 사라진다.
  비밀번호는 `OpenOptions` 로 흘러 시그니처에 보인다 — `batch` 가 워커 스레드로
  갈라질 때 인증이 조용히 사라지던 함정(`is_batch_invocation` 이 네 옵션을 통째로
  거부하는 이유)이 구조적으로 없어진다.
- `info_json_value` 는 `opened.info()` 를 감싸 `schemaVersion`·`source`·`warnings` 만
  얹는 얇은 함수가 된다.
- `export-text --json` 은 `opened.export_text(...)` 산출에 껍데기만 붙인다.
- **무회귀 조건**: 봉투 필드 이름·순서는 이미 맞춰 두었다. 이관 PR 은 기존 계약
  테스트를 그대로 통과해야 하며, 통과가 곧 등가 증명이다.

### 4.2 MCP (`src/mcp_serve.rs`)

`session_open` 은 서비스 호출 + 핸들 등록으로 줄어든다.

```rust
let opened = match SERVICE.open_path(Path::new(path), &opts) {
    Ok(opened) => opened,
    Err(error) => return tool_error_typed(&error),   // code + nextCall 을 타입에서
};
let detected_format = opened.format();               // 재판별 없음
let size_bytes = opened.size_bytes();
sessions.insert(SessionDoc { core: opened.into_core(), detected_format, size_bytes, .. });
```

가장 큰 이득은 **오류 갈래를 얻는 것**이다. 지금 MCP 는 모든 열기 실패를
`"{path} 파싱 실패: {e}"` 하나로 뭉친다. `ServiceError::code()` 를 봉투의 `code` 로,
`needs_password()` 를 `nextCall`(같은 도구 + `password`) 제안 조건으로 쓰면
에이전트가 실패에서 다음 수를 읽을 수 있다. 세션 조회 도구(`hwp_doc_search`·
`hwp_doc_text`)의 창(offset/limit) 계산은 `SearchOptions`/`TextExportOptions` 로
대체된다 — 지금은 CLI 쪽 헬퍼를 `crate::` 로 빌려 쓰는데, 그 경로는 **바이너리
크레이트 안에서만** 성립해서 WASM 은 애초에 낄 수 없었다.

### 4.3 WASM (`src/wasm_api.rs`)

WASM 은 `#[wasm_bindgen]` 표면이라 값을 JSON 문자열로 넘긴다. 서비스 산출이 전부
`Serialize` 이므로 그 벽을 그대로 통과한다.

```rust
#[wasm_bindgen(js_name = openDocument)]
pub fn open_document(bytes: &[u8], password: Option<String>) -> Result<String, JsValue> {
    let opts = OpenOptions { password, max_bytes: None };
    match SERVICE.open_bytes(bytes, &opts) {
        Ok(opened) => Ok(serde_json::to_string(&opened.info()).unwrap()),
        // 브라우저가 error.code 로 분기한다 — 문장을 파싱하지 않는다.
        Err(error) => Err(JsValue::from_str(&format!(r#"{{"code":"{}"}}"#, error.code()))),
    }
}
```

- `open_path` 는 `#[cfg(not(target_arch = "wasm32"))]` 이라 WASM 빌드에 들어가지
  않는다. WASM 은 `open_bytes` 만 쓴다.
- `getDocumentInfo` 는 `DocumentInfo` 로 수렴한다 — 1.1 의 "글꼴 두 답"이 여기서
  닫힌다. **호환 주의**: 현행 `get_document_info` 의 `fontsUsed`(정렬·중복 제거·대체
  해소)와 `fontSubstitutions` 는 렌더러가 쓰는 값이므로, 이관 PR 은 `DocumentInfo`
  에 `fonts` 를 두고 렌더 전용 필드는 별도 질의로 남기거나 필드를 추가해야 한다.
  같은 이름으로 다른 값을 주는 것만 금지한다.
- `searchAllText` 는 좌표가 붙는 `search` 로 수렴한다. 브라우저 뷰어가 "몇 쪽"에
  답할 수 있게 되는 부수 효과가 있다.

### 4.4 이관 순서 제안

1. MCP `session_open` — 표면이 가장 작고, 오류 갈래 이득이 가장 크다.
2. CLI `info`·`export-text`·`search` 3개 명령 — 봉투 계약 테스트가 등가를 지킨다.
3. CLI 나머지 명령의 `fs::read` + `load_document` 블록 일괄 치환.
4. WASM `getDocumentInfo`·`searchAllText` — 브라우저 호환 표면이라 마지막.

## 5. 비범위

이 축이 **다루지 않는** 것.

- **편집·저장.** 읽기 전용이다. 편집이 필요하면 `into_core()` 로 코어를 가져가
  기존 경로로 계속한다.
- **렌더링**(SVG/PNG/PDF), **조판 옵션**(DPI·조판부호·디버그 오버레이).
- **MCP 세션 수명**(`docId` 발급·만료·저널) — 세션은 MCP 의 정책이다.
- **봉투 껍데기** — `schemaVersion`·`source`·`untrustedContent` 출처 표지는 표면이
  붙인다(`mydocs/tech/envelope_provenance.md`).
- **인자 파싱·exit code 결정.** 서비스는 판정을 주고, 정책은 표면이 정한다.
- **LLM·네트워크·전역 상태.** 결정적이고 부수효과가 없다.

## 6. 검증

`src/service/mod.rs` 의 `#[cfg(test)] mod tests` 16건. 샘플 경로는
`env!("CARGO_MANIFEST_DIR")` 기준이라 작업 디렉터리에 의존하지 않는다.

| 테스트 | 덮는 계약 |
|---|---|
| `open_path_reads_and_parses_hwpx_sample` | 열기 성공·형식·원본 크기·출처 보존 |
| `format_is_auto_detected_from_bytes_alone` | 확장자 없이 바이트만으로 HWP/HWPX 판별 |
| `missing_path_is_not_found_not_parse_failure` | `NotFound` + `code()` + `is_usage_error()` |
| `unrecognized_bytes_are_unsupported_format` | `UnsupportedFormat{Unknown}` · `{Empty}` 구분 |
| `size_limit_rejects_before_parsing` | `TooLarge`, 호출 단위 상한이 기본값을 덮어씀 |
| `info_reports_format_size_and_counts` | 메타 필드 + camelCase 직렬화 어휘 |
| `title_scan_can_be_disabled` | 기능 플래그, 나머지 메타의 결정성 |
| `export_text_keeps_one_entry_per_page` | 쪽 주소 보존 |
| `export_text_truncation_reports_omission_and_next_offset` | 절단 어휘 |
| `export_text_out_of_range_page_is_data_not_error` | 범위 밖 쪽 = 데이터 |
| `search_returns_matches_with_coordinates` | 검색 매치 + 좌표 |
| `search_without_match_is_ok_not_error` | 0건·빈 검색어 = `Ok` |
| `search_window_walks_all_matches_without_moving_the_total` | offset/limit 창, 총량 불변 |
| `error_codes_and_severity_are_stable` | `code()`·`is_usage_error()`·`needs_password()` 표 |
| `open_failure_classification_derives_needles_from_types` | 3.2 의 바늘이 타입에서 나옴 |
| `service_is_deterministic_and_read_only` | 같은 입력 → 같은 산출, 원본 무훼손 |

## 관련 문서

- `ROADMAP.md` — "rhwp가 공식적으로 맡는 범위" 표
- [`mydocs/tech/envelope_provenance.md`](envelope_provenance.md) — 봉투 출처 표지 계약
- [`mydocs/manual/cli_commands.md`](../manual/cli_commands.md) — CLI 명령 레퍼런스
- [`mydocs/manual/mcp_integration_guide.md`](../manual/mcp_integration_guide.md) — MCP 통합 절차
