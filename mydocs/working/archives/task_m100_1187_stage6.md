# Stage 6 보고서 — Task M100-1187: 글상자 vpos 중복 보정

## 배경

PR #1190 1차 수정은 글상자 콘텐츠 clip 은 적용했지만, `BookReview.hwp` 1쪽의 큰 글상자 목차가 한컴 기준보다 아래로 밀렸다. 그 결과 정상 표시되어야 하는 `5장`, `6장`, `에필로그` 줄이 clip 밖으로 나가 보이지 않았다.

## 원인

`layout_textbox_content` 는 글상자 문단의 `line_seg.vertical_pos` 를 사용해 문단 y 좌표를 이미 계산한다. 그런데 `layout_composed_paragraph` 의 본문 column-top fallback 이 `cell_ctx.is_some()` 인 글상자 첫 문단에도 적용되어 첫 문단 vpos 가 한 번 더 더해졌다.

이 중복 오프셋이 뒤따르는 문단의 `para_y.max(vpos_y)` 흐름에도 전파되어 큰 글상자 목차 전체가 약 133px 아래로 밀렸다.

## 수정

- `src/renderer/layout/paragraph_layout.rs`
  - column-top `spacing_before`/`line_seg.vertical_pos` fallback 을 `cell_ctx.is_none()` 인 본문 흐름에만 적용하도록 제한했다.
  - 글상자/표 셀처럼 호출자가 이미 explicit y 를 계산한 경로는 vpos 를 중복 적용하지 않는다.
- `tests/issue_1187_textbox_clip.rs`
  - `5장`, `6장`, `에필로그` 목차 줄이 렌더 트리에 남아 있고 큰 글상자 clip 하단 안에 배치되는지 검증한다.
  - SVG 공백 글리프 생략을 고려해 공백 제거 후 문자열을 비교한다.

## 산출물 확인

```bash
target/debug/rhwp export-svg samples/basic/BookReview.hwp -p 0 -o /private/tmp/rhwp-task1187-stage6-svg
```

큰 글상자 clip:

- `x=47.91999999999997`
- `y=516.5600000000001`
- `width=687.5466666666667`
- `height=487.88`
- clip bottom: `1004.44`

보정 후 주요 목차 baseline:

- `5장`: `y=887.34`
- `6장`: `y=924.03`
- `에필로그`: `y=960.73`

모두 clip bottom 안에 들어온다.

## 검증

통과:

```bash
cargo fmt --all -- --check
cargo build --bin rhwp
cargo test --test issue_1187_textbox_clip
cargo test --test issue_1052_footnote_in_textbox --test issue_919_textbox_hit_test --test issue_1028_hwpx_textbox_vertical
cargo test --lib paint::builder::tests
cargo test --lib paint::json::tests::serializes_textbox_clip_kind
cargo test --lib paint::schema::tests::layer_tree_schema_constants_match_schema
cargo test --lib renderer::svg_layer::tests
wasm-pack build --target web --dev
npm run build
```

rhwp-studio 시각 검증:

- URL: `http://127.0.0.1:5175/?url=/samples/basic/BookReview.hwp&filename=BookReview.hwp`
- Browser: in-app Browser
- 콘솔 error/warn: 없음
- Vite overlay: 없음
- 스크롤 후 1쪽 큰 글상자 하단에서 `5장`, `6장`, `에필로그`, 저자 정보 글상자가 모두 보임.

스크린샷:

- `/private/tmp/rhwp-task1187-stage6-after.png`

## 결론

Stage 6 보정 후 #1187 수정은 한컴 기준처럼 정상 목차 줄을 유지하면서도 글상자 콘텐츠를 clip 한다.
