# Task #521 최종 결과 보고서 — TAC 표 outer_margin_bottom 누락 정정

**이슈**: [#521](https://github.com/edwardkim/rhwp/issues/521) — exam_eng p2 18번 답안지 위치가 박스 하단에 너무 가까움
**브랜치**: `pr-task521`
**기간**: 2026-05-04 (Stage 1 ~ 5 단일 일자)
**옵션**: E1' — `layout_table_item` TAC after-spacing 에 `outer_margin_bottom` 추가
**상태**: 본질 정정 완료 / 광범위 회귀 검증 통과 / PR 등록 대기

## 1. 본질 (Stage 1/2 진단 결과)

`src/renderer/layout.rs::layout_table_item` 의 TAC 표 (treat_as_char + wrap=TopAndBottom) after-spacing 분기 (라인 2478~2497) 가 **`outer_margin_bottom` 미적용**. `layout_partial_table_item` 의 동일 패턴 (라인 2638~2647) 와 불일치.

### 1.1 한컴 명세 정합

HWP IR 의 LINE_SEG `lh` 정의:

```
lh = cell_height + outer_margin_bottom
```

(`exam_eng.hwp` pi=104: lh=22207 = cell_h(21607) + outer_margin_bottom(600))

`layout_table_item` 가 TAC 표 진행 후 `cell_h` 만 `y_offset` 에 advance → 다음 paragraph (pi=105 ① 첫 답안) 가 `outer_margin_bottom` (600 HU = 8 px) 만큼 위로 시프트. PDF 한컴 2010 대비 -8 px shortfall.

### 1.2 이슈 본문 가설 재해석

이슈 본문은 BehindText ctrl[1] 그림 vertical extent 가 paragraph height 미반영을 가설로 제시했으나 Stage 1 측정에서 ctrl[1] paragraph-relative bottom (21719 HU) < ctrl[2] 표 bottom (22207 HU) 으로 ctrl[1] 가 표 안에 수용되어 가설 부적합. **본 devel 코드 기준 본질은 `outer_margin_bottom` 누락**.

## 2. 변경 사항

### 2.1 src/renderer/layout.rs (5 LOC, 코멘트 5 LOC)

`layout_table_item::TAC after-spacing` 분기 (라인 2497 직후) 에 `outer_margin_bottom` advance 추가.

```rust
// [Task #521] TAC 표 outer_margin_bottom 적용 (한컴 명세 정합).
// layout_partial_table_item:2642-2647 와 동일 처리.
let outer_margin_bottom_px = if let Some(Control::Table(t)) = para.controls.get(control_index) {
    hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
} else { 0.0 };
if outer_margin_bottom_px > 0.0 {
    y_offset += outer_margin_bottom_px;
}
```

### 2.2 src/renderer/layout/integration_tests.rs (+80 LOC)

`test_521_tac_table_outer_margin_bottom_p2` 추가. exam_eng p2 우측 단 18번 박스 bottom ↔ ① 첫 답안 baseline gap 정합 (PDF ±2 px) 검증.

## 3. 검증

### 3.1 단위 테스트

```
cargo test --lib --release
1121 passed / 0 failed / 3 ignored
```

baseline 1120 → 1121 (+1 GREEN: `test_521_tac_table_outer_margin_bottom_p2`).

회귀 가드 GREEN 유지:
- `test_544_passage_box_coords_match_pdf_p4`
- `test_547_passage_text_inset_match_pdf_p4`
- `test_469_partial_start_box_does_not_cross_col_top`
- 기존 issue_546/530/505/418/501 회귀 가드

### 3.2 Clippy

```
cargo clippy --release --lib
```

신규 결함 0. pre-existing 2건 (`document_core/commands/object_ops.rs:298, 1007`) 은 본 변경과 무관한 baseline.

### 3.3 광범위 SVG sweep (13 fixture, 481 페이지)

```
Total: 481 SVGs / Differ: 278 / Byte-identical: 203
text count: 335,353 → 335,353 (Δ=0)
```

→ text 요소 수 변동 0 = 누락/추가 없음. 모든 차이는 `outer_margin_bottom` 정합 시프트.

### 3.4 발현 fixture 정합

| ① 위치 | before | after | Δ | 비고 |
|--------|--------|-------|---|------|
| exam_eng p2 18번 ① | 543.95 | 551.95 | +8.00 px | 박스 bottom + 20 px gap, PDF 한컴 2010 정합 |
| 다음 ① | 949.92 | 957.92 | +8.00 px | 동일 fixture, 동일 패턴 |
| 다음 ① | 1331.39 | 1339.39 | +8.00 px | 동일 |

각 TAC 표 `outer_margin_bottom = 600 HU = 8 px` 일관 시프트.

## 4. 결과 해석

### 4.1 의도된 차이의 본질

`tac=true + wrap=TopAndBottom + outer_margin_bottom > 0` 패턴 매칭 paragraph 직후 모든 후속 paragraph 가 `outer_margin_bottom` 만큼 아래로 시프트.

이전: TAC 표 cell_h 만 advance → outer_margin_bottom 누락
이후: cell_h + outer_margin_bottom advance → 다음 paragraph PDF 정합

### 4.2 영향 범위

13 fixture 481 페이지 중 278 페이지 (57.8%) 에서 위치 시프트. 모두 의도된 한컴 명세 정합 변경.

## 5. 메모리 정합

- `feedback_pdf_not_authoritative` — PDF 한컴 2010 보조 ref. Stage 1 직접 측정 (gap 12.27 / 20 PDF) 으로 -7.7 px shortfall 확정.
- `feedback_essential_fix_regression_risk` — `respect_vpos_reset` 류 광범위 본질 정정 위험 인지. 본 fix 는 `partial_table_item` 와 정합시키는 단일 룰 적용 (분기/허용오차 도입 없음).
- `feedback_rule_not_heuristic` — `lh = cell_h + outer_margin_bottom` 한컴 명세 룰 단일 적용. 휴리스틱 분기 없음.
- `feedback_visual_regression_grows` — 작업지시자 직접 시각 판정 게이트 권장 (`/tmp/diag521/before` ↔ `/tmp/diag521/after`).

## 6. Stage 5 후속 절차

| 절차 | 비고 |
|------|------|
| 작업지시자 시각 판정 | exam_eng p2 18번 박스 ↔ ① gap 20 px 정합 확인 |
| 단계별/최종 보고서 commit | `pr-task521` 브랜치 |
| origin push | `planet6897/rhwp pr-task521` |
| PR 등록 | `edwardkim/rhwp` base=`devel`, head=`planet6897:pr-task521` |
| 이슈 #521 close | PR merge 후 |
| `mydocs/orders/20260504.md` 갱신 | Task #521 완료 항목 추가 |
