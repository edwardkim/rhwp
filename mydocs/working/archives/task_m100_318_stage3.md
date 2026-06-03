# Task #318 3단계 완료 보고서: B 수정

상위: 구현 계획서 `task_m100_318_impl.md`
선행: `task_m100_318_stage2.md` (origin = PartialParagraph 경로의 is_wrap_host 가드 누락)

## 변경

### `src/renderer/layout.rs::layout_column_item` PartialParagraph 분기

`FullParagraph` 경로 (`layout.rs:1639`) 와 동일한 `is_wrap_host` 가드 추가:

```rust
PageItem::PartialParagraph { para_index, start_line, end_line } => {
    if let Some(para) = paragraphs.get(*para_index) {
        // Task #318: wrap=Square 표 호스트 문단의 텍스트는
        // layout_wrap_around_paras (자가 wrap 경로) 가 처리한다. PartialParagraph
        // 측에서 같은 paragraph 를 layout_partial_paragraph 로 다시 호출하면
        // 호스트 텍스트 + 인라인 수식이 중복 emit 됨 (#301 회귀).
        // FullParagraph 경로 (`is_wrap_host` 가드, layout.rs:1639) 와 동일한 처리.
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

### `tests/issue_301.rs::z_table_equations_rendered_once`

`#[ignore]` 제거 (4단계 산출이지만 검증 위해 본 단계에서 선반영 → 통과 확인).

## 검증

### issue_301 통과

```
cargo test --test issue_301
test result: ok. 1 passed; 0 failed; 0 ignored
```

| 값 | 측정 | 기대 |
|----|------|------|
| 0.1915 | 1 | 1 ✓ |
| 0.3413 | 1 | 1 ✓ |
| 0.4332 | 1 | 1 ✓ |
| 0.4772 | 2 | 2 ✓ |

### 4샘플 페이지 수 무회귀

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15 | 15 ✓ |
| exam_math | 20 | 20 ✓ |
| exam_kor | 24 | 24 ✓ |
| exam_eng | 9 | 9 ✓ |

### 전체 테스트

```
cargo test
992 lib + 25 어댑터 + 14 통합 + 6 svg_snapshot + issue_301 + ... 모두 PASS (0 failed, 0 ignored)
```

## 영향 범위

`PageItem::PartialParagraph` 처리 시 paragraph 의 첫 컨트롤 중 wrap=Square 표 (treat_as_char=false) 가 있을 때만 분기 진입. 이 경우 텍스트는 wrap-around 경로에서 처리되므로 PartialParagraph 호출이 redundant 였음 → 가드만 추가, 동작 변경 없음.

리스크 검증:
- pi=27 외 다른 wrap=Square 표가 PartialParagraph 로 진입하는 케이스 — 4샘플/골든 SVG 회귀로 검출 가능. 회귀 0건.

## 산출

- `src/renderer/layout.rs` (수정)
- `tests/issue_301.rs` (`#[ignore]` 제거)
- 본 보고서

## 다음 단계

4단계: 진단 도구 회수 + 최종 보고서.

(본 task 에서는 1단계의 `table_partial.rs` A fix 외에 임시 도구가 도입되지 않았으므로 4단계는 보고서 작성만.)
