# PR #1346 처리 보고서 — WebCanvas layer adapter 축소 및 option metadata 분리

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1346 |
| 작성자 | `seo-rii` |
| 관련 이슈 | #536 |
| 검토 브랜치 | `local/pr1346-upstream` |
| 통합 방식 | 현재 `local/devel` 기준 non-merge commit cherry-pick |
| 원 PR head | `58168282` |
| 반영 커밋 | `b5edc1db`, `4fb6a74f`, `d5aa4ebc` |
| devel merge | `2bc411f4` |
| PR close | `2026-06-09T23:53:48Z` |

## 2. 처리 내용

PR #1346의 non-merge commit 3개를 현재 `local/devel` 위에 cherry-pick했다.

반영 커밋:

- `b5edc1db` — `refactor(render): replay WebCanvas PaintOps directly`
- `4fb6a74f` — `feat(render): split layer option metadata`
- `d5aa4ebc` — `fix(render): backfill layer option compatibility mirrors`

PR 브랜치에 포함되어 있던 merge commit `68683a26`은 수용 대상에서 제외했다.

핵심 변경:

- WebCanvas layer replay에서 `PaintOp`를 임시 `RenderNode`로 재조립하던 adapter 제거
- RenderNode 경로와 PaintOp 직접 replay 경로가 공통 leaf drawing helper를 사용하도록 정리
- PageLayerTree JSON schema minor를 `1.16`으로 갱신
- `buildOptions`/`debugOptions`를 canonical metadata로 추가
- 기존 `outputOptions` mirror를 유지해 구 소비자 호환성 보존
- Studio wasm bridge에서 old/new option metadata normalize 보강

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

Cherry-pick 후 로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib serializes_layer_option_metadata -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_page_layer_tree_export_preserves_output_options -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib layer_tree_schema_constants_match_schema -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --test render_p22_web_canvas_contract -- --nocapture` | 통과 |
| `cd rhwp-studio && node --test tests/render-backend.test.ts` | 통과, 23 passed |
| `CARGO_INCREMENTAL=0 cargo check --lib --target wasm32-unknown-unknown -j 2` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과, `Done in 1m 49s` |

WASM 빌드는 1차 시도에서 Docker daemon 미실행으로 실패했으나, Docker Desktop 기동 후 재시도하여 통과했다.

## 4. 기술 판정

**수용 가능**.

이번 PR은 WebCanvas layer replay의 전환기 adapter를 줄이고 PageLayerTree/PaintOp 계약에 더
직접적으로 맞추는 내부 구조 개선이다. `RenderNode::new` 재조립을 제거한 것은 PageLayerTree를
"렌더 트리 복제본"이 아니라 "paint replay 계약"으로 독립시키는 방향에 맞다.

PageLayerTree option metadata 분리는 의미상 타당하다. `clipEnabled`와 `debugOverlay`는 단순
출력 옵션보다는 build/debug 정책에 가까우므로 `buildOptions`/`debugOptions`로 분리하는 것이
계약을 명확하게 한다. 동시에 기존 `outputOptions` mirror가 유지되어 단기 호환성 리스크는 낮다.

남는 리스크는 WebCanvas state reset 계열이다. 다만 filter/global alpha/transform reset 경로를
검토했고, GitHub Canvas visual diff와 로컬 WASM 빌드가 통과했으므로 수용 가능한 수준으로 본다.

## 5. 시각 판정

현재 자동 시각 검증은 GitHub Canvas visual diff에서 통과했다.

다만 WebCanvas replay 경로를 직접 수정하는 PR이므로, 메인테이너 직접 시각 판정은 최종 수용 전
게이트로 둘 수 있다. 직접 시각 판정을 생략할 경우 근거는 다음과 같다.

- PR 자체 GitHub Canvas visual diff pass
- PR 변경이 렌더 결과 의도 변경이 아니라 adapter 축소와 metadata 분리 중심
- cherry-pick 후 Rust/Studio/WASM 검증 통과

## 6. 후속 절차

처리 완료:

- [x] `mydocs/pr/pr_1346_report.md` 및 주문서 갱신 커밋 — `fd734a48`
- [x] `local/devel` → `devel` no-ff merge — `2bc411f4`
- [x] `origin/devel` push — `2bc411f4`
- [x] PR #1346에 처리 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1346#issuecomment-4665140905
- [x] PR #1346 close — `2026-06-09T23:53:48Z`
- [x] 관련 이슈 #536은 tracking issue이므로 close하지 않음
