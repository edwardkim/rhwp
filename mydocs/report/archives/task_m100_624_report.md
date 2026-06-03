# Task #624 최종 보고서

**제목**: exam_science p2 7번 글상자 ㉠ 사각형 y-위치 회귀 정정 — Task #520 부분 회귀 복원
**마일스톤**: M100 (v1.0.0)
**브랜치**: `local/task624` (base: `pr-task618`)
**이슈**: https://github.com/edwardkim/rhwp/issues/624
**선행 task**: #520 (`313e65d`), #548 cherry-pick PR #561 (`3de0505`)

---

## 1. 요약

PR #561 cherry-pick (`3de0505`) 시 Task #520 정정 일부 (3 라인) 누락 회귀를 정정. exam_science p2 7번 글상자 ㉠ 사각형이 Line 1 영역 (y=213.95, 본문 "분자당 구성" 위 겹침) 에서 Line 2 영역 (y=235.41, " 이다." 앞) 으로 정확히 이동.

코드 변경: `src/renderer/layout/table_layout.rs` +9/-2 (3 라인 본질 + 주석).
검증: cargo test --lib 1135 passed (회귀 0) / 광범위 fixture sweep 158 fixture 1,496 페이지 / 1 페이지 의도된 정정 / 회귀 0.

## 2. 문제 정의

### 2.1 증상

`samples/exam_science.hwp` page 2 의 7번 문제 글상자 (pi=33 ci=0, 1x1 Table) 안 p[1] 단락의 ㉠ 사각형 (`Control::Shape`, `treat_as_char=true`, ls[1] 위치) 이 Line 1 영역 (y≈213.95 px) 에 잘못 그려져 본문 텍스트 "분자당 구성 …" 위에 겹쳐 보임.

기대: ㉠ 사각형이 Line 2 (y≈235.65 px) 에서 " 이다." 앞에 위치.

### 2.2 회귀 출처

| 시점 | 커밋 | Picture branch tac_img_y | Shape branch tac_img_y | shape_area.y | layout_cell_shape para_y |
|---|---|---|---|---|---|
| Task #520 | `313e65d` | `seg.vpos - first_vpos` ✓ | `seg.vpos - first_vpos` ✓ | `tac_img_y` ✓ | `tac_img_y` ✓ |
| Task #544 v2 Stage 3 (원저자 @planet6897) | `9dc40dd` | `seg.vpos - first_vpos` ✓ | `seg.vpos - first_vpos` ✓ | `tac_img_y` ✓ | `tac_img_y` ✓ |
| **Task #548 cherry-pick (PR #561)** | **`3de0505`** | **`seg.vpos` ✗ (회귀)** | `seg.vpos - first_vpos` ✓ | **`para_y_before_compose` ✗ (회귀)** | **`para_y_before_compose` ✗ (회귀)** |

`git diff 9dc40dd 3de0505 -- src/renderer/layout/table_layout.rs` 으로 누락 3 건 정확히 식별.

## 3. 정정 내용

### 3.1 Picture 분기 `tac_img_y` 산식 (`src/renderer/layout/table_layout.rs:1605~1607`)

```rust
// Before
if let Some(seg) = para.line_segs.get(target_line) {
    tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
}

// After
if let Some(seg) = para.line_segs.get(target_line) {
    let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
    tac_img_y = para_y_before_compose
        + hwpunit_to_px(seg.vertical_pos - first_vpos, self.dpi);
}
```

### 3.2 Shape 분기 `shape_area.y` (`src/renderer/layout/table_layout.rs:1814~1820`)

```rust
// Before
let shape_area = LayoutRect {
    x: inline_x,
    y: para_y_before_compose,
    ...
};
self.layout_cell_shape(..., para_y_before_compose, ...);

// After
let shape_area = LayoutRect {
    x: inline_x,
    y: tac_img_y,
    ...
};
self.layout_cell_shape(..., tac_img_y, ...);
```

## 4. 검증 결과

### 4.1 단위 / 통합 테스트

| 테스트 | 결과 |
|---|---|
| `test_624_textbox_inline_shape_y_on_line2_p2_q7` | RED → **GREEN** |
| `cargo test --lib` (전체) | **1135 passed / 0 failed / 2 ignored** |
| `cargo test --test svg_snapshot` | **6/6 passed** |
| `cargo clippy --lib -- -D warnings` | **clean** |

### 4.2 광범위 fixture sweep

158 sample × 평균 ~9.5 페이지 = **1,496 페이지** sweep:

| 카테고리 | 페이지 수 | 비고 |
|---|---|---|
| 의도된 정정 | **1** | exam_science_002.svg ㉠ 사각형 + ㉠ 텍스트 y +21.47 px |
| 회귀 | **0** | 다른 157 fixture 1,495 페이지 byte-identical |

### 4.3 정정 효과 (exam_science p2 7번 글상자 ㉠)

```diff
-<rect x="117.066" y="213.946" width="62.986" height="22.880" fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
-<text x="141.56"  y="229.986" ...>㉠</text>
+<rect x="117.066" y="235.413" width="62.986" height="22.880" fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
+<text x="141.56"  y="251.453" ...>㉠</text>
```

