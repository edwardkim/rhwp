# PR: Task #318 — 분할 표 + wrap=Square 호스트 문단 인라인 수식 중복 emit 회귀 수정

## 제목

```
Task #318: 분할 표 + wrap=Square 호스트 문단 인라인 수식 중복 emit 회귀 수정
```

## 본문

## 배경

#313 (TypesetEngine main 전환) 후 `tests/issue_301.rs::z_table_equations_rendered_once` 회귀. task314 브랜치 (커밋 `78fb1f1`) 에서 임시 `#[ignore]` 처리되어 있던 것을 정식 수정하고 ignore 제거.

선행:
- #313 (TypesetEngine main 전환) — 회귀 도입 시점
- #301 (z-table 수식 이중 렌더링 수정) — 본 task 가 보강하는 가드의 원조

## 두 origin

### A. 분할 표 셀 수식 중복 (`table_partial.rs`)

`#301` 의 `table_layout.rs` 가드 (`already_rendered_inline`) 가 분할 표 경로에 미적용. 빈 runs 셀 + TAC 수식이 `paragraph_layout` 과 `table_partial.rs::Control::Equation` 두 곳에서 emit 되어 중복.

영향: `0.1915`, `0.3413`, `0.4332` 가 각 2회 → 1회로 정상화.

### B. wrap=Square 호스트 문단 중복 (`layout.rs`)

`PageItem::PartialParagraph` 분기에 `is_wrap_host` 가드 누락. `FullParagraph` 분기 (`layout.rs:1639`) 는 가드가 있어 정상 동작했으나 `PartialParagraph` 는 누락. wrap=Square 표 호스트 문단의 텍스트가 `layout_partial_paragraph` (PartialParagraph PageItem) + `layout_partial_paragraph` (`layout_wrap_around_paras` 내부) 두 경로에서 동일 paragraph 를 렌더 → 호스트 텍스트 + 인라인 수식이 두 개 y 좌표에 중복 emit.

영향: `0.4772` body 위치 1회 → 2회 (z-table 1 합쳐 3 → 2).

## 변경

### 1. `src/renderer/layout/table_partial.rs:766` `Control::Equation`

```rust
let already_rendered_inline = tree
    .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
    .is_some();
if already_rendered_inline {
    inline_x += eq_w;
    continue;
}
```

### 2. `src/renderer/layout.rs::layout_column_item` `PartialParagraph` 분기

```rust
PageItem::PartialParagraph { para_index, start_line, end_line } => {
    if let Some(para) = paragraphs.get(*para_index) {
        let is_wrap_host = para.controls.iter().any(|c| {
            if let Control::Table(t) = c {
                !t.common.treat_as_char
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
            } else { false }
        });
        if is_wrap_host {
            return (y_offset, false);
        }
        // ... (이하 기존 처리)
    }
}
```

### 3. `tests/issue_301.rs::z_table_equations_rendered_once`

`#[ignore]` 제거. task314 커밋 `78fb1f1` 의 임시 처리 회수.

## 검증

### issue_301 통과

| 값 | 측정 | 기대 |
|----|------|------|
| 0.1915 | 1 | 1 ✓ |
| 0.3413 | 1 | 1 ✓ |
| 0.4332 | 1 | 1 ✓ |
| 0.4772 | 2 | 2 ✓ |

### 4샘플 무회귀

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15 | 15 ✓ |
| exam_math | 20 | 20 ✓ |
| exam_kor | 24 | 24 ✓ |
| exam_eng | 9 | 9 ✓ |

### 전체 테스트

```
cargo test
992 lib + 25 어댑터(0 ignored) + 6 svg_snapshot + issue_301 + 통합 모두 PASS
```

## 영향 범위

- A 가드: HWPX/HWP 출처 무관. 빈 runs 셀 + 인라인 TAC 수식이 있는 분할 표만 분기 진입. 다른 셀 동작 무변경.
- B 가드: wrap=Square + non-TAC 표를 가진 PartialParagraph 호스트 paragraph 만 분기 진입. 본문은 wrap_around 경로에서 처리되므로 결과 동일, 중복만 제거.

## 후속 사안 (선택)

- TypesetEngine 이 wrap=Square 호스트 paragraph 에 대해 PartialParagraph 를 emit 하는 동작이 정상인지 재평가. 현재 layout 측 가드로 회피하지만, 의도가 명확하면 PartialParagraph 자체를 emit 하지 않는 것이 더 깔끔.

## 단계별 진행

| 단계 | 내용 | 보고서 |
|------|------|--------|
| 1 | A 수정 (`table_partial.rs` 가드) — 3 of 4 정상화 | `mydocs/working/task_m100_318_stage1.md` |
| 2 | B 정밀 진단 (PartialParagraph + wrap_around 이중 호출) | `mydocs/working/task_m100_318_stage2.md` |
| 3 | B 수정 (`layout.rs` is_wrap_host 가드) + 검증 | `mydocs/working/task_m100_318_stage3.md` |
| 4 | 최종 정리 | `mydocs/working/task_m100_318_stage4.md` |

최종 보고서: `mydocs/report/task_m100_318_report.md`

## Test plan

- [x] `cargo test --test issue_301` 1 passed (ignore 제거 후 통과)
- [x] `cargo test` 전체 PASS, 0 failed
- [x] 4샘플 페이지 수 무변화
- [x] 골든 SVG 6건 무회귀

closes #318
