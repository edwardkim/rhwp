---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6290 self-review — 사용자 배율 검증과 보기 설정 원자 적용

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`,
  `visual_fixture_evidence.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- local code candidate: `2e150de6e0f88ab914bac11f00dcbf557d1c099d`

대화상자 오류 상태와 쪽 배치·이동·배율 transaction은 실제 사용자 상호작용이 제품 계약이므로
`visual_fixture_evidence.md`의 증거 원칙을 적용해 실제 Chrome E2E와 HTML 보고서·오류 화면을 사용했다.
보고서와 PNG는 재현 가능한 gitignore 산출물이며 source PR에는 포함하지 않는다.

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6290](https://github.com/edwardkim/rhwp/pull/6290) |
| 작성자 | `postmelee` |
| 관련 이슈 | [#6109](https://github.com/edwardkim/rhwp/issues/6109) |
| stack | 하단 [#6289](https://github.com/edwardkim/rhwp/pull/6289) → 상단 #6290 |
| base / head | `codex/issue-6108-zoom-fit` / `codex/issue-6109-zoom-dialog-transaction` |
| local 규모 | 22 files, +1,355 / -56, 4 commits |
| local base | `codex/issue-6108-zoom-fit@1276bb431` |
| remote 상태 | Open, non-draft, 기존 remote candidate CI 성공; 새 local stack은 push 전 |

이 문서의 local candidate는 review 보정과 최신 하단 head 재기반화를 더 포함하므로 push 뒤 GitHub 상태가
다시 계산된다. 작업지시자가 원격 CI를 직접 확인하므로 이 작업에서는 CI 완료를 기다리거나 merge하지 않는다.

## 목적과 변경 범위 정합성

- 빈 값·비정수·10% 미만·500% 초과 배율은 제출 전에 오류로 설명하고 보기 상태를 바꾸지 않는다.
- 오류 문구는 `role="alert"`, `aria-invalid`, 입력 focus/select 복원과 밝은·어두운 테마의
  `--ui-danger-strong` 의미 토큰을 공유한다.
- 쪽 배치·쪽 이동·맞춤 모드·배율·anchor를 하나의 `page-view-settings-changed` transaction으로 전달한다.
- CanvasView는 최종 snapshot이 반영된 뒤 `recalcLayout()`을 한 번만 실행한다.
- 하단 #6289의 `resolveZoomDialogFitMode()`를 그대로 사용해 여러 쪽 `fitPage` 저장 규칙을 보존한다.
- slider·pinch 성능, render scale, 페이지 LRU와 반응형 눈금자는 별도 이슈 범위에 남긴다.

## self-review findings와 처리

### 수정 완료 — 하단 여러 쪽 맞춤 보정 연결

rebase 충돌을 해결할 때 상단의 원자 transaction과 하단의 `resolveZoomDialogFitMode()`를 함께 유지했다.
따라서 여러 쪽은 계산된 배율뿐 아니라 `fitPage` 의미도 한 snapshot에 저장되며, 상단 E2E와 하단
`e2e:zoom-fit-mode` 32/32를 같은 최상단 head에서 재검증했다.

### 수정 완료 — Stage 3 문서 계보

누락됐던 `mydocs/working/task_m100_6109_stage3.md`를 원래 Stage 3 통합검증 커밋에 autosquash했다.
최신 `devel` 충돌 해소 뒤 상단을 하단 review head `1276bb431` 위로 다시 재기반화했다. range-diff와
`rhwp-studio` tree 비교에서 상단 제품 변경은 이전 candidate와 동일함을 확인했다.

### 설명 보완 — E2E MANIFEST 세 행

신규 `zoom-dialog-transaction.test.mjs`를 원장에 추가하면 양방향 완전성 검사가 이미 tracked됐지만 누락된
`issue-4969-shaping-replay.test.mjs`, `issue-6117-cell-underline-canvas2d.test.mjs`도 함께 탐지한다. 두 기존
스크립트의 실제 용도·fixture·배선을 확인하고 누락 행만 복원해 tracked 120 / manifest 120을 맞췄다.
제품 범위를 확장한 것이 아니라 기존 원장 drift를 신규 검사의 필수 gate 안에서 정합화한 변경이다.

### 추가 blocker 없음

- `input[type=number]`는 실제 브라우저에서 비숫자 문자열을 빈 문자열로 정규화하므로 UI E2E의 비숫자 전용
  문구가 직접 도달하지 않는 점은 결함이 아니다. 순수 validator 테스트는 해당 방어 경로를 유지한다.
- source regex 테스트는 배선의 보조 정적 계약이다. 실제 오류 ARIA·focus·테마와 event/recalc 횟수는 Chrome
  E2E가 사용자 행위로 검증한다.
- 개별 설정 setter와 zoom 없는 legacy payload는 호환 경로로 유지된다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Studio 전체 `npm test` | 1,235 tests, 1,234 pass / 1 skip / 0 fail |
| TypeScript·Vite production build | 통과, 239 modules |
| Chrome `e2e:zoom-dialog-transaction` | 37/37 assertion 통과 |
| Chrome `e2e:zoom-fit-mode` | 하단 회귀 32/32 assertion 통과 |
| E2E 원장 | tracked 120 / manifest 120 |
| `git diff --check` | 통과 |

Chrome E2E는 오류 입력의 무변경 종료, ARIA·focus·테마 토큰, 유효값 교정, Enter/Escape/취소, 10%·500%
경계와 두 쪽+이동+휠 좌우+137% transaction을 검증했다. 마지막 시나리오에서 page-view event,
`zoom-changed`, `recalcLayout()`은 각각 한 번이며 recalc 순간에는 최종 상태가 모두 반영돼 있다.

Rust source와 integration test source를 바꾸지 않았다. 저장소 push 필수 gate인 파생 suite 준비와
`cargo fmt --all`, `cargo fmt --all -- --check`는 stack 제출 직전에 최종 실행하고 파생 산출물은 source PR에
포함하지 않는다.

## 최종 권고

review에서 확인한 하단 맞춤 연결, Stage 3 계보와 MANIFEST 설명을 모두 보정했고 추가 blocker는 발견하지
않았다. self-review는 **완료 / 조건부 merge 권고**다.

하단 #6289와 상단 #6290을 native stack으로 갱신하고 최신 head·base 관계를 확인해야 한다. GitHub Actions
완료 확인과 최종 merge는 작업지시자가 별도로 수행하며, 이 작업에서는 CI 완료를 기다리거나 merge하지 않는다.
