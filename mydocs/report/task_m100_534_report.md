# Task #534 최종 보고서 — exam_kor 18p Square wrap 표 + TAC 그림 위치 결함

**작성일**: 2026-05-02
**이슈**: [#534](https://github.com/edwardkim/rhwp/issues/534)
**브랜치**: `local/task534`
**최종 commit**: `4abee04`

## 1. 요약

> exam_kor.hwp 18 페이지 우측 단 [A]/[B] 표시기 옆 인라인 그림이 단 영역을 30-70% 침범하는 결함 (`작업지시자 분석 요청 — 18 페이지 오른쪽 [A], [B] 와 글상자/그림(?)의 위치가 PDF와 상이함`) 의 본질을 **layout_shape_item 의 TAC Picture x 좌표 계산이 paragraph_layout 의 inner padding 로직과 비정합** 으로 확정. wrap_host paragraph 는 paragraph_layout 미호출되어 layout_shape_item 만 emit → inner_pad 누락된 위치 (~11.33 px 좌측 시프트). `src/renderer/layout.rs::layout_shape_item` 의 TAC Picture 분기에 inner_pad 가산 (+19 라인) 으로 정정. lib 1116 / svg_snapshot 6 / issue_418/501 회귀 0, 광범위 8 샘플 192 페이지 중 190 byte-identical, 변경 2 페이지 모두 의도된 정정.

## 2. 본질 발견 흐름

| 단계 | 발견 |
|------|------|
| 사용자 분석 요청 | "18 페이지 오른쪽 [A], [B] 와 글상자/그림(?)의 위치가 PDF와 상이함" |
| Stage 1 1차 측정 | image x: pi=46/54 (단독 그림) = 654.05, pi=50/56 (표+그림) = 593.39, diff=60.66 px |
| Stage 1 가설 다중 영역 | A. 다른 emit 경로 / B. HWP IR i32 wraparound / C. wrap_area 영향 |
| Stage 1 추가 조사 (eprintln) | **가설 A 확정** — paragraph_layout 가 emit (단독), layout_shape_item 가 emit (wrap_host) |
| 본질 정정 | inner_pad_left (visible stroke + border_spacing[0,1]=0 케이스) 가 layout_shape_item 에 누락 |

## 3. 변경

### 3-1. 코드 (`src/renderer/layout.rs::layout_shape_item`)

TAC Picture 분기에 inner_pad 가산 (paragraph_layout.rs::layout_composed_paragraph line 711-716 와 정합):

```rust
let para_border_fill_id_pre = para_style_ref.map(|s| s.border_fill_id).unwrap_or(0);
let has_visible_stroke = if para_border_fill_id_pre > 0 {
    let idx = (para_border_fill_id_pre as usize).saturating_sub(1);
    styles.border_styles.get(idx)
        .map(|bs| bs.borders.iter().any(|b|
            !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0))
        .unwrap_or(false)
} else { false };
let bs_left_px = para_style_ref.map(|s| s.border_spacing[0]).unwrap_or(0.0);
let bs_right_px = para_style_ref.map(|s| s.border_spacing[1]).unwrap_or(0.0);
let inner_pad_left = if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
    para_margin_left
} else {
    0.0
};
let effective_margin_left = if para_indent > 0.0 {
    para_margin_left + para_indent + inner_pad_left
} else {
    para_margin_left + inner_pad_left
};
```

### 3-2. 변경량

| 영역 | 추가 | 삭제 | 수정 |
|------|------|------|------|
| `src/renderer/layout.rs` | 19 | 0 | 4 (effective_margin_left 분기) |

## 4. 회귀 차단 가드

| 가드 | 보호 영역 |
|------|----------|
| `has_visible_stroke` (border_fill_id > 0 + 실제 stroke 있는 line_type/width) | 테두리 없는 paragraph 영향 없음 |
| `bs_left_px == 0.0 && bs_right_px == 0.0` | 직접 border_spacing 보유 paragraph 영향 없음 |
| `tac=true` 분기 | non-TAC Picture 영향 없음 |
| `paragraph_layout 우선 emit` | 단독 TAC Picture (paragraph_layout 가 먼저 emit) 는 already_registered=true 로 skip |

## 5. 검증 결과

### 5-1. 단위/통합 테스트

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed** |
| `cargo test --test svg_snapshot` | 6/6 |
| `cargo test --test issue_418` | 1/1 |
| `cargo test --test issue_501` | 1/1 |

### 5-2. 광범위 샘플 회귀

```bash
scripts/svg_regression_diff.sh build b669ab5 4abee04 ...
```

| 샘플 | total | same | diff |
|------|-------|------|------|
| 2010-01-06 / aift / exam_eng | 91 | 91 | 0 |
| **exam_kor** | **20** | **18** | **2** (p18 + p19) |
| exam_math / exam_math_no / exam_science / synam-001 | 81 | 81 | 0 |
| **합계** | **192** | **190** | **2** |

### 5-3. 시각 검증

| 위치 | image x (수정 전) | image x (수정 후) | shift |
|------|------------------|------------------|-------|
| p18 pi=50 (Square 표 + 그림) | **593.39** | **604.72** | **+11.33 px ✓** |
| p18 pi=56 (Square 표 + 그림) | 593.39 | 604.72 | +11.33 px ✓ |
| p19 pi=60 (Square 표 + 그림) | 128.51 | 139.84 | +11.33 px ✓ |

→ 모두 inner_pad (= box_margin_left = 11.33 px) 만큼 정합 회복.

## 6. 산출물

| 산출물 | 위치 |
|--------|------|
| 수행계획서 | `mydocs/plans/task_m100_534.md` |
| Stage 1 보고서 (Root cause 1차 + 추가 조사) | `mydocs/working/task_m100_534_stage1.md` |
| Stage 3 보고서 (코드 적용 + 회귀) | `mydocs/working/task_m100_534_stage3.md` |
| **본 최종 보고서** | `mydocs/report/task_m100_534_report.md` |
| 코드 변경 | `src/renderer/layout.rs` (+19 라인) |

## 7. 본질 학습

### 7-1. paragraph_layout vs layout_shape_item 정합

TAC Picture 의 x 좌표는 두 경로에서 emit 가능:
- **paragraph_layout**: 텍스트 있는 paragraph 또는 빈 문단 + TAC 만 있는 일반 paragraph
- **layout_shape_item**: wrap_host paragraph (Square wrap 표 보유, paragraph_layout 미호출)

두 경로의 effective_margin_left 계산은 **반드시 정합** 해야 함. 본 결함은 paragraph_layout 의 inner_pad 로직 (Task #?347 근방, visible stroke 보유 paragraph 의 border 안쪽 padding) 이 layout_shape_item 에 미적용된 케이스.

### 7-2. wrap_host paragraph 의 미고려 영역

is_wrap_host 분기에서 layout_paragraph 미호출은 의도된 동작 (wrap host 텍스트는 layout_wrap_around_paras 가 처리). 그러나 빈 문단 + TAC 그림 케이스는 이 분기로 인해 paragraph_layout 의 빈 문단 + TAC Picture emit 경로 (line 2153-2222) 도 미작동 → layout_shape_item fallback 의 정확성에 의존.

→ wrap_host paragraph 의 controls 처리는 fallback 경로 (layout_shape_item) 의 정합성 더 중요. 향후 유사 결함 (wrap_host + 다른 inline controls) 디버깅 시 본 패턴 확인 필요.

### 7-3. 메모리 정합

- `feedback_essential_fix_regression_risk` — Stage 1 의 가설 다중 영역 식별 → eprintln 추가 조사 후 단일 root cause 확정 → 회귀 위험 최소 fix 도출 (회귀 0)
- `feedback_rule_not_heuristic` — paragraph_layout 의 inner_pad 룰 (visible stroke + border_spacing[0,1]=0) 단일 룰 1:1 정합 적용. 분기/허용오차 없음

## 8. close 흐름

작업지시자 시각 판정 통과 시:
1. local/task534 → local/devel merge
2. local/devel → devel merge + push
3. issue #534 close
