# Task #318 1단계 완료 보고서: A 수정 (table_partial 가드)

상위: 구현 계획서 `task_m100_318_impl.md`

## 변경

### `src/renderer/layout/table_partial.rs:766` `Control::Equation` 분기

`#301` 의 `table_layout.rs:1610` 와 동일 패턴으로 `already_rendered_inline` 가드 추가:

```rust
Control::Equation(eq) => {
    let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
    let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);

    // 빈 runs 셀 + TAC 수식: paragraph_layout(Task #287 경로)이
    // layout_composed_paragraph 안에서 이미 렌더 후
    // set_inline_shape_position 호출. 중복 emit 방지
    // (Issue #301 의 분할 표 경로 보강 — Task #318).
    let already_rendered_inline = tree
        .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
        .is_some();
    if already_rendered_inline {
        inline_x += eq_w;
        continue;
    }
    // ... (이하 기존 렌더 로직)
}
```

## 검증

### issue_301 부분 통과 (3 of 4 정상화)

| 값 | 이전 | 현재 | 기대 |
|----|------|------|------|
| 0.1915 | 2 | **1** | 1 ✓ |
| 0.3413 | 2 | **1** | 1 ✓ |
| 0.4332 | 2 | **1** | 1 ✓ |
| 0.4772 | 3 | 3 | 2 (B 잔존) |

`z_table_equations_rendered_once` 는 여전히 FAIL (0.4772 origin B 미해결). A 수정만으로 z-table 셀 중복 3건 해소 확인.

### 4샘플 페이지 수 무회귀

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15 | 15 ✓ |
| exam_math | 20 | 20 ✓ |
| exam_kor | 24 | 24 ✓ |
| exam_eng | 9 | 9 ✓ |

### 골든 SVG 무회귀

```
cargo test --test svg_snapshot
test result: ok. 6 passed; 0 failed
```

## 산출

- `src/renderer/layout/table_partial.rs` (수정)
- 본 보고서

## 다음 단계

2단계: B 진단. pi=27 (PartialParagraph + wrap=Square Table) 호스트 텍스트가 SVG 에 두 번 출현하는 origin 식별 + 인라인 수식 ci 매핑 붕괴 (두 위치 모두 0.4772 출력) 원인 파악.
