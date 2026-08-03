---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
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
같은 가시성 branch의 기준을 다시 확인하고, contributor commit을 rebase, amend, reset, force-push하지 않는다.

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

### 9.3.2 review-only fast-pass

[공용 review-only fast-pass](review_only_fast_pass.md)를 함께 읽는다. collaborator가 contributor의
current code head를 local 검증한 뒤 review 문서·오늘할일·허용된 신규 기준 자료만 source branch에 추가하면
공용 가이드의 **A 경로**다. 직전 code candidate의 녹색 Build & Test와 최신 head aggregate를 모두 확인한다.

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
