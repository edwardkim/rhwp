# Stage 2 단계별 보고서 — Task #554

> **이슈**: [#554](https://github.com/edwardkim/rhwp/issues/554)
> **구현계획서**: `mydocs/plans/task_m100_554_impl.md`
> **단계**: Stage 2-1 — HWPX 휴리스틱 + 조건부 -1600 보정 구현
> **상태**: 완료, 작업지시자 승인 대기
> **작성일**: 2026-05-03

---

## 1. 구현 내용

### 1.1 변경 파일

| 파일 | 변경 | 설명 |
|------|------|------|
| `src/parser/hwpx/header.rs` | +28 LOC | `parse_hwpx_hwpml_version()` 신규 함수 추가 — `<hh:head version="X.Y">` 추출 |
| `src/parser/hwpx/mod.rs` | +13 LOC | hwpml 버전 = "1.4" 식별 + 모든 SectionDef.page_def.margin_bottom -= 1600 (post-process) |

### 1.2 핵심 로직

**hwpml 버전 추출** (`header.rs`):

```rust
pub fn parse_hwpx_hwpml_version(xml: &str) -> Option<String> {
    // <hh:head version="X.Y"> root element 의 version 속성 추출
    // 헤더 root만 읽고 즉시 반환 (비용 최소)
}
```

**조건부 보정** (`mod.rs`):

```rust
let hwpml_version = header::parse_hwpx_hwpml_version(&header_xml);
let is_hwp3_origin = hwpml_version.as_deref() == Some("1.4");
// ... 섹션 파싱 후
if is_hwp3_origin {
    for section in sections.iter_mut() {
        section.section_def.page_def.margin_bottom =
            section.section_def.page_def.margin_bottom.saturating_sub(1600);
    }
}
```

### 1.3 설계 선택 근거

- **별도 함수**: 기존 `parse_hwpx_header` 시그니처를 유지하여 다른 호출자 영향 0
- **post-process**: 한컴 변환본 식별 후 page_def 단일 필드만 보정 (광범위 수정 회피)
- **`as_deref() == Some("1.4")`**: 정확한 문자열 매칭 (1.5/1.6 등 향후 버전은 자동 제외)

## 2. 검증 결과

### 2.1 HWPX 변환본 (Task #554 핵심 대상)

| 파일 | 한컴 정답 | 변경 전 | 변경 후 | 평가 |
|------|----------|---------|---------|------|
| hwp3-sample-hwpx.hwpx | 16 | 17 (+1) | **15** (-1) | ⚠️ over-correct (Stage 1 예상) |
| hwp3-sample5-hwpx.hwpx | 64 | 68 (+4) | **64** ✓ | ✅ 정답 |

`hwp3-sample-hwpx.hwpx` 의 -1 over-correct 는 Stage 1 보고서에서 이미 예상한 잔존 사항. 단일 -1600 값으로 sample/sample5 모두 정합 어려움. 해결은 보정값 정밀화 (가변 tolerance 등) 후속 task로 분리.

### 2.2 HWPX 일반 fixture 회귀 0

| 파일 | 변경 전 | 변경 후 | 평가 |
|------|---------|---------|------|
| 표-텍스트.hwpx | 1 | 1 | ✅ |
| 2025년 기부·답례품 실적.hwpx | 30 | 30 | ✅ |
| table-vpos-01.hwpx | 5 | 5 | ✅ |
| tac-img-02.hwpx | 73 | 73 | ✅ |

→ **hwpml="1.5" fixture 모두 회귀 0**.

### 2.3 자동 회귀 검증

| 검사 | 결과 |
|------|------|
| `cargo test --lib` | **1113 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | 6/6 passed |
| `cargo test --test issue_546` | passed (Task #546 정합) |
| `cargo clippy --lib -- -D warnings` | **0건** |

## 3. 잔존 사항

### 3.1 hwp3-sample-hwpx.hwpx -1 over-correct (의도됨)

- 정답 16 vs 결과 15
- Stage 1 진단으로 이미 식별. 단일 -1600 값으로는 sample 변환본 정합 불가
- 해결 옵션 (별도 task):
  - (A) paragraph 수 기반 가변 tolerance
  - (B) 페이지 수 모니터링 후 동적 보정
  - (C) 본질적 한글97 페이지네이션 모델 재구현

### 3.2 HWP5 미적용 (Stage 2-2 대상)

본 단계는 HWPX만 처리. HWP5 (`hwp3-sample-hwp5.hwp`, `hwp3-sample4-hwp5.hwp`, `hwp3-sample5-hwp5.hwp`) 는 Stage 2-2 에서 별도 휴리스틱 (PS/CS 비율) 으로 처리.

## 4. 작업지시자 승인 요청

본 Stage 2-1 보고서 검토 후:

1. **Stage 2-1 구현 승인**:
   - HWPX hwpml 버전 추출 함수 (`parse_hwpx_hwpml_version`)
   - HWP3 origin 감지 + post-process margin_bottom 보정
2. **잔존 사항 (-1 over-correct) 의도된 trade-off 승인**:
   - hwp3-sample-hwpx.hwpx (16 → 15)
   - 향후 별도 task로 정밀화
3. **Stage 2-2 (HWP5 구현) 진행 승인**

승인 후 Stage 2-2로 진행한다.
