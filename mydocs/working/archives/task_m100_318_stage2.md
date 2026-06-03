# Task #318 2단계 완료 보고서: B 정밀 진단

상위: 구현 계획서 `task_m100_318_impl.md`
선행: `task_m100_318_stage1.md` (A 수정 완료, 0.4772 잔존)

## Origin 확정

`src/renderer/layout.rs` 의 두 경로가 **동일 호스트 문단을 두 번 렌더**.

### 경로 1: `PartialParagraph` PageItem (line 1799)

```rust
PageItem::PartialParagraph { para_index, start_line, end_line } => {
    if let Some(para) = paragraphs.get(*para_index) {
        // ... (wrap=Square host 검사 없음)
        y_offset = self.layout_partial_paragraph(
            tree, col_node, para, comp.as_ref(), styles, col_area,
            y_offset, *start_line, *end_line, page_content.section_index,
            *para_index, None, Some(bin_data_content),
        );
    }
}
```

→ `layout_partial_paragraph` 호출 → `layout_composed_paragraph` → 텍스트 + 인라인 수식 emit + `set_inline_shape_position`.

### 경로 2: `layout_wrap_around_paras` (line 2103, 2522)

표 PageItem 처리 중 wrap=Square 표면 호출됨:

```rust
let table_is_square = matches!(t.common.text_wrap, ::TextWrap::Square);
if !is_tac && table_is_square {
    self.layout_wrap_around_paras(
        tree, col_node, paragraphs, composed, styles, col_area,
        page_content.section_index, para_index, wrap_around_paras,
        table_y_before, y_offset,
        wrap_text_x, wrap_text_width, 0.0,
        bin_data_content,
    );
}
```

이 안에서 (line 2509-2526):

```rust
let has_host_text = table_para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
if table_content_offset == 0.0 {
    if has_host_text {
        // ... (Task #295: 자가 wrap host 다중 줄 렌더링)
        self.layout_partial_paragraph(
            tree, col_node, table_para, Some(comp), styles,
            &wrap_area, table_y_start, start_line, text_end_line,
            section_index, table_para_index, None, Some(bin_data_content),
        );
    }
}
```

→ 같은 `layout_partial_paragraph` 호출 → 같은 텍스트 + 인라인 수식 두 번째 emit. `set_inline_shape_position` 가 두 번째 좌표로 덮어써짐.

## FullParagraph 경로와의 비대칭

`FullParagraph` PageItem 처리 (`layout.rs:1638-1672`) 는 명시적 `is_wrap_host` 가드:

```rust
let is_wrap_host = para.controls.iter().any(|c| {
    if let Control::Table(t) = c {
        !t.common.treat_as_char && matches!(t.common.text_wrap, ::TextWrap::Square)
    } else { false }
});
let has_real_text = !is_wrap_host && para.text.chars().any(...);
if has_real_text {
    // ... layout_partial_paragraph 호출
}
```

→ FullParagraph + wrap=Square host 면 텍스트 렌더링 스킵 (wrap_around 가 처리). 정상 동작.

`PartialParagraph` 경로 (line 1765-1815) 에는 이 가드가 없음 → 회귀.

## 왜 pi=27 이 PartialParagraph 로 emit 되나

`exam_math.hwp` pi=27 은 line_segs 5건 (전체 줄). TypesetEngine 이 `PartialParagraph { lines: 0..5 }` 로 emit. 이유는 wrap=Square 표 + 본문 줄을 함께 가진 경우 TypesetEngine 이 PartialParagraph 를 선호하기 때문 (호스트 본문 줄과 표를 분리 처리).

본 task 범위 외이지만 PartialParagraph 출현 자체는 정상 동작이므로 layout 측 가드 보강이 적절.

## ci 매핑은 정상

당초 가설 (b — wrap 경로의 ci 매핑 붕괴) 은 부정. 두 경로 모두 같은 paragraph 의 같은 인라인 수식들을 정상적으로 emit. 단지 **각 emit 의 위치가 다른 y 좌표** 일 뿐.

z-table 셀의 0.4772 (1) + 호스트 문단의 0.4772 line (1) → 정상 = 2회.
호스트 문단이 두 번 emit 되어 0.4772 line 도 2번 → 3회 (현재).

따라서 SVG 의 y=441 과 y=591 두 위치 모두 같은 줄 ("이용하여 구한 것이 0.4772 일 때") 를 그린 결과. 한 위치만 나오면 정상.

## 수정 방안 (3단계)

`PartialParagraph` PageItem 처리에 `FullParagraph` 와 동일한 `is_wrap_host` 가드 추가:

```rust
PageItem::PartialParagraph { para_index, start_line, end_line } => {
    if let Some(para) = paragraphs.get(*para_index) {
        // Task #318: wrap=Square 표 호스트 문단의 텍스트는 layout_wrap_around_paras 가
        // 처리하므로 PartialParagraph 측 호출 스킵 (FullParagraph 동일 가드).
        let is_wrap_host = para.controls.iter().any(|c| {
            if let Control::Table(t) = c {
                !t.common.treat_as_char
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
            } else { false }
        });
        if is_wrap_host {
            return (y_offset, false);
        }
        // ... (기존 처리)
    }
}
```

리스크:
- wrap=Square 표가 PartialParagraph 로 진입하는 케이스가 자가 wrap (host=table_para_index) 외에 별도로 있다면 본 가드가 텍스트 누락을 만들 수 있음 → 4샘플 + 골든 SVG 회귀 검증으로 확인

## 산출

- 본 보고서

## 다음 단계

3단계: PartialParagraph PageItem 처리에 `is_wrap_host` 가드 추가 + 4샘플 무회귀 + issue_301 통과 확인.
