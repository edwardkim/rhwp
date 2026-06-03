# Task #1151 v3 Stage 3 완료 보고서 — paragraph 의 sibling TopAndBottom 표 + inline picture 시각 layout 정합

수행계획서: [task_m100_1151_v3.md](../plans/task_m100_1151_v3.md) · 구현계획서: [task_m100_1151_v3_impl.md](../plans/task_m100_1151_v3_impl.md) · v3 root cause: [topandbottom_table_inline_picture_layout.md](../tech/topandbottom_table_inline_picture_layout.md)

## 1. 진입점 진단 (Stage 3-1)

`samples/tac-verify/scenario-{a..d}-after.hwp` 의 paragraph 0.0 이 렌더링되는 path 추적 결과:

- **paragraph_layout.rs 의 3개 picture 분기 (2647 / 3026 / 3164)** — 진입 안 됨 (debug println 으로 검증).
- **`src/renderer/layout.rs:4514-4518` 의 Task #347 fallback path** — 진입. paragraph_layout 미진입 시 (text="" + tac picture) layout.rs 가 직접 ImageNode emit.
- `pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset)` (라인 4518) → paragraph 시작 y 사용 → sibling 표가 차지하는 영역 무시 → 표와 같은 시작점에서 picture 그려져 **오버랩**.

## 2. 변경 내용 (Stage 3-2)

### `src/renderer/layout/paragraph_layout.rs`

#### 신규 helper — `calc_sibling_topandbottom_table_reserved_hu`

`pub(crate)` 자유 함수 (라인 4138 근방):

```rust
pub(crate) fn calc_sibling_topandbottom_table_reserved_hu(
    controls: &[crate::model::control::Control],
) -> i32 {
    controls.iter().map(|c| match c {
        Control::Table(t) if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
            && !t.common.treat_as_char =>
        {
            t.common.height as i32
                + t.outer_margin_top as i32
                + t.outer_margin_bottom as i32
        }
        _ => 0,
    }).sum()
}
```

처리:
- controls 순회 → `Control::Table` 중 `wrap=TopAndBottom && !tac` 인 항목만 추출.
- 합산: `common.height + outer_margin_top + outer_margin_bottom` (HWPUNIT).
- 다른 모든 control (Table+TAC, Square wrap 표, Picture, Shape 등) → 0.
- 표가 없으면 0 반환 → fix 적용 path 의 가산값=0 → 기존 동작 보존 (회귀 0).

#### paragraph_layout 내부의 추가 fix (라인 2647 근방)

text run 처리 중 inline picture path 에도 같은 helper 호출 + img_y 가산. 본 task 의 핵심 케이스 (빈 paragraph) 는 layout.rs:4514 path 이지만, paragraph 에 text 가 있는 + sibling 표 + inline picture 케이스의 정합도 같이 보강.

### `src/renderer/layout.rs:4514` — Task #347 fallback path 의 pic_y 보정

```rust
if let Control::Picture(pic) = ctrl {
    if pic.common.treat_as_char {
        let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
        let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
        // [Task #1151 v3] sibling wrap=TopAndBottom 표 (tac=false) reserved 영역 가산
        let sibling_table_reserved_hu =
            super::layout::paragraph_layout::calc_sibling_topandbottom_table_reserved_hu(&para.controls);
        let sibling_table_reserved_px = hwpunit_to_px(sibling_table_reserved_hu, self.dpi);
        let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset)
            + sibling_table_reserved_px;
        ...
    }
}
```

helper 호출이 fallback path 의 picture y 계산에 한 줄 추가만. sibling 표가 없으면 reserved=0 → pic_y 변화 없음 (회귀 0).

### 단위 테스트 (`#[cfg(test)] mod issue_1151_v3_helper_tests`)

paragraph_layout.rs 의 helper 옆에 6 테스트 추가:

| 테스트 | 시나리오 | 단언 |
|--------|---------|------|
| `topandbottom_table_reserved_single` | scenario-a 등가 (12498 + 283 + 283) | == 13064 ✓ |
| `topandbottom_table_reserved_none_when_no_table` | 표 없음 | == 0 (회귀 가드) |
| `topandbottom_table_reserved_excludes_tac_table` | TAC 표 | == 0 |
| `topandbottom_table_reserved_excludes_square_wrap` | wrap=Square 표 | == 0 |
| `topandbottom_table_reserved_ignores_picture_control` | Picture sibling | == 0 |
| `topandbottom_table_reserved_sums_multiple_tables` | 표 2개 | == 합산 |

## 3. SVG 좌표 시각 검증 (Stage 3-3)

`./target/debug/rhwp export-svg samples/tac-verify/scenario-*-after.hwp` 의 image y 변화 (v3 fix 전 후):

| 시나리오 | before (fix 전) | after (fix 후) | 변화량 (px) | 합산 정합 (HU) |
|---------|------------------|------------------|---------------|------------------|
| A (1×1 작은) | 132.27 | **306.45** | +174.18 | 12498 + 283 + 283 = 13064 HU = 174.19 px ✓ |
| B (1×1 큰) | 132.27 | **328.79** | +196.52 | 14739 HU (≈ 14173 + 283 + 283) |
| C (3×3 중앙) | 132.27 | **511.83** | +379.56 | 28467 HU (3 row × 표 reserved) |
| D (본문, 표 없음) | 132.27 | **132.27** | 0 | 0 HU (회귀 0) ✓ |

### 한컴 산출물 정합 단언 (Scenario A)

- 표 위치: y=132.27, height=166.64 → 표 끝 (페이지) y = 298.91
- 표 outer_margin bottom = 1.0mm = 283 HU = 3.77 px → 표 끝 + margin = 302.68
- v3 fix 후 picture y = **306.45 > 302.68** ✓ (한컴 정합)

## 4. 자동 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib issue_1151_v3_helper` | **6 passed, 0 failed** ✓ |
| `cargo test --lib` 전수 | **1442 passed, 0 failed, 6 ignored** (Stage 2 1436 + v3 helper 6, 회귀 0) ✓ |
| `cargo clippy --lib -- -D warnings` | clean ✓ |
| `cargo fmt --all -- --check` | clean ✓ |
| v2 통합 테스트 (`integration_tac_toggle_matches_hancom_scenario_{a..d}`) | 4/4 PASS (model 정합 유지) |
| Scenario D 회귀 가드 | image y 132.27 불변 ✓ |

## 5. Stage 4 진입 조건

- helper + fix 시그니처 확정 ✓
- 본문/v1/v2 회귀 0 ✓
- 4 시나리오 SVG 좌표 한컴 정합 ✓
- helper 단위 검증 통과 ✓

→ Stage 4 (WASM 재빌드 + rhwp-studio dev server 재시연 + 한컴 편집기 시각 1:1 비교) 진행 가능.
