---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# Collaborator self-merge 후보

이 경로는 collaborator가 본인 PR을 merge 후보로 준비할 때만 쓴다. maintainer의 외부 contributor PR 일반
처리를 대체하지 않는다.

## 8.1 적용 조건

- PR 작성자 또는 준비자가 repository collaborator다.
- PR 번호가 이미 있어 review 문서명을 확정할 수 있다.
- merge 뒤 별도 문서 commit을 만들지 않기 위해 review 문서를 현재 PR diff에 포함한다.
- ready 전환, self-review 기록 확정, merge 판단은 작업지시자 승인 뒤에만 한다.

## 8.2 문서와 오늘할일

review 문서는 처음부터 archive 경로에 둔다.

~~~text
mydocs/pr/archives/pr_N_review.md
mydocs/pr/archives/pr_N_review_impl.md
mydocs/pr/archives/pr_N_report.md          # 필요 시
mydocs/orders/YYYYMMDD.md                  # 갱신이 필요한 경우
~~~

### 8.2.1 PR 채번과 오늘할일 생성·갱신 시점

오늘할일은 이슈 등록·branch 생성·조사·계획·구현 중간에는 만들거나 갱신하지 않는다. 구현과 로컬
검증이 끝나고 작업지시자가 remote push와 PR 생성을 승인한 최종 준비 시점에 다음 순서로
작성한다.

1. 검증을 마친 후보 commit을 원격 작업 branch에 push한다.
2. Draft 지시가 없으면 Open PR을 생성해 번호 `N`을 받는다.
3. reviewer를 지정하지 않고 self-review를 `mydocs/pr/archives/pr_N_review.md`와 필요한 오늘할일에 기록한다.
4. review 문서와 오늘할일을 같은 source branch의 후속 commit으로 push해 PR diff에 포함한다.

PR 생성 전에 번호를 예측해 review 파일명을 만들지 않는다. 이미 active 경로에 만든 review 문서는
다음 PR에 임시로 동반하지 말고, 해당 PR 번호가 확정된 뒤 archive 경로와 파일명을 확정한다.

이 시점에 local CI 검증이 완료됐다면 review 문서와 오늘할일에는 결과를 과거형으로 적는다. 검증을
다시 실행할 계획처럼 쓰지 말고, 남은 GitHub Actions·작업지시자 승인·merge만 미래 조건으로 분리한다.

## 8.3 remote push

collaborator는 권한 제약이 없는 한 fork origin이 아니라 원본 remote upstream의 작업 branch로 push한다.

~~~bash
git push upstream HEAD:task_m100_<issue>
~~~

## 8.4 merge 전 조건

- 최신 PR head의 GitHub Actions가 통과한다.
- 필요한 review, review_impl, 오늘할일이 PR diff에 포함된다.
- draft·mergeable·head SHA·CI 상태는 작성 시점 참고값으로만 기록한다.
- 작업지시자 승인을 받는다.

### 8.4.1 명시 지시된 maintainer `--admin` merge 예외

일반 collaborator는 `--admin`으로 branch protection을 우회하지 않는다. 단, maintainer 권한을 가진
실행자가 collaborator self PR을 처리하면서 작업지시자로부터 **해당 PR의 `--admin` merge 명시 지시**를
받은 경우에는 다음 조건을 모두 만족할 때만 사용할 수 있다.

- code candidate와 review·오늘할일 trailing head 각각의 최신 GitHub Actions가 성공했고, 실패·대기 중인
  check가 없다.
- trailing head의 `mergeable`은 `MERGEABLE`, `mergeStateStatus`는 `CLEAN`이며, merge 직전에 다시
  조회한 head SHA가 명령의 `--match-head-commit` 값과 같다.
- trailing commit은 review, 오늘할일, stage·절차 문서만 추가한다. source, test, fixture, workflow,
  baseline, sample 변경이 trailing commit에 섞이면 이 예외를 적용하지 않는다.
- 이 옵션은 reviewer 부족, 실패한 검증, 오래된 code candidate, 충돌을 우회하는 용도로 사용하지 않는다.

위 조건에서는 다음과 같이 squash merge할 수 있다. 권한이 없는 collaborator의 토큰에서는 명령이 실패할
수 있으며, 그 경우 정상 merge 경로로 되돌아간다.

~~~bash
gh pr merge N --repo edwardkim/rhwp --squash --admin \
  --match-head-commit <latest-trailing-head>
~~~

merge 뒤에는 이 PR 자체가 review 기록을 포함했는지와 issue 상태를 확인하기 위해
[merge 후속 처리](post_merge.md)를 적용한다. 이 과정이 끝나면 이번 PR만을 위해 만든 local worktree는
clean 상태와 다른 작업의 소유 여부를 확인한 뒤 제거한다. 단순히 다음 작업에 재사용할 편의만으로 유지하지
않으며, 제거할 수 없는 활성 작업 또는 사용자 보존 지시가 있으면 그 사유와 경로를 최종 상태에 남긴다.

작업지시자가 이 경로의 PR 병합과 `merge 후 후속 처리`를 승인했다면, 그 승인은 이번 PR 전용의 clean한
local branch와 local worktree를 제거하는 데에도 적용된다. 따라서 조건을 만족한 뒤에는 별도의 "정리" 지시를
기다리지 않고 후속 처리에서 제거한다. 이 승인은 원격 head branch 삭제, 기본 작업공간, 공유 target,
사용자·다른 도구의 branch/worktree 삭제에는 적용되지 않으며, 그 대상은 별도 승인과 소유 확인이 필요하다.
