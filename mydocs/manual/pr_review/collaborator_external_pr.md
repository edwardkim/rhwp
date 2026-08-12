---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# Collaborator 매개 외부 PR 처리

이 경로는 repository collaborator가 외부 contributor PR을 검토하고, 필요하면 PR head에 review 기록이나
보정 commit을 더해 merge를 준비할 때 적용한다. maintainer 일반 경로를 대체하지 않는다.

## 9.1 적용 조건

- PR 작성자는 외부 contributor다.
- collaborator가 review, 문서화, merge 준비를 담당한다.
- maintainer_can_modify가 true이거나 contributor가 collaborator의 source branch push를 허용했다.
- review 문서만을 위한 별도 PR보다 현재 PR head에 운영 기록을 넣는 편이 단순하다.
- GitHub review/comment, 실제 remote push, ready 전환, merge는 각각 작업지시자 승인 뒤에 수행한다.

maintainer_can_modify가 false이면 이 경로를 쓰지 않는다. maintainer 일반 경로 또는 작업지시자가 승인한
별도 PR 경로로 전환한다.

### 9.1.1 기본 작업공간: `devel` 기반 체리픽 통합 검토

사용자가 VS Code·터미널에서 commit 그래프와 작업 상태를 볼 수 있어야 하는 외부 PR 검토의 기본 경로는,
사용자가 열어 둔 **주 작업공간**에서 최신 `upstream/devel` 위에 review branch를 만들고 contributor의
기능 commit을 순서대로 cherry-pick하는 통합 경로다. 기본 작업공간이 clean하면 검토만을 위해 별도
`git worktree add`를 만들지 않는다. 별도 worktree는 주 작업공간이 dirty여서 격리가 꼭 필요하거나
작업지시자가 명시적으로 요구한 경우에만 쓴다.

~~~bash
git status --short --branch
git fetch upstream devel pull/N/head:refs/remotes/upstream/prN-head
git switch devel
git merge --ff-only upstream/devel
git switch -c review/<contributor>-<yyyymmdd> upstream/devel
git log --reverse --no-merges --format='%H %s' \
  upstream/devel..upstream/prN-head
# 위 목록을 오래된 commit부터 하나씩 적용하고, 각 충돌을 이 branch에서 해결한다.
git cherry-pick <contributor-commit-sha>
git diff --check upstream/devel...HEAD
~~+

- 적용한 원 PR 번호·SHA·cherry-pick 순서와 conflict 보정은 review 문서에 남긴다. 현재 branch는
  `devel` 위의 contributor 변경과 메인터너 보정을 한 그래프로 보여야 한다.
- 통합 branch의 code·test 보정은 contributor source를 rewrite하지 않는다. 완료하면 원본 저장소의 임시
  head branch로 push해 `devel` 대상 통합 PR을 만들고, 그 PR이 merge된 뒤 원 PR은 merge된 통합 PR을
  링크한 comment와 함께 close한다.
- PR head가 최신 `devel`과 이미 같은 history를 공유하더라도, integration branch의 기준은 항상
  `upstream/devel`이다. merge commit은 cherry-pick하지 않고 기능·test·문서 commit만 적용한다.
- 주 작업공간이 dirty이면 checkout·cherry-pick을 시작하지 않는다. 사용자 변경의 소유와 상태를 먼저
  확인하고, 작업지시자가 격리를 승인한 경우에만 정확한 worktree 경로를 사용한다.

원 contributor PR head를 그대로 유지하면서 그 source branch에 collaborator commit을 직접 push해야 하는
경우는 아래 9.3.1의 예외 경로다. 이 경우에도 사용자 가시성이 필요하면 별도 worktree 대신 같은 주
작업공간에서 source head를 checkout한 `review/<contributor>-<yyyymmdd>` branch를 사용한다. direct source
경로와 위 체리픽 통합 경로를 한 PR에 섞거나, contributor history를 rebase·amend·force-push하지 않는다.

## 9.2 문서 경로

현재 contributor PR head에 다음을 포함할 수 있다.

~~~text
mydocs/pr/archives/pr_N_review.md
mydocs/pr/archives/pr_N_review_impl.md     # 필요 시
mydocs/pr/archives/pr_N_report.md          # 필요 시, 사전 판단 보고서
mydocs/orders/YYYYMMDD.md                  # 갱신이 필요한 경우
~~~

### 9.2.1 오늘할일 생성·갱신 시점

