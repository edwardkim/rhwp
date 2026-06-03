# Task #501 Stage 2 — 회귀 origin 확정 + 정정 방향

## 진단 정밀화 — SVG cell-clip 측정

`output/mel-001_002.svg` 의 pi=22 표 영역 cell-clip rect 분석 (y=637.4~):

| 셀 영역 | y | h | 정합 추정 |
|---------|---|---|----------|
| cell-clip-3588 (r=0 line1) | 637.4 | **12.36px** | 행 0 영역 일부 |
| cell-clip-3640 (r=0 left rs?) | 649.76 | **35.44px** | rs=1 cs=3 셀[0] 영역 |
| cell-clip-3673 (rs=2 합계) | 657.76 | **27.45px** | r=1, rs=2 합계 (정상=34.13) |
| cell-clip-3703 (r=1 small) | 685.20 | 35.44px | ? |

**행 0 영역 (정상 = 26.4px) 이 12.36~20.35px 로 축소** 되어 표시. 표 자체 높이 146.13px 는 정합.

## 회귀 본질 확정 — 행 높이 비례 축소

`src/renderer/height_measurer.rs:743-752` 의 **TAC 표 비례 축소 로직**:

```rust
let common_h = hwpunit_to_px(table.common.height as i32, self.dpi);
let table_height = if table.common.treat_as_char && common_h > 0.0
    && raw_table_height > common_h + 1.0 {
    let scale = common_h / raw_table_height;
    for h in &mut row_heights {
        *h *= scale;
    }
    common_h
} else {
    raw_table_height
};
```

**조건**:
- `treat_as_char = true` (mel-001 pi=22 정합)
- `common_h = 146.13px` (mel-001 IR 정합)
- `raw_table_height > 147.13` 이면 비례 축소 적용

회귀 가설:
- `raw_table_height` 가 약 **327px** 정도로 잘못 계산됨 → scale ≈ 0.45
- 각 행 높이가 0.45 배로 축소
- 결과: 각 행 12.36~20.35px (정상 26.4px 의 0.46~0.77 배)

## 회귀 origin 후보 영역

`raw_table_height` 가 잘못 계산되는 원인 후보:

| 영역 | 가능성 |
|------|--------|
| 1단계 — cell.height (HU) 직접 사용 | 정합 (HWP IR 값 그대로) |
| **2단계 — required_height = content_height + pad** | **회귀 가능 영역** ★ |
| 2-b단계 — rowspan deficit 보정 (`row_heights[r + span - 1] += deficit`) | **회귀 가능 영역** ★ |

### 핵심 의심 — 2-b단계 deficit 누적

`height_measurer.rs:639-654` (resolve_row_heights) + `height_measurer.rs:715-727` (measure_table_impl) 의 동일 로직:

```rust
for cell in &table.cells {
    let r = cell.row as usize;
    let span = cell.row_span as usize;
    if span > 1 && r + span <= row_count {
        let content_height = self.calc_cell_paragraphs_content_height(...);
        let required_height = content_height + pad_top + pad_bottom;
        let combined: f64 = (r..r + span).map(|i| row_heights[i]).sum();
        if required_height > combined {
            let deficit = required_height - combined;
            row_heights[r + span - 1] += deficit;  // ★ 마지막 행에 deficit 추가
        }
    }
}
```

mel-001 의 rs=2 셀 다수 (합계, 본부, 소속기관, 기타):
- 합계 (rs=2): content_height = "합계" 1줄 = 13.33px + pad ≈ 18px. combined = 17.07 × 2 = 34.13px. **required < combined** → deficit 0
- 그런데 **`calc_cell_paragraphs_content_height` 가 잘못 큰 값을 반환** 가능성:
  - corrected_line_height 적용 시 line 의 max_fs 가 13.33px 이상이면 라인이 13.33-21.33px 등 더 큰 값
  - line_spacing 누적
  - non_inline_controls_height 추가

