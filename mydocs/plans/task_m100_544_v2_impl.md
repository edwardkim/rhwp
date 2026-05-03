# Task #544 v2 구현계획서

본 문서는 `task_m100_544_v2.md` 수행계획서의 4 단계 분할 구현 상세.

브랜치: `local/task544_v2` (`local/devel` 분기, devel sync 후)
원 fix commits: `7ba2ecbe` (#544 S2), `b3586723` (#547 S2), `9576f364` (#548 S2)
원 TDD commits: `965ea51a` (#544 S1), `9bec6d8a` (#547 S1), `f4bced43` (#548 S1)

## Stage 1 — TDD RED 복원 + Task #552 양립 사전 검증

### 작업

1. `src/renderer/layout/integration_tests.rs` 끝에 3 개 테스트 복원:
   - `test_544_passage_box_coords_match_pdf_p4` (RED, `#[ignore]`)
   - `test_547_passage_text_inset_match_pdf_p4` (RED, `#[ignore]`)
   - `test_548_cell_inline_shape_first_line_indent_p8` (RED, `#[ignore]`)
   - 각 commit 의 원본 코드 그대로 복사

2. `cargo test --lib` 1119 baseline 통과 확인 (ignore 된 3 건은 미실행)

3. 사전 측정: `#[ignore]` 일시 해제 후 `cargo test test_544 test_547 test_548 -- --ignored` 실행
   - test_544: box_top_y / box_left_x / box_width 측정값 기록 (Task #552 후 box_top_y 가 이미 PDF 정합인지 확인 — 양립 검증 핵심)
   - test_547: min_x 측정값
   - test_548: puko_x 측정값
   - 측정 후 다시 `#[ignore]` 복원

4. 결과 → `mydocs/working/task_m100_544_v2_stage1.md` 작성:
   - Task #552 가 box_top_y 를 이미 보정했는지 (Phase B 필요/불필요 결정)
   - 3 개 측정값 + PDF 기대값 + drift 표
   - Phase B skip 여부 결정

### 산출물

- `src/renderer/layout/integration_tests.rs` (+3 테스트)
- `mydocs/working/task_m100_544_v2_stage1.md`

### 검증

- cargo test --lib (1119 baseline 무회귀)
- 3 개 측정값 확보

### 커밋

`Task #544 v2 Stage 1: TDD RED 복원 + Task #552 양립 사전 검증`

---

## Stage 2 — Phase A 재적용 (paragraph_layout.rs box_x + inner_pad 분기 제거)

### 작업

1. **#547**: `paragraph_layout.rs:690~716` inner_pad 분기 제거 (`b3586723` 와 동일):
   - `has_visible_stroke`, `bs_left_px`, `bs_right_px`, `inner_pad_left`, `inner_pad_right` 변수 제거
   - `margin_left = box_margin_left;`
   - `margin_right = box_margin_right;`
   - `para_border_fill_id_pre` 사용처 없으면 제거

2. **#544 (1)**: `paragraph_layout.rs:2687-2691` box_x/w 산식 정정:
   ```rust
   let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
       (ox, ow)
   } else {
       (col_area.x, col_area.width)
   };
   ```

3. test_544 / test_547 의 `#[ignore]` 제거 (RED → GREEN 전환)
   - test_544 box_top_y assertion 은 Stage 1 결정에 따라:
     - Task #552 가 이미 보정 시 → 그대로 GREEN
     - 미보정 시 → Stage 3 의 Phase B 후 GREEN (Stage 2 에서는 box_left_x / box_width 만 검증하도록 split 또는 별도 test_544_xy 추가)

4. cargo test --lib → 1121 통과 (test_544 box_top_y 영역 제외 가능)

### 검증

- cargo test --lib 1121 통과
- test_547 GREEN
- test_544 (좌표/폭 영역) GREEN
- Task #552 test_552_passage_box_top_gap_p2_4_6 GREEN 유지

### 커밋

`Task #544 v2 Stage 2: paragraph border 좌표/inset 산식 정정 (#547 + #544 (1) 재적용)`

---

## Stage 3 — Phase B (조건부) + Phase C 재적용

### 작업

#### Phase B (Task #544 (2) — Stage 1 결정에 따라 조건부)

Stage 1 측정에서 box_top_y 가 PDF 정합 (±2 px) 이면 skip.
미정합이면 다음 재적용:

- `src/renderer/layout.rs` LayoutEngine struct: `paragraph_border_y_correction_px: Cell<f64>` 필드 추가 (initialized 0.0)
- `src/renderer/layout.rs:1490` 부근 (vpos correction 가드 skip 케이스): 다음 paragraph 가 border 가지면 trailing-ls 만큼 set
  - **Task #552 양립**: `next_para_starts_visible_border` 가 이미 set 된 경우 set 회피 (이중 보정 방지). 또는 reverse — Task #552 의 trailing-ls 보존이 동일 paragraph 에서 이미 적용 시 본 보정 스킵 가드.
- `src/renderer/layout/paragraph_layout.rs:786` `bg_y_start` 산식:
  ```rust
  let bg_y_start = if para_border_fill_id > 0 {
      let corrected = y_start + self.paragraph_border_y_correction_px.get();
      self.paragraph_border_y_correction_px.set(0.0);
      corrected
  } else {
      self.paragraph_border_y_correction_px.set(0.0);
      y
  };
  ```

#### Phase C (Task #548 — table_layout.rs)

원 commit `9576f364` 그대로 재적용:
- `effective_margin_left_line` 헬퍼 추가
- inline_x 산출 3 분기 (초기 / Picture target_line reset / Shape target_line reset) Left/Justify 케이스에 line_margin 가산
- `para_margin_left_px` / `para_indent_px` 추출 추가

### 검증

- cargo test --lib 1122 통과
- test_544 / test_547 / test_548 모두 GREEN
- test_552 / Task #525 / #530 / #505 / #418 / #501 / #546 무회귀
- 광범위 svg_snapshot 6 fixture 회귀 측정 (드리프트 시 의도된 정정인지 분석)

### 커밋

`Task #544 v2 Stage 3: Phase B (필요 시) + Phase C 재적용 (#548 셀 inline TAC Shape margin)`

---

## Stage 4 — 광범위 회귀 + 최종 보고서

### 작업

1. 광범위 회귀 검증:
   - svg_snapshot 6 fixture (basic / treatise / hwpspec / 21_언어_기출 / aift / water-mark)
   - exam_kor / exam_eng / exam_math / exam_science / exam_social
   - byte-identical 비율 측정. 변경 페이지의 paragraph border 좌표 변화 분석 (의도된 정정 vs 회귀 분리)

2. 시각 판정 보고:
   - 21_언어_기출 페이지 2 [1~3] / 페이지 2 [4~6] / 페이지 4 [7~9] / 페이지 8 셀 5 [푸코]
   - PDF 한컴 2010 비교
   - SVG 1차 → rhwp-studio web Canvas 2차

3. 최종 보고서 `mydocs/report/task_m100_544_v2_report.md`:
   - Phase A/B/C 적용 범위 + commit 매핑
   - 측정 결과 (Stage 1 사전 / Stage 2/3 사후)
   - 광범위 회귀 결과
   - WASM size delta
   - clippy 0
   - 잔존 사항 (있을 경우)

4. `mydocs/orders/20260503.md` 갱신:
   - Task #544 v2 완료 항목 추가
   - "잔존 (별도 이슈 후보)" 항목 정정 또는 제거

### 검증

- cargo test --lib 1122 GREEN
- clippy 0
- WASM build 성공
- 작업지시자 시각 판정 1차 (SVG) 통과 → 2차 (rhwp-studio web Canvas) 통과

### 커밋

`Task #544 v2 Stage 4: 최종 보고서 + 광범위 회귀 검증 + orders 갱신`

---

## 머지 절차 (Stage 4 승인 후)

```
git checkout local/devel
git merge local/task544_v2 --no-ff -m "Merge local/task544_v2: Task #544 v2 회귀 정정 (#544/#547/#548 재적용)"
git checkout devel
git merge local/devel --no-ff -m "Merge local/devel: Task #544 v2 회귀 정정"
git push origin devel
```

## 위험 / 잔존 가능성

1. **Task #552 와의 이중 보정**: Phase B 가 필요한지 Stage 1 측정으로 판단. 두 메커니즘이 동일 케이스에서 동시 작동하면 박스 top 이 trailing-ls 만큼 더 위로 시프트.
2. **광범위 회귀**: paragraph margin 이 0 이 아닌 다른 paragraph 의 박스 좌표가 모두 변경됨. 한컴 PDF 정합이 의도이므로 byte-identical 율 하락은 예상 → 시각 판정 게이트.
3. **셀 내부 inline TAC Shape (#548)**: align != Left/Justify 인 케이스 회귀 가능. inner_area.width 가 line_margin 보다 작은 경계 케이스 점검.

## 코드 영향 요약

| 파일 | 변경 LOC (예상) | 영역 |
|------|-----------------|------|
| `src/renderer/layout/paragraph_layout.rs` | +8 / -25 | inner_pad 분기 제거 + box_x/w |
| `src/renderer/layout.rs` | +9 / -0 (Phase B) | LayoutEngine Cell + trailing-ls 보정 |
| `src/renderer/layout/table_layout.rs` | +40 / -4 | effective_margin_left_line + 3 분기 |
| `src/renderer/layout/integration_tests.rs` | +200 / -0 | TDD 3 건 |
