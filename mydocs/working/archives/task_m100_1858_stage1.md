# 단계별 완료보고서 — Task #1858 Stage 1

## 범위

발현 1(3143097 Paper flow 폭발) 수정 + 회귀 게이트 + 검증. 발현 2(36389312 세로 오프셋)는
후속 분리.

## 변경

### 소스 (`src/renderer/typeset.rs`, `is_paper_topbottom_block` 가드 ~11827)

같은 host 문단에 선행 Paper 자리차지 표가 있으면(co-anchored 2번째 이후) 후속 상자도
절대배치(0 flow)하도록 가드 확장:

```rust
let can_sync = target_y > st.current_height && target_y + pre_lines_h <= available;
let has_preceding_paper_float = para.controls.iter().take(ctrl_idx).any(|c| {
    matches!(c, Control::Table(t)
        if !t.common.treat_as_char
            && matches!(t.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(t.common.vert_rel_to, VertRelTo::Paper))
});
if can_sync || has_preceding_paper_float {
    if can_sync { st.current_height = target_y; }
    self.place_table_with_text(..., 0.0, is_first_placed, is_last_placed);
    return;
}
```

host 텍스트는 `place_table_with_text` 내부에서 `is_first_placed` 로 게이트되어 후속 상자에서
중복되지 않는다. 단독 Paper 상자(선행 없음·sync 불가)는 기존대로 flow.

### 게이트 (`samples/issue1858_paper_anchor_float_stack.hwpx` + `tests/issue_1858.rs`)

실문서 3143097(pi=2 에 vert=용지 상자 22개) fixture + `page_count == 1` 단언.
게이트 유효성: 구(PR#1844) 바이너리 4쪽 확인.

## 검증 결과

| 검증 | 결과 |
|---|---|
| 3143097 페이지수 | 4→**1** (한컴 정답지 일치), 22개 상자 전부 1쪽 |
| 3143097 시각 렌더 | 정상 양식(우체국 요금후납·독촉고지서·부과내역·납부고지서·바코드·도장) |
| **blast-radius 5,198건** | #1858 추가 변화 3143097(3→1)뿐, 나머지 전부 불변 (회귀 제로) |
| 성숙 앵커/footer 테스트 | issue_1611/1624/1658/1418 통과 |
| float 테스트 | issue_1510/1663/1853 통과 |
| issue_1858 신규 게이트 | 통과 (구 바이너리 4쪽으로 유효성 확인) |
| lib 단위 | 2073 passed, 0 failed |
| rustfmt(변경 파일) | 내용 diff 없음 |

## 판단

가드를 **넓히되 co-anchored 조건으로 한정**하여 3143097 폭발이 정상 1쪽으로 복귀하고,
단독 Paper 상자·성숙 Page+Bottom 경로(#1611/#1658)는 불변. blast-radius 로 collateral 제로
실증. Stage 1 완료. 발현 2는 별도 후속.
