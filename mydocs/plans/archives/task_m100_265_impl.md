---
타스크: #265 HWP 3.0 파일 감지 + 친절한 에러 메시지
구현계획서
브랜치: local/task265
작성일: 2026-04-24
---

# 구현계획서

## 1. 사전 조사 결과 (수행계획서 이후 추가)

### 1.1 프론트엔드 에러 표시 흐름 확인

- `rhwp-studio/src/main.ts:455-468 (loadFile)`: `catch (error)` 에서 `파일 로드 실패: ${error}` 로 WASM 에러 문자열을 **그대로 노출**
- `rhwp-chrome/build.mjs`: Chrome 확장은 rhwp-studio 를 빌드 결과로 포함. 자체 에러 UI 없음
- `rhwp-firefox` · github.io: 동일 구조 (studio 재사용)

**결론**: WASM 경계의 `ParseError::Display` 가 적절한 한국어 메시지를 반환하면 **3 프론트엔드 모두 자동 반영** — 프론트엔드 수정 불필요. 수행계획서 Stage 3 (프론트엔드 검증) 은 "그대로 노출되는지 1회 확인" 수준으로 축소.

### 1.2 `detect_format` 현재 구조

`src/parser/mod.rs:49-62`:
```rust
pub fn detect_format(data: &[u8]) -> FileFormat {
    if data.len() >= 8 {
        if data[0] == 0xD0 && data[1] == 0xCF && data[2] == 0x11 && data[3] == 0xE0 {
            return FileFormat::Hwp;
        }
        if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 {
            return FileFormat::Hwpx;
        }
    }
    FileFormat::Unknown
}
```

### 1.3 `parse()` 디스패치 위치

`src/parser/mod.rs:521` (확인됨). 포맷에 따른 분기 진입점.

### 1.4 파일 시그니처 정확한 길이

`samples/issue_265.hwp` 실측:
```
"HWP Document File V3.00 " (23바이트 + 공백 1) + 바이너리 헤더
```

공식 HWP 3.0 스펙: `"HWP Document File V3.00 "` (끝에 공백, 총 24 ASCII 바이트) 이후 3바이트 구분자 `\x1a\x01\x02` 와 checksum.

**안전한 감지**: 첫 **17바이트** `"HWP Document File"` 까지만 체크하여 공백/소수점 변형 (`V3.00` · `V3.01` · `V5.0` 구버전 등) 에도 관대. 실제 케이스에서 HWP 2.x/3.x/한글 워디안 초기 버전까지 포괄.

## 2. 파일 단위 변경

### 2.1 `src/parser/mod.rs` — `FileFormat::Hwp3` + 감지 로직

```rust
pub enum FileFormat {
    /// HWP 5.0 바이너리 (CFB/OLE 컨테이너)
    Hwp,
    /// HWPX (XML 기반, ZIP 컨테이너)
    Hwpx,
    /// HWP 3.0 바이너리 (미지원, 감지만 - Issue #265)
    Hwp3,
    /// 알 수 없는 포맷
    Unknown,
}

pub fn detect_format(data: &[u8]) -> FileFormat {
    if data.len() >= 8 {
        // CFB/OLE 시그니처 (HWP 5.0)
        if data[0] == 0xD0 && data[1] == 0xCF && data[2] == 0x11 && data[3] == 0xE0 {
            return FileFormat::Hwp;
        }
        // ZIP 시그니처 (HWPX)
        if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 {
            return FileFormat::Hwpx;
        }
    }
    // HWP 3.0 바이너리 시그니처 — "HWP Document File" 프리픽스
    // (Issue #265: V3.00, 2.x/초기 워디안 등 구버전 포괄)
    if data.len() >= 17 && &data[0..17] == b"HWP Document File" {
        return FileFormat::Hwp3;
    }
    FileFormat::Unknown
}
```

### 2.2 `src/parser/mod.rs` — `ParseError::UnsupportedFormat`

```rust
pub enum ParseError {
    CfbError(cfb_reader::CfbError),
    ...
    EncryptedDocument,
    /// 감지는 되었으나 지원하지 않는 포맷 (Issue #265)
    UnsupportedFormat { format: &'static str, hint: &'static str },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ...
            ParseError::UnsupportedFormat { format, hint } =>
                write!(f, "지원하지 않는 포맷입니다: {}. {}", format, hint),
        }
    }
}
```

