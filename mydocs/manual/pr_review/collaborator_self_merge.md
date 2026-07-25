---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# Collaborator self-merge 후보

이 경로는 collaborator가 본인 PR을 merge 후보로 준비할 때만 쓴다. maintainer의 외부 contributor PR 일반
처리를 대체하지 않는다.

## 8.1 적용 조건

- PR 작성자 또는 준비자가 repository collaborator다.
- PR 번호가 이미 있어 review 문서명을 확정할 수 있다.
- merge 뒤 별도 문서 commit을 만들지 않기 위해 review 문서를 현재 PR diff에 포함한다.
- ready 전환, review approval, merge 판단은 작업지시자 승인 뒤에만 한다.

## 8.2 문서와 오늘할일

review 문서는 처음부터 archive 경로에 둔다.

~~~text
mydocs/pr/archives/pr_N_review.md
mydocs/pr/archives/pr_N_review_impl.md
mydocs/pr/archives/pr_N_report.md          # 필요 시
mydocs/orders/YYYYMMDD.md                  # 갱신이 필요한 경우
~~~

### 8.2.1 오늘할일 생성·갱신 시점

오늘할일은 이슈 등록·branch 생성·조사·계획·구현 중간에는 만들거나 갱신하지 않는다. 변경 범위·검증·merge
판단·PR 생성 승인이 확정된 **최종 준비 시점**에 최신 devel의 오늘할일을 반영하고, 최초 remote push와 PR
생성 전에 같은 PR diff에 포함한다.

PR 번호 발급 뒤 번호를 보태기 위해 오늘할일을 다시 만들거나 갱신하지 않는다. 이미 active 경로에 만든
review 문서는 다음 PR에 임시로 동반하지 말고 같은 PR 준비 단계에서 archive 경로로 옮긴다.

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

merge 뒤에는 이 PR 자체가 review 기록을 포함했는지와 issue 상태를 확인하기 위해
[merge 후속 처리](post_merge.md)를 적용한다.
