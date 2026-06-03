# Task M100-1053 구현 계획서 — 미지원 파일 오류코드 정정

## 1. 구현 방침

이번 작업은 "지원하지 않는 파일을 파싱하려고 시도하다가 저수준 오류를 노출하는"
흐름을 포맷 감지 단계에서 차단하는 데 집중한다.

핵심 방침:

- 포맷 감지(`detect_format`)에서 legacy HWPML을 별도 식별한다.
- `parse_document()`는 알 수 없는 포맷을 HWP 파서로 보내지 않고 명시적 미지원 오류를 반환한다.
- 오류에는 안정적인 코드와 사용자용 한국어 설명을 함께 싣는다.
- 기존 HWP 5.0/HWPX/HWP3 경로는 최대한 건드리지 않는다.

## 2. 수정 대상

| 파일 | 변경 내용 |
|------|-----------|
| `src/parser/mod.rs` | legacy HWPML 감지, 미지원 포맷 오류코드, `parse_document()` 분기, 단위 테스트 |
| `src/error.rs` | `UnsupportedFormat` 표시 문자열 보존 테스트 갱신 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 문서 로드 실패 시 내부 문서 참조와 파일 핸들 상태 초기화 |
| `rhwp-studio/src/main.ts` | 기존 `showLoadError()` 사용자 알림 흐름 유지 확인. 필요 시 주석/테스트만 보강 |
| `mydocs/working/task_m100_1053_stage1.md` | 구현 후 단계 보고서 |
| `mydocs/report/task_m100_1053_report.md` | 최종 보고서 |