### 2.3 `src/parser/mod.rs` — `parse()` 디스패치 확장

`parse()` 진입점 (line ~521):

```rust
match detect_format(data) {
    FileFormat::Hwp => parse_hwp(data, options),
    FileFormat::Hwpx => hwpx::parse(data),
    FileFormat::Hwp3 => Err(ParseError::UnsupportedFormat {
        format: "HWP 3.0",
        hint: "현재 rhwp 는 HWP 5.0 과 HWPX 만 지원합니다. \
               한컴오피스 또는 LibreOffice 에서 파일을 연 뒤 \
               HWP 5.0 포맷으로 다시 저장하여 시도해주세요.",
    }),
    FileFormat::Unknown => Err(/* 기존 Unknown 에러 */),
}
```

### 2.4 `src/parser/mod.rs` — 테스트

```rust
#[test]
fn test_detect_format_hwp3() {
    let hwp3_header = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05\x00\x00";
    assert_eq!(detect_format(hwp3_header), FileFormat::Hwp3);
}

#[test]
fn test_detect_format_hwp3_too_short() {
    // 17바이트 미만이면 감지하지 않음
    let short = b"HWP Document Fil";  // 16바이트
    assert_eq!(detect_format(short), FileFormat::Unknown);
}

#[test]
fn test_detect_format_hwp3_exact_17_bytes() {
    // 경계: 정확히 17바이트
    let exact = b"HWP Document File";
    assert_eq!(detect_format(exact), FileFormat::Hwp3);
}

#[test]
fn test_parse_hwp3_returns_unsupported_error() {
    // issue_265.hwp 의 헤더를 모사하여 UnsupportedFormat 반환 확인
    let hwp3_data = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05";
    let result = parse(hwp3_data, &ParseOptions::default());
    match result {
        Err(ParseError::UnsupportedFormat { format, .. }) => {
            assert_eq!(format, "HWP 3.0");
        }
        _ => panic!("expected UnsupportedFormat error"),
    }
}
```

### 2.5 `samples/issue_265.hwp` 실제 파싱 테스트

이미 저장소에 편입된 파일을 이용한 통합 테스트 1건:

```rust
#[test]
fn test_issue_265_hwp3_sample() {
    let data = std::fs::read("samples/issue_265.hwp").expect("sample exists");
    assert_eq!(detect_format(&data), FileFormat::Hwp3);
    let err = parse(&data, &ParseOptions::default()).unwrap_err();
    assert!(matches!(err, ParseError::UnsupportedFormat { .. }));
    let msg = format!("{}", err);
    assert!(msg.contains("HWP 3.0"));
    assert!(msg.contains("HWP 5.0"));  // hint 에서 언급
}
```

### 2.6 WASM 경계 — 추가 변경 없음

`Display::fmt` 구현으로 WASM 에 자동 전파. WASM API 에서 ParseError 를 문자열화해 JS 로 넘기는 경로는 기존 그대로.

## 3. 단계별 실행

### Stage 1 — `detect_format` + `Hwp3` variant + 단위 테스트 (3건)

작업:
1. `FileFormat::Hwp3` enum variant 추가
2. `detect_format` 에 시그니처 체크 추가 (17바이트 프리픽스)
3. 단위 테스트 3건 추가

검증:
- `cargo test --lib test_detect_format` (신규 3건 통과 확인)
- `cargo test --lib` 전체 957+ passed

산출물:
- `src/parser/mod.rs` 수정
- `mydocs/working/task_m100_265_stage1.md`

### Stage 2 — `ParseError::UnsupportedFormat` + `parse()` 디스패치

작업:
1. `ParseError::UnsupportedFormat { format, hint }` variant 추가
2. `Display` 구현 확장
3. `parse()` 에서 `Hwp3` 감지 시 해당 에러 반환
4. 통합 테스트 1건 (`samples/issue_265.hwp`) 추가

검증:
- `cargo test --lib test_issue_265` 통과
- `cargo test --lib` 958 passed (신규 4건 합산)
- `cargo clippy --lib -- -D warnings` clean

산출물:
- `src/parser/mod.rs` 수정
- `mydocs/working/task_m100_265_stage2.md`

### Stage 3 — 프론트엔드 확인