오늘할일은 최초 조사나 local 검증 중에는 만들지 않는다. contributor PR에 넣을 최종 review 묶음을 작성할 때
같은 commit으로 생성·갱신한다. report를 쓰면 merge SHA, 실제 merge 시각, issue close 완료를 미리 단정하지
않고 수용·보류 판단, merge 전 조건, merge 뒤 확인 항목만 적는다.

오늘할일이 필요하면 merge 후 별도 PR로 늦게 만들지 않는다. review 문서와 함께 contributor PR head에
포함하고, 뒤에 추가한 review-only commit은 9.3.2의 fast-pass 조건으로 판정한다.

local CI 검증이 완료된 경우 review 문서와 오늘할일은 실행 결과를 과거형으로 기록한다. 아직 실행하지
않은 GitHub Actions·작업지시자 승인·merge만 미래 조건으로 남기며, 완료 검증을 "실행할 예정"으로
표현하지 않는다.

source branch의 오늘할일이 최신 `upstream/devel`보다 오래된 경우에는
[최신 `devel` 오늘할일을 보존하는 trailing 기록](../pr_review_workflow.md#321-최신-devel-오늘할일을-보존하는-trailing-기록)을
따른다. 최신 내용을 source에 복사하거나 `devel`을 병합하지 않고, merge tree에서 기존 기록과 새 기록이
함께 보존되는지 검증한다.

## 9.3 PR head push

contributor 원 commit을 rewrite하지 않는다. review 문서·오늘할일·보정 code는 별도 commit으로 나누고,
보정이 있으면 review 문서에 contributor 원 변경과 collaborator 추가 변경을 구분한다.

### 9.3.0 LFS 대상 사전 판독

review-only 문서 push를 포함한 **모든** contributor source branch push는 dry-run이나 실제 push 전에,
contributor 원 head와 local HEAD 사이 변경 파일이 LFS 추적 대상인지 먼저 판독한다. LFS 대상이 아닌
Markdown-only commit에도 Git LFS pre-push hook은 lock 검증을 시도할 수 있으므로, 실패한 일반 push를
근거로 뒤늦게 판단하지 않는다.

~~~bash
review_source_sha=<push 직전 contributor head SHA>
git diff --name-only -z "$review_source_sha" HEAD |
  while IFS= read -r -d '' review_path; do
    git check-attr filter -- "$review_path"
  done
git lfs status
~~~

- 출력에 `filter: lfs`가 하나라도 있거나 `git lfs status`가 새 object push를 보이면 LFS 대상이다.
  정상 LFS pre-push hook을 포함한 dry-run과 실제 push를 사용하고, object·lock 권한 문제를 먼저
  해결한다.
- 둘 다 없으면 LFS object는 이번 ref update와 무관하다. 처음부터 `GIT_LFS_SKIP_PUSH=1`을 붙인
  dry-run과 실제 push를 사용해 LFS lock 검증만 건너뛴다. 다른 pre-push hook을 무력화하거나
  `core.hooksPath`를 바꾸지 않는다.

원격 SHA가 source SHA와 다른 경우에는 이 판독을 시작하지 않는다. 새 contributor commit을 fetch해
source SHA를 다시 고정한 뒤 재판독한다.

~~~bash
git fetch upstream pull/N/head:local/prN
git switch local/prN
# local 검증과 archive review·오늘할일 작성
git commit -m "docs: PR #N 검토 기록"
# 9.3.0의 LFS 판독 결과에 따라 둘 중 하나를 선택
git push https://github.com/<contributor>/rhwp.git HEAD:<head-branch>
GIT_LFS_SKIP_PUSH=1 git push https://github.com/<contributor>/rhwp.git HEAD:<head-branch>
~~~

push 뒤 PR head SHA가 local HEAD와 같은지 확인하고, 9.3.2 fast-pass 또는 최신 head full CI 결과가
merge 가능한 상태인지 확인한다.

### 9.3.1 contributor PR head 직접 보정

차단 결함을 collaborator가 고치기로 하면 별도 통합 branch로 옮기지 않고 maintainer_can_modify가 true인
**현재 contributor PR head 위에만** 추가 commit을 만든다.

#### 9.3.1.1 source head 고정과 가시성 branch 연속 사용

push 직전 PR head SHA, `git ls-remote` SHA, 가시성 branch의 시작 source SHA가 모두 같아야 한다. 처음
가시성 branch를 만든 뒤에는 검토·보정·review 문서 commit을 **같은 local branch**에서 연속해 만든다.
보정만을 위해 `review/prN-maintainer` 같은 두 번째 local branch를 새로 만들거나 checkout하지 않는다. 이
규칙은 사용자가 VS Code와 터미널에서 contributor 원 변경부터 메인터너 보정까지 하나의 그래프로 확인할 수
있게 한다.

~~~bash
gh pr view N --repo edwardkim/rhwp \
  --json headRefName,headRefOid,headRepository,maintainerCanModify
git ls-remote --heads https://github.com/<contributor>/rhwp.git refs/heads/<head-branch>
git fetch https://github.com/<contributor>/rhwp.git \
  refs/heads/<head-branch>:refs/remotes/contributor/prN-head
git switch review/<contributor>-<yyyymmdd>
git rev-parse HEAD
~~~

가시성 branch의 첫 source commit SHA는 fetch한 PR head와 같아야 하며, 이후 collaborator commit은 그 위에만
추가한다. 세 SHA가 다르면 보정을 시작하지 않는다. contributor가 새 commit을 push했으면 새 head를 fetch해
같은 가시성 branch의 기준을 다시 확인한다. 이때 기존 reviewer 기록 위로 새 source를 일반 merge하지 않는다.
아직 원격에 push하지 않은 collaborator commit만 새 source 위로 replay할 수 있으며, 정확한 선행조건과 명령은
[2.6.1 외부 PR review 기록의 source head 정렬](multi_pr_update_branch.md#261-외부-pr-review-기록의-source-head-정렬)을
따른다. contributor commit은 rebase, amend, reset, force-push하지 않는다.

#### 9.3.1.2 commit 분리와 LFS dry-run

- code·regression test 보정과 review·오늘할일은 별도 commit으로 만든다.
- dry-run 전에 [9.3.0의 LFS 대상 사전 판독](#930-lfs-대상-사전-판독)을 완료한다. 이 판독은
  code 보정뿐 아니라 review-only 문서 push에도 동일하게 적용한다.
- LFS 대상이면 정상 pre-push hook을 포함한 dry-run을 실행한다.

~~~bash
git push --dry-run https://github.com/<contributor>/rhwp.git HEAD:<head-branch>
~~~

LFS 대상이 전혀 없다고 사전 판독됐으면 첫 dry-run부터 다음 명령으로 Git ref write를 분리 확인한다.
core.hooksPath를 무력화해 다른 pre-push hook 전체를 건너뛰지 않는다.

~~~bash
GIT_LFS_SKIP_PUSH=1 \
  git push --dry-run https://github.com/<contributor>/rhwp.git HEAD:<head-branch>
~~~

#### 9.3.1.3 승인 뒤 실제 push와 CI

작업지시자가 push를 승인했고 마지막 remote SHA가 보정 시작 SHA와 같을 때만 collaborator 추가 commit을
contributor source branch에 push한다. [9.3.0의 사전 판독](#930-lfs-대상-사전-판독)에서 LFS 대상이
있었으면 정상 push를, 없었으면 처음부터 `GIT_LFS_SKIP_PUSH=1` push를 사용한다.

~~~bash
GIT_LFS_SKIP_PUSH=1 \
  git push https://github.com/<contributor>/rhwp.git HEAD:<head-branch>
git ls-remote --heads https://github.com/<contributor>/rhwp.git refs/heads/<head-branch>
gh pr view N --repo edwardkim/rhwp --json headRefOid
~~~

remote ref와 PR headRefOid가 local HEAD와 같은지 확인한다. code 또는 test commit이 하나라도 포함되면
review-only fast-pass를 적용하지 않고 최신 head full CI를 기다린다.

#### 9.3.1.4 최신 `devel` 호환 보정

`MERGEABLE`은 Git의 텍스트 충돌이 없다는 참고값이다. 최신 `upstream/devel`에 PR source가 바꾼 공용
struct·trait·API의 새 사용처가 추가된 경우, GitHub가 계산한 current-base merge tree는 충돌 없이 만들어져도
workspace 컴파일 또는 테스트에서 실패할 수 있다. 예를 들어 source가 struct 필드를 추가한 뒤 최신 `devel`이
그 struct literal을 새로 만들면, merge tree에서 새 필드 초기화 누락 오류가 난다.

이 경로는 다음을 모두 만족할 때만 사용한다.

1. 최신 PR head의 GitHub CI 또는 local `git merge-tree` 기반 재현에서 current-base 호환 오류를 확인했다.
2. source SHA, contributor remote ref, PR `headRefOid`, 정확한 최신 `upstream/devel` SHA를 기록하고 다시
   일치 여부를 확인했다.
3. `maintainerCanModify=true`이거나 contributor가 source branch push를 명시적으로 허용했다.
4. 문서 기록만 추가하는 경우가 아니다. review·오늘할일만 필요하면 9.3.2와
   [통합 워크플로우 3.2.1절](../pr_review_workflow.md#321-최신-devel-오늘할일을-보존하는-trailing-기록)을
   따른다.

먼저 clean한 같은 visibility review branch에서 호환 필요성을 고정한다.

~~~bash
git fetch upstream devel
source_sha=$(git rev-parse HEAD)
base_sha=$(git rev-parse upstream/devel)
git status --short --branch
git merge-tree --write-tree upstream/devel HEAD
gh pr view N --repo edwardkim/rhwp --json headRefOid,headRefName,maintainerCanModify,mergeable
git ls-remote --heads https://github.com/<contributor>/rhwp.git refs/heads/<head-branch>
~~~

필요성이 확인되면 contributor commit을 rebase, amend, reset, force-push하지 않는다. 동일한 review branch에서
현재 `upstream/devel`을 parent로 갖는 merge commit을 먼저 만들고, 그 다음 호환 보정 code/test를 별도 commit으로
만든다. 최신 `devel`의 대규모 변경은 merge parent로 들어갈 수 있지만 reviewer의 기능 변경으로 집계하지 않으며,
최종 PR 고유 diff는 항상 `upstream/devel...HEAD`로 읽는다.

~~~bash
git merge --no-ff upstream/devel -m "merge: 최신 devel을 PR #N source에 반영"
# 현재 base와 source API의 실제 호환 오류만 보정한다.
git add <code-or-test-path>
git commit -m "test: 최신 devel 호환 초기화를 보완"
git diff --check upstream/devel...HEAD
~~~

호환 보정에는 해당 focused test와 CI 실패 명령에 대응하는 로컬 build/test를 실행한다. source·test commit이
포함됐으므로 9.3.2 fast-pass를 적용하지 않고, push 후 새 PR head의 Full CI·CodeQL·필요한 Render Diff를
모두 확인한다. 검토 기록과 오늘할일은 이 새 code head가 녹색이 된 뒤에만 trailing docs-only commit으로
추가한다.

### 9.3.2 review-only fast-pass

[공용 review-only fast-pass](review_only_fast_pass.md)를 함께 읽는다. collaborator가 contributor의
current code head를 local 검증한 뒤 review 문서·오늘할일·허용된 신규 기준 자료만 source branch에 추가하면
공용 가이드의 **A 경로**다. `devel` 전진은 Update branch를 요구하지 않으며, 직전 code candidate와 같은 PR
identity의 녹색 Build & Test·CodeQL·필요한 Render Diff와 최신 head aggregate를 모두 확인한다.
최종 묶음 직전에 contributor가 새 source를 push했다면, 먼저
[2.6.1 외부 PR review 기록의 source head 정렬](multi_pr_update_branch.md#261-외부-pr-review-기록의-source-head-정렬)을
완료한다.

code 또는 test 보정이 하나라도 있으면 fast-pass가 아니며 최신 head full CI를 기다린다. 이미 완료된 원 PR의
기록만 담는 별도 후속 PR은 공용 가이드의 **B 경로**이며, 이 외부 PR 문서가 아니라 collaborator self 경로와
공용 fast-pass 가이드를 선택한다.

## 9.4 merge 전 조건

- 최신 head의 full CI 또는 9.3.2 fast-pass가 branch protection을 만족한다.
- 필요한 review 문서와 오늘할일이 PR diff에 있다.
- report는 사전 판단 형식이다.
- contributor에게 review 또는 PR comment로 결과를 남긴다. 단, 이미 완료된 원 PR의 기록만 담는 별도
  fast-pass PR은 추가 contributor comment 대상이 아니다.
- 최신 mergeable 상태와 작업지시자 승인을 확인한다.

원 코드 PR을 merge한 뒤에는 [merge 후속 처리](post_merge.md)를 적용한다. 이미 완료된 원 PR의
review·asset·오늘할일만 반영한 별도 fast-pass PR은 issue close/comment와 오늘할일 생성을 반복하지 않되,
devel sync와 branch/worktree/target 정리는 수행한다.
