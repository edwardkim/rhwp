# PR #1312 리뷰 — renderer baseline sweep reports

**작성일**: 2026-06-08  
**PR**: https://github.com/edwardkim/rhwp/pull/1312  
**작성자**: `seo-rii`  
**제목**: `render: add renderer baseline sweep reports`

## 1. 메타

| 항목 | 값 |
|---|---|
| base | `devel` |
| head | `render-p21` |
| draft | false |
| mergeable | true |
| commits | 10 |
| changed files | 11 |
| 규모 | +2699 / -34 |
| 관련 이슈 | Refs #536 |

## 2. 변경 범위

이번 PR은 기본 렌더링 동작을 CI gate 수준에서 바꾸는 PR이 아니라, renderer 전환/비교를 위한 수동 진단 파이프라인을 추가하는 PR이다.

주요 변경:

- `.github/workflows/full-renderer-sweep.yml`
  - `workflow_dispatch` 전용 full renderer sweep workflow 추가.
  - legacy SVG, layer SVG, native Skia PNG, browser Canvas2D/CanvasKit 산출물과 report artifact를 수집.
- `scripts/renderer_baseline.py`
  - manifest 기반 baseline capture driver 추가.
  - native/browser capture, CanvasKit surface별 report, performance summary, parity report 생성.
- `scripts/renderer_baseline_manifest.json`
  - 대표 샘플 20개 corpus 정의.
- `rhwp-studio/e2e/renderer-baseline*.mjs`
  - browser Canvas2D/CanvasKit capture 및 native Skia vs CanvasKit diff report 생성.
- `src/paint/replay_order.rs`
  - layer tree의 replay plane 포함 여부 판정 helper 추가.
- `src/renderer/skia/renderer.rs`, `src/renderer/web_canvas.rs`
  - replay plane이 없는 경우 불필요한 plane replay를 건너뛰도록 shared helper 사용.

## 3. GitHub 상태

GitHub checks:

- Build & Test: pass
- CodeQL: pass
- Canvas visual diff: pass
- Analyze javascript-typescript/python/rust: pass
- WASM Build: skipped

Copilot/CodeQL 코멘트 확인:

- CodeQL regex injection 코멘트는 outdated/resolved.
- Copilot 코멘트 6건은 현재 코드 기준으로 반영 확인.
  - `comparePngBuffers()`가 `ignoreChannelDelta`, `maxDiffRatio` 반환.
  - manifest/report 경로는 `repo_relative()` 또는 `try/except` fallback 사용.
  - Chrome cache 탐색의 `readdirSync()` 예외를 skip 처리.
  - 불필요한 `exactDiff` buffer allocation 제거.
  - invalid `--canvaskit-surface`는 error 처리.
  - `layer_node_has_replay_plane`은 `pub(crate)`이며 `paint` public re-export에서 제외.

## 4. 로컬 검증

PR branch: `local/pr1312-upstream`

통과:

- `cargo fmt --all -- --check`
- `cargo check --lib`
- `cargo test replay_order --lib`
- `node --check rhwp-studio/e2e/helpers.mjs`
- `node --check rhwp-studio/e2e/renderer-baseline.mjs`
- `node --check rhwp-studio/e2e/renderer-baseline-native-diff.mjs`
- `python3 -m json.tool scripts/renderer_baseline_manifest.json`
- `python3 scripts/renderer_baseline.py --help`
- `python3 scripts/renderer_baseline.py --skip-browser --skip-native --filter paragraph-basic --output /tmp/rhwp-renderer-baseline-smoke-pr1312`
- `npm --prefix rhwp-studio run build`
- `docker compose --env-file .env.docker run --rm wasm`

참고:

- 로컬 환경에는 `wasm-pack` 실행 파일이 없어 PR 본문의 `wasm-pack build --target web --release --no-opt`는 직접 실행하지 못했다.
- 프로젝트 표준 WASM 검증 경로인 Docker WASM 빌드는 성공했다.

## 5. 리스크 평가

- 수동 workflow이므로 기본 PR gate를 무겁게 만들지 않는다.
- baseline runner는 샘플 경로를 `samples/` 상대 경로로 제한하고 URL/절대 경로/상위 디렉터리 escape를 방지한다.
- 새 dependency `pixelmatch`, `pngjs`는 rhwp-studio baseline report 생성 목적이며 package-lock에 반영되어 있다.
- `layer_node_has_replay_plane` helper는 internal renderer plumbing으로 유지되어 public API 고정 리스크가 낮다.
- #536은 `Refs` 성격이므로 이번 PR merge 후 자동 close 대상은 아니다.

## 6. 권고

**수용 권고**.

사유:

- PR 목적이 수동 renderer baseline sweep 체계 구축으로 명확하다.
- 기본 렌더링 의미 변경은 제한적이며, replay plane skip helper는 WebCanvas/Skia 경로의 중복 제거와 최적화에 가깝다.
- GitHub checks와 로컬 검증이 모두 통과했다.
- bot review 지적사항은 현재 head 기준으로 코드 반영 확인됐다.

권장 절차:

1. Copilot unresolved thread 중 코드 반영 완료된 항목 resolve.
2. PR #1312 admin merge.
3. PR 감사 코멘트 작성.
4. `local/devel` sync.
5. 리뷰 문서 archive 및 `mydocs/orders/20260608.md` 기록.
