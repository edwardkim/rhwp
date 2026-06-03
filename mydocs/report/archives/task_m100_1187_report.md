# Task M100-1187 최종 보고서 — BookReview.hwp 글상자 내용 overflow 회귀 수정

## 개요

- 이슈: #1187 `BookReview.hwp 글상자 내용이 영역 밖으로 출력되는 회귀`
- 기준: `upstream/devel` `1eb76529fce21d0a5330720f7b458831c5252fdf`
- 작업 브랜치: `local/task1187`
- 작업 worktree: `/private/tmp/rhwp-task1187`

## 원인

`layout_textbox_content` 는 글상자 내부 영역(`inner_area`)을 계산하지만, 실제 글상자 문단/표/도형 자식은 shape node 직속으로 배치됐다. SVG/paint layer 의 clip 처리는 Body/TableCell 중심이라 `TextBox` 콘텐츠에는 적용되지 않았고, `BookReview.hwp` 의 일부 문단처럼 `line_seg.vertical_pos` 가 내부 높이를 초과하면 글상자 밖까지 그대로 출력됐다.

PR 1차 수정 후 추가 시각 검증에서 정상으로 보여야 하는 큰 글상자 하단 `5장`, `6장`, `에필로그` 목차 줄이 보이지 않는 문제가 확인됐다. 이는 글상자 문단 y 좌표를 `layout_textbox_content` 가 이미 `line_seg.vertical_pos` 로 잡은 뒤, `layout_composed_paragraph` 의 본문 column-top fallback 이 글상자 첫 문단에도 다시 vpos 를 더해 전체 목차를 아래로 밀었기 때문이다.

Stage 6 의 1차 보정은 `cell_ctx.is_none()` 으로 fallback 을 막았으나, `cell_ctx` 는 글상자뿐 아니라 표 셀 문단에도 사용되므로 CI 의 `svg_snapshot` 에서 표 셀 y 좌표 부작용을 만들었다. 최종 보정은 명시 플래그로 글상자 내부 문단에만 fallback 생략을 적용한다.

## 수정 요약

1. `tests/issue_1187_textbox_clip.rs` 를 추가해 `BookReview.hwp` 1쪽의 SVG 글상자 clipPath 회귀를 고정했다.
2. `src/renderer/layout/shape_layout.rs` 에서 글상자 내부 콘텐츠를 `RenderNodeType::TextBox` 노드 아래로 모았다.
3. `src/renderer/svg.rs` 에서 `RenderNodeType::TextBox` 에 `textbox-clip-{id}` clipPath 를 적용했다.
4. `src/paint` layer 경로에 `ClipKind::TextBox` 를 추가하고, `LayerBuilder` 가 TextBox 를 `ClipRect` 로 내리도록 했다.
5. PageLayerTree JSON 에 `"clipKind":"textBox"` 를 추가하면서 `schemaMinorVersion` 을 `14 -> 15` 로 올렸다.
6. `svg_layer`, `web_canvas`, `canvaskit_policy`, `rhwp-studio` 타입 정의를 새 clip kind 에 맞췄다.
7. `layout_composed_paragraph` 에 `suppress_column_top_vpos_fallback` 플래그를 추가해 글상자 내부 문단에만 column-top vpos fallback 생략을 적용했다.
8. #1187 테스트를 보강해 `5장`, `6장`, `에필로그` 줄이 큰 글상자 clip 하단 안에 남는지 확인한다.
9. PR #1190 CI 실패 보정으로 `issue-267`, `issue-617` SVG snapshot golden 을 텍스트박스 clip 출력에 맞춰 갱신했다.

## 산출물 확인

최종 SVG:

- `/private/tmp/rhwp-task1187-stage6-svg/BookReview_001.svg`

확인된 clipPath:

- `textbox-clip-33`
- `textbox-clip-52`
- `textbox-clip-103`

회귀가 발생한 큰 글상자 clip rect:

- `x=47.91999999999997`
- `y=516.5600000000001`
- `width=687.5466666666667`
- `height=487.88`

큰 글상자 콘텐츠는 `<g clip-path="url(#textbox-clip-52)">` 아래로 들어가며, 우측 하단 저자 정보 글상자는 별도 `textbox-clip-103` 으로 유지된다.

보정 후 주요 목차 baseline:

- `5장`: `y=887.34`
- `6장`: `y=924.03`
- `에필로그`: `y=960.73`

큰 글상자 clip bottom 은 `1004.44` 이므로 정상 목차 줄이 clip 안에 남는다.

## 검증

통과:

```bash
cargo fmt --check
git diff --check
cargo build --bin rhwp
cargo test --test svg_snapshot
cargo test --test issue_1187_textbox_clip
cargo test --lib paint::builder::tests
cargo test --lib paint::json::tests::serializes_textbox_clip_kind
cargo test --lib paint::schema::tests::layer_tree_schema_constants_match_schema
cargo test --lib renderer::svg_layer::tests
cargo test --test issue_1052_footnote_in_textbox
cargo test --test issue_919_textbox_hit_test
cargo test --test issue_1028_hwpx_textbox_vertical
wasm-pack build --target web --dev
npm run build
```

결과 요약:

- #1187 회귀 테스트: 2 passed
- paint builder 테스트: 7 passed
- svg layer 테스트: 3 passed
- 기존 글상자/각주/히트테스트/세로쓰기 관련 테스트: 모두 통과
- `wasm-pack build --target web --dev`: 통과
- `npm run build`: 통과
- PR #1190 CI 실패 재현 케이스 `issue_267_ktx_toc_page`, `issue_617_exam_kor_page5`: 통과

rhwp-studio 시각 검증:

- URL: `http://127.0.0.1:5175/?url=/samples/basic/BookReview.hwp&filename=BookReview.hwp`
- Browser: in-app Browser
- Vite overlay: 없음
- console error/warn: 없음
- 1쪽 큰 글상자 하단에서 `5장`, `6장`, `에필로그`, 저자 정보 글상자 표시 확인
- 스크린샷: `/private/tmp/rhwp-task1187-stage6-after.png`

## 커밋

- `90d16834` Task #1187 Stage 1: BookReview textbox clip regression test
- `fb351794` Task #1187 Stage 2: route textbox content through render node
- `befdc1f7` Task #1187 Stage 3: clip textbox content in SVG
- `ffdd45b4` Task #1187 Stage 4: clip textbox content in paint layer
- `cacbe332` Task #1187 Stage 5: finalize textbox clip fix
- `566eb3f1` Task #1187 Stage 6: correct textbox vpos fallback
- Stage 7: fix CI svg snapshot fallout

## 남은 절차

- PR #1190 갱신 가능.
- 이슈 #1187 close 는 작업지시자 승인 전에는 수행하지 않는다.