- `<rect>` y: **213.95 → 235.41** (Δ +21.47 px = (3220 - 1610)/75 = ls[1].vpos - ls[0].vpos 정확)
- `<text>㉠` y: **229.99 → 251.45** (Δ +21.47 px)

→ ㉠ 사각형 + 그 안 ㉠ 텍스트가 Line 2 영역으로 정확히 이동. 본문 "분자당 구성" 침범 해소.

## 5. 안전성 분석 (edge cases)

| 케이스 | 정정 후 동작 | 사유 |
|---|---|---|
| `first_vpos = 0` (cell 첫 paragraph) | 동일 동작 보장 | `seg.vpos - 0 = seg.vpos`, 산식 결과 동일 |
| `target_line = 0` (rect on ls[0]) | 동일 동작 보장 | `target_line > current_tac_line` 가드로 if 블록 미진입, `tac_img_y == para_y_before_compose` 초기값 유지 |
| `line_segs.len() = 1` (single-line) | 동일 동작 보장 | `composed.lines.len() = 1` 이면 `target_line = 0` 으로 위와 동일 |
| `wrap = Square / InFrontOfText / TopAndBottom` | 본 분기 영향 없음 | `treat_as_char=true` 만 처리, wrap 모드는 IR 보존만 |

→ 정정의 부수 효과 없음 — 정확히 회귀 사례만 해소.

## 6. 회귀 발생 4 조건 분석

`Control::Shape(shape) if shape.common().treat_as_char` 분기에서 다음 4 조건 모두 성립 시 회귀 가시화:

| 조건 | exam_science p[1] | exam_kor p[0] | synam-001 p[6/0/8] | 21_언어 p[0] | k-water p[0] |
|---|---|---|---|---|---|
| (a) cell 안 paragraph | ✓ | ✓ | ✓ | ✓ | ✓ |
| (b) multi-line (`line_segs.len() ≥ 2`) | ✓ (2) | ✓ (4) | ✓ (2/3/3) | ✓ (10) | ✓ (2) |
| (c) `target_line > 0` (rect on ls[1]+) | ✓ | ? | ? | ? | ? |
| (d) `first_vpos > 0` (paragraph[i>0]) | **✓ (1610)** | ✗ (0) | ✗/✗/✗ | ✗ (0) | ✗ (0) |
| **회귀 가시화** | **YES** | NO | NO | NO | NO |

→ exam_science p[1] 만이 (c) AND (d) 동시 성립 — 광범위 sweep 결과 (1/1496) 와 정확히 부합.

## 7. 후속 권고

1. **PR cherry-pick 시 base diff 자동 점검**: 본 결함은 `9dc40dd → 3de0505` 비교만으로 즉시 식별 가능. 동일 함수 영역의 산식이 base 와 cherry-pick 후 차이 발생 시 경고 도입.
2. **회귀 테스트 가드**: 본 task 의 `test_624_textbox_inline_shape_y_on_line2_p2_q7` 가 RED → GREEN 가드 역할. 추가 cherry-pick 시 본 테스트 RED 발생하면 즉시 차단.
3. **`local/devel` ↔ `devel` 동기화 점검**: 173 commit 차이의 다른 영역에도 유사 회귀 가능. `local/devel` ahead 인 fix 들 (Task #520 외) 도 `devel` 에 정확히 반영되어 있는지 검증 권고.

## 8. 산출물

### 8.1 코드 변경

| 파일 | LOC | 비고 |
|---|---|---|
| `src/renderer/layout/table_layout.rs` | +9 / -2 | Picture 산식 + Shape 위치 (3 라인 본질 + 주석) |
| `src/renderer/layout/integration_tests.rs` | +59 | Stage 1 RED 테스트 |

### 8.2 문서

| 문서 | 단계 |
|---|---|
| `mydocs/plans/task_m100_624.md` | Stage 0 — 수행 계획서 |
| `mydocs/plans/task_m100_624_impl.md` | Stage 0 — 구현 계획서 |
| `mydocs/working/task_m100_624_stage1.md` | Stage 1 — TDD RED |
| `mydocs/working/task_m100_624_stage2_analysis.md` | Stage 2 (분석) |
| `mydocs/working/task_m100_624_stage2.md` | Stage 2 (정정) |
| `mydocs/report/task_m100_624_report.md` | Stage 3 — 본 보고서 |
| `mydocs/orders/20260506.md` | 오늘 할일 갱신 |

### 8.3 commit chain (`local/task624`)

```
4b094b6 Task #624 Stage 2: Task #520 부분 회귀 정정 (3 line)
aee08d6 Task #624 Stage 2 분석 보고서 (코드 정정 보류)
bdb6671 Task #624 Stage 1: TDD RED 통합 테스트
1237a78 Task #624 Stage 0: 수행 계획서 + 구현 계획서
```

## 9. 결론

PR #561 cherry-pick 시 Task #520 의 일부 정정 누락 회귀 (3 라인) 를 복원. 광범위 1,496 페이지 sweep 으로 회귀 0건 확인. 정정 안전성 (edge case 4종) 정합 — 의도된 1 페이지만 변경, 다른 1,495 페이지 byte-identical.

`closes #624`.
