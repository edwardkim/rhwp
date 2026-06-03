---
타스크: #265 HWP 3.0 파일 감지 + 친절한 에러 메시지
단계: Stage 2 — ParseError::UnsupportedFormat + parse() 디스패치
브랜치: local/task265
작성일: 2026-04-24
---

# Stage 2 완료 보고서

## 1. 목적

`ParseError::UnsupportedFormat { format, hint }` variant 추가 + `parse_document` 에서 `Hwp3` 감지 시 해당 에러 반환. `samples/issue_265.hwp` 실파일 통합 테스트로 검증.

## 2. 변경 사항

### 2.1 `ParseError::UnsupportedFormat` variant 추가

```rust
pub enum ParseError {
    ...
    EncryptedDocument,
    /// 감지는 되었으나 지원하지 않는 포맷 (Issue #265)
    UnsupportedFormat { format: &'static str, hint: &'static str },
}
```

### 2.2 `Display` 구현 확장

```rust
ParseError::UnsupportedFormat { format, hint } =>
    write!(f, "지원하지 않는 포맷입니다: {format}. {hint}"),
```

### 2.3 `parse_document` 분기 추가

```rust
pub fn parse_document(data: &[u8]) -> Result<Document, ParseError> {
    match detect_format(data) {
        FileFormat::Hwpx => HwpxParser.parse(data),
        FileFormat::Hwp3 => Err(ParseError::UnsupportedFormat {
            format: "HWP 3.0",
            hint: "현재 rhwp 는 HWP 5.0 과 HWPX 만 지원합니다. \
                   한컴오피스 또는 LibreOffice 에서 파일을 연 뒤 \
                   HWP 5.0 포맷으로 다시 저장하여 시도해주세요.",
        }),
        _ => HwpParser.parse(data),
    }
}
```

### 2.4 통합 테스트 2건 추가

- `test_parse_document_hwp3_returns_unsupported_error`: HWP 3.0 합성 헤더로 UnsupportedFormat + format="HWP 3.0" + hint 에 "HWP 5.0" 포함 확인
- `test_parse_document_issue_265_sample`: **실제 제보 파일** `samples/issue_265.hwp` 로 end-to-end 검증
  - `detect_format` → `Hwp3` 감지
  - `parse_document` → `UnsupportedFormat` 반환
  - Display 메시지에 "HWP 3.0" · "HWP 5.0" 포함

### 2.5 — **추가 버그 수정**: `From<ParseError> for HwpError` 가 `Debug` 대신 `Display` 사용

프론트엔드 실로그 확인 중 발견된 치명적 문제. `src/error.rs:19-21` 이 `format!("{:?}", e)` (Debug) 로 에러를 감싸고 있어 **Stage 2 의 친절한 Display 메시지가 사용자에게 전달되지 않음**. CFB 매직 바이트 같은 내부 구조도 그대로 누출됨.

**수정**: `src/error.rs` 의 세 `From` 구현이 모두 `format!("{e}")` (Display) 사용하도록 변경.

```rust
// Before (3곳)
HwpError::InvalidFile(format!("{:?}", e))  // Debug
HwpError::RenderError(format!("{:?}", e))  // Debug

// After
HwpError::InvalidFile(format!("{e}"))   // Display
HwpError::RenderError(format!("{e}"))   // Display
```

회귀 방어 테스트 1건 추가 (`parse_error_to_hwp_error_uses_display_not_debug`):
- `ParseError::UnsupportedFormat` 이 `HwpError` 로 변환된 후 Display 포맷에 "HWP 3.0" · 힌트 문구 포함 확인
- `UnsupportedFormat` · `{ format` 같은 Debug 형식 흔적 누출 0 확인

## 3. 검증 결과

| 테스트 | 결과 |
|---|---|
| `cargo test --lib parse_error_to_hwp` | 1 passed (Display 회귀 방어) |
| `cargo test --lib` 전체 | **963 passed / 0 failed / 1 ignored** (기존 957 + 신규 6) |
| `cargo clippy --lib -- -D warnings` | clean |

신규 테스트 6건 breakdown:
- Stage 1: `test_detect_format_hwp3`, `_exact_17_bytes`, `_too_short` (3건)
- Stage 2: `test_parse_document_hwp3_returns_unsupported_error`, `test_parse_document_issue_265_sample` (2건)
- Stage 2 (추가): `parse_error_to_hwp_error_uses_display_not_debug` (1건)

## 4. 동작 예시

사용자가 `samples/issue_265.hwp` 를 로드하면:

**Before**:
```
파일 로드 실패: 문서 파싱 실패: 유효하지 않은 파일:
CFB 오류: CFB 열기 실패: Invalid CFB file (wrong magic number): [48, 57, 50, 20, 44, 6f, 63, 75]
```

**After**:
```
파일 로드 실패: 지원하지 않는 포맷입니다: HWP 3.0. 현재 rhwp 는 HWP 5.0 과 HWPX 만 지원합니다.
한컴오피스 또는 LibreOffice 에서 파일을 연 뒤 HWP 5.0 포맷으로 다시 저장하여 시도해주세요.
```

`rhwp-studio/src/main.ts:463` 의 `파일 로드 실패: ${error}` interpolation 으로 WASM 에러 메시지가 그대로 노출.

## 5. 다음 단계

Stage 3 — WASM 재빌드 + 3 프론트엔드 (rhwp-studio / Chrome ext / github.io) 확인.

## 6. 산출물

- `src/parser/mod.rs` 수정 (UnsupportedFormat variant + parse_document 분기 + 테스트 2건)
- 본 문서 (`mydocs/working/task_m100_265_stage2.md`)

Stage 2 완료.
