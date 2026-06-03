---
타스크: #265 HWP 3.0 파일 감지 + 친절한 에러 메시지
단계: Stage 1 — FileFormat::Hwp3 + detect_format 확장 + 테스트 3건
브랜치: local/task265
작성일: 2026-04-24
---

# Stage 1 완료 보고서

## 1. 목적

`FileFormat` enum 에 `Hwp3` variant 추가 + `detect_format` 에 HWP 3.0 시그니처 체크 추가. 단위 테스트 3건으로 검증.

## 2. 변경 사항

### 2.1 `src/parser/mod.rs` — `FileFormat::Hwp3` 추가

```rust
pub enum FileFormat {
    Hwp,
    Hwpx,
    /// HWP 3.0 바이너리 (미지원 — 감지만, Issue #265)
    Hwp3,
    Unknown,
}
```

### 2.2 `src/parser/mod.rs` — `detect_format` 확장

CFB/ZIP 시그니처 체크 이후에 HWP 3.0 프리픽스 체크 추가:

```rust
// HWP 3.0 바이너리 (Issue #265): "HWP Document File" 프리픽스.
// V3.00 ~ 2.x/초기 한컴 워디안까지 관대하게 포괄.
if data.len() >= 17 && &data[0..17] == b"HWP Document File" {
    return FileFormat::Hwp3;
}
```

### 2.3 테스트 3건 추가

- `test_detect_format_hwp3`: 완전한 HWP 3.0 헤더 샘플 → `Hwp3` 감지
- `test_detect_format_hwp3_exact_17_bytes`: 경계 케이스 (정확히 17바이트 프리픽스)
- `test_detect_format_hwp3_too_short`: 경계 케이스 (16바이트 → `Unknown`)

## 3. 검증 결과

```
cargo test --lib test_detect_format

running 7 tests
test parser::tests::test_detect_format_hwp ... ok
test parser::tests::test_detect_format_hwp3 ... ok
test parser::tests::test_detect_format_hwp3_exact_17_bytes ... ok
test parser::tests::test_detect_format_hwp3_too_short ... ok
test parser::tests::test_detect_format_hwpx ... ok
test parser::tests::test_detect_format_too_short ... ok
test parser::tests::test_detect_format_unknown ... ok

test result: ok. 7 passed; 0 failed
```

기존 4건 + 신규 3건 모두 통과.

## 4. Exhaustive match 점검

`FileFormat` 사용처 전수 조사:

| 파일 | 위치 | 패턴 | Hwp3 영향 |
|---|---|---|---|
| `src/wasm_api.rs:2825-2828` | `get_source_format` | `match { Hwpx => "hwpx", _ => "hwp" }` | 영향 없음 (Hwp3 는 UnsupportedFormat 에러로 거부되어 `source_format` 에 들어가지 않음) |
| `src/document_core/converters/hwpx_to_hwp.rs:218,239` | `convert_if_hwpx_source` | `!= FileFormat::Hwpx`, `FileFormat::Hwp` 비교 | 영향 없음 (equality 비교, match 아님) |
| `src/parser/mod.rs:521` | `parse` 디스패치 | `match` — Stage 2 에서 `Hwp3` 분기 추가 예정 | Stage 2 에서 처리 |

`cargo build --lib`: **0 warning, 0 error** — 기존 코드에 match 누락 없음.

## 5. 다음 단계

Stage 2 — `ParseError::UnsupportedFormat { format, hint }` variant 추가 + `parse()` 에서 `Hwp3` 감지 시 해당 에러 반환. `samples/issue_265.hwp` 통합 테스트 1건 추가.

## 6. 산출물

- `src/parser/mod.rs` 수정 (Hwp3 variant + detect_format 확장 + 테스트 3건)
- 본 문서 (`mydocs/working/task_m100_265_stage1.md`)

Stage 1 완료.