각 rs=2 셀이 deficit 추가하면 누적되어 `raw_table_height` 가 매우 큼.

## 회귀 origin commit 후보

크롬 v0.2.1 (5665d49, 2026-04-19) ↔ devel HEAD 사이에서 본 영역 변경 commit:

| Commit | 영역 | 영향 가능성 |
|--------|------|------------|
| `4452894` Task #324 stage2 v2 | compute_cell_line_ranges 누적위치 재작성 | line range 측정 — 본 회귀와 직접 관련 |
| `f7f0d42` Task #431 | 분할 표 셀 내 문단 미출력 (단위 mismatch) | HU/px 단위 정합 |
| **`f3ba9eb` Task #347 cont** | 셀 첫 줄 y vpos 적용 | text_y_start 계산 — 본 회귀와 다른 영역 (vpos=0 case) |
| `34e432f` Task #347 | 표/그림 절대 좌표 5건 | 본 회귀와 다른 영역 |
| Task #185 | HeightMeasurer line_height 보정 (corrected_line_height) | content_height 측정 영향 |

## 정정 방향 (3 후보)

### A. content_height 의 줄간격 누적 정합 점검

`measure_table_impl` 의 line loop (line 547-564) 에서 `is_cell_last_line` 체크 + line_spacing 적용. 음수 line_spacing (mel-001 의 ls=-300/-400) 이 `hwpunit_to_px(line.line_spacing)` 으로 음수 px 가 되어 lines_total 줄어들지만, 어떤 cell paragraph 의 lh 가 정상보다 큰 값을 가져 deficit 누적.

### B. rowspan deficit 추가 정책

`row_heights[r + span - 1] += deficit` — 본 정책이 다중 rs 셀에 누적 적용되어 raw_table_height 가 매우 큼. **deficit 을 rs 셀의 last 행 단독 추가가 아닌 rs 행 균등 분배** 또는 **deficit 자체를 IR cell.h (HWP 저장값) 와 max() 정책** 으로 변경.

### C. TAC 표의 raw_table_height 계산에서 IR cell.h 절대 우선

TAC 표 (`treat_as_char=true`) 는 한컴이 표 높이를 IR `common.height` 로 고정. `raw_table_height` 가 common_h 보다 크면 **IR cell.height 를 권위 기준** 으로 사용 (`required_height = content_height + pad` 폴백 차단). 이 정책이 한컴 정합. 다만 **셀 콘텐츠가 IR cell.h 를 넘는 케이스 → 클리핑** 으로 처리.

→ **C 가 메모리 룰 `feedback_hancom_compat_specific_over_general` 정합** (구조 가드 우선).

## 구현 계획 (Stage 3 — task_m100_501_impl.md 작성 예정)

| 단계 | 영역 |
|------|------|
| 1 | 단위 테스트 추가 (mel-001 p2 pi=22 표 행 높이 정합) — Red |
| 2 | `measure_table_impl` 의 TAC 표 raw_table_height 계산에서 **IR cell.height (rs=1) 우선 정책** 적용 (옵션 C) |
| 3 | rs>1 deficit 정책 검토 — TAC 표에서는 deficit 차단 (IR cell.h 신뢰) |
| 4 | 광범위 회귀 점검 — synam-001 / k-water-rfp / aift / hwpspec / exam_kor 회귀 0 |
| 5 | 작업지시자 시각 검증 (Stage 5) |

## 위험 영역

- **TAC 표 (treat_as_char=true) 회귀 영역 매우 광범위** — exam_eng, exam_science 등 모든 시험지 표가 TAC. 변경 영향 큰 영역.
- **회귀 점검 시 byte 비교 + debug-overlay 시각 비교 필수**
- 메모리 룰 `feedback_v076_regression_origin` 적용 — 작업지시자 직접 시각 검증을 게이트로

## 다음 단계

작업지시자 승인 후 구현 계획서 (`mydocs/plans/task_m100_501_impl.md`) 작성.
