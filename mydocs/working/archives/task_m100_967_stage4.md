# Task #967 Stage 4 — 다중 sample 회귀 검증

## 1. cargo test --release --lib

**결과**: **1288 passed, 0 failed, 2 ignored**

## 2. sample18 단위 검증 (Fix B)

| Format | Pre-Fix | Post-Fix | 한컴 |
|--------|---------|----------|------|
| `hwp3-sample18.hwp` (HWP3) | 69 | **67 ✓** | 67 |
| `hwp3-sample18-hwp5.hwp` (HWP5) | 67 | 67 | 67 |
| `hwp3-sample18-hwp5.hwpx` (HWPX) | 74 | 74 | 67 (별도 issue) |

→ HWP3 한컴 정합.

## 3. 다중 sample page count 회귀 검증

| Sample | Pre-Fix | Post-Fix | 차이 |
|--------|---------|----------|------|
| hwp3-sample.hwp | 16 | 16 | 0 |
| hwp3-sample10.hwp | 763 | 763 | 0 |
| hwp3-sample11.hwp | 151 | 151 | 0 |
| hwp3-sample13.hwp | 3 | 3 | 0 |
| hwp3-sample14.hwp | 11 | 11 | 0 |
| hwp3-sample16.hwp | 64 | 64 | 0 |
| **hwp3-sample18.hwp** | **69** | **67** | **-2 ✓** |
| hwp3-sample19.hwp | 2 | 2 | 0 |
| hwp3-sample4.hwp | 36 | 36 | 0 |
| hwp3-sample5.hwp | 64 | 64 | 0 |
| hwp_table_test*.hwp | 3 | 3 | 0 |
| **multi-table-001/002.hwp** | **2** | **2** | **0** (hwp-multi-001 회귀 차단 유지) |
| exam_kor.hwp | 20 | 20 | 0 |
| exam_math.hwp | 20 | 20 | 0 |
| exam_eng.hwp | 8 | 8 | 0 |

→ **sample18 만 정확히 -2. 다른 모든 sample 회귀 0**.

특히 **hwp-multi-001** (Stage 1 가드의 회귀 차단 대상) 도 변경 없음 → fix 가 빈 paragraph + 다음 force_break case 만 정확 catch.

## 4. 평가

- 단위 검증 (sample18 page count): ✓ (69 → 67)
- cargo test 전체: ✓ (1288/0/2)
- 다중 sample page count 회귀: ✓ 0
- hwp-multi-001 회귀 차단 보존: ✓

→ Stage 5 진행 (commit + PR).
