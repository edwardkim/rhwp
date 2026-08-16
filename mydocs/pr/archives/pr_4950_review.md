---
kind: pr-review
status: code-ci-complete
pr: 4950
author: edwardkim
base: devel
head: task_m100_4939
last_verified: 2026-08-16
---

# PR #4950 자체검토 - 폰트 메트릭·fallback 규칙 원장과 회귀 기준선

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#4950](https://github.com/edwardkim/rhwp/pull/4950) |
| 작성자 | `edwardkim` (collaborator self-review) |
| base / head | `devel` / `task_m100_4939` |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| code candidate | `3b5da40858d2e1b1bfb5e205cdc7f17f3141c74c` |
| 규모 | 23 files, +120,713 / -0, 8 commits |
| 관련 이슈 | [#4939](https://github.com/edwardkim/rhwp/issues/4939) |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 기록 당시 PR은 Open non-draft이며
`MERGEABLE`이었다. merge 직전에는 review-only trailing head, GitHub Actions와 mergeability를 다시 확인한다.

base route: `collaborator_self_merge.md`

modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
`rework_and_exceptions.md`, `visual_fixture_evidence.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_self_merge.md`,
`intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
`rework_and_exceptions.md`, `visual_fixture_evidence.md`

## 변경 검토

- W0는 30개 source boundary와 12개 owner, `FONT_METRICS` 600행·401개 unique name·1,856개 lookup
  projection을 결정론적 baseline으로 고정한다.
- W1은 실행 source에서 1,352개 candidate를 수집하고, evidence와 backend/profile을 갖춘 1,507개 ledger
  rule로 판정한다. 승인된 profile split 154개 외에는 candidate 하나가 정확히 ledger 한 행에 대응한다.
- 근거가 부족한 44개 규칙은 추정·삭제하지 않고 `unknown`으로 유지한다. identity alias는 0개이며,
  14개 conflict group과 5개 self-loop는 precedence·known limitation을 명시했다.
- Canvas2D CSS supply와 CanvasKit SFNT supply, layout metric과 paint, source exact·official successor·Hancom
  missing-font oracle를 서로 다른 decision plane과 profile로 보존한다.
- `src/`, `rhwp-studio/src/`, `web/`, font asset과 HWP/HWPX/PDF fixture는 변경하지 않았다. JSON 원장은
  historical investigation snapshot이며 runtime registry로 import하지 않는다.
- private 10k corpus 원문·본문·식별 파일 목록과 저장소 밖 font bytes는 포함하지 않았다.

생성 원장 때문에 1,000줄을 크게 넘는 대형 PR이다. 그러나 대다수 증가는 canonical JSON이며, schema,
source digest, candidate coverage, conflict/order, cycle, identity와 orphan evidence validator로 수동 전수 판독을
대체할 결정적 감사 경로를 제공한다. 따라서 즉시 admin merge하지 않고 code candidate와 trailing head의 CI를
각각 확인한다.

## CodeQL 보정

최초 candidate `c9fb50de4`의 CodeQL은 `scripts/font_rule_candidates.mjs`의 Studio substitution tuple 정규식
두 곳에서 `js/redos` High 경고 4건을 보고했다. escape 대안 `\\.`과 일반 문자 대안 `[^']`가 역슬래시를
동시에 소비할 수 있어 백트래킹 경로가 겹친 것이 원인이었다.

보정 commit `3b5da4085`는 일반 문자 대안을 `[^'\\]`로 제한해 두 분기를 상호 배타적으로 만들었다.
10,000개 escape pair를 넣는 회귀 시험을 추가했고, candidate JSON은 저장본과 byte-identical이었다. 보정
head의 JavaScript/TypeScript CodeQL과 최종 CodeQL check가 성공해 신규 경고 4건이 해소됐음을 확인했다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Stage 1~4 Node contract | 26/26 PASS; ReDoS 회귀 1건 포함 |
| candidate 재생성 | 1,352개, 저장본과 byte equal |
| ledger validator | 1,507 rules, error 0 |
| W0 baseline 재생성 | byte equal, SHA-256 `a0fac05c…b466` |
| W1 ledger 재생성 | byte equal, SHA-256 `284afd72…8c23` |
| Rust font metrics | 9/9 PASS |
| Studio font contract | 33/33 PASS |
| frontend font asset | 6/6 PASS |
| fresh native/WASM build | 각각 성공 |
| native/WASM parity | 공개 sample 7개, 167페이지, mismatch 0 |
| 최신 devel merge simulation | tree `5c7c456bc5719bc4b0472faf9a89c5eda7a725e6`, conflict 0 |
| `git diff --check` | 통과 |

fresh native/WASM parity는 `c9fb50de4`에서 수행했다. 이후 `3b5da4085`는 Node 조사 수집기의 정규식과 그
테스트만 바꾸며 Rust·Studio·WASM source와 fixture를 바꾸지 않았다. 따라서 동일 binary parity를 다시
생성하지 않고 exact code candidate의 GitHub Full CI·Native Skia 결과를 함께 확인했다.

## GitHub code candidate CI

exact code candidate `3b5da4085`의 [CI run 31946428992](https://github.com/edwardkim/rhwp/actions/runs/31946428992)와
[CodeQL run 31946428855](https://github.com/edwardkim/rhwp/actions/runs/31946428855)를 완료까지 관찰했다.

| CI | 결과 |
| --- | --- |
| CI / CodeQL preflight | 성공 / 성공 |
| Lint (fmt, clippy, WASM check) | 성공 |
| Frontend package gates | 성공 |
| Native Skia tests | 성공 |
| test archive builders | 1, 3, slow+2 모두 성공 |
| regular test shards | 1/3, 2/3, 3/3 모두 성공 |
| slow shard | 성공 |
| Build & Test aggregate | 성공 |
| CodeQL | JavaScript/TypeScript, Python, Rust와 최종 check 모두 성공 |
| 영향 정책상 skip | WASM Build, Frontend unit gates |

## 시각·fixture 판정

제품 font selection, renderer/layout/paint 경로, sample, golden과 기준 PDF를 변경하지 않았다. 따라서 새
시각 개선 주장을 위한 PDF pixel sweep이나 대표 review asset은 만들지 않았다. 기존 공개 fixture 7개
167페이지의 native/WASM byte parity와 제품 source 0-delta를 출력 무회귀 근거로 사용했다.

## 위험과 후속 경계

- 44개 `unknown`은 누락이 아니라 명시적 조사 대기 상태다. 이번 PR에서 임의 target이나 identity로 승격하지
  않는다.
- W2 Font Decision Trace, W3 이후 coverage·장평·자간·고정 프레임 계측, W5 oracle provenance 강제와 W7
  canonical registry는 별도 이슈·승인 게이트로 진행한다.
- historical snapshot의 source digest가 달라지면 재생성기가 실패해야 한다. 최신 source로 조용히 다시 찍어
  과거 기준을 덮어쓰지 않는다.
- review·오늘할일 trailing commit 뒤에는 review-only fast-pass의 candidate SHA·single-parent 범위와 최신
  Build & Test aggregate를 다시 확인한다.

## 최종 권고

**보류.** W0·W1 완료 조건, 로컬 결정성·출력 무회귀와 code candidate Full CI·CodeQL은 통과했다.
review-only trailing head의 필수 GitHub Actions와 최신 `MERGEABLE`/`CLEAN`을 확인한 뒤 작업지시자 승인에
따라 병합한다.
