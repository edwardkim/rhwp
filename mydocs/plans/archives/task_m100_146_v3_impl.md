# 구현계획서 v3: TAC 표 선행 공백 x 좌표 반영

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146_v3.md`

## 0. 진입점

- **수정 대상**: `src/renderer/layout.rs:1948-1959` `tbl_inline_x` 계산 분기
- **참조 데이터**: `composed[para_index].lines[0].runs` — 첫 줄 runs 목록 (TAC 앞 텍스트 포함)
- **측정 함수**: `estimate_text_width(&str, &TextStyle)` — 기존 pub(crate) 재사용

## 1. 단계4 구현

### 1.1 선행 텍스트 폭 계산 헬퍼 (layout.rs 내부 private)

```rust
/// TAC 표 앞의 선행 텍스트(주로 공백) 폭을 계산.
/// composed.lines[0] 의 runs 중 TAC 이전 부분만 합산.
fn compute_tac_leading_width(
    composed: &ComposedParagraph,
    styles: &ResolvedStyleSet,
    control_index: usize,
    default_tab_width: f64,
) -> f64 {
    let Some(first_line) = composed.lines.first() else { return 0.0; };
    let mut width = 0.0;
    for run in &first_line.runs {
        // TAC 자체 또는 이후 run 은 스킵 (run.tac_control_index == Some(control_index))
        // run.text 전체를 텍스트 측정 → 누적
        let text_style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
        let mut style = text_style;
        style.default_tab_width = default_tab_width;
        width += estimate_text_width(&run.text, &style);
        if matches!(run.tac_control_index, Some(ci) if ci == control_index) {
            break; // TAC 위치 도달, 이전까지만 합산
        }
    }
    width
}
```

> 실제 run 구조체의 필드명(`tac_control_index`, `lang_index`)은 소스 확인 후 조정. TAC 식별자가 별도 필드이면 그것 사용, 없으면 `run.text` 내 `\u{FFFC}` (object replacement char) 위치로 구분.

### 1.2 tbl_inline_x 분기 수정

기존 (layout.rs:1953-1959):
```rust
let tbl_inline_x = if let Some((ix, _)) = inline_pos {
    Some(ix)
} else if !is_tac && tbl_is_square {
    Some(col_area.x)
} else {
    None
};
```

수정 후:
```rust
let tbl_inline_x = if let Some((ix, _)) = inline_pos {
    Some(ix)
} else if is_tac {
    // TAC 문단에 PageItem::FullParagraph 가 발행되지 않아 paragraph_layout 가
    // 호출되지 않는 케이스: 선행 텍스트(공백 포함) 폭을 직접 계산해 표 x 좌표에 반영.
    let composed_para = composed.get(para_index);
    let leading = composed_para
        .map(|c| compute_tac_leading_width(c, styles, control_index, /*tab_w*/ 0.0))
        .unwrap_or(0.0);
    Some(col_area.x + effective_margin + leading)
} else if tbl_is_square {
    Some(col_area.x)
} else {
    None
};
```

### 1.3 단위 테스트 (신규 1~2건)

위치: `src/renderer/layout/integration_tests.rs` 또는 `tests.rs`

- `test_tac_table_leading_spaces_inline_x`:
  - Minimal Paragraph: text="    " (4 spaces), controls=[Table{treat_as_char=true}]
  - ParaShape: margin_left=0, indent=-something, CharShape: spacing=-8%, font_size=20
  - 기대: 표의 inline x ≈ body_left + (space_w × 4) (space_w = 20×0.5×(1-0.08) = 9.2 → total 36.8)
  - 검증: build_render_tree 후 table node.bbox.x 또는 inline_shape_position 확인

### 1.4 재현 검증

```bash
cargo run --bin rhwp -- export-svg samples/text-align.hwp -o output/svg/text-align/
grep 'cell-clip-13' output/svg/text-align/text-align.svg | head -1
# expected: rect x 가 약 112 (기존 75.59 → 수정 후 112 근처)
```

### 1.5 단계4 커밋

- 커밋 1: 소스 (`src/renderer/layout.rs`) + 단위 테스트
- 메시지: `Task #146: TAC 표 선행 텍스트 폭을 inline x 좌표에 반영`

## 2. 단계5 검증·보고

### 2.1 테스트 스위프

- `cargo test --lib`: 931 passed (v2 의 929 + 신규 2~3) 이상 기대
- `cargo test --test svg_snapshot`: 실패 발생 시 각 샘플에 대해 PDF 기준으로 가까워짐 증빙 → `UPDATE_GOLDEN=1` 재생성
- `cargo clippy --lib -- -D warnings` 통과

### 2.2 스모크 스위프

TAC 표 + 선행 공백 가능성이 높은 샘플:
- `samples/biz_plan.hwp` (재확인)
- `samples/basic/` 하위 표가 많은 문서
- `samples/exam_*.hwp`

### 2.3 결과보고서 v3

`mydocs/report/task_m100_146_report_v3.md` 신규 작성 (v2 보고서는 Geometric Shapes 에 대한 기록으로 보존):
- v2 이후 추가된 증상, 원인, 수정 요약
- 좌표 수렴 표 (before/after/PDF 환산)
- svg_snapshot 영향 샘플 목록
- 전체 타스크 종결 요약

### 2.4 orders 갱신

`mydocs/orders/20260423.md` 하단에 v3 체크리스트 추가.

### 2.5 단계5 커밋

- 커밋 1 (영향 시): svg_snapshot golden
- 커밋 2: v3 보고서 + orders 갱신

## 3. 산출물 체크리스트

- [ ] `src/renderer/layout.rs` tbl_inline_x 분기 확장 + 헬퍼 함수
- [ ] `src/renderer/layout/integration_tests.rs` 신규 테스트
- [ ] `mydocs/working/task_m100_146_stage4.md`
- [ ] `mydocs/working/task_m100_146_stage5.md`
- [ ] `mydocs/report/task_m100_146_report_v3.md`
- [ ] svg_snapshot golden (영향 시)
- [ ] orders/20260423.md 갱신

## 4. 롤백

단계4 수정은 `is_tac && inline_pos.is_none()` 분기 한정이라 회귀 발생 시 해당 분기만 제거하면 원복. 헬퍼 함수는 미사용 상태로 남거나 함께 제거.
