# Task #455 Stage 1 — 진단 결과

## 결론

원인은 **layout 버그**. 위치는 `src/renderer/layout/paragraph_layout.rs` 의 `skip_text_for_inline_shape` 플래그.

## 진단 과정

`paragraph_layout::layout_composed_paragraph` 진입 시점에 임시 stderr 디버그를 삽입하여 pi=33 의 composed.lines 를 확인했다.

```
[T455] layout_composed_paragraph s=0 pi=33 start=0 end=6 y_start=282.0 composed.lines.len=6
[T455]   line[0] runs=1 sw=30044 lh=1150 char_start=0 text='서양의 과학과 기술, 천주교의 수용을 반대했던 이항로를 비롯한 '
[T455]   line[1] runs=1 sw=30044 lh=1417 char_start=35 text='척사파의 주장은 개항 이후에도 지속되었지만, 는 거스를 '
[T455]   line[2] runs=3 sw=30044 lh=1150 char_start=66 text='수 없는 대세로 자리 잡았다. 개물성무(開物成務)와 화민성속'
...
[T455]   tac_controls=[(60, 2551, 0)]
```

핵심:
- `line[1]` (line 2) 의 runs 가 **정상 채워져 있다**: text "척사파의 주장은 개항 이후에도 지속되었지만, 는 거스를 ".
- composer 는 정상. 따라서 버그는 **render 단계**.
- `tac_controls=[(60, 2551, 0)]` — 글상자는 char 위치 60 에 1개, 폭 2551 HU.

## 버그 위치

`src/renderer/layout/paragraph_layout.rs` 의 tac 분기점 처리(현재 줄에 tac 컨트롤이 있는 경우):

```rust
// 인라인 Shape 중 글상자(TextBox)가 있는 경우에만 텍스트 스킵
// (글상자 텍스트는 table_layout에서 렌더링)
// 단순 도형(사각형, 원 등)은 TextBox가 없으므로 텍스트를 여기서 렌더링
let skip_text_for_inline_shape = has_tac_shape && para.map(|p| {
    tac_offsets_px.iter().any(|(_, _, ci)| {
        if let Some(Control::Shape(s)) = p.controls.get(*ci) {
            s.drawing().map(|d| d.text_box.is_some()).unwrap_or(false)
        } else { false }
    })
}).unwrap_or(false);

// ...
if !skip_text_for_inline_shape {
    line_node.children.push(sub_run_node);  // tac 앞 텍스트
}
x += seg_w;
// ...
if !skip_text_for_inline_shape {
    line_node.children.push(sub_run_node);  // 마지막 tac 이후 텍스트
}
```

## 잘못된 의도

주석에 따르면 "글상자 텍스트는 별도 패스에서 렌더링되므로 여기서 텍스트를 스킵" 이라는 의도였으나, 여기서 "텍스트" 는 글상자의 **외부 문단 본문 텍스트**(line 2 의 "척사파의 ... 거스를 ")이지 글상자 **내부 텍스트**("개화") 가 아니다. 두 텍스트는 서로 무관하며, 외부 본문 텍스트는 항상 렌더되어야 한다.

`shape_layout.rs:218` 에서는 `tree.get_inline_shape_position` 으로 paragraph_layout 이 등록한 위치(`set_inline_shape_position` at paragraph_layout.rs:1786)를 가져와 글상자 외곽 + 내부 "개화" 를 렌더한다. 즉 글상자 자체는 별도 패스에서 정상 렌더됨. 외부 문단 텍스트도 렌더되어야 하지만 `skip_text_for_inline_shape=true` 가 되어 통째로 누락되었다.

## 수정 방향

`skip_text_for_inline_shape` 분기 제거. 외부 본문 텍스트는 항상 렌더한다. 글상자 본체와 내부 텍스트는 shape_layout 에서 별도 렌더되므로 중복되지 않는다.

다음 단계(Stage 2)에서 수정 적용 후 회귀 검증.
