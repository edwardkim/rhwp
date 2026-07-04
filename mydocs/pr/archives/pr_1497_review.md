# PR #1497 검토 기록 - Web Canvas paint plane replay 순서 보정

## PR 메타

| 항목 | 내용 |
|---|---|
| 번호 | #1497 |
| 제목 | `fix: replay web canvas planes in paint order` |
| 작성자 | `humdrum00001010` |
| 작성자 상태 | 외부 contributor |
| base | `devel` |
| head | `humdrum00001010/codex/web-canvas-replay-order-devel` |
| 관련 이슈 | 없음. #1496 리뷰 피드백 반영 재제출 |
| 규모 | 문서 작성 시점 참고값: 2 files, +70 / -4 |
| PR 원 커밋 | `355c5e980aad0895e4b71ff1a76e4c67a9da252f` |
| maintainer can modify | 문서 작성 시점 참고값: true |
| review request | 문서 작성 시점 참고값: `postmelee` |
| labels / milestone | 문서 작성 시점 참고값: 없음 / 없음 |
| merge 상태 | 문서 작성 시점 참고값: 원 PR head는 `BEHIND`, 최신 `upstream/devel` merge commit 반영 후 재확인 필요 |
| CI 상태 | 문서 작성 시점 참고값: 원 PR head 기준 GitHub Actions 통과, 문서/merge commit push 후 최신 head 기준 재확인 필요 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 확정 사실로 기록하지 않는다.
최종 merge 판단은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후에만 수행한다.

## 배경

이 PR은 #1496에서 지적한 base branch 문제를 정리해 `devel` 기준으로 다시 제출한 PR이다.
#1496은 base가 `main`이어서 `upstream/devel..upstream/pr/1496` 비교에 실제 수정 외 main-only 커밋이 섞였다.
#1497은 base가 `devel`이고, PR diff는 Web Canvas 렌더러와 계약 테스트 2파일로 제한되어 있다.

PR 본문에 따르면 재현 문서는 page 4에서 behind-text full-page group이 body text보다 raw layer tree 뒤쪽에 나타나는
케이스다. Web Canvas `LayerFilter::All`이 raw child 순서 그대로 layer tree를 순회하면 behind-text group이
본문 텍스트 위에 그려져 TOC text가 가려질 수 있다. Skia 렌더러는 이미 `PaintReplayPlane::ORDERED` 순서로
replay하므로, Web Canvas도 같은 logical paint-plane 순서를 따르게 하는 것이 수정의 핵심이다.

## 변경 범위 분석

### `src/renderer/web_canvas.rs`

- `WebCanvasRenderer`에 `active_replay_plane: Option<PaintReplayPlane>` 상태를 추가했다.
- `LayerFilter::All`일 때 `PaintReplayPlane::ORDERED`를 순회하면서 layer tree를 plane별로 replay한다.
- 각 pass 전에 `layer_node_has_replay_plane`으로 해당 plane에 그릴 내용이 있는지 확인해 불필요한 replay를 건너뛴다.
- `should_render_op`에서 active plane이 있으면 `paint_op_replay_plane_with_layer` 결과와 일치하는 PaintOp만 그린다.
- replay 후 이전 `active_replay_plane`을 복원하므로, 다른 `LayerFilter` 경로나 이후 렌더 호출에 상태가 남지 않는다.

### `show_control_codes` group label 보정

#1496 검토에서 `show_control_codes`가 켜진 경우 `[표]`, `[글상자]`, `[머리말]` 같은 group label은 `PaintOp`가 아니어서
plane별 replay pass마다 중복 출력될 수 있다는 우려가 있었다.

#1497은 이 부분을 추가로 보정했다.

- `group_label_matches_replay_plane`을 추가해 inherited layer metadata에서 계산한 `render_layer_replay_plane`과
  active plane을 비교한다.
- `should_render_group_label`을 통해 `show_control_codes` 조건과 replay plane 조건을 함께 적용한다.
- `render_layer_node`의 group label 출력 지점이 `self.show_control_codes` 직접 확인에서
  `self.should_render_group_label(active_layer)` 호출로 바뀌었다.

따라서 일반 PaintOp뿐 아니라 control-code group label도 active replay plane에 맞는 pass에서만 출력된다.

### `tests/render_p22_web_canvas_contract.rs`

기존 Web Canvas 계약 테스트에 다음 확인이 추가되었다.

- `LayerFilter::All`에서 `active_replay_plane`을 추적하는지 확인
- `PaintReplayPlane::ORDERED` 순회가 존재하는지 확인
- 각 pass에서 active plane을 설정하는지 확인
- group label 출력이 별도 gate를 통과하는지 확인

테스트는 현재 source 문자열 기반 계약 테스트다. 구현 의도와 핵심 구조를 고정하는 데는 도움이 되지만, 실제 paint 결과나
시각 순서 회귀를 강하게 막는 테스트는 아니다. PR 본문의 Canvas2D 비교와 Render Diff CI는 긍정적인 보조 근거이며,
장기적으로는 재현 fixture나 `PageLayerTree` 기반 순서 검증이 더 강한 회귀 테스트가 된다.

## 로컬 검증 결과

검증은 2026-06-24 KST 기준 최신 `upstream/devel`(`b1024f6b`)을 PR head에 merge commit으로 반영한 임시
worktree에서 수행했다.

- 최신 `devel` merge: 충돌 없음
- `git diff --check upstream/devel...HEAD`: passed
- `cargo fmt --check`: passed
- `cargo test --test render_p22_web_canvas_contract`: passed, 3 passed
- `cargo check --target wasm32-unknown-unknown --lib`: passed

## 주요 문제점 / 리스크

- 관련 이슈가 PR 본문에 명시되어 있지 않다. 다만 본 PR은 #1496 리뷰 피드백을 반영한 `devel` 기준 재제출이고,
  PR 본문에 재현 HWPX와 Canvas2D 비교 근거가 포함되어 있다. 변경 범위가 작고 Web Canvas replay order 결함에 직접
  대응하므로, 연결 이슈 부재는 merge blocker로 보지 않는다. 자동 close 대상 issue는 없다.
- 추가 테스트는 source 문자열 검사다. 현재 변경처럼 구조적 계약을 고정하는 용도에는 맞지만, 실제 canvas paint 결과,
  plane 순서, label 중복 출력을 픽셀 또는 render tree 수준에서 검증하지는 않는다.
- `show_control_codes` group label은 active layer metadata에 의존해 replay plane을 판단한다. 현재
  `render_layer_node`의 inherited `active_layer` 흐름과 맞지만, 향후 layer metadata가 없는 label 유형이 추가되면
  같은 관점의 재검토가 필요하다.
- 수정은 Web Canvas `LayerFilter::All` 경로를 건드린다. Skia 렌더러와 개별 `LayerFilter` 경로는 의도상 기존
  동작을 유지한다.

## 최종 권고

조건부 merge 후보로 본다.

최종 merge 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- 이 review 문서와 `pr_1497_review_impl.md`가 PR diff에 포함됨
- GitHub review 또는 PR comment로 검토 결과를 contributor에게 남김
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 승인
