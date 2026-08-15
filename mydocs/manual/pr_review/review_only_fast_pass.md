---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# Review-only fast-pass

이 가이드는 contributor code PR 뒤에 review 기록을 추가하거나, 별도 문서·기준 자료 PR 전체가
review-only인 경우에 적용하는 공용 modifier다. maintainer·collaborator 기본 경로와 함께 읽는다.

[CI workflow](../../../.github/workflows/ci.yml)의 preflight는 다음 허용 범위를 사용한다.

- mydocs 아래 모든 파일 — 파일 상태와 확장자를 제한하지 않는다. 따라서 `mydocs/pr/assets` 등에
  올리는 PDF, HWP/HWPX, PNG 등 검토 증적도 문서-only PR과 같은 허용 범위다.
- added 상태의 samples 아래 hwp, hwpx, pdf, png
- added 상태의 pdf, pdf-2020, pdf-large 아래 PDF

기존 samples 또는 세 PDF 디렉터리 파일의 수정·삭제·rename, source, test, workflow, Cargo.lock,
golden, baseline은 허용 범위가 아니다.

## A. code PR 뒤의 trailing review-only commit

contributor code PR의 뒤에 review 문서·오늘할일·허용된 신규 기준 자료를 추가하면 workflow는 현재 head에서
거꾸로 확인해, **같은 PR source branch에서 실행된 녹색 code candidate SHA와 이후 변경이 모두 review-only인
가장 최근 head**의 결과를 재사용한다. base가 전진했더라도 source·test가 바뀌지 않았다면 문서 기록만을 위해
Update branch, merge, rebase를 수행하지 않는다. 따라서 직전 green PR head 자체가 review-only commit이어도,
그 commit의 full CI가 성공했고 그 뒤에 허용된 기록만 추가됐다면 candidate가 될 수 있다.

다음 조건을 모두 만족해야 한다.

1. candidate 이후 current head까지의 review-only commit은 single-parent다. 단, current base 병합 bridge는
   한 번만 예외로 허용한다.
2. current base 병합 bridge는 정확히 2-parent이고 parent 하나만 현재 PR base SHA와 같아야 한다. preflight가
   `git merge-tree`로 계산한 자동 3-way merge tree가 실제 merge commit tree와 같으면 그대로 재사용한다.
   자동 병합이 충돌한 경우에는 `git show --remerge-diff`가 보고하는 **수동 충돌 해소 경로 전체가 `mydocs/`
   아래일 때만** 재사용한다. source, test, workflow, sample, PDF 등 하나라도 포함되거나 경로를 확인할 수 없으면
   full CI로 fallback한다. 이 확인은 current base에 있는 검사기를 사용하며 PR source를 실행하지 않는다.
3. candidate SHA는 현재 PR commit history의 code 후보여야 하며, CI·CodeQL·Render Diff 결과는 같은 PR의
   head branch, source repository, event, candidate SHA와 정확히 일치해야 한다. 현재 base 전진은 단독으로
   재사용 거부 사유가 아니다.
4. 후보는 최신순으로 조회한다. workflow가 없거나 진행 중인 후보는 더 이전 후보를 계속 확인하되,
   가장 최근 완료 후보가 failed이면 full CI로 fallback한다.
5. 채택한 candidate SHA의 Build & Test check 또는 같은 SHA의 CI workflow 집계 job이 completed이고
   conclusion이 success, skipped, neutral 중 하나여야 한다.
6. push 뒤 최신 head의 preflight와 branch protection이 요구하는 Build & Test aggregate를 확인한다.
   heavy worker가 skipped인 것은 정상이나 aggregate가 pending 또는 failing이면 merge하지 않는다.