필요 시 추가:

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-studio/e2e/unsupported-format-error.test.mjs` | 브라우저 로드 실패 토스트에 오류코드가 표시되는지 검증 |

## 3. 세부 구현

### 3.1 `FileFormat` 확장

후보:

```rust
pub enum FileFormat {
    Hwp,
    Hwpx,
    Hwp3,
    LegacyHwpml,
    Unknown,
}
```

`LegacyHwpml`은 raw XML 기반 HWPML 계열을 의미한다. 버전 문자열은 별도 helper로
추출하고, 감지 실패 시 generic `HWPML`로 표시한다.

### 3.2 legacy HWPML 감지 helper

후보 함수:

```rust
fn detect_legacy_hwpml(data: &[u8]) -> bool
fn detect_legacy_hwpml_version(data: &[u8]) -> Option<&'static str>
```

초기 구현 기준:

- 앞부분 공백/BOM을 허용한다.
- XML 선언(`<?xml`) 또는 root 토큰 근처에서 `HWPML`을 찾는다.
- `Version="2.1"` 또는 `version="2.1"`이 있으면 `HWPML 2.1`로 표시한다.
- `HWPX` ZIP(`PK\x03\x04`)은 기존 `Hwpx` 분기에 먼저 걸리므로 영향 없음.

UTF-16 XML까지 한 번에 처리할지는 Stage 1 구현 중 실제 fixture 여부에 따라 결정한다.
fixture가 없다면 UTF-8/ASCII 합성 가드로 시작하고, 후속 이슈로 분리 가능하게 둔다.

### 3.3 `ParseError::UnsupportedFormat` 확장

후보:

```rust
pub enum ParseError {
    ...
    UnsupportedFormat {
        code: &'static str,
        format: &'static str,
        hint: &'static str,
    },
}
```

`Display` 목표:

```text
지원하지 않는 포맷입니다: HWPML 2.1. 오류코드: UNSUPPORTED_HWPML. 현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다.
```

기존 Task #265 테스트는 Debug 형식이 노출되지 않는다는 본질을 유지하면서
`code` 필드 포함 여부를 갱신한다.

### 3.4 `parse_document()` 분기 정정

현재:

```rust
match detect_format(data) {
    FileFormat::Hwpx => HwpxParser.parse(data),
    FileFormat::Hwp3 => Hwp3Parser.parse(data),
    _ => HwpParser.parse(data),
}
```

계획:

```rust
match detect_format(data) {
    FileFormat::Hwp => HwpParser.parse(data),
    FileFormat::Hwpx => HwpxParser.parse(data),
    FileFormat::Hwp3 => Hwp3Parser.parse(data),
    FileFormat::LegacyHwpml => Err(ParseError::UnsupportedFormat {
        code: "UNSUPPORTED_HWPML",
        format: detect_legacy_hwpml_version(data).unwrap_or("HWPML"),
        hint: "현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다. 한컴오피스에서 HWP 5.0 또는 HWPX로 다시 저장한 뒤 열어주세요.",
    }),
    FileFormat::Unknown => Err(ParseError::UnsupportedFormat {
        code: "UNSUPPORTED_FILE_FORMAT",
        format: "알 수 없는 파일 형식",
        hint: "현재 rhwp는 HWP 5.0, HWPX, 일부 HWP 3.0 문서만 지원합니다.",
    }),
}
```

주의: HWP 5.0 CFB 시그니처는 계속 HWP 파서로 보내야 한다. 즉 손상된 HWP 5.0은
지원 포맷 내부 오류로 남기고, signature 자체를 모르는 파일만 generic unsupported로 처리한다.

### 3.5 rhwp-studio 로드 실패 상태 초기화

작업지시자 추가 요구:

```text
지원되지 않는 문서 로딩시 오류가 발생할 경우 기존 사용자 알림 UI 방식으로 처리하고,
다음 문서 정상 처리를 위해 초기화도 해야 합니다.
```

현재 `main.ts`는 로드 실패 시 이미 `showLoadError()`를 호출한다. 따라서 새 UI를
추가하지 않고 기존 토스트/상태 표시줄 흐름을 유지한다.

정정 대상은 `rhwp-studio/src/core/wasm-bridge.ts::loadDocument()`다.

현재 위험:

```typescript
if (this.doc) {
  this.doc.free();
}
this._fileName = fileName ?? 'document.hwp';
this._currentFileHandle = null;
this.doc = new HwpDocument(data); // 여기서 throw 가능
```

계획:

- 기존 문서를 해제한 직후 `this.doc = null`로 명시 초기화한다.
- 새 `HwpDocument(data)` 생성과 후속 `convertToEditable()`/초기화 단계는 `try` 안에서 수행한다.
- 실패 시 생성 중인 문서가 있다면 `free()` 후 `this.doc = null`, `_currentFileHandle = null`로 정리하고 오류를 그대로 throw한다.
- 오류는 상위 `loadFile()`/`loadRemoteDocument()`의 기존 `showLoadError()`가 처리하게 둔다.
- 정상 로드 성공 시에만 `this.doc`, `_fileName`, 파일명 설정, 외부 이미지 fetch를 확정한다.

후보 구조:

```typescript
loadDocument(data: Uint8Array, fileName?: string): DocumentInfo {
  this.releaseDocument();
  const nextFileName = fileName ?? 'document.hwp';
  let nextDoc: HwpDocument | null = null;
  try {
    nextDoc = new HwpDocument(data);
    nextDoc.convertToEditable();
    this.doc = nextDoc;
    this._fileName = nextFileName;
    this.doc.setFileName(this._fileName);
    ...
    return info;
  } catch (error) {
    try { nextDoc?.free(); } catch { /* noop */ }
    this.doc = null;
    this._currentFileHandle = null;
    throw error;
  }
}
```

세부 구현 시 `ensureParagraphStableIds()`가 `this.doc`를 필요로 하므로,
`nextDoc`을 `this.doc`에 대입하는 위치를 안전하게 조정한다.

## 4. 테스트 계획

`src/parser/mod.rs` 단위 테스트 후보:

1. `test_detect_format_legacy_hwpml_21`
   - 합성 XML:
     ```xml
     <?xml version="1.0" encoding="UTF-8"?>
     <HWPML Version="2.1"></HWPML>
     ```
   - 기대: `FileFormat::LegacyHwpml`
2. `test_parse_document_legacy_hwpml_returns_unsupported_code`
   - 기대: `ParseError::UnsupportedFormat { code: "UNSUPPORTED_HWPML", format: "HWPML 2.1", ... }`
3. `test_parse_document_unknown_returns_unsupported_file_format`
   - 기대: `UNSUPPORTED_FILE_FORMAT`, CFB 오류 문자열 없음
4. 기존 감지 회귀:
   - HWP CFB → `FileFormat::Hwp`
   - HWPX ZIP → `FileFormat::Hwpx`
   - HWP3 prefix → `FileFormat::Hwp3`

`src/error.rs` 테스트:

- `ParseError::UnsupportedFormat` → `HwpError` → `Display` 경로에서
  - 오류코드 포함
  - format/hint 포함
  - `UnsupportedFormat` 또는 `{ format` 같은 Debug 표현 미노출

필요 시 rhwp-studio E2E:

- 합성 `.hwpml` 파일을 브라우저 input에 주입
- 토스트/상태 표시줄에 `UNSUPPORTED_HWPML`과 `HWPML 2.1`이 보이는지 확인
- 이어서 정상 샘플을 로드해 문서 정보/페이지 렌더링이 성공하는지 확인

## 5. 검증 명령

기본 Rust 검증:

```bash
cargo test --release --lib
```

프로젝트 관례상 전체 테스트가 필요하면:

```bash
node --experimental-strip-types --test tests/*.test.ts
```

rhwp-studio 표시 확인이 필요하면:

```bash
cd rhwp-studio
npm run build
node e2e/unsupported-format-error.test.mjs --mode=headless
```

## 6. 리스크와 대응

| 리스크 | 대응 |
|--------|------|
| XML HWPML 인코딩이 UTF-16/EUC-KR인 경우 초기 감지가 누락될 수 있음 | Stage 1에서 fixture 확인. 없으면 UTF-8/ASCII 토큰 감지부터 시작하고 후속 확장 여지 문서화 |
| `Unknown`을 HWP 파서로 보내던 기존 동작 변경으로 테스트 기대값이 달라짐 | 변경 목적이 #1053의 본질이므로 단위 테스트로 새 contract 고정 |
| `UnsupportedFormat` 필드 확장으로 기존 테스트/생성자 수정 필요 | 검색 후 모든 생성 지점 갱신. Debug 누출 방지 테스트 유지 |
| JS 경계에서 오류코드가 구조화 객체가 아니라 문자열로만 전달됨 | 이번 범위에서는 안정 코드 문자열 노출을 contract로 둔다. 별도 구조화 오류 API는 후속 task 후보 |

## 7. 승인 요청

본 구현 계획 승인 후 다음 순서로 진행한다.

1. `src/parser/mod.rs`에 legacy HWPML 감지와 미지원 오류 분기 추가
2. `src/error.rs` 테스트 갱신
3. Rust 테스트 실행
4. 필요 시 rhwp-studio 빌드/E2E 확인
5. 단계 보고서와 최종 보고서 작성
