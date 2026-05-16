# Task #928 Stage 2: 코드 trace + ROOT CAUSE 확정 + Fix 방향

## 1. ROOT CAUSE 확정

회귀 위치: **`src/renderer/layout/table_layout.rs::layout_table_cells`** (라인 1253~).

해당 함수는 셀 paragraph 마다 다음 순서로 처리한다:

| 단계 | 라인 | 동작 |
|------|------|------|
| ① | 1625 | `layout_composed_paragraph()` 호출 — paragraph 텍스트를 `run_tacs` split 으로 inline 발행 (TAC 컨트롤 위치 좌표 기록은 `set_inline_shape_position`) |
| ② | 1691 | `para.controls.iter()` 두 번째 컨트롤 루프 — Picture / Shape / Equation / Table 별 분기 |
| ③ | 2158 | "마지막 인라인 Shape 이후의 남은 텍스트 렌더링" 블록 — `prev_tac_text_pos` 기반 trailing 텍스트 발행 |

### Picture 분기 (라인 1693-1769) — 가드 존재

```rust
let will_render_inline = composed.tac_controls.iter().any(|&(abs_pos, _, ci)| {
    ci == ctrl_idx && composed.lines.iter().any(|line| {
        let line_chars: usize = line.runs.iter().map(|r| r.text.chars().count()).sum();
        abs_pos >= line.char_start && abs_pos < line.char_start + line_chars
    })
});
if !will_render_inline {
    // 수동 inline 발행
} else {
    inline_x += pic_w;  // 단순 x 진행만
}
```

→ ① 단계에서 `layout_composed_paragraph` 가 picture 를 이미 inline 처리한 경우 수동 재발행 스킵. 같은 셀 p[2] 의 그림 3개가 회귀 없는 이유.

### Shape 분기 (라인 1812-1963) — **가드 없음**

```rust
Control::Shape(shape) => {
    if shape.common().treat_as_char {
        // target_line / inline_x / tac_img_y 계산
        // ...
        // text_before 추출 + TextRunNode emit (1860-1947)
        // layout_cell_shape 호출 (1959)
    }
}
```

→ ① 단계에서 이미 inline 발행했는지 확인하지 않고 **항상** `text_before` 발행. 회귀 발생.

### 트레일링 텍스트 블록 (라인 2158-2231) — **가드 없음**

```rust
if prev_tac_text_pos > 0 {
    // remaining_text 추출 + TextRunNode emit
}
```

→ Shape 분기가 `prev_tac_text_pos = tac_pos` 설정 후 이 블록이 마지막 shape 이후의 텍스트를 다시 발행.

## 2. 회귀 메커니즘 (Δy=1.39px 의 출처)

- ① 의 `layout_composed_paragraph` 가 paragraph baseline = y=421.73 (정상 위치) 에 `(가) ⇨ [shape 자리] ⇨ (나)` 전체 발행 (run_tacs split 결과)
- ②③ 가 새로 계산한 baseline = `text_y = para_y_before_compose + (adjacent_shape_h - font_line_h).max(0.0)` (라인 1921) 에 text_before / remaining 발행 → y=423.12 (1.39px 아래)
- 두 발행이 평행이동 (Δx ≈ +53) 으로 보이는 이유: ②③ 가 `inline_x` 를 좌측 padding + indent 부터 다시 누적하기 때문 (① 의 정렬 결과와 시작 x 가 다름)

## 3. 검증: p[2] 그림 vs p[1] 사각형 차이

| paragraph | 컨트롤 종류 | will_render_inline 가드 | 결과 |
|-----------|------------|------------------------|------|
| p[1] | Shape (사각형) | ❌ 없음 | 텍스트 2회 발행 (회귀) |
| p[2] | Picture × 3 | ✅ 있음 (라인 1698) | 텍스트 1회 발행 (정상) |

→ ROOT CAUSE 가설 A (paragraph_layout 의 inline TAC split + table_layout 의 수동 재발행 동시 발생) 확정.

## 4. Fix 방향 후보

### 방향 α (권장): Shape 분기에 will_render_inline 가드 추가

Picture 분기와 동일 패턴으로 Shape 분기에 `will_render_inline` 가드 추가.

```rust
Control::Shape(shape) => {
    if shape.common().treat_as_char {
        let will_render_inline = composed.tac_controls.iter().any(|&(abs_pos, _, ci)| {
            ci == ctrl_idx && composed.lines.iter().any(|line| {
                let line_chars: usize = line.runs.iter().map(|r| r.text.chars().count()).sum();
                abs_pos >= line.char_start && abs_pos < line.char_start + line_chars
            })
        });
        if !will_render_inline {
            // 기존 text_before + layout_cell_shape 경로 유지
        } else {
            inline_x += shape_w;  // x 진행만, 텍스트/도형 모두 ①에서 처리됨
            // 단 도형 자체 렌더링이 ① 에서 누락된다면 별도 호출 필요
        }
    }
}
```

**위험**: `will_render_inline=true` 일 때 도형 자체가 ①에서 렌더되지 않으면 사각형이 사라진다. Stage 3 진입 전 검증 필요:
- paragraph_layout 의 `set_inline_shape_position` 호출이 실제 도형 렌더링을 트리거하는지 (shape_layout 별도 패스 호출 여부)
- 셀 내부 도형의 경우 별도 패스 호출 경로 존재 여부

### 방향 β: 트레일링 텍스트 블록도 가드

라인 2158 의 trailing 텍스트 블록도 inline-already-rendered 가드 추가. 방향 α 와 짝.

### 방향 γ (비권장): paragraph_layout 의 inline TAC split 을 Shape 에 대해 비활성화

표 셀 내부에서 Shape tac=true 일 때 paragraph_layout 이 run_tacs split 을 안 하도록 조건 추가. 회귀 영향이 크고 본문 paragraph 케이스가 미상이라 비권장.

→ **방향 α + β 조합 선택**. Stage 3 에서 verify-then-fix.

## 5. 구현 계획서

별도 파일 `task_m100_928_impl.md` 에서 3~6 단계 구성.

## 6. Stage 2 결정 사항

- ✅ ROOT CAUSE 코드 위치 확정: `src/renderer/layout/table_layout.rs:1812-1963` (Shape 분기 가드 누락) + `:2158-2231` (트레일링 텍스트 가드 누락)
- ✅ Fix 방향 α+β 선택
- ⏳ Stage 3 진입 전: 셀 내부 inline Shape 도형 렌더링 경로 검증 (가드 true 일 때 도형 사라짐 회귀 차단)
