# Stage 3 보고서 — Task M100-1187: SVG TextBox clipPath 적용

## 목표

`RenderNodeType::TextBox` 렌더 노드가 SVG 출력에서 자신의 내부 영역으로 콘텐츠를 clip 하도록 만들어, `BookReview.hwp` 글상자 내용이 점선 영역 밖으로 보이는 회귀를 SVG 경로에서 해소한다.

## 변경

### src/renderer/svg.rs

- `RenderNodeType::TextBox` 진입 시 `textbox-clip-{node.id}` clipPath 를 생성한다.
- clip rect 는 Stage 2 에서 만든 TextBox 노드의 `bbox` 를 그대로 사용한다.
- TextBox 하위 콘텐츠 출력 전에 `<g clip-path="url(#textbox-clip-...)">` 를 열고, 하위 콘텐츠와 control-code marker 출력 뒤에 그룹을 닫는다.
- 기존 `Body`, `TableCell` clip 처리와 동일한 패턴을 사용했다.

## 확인

### 컴파일/테스트

```bash
cargo fmt --check
cargo test --test issue_1187_textbox_clip
cargo build --bin rhwp
```

- `cargo fmt --check`: 통과
- `issue_1187_textbox_clip`: 1 passed
- `cargo build --bin rhwp`: 통과

### SVG 구조 확인

명령:

```bash
target/debug/rhwp export-svg samples/basic/BookReview.hwp -p 0 --debug-overlay --show-control-codes --show-grid -o /private/tmp/rhwp-task1187-stage3-svg
rg -n "textbox-clip|body-clip|cell-clip|\\[글상자\\]" /private/tmp/rhwp-task1187-stage3-svg/BookReview_001.svg
```

확인 결과:

- SVG defs 에 `textbox-clip-36`, `textbox-clip-55`, `textbox-clip-106` 이 생성됐다.
- 회귀가 발생한 큰 글상자 clip rect 는 다음 값으로 확인됐다.
  - `x=47.91999999999997`
  - `y=516.5600000000001`
  - `width=687.5466666666667`
  - `height=487.88`
- 큰 글상자 콘텐츠는 `<g clip-path="url(#textbox-clip-55)">` 아래에 배치된다.
- 우측 하단 저자 정보 글상자도 별도 `textbox-clip-106` 아래에 배치된다.

## 다음 (Stage 4)

paint layer 경로에서도 `RenderNodeType::TextBox` 에 대응하는 `ClipRect` push/pop 을 추가해 SVG 외 렌더 경로의 동작을 맞춘다.
