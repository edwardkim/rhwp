# task_m100_3790 Stage 4 — Rust·Native Skia 조건화

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `issue-3790-stage4-rust-native`
- **최신 동기화 기준**: `upstream/devel` `d3fb9de7c0c0648e3d8126c25467e2c78a054337`
- **첫 devel merge head**: `b0be8673149bbd00ebb67f6d5e62b70025cfa612`
- **최종 code head**: `5eeab15fd291b2b4b27d3b8a77498fcc0ca5723b`
- **merge commit**: `8d48a4c07fad6bcccebbc2adddef4685456bb313`
- **상태**: 완료. PR #4032 merge, canary PR #4078 실측 완료
- **기록일**: 2026-08-05 KST, canary 실측 2026-08-06 KST

## 선행 canary

Stage 3 canary PR #3951의 selective run은 `frontend_mode=unit`, `render_required=false`를 판정했고 unit
gate만 59초에 실행했다. 같은 SHA의 수동 full에서 package gate는 2분 47초, Canvas는 5분 59초에
성공했으므로 Stage 3의 직접 runner time 절감은 7분 47초다. 수동 full 전체를 중단시킨 기존 cold
release archive 30분 timeout은 #4029에서 별도로 추적하며 Stage 4 영향축 판단 근거와 섞지 않는다.

## 변경 요약

preflight의 `rust_required`와 `native_skia_required`를 CI job 조건과 aggregate 진리표에 연결했다.

| 영향축 | Rust lint·3 builders·4 workers | Native Skia |
| --- | --- | --- |
| `rust=true`, `native=true` | 모두 `success` | `success` |
| `rust=true`, `native=false` | 모두 `success` | `skipped` |
| `rust=false`, `native=true` | 모두 `skipped` | `success` |
| `rust=false`, `native=false` | 모두 `skipped` | `skipped` |

worker는 해당 builder의 성공에 더해 Native job이 영향축과 일치하는 `success|skipped`인지 확인한 뒤
실행한다. aggregate는 각 job 결과를 개별적으로 검증하며 알 수 없는 축 값이나 부분 성공을 실패시킨다.
review-only fast-pass와 Stage 3의 frontend `none|unit|package` 진리표는 유지한다. CodeQL 언어 조건화는
Stage 5까지 기존 동작을 유지한다.

## 분류 경계 보완

Native Skia job의 실제 `cargo test --test` 대상과 classifier 경로를 전수 대조했다. 일반 Rust 경로로
분류되던 아래 두 통합 테스트 변경은 Native 검증을 건너뛸 수 있어 classifier v2의 독립 경계로 고정했다.

- `tests/issue_2225_missing_picture_placeholder.rs`
- `tests/render_p37_direct_pdf_export.rs`

두 파일은 `rust_required=true`, `native_skia_required=true`, `render_required=false`로 판정한다.

review F1에서 default-feature 테스트가 직접 소비하는 데이터 경계를 추가로 확인했다.

- `ttfs/**`·`tests/fixtures/fonts/**`의 `.otf|.ttc|.ttf|.woff|.woff2`
- `samples/render-p35-font-native-bitmap.hwpx`

이 경로는 `rust=true`, `native=true`, `render=true`, data-only이므로 `codeql=none`으로 판정한다.
`assets/fonts/**`, render 생성 Python, render 문서는 `rust=false`, `native=true`를 유지해 불필요하게
Rust lane을 넓히지 않는다. workflow·classifier·Cargo·WASM·rename·미분류 변경은 계속 full로 닫힌다.

## review F1–F6 보정