작업:
1. WASM 재빌드 (`docker compose --env-file .env.docker run --rm wasm`)
2. `rhwp-studio` 로컬 실행 (`cd rhwp-studio && npx vite --port 7700`)
3. `samples/issue_265.hwp` 로드 시도 → 에러 메시지가 새 문구로 표시되는지 확인
4. Chrome ext / github.io 빌드 경로는 studio 와 동일 소스라 스모크 생략 (재빌드 시점에 자연 반영)

검증:
- 스크린샷 또는 console 로그 캡처로 새 에러 메시지 확인
- 기존 HWP 5.0 샘플 로드는 정상 작동 확인 (회귀 0)

산출물:
- `mydocs/working/task_m100_265_stage3.md`
- 브라우저 확인 결과 (스크린샷 또는 로그)

### Stage 4 — 회귀 검증

작업:
1. `cargo test --lib` 전체 그린
2. `cargo test --test svg_snapshot` 3 passed
3. `cargo clippy --lib -- -D warnings` clean
4. 스모크: 기존 HWP 5.0 샘플 3건 (`samples/exam_kor.hwp`, `biz_plan.hwp`, `text-align.hwp`) · HWPX 1건 (`samples/hwpx/hwpx-02.hwpx`) export-svg 정상 작동 확인

검증:
- 기존 기능 영향 0

산출물:
- `mydocs/working/task_m100_265_stage4.md`

### Stage 5 — 문서 + 제보자 회신 + 이슈 close

작업:
1. `mydocs/report/task_m100_265_report.md` (최종 보고서)
2. `mydocs/orders/20260424.md` 신규 작성 (오늘 주요 타스크 기록)
3. `mydocs/orders/20260423.md` 의 어제 다음 작업 후보 섹션에서 PR #265 대응 항목 처리됨으로 표기
4. 제보자 @jangster77 에게 이슈 해결 코멘트:
   - merge 커밋 해시 + 확인 방법 (rhwp-studio 최신판 재접속)
   - HWP 3.0 정식 지원은 별도 이슈로 분리 예정 안내
5. 이슈 #265 수동 close (`closes #265` 가 커밋 메시지에 있더라도 안전 차원에서 수동)

산출물:
- 최종 보고서 · 오늘할일 · 제보자 회신 코멘트 · 이슈 close

## 4. 커밋 단위 제안

Stage 1~2 통합 (코드 + 테스트 한 번에) + Stage 3~4 검증 (필요 시 한 번만) + Stage 5 문서 — 총 **2~3 커밋**:

1. `Task #265: HWP 3.0 감지 + 친절한 에러 메시지 (closes #265)` — detect_format/parse 수정 + 테스트 4건
2. (필요 시) WASM 재빌드 결과물 (`pkg/rhwp_bg.wasm`)
3. `Task #265 Stage 5 + 최종 보고서: 문서 + orders + 회신` — 문서 전체

## 5. 위험 관리

| 위험 | 완화 |
|---|---|
| "HWP Document File" 프리픽스가 HWP 5.0 문서에도 실제 텍스트로 포함될 수 있음 | 시그니처 체크를 **바이트 0부터** 수행. HWP 5.0 은 CFB 컨테이너 시작 바이트가 `0xD0CF11E0` 이므로 우선 매칭되어 경로 분기. HWP 3.0 체크는 CFB/ZIP 실패 후에만 수행 |
| 테스트에서 `samples/issue_265.hwp` 파일을 읽을 때 경로 문제 | `cargo test` 는 기본 작업 디렉터리가 프로젝트 루트이므로 `samples/issue_265.hwp` 상대 경로로 읽기 가능 (기존 테스트도 동일 관례) |
| 다른 `FileFormat` 비교 코드에서 `Hwp3` variant 추가로 `match` 불완전해질 가능성 | Rust 컴파일러가 exhaustive match 를 강제하므로 빌드 단계에서 모두 포착 |
| WASM 재빌드 누락 시 배포판에 반영 안 됨 | Stage 3 에서 WASM 재빌드 필수 |

## 6. 예상 소요

- Stage 1 (detect_format): 10분
- Stage 2 (ParseError + dispatch + 통합 테스트): 15분
- Stage 3 (프론트엔드 확인): 10분
- Stage 4 (회귀): 5분
- Stage 5 (문서): 20분
- **합계: 약 60분**

## 7. 승인 요청

본 구현계획서 승인 시 Stage 1 착수.
