# Task #474 Stage 1 — 베이스라인 측정

## 대상 결함 정황 (k-water-rfp.hwp)

| 페이지 | items | used | 정황 |
|-------|-------|------|------|
| **5** | 9 | **196.3 px (21%)** | PartialTable rows=0..2 (4행 중 2행만), 잔여 ~482 px 비어있음 |
| **6** | 3 | 897.1 px | PartialTable rows=2..4 (보호 블록), hwp_used≈2045.9, diff=-1148.7 |

표 pi=52 정황:
- 4행 × 4열 (셀=15)
- 셀[8] r=2, c=0, **rs=2** (rowspan=2): rows 2~3 보호 블록
- 셀[14] r=3, c=3, paras=24 → row 3 height = 729 px (페이지 1배 이상)
- **표 정책: RowBreak** (HWP attr=0x04000006, bit 0-1 = 0x02)

## PR #401 v2 정정 케이스 (synam-001.hwp) — 회귀 위험 점검

| 페이지 | items | used | 정황 |
|-------|-------|------|------|
| 5 | 6 | 138.1 px | PartialTable rows=0..5 |
| 6 | 6 | 984.6 px | PartialTable rows=4..8 (보호 블록 + 잔여) |

표 pi=69 정황:
- 8행 × 3열, 셀=17
- **셀[4] r=2, c=0, rs=5** (rowspan=5!): rows 2~6 보호 블록 (5 rows)
- **표 정책: RowBreak** (k-water-rfp 와 동일)

## 결정적 분석 — synam-001 회귀 위험 없음

`snap_to_block_boundary` 코드:

```rust
if block_size <= BLOCK_UNIT_MAX_ROWS {  // = 3
    block_start  // 보호 후퇴 (block_size 2~3 만 영향)
} else {
    end_row  // BLOCK_UNIT_MAX_ROWS 초과 시 분할 허용 (4 이상 — 영향 없음)
}
```

→ **synam-001 의 5-row 블록 (rs=5) 은 이미 `block_size > 3` 이라 분할 허용**.

본 정정의 RowBreak 가드 추가는:
- k-water-rfp pi=52 (block_size=2) → 보호 해제 (정정 의도)
- synam-001 pi=69 (block_size=5) → **이미 분할 허용 상태 그대로** (PR #401 v2 의 큰 rowspan 분할 정책 정합)

→ **synam-001 회귀 위험 없음** ✅

## 작업지시자 시각 검증 (Stage 1 baseline)

작업지시자 직접 검증 — synam-001.hwp 5쪽 / 6쪽 (한컴 정답지 비교):
- **5쪽: 정상** ✅
- **6쪽: 정상** ✅

→ PR #401 v2 의 정정 + 본 사이클 4 PR 누적 정정 정합 보존 확인. Task #474 정정 진행해도 synam-001 회귀 위험 없음.

## RowBreak 가드 정합 영역

본 정정으로 영향 받는 표:
- **block_size 2~3 + RowBreak 표** ← 본 결함 케이스 (k-water-rfp pi=52)
- block_size 2~3 + None / CellBreak 표 ← PR #401 v2 보호 정책 그대로 보존
- block_size > 3 표 ← 이미 분할 허용 (영향 없음)

## 광범위 baseline SVG 추출

10 샘플 / 305 페이지 SVG 추출 완료 (`output/svg/task474-baseline/`).

## 다음 단계

Stage 2: 정정 적용
- `MeasuredTable::allows_row_break_split()` 메서드 추가
- `snap_to_block_boundary` 에 RowBreak 가드
- `paginate_table` (typeset.rs) + `split_table_rows` (engine.rs) 의 `first_block_protected` / `cur_block_protected` / `next_block_protected` 가드
