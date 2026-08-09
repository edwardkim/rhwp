---
kind: pr_review
status: maintainer-review
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4266 검토 — #4150 IME 조합 오버레이 stale 좌표 + HF/FN 재정박 offset

## 결론

**수용 권고.** 커밋 1(`e6a20f92d`)은
IME 조합 중 같은 페이지 안 reflow가 일어나면 조합 오버레이가 옛 좌표에 그려져 실제 캔버스 텍스트와
겹치던 결함(#4150)을 `compositionAnchorRect` 캐시 제거로 고치고, 조합 replace가 wasm deferred replace
범위 가드에 거부될 때 `onInput` 밖으로 예외가 던져져 조합 추적이 wedge되던 경로를 try/catch로 방어한다.
커밋 2(`c03ee47a0`)는 이 리뷰 과정에서 발견한 회귀 — 위 catch 재정박이 머리말/꼬리말·각주 모드에서
`cursor.getPosition()`의 stale 본문 offset을 그대로 썼던 것 — 을 `onCompositionStart`와 동일한
`hfCharOffset`/`fnCharOffset` override로 고쳤다.

2026-08-09 메인터너 재검토에서 최신 `upstream/devel`과의 merge simulation, TypeScript·Studio 전체 test,
그리고 고정 `target/pr-review` 전체 Rust 회귀를 다시 확인했다. contributor 코드 head의 CI는 모두 성공했고,
후속 메인터너 보정은 review target 재사용과 장시간 baseline 우선 실행을 위한 nextest config·검토 문서뿐이다.

## 검토 경로

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_external_pr.md, intake_and_review.md,
                  local_validation.md
devel base: e919655a7 (upstream/devel HEAD at PR 생성 시점) → 59b31e5ce (rebase 뒤 최신)
validated code head: c03ee47a0142154c737c3a7c7a6da857fb4de764 (rebase 전) → a68667af4/241d2f91e/0d1e95ea0 (rebase 후, 아래 참고)
```

메인터너 재검토 기준: `upstream/devel` `c391dbbdc`, contributor code head
`f2a95a6693212b19b9ac210b4424a356daf4b6a9`, merge simulation
`e771871c0`. simulation은 local 검토 전용이며 contributor source branch에 push하지 않는다.

원 후보는 `integration/all-works`(로컬, #4180/#4179 등 다른 이슈 커밋과 커밋되지 않은 무관한 변경이
섞인 통합 branch)에서 나왔다. 그 branch를 그대로 push하면 base가 크게 뒤처지고(분기 이후 devel
185커밋 진행) 무관한 이슈가 같은 PR에 섞이므로, `e6a20f92d`/`c03ee47a0` 두 커밋만 최신
`upstream/devel` 위 별도 worktree에 cherry-pick해 이 PR로 분리했다. cherry-pick은 두 커밋 모두
conflict 없이 적용됐다(`input-handler.ts` auto-merge).

별도 `review_impl` 문서는 만들지 않았다. 단일 이슈의 2-커밋 fix이고 실행 순서·rollback 경계가 이 문서
하나로 충분히 명확하다.

## rebase — #4245와의 conflict 해소

PR 생성 뒤 `mergeable: CONFLICTING`이 됐다. 원인은 같은 날 별도 세션이 처리한
[이슈 #4245](https://github.com/edwardkim/rhwp/issues/4245)(`studio: IME 조합 replace가 wasm
범위 가드에 거부되면 uncaught로 조합 추적이 wedge된다`, "관련: #4150 조합 계열"로 명시)가
[PR #4265](https://github.com/edwardkim/rhwp/pull/4265) 통합에 실려 이미 `upstream/devel`에
merge된 것이었다. `#4245`의 커밋(`abb5a1485`)은 `onInput`의 같은 catch 블록을 고치지만
**이 PR에서 발견해 고친 HF/FN `charOffset` override가 없는 이전 버전**이었고, `compositionAnchorRect`
캐시 제거(#4150의 핵심 시각 결함 수정)는 포함하지 않았다.

`git rebase upstream/devel`을 실행한 결과 수동 conflict 마커 없이 자동 3-way merge로 정리됐다 —
devel의 `#4245` 커밋과 이 PR의 1차 커밋이 텍스트상 동일한 초기 patch였기 때문에, git이 이를 이미
적용된 변경으로 인식하고 2차 커밋(HF/FN override)만 그 위에 얹었다. rebase 뒤
`input-handler-text.ts`의 catch 블록을 직접 읽어 HF/FN override가 살아있는지 확인했다(아래
로컬 검증 재실행 결과도 새 head 기준으로 갱신). 이 PR은 `#4245`와 중복이 아니라 `#4245`가
놓친 시각 결함(#4150 본 증상)과 `#4245`가 devel에 남긴 HF/FN offset 버그를 함께 고치는
상위 집합이다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4266](https://github.com/edwardkim/rhwp/pull/4266) |
| 관련 이슈 | #4150 |
| 작성자 | `humdrum00001010` (fork 기반 — `upstream` 직접 push 권한 없음, 403 확인 후 `origin` fork로 push) |
| reviewer | 최근 동일 작성자 PR(#4259–#4262)과 동일하게 `jangster77` 지정 시도 — 권한 부족으로 `gh pr edit --add-reviewer` 실패(`RequestReviewsByLogin` 권한 없음). 작업지시자가 수동으로 지정 필요 |
| 대상 / head | `devel` / `humdrum00001010:task_m100_4150` (fork branch) |
| 생성 상태 | open, non-draft |
| 생성 시점 규모 | 5 files, +253 / -87, 2 commits |
| 생성 시점 merge 상태 | mergeable, `BLOCKED` — CI 진행 중 참고값 |

위 head·규모·merge 상태는 PR 생성 직후 참고값이다. merge 직전에 다시 확인한다.

## 렌더 영향 판정

시각·fixture 증적 보조 경로는 적용하지 않는다. 조합 블랙박스 오버레이(`caret-renderer.ts`의 `compEl`)는
canvas paint가 아니라 절대좌표 DOM 엘리먼트 위치 계산이고, 이 PR은 `src/renderer`, `wasm_api.rs`,
golden/fixture, HWP/HWPX sample을 전혀 건드리지 않는다.

## 로컬 검증

최초 cherry-pick code head(`c03ee47a0`)를 검증한 뒤, `upstream/devel` rebase(위 절)로 head가
`0d1e95ea0`(code 변경은 `a68667af4`/`241d2f91e`)로 바뀌어 새 head 기준으로 전부 재실행했다.
Rust/wasm 변경이 없는 rhwp-studio 전용 PR이라 [4.3 표](../../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
"rhwp-studio만 변경" 행을 따랐다.

| 검증 | rebase 전 (`c03ee47a0`) | rebase 후 (`0d1e95ea0`) |
| --- | --- | --- |
| `npx tsc --noEmit` | PASS | PASS |
| `node --test tests/*.test.ts ../npm/editor/tests/*.test.mjs` | 805/805 pass | 814/814 pass (devel에 병합된 다른 PR 테스트 포함) |
| `git diff --check` (devel..HEAD) | PASS | PASS |
| 새 행위 테스트 회귀 확인 | `composition-hf-fn-reanchor.runner.mjs`를 수정 전 소스로 되돌려 재실행 → HF 케이스에서 의도대로 fail(`expected 0, actual 1`), 원복 후 재실행 → pass. `git diff`로 원복 무결성 확인 | rebase 후 파일을 직접 읽어 HF/FN override가 살아있음을 확인(위 절) — 소스 되돌리기 재실행은 rebase 전 결과를 재사용 |

wasm 타입 선언은 이 worktree에 fresh `wasm-pack build`를 새로 돌리지 않고, 이 PR과 무관한 Rust 소스로
만든 기존 `pkg/`(원 checkout, 오늘자 빌드)를 복사해 재사용했다 — 이 PR이 `.rs` 파일을 전혀 바꾸지 않아
API 표면이 같으므로 `tsc` 타입체크 목적에는 안전하다고 판단했다. 실제 IME 조합(OS 후보창)을 통한
수동 재현은 자동화 범위 밖이라 생략했고, 대신 `:7701` 실행 중인 dev 서버에서 synthetic
compositionstart/update/end 이벤트로 골든 패스 무오류를 확인했다(이전 리뷰 기록).

## 발견한 문제

없음. `e6a20f92d`(1차 커밋) 리뷰에서 지적한 HF/FN 재정박 offset 결함은 `c03ee47a0`(2차 커밋)이
`onCompositionStart`와 동일 규약으로 정확히 고쳤고, 그 수정을 검증하는 행위 테스트가 수정 전 상태에서
실패함을 직접 확인했다.

## GitHub Actions와 남은 게이트

- PR 생성 직후 `CI preflight`/`CodeQL preflight`/`Render Diff preflight`는 성공, `Frontend package gates`
  등 나머지는 진행 중이었다(참고값, 재확인 필요).
- `mergeStateStatus: BLOCKED`는 CI 진행 중 참고값이며 최신 head의 required check 전체 성공을 merge
  직전에 다시 확인해야 한다.
- reviewer 지정은 권한 부족으로 실패했다 — 작업지시자가 GitHub에서 직접 `jangster77`(또는 다른
  reviewer)을 지정해야 한다.

### 2026-08-09 메인터너 재검토 결과

| 검증 | 결과 |
| --- | --- |
| 최신 `upstream/devel` merge simulation | clean, `git diff --check` 통과 |
| `rhwp-studio` TypeScript | `npx tsc --noEmit` PASS (2.682초) |
| Studio 전체 test | `node --test tests/*.test.ts ../npm/editor/tests/*.test.mjs` 814/814 PASS (5.999초) |
| Linux 고정 review target | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 5,470/5,470 PASS, 35 skipped, 7분 56초 |
| Windows 고정 review target | cmd에서 `overflow_cell_baseline` cold 18분 55초, warm 6분 11초, 모두 PASS |

원 code head `f2a95a669`의 CI preflight, Frontend package gates, CodeQL(JavaScript/TypeScript·Python·Rust),
Canvas visual diff와 Build & Test aggregate는 성공했다. Rust-heavy job은 studio 전용 변경이라 preflight가
의도적으로 skip했다. nextest config 보정이 추가된 새 head에서는 해당 config 변경을 포함한 최신 CI를 다시
확인한다.

## 최종 권고

contributor의 IME 재정박 수정은 수용한다. 메인터너 nextest config·검토 기록을 source branch에 trailing
commit으로 반영한 뒤, 그 최신 head의 required CI 성공과 reviewer 지정·승인을 확인해 merge를 권고한다.
