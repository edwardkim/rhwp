# Task #452: 단락 마지막 줄 trailing line_spacing 누락 수정 — 최종 결과보고서

## 1. 문제

`samples/exam_kor.hwp` 1페이지 좌측 단에서 pi=1("밑줄 긋기는 일상적으로...") → pi=2("통상적으로 독자는...") 전환 간격이 단락 내 줄 간격보다 좁게 렌더링됨.

| 측정 (Stage 1 baseline) | 값 |
|------------------------|------|
| pi=1 단락 내 step | 24.51 px |
| pi=1.line9 ↔ pi=2.line0 step | **15.34 px** ← 9.17 px (= 1 ls) 부족 |
| 기대값 (PDF 정합) | 24.51 px (1838 HU) |

## 2. 원인

`src/renderer/layout/paragraph_layout.rs:2511-2520` 의 `is_para_last_line` 분기가 단락 마지막 visible 줄에서 trailing `line_spacing` 을 제외하고 y 를 `lh` 만 전진. 이 y 가 `layout_paragraph` 반환값 → 다음 단락 `y_start` 가 되어 다음 단락 첫 줄이 1 ls 만큼 위로 당겨짐.

Task #332 stage 2 의 의도(typeset 의 `height_for_fit` 와 layout 정합)가 **layout 만 trailing 제외 → pagination/engine 의 `current_height += para_height` 누적과 1 ls drift** 발생한 절반의 정합. 

## 3. 해결 (옵션 A)

`is_para_last_line` 분기 제거. `is_cell_last_line && cell_ctx.is_some()` (셀 내 마지막 문단 마지막 줄) 만 trailing 제외 보존. 본문 단락의 모든 줄에서 `y += lh + ls` 통일 → pagination 누적과 정합.

```rust
let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else {
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}
```

## 4. 검증 결과

### 4.1 정량 검증 (exam_kor 1페이지)

| 측정 | Before | After | 기대 |
|------|--------|-------|------|
| pi=1.line9 ↔ pi=2.line0 step | **15.34 px** | **24.50 px** ✓ | 24.51 px |
| 단락내 step | 24.51 px | 24.50 px | 24.51 px ✓ |

### 4.2 자동 테스트

- `cargo test --lib --release`: **1066 passed** (회귀 0)
- `cargo test --release --test svg_snapshot`: **6/6 passed** (`UPDATE_GOLDEN=1` 으로 2건 baseline 재갱신)

### 4.3 페이지 수 회귀 (10 종 샘플)

10/10 샘플 페이지 수 동일 (exam_kor 20 / aift 77 / biz_plan 6 / 2022국립국어원 40 / exam_eng 8 / exam_math_8 1 / k-water-rfp 28 / kps-ai 80 / synam-001 35 / 21_언어 15).

### 4.4 Task #332 회귀 점검

`samples/21_언어_기출_편집가능본.hwp` page 1 col 1 의 pi=26 + 보기 ①②③ (pi=27, pi=28, pi=29) 모두 page 1 col 1 에 fit. **#332 회귀 0**.

이론적 안전성: pagination engine 의 fit 판정은 `effective_trailing` 사용 → 본 수정으로 fit 판정 로직 자체는 변하지 않음. layout y 시프트만 정합되어 페이지 분배 결과 동일.

### 4.5 시각 검토

- **exam_kor page 1**: pi=1↔pi=2 간격 정상화 + 다른 단락 경계도 모두 단락내 step 과 동일 → PDF 시각 정합 향상.
- **exam_kor page 20** (마지막 페이지): 콘텐츠 잘림 없음, 페이지 배분 유지.
- **golden SVG 2건** (issue-147 aift-page3, issue-157 page-1): baseline 갱신 후 PNG 시각 검토 통과.

## 5. 부수 효과

- `LAYOUT_OVERFLOW` 경고 1건 (issue-157 pi=28, 10.9 px 오버플로): 페이지 마지막 단락의 trailing ls 가 col_bottom 을 살짝 넘는 cosmetic 효과. 빈 공간이므로 시각 무영향. 알려진 부수 효과.
- 모든 본문 단락이 있는 페이지의 SVG y 좌표가 trailing ls 누적분만큼 시프트 다운. 콘텐츠/구조 변화 0, 의도된 정합 효과.

## 6. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | `is_para_last_line` 분기 제거 (~10 줄) |
| `tests/golden_svg/issue-147/aift-page3.svg` | baseline 재갱신 |
| `tests/golden_svg/issue-157/page-1.svg` | baseline 재갱신 |
| `mydocs/plans/task_m100_452.md` | 수행계획서 |
| `mydocs/plans/task_m100_452_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_452_stage{1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_452_report.md` | 본 보고서 |
| `mydocs/orders/20260429.md` | 상태 갱신 |

## 7. 잔여 / 후속

- 본 수정으로 단락 경계 trailing line_spacing 처리는 정합 회복. 향후 pagination 과 layout 의 advance 모델을 단일화하는 통합 리팩토링은 #336 (LINE_SEG vpos 기반 통합 재설계) 와 함께 진행 권장.
- `LAYOUT_OVERFLOW` 경고를 cosmetic 으로 식별하는 별도 가드는 Stage 4 범위 외 — 후속 task 로 분리 가능.