| 항목 | 대응 |
| --- | --- |
| F1 | Rust test-owned font/HWPX 입력을 `rust-test-input`으로 분리하고 과대 분류 방지 테스트를 추가 |
| F2 | Native Skia job의 frontend `none|unit|package` 진리표 정적 단언 복구 |
| F3 | aggregate harness를 다음 step 또는 job 경계에서 자르고 GitHub과 같이 `bash -e -o pipefail`로 실행 |
| F4 | canonical `pr_review_workflow.md` §3.1을 Stage 4 조건부 그래프로 갱신 |
| F5 | 기존 Native Skia test 누락을 [#4040](https://github.com/edwardkim/rhwp/issues/4040)으로 분리 |
| F6 | `mydocs/pr/archives/pr_4032_review.md`와 2026-08-05 오늘 기록을 trailing commit으로 준비 |

## 검증

| 검증 | 결과 |
| --- | --- |
| `actionlint .github/workflows/ci.yml .github/workflows/render-diff.yml` | 통과 |
| `node --check scripts/ci-impact-classifier.cjs` | 통과 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 27/27 통과 |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py scripts/tests/test_render_diff_workflow.py` | 22/22 통과 |
| aggregate shell 진리표 실행 — frontend-only, Rust 비렌더, Rust render, 비-Rust Native | 모두 통과 |
| aggregate shell 불일치·unknown 축 입력 | 모두 의도대로 실패 |
| `git diff --check` | 통과 |

장시간 Rust 전체 CI는 workflow/classifier 변경이 `fail-closed:classifier-contract`로 진입한 원격
full lane에서 확인했다.

- 보정 head `1f12a5fe0`: CI 30923071182, Render Diff 30923070493, CodeQL 30923070506 통과
- 첫 devel merge head `b0be86731`: CI 30924641673, Render Diff 30924638772, CodeQL 30924638749 통과
- 그 aggregate: shard `3698+693+840+1=5232`, expected runnable `5232` 일치
- 최종 code head `5eeab15fd`: CI 31004297167, Render Diff 31004296886, CodeQL 31004296907 통과
- 최종 aggregate: shard `3714+753+784+1=5252`, expected runnable `5252` 일치, preflight
  `no-trailing-review-only-commits`로 full lane 진입, `frontend_mode=package`로 Frontend unit gates만 skip

- review-only trailing commit `3a5c6587c`: preflight가 `fast_pass=true`,
  `reason=build-and-test-green:success`, `candidate_sha=5eeab15fd`로 `5eeab15fd`의 녹색
  `Build & Test`를 재사용했다. heavy worker 전량 skip, CI 89초·CodeQL 70초·Render Diff 22초.
- merge 뒤 devel push full lane(`8d48a4c07`): CI 31026511582, CodeQL 31026511634 통과.

## canary 실측 — PR #4078

Stage 3 canary [PR #3951](https://github.com/edwardkim/rhwp/pull/3951)과 같은 2파일 frontend-only
변경(`rhwp-studio/src/command/shortcut-map.ts`, `rhwp-studio/tests/shortcut-map.test.ts`)을 써서
Stage 4만이 차이가 되도록 대조했다. canary head는 `eac12e9e3`, base는 `8d48a4c07`다.
classifier v2는 `frontend_mode=unit`, `render_required=false`, `rust_required=false`,
`native_skia_required=false`, `reason=classified:studio-unit`을 판정했다.

| job | #3951 Stage 3 | #4078 Stage 4 |
| --- | --- | --- |
| CI preflight | 9s | 7s |
| Frontend unit gates | 59s | 127s |
| Frontend package gates | skipped | skipped |
| Lint (fmt, clippy, WASM check) | 218s | skipped |
| build-test-archive-a / -b / -slow | 387s / 239s / 324s | 모두 skipped |
| test-regular-shard-1 / -2 / -3 | 202s / 180s / 133s | 모두 skipped |
| test-slow-shard | 228s | skipped |
| Native Skia tests | 368s | skipped |
| WASM Build | skipped | skipped |
| Build & Test aggregate | 8s success | 3s success |

| workflow | #3951 runner | #4078 runner | #3951 wall | #4078 wall |
| --- | --- | --- | --- | --- |
| CI | 2,355s | 137s | 857s | 148s |
| CodeQL | 875s | 794s | 655s | 575s |
| Render Diff | 8s | 8s | 12s | 12s |
| 합계 | 3,238s | 939s | 857s | 575s |

Stage 4가 새로 생략한 9개 job의 직접 runner time은 2,279초다. CI workflow runner time은 2,218초
(94.2%), 세 workflow 합계는 2,299초(71.0%) 줄었고 wall clock은 857초에서 575초가 됐다.
`Build & Test` aggregate가 이 `success|skipped` 조합을 수용해 Stage 4 진리표도 실제 selective run에서
검증됐다.

Frontend unit gates의 59초→127초 증가는 조건화 결과가 아니라 Studio 테스트가 764건으로 늘어난 영향과
runner 편차이므로 절감 계산에서 분리한다. CodeQL 차이(875초→794초)도 runner 편차이며, 언어 조건화는
Stage 5 범위라 3개 언어를 그대로 분석했다.

측정 뒤 #4078은 merge하지 않고 close했고 branch·worktree를 정리했다.

## cache 기준선 대조 — 완료

스윕 직후 동일 조건으로 대조했다.

| 시점 (UTC) | 정리 전 | 정리 대상 | 정리 후 |
| --- | --- | --- | --- |
| #3810 기준선 08-02 14:43 수동 | 42개 / 10.13GB | 18개 / 5.40GB | 24개 / 4.73GB |
| 08-02 19:39 cron | 31개 / 5.91GB | 1개 / 0.06GB | 30개 / 5.85GB |
| 08-03 20:03 cron | 50개 / 10.24GB | 10개 / 3.08GB | 40개 / 7.16GB |
| 08-04 20:03 cron | 50개 / 8.64GB | 1개 / 0.18GB | 49개 / 8.46GB |
| 08-05 17:28 dry-run | 53개 / 10.01GB | 3개 / 1.17GB | 50개 / 8.84GB |

기준선 대비 +4.11GB(+87%), 무료 한도 10GB의 88%다. 마지막 값은 비파괴
`workflow_dispatch dry_run=true` 실행 [31030157435](https://github.com/edwardkim/rhwp/actions/runs/31030157435)의
예상치이며 삭제는 일어나지 않았다.

Stage 4는 원인이 아니다. Stage 4 merge는 08-05 16:42 UTC인데 회귀 추세는 그 전에 완성돼 있었고,
Stage 4는 오히려 frontend-only PR에서 Rust lane 캐시 생성을 줄인다. 세대 상한도 정상으로,
(그룹, ref) 쌍 42개 중 2세대 초과는 3개뿐이며 전부 마지막 스윕 이후 생성분이다. 실제 원인은
쌍 수 증가로 KEEP=2의 하한이 올라간 것과 삭제된 브랜치의 고아 캐시(약 0.53GB)다. 대응은
[#4080](https://github.com/edwardkim/rhwp/issues/4080)으로 분리했다.

## 다음 단계

1. Stage 5 CodeQL 언어 조건화로 진행한다. Stage 4 이후 frontend-only PR의 critical path가 CI에서
   CodeQL로 옮겨갔고, wall clock 575초 중 `Analyze (rust)` 단독이 563초를 차지한다. 언어 조건화가
   적용되면 `Analyze (javascript-typescript)` 수준(약 140초)까지 내려갈 여지가 있다.
