---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6289 self-review — 쪽 배치별 맞춤 배율 계산 단일화

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`,
  `visual_fixture_evidence.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- local code candidate: `2cab279d477dfcbd3ea69886c0fcc087953087b1`

문서 renderer·layout·paint와 HWP/HWPX fixture는 바꾸지 않는다. 다만 실제 쪽 배치와 맞춤 배율의 화면
관계가 제품 계약이므로 `visual_fixture_evidence.md`의 증거 원칙을 적용해 실제 Chrome E2E와 그 HTML
보고서·대표 화면을 사용했다. 보고서와 PNG는 재현 가능한 gitignore 산출물이며 source PR에는 포함하지
않는다.

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6289](https://github.com/edwardkim/rhwp/pull/6289) |
| 작성자 | `postmelee` |
| 관련 이슈 | [#6108](https://github.com/edwardkim/rhwp/issues/6108) |
| stack | 하단 #6289 → 상단 [#6290](https://github.com/edwardkim/rhwp/pull/6290) |
| base / head | `devel` / `codex/issue-6108-zoom-fit` |
| local 규모 | 23 files, +1,004 / -198, 7 commits |
| local base | `upstream/devel@a6c7e7bb3` |
| remote 상태 | Open, non-draft, 기존 remote head는 오늘할일 충돌로 `CONFLICTING/DIRTY` |

기존 remote code candidate의 `Build & Test`, Frontend package gates, CodeQL, Render Diff, Adapter
inter-diff와 Proptest는 성공했다. 이후 최신 `devel@a6c7e7bb3`의 통합 검토 기록과 이 PR의 M100 표가
`mydocs/orders/20260828.md` 끝에 함께 추가돼 GitHub가 충돌로 판정했다. 양쪽 기록을 합집합으로 보존해
rebase했고, 이전·현재 candidate의 range-diff에서 제품 변경은 동일하며 오늘할일 문맥만 추가됐음을 확인했다.
새 local candidate의 merge tree는 충돌 없이 생성됐다. push 뒤 GitHub 상태가 다시 계산되며 metadata와 CI는
변동 값이므로 stack 제출·merge 직전에 다시 확인한다.

## 목적과 변경 범위 정합성

- 자동·한 쪽은 1×1, 두 쪽·맞쪽은 2×1, 여러 쪽은 지정한 `columns×rows`로 해석한다.
- 폭 맞춤은 현재 행 전체 폭, 쪽 맞춤은 전체 가로×세로 블록과 내부 page gap을 사용한다.
- 메뉴·상태 표시줄·대화상자·저장 복원이 한 metrics helper와 resolver를 공유한다.
- 여러 쪽에서는 적용되지 않는 비율 선택을 잠그고 전체 배열 `fitPage` 규칙을 저장한다.
- legacy 쪽 배치 이벤트, 공개 wrapper와 소비되지 않는 `is-neutral` 상태를 제거한다.
- 입력 오류 UI와 원자 view-settings transaction은 상위 PR #6290에 남겨 stack 경계를 유지한다.

## self-review findings와 처리

### 수정 완료 — 여러 쪽 맞춤 규칙 저장 누락

기존 remote head는 여러 쪽 2×2의 수치 배율을 올바르게 `0.315`로 계산했지만 비활성 비율 라디오의
선택값을 따라 `zoomFitMode=none`을 저장했다. 따라서 새 세션이나 다른 쪽 크기에서 2×2 전체 맞춤을 다시
계산하지 못했다.

`resolveZoomDialogFitMode()`로 여러 쪽을 `fitPage`로 고정하고, 한 쪽·두 쪽·맞쪽은 기존 선택값을
보존했다. 단위 테스트와 실제 Chrome에서 저장 직후·새 세션·다른 쪽 크기 재계산을 모두 확인했다.

### 수정 완료 — 죽은 `is-neutral` literal

Stage 2에서 class toggle과 CSS 소비처를 제거했지만 `index.html` literal이 남아 있었다. literal을 제거하고
HTML·런타임 어느 쪽에도 상태가 다시 생기지 않는 회귀 검사를 추가했다.

### 수정 완료 — Stage 3 절차 문서

결과 보고서에만 있던 Stage 3를 `mydocs/working/task_m100_6108_stage3.md`로 분리해 제품 보정·회귀 계약과
검증 수치를 해당 제품 커밋에 함께 기록했다. rebase 뒤 Stage 1·2 commit 참조도 현재 stack SHA로
현행화했다.

### 추가 blocker 없음

- 여러 쪽 외의 수치 배율은 기존처럼 맞춤을 풀어 `none`을 저장한다.
- 저장 복원은 정규화된 `userSettings` 배치와 같은 resolver를 사용한다.
- 10~500% clamp, 프레임 여백, page gap과 배치 topology 계약은 변경하지 않았다.
- E2E source regex는 보조 정적 계약이고, 실제 브라우저의 저장·재시작 경로가 제품 행위를 직접 고정한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| focused unit | 20 pass / 0 fail |
| Studio 전체 `npm test` | 1,226 tests, 1,225 pass / 1 skip / 0 fail |
| TypeScript·Vite production build | 통과, 238 modules |
| Chrome `e2e:zoom-fit-mode` | 32/32 assertion 통과 |
| `git diff --check` | 통과 |

Chrome E2E는 자동·한 쪽·두 쪽·맞쪽·여러 쪽의 폭/쪽 맞춤, 비율 잠금, 문서별·세션별 복원을 검증했다.
특히 여러 쪽 2×2는 `fitPage` 저장 뒤 다른 쪽 크기에서 기대값 `0.446`으로 다시 계산됐다.

Rust source와 integration test source를 바꾸지 않아 Rust unit·clippy·WASM 제품 검증은 직접 변경 범위가
아니다. 다만 저장소 push 필수 gate인 파생 suite 준비 뒤 `cargo fmt --all`과
`cargo fmt --all -- --check`는 stack 제출 직전에 별도로 실행한다.

## 최종 권고

review에서 확인한 제품 정확성 결함, 죽은 literal과 Stage 3 누락을 모두 보정했고 추가 blocker는 발견하지
않았다. self-review는 **완료 / 조건부 merge 권고**다.

하단 #6289와 상단 #6290을 재기반화된 stack으로 제출한 뒤 두 PR의 최신 head·stack 관계와 mergeability를
다시 확인해야 한다. GitHub Actions 완료 확인과 최종 merge는 작업지시자가 별도로 수행하며, 이 작업에서는
CI 완료를 기다리거나 merge하지 않는다.
