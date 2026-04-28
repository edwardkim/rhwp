# Task #402 Stage 2: 수정 구현

## 변경 요약

원래 구현계획에서는 `layout.rs::layout_shape_item` 한 곳만 수정할 예정이었으나, 실제 검증 과정에서 **페이지네이션 엔진(typeset.rs)에도 동일 패턴의 누락이 있음**을 발견하여 두 곳을 함께 수정했다.

| # | 파일 | 함수 | 역할 |
|---|------|------|------|
| 1 | `src/renderer/layout.rs` | `layout_shape_item` | inline TAC 그림의 y 좌표 결정 — 선행 TAC 컨트롤이 있으면 `para_start_y`를 진행된 `y_offset`으로 갱신 |
| 2 | `src/renderer/typeset.rs` | `typeset_table_paragraph` | 페이지 분할 — 선행 TAC 컨트롤이 있는 TAC 그림의 line_seg 높이를 `current_height`에 누적하고, 초과 시 다음 페이지로 분할 |

## 1. `layout.rs::layout_shape_item` (line 2513~)

```rust
// Task #402: 같은 paragraph 안에 TAC 컨트롤(표/그림/도형) 2개 이상이 서로 다른 line에
// 배치된 경우, 두 번째 이후의 그림은 paragraph 시작 y가 아니라 진행된 y_offset
// (선행 TAC 후속 위치)에 그려져야 표와 겹치지 않는다. control_index 이전에 같은
// paragraph의 TAC 컨트롤이 있고 y_offset이 기존 등록값보다 진행됐으면 갱신한다.
let has_prior_tac_in_para = paragraphs.get(para_index)
    .map(|p| p.controls.iter().take(control_index).any(|c| match c {
        Control::Table(t) => t.common.treat_as_char,
        Control::Picture(p) => p.common.treat_as_char,
        Control::Shape(s) => s.common().treat_as_char,
        _ => false,
    }))
    .unwrap_or(false);
if has_prior_tac_in_para {
    let needs_update = para_start_y.get(&para_index)
        .map(|&existing| y_offset > existing + 1.0)
        .unwrap_or(true);
    if needs_update {
        para_start_y.insert(para_index, y_offset);
    }
} else {
    para_start_y.entry(para_index).or_insert(y_offset);  // 기존 동작
}
```

**효과:** pi=57 ci=1 그림의 `pic_y`가 578.09 → 919.40으로 변경. 표 위로 그림이 그려지던 겹침 해소.

## 2. `typeset.rs::typeset_table_paragraph` (line 1123~)

기본 페이지네이션 엔진은 `paginate_with_measured`(engine.rs)가 아닌 **`TypesetEngine::typeset_section`(typeset.rs)** — `RHWP_USE_PAGINATOR=1` 환경변수가 있을 때만 fallback. 따라서 실제 효과를 내려면 typeset.rs를 수정해야 한다.

기존:
```rust
Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
    st.current_items.push(PageItem::Shape { para_index, control_index });
}
```

수정 후:
```rust
Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
    // Task #402: 선행 TAC 컨트롤이 있는 TAC 그림은 자기 line_seg 높이를 누적/분할
    let tac_separate_line_h: Option<f64> = match ctrl {
        Control::Picture(p) if p.common.treat_as_char => Some(()),
        Control::Shape(s) if s.common().treat_as_char => Some(()),
        _ => None,
    }.and_then(|_| {
        let prior_tac_count = para.controls.iter().take(ctrl_idx).filter(|c| match c {
            Control::Table(t) => t.common.treat_as_char,
            Control::Picture(p) => p.common.treat_as_char,
            Control::Shape(s) => s.common().treat_as_char,
            _ => false,
        }).count();
        if prior_tac_count == 0 { return None; }
        para.line_segs.get(prior_tac_count).map(|seg| {
            let lh = hwpunit_to_px(seg.line_height, self.dpi);
            let ls_extra = if seg.line_spacing > 0 {
                hwpunit_to_px(seg.line_spacing, self.dpi)
            } else { 0.0 };
            lh + ls_extra
        })
    });
    if let Some(line_h) = tac_separate_line_h {
        if !st.current_items.is_empty()
            && st.current_height + line_h > st.available_height() + 0.5
        {
            st.advance_column_or_new_page();   // 자기 line이 안 들어가면 다음 페이지
        }
    }
    st.current_items.push(PageItem::Shape { para_index, control_index });
    if let Some(line_h) = tac_separate_line_h {
        st.current_height += line_h;
    }
}
```

**효과:** pi=57 ci=1 그림의 line 높이(369.4px)가 페이지 누적에 반영됨. 7쪽에 들어가지 않으므로 8쪽 시작으로 분할. 결과적으로 pi=58 이후 모든 후속 항목도 8쪽 이후로 이동.

## 진단 디버깅 — 잘못된 엔진 발견

초기에 `paginate_with_measured/engine.rs::process_controls`를 수정했으나 효과가 없었다. `eprintln!` 로그가 전혀 발생하지 않아 추적한 결과:

- `src/document_core/queries/rendering.rs:840`: 환경변수 분기
  ```rust
  let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
  let mut result = if use_paginator {
      paginator.paginate_with_measured_opts(...)   // engine.rs (현재 fallback)
  } else {
      typesetter.typeset_section(...)              // typeset.rs (기본)
  };
  ```

기본 경로는 `typeset.rs`. engine.rs 변경은 모두 원복했고, typeset.rs에만 수정을 남겼다.

## 검증 (Stage 2 시점)

```
$ cargo build         # OK
$ cargo test          # 1023 passed; 0 failed; 1 ignored
```

샘플 7~8쪽 시각 비교 (qlmanage thumbnail):
- 7쪽: 막대차트 + 빈 12행 표 (PDF와 일치, 파이 차트 겹침 없음) ✅
- 8쪽: 파이 차트 정상 표시 + 빈 box 영역 ✅

페이지 수 변화: 27 → 30 페이지 (분할로 인한 정상적 증가).

## 비고

- `cargo clippy`는 `commands/table_ops.rs`, `commands/object_ops.rs`의 `panicking_unwrap` 2건 에러를 보고하지만, 둘 다 base 브랜치에 이미 존재하는 기존 문제로 이번 수정과 무관함을 `git stash` 후 재실행으로 확인.
- engine.rs의 `process_controls`도 동일한 누락이 있으므로 추후 paginator 사용 시 동일 수정이 필요하지만, 현재 default 경로가 typeset이므로 본 타스크 범위에서는 typeset만 수정한다.

## Stage 3 예정

- 광범위 회귀 검증 (전 샘플 SVG 재생성 후 스팟 체크)
- 최종 보고서 작성 (`mydocs/report/task_402_report.md`)
