# PR #6668 검토 기록

- 원 PR: <https://github.com/edwardkim/rhwp/pull/6668>
- 제목: `renderer: make PageLayerTree the canonical SVG paint contract`
- 기여자 head: `295d1c29e69f5e4088bbbe25c12ece15997e0a47`
- 통합 검토 branch: `review/humdrum00001010-green-20260903`
- 통합 적용: provenance-preserving `-x` 3개, 마지막 `ca994cef7f99ddf306df2181f7699046103f6b0b`
- 연결 이슈: `Closes #6520` (병합 뒤 실제 auto-close 여부를 확인한다)

## 판정: 수용 가능

`PageLayerTree`를 production SVG의 canonical paint contract로 사용하도록 전환하고, 기존 SVG backend는 명시적 `--backend legacy` 진단 경로로 제한한다. 기본 layer 경로와 legacy 진단 경로의 option 혼용 차단, clip/plane replay, text visual source identity를 집중 검토했다.

## 로컬 검증

- `CARGO_TARGET_DIR=target/pr-review cargo nextest run --test regression_suite_015 -E 'test(issue_6520_svg_layer_contract::production_svg_routes_share_the_screen_layer_contract)|test(issue_6520_svg_layer_contract::cli_legacy_backend_is_explicit_and_cannot_override_layer_options)|test(issue_6520_svg_layer_contract::direct_replay_preserves_plane_order_nested_clips_and_debug_projection)'`: 3 passed
- `rhwp export-svg` 기본 경로가 `layer` backend임을 확인했다.
- `--backend legacy`는 명시적으로 동작하고 `--profile` 등 layer option과 함께 쓰면 거부됨을 확인했다.
- Studio `npm test`: 1373 passed, 1 skipped, 0 failed.
- Studio `npm run build`: 성공. Vite의 externalized Node module/large chunk warning만 관찰됐고 빌드 실패는 없었다.

## 시각 증적

- 대상: `samples/issue-617/exam-kor` page 5
- 동일 input에서 layer SVG와 explicit legacy SVG는 byte hash가 다르지만 `rsvg-convert` raster 비교의 absolute error는 `0`; 사람이 확인한 side-by-side 이미지도 동일했다.
- stable asset: `mydocs/pr/assets/pr_6666_6668_humdrum00001010_20260903/issue6520-layer-vs-legacy-page-005.png`
- 이 parity는 legacy 출력 보존을 보여 주는 보조 증적이며, Hancom PDF fidelity 전체를 주장하지 않는다.

## 병합 뒤 처리 계획

통합 PR의 최종 head CI와 devel post-merge CI가 모두 성공한 뒤에만 `#6668` 및 Issue `#6520`의 실제 close 상태를 확인하고, `post_merge.md`에 따라 comment/close를 처리한다. 이 검토 시점에는 원 PR/issue에 comment, close, merge를 수행하지 않았다.
