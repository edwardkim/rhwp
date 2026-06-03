# Task #301 Stage 2 완료 보고서: 본 수정

- **이슈**: #301
- **브랜치**: `local/task301`
- **단계**: 2 / 4

## 변경 파일

- `src/renderer/layout/table_layout.rs:1602-1620` Equation 분기 가드 추가

## 수정 내용

`Control::Equation` 분기에 `tree.get_inline_shape_position()` 검사 추가.
paragraph_layout(Task #287의 빈-runs 경로 또는 일반 인라인 경로)이 이미 렌더 후
`set_inline_shape_position`을 호출했으면 `inline_x`만 진행하고 직접 emit을 스킵.

```rust
let already_rendered_inline = tree
    .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
    .is_some();
if has_text_in_para || already_rendered_inline {
    inline_x += eq_w;
} else {
    // 기존 fallback: line_segs 미존재 등으로 paragraph_layout이 안 그린 케이스
    ...
}
```

기존 `has_text_in_para` 검사는 보조 가드로 유지 (paragraph_layout이 부분적으로
렌더하는 코너 케이스 보호).

## 검증

### 회귀 테스트 GREEN
```
$ cargo test --test issue_301 --release
running 1 test
test z_table_equations_rendered_once ... ok
test result: ok. 1 passed; 0 failed; ...
```

### SVG 출력 확인
```
$ grep -c '0\.1915\|0\.3413\|0\.4332' output/svg/exam_math_p12_fixed/exam_math_012.svg
3   (수정 전: 6)
```

각 z-table 값이 정확히 1회만 출현.

### 시각 검증
PNG 변환 결과 z-table이 깔끔하게 한 번만 렌더링됨 (이전 SVG의 텍스트 겹침 사라짐).

## 다음 단계

Stage 3: 다른 샘플(`exam_math_8.hwp`, `exam_math_no.hwp` 등) 회귀 확인 + 전체 `cargo test`.

## 승인 요청

본 단계 결과에 대한 작업지시자 승인 요청.
