# Stage 5 보고서 — Task M100-1187: 최종 통합 검증

## 목표

`BookReview.hwp` 글상자 내용 overflow 회귀가 SVG 경로와 paint layer 경로에서 모두 해소됐는지 최종 확인하고, 최종 보고서를 정리한다.

## 최종 산출물 확인

명령:

```bash
target/debug/rhwp export-svg samples/basic/BookReview.hwp -p 0 --debug-overlay --show-control-codes --show-grid -o /private/tmp/rhwp-task1187-final-svg
rg -n "textbox-clip|body-clip|cell-clip|\\[글상자\\]" /private/tmp/rhwp-task1187-final-svg/BookReview_001.svg
```

확인:

- `textbox-clip-36`, `textbox-clip-55`, `textbox-clip-106` 이 생성됐다.
- 회귀가 발생한 큰 글상자 콘텐츠는 `<g clip-path="url(#textbox-clip-55)">` 아래에 배치된다.
- 큰 글상자 clip rect:
  - `x=47.91999999999997`
  - `y=516.5600000000001`
  - `width=687.5466666666667`
  - `height=487.88`
- 우측 하단 저자 정보 글상자는 별도 `textbox-clip-106` 아래에 남아 있다.

## 최종 검증

```bash
cargo fmt --check
cargo build --bin rhwp
cargo test --test issue_1187_textbox_clip
cargo test --lib paint::builder::tests
cargo test --lib paint::json::tests::serializes_textbox_clip_kind
cargo test --lib paint::schema::tests::layer_tree_schema_constants_match_schema
cargo test --lib renderer::svg_layer::tests
cargo test --test issue_1052_footnote_in_textbox
cargo test --test issue_919_textbox_hit_test
cargo test --test issue_1028_hwpx_textbox_vertical
```

결과:

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- `issue_1187_textbox_clip`: 2 passed
- `paint::builder::tests`: 7 passed
- `paint::json::tests::serializes_textbox_clip_kind`: 1 passed
- `paint::schema::tests::layer_tree_schema_constants_match_schema`: 1 passed
- `renderer::svg_layer::tests`: 3 passed
- `issue_1052_footnote_in_textbox`: 4 passed
- `issue_919_textbox_hit_test`: 5 passed
- `issue_1028_hwpx_textbox_vertical`: 2 passed

추가 확인:

```bash
npm run build
```

- 실패: `tsc: command not found`
- 원인: `/private/tmp/rhwp-task1187/rhwp-studio/node_modules` 및 전역 `tsc` 가 없는 로컬 환경 상태다.
- 의존성 설치는 네트워크가 필요하므로 수행하지 않았다.

## 결론

`BookReview.hwp` 1쪽의 큰 글상자 overflow 콘텐츠는 SVG와 paint layer 양쪽에서 글상자 내부 영역으로 clip 된다. 원문 콘텐츠는 삭제하지 않고 렌더 출력 단계에서만 숨기며, 별도 저자 정보 글상자는 유지된다.
