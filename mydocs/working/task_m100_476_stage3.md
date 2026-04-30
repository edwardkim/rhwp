# Task M100 #476 Stage 3 완료 보고서

## 1. 핵심 발견

### 1-1. 메인 paginator는 `TypesetEngine`

수행 계획서/Stage 1·2에서 `Paginator::paginate_with_measured`를 메인으로 가정했으나, `src/document_core/queries/rendering.rs:889-915` 확인 결과:

```rust
let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
let mut result = if use_paginator {
    paginator.paginate_with_measured_opts(...)
} else {
    use crate::renderer::typeset::TypesetEngine;
    let typesetter = TypesetEngine::new(self.dpi);
    typesetter.typeset_section(...)
};
```

**기본 메인 페이지네이션은 `TypesetEngine`** 이고 `Paginator`는 `RHWP_USE_PAGINATOR=1` 시 fallback. 양쪽 모두 같은 PageItem::Shape 등록 패턴(무조건 `current_items.push`) 가짐.

### 1-2. 양쪽 모두 수정 필요

A 본질 수정은 `Paginator`와 `TypesetEngine` 양쪽에 동일하게 적용해야 한다.

## 2. 변경 내용

### 2-1. 공용 helper 추출

`src/renderer/pagination.rs`에 `pub fn find_inline_control_target_page(...)` 추가:

```rust
pub fn find_inline_control_target_page(
    pages: &[PageContent],
    current_items: &[PageItem],
    para_idx: usize,
    ctrl_idx: usize,
    para: &Paragraph,
) -> Option<(usize, usize)> {
    // 박스 char 위치 → line index
    let positions = para.control_text_positions();
    let ctrl_text_pos = *positions.get(ctrl_idx)?;
    let target_line = para.line_segs.iter().enumerate()
        .rev()
        .find(|(_, ls)| (ls.text_start as usize) <= ctrl_text_pos)
        .map(|(i, _)| i)
        .unwrap_or(0);

    // 1) 현재 페이지 검사 → in_current 면 None (= push to current)
    // 2) 이전 페이지/단 검색 → 발견 시 (page_idx, col_idx) 반환
    ...
}
```

### 2-2. `Paginator::process_controls`의 Shape 분기 수정

`engine.rs:987-1014`:
- treat_as_char Shape 시 `find_inline_control_target_page` 호출
- 반환값 `Some((page_idx, col_idx))` → `st.pages[page_idx].column_contents[col_idx].items.push(item)`
- 반환값 `None` → 기존대로 `st.current_items.push(item)`

### 2-3. `TypesetEngine`의 동일 로직 적용

`typeset.rs:638` `Control::Shape | Control::Picture | Control::Equation` 분기:
- 동일한 라우팅 패턴 적용
- treat_as_char Shape만 라우팅 대상 (Picture/Equation은 기존 동작 유지)

## 3. 검증 결과

### 3-1. 핵심 시각 검증

`samples/21_언어_기출_편집가능본.hwp`:

| 항목 | 페이지 11 | 페이지 12 |
|------|-----------|-----------|
| paragraph 228 박스 | ✓ x=156.2, y=790.9 (단 0) | — |
| paragraph 238 박스 | ✓ **x=616.1, y=1313.3 (단 1 끝)** ← 신규 정상 출현 | — (잘못된 fallback 출현 없음) |
| paragraph 251 박스(23번) | — | ✓ x=145.2, y=1166.0 |

**페이지 11 단 1 끝 본문 "[배너지와 뒤플로]" 박스가 PDF와 일치하는 위치에 정상 출현**. 페이지 12의 잘못된 박스(y=742.45)는 제거됨.

### 3-2. 단위/통합 테스트

```
cargo test --release
test result: ok. 1078 passed; 0 failed; 1 ignored; (lib)
test result: ok. 14, 25, 1, 6, ... (statements integration tests 모두 통과)
```

`test_task78_rectangle_textbox_inline_images` 포함 모든 회귀 통과.

### 3-3. D 차단 발동 확인

```
$ RHWP_DEBUG_LAYOUT=1 ./target/release/rhwp export-svg ... -p 11
(no '#476 skip' messages)
```

A 수정 후 페이지 12에서 D 차단 발동 없음 — paginator가 박스를 올바른 페이지(11)에 라우팅했기 때문. D 차단은 **회귀 안전장치**로 그대로 유지(`shape_layout.rs:218`).

## 4. 잔여 영향

### 4-1. 23번 paragraph 위치 어긋남(별개 이슈)

페이지 12 단 0의 23번 paragraph가 단 끝(y≈1166)에 그려지고 답안 일부가 잘림 — 단 0 사용량 `used=1012px vs hwp_used=1219px (diff=-206.8px)`.

본 task의 박스 라우팅 수정과 별개의 layout 버그(paragraph 238의 페이지 12 영역 lines 4..26 높이 계산 또는 단 분배 문제로 추정). 별도 이슈로 분리.

### 4-2. 디버그 로그 정리

paragraph_layout.rs의 임시 디버그 로그(`#476 reg`) 제거. shape_layout.rs의 `#476 skip` 로그는 D 차단 발동 시에만 stderr 출력하므로 안전장치로 유지.

## 5. 다음 단계

Stage 4 — 광범위 회귀 검증 + 최종 보고서 작성 + 커밋.
