# PR #1678 검토 - 브라우저 캔버스 이미지 리플레이 안정화

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1678](https://github.com/edwardkim/rhwp/pull/1678) |
| 제목 | `[렌더러 보정] 브라우저 캔버스 이미지 리플레이 안정화` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-canvas-replay` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 3 파일, +102/-0 |
| 원 PR 커밋 | `cac9415e3ac8` |
| 로컬 적용 커밋 | `ab62ef541` (`git cherry-pick -x`) |

## 2. 변경 범위

브라우저 `WebCanvas` 경로에서 image data URL을 `HtmlImageElement`에 넣은 직후 아직 decode가 끝나지 않아 첫 pass에서 이미지가 누락될 수 있는 문제를 줄이는 PR이다.

- `src/renderer/web_canvas.rs`에 bitmap byte를 동기적으로 `image::load_from_memory`로 디코드하는 경로를 추가한다.
- full/cropped 이미지가 첫 render pass에서 offscreen canvas 기반 `drawImage(canvas, ...)`로 그려질 수 있게 한다.
- WMF/PCX/SVG처럼 기존 fallback이 필요한 형식은 `HtmlImageElement` 경로에 남긴다.
- `tests/render_p22_web_canvas_contract.rs`가 추가되어 canvas contract를 좁게 검증한다.

## 3. 검토 의견

문제 범위가 browser canvas first-paint에 한정되어 있고, fallback 형식을 기존 경로에 남긴 점은 좋다. native SVG/PNG 렌더러보다는 WASM/browser path의 안정성 개선으로 보는 것이 맞다.

다만 `image` crate 기반 decode를 WebCanvas 경로에 추가하므로 WASM 빌드 크기와 supported image format 조합을 확인해야 하는 변경이다. 이번 검토에서는 #1676의 image payload 정규화와 함께 실제 WASM 빌드를 통과시켜 dependency impact가 빌드 실패로 이어지지 않음을 확인했다.

`HtmlImageElement` fallback을 완전히 제거하지 않았기 때문에 unsupported/async-only 형식의 회귀 위험은 낮아 보인다. 그래도 브라우저 실제 실행 또는 WASM build 검증 없이 merge하는 것은 부적절하다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 1개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태다. #1676의 이미지 정규화 이후 순서로 적용했으며, 현 로컬 브랜치에서는 두 변경이 함께 존재한다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과, `render_p22_web_canvas_contract` 포함)
- 완료: `cargo fmt --check`
- 완료: `git diff --check`
- 완료: `cargo clippy --all-targets -- -D warnings` (최초 공통 검증 18m 25s warning 0, TIFF 보강 후 최종 재실행 17m 28s warning 0)
- 완료: `cargo test --doc` (0 passed, 0 failed, 1 ignored)
- 완료: `cargo test --test svg_snapshot` (8 passed)
- 완료: `cd rhwp-studio && npx tsc --noEmit`
- 완료: `cd rhwp-studio && npm test` (153 passed)
- 완료: `wasm-pack build --target web --out-dir pkg` (1m 25s)
- 중단/무효: `cargo check --all-targets --message-format=short`는 workflow 문서에 없는 명령이라 중단했으며 검증 결과로 기록하지 않는다.

### 5.1 PR 내용별 targeted 검증

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1678 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| bitmap bytes를 `HtmlImageElement` fallback 전에 decode한다 | `cargo test --profile release-test --test render_p22_web_canvas_contract` 중 `web_canvas_decodes_bitmap_bytes_before_html_image_fallback` 통과 |
| replay plane ordering과 group label contract가 유지된다 | 같은 명령의 `web_canvas_all_filter_replays_logical_planes_in_order`, `web_canvas_control_code_group_labels_follow_active_replay_plane`, `web_canvas_layer_leaf_replay_does_not_rebuild_render_nodes` 통과 |

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. browser canvas image decode contract와 직접 연결된 failure는 확인되지 않았다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

수용 후보. browser canvas contract는 targeted 검증으로 통과했고, 실제 WASM 빌드도 확인했다. 이 PR 단독으로는 페이지 수를 바꾸는 변경은 아니지만, #1676과 결합해 이미지 payload 처리 경로가 바뀌므로 렌더 영향 PR로 다룬다.
