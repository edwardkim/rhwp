# Task #477 Stage 1 — 베이스라인 측정

## 결함 정황 정합

`samples/k-water-rfp.hwp` 페이지 16, 표 pi=186:

| 항목 | 정합 |
|------|------|
| 표 셀 크기 | 46482 × 12353 HU (164.0 × 43.6 mm) |
| **셀 안 그림 크기** | **46771 × 10045 HU (165.0 × 35.4 mm)** ← 셀 폭 1mm 초과 |
| 그림 속성 | tac=true, wrap=TopAndBottom |
| 정황 | crop=(0,0,42180,10440), orig=105180×26040 |

비교 대상 (정상 표 pi=184):

| 항목 | 정합 |
|------|------|
| 표 셀 크기 | 46482 × 12353 HU (164.0 × 43.6 mm) |
| 셀 안 그림 크기 | 45462 × 9827 HU (160.4 × 34.7 mm) ← 셀 폭 미만 |

→ 결함은 그림 폭이 셀 폭을 초과한 데이터 (한컴 에디터는 자동 클램프, rhwp 는 미적용).

## 호출부 4개소 점검 결과

| # | 파일 | 라인 | 결함 | inner_area 사용 가능 |
|---|------|------|------|------------------|
| 1 | `table_layout.rs::layout_table_cells` | 1539 | ✅ pic_area.width=pic_w | ✅ 같은 함수 1327 정의 |
| 2 | `table_partial.rs` | 715 | ✅ pic_area.width=pic_w | ✅ 같은 함수 685 등 사용 중 |
| 3 | `shape_layout.rs` (TAC 인라인) | 1523 | ✅ pic_container.width=pic_w | ✅ 같은 함수 1397 등 사용 중 |
| 4 | `shape_layout.rs` (비-TAC) | 1533 | ❌ pic_container.width=inner_area.width | (이미 정상) |

→ 정정 대상 **3개소**. shape_layout.rs:1533 은 이미 inner_area.width 사용 중이라 정정 불필요.

## 베이스라인 SVG

```bash
cargo run --release --quiet --bin rhwp -- export-svg samples/k-water-rfp.hwp -p 15 \
  --debug-overlay -o output/svg/task477-baseline/
```

→ `output/svg/task477-baseline/k-water-rfp_016.svg` (1.04 MB).

## 다음 단계

Stage 2: 호출부 3개소 정정
- `table_layout.rs:1532-1541` — clamped_w/h + inline_x 갱신
- `table_partial.rs:707-717` — 동일 패턴
- `shape_layout.rs:1515-1524` — 동일 패턴
