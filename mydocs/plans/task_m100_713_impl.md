# Task #713 구현 계획서

**Issue**: [#713](https://github.com/edwardkim/rhwp/issues/713)
**브랜치**: `local/task713` (stream/devel 베이스)
**수행 계획서**: [`task_m100_713.md`](task_m100_713.md)
**작성일**: 2026-05-08

---

## 1. TDD 전략

### 1.1 RED 테스트 (Stage 1)

**파일**: `tests/issue_713.rs` (신규)

**의도**: 12x5 표 (`pi=586`) 의 PartialTable 페이지 분할에서 `split_end_limit` (인트라-로우 분할 잔여) 가 0 이어야 함을 단언.

```rust
//! Issue #713: RowBreak 표 인트라-로우 분할 차단

use rhwp::renderer::pagination::{Paginator, PaginationOptions, PageItem};
// ... (실제 import 는 paginate API 확인 후)

#[test]
fn issue_713_rowbreak_table_no_intra_row_split() {
    let path = "samples/2022년 국립국어원 업무계획.hwp";
    let doc = HwpDocument::from_bytes(&fs::read(path)?).unwrap();
    // 페이지네이션 결과 수집
    // pi=586 ci=0 의 모든 PartialTable 항목에서 split_start/split_end == 0 확인
    let pages = ...;
    for page in pages {
        for item in &page.items {
            if let PageItem::PartialTable { para_index: 586, control_index: 0,
                split_start_content_offset, split_end_content_limit, .. } = item
            {
                assert_eq!(*split_start_content_offset, 0.0,
                    "RowBreak 표 인트라-로우 분할 (split_start) 검출");
                assert_eq!(*split_end_content_limit, 0.0,
                    "RowBreak 표 인트라-로우 분할 (split_end) 검출");
            }
        }
    }
}
```

**대안 (간소화)**: dump-pages 출력의 `split_start`/`split_end` 검사 → 본 케이스에 직접 적용 가능.

### 1.2 GREEN 단계 (Stage 3)

가설 H1 (page_break 모드 가드) 적용:

```rust
// typeset.rs:1916 영역 수정 예시
let allow_intra_row_split = next_can_intra_split
    && !matches!(mt.page_break, crate::model::table::TablePageBreak::RowBreak);
if allow_intra_row_split && mt.is_row_splittable(r) {
    // ... split_end_limit 설정
}
```

동일 패턴 횡단 적용:
- `typeset.rs:1740 영역` (첫 행 분할 진입)
- `typeset.rs:1859 영역` (`approx_end <= cursor_row`)
- `typeset.rs:1916 영역` (`approx_end < row_count`) — **본 결함 트리거**
- `pagination/engine.rs` 의 동일 패턴 (점검 후 발견 시 동일 가드)

---

## 2. 분석 도구 (Stage 2)

### 2.1 디버그 인스트루먼트

`RHWP_TASK713_DEBUG=1` 환경변수, 추가 위치:

1. `typeset.rs:1916 영역` — `r`, `next_can_intra_split`, `is_row_splittable`, `mt.page_break`, `split_end_limit` 출력
2. `typeset.rs:1865 영역` — `cur_can_intra_split`, `cur_block_protected` 출력
3. `pagination/engine.rs` 유사 분기 (있을 경우)

GREEN 후 모두 제거.

### 2.2 가설 검증 절차

1. RHWP_TASK713_DEBUG=1 로 page 36 (또는 31) 페이지네이션 실행 → stderr 트레이스 수집
2. row 8 진입 시 `next_can_intra_split=true`, `page_break=RowBreak`, `split_end_limit=17.6` 확인
3. H1 적용: `next_can_intra_split` 후 page_break 가드로 `allow_intra_row_split=false` 변경
4. 결과: `end_row = approx_end (= 8)`, `split_end_limit = 0` → row 8 전체가 다음 페이지로

---

## 3. 단계별 산출물

| Stage | 파일 / 변경 | 검증 |
|-------|-----------|------|
| 0 | 수행 계획서 + 구현 계획서 | 작성 + 커밋 |
| 1 (RED) | `tests/issue_713.rs` 신규 | `cargo test issue_713 -- --nocapture` FAIL |
| 2 (분석) | (선택) `RHWP_TASK713_DEBUG` 인스트루먼트 일시 추가 | 트레이스 수집 + H1 확정 |
| 3 (GREEN) | `typeset.rs` (3 위치) ± `pagination/engine.rs` 정정 | RED PASS, 분할 0 확인 |
| 4 (회귀) | `cargo test --release` + 골든 SVG | 회귀 0 |
| 5 (광범위) | 181 샘플 페이지 수 + RowBreak 표 횡단 | 의도되지 않은 변경 0 |
| 6 (보고) | 최종 결과 보고서 + close #713 + PR | `mydocs/report/task_m100_713_report.md` |

---

## 4. Stage 별 상세

### Stage 1 (RED)

1. `tests/issue_713.rs` 작성 — page 36 의 pi=586 PartialTable split_end=0.0 단언
2. `cargo test --test issue_713` 실행 → FAIL (현재 split_end=17.6)
3. 보고서 + 커밋

### Stage 2 (분석)

1. `RHWP_TASK713_DEBUG` 트레이스 추가 (3 위치)
2. trace 수집 — row 8 진입 시 분기 결정 흐름 확인
3. H1 가드 적용 시 동작 검증
4. (Stage 3 와 통합 커밋 — 인스트루먼트는 Stage 3 에서 제거)

### Stage 3 (GREEN)

1. `typeset.rs:1916` 인트라-로우 분할 분기에 `!matches!(mt.page_break, TablePageBreak::RowBreak)` 가드 추가
2. 동일 패턴 횡단 적용 (typeset.rs L1740, L1859, pagination/engine.rs)
3. 인스트루먼트 정리
4. RED PASS, page 36 split_end=0 확인
5. 보고서 + 커밋

### Stage 4 (회귀)

1. `cargo test --release` 전체
2. 회귀 보고서 + 커밋

### Stage 5 (광범위)

1. 181 샘플 페이지 수 비교 (before / after)
2. RowBreak 표 보유 샘플 식별 후 시각 검증
3. 보고서 + 커밋

### Stage 6 (최종)

1. 최종 결과 보고서 작성
2. closes #713 커밋
3. (작업지시자 승인 후) `pr-task713` 브랜치 생성 (stream/devel 베이스), origin push, PR 생성

---

## 5. 위험 완화

| 위험 | 완화 |
|------|------|
| Task #474 의도 손상 | `allows_row_break_split()` 자체 변경 없음. `snap_to_block_boundary` 동작 유지 |
| RowBreak 단일 행 > 페이지 높이 케이스 | `effective_first_row_h > avail_for_rows` 분기 (typeset.rs:1883) 는 별도 분기 → last-resort 분할 fallback 유지 검토 |
| 다른 RowBreak 표 회귀 | Stage 5 광범위 검증 |

## 6. 비범위

- `None` (page_break == None) 모드 동작 변경
- `CellBreak` 모드 동작 변경
- `allows_row_break_split()` 함수 의미 분리 리팩토링 (별도 후속)
