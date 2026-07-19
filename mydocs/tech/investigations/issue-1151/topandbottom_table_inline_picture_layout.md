---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1151/README.md
last_verified: 2026-07-16
---

# Paragraph 안의 TopAndBottom 표 + sibling inline picture 의 layout 정합 — v3 root cause 분석

대상 task: [Task #1151 v3](../../../plans/archives/task_m100_1151_v3.md) · v2 model 정합: [hancom_picture_tac_toggle.md](hancom_picture_tac_toggle.md) · 관련 규칙: [table_layout_rules.md](../../table_layout_rules.md)

## 1. 문제 정의

같은 paragraph 의 sibling control 로 다음이 동시에 있을 때:
- `Control::Table` — `wrap=TopAndBottom`, `treat_as_char=false` (자리차지 표)
- `Control::Picture` — `treat_as_char=true`, `horz/vert_rel_to=Para`, `h/v_offset=0` (inline)

한컴은 picture 를 표 **아래** 영역에 inline 으로 배치하지만, rhwp 는 picture 와 표를 같은 paragraph 시작 y 좌표에서 동시에 그려 시각적으로 **오버랩** 발생.

검증 산출물:
- 모델: `samples/tac-verify/scenario-a-after.hwp` (한컴 산출물)
- 시각: `rhwp export-svg` 결과의 `<image>` 와 `<rect>` 좌표가 둘 다 `(113.39, 132.27)` = body 좌상단으로 일치 → 오버랩
- 사용자 직접 dev server 시각 확인으로 재현 (2026-05-29)

## 2. v2 model 정합 확인

v2 fix 의 `migrate_picture_floating_to_inline` 이 만든 model 은 한컴 산출물 model 과 정확 일치 (4 통합 테스트 PASS). 본 문제는 **mutation 영역이 아니라 renderer 영역**.

| 항목 | 한컴 산출물 | rhwp v2 토글 결과 | 일치 |
|------|-------------|--------------------|------|
| picture treat_as_char | true | true | ✓ |
| picture h/v_rel_to | Para | Para | ✓ |
| picture h/v_offset | 0, 0 | 0, 0 | ✓ |
| parent ls[0].line_height | picture height | picture height | ✓ |

## 3. Layout 알고리즘 추적

### 3-1. picture (tac=true) 자리 결정 — `paragraph_layout.rs`

- `paragraph_layout.rs:994-1003` 의 `tac_offsets_px` 가 paragraph 내 `tac=true` control 의 폭/위치를 수집.
- 라인 1540-1553 의 자리 계산은 **text flow 기반 (column_start, run 누적 x)** 으로 동작.
- `tac_offsets_px` 는 **sibling Table control 의 vertical offset 을 반영하지 않음**.

### 3-2. wrap=TopAndBottom 표의 vertical 배치 — `shape_layout.rs` + `pagination/engine.rs`

- `shape_layout.rs:2684-2782` 의 `calculate_shape_reserved_heights`:
  - `wrap == TopAndBottom && !treat_as_char` 인 control 만 `shape_reserved` 에 등록 → 표가 본문 흐름에서 차지하는 vertical 영역을 paragraph 의 y_offset 으로 반영.
  - 라인 2736 근방: `if matches!(common.vert_rel_to, VertRelTo::Para) { continue; }` — **vert_rel_to=Para 인 control 은 명시 제외**.
- `pagination/engine.rs:1707` 의 `paginate_table_control`:
  - non-TAC 표는 `place_table_fits` 에서 y_offset 정진 (별도 layout path).

### 3-3. 두 알고리즘의 결합 결함

| 결합 | 한컴 기대 | rhwp 현재 |
|------|-----------|-----------|
| paragraph 0.0 의 sibling Table + inline picture 둘 다 있음 | 표가 paragraph 위쪽 (vpos ≈ 13064) 부터 자리차지 → picture 가 그 다음 영역에 inline 배치 | 표와 picture 둘 다 paragraph 시작 y 에서 그려짐 (overlap) |
| picture y 결정 | sibling 표의 하단 y 까지 누적 | text flow 만 반영 (표 영역 무시) |
| shape_reserved 에 inline picture 영향 | (해당 없음 — picture 는 inline 글리프) | 영향 없음 (정상) |

**근본 원인**: `paragraph_layout` 의 `tac_offsets_px` 가 sibling non-TAC TopAndBottom 표가 차지한 vertical 영역을 inline picture 의 y 위치 결정에 반영하지 않음.

## 4. Fix 후보 비교

### Fix #1 — `shape_reserved` 의 vert_rel_to=Para 제외 로직 수정

- 위치: `src/renderer/layout/shape_layout.rs:2733-2738`
- 변경: vert_rel_to=Para inline picture 를 shape_reserved 에 포함 / 또는 표만 포함하도록 좁힘.
- 위험: 다른 vert_rel_to=Para 케이스 (예: 본문 floating Para-relative picture) 회귀 가능. 제외 로직이 원래 어떤 이유로 도입되었는지 추적 필요. **위험 ↑**.

### Fix #2 — `paragraph_layout` 에 sibling TopAndBottom 표 높이 누적 ★ 권장

- 위치: `src/renderer/layout/paragraph_layout.rs:1540-1560` 또는 그 상위 layout 결정 path.
- 변경: tac picture 의 y 계산 직전, 같은 paragraph 의 controls 중 `Control::Table` 의 `wrap=TopAndBottom && !treat_as_char` 인 항목들의 하단 y 를 누적 → picture.y 에 가산.
- 의사 코드:
  ```rust
  let tac_table_reserved_hu: i32 = para.controls.iter()
      .filter_map(|c| match c {
          Control::Table(t) if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
              && !t.common.treat_as_char => Some(t.common.height as i32),
          _ => None,
      })
      .sum();
  // picture.y += hwpunit_to_px(tac_table_reserved_hu, dpi);
  ```
- 위험: sibling 표의 정확한 차지 영역 (`outer_margin` 포함) 을 어떻게 계산하느냐가 관건. 단순 `common.height` 만 더하면 outer margin 미반영 가능. → 별도 helper `calc_table_reserved_below_y` 가 필요.
- 영향 범위: paragraph_layout 의 tac 위치 결정만. 다른 케이스 (본문 inline picture, sibling 표 없음) 회귀 0 예상.
- **권장 이유**: 지역적, 명시적, 다른 layout path 에 영향 없음.

### Fix #3 — `measure_paragraph` 의 vpos 의미 확장

- 위치: `src/renderer/layout/paragraph_layout.rs:1033-1046` (Task #1012 fallback)
- 변경: paragraph 첫 line_seg 의 vpos > 0 면 그 vpos 가 sibling 표 때문인지 판정 → picture 의 line offset 으로 사용.
- 위험: vpos 의 의미가 모호해짐. 다른 fallback 경로 회귀 가능. **위험 중**.

## 5. v3 권장 fix: Fix #2

Fix #2 의 구체 단계:

1. **새 helper 신설**: `calc_sibling_topandbottom_table_height_hu(controls: &[Control]) -> i32`
   - controls 순회 → `Control::Table` 중 `wrap=TopAndBottom && !treat_as_char` 인 항목의 `common.height + outer_margin.top + outer_margin.bottom` 누적.
2. **`tac_offsets_px` 또는 그 사용처 갱신**:
   - tac picture 의 line 자리 결정 시점에 helper 호출 → 결과 (HU) 를 px 환산하여 picture 의 y 위치에 가산.
3. **수직 정렬**: paragraph 의 line_segs[0].vpos 가 이미 vpos=13064 인 경우와 표 차지 영역의 결합을 명확히 (이중 가산 방지).
4. **단위 / 시각 테스트**:
   - `cargo test --lib` 회귀 0
   - SVG 좌표 단언: `scenario-a-after.svg` 의 picture y > table y + table height (한컴과 같은 시각)
   - 사용자 dev server 시각 재확인

## 6. 회귀 검증 케이스 (v3 Stage 4 의 단위/시각 테스트 항목)

| 케이스 | 기대 |
|--------|------|
| paragraph 안 TopAndBottom 표 + sibling inline picture (Scenario A 등가) | picture y > 표 y + 표 height + margin |
| 본문 inline picture (sibling 표 없음) | 기존 동작 그대로 (회귀 0) |
| 본문 TopAndBottom 표 (inline picture 없음) | 기존 동작 그대로 |
| 같은 paragraph 에 wrap=TAC 표 (treat_as_char=true) + inline picture | 기존 line_height 합산 path 사용 (Fix #2 영향 없음) |
| 본문 floating picture (wrap=Square 등) | 기존 동작 그대로 |

## 7. 한컴 산출물 SVG 단언 기준 (Scenario A)

dump 분석으로 확정된 한컴 정합 좌표:

| 요소 | x | y | 비고 |
|------|---|---|------|
| 표 자체 rect | 113.39 | 132.27 (body top) | 한컴 paragraph 0.0 시작점 |
| 표 height | — | 166.64 px | 표 outer 높이 |
| 표 하단 y | — | **298.91** | = 132.27 + 166.64 (대략, outer_margin 추가 가능) |
| Picture y (한컴 기대) | 113.39 | **≥ 298.91** | 표 하단 아래 |

현재 rhwp 의 picture y=132.27 → 표 위와 같은 y → 오버랩. Fix 후 picture y 가 표 하단 + outer_margin 이상으로 가야 정합.
