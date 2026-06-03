# Task #713 최종 결과 보고서

**제목**: 분할 표 row top portion 이 너무 작은 sliver(< 25px) 일 때 인트라-로우 분할 대신 행 단위 push (orphan 회피)
**Issue**: [#713](https://github.com/edwardkim/rhwp/issues/713)
**브랜치**: `local/task713` (stream/devel 베이스)
**작업 기간**: 2026-05-08 (단일 세션)
**최종 상태**: ✅ closes #713

---

## 1. 결함 요약

`samples/2022년 국립국어원 업무계획.hwp` 12x5 일정 표 (`pi=586 ci=0`) 의 행 8 (`한국어교육 내실화 및 교원 전문화` 섹션) 이 페이지 경계에서 **17.6 px sliver 분할** — 행 8 top 17.6 px 만 페이지 31 (또는 36 baseline) 에 두고 나머지 245.9 px 를 다음 페이지로.

**PDF 권위 (한글 2022)**: 페이지 32 첫 줄 = "한국어교육 내실화" — 행 8 전체가 페이지 32 상단부터 시작 (분할 0).

**결과**: 본 정정 후 행 8 전체가 다음 페이지 상단으로 이동 (PDF 정합).

## 2. Root cause 분석 흐름 (3 단계 가설)

### H1 폐기 — RowBreak 인트라-로우 분할 차단

`src/renderer/typeset.rs::next_can_intra_split` 분기에 RowBreak 가드 추가.

**결과**: 광범위 회귀 3 샘플 (inner-table-01, k-water-rfp, synam-001) — PDF 정합 깨짐. 한컴 PDF 가 RowBreak 표라도 인트라-로우 분할 허용함을 확인. 폐기.

### H3 시도 (활성 경로 미식별) — `remaining_content` 임계값

`src/renderer/pagination/engine.rs` 에 `remaining_content >= 20 px` 가드 추가. **효과 0** (trace 미출력).

원인: `src/document_core/queries/rendering.rs:1041-1042` 분기로 활성 경로가 `typeset.rs` 임을 확인.

```rust
let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
let mut result = if use_paginator {
    paginator.paginate_with_measured_opts(...)  // engine.rs (fallback)
} else {
    // typeset.rs (active)
};
```

### H4 채택 — `avail_content_for_r` 임계값

`PartialTable.split_end_content_limit = avail_content_for_r` 는 현 페이지의 행 r top portion. 본 결함은 이 값이 17.6 px 로 너무 작음 — 한 줄 정도의 orphan sliver.

`avail_content_for_r >= MIN_TOP_KEEP_PX` (25 px) 가드를 active 경로(`typeset.rs`) 에 추가.

## 3. 정정 (`src/renderer/typeset.rs:1892-1905`)

```rust
if next_can_intra_split && mt.is_row_splittable(r) {
    let avail_content_for_r = (remaining_avail - row_cs - padding).max(0.0);
    let total_content = mt.remaining_content_for_row(r, 0.0);
    let remaining_content = total_content - avail_content_for_r;
    let min_first_line = mt.min_first_line_height_for_row(r, 0.0);
    // [Task #713] avail_content_for_r 가 한 줄 정도로 너무 작으면 (orphan)
    // 분할 대신 행 전체를 다음 페이지로 push.
    const MIN_TOP_KEEP_PX: f64 = 25.0;
    if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
        && avail_content_for_r >= min_first_line
        && avail_content_for_r >= MIN_TOP_KEEP_PX
        && remaining_content >= MIN_SPLIT_CONTENT_PX
    {
        end_row = r + 1;
        split_end_limit = avail_content_for_r;
    }
}
```

**임계값 25 px** 결정:
- 본 결함: `avail_content_for_r = 17.6 px` < 25 → 차단
- synam-001 p23 (정합 분할): 27.3 px ≥ 25 → 허용 (변경 없음)
- 다른 분할 (93/437/510 px): 모두 ≥ 25 → 변경 없음

순수 코드 변경: **5 라인** (가드 1 + 상수 1 + 주석 3).

## 4. 검증

### TDD RED → GREEN

```
RED:  row 8 cells appear on pages={35, 36} clipped_cells=10
GREEN: row 8 cells appear on pages={36}    clipped_cells=0
```

### 페이지 시각 (페이지 36/37)

| 항목 | Before | After |
|------|--------|-------|
| 페이지 36 | rows=0..9 split_end=17.6 (row 8 sliver) | rows=0..**8** split_end=**0** (row 7 까지) |
| 페이지 37 | rows=8..12 split_start=17.6 (row 8 잔여) | rows=8..12 split_start=**0** (row 8 처음부터) |
| 페이지 37 첫 줄 | row 8 의 245.9 px portion | **"한국어교육 내실화" (row 8 시작)** ← PDF 정합 |

### cargo test --release

```
passed=1250  failed=0  ignored=3
```

### 181 샘플 페이지 수 회귀

```
diff /tmp/task713_pagecount_before.txt /tmp/task713_h4_after.txt: 0 lines
```

→ **181 샘플 회귀 0**.

## 5. 변경 파일

| 파일 | 추가 | 삭제 | 비고 |
|------|------|------|------|
| `src/renderer/typeset.rs` | +9 | -2 | 가드 추가 |
| `tests/issue_713.rs` | +96 | 0 | RED 회귀 테스트 |
| `mydocs/plans/task_m100_713.md` | +173 | 0 | 수행 계획서 |
| `mydocs/plans/task_m100_713_impl.md` | +172 | 0 | 구현 계획서 |
| `mydocs/working/task_m100_713_stage{1,2,3,4}.md` | 4 파일 | 0 | 단계별 보고서 |
| `mydocs/report/task_m100_713_report.md` | (본 문서) | | 최종 보고서 |

## 6. 단계별 진행 (Stage timeline)

| Stage | 내용 | 결과 |
|-------|------|------|
| 0 | 수행 + 구현 계획서 | 6 stage 정의 |
| 1 (RED) | tests/issue_713.rs FAIL 확인 (10 cells clipped, 2 pages) | ✅ |
| 2 (분석) | H1 시도 (회귀 3) → 폐기, H3 시도 (효과 0) → 활성 경로 재식별 | ✅ |
| 3 (GREEN) | H4 (avail_content_for_r >= 25) 적용, RED→GREEN | ✅ |
| 4 (회귀) | cargo test --release 1250/0/3 | ✅ |
| 5 (광범위) | 181 샘플 페이지 수 0 차이 | ✅ |
| 6 (보고) | 본 문서 + closes #713 | ✅ |

## 7. 후속 / 미해결

### 활성 경로 명확화 (별도 후속)

`pagination/engine.rs` 와 `typeset.rs` 에 동일 로직이 양분되어 있어 디버깅 시 혼란. `engine.rs` 가 fallback 만 사용된다면 dead code 정리 또는 단일 코드로 통합 필요. **본 타스크 비범위** — 별도 검토.

### enum 매핑 검증 (별도 후속)

Stage 2 분석에서 발견한 `TablePageBreak` enum 매핑이 HLHwp-Old 정의와 거꾸로일 가능성 — 본 타스크 정정과 직교 (Task #474 의 내장 보정 효과로 동작 정합). 명시적 정정 여부는 별도 검토 필요. **본 타스크 비범위**.

## 8. 결론

PartialTable 의 인트라-로우 분할 시 페이지 끝의 작은 sliver (< 25 px) 를 orphan 으로 판정하여 행 단위로 다음 페이지로 push 하는 가드 추가. 5 라인 정정으로 PDF 권위 자료와 시각 정합 회복. 회귀 0.

**closes #713**
