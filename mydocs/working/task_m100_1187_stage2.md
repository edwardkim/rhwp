# Stage 2 보고서 — Task M100-1187: 글상자 콘텐츠 TextBox 노드 분리

## 목표

`layout_textbox_content` 에서 글상자 내부 콘텐츠를 shape node 직속이 아니라 `RenderNodeType::TextBox` 컨테이너 아래로 모아, 다음 단계에서 SVG/paint layer clip 을 적용할 수 있는 렌더 트리 구조를 만든다.

## 변경

### src/renderer/layout/shape_layout.rs

- `inner_area` 계산 직후 `RenderNodeType::TextBox` 노드를 생성한다.
- 일반 가로쓰기 문단의 `layout_composed_paragraph` parent 를 `shape_node` 에서 `textbox_node` 로 변경했다.
- overflow 수신 글상자 분기에서도 `textbox_node` 를 parent 로 사용하고, 콘텐츠가 있으면 shape node 아래에 push 한다.
- 세로쓰기 분기 `layout_vertical_textbox_text_with_paras` 도 `textbox_node` 를 parent 로 받도록 호출 경로를 변경했다.
- 글상자 내부 인라인/절대 picture, nested shape, equation fallback, embedded table 도 모두 `textbox_node` 아래에 배치되도록 바꿨다.
- 도형 자체의 배경/테두리/이미지 채우기는 기존 shape node 에 유지했다.

## 확인

### 컴파일/테스트

```bash
cargo test --test issue_1187_textbox_clip
```

- 컴파일 성공
- 테스트는 예상대로 아직 실패
- 실패 지점: `BookReview.hwp 글상자 콘텐츠에는 textbox clipPath 가 필요함`
- 의미: Stage 2 는 렌더 트리 구조만 만들었고, SVG clipPath 출력은 Stage 3 범위다.

```bash
cargo build --bin rhwp
cargo test --test issue_1052_footnote_in_textbox
cargo test --test issue_919_textbox_hit_test
cargo test --test issue_1028_hwpx_textbox_vertical
cargo fmt --check
```

- `cargo build --bin rhwp`: 통과
- `issue_1052_footnote_in_textbox`: 4 passed
- `issue_919_textbox_hit_test`: 5 passed
- `issue_1028_hwpx_textbox_vertical`: 2 passed
- `cargo fmt --check`: 통과

### SVG 구조 확인

명령:

```bash
target/debug/rhwp export-svg samples/basic/BookReview.hwp -p 0 --show-control-codes -o /private/tmp/rhwp-task1187-stage2-svg
```

확인 결과:

- `[글상자]` 마커 3개가 출력되어 `TextBox` 노드가 SVG traversal 에 들어온 것을 확인했다.
- 아직 `textbox-clip-` 은 없음. Stage 3 에서 `RenderNodeType::TextBox` SVG clipPath 를 추가해야 한다.

## 다음 (Stage 3)

`src/renderer/svg.rs` 에서 `RenderNodeType::TextBox` 진입/종료 시점에 clipPath 그룹을 생성하고 닫아, `BookReview.hwp` 큰 글상자 하단 밖 텍스트가 실제 SVG에서 보이지 않도록 한다.