trailing range에 reviewer가 만든 일반 merge commit이 있으면 fast-pass는 이를 재사용하지 않는다. current base
병합만 위 조건을 만족하는 한 번의 bridge로 예외 허용한다. 따라서 마지막 commit이 오늘할일 충돌을 `mydocs/`
안에서만 해소한 current-base merge여도 같은 PR source의 녹색 candidate를 재사용할 수 있다. 이 호환 경로는 문서
기록을 위해 source에 `devel`을 병합하라는 절차가 아니다. 일반 절차는 여전히 Update branch, merge, rebase 없이
review-only commit만 code candidate 위에 잇는다. contributor가 review 도중 source를 갱신한 경우에는 새 source를
기존 reviewer 기록에 merge하지 말고 [2.6.1 외부 PR review 기록의 source head 정렬](multi_pr_update_branch.md#261-외부-pr-review-기록의-source-head-정렬)을
적용해 reviewer 기록만 새 source 위로 replay한다.

local Cargo 성공만으로 candidate의 GitHub Actions를 대체하지 않는다. candidate workflow의 PR identity 불일치,
가장 최근 완료 candidate check의 failed, 허용되지 않은 merge 형태, 허용 경로 밖 변경은 full CI fallback이다.
후보가 전부 missing 또는 진행 중이면 green 검증을 찾지 못한 것이므로 역시 full CI를 실행한다.

collaborator가 contributor code를 local에서 검증한 뒤 review·오늘할일만 같은 source head에 추가하는 경우도
이 A 경로다. local 검증 결과와 candidate SHA, 재사용한 Build & Test URL을 review 문서에 기록한다.

### A.1 CI 실행 정책을 바꾼 PR의 trusted 재사용

PR 전체 변경에 `.github/workflows/**`, `.github/actions/**`, CI impact classifier·policy 또는
review-only merge 검사기가 포함되면 PR head의 preflight만으로 A 경로를 허용하지 않는다. 해당 PR이 자신이
바꾼 workflow로 검증을 생략할 수 있기 때문이다. 기본값은 `ci-execution-surface-change` Full 실행이다.

예외는 기본 브랜치에 등록된 `CI Impact Policy Controller`가 current head에 `CI Impact Policy` status를
발행한 same-repository PR뿐이다. controller는 PR head나 artifact를 실행하지 않고 live base의 정책 코드로
다음을 독립 확인한다.

1. exact code candidate의 CI·CodeQL과 변경 범위상 필요한 Render Diff가 같은 repository·branch·PR·SHA에서
   **fast-pass가 아닌 Full 실행**으로 성공했다. CodeQL은 언어별 Analyze job과 같은 run 이후 GHAS CodeQL
   check까지 성공 또는 neutral이어야 한다.
2. candidate 뒤 current head까지는 허용된 single-parent review-only commit뿐이다. current-base bridge는
   trusted controller가 commit object만 fetch해 자동 merge tree 일치 또는 `mydocs/` 한정 충돌 해소를 다시
   검증한 경우에만 허용한다.
3. status는 exact current head에 묶이고 policy version, `rfp=1`, current base SHA, trusted controller
   `pull_request_target` run identity가 모두 일치한다.

CI·CodeQL·Render Diff preflight는 이 status를 짧게 기다린 뒤에만 기존 candidate 탐색을 계속한다. status가
누락·pending·failed·stale이거나 candidate 실행·GHAS check·commit/file 목록·merge tree 중 하나라도 불완전하면
재사용하지 않고 Full 실행한다. 외부 fork의 실행 정책 변경은 이 예외 대상이 아니다.

`pull_request_target`과 `workflow_run`은 기본 브랜치에 있는 workflow만 등록한다. 따라서 controller 변경을
`devel`에 병합한 것만으로 이 예외가 활성화되지 않으며, 정상 release 절차로 `main`에 반영된 뒤부터 적용한다.

## B. PR 전체가 review-only

PR 전체 파일이 허용 범위에만 있으면 preflight는 base SHA를 candidate로 기록하고
all-review-only-no-code-impact fast-pass를 즉시 선택한다. candidate의 과거 Build & Test를 별도로 조회하지
않으며 heavy worker는 skipped된다. 최신 head의 preflight와 최종 Build & Test aggregate가 success인지
확인한다.

따라서 순수 문서·review 기준 자료 PR에 A 경로의 candidate-check 조회 조건을 잘못 적용하지 않는다.

## Full CI fallback

다음 중 하나면 fast-pass로 단정하지 않고 workflow의 full CI 결과를 기다린다.

- code, test, CI workflow, Cargo.lock 변경. 단, CI workflow 변경 뒤 review-only 기록만 추가된 경우에는
  A.1의 trusted controller 증명이 전부 성립할 때만 예외로 한다.
- 기존 sample, PDF, golden, baseline, fixture의 수정·삭제·rename
- 허용 목록 밖의 신규 파일
- A 경로의 candidate workflow 누락·실패·미완료·PR identity 불일치, current-base merge tree 불일치,
  `mydocs/` 밖 충돌 해소·해소 경로 조회 실패·복수 base merge 또는 허용되지 않은 merge 형태
- preflight가 fast_pass=false를 반환

fast-pass는 merge 조건을 없애지 않는다. 최신 head, mergeable 상태, required aggregate, 메인테이너 승인을
확인한다. 완료된 원 PR의 기록만 담는 별도 B 경로 PR은 merge 뒤 issue/PR comment와 오늘할일을 반복하지 않고
devel sync와 branch/worktree/target cleanup만 수행한다.
