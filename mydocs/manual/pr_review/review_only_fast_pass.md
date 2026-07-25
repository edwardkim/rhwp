---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# Review-only fast-pass

이 가이드는 contributor code PR 뒤에 review 기록을 추가하거나, 별도 문서·기준 자료 PR 전체가
review-only인 경우에 적용하는 공용 modifier다. maintainer·collaborator 기본 경로와 함께 읽는다.

[CI workflow](../../../.github/workflows/ci.yml)의 preflight는 다음 허용 범위를 사용한다.

- mydocs 아래 모든 파일 — 파일 상태와 확장자를 제한하지 않는다. 따라서 `mydocs/pr/assets` 등에
  올리는 PDF, HWP/HWPX, PNG 등 검토 증적도 문서-only PR과 같은 허용 범위다.
- added 상태의 samples 아래 hwp, hwpx, pdf, png
- added 상태의 pdf 아래 PDF

기존 samples 또는 pdf 파일의 수정·삭제·rename, source, test, workflow, Cargo.lock, golden, baseline은
허용 범위가 아니다.

## A. code PR 뒤의 trailing review-only commit

contributor code PR의 뒤에 review 문서·오늘할일·허용된 신규 기준 자료를 추가하면 workflow는 현재 head에서
거꾸로 확인해, **현재 base를 포함하고 이후 변경이 모두 review-only인 가장 최근 green PR head**의 결과를
재사용한다. 따라서 직전 green PR head 자체가 review-only commit이어도, 그 commit의 full CI가 성공했고 그 뒤에
허용된 기록만 추가됐다면 candidate가 될 수 있다.

다음 조건을 모두 만족해야 한다.

1. candidate 이후 current head까지의 review-only commit은 single-parent다.
2. review-only Update branch merge를 candidate로 쓰는 경우에는 현재 PR base를 parent로 포함한 정확히
   2-parent merge이고, 그 뒤에 적어도 하나의 single-parent review-only commit이 있어야 한다.
3. candidate SHA는 현재 base를 ancestor로 포함한다. base가 바뀐 뒤 update branch를 하지 않은 옛 run은
   재사용하지 않는다.
4. 후보는 최신순으로 조회한다. check/workflow가 없거나 진행 중인 후보는 더 이전 후보를 계속 확인하되,
   가장 최근 완료 후보가 failed이면 full CI로 fallback한다.
5. 채택한 candidate SHA의 Build & Test check 또는 같은 SHA의 CI workflow 집계 job이 completed이고
   conclusion이 success, skipped, neutral 중 하나여야 한다.
6. push 뒤 최신 head의 preflight와 branch protection이 요구하는 Build & Test aggregate를 확인한다.
   heavy worker가 skipped인 것은 정상이나 aggregate가 pending 또는 failing이면 merge하지 않는다.

local Cargo 성공만으로 candidate의 GitHub Actions를 대체하지 않는다. current base 불일치, 가장 최근 완료
candidate check의 failed, 허용되지 않은 merge 형태, 허용 경로 밖 변경은 full CI fallback이다. 후보가 전부
missing 또는 진행 중이면 green 검증을 찾지 못한 것이므로 역시 full CI를 실행한다.

collaborator가 contributor code를 local에서 검증한 뒤 review·오늘할일만 같은 source head에 추가하는 경우도
이 A 경로다. local 검증 결과와 candidate SHA, 재사용한 Build & Test URL을 review 문서에 기록한다.

## B. PR 전체가 review-only

PR 전체 파일이 허용 범위에만 있으면 preflight는 base SHA를 candidate로 기록하고
all-review-only-no-code-impact fast-pass를 즉시 선택한다. candidate의 과거 Build & Test를 별도로 조회하지
않으며 heavy worker는 skipped된다. 최신 head의 preflight와 최종 Build & Test aggregate가 success인지
확인한다.

따라서 순수 문서·review 기준 자료 PR에 A 경로의 candidate-check 조회 조건을 잘못 적용하지 않는다.

## Full CI fallback

다음 중 하나면 fast-pass로 단정하지 않고 workflow의 full CI 결과를 기다린다.

- code, test, CI workflow, Cargo.lock 변경
- 기존 sample, PDF, golden, baseline, fixture의 수정·삭제·rename
- 허용 목록 밖의 신규 파일
- A 경로의 candidate check 누락·실패·미완료 또는 허용되지 않은 merge 형태
- preflight가 fast_pass=false를 반환

fast-pass는 merge 조건을 없애지 않는다. 최신 head, mergeable 상태, required aggregate, 작업지시자 승인을
확인한다. 완료된 원 PR의 기록만 담는 별도 B 경로 PR은 merge 뒤 issue/PR comment와 오늘할일을 반복하지 않고
devel sync와 branch/worktree/target cleanup만 수행한다.
