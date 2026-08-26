# task_m100_4431 Stage 1 — RenderNode.dirty 죽은 플래그 제거

- **이슈**: [#4431](https://github.com/edwardkim/rhwp/issues/4431)
- **브랜치**: `fix/issue-4431-render-node-dead-dirty`
- **분기 기준**: `upstream/devel` `9f5911e86` (0 behind)
- **상태**: 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 이슈 주장 중 틀린 것을 먼저 정정한다

이슈 본문은 `RenderNode` 가 `Serialize` 를 파생하고 `dirty` 에 `skip_serializing_if` 가 없으니
**모든 노드가 `"dirty":true` 를 싣고 wasm 경계를 넘는다**고 적었다. **아니다.**

파생은 있지만 실제로 쓰이는 JSON 경로는 손으로 쓴 `write_json`(`render_tree.rs:187`,
`to_json` `:181`)이고 이 함수는 `dirty` 를 방출하지 않는다. `serde_json::to_string` 이나
`serde_wasm_bindgen` 으로 트리를 내보내는 호출부도 `src/` 안에 없다.

실측으로 확인했다 — `export-render-tree` 로 393쪽 문서를 내보낸 JSON **5,729,677바이트,
노드 61,389개**에 `"dirty"` 가 **0건**이다.

**바이트 절감은 0이다.** 이슈에 코멘트로 정정하고 제목도 고쳤다.

## 2. 남는 근거 — 관측자 없는 관측자 패턴

읽는 쪽이 없다는 것은 맞다. 다만 "아무도 안 건드린다"도 정확하지 않았다 — **쓰는 쪽은
프로덕션에 있었다.**

- 쓰기: `src/document_core/queries/rendering.rs:6004-6005` 의 `line_node.invalidate()`,
  `tree.root.invalidate()`
- 읽기: `has_dirty_nodes`(`render_tree.rs:173`), `RenderTree::has_dirty_nodes`(`:1548`) —
  **프로덕션 호출부 없음**

값이 어디로도 흘러가지 않는 막다른 플래그다. 지운다.

## 3. 지운 것

`RenderNode.dirty` 필드, `invalidate`, `mark_clean`, `mark_clean_recursive`,
`has_dirty_nodes`, `RenderTree::has_dirty_nodes`, 그리고 이들만 검증하던
`test_render_node_dirty_flag`. 유일한 프로덕션 쓰기 두 줄도 함께 제거했다.

총 59줄 삭제, 추가 0줄.

## 4. 검증

동작이 바뀌지 않으므로 red→green 쌍이 없다. **호출부가 없다는 것은 컴파일이 증명한다** —
필드와 함수를 지우고도 전 크레이트가 빌드되면 그것이 증거다.

- `cargo fmt --all -- --check` exit 0
- `cargo clippy --all-targets -- -D warnings` exit 0
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` exit 0 —
  `test result: ok` 블록 **502개, FAILED 0건**

렌더러 범위이므로 `local_validation.md` §4.3 추가 게이트도 실행했다.

- `cargo test --profile release-test --features skia --lib` exit 0
- `cargo test --profile release-test --features skia --test issue_2225_missing_picture_placeholder` exit 0
- `cargo test --profile release-test --features skia --test render_p37_direct_pdf_export` exit 0
- `wasm-pack build --target web` 성공

**시각 증거는 별도로 만들지 않았다.** 이 변경이 렌더 출력에 닿을 수 있는 유일한 경로가
`write_json` 인데 그 함수가 `dirty` 를 애초에 방출하지 않음을 소스와 실측 양쪽으로 확인했고,
Skia 3종이 실제 래스터 경로를 통과했다. 출력 바이트가 달라질 수 있는 자리가 없다.

## 5. 미처리

GitHub Actions, 작업지시자 승인, merge.
