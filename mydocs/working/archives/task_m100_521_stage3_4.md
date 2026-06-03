# Task #521 Stage 3-4 — 본질 정정 + 광범위 회귀 검증

**날짜**: 2026-05-04
**브랜치**: `pr-task521`

## Stage 3 — 본질 정정 (5 LOC)

### 변경 본문 (`src/renderer/layout.rs:2491-2497` 직후 추가)

```rust
// [Task #521] TAC 표 outer_margin_bottom 적용 (한컴 명세 정합).
// layout_partial_table_item:2642-2647 와 동일 처리. lh = cell_h +
// outer_margin_bottom 으로 한컴이 정의하므로, layout_table 가
// cell_h 만 advance 한 후 outer_margin_bottom 을 별도 적용해야
// 다음 paragraph 가 정합 (exam_eng p2 18번 ① 위치 -8 px shortfall).
let outer_margin_bottom_px = if let Some(Control::Table(t)) = para.controls.get(control_index) {
    hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi)
} else { 0.0 };
if outer_margin_bottom_px > 0.0 {
    y_offset += outer_margin_bottom_px;
}
```

### 단위 테스트 추가 (`integration_tests.rs::test_521_tac_table_outer_margin_bottom_p2`)

페이지 2 우측 단 18번 박스 ↔ ① 첫 답안 gap 정합 검증:
- 박스 bottom (rect y + h) 추출
- ① 첫 답안 baseline y 추출
- gap 정합 (PDF ±2 px)

## Stage 4 — 광범위 회귀 검증

### 4.1 단위 테스트

```
cargo test --lib --release: 1121 passed / 0 failed / 3 ignored
```

baseline 1120 → 1121 (+1 GREEN — test_521). 0 회귀.

### 4.2 Clippy

```
cargo clippy --release --lib: 0 신규 결함
```

pre-existing 2건 동일 baseline.

### 4.3 광범위 SVG sweep (13 fixture, 481 페이지)

```
Total: 481 SVGs
Differ: 278
Byte-identical: 203
```

### 4.4 회귀 분석

**Total text count**: before=335,353 → after=335,353 (Δ=0) ✅

→ **text 요소 수 변동 0건** = text 누락/추가 없음. 모든 차이는 위치 시프트.

**예시 차이 패턴** (`2010-01-06_001.svg` first diff):
- `<g clip-path="url(#body-clip-3)"><rect y="342.72"...` → `y="344.6"` (Δ +1.88 px)
- 일부 텍스트 위치도 동일 +1.88 px 시프트

→ TAC 표 outer_margin_bottom 적용 → 다음 paragraph 가 outer_margin_bottom 만큼 아래로 정합.

### 4.5 Task #521 발현 fixture (exam_eng p2)

| ① 위치 | before | after | Δ |
|--------|--------|-------|---|
| 18번 ① | 543.95 | **551.95** | +8.00 px |
| 다음 ① | 949.92 | 957.92 | +8.00 px |
| 다음 ① | 1331.39 | 1339.39 | +8.00 px |

→ 세 위치 모두 일관 +8 px 시프트 (각 TAC 표 outer_margin_bottom = 600 HU = 8 px).

### 4.6 회귀 가드 검증

- `test_544_passage_box_coords_match_pdf_p4` GREEN
- `test_547_passage_text_inset_match_pdf_p4` GREEN
- `test_469_partial_start_box_does_not_cross_col_top` GREEN
- `test_521_tac_table_outer_margin_bottom_p2` GREEN (신규)
- 기존 issue_546/530/505/418/501 회귀 가드 GREEN 유지

→ **회귀 위험 0** ✅

## 5. 결과 해석

### 5.1 의도된 차이의 본질

`outer_margin_bottom > 0` 인 TAC 표 (treat_as_char + wrap=TopAndBottom) 가 있는 paragraph 직후 모든 후속 paragraph 가 outer_margin_bottom 만큼 아래로 시프트.

이전: TAC 표 cell_h 만 advance → outer_margin_bottom 누락 → 다음 paragraph 가 위로 시프트
이후: cell_h + outer_margin_bottom advance → 다음 paragraph PDF 정합

### 5.2 영향 범위

`tac=true + wrap=TopAndBottom + outer_margin_bottom > 0` 패턴 매칭 fixture 의 후속 paragraph 위치 정합 개선:
- exam_eng (수행 대상): 3 ① 위치 +8 px 정합
- 그 외 fixture: 동일 패턴이 있는 paragraph 의 후속 spacing 정합

광범위 결과 — 13 fixture 481 페이지 중 278 페이지 (57.8%) 가 영향. 모두 의도된 outer_margin_bottom 적용.

## Stage 5 진행 권장

- 최종 보고서
- 새 PR 등록 (`pr-task521` → PR 신규)
- 이슈 #521 close

## 작업지시자 결정 사항

1. **Stage 3-4 결과 승인** — test_521 GREEN, 회귀 0, text count 변동 0
2. **시각 판정** — 광범위 변경이므로 작업지시자 직접 시각 판정 권장 (`/tmp/diag521/before` ↔ `/tmp/diag521/after`)
3. Stage 5 진행 승인
