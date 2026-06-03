# Task #713 Stage 3 (GREEN) 완료 보고서

**Issue**: [#713](https://github.com/edwardkim/rhwp/issues/713)
**Stage**: 3 — TDD GREEN (정정 적용)
**작성일**: 2026-05-08

---

## 1. Root cause 재정정 (가설 H4)

### Stage 2 의 가설 폐기 후 신규 가설

| 가설 | 결과 |
|------|------|
| H1 — RowBreak 인트라-로우 분할 차단 | 3 샘플 회귀 (PDF 정합 깨짐) → 폐기 |
| H3 — `remaining_content` 임계값 가드 | 활성 경로(typeset.rs)에 미적용으로 효과 0 → 재시도 |
| **H4 — `avail_content_for_r` (top portion) 임계값 가드** | **GREEN, 회귀 0** ✓ |

### 활성 경로 식별

`src/document_core/queries/rendering.rs:1041-1042`:

```rust
// TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback 가능.
let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
```

→ `typeset.rs::typeset_section` 이 활성, `pagination/engine.rs::paginate_with_measured` 는 fallback.
Stage 2 의 `engine.rs` 수정 시도가 효과 없었던 이유.

### 본질

PartialTable 의 `split_end_content_limit` = `avail_content_for_r` (현 페이지에 들어가는 행 r 의 top portion 높이).

본 결함: avail_content_for_r ≈ **17.6 px** — 행 8 의 한 줄 정도 sliver 만 page 31 에 두고 나머지 245.9 px 를 page 32 로. 한컴 PDF 는 sliver 두지 않고 행 전체를 다음 페이지로 push (orphan 회피).

다른 분할 케이스 (synam-001 p23 의 27.3 px 등) 는 한컴 PDF 와 정합 → orphan 임계값을 그 사이에 두면 본 결함 차단 + 회귀 0.

## 2. 정정 (`src/renderer/typeset.rs:1892-1905`)

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
        && avail_content_for_r >= MIN_TOP_KEEP_PX  // ← 신규 가드
        && remaining_content >= MIN_SPLIT_CONTENT_PX
    {
        end_row = r + 1;
        split_end_limit = avail_content_for_r;
    }
}
```

**임계값 25 px** 결정 근거:
- 본 결함 (2022 국립국어원 row 8): `avail_content_for_r = 17.6 px` < 25 → 차단
- synam-001 p23 (정합): `avail_content_for_r = 27.3 px` ≥ 25 → 허용 (변경 없음)
- 다른 정합 분할 (93.0/437.5/510.7 px): 모두 ≥ 25 → 변경 없음

## 3. 검증

### 회귀 테스트 (TDD)

```
$ cargo test --test issue_713 -- --nocapture
[issue_713] page_count=40 row 8 cells found across 5 (page, clip) entries
[issue_713] row 8 cells appear on pages={36} clipped_cells=0
test issue_713_rowbreak_table_no_intra_row_split ... ok
```

→ **RED → GREEN**: row 8 (5 cells) 단일 페이지(36)에만 존재, clip=0.

### Before/After 비교 (페이지 36/37)

| 항목 | Before (RED) | After (GREEN) |
|------|--------------|---------------|
| 페이지 36 | rows=0..9 split_end=17.6 (행 8 sliver) | **rows=0..8 split_end=0** (행 7 까지) |
| 페이지 37 | rows=8..12 split_start=17.6 (행 8 잔여) | **rows=8..12 split_start=0** (행 8 부터) |
| 페이지 37 첫 줄 | 행 8 의 245.9 px 부분 | **"한국어교육 내실화" (행 8 시작)** |

PDF 권위 (한글 2022 page 32 첫 줄 = "한국어교육 내실화") 와 정확 일치 ✓

### 회귀 (`cargo test --release`)

```
passed=1250  failed=0  ignored=3
```

회귀 0.

### 광범위 회귀 (181 샘플 페이지 수)

```
$ diff /tmp/task713_pagecount_before.txt /tmp/task713_h4_after.txt
(0 lines)
```

→ **181 샘플 페이지 수 회귀 0**. H1 가설에서 회귀했던 inner-table-01, k-water-rfp, synam-001 모두 정합.

## 4. 변경 파일

| 파일 | 추가 | 삭제 |
|------|------|------|
| `src/renderer/typeset.rs` | +9 | -2 |

순수 코드 변경: **5 라인** (가드 1 라인 + 주석 + const 1 라인). 기능 = 가드 1 추가.

## 5. 다음 단계 (Stage 4 — 회귀 확정 + Stage 5 광범위 + Stage 6 보고)

이미 위에서 1250/1250 + 181 샘플 0 차이 확인. Stage 4-5 로 통합 보고 + Stage 6 최종 보고서 작성 후 close #713.

## 승인 요청

Stage 3 GREEN 완료. Stage 4-5-6 진행 승인 요청.
