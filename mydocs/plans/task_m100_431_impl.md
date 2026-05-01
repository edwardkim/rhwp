# Task #431 구현 계획서 — 분할 표 셀 내 문단 미출력 + dump-pages 진단 도구 보강

## 정정 본질 정리

### 결함 정합 (Stage 1 베이스라인 측정 결과)

| 페이지 | dump-pages 정황 | SVG bytes | text 갯수 | 정상/결함 |
|--------|---------------|----------|----------|----------|
| 14 | rows=2..7 | 237,476 | 859 | 정상 |
| **15** | rows=6..7 | **3,230** | **6** | **결함 — 빈 페이지** |
| 16 | rows=6..7 | 335,623 | 1,210 | 정상 |

`dump-pages` 결과는 **3 페이지 모두 동일** (`rows=6..7`) 이지만 SVG 출력은 **페이지 15 만 빈 페이지**. 즉 typeset 단계의 PartialTable fragment 정의 (`split_start_content_offset` / `split_end_content_limit`) 가 페이지마다 다르지만 dump-pages 가 노출 안 함.

### 회귀 origin 확정

- **commit `af6753f5`**, Task #362 (메인테이너 직접 정정, 2026-04-27)
- 변경 영역: `typeset.rs`, `table_layout.rs`, `table_partial.rs`, `height_measurer.rs`

## 작업 범위 (확장)

작업지시자 결정으로 본 작업 범위에 **dump-pages 진단 도구 보강** 포함:

1. **dump-pages 에 PartialTable 의 split_start_content_offset / split_end_content_limit 정보 추가** (`src/document_core/queries/rendering.rs:1432`)
2. **본 결함 정정** — 페이지 15 의 빈 페이지 결함 정정

## 정정 영역 (예상)

### 1) dump-pages PartialTable 출력 보강

`rendering.rs::dump_page_items` 의 `PageItem::PartialTable` 분기:

```rust
// 현재
PageItem::PartialTable { para_index, control_index, start_row, end_row, is_continuation, .. } => {
    out.push_str(&format!("    PartialTable   pi={} ci={}  rows={}..{}  cont={}  {}  {}\n",
        para_index, control_index, start_row, end_row, is_continuation, table_info, vpos_info));
}

// 정정 후
PageItem::PartialTable { para_index, control_index, start_row, end_row, is_continuation,
                         split_start_content_offset, split_end_content_limit } => {
    let split_info = if *split_start_content_offset > 0.0 || *split_end_content_limit > 0.0 {
        format!(" split_start={:.1} split_end={:.1}", split_start_content_offset, split_end_content_limit)
    } else { String::new() };
    out.push_str(&format!("    PartialTable   pi={} ci={}  rows={}..{}  cont={}  {}  {}{}\n",
        para_index, control_index, start_row, end_row, is_continuation, table_info, vpos_info, split_info));
}
```

### 2) 본 결함 정정 (Stage 2 에서 결정)

dump-pages 강화 후 페이지 14, 15, 16 의 split offset/limit 비교 → 페이지 15 의 fragment 가 어느 영역을 가리키는지 진단 후 정정.

## Stage 별 작업

### Stage 1: 베이스라인 측정 + dump-pages 보강

1. `dump_page_items` 의 PartialTable 출력에 split_start/end 추가
2. 페이지 14/15/16 의 split_start_content_offset / split_end_content_limit 정확값 추출
3. 본 결함의 정확한 코드 경로 식별

### Stage 2: 정정 적용

진단 결과에 따라 정정. 가능한 영역:
- `typeset.rs::compute_partial_table_fragments` 의 페이지 15 fragment 정의 정정
- `table_layout.rs::render_partial_table` 의 split offset/limit 처리 정정
- Task #362 의 `bigger_than_page` 가드 또는 `compute_cell_line_ranges` 가드 정합화

### Stage 3: 광범위 회귀 검증

- `cargo test --lib`: 1080+ passed 유지
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_418`: 1/1
- `cargo clippy --lib -- -D warnings`: 0건
- **kps-ai.hwp 광범위 회귀 점검** (Task #362 의 정정 의도 보존)
  - 88 → 79 페이지 유지
  - p56 외부 표 안 콘텐츠 클립 차단 보존
  - p67 PartialTable nested 표 정상 표시
  - p68-70 빈 페이지 2개 차단
  - p72-73 pi=778 표 누락 차단

### Stage 4: 시각 검증 (작업지시자)

- synam-001 페이지 14, 15, 16 SVG 모두 셀 내 문단 정상 출력 확인
- kps-ai 회귀 영역 시각 판정

### Stage 5: 최종 결과보고서 + 오늘할일 갱신 + 머지

## 검증 게이트

- `cargo test --lib`: 회귀 0건 (1080 passed 유지)
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_418`: 1/1
- `cargo clippy --lib -- -D warnings`: 0건
- WASM 빌드 정상
- 작업지시자 시각 판정 (synam-001 + kps-ai)

## 위험 정황 + 회피

- **Task #362 의 정정 의도 회귀**: kps-ai 의 정합 영역이 본 정정으로 회귀하지 않도록 광범위 회귀 점검 필수
- **단순 가드로 정정 시 다른 분할 표 케이스 영향**: 메모리 `feedback_hancom_compat_specific_over_general` 정합 — 케이스별 명시 가드 우선

## 단위 테스트 추가 (선택)

본 결함 케이스의 단위 테스트:
- `synam-001 p15 PartialTable rows=6..7 셀 내 문단 출력` 통합 테스트
- dump-pages 출력 포맷 단위 테스트 (split_start/end 노출 확인)
