---
name: feedback_subissue_linking
description: 라운드형/실행형 이슈는 생성 직후 부모(umbrella) 이슈의 GitHub 서브 이슈로 등록해 연결 관리 — 작업지시자 확인 방식 (2026-07-09)
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

**라운드형·실행형 이슈는 생성 직후 부모 이슈의 GitHub 서브 이슈로 등록**해 연결
관리한다 (작업지시자: "서브 이슈인 경우 부모 이슈의 하위 이슈로 등록해서 연결관리하는
것은 좋은 방식입니다").

**Why**: #1904(1차 리팩토링 umbrella)에서 라운드 이슈 16건 중 2건이 누락/오귀속
(#2003 미등록, #1925 는 계획 이슈 #1883 에 잘못 귀속)돼 있던 것을 발견·정비한 사례.
서브 이슈로 걸어두면 부모 이슈에서 진행률(completed/total)이 자동 집계되고 라운드
추적이 한 화면에서 된다.

**How to apply**:
- 이슈 생성 직후 한 절차로: `ID=$(gh api repos/{r}/issues/{N} --jq .id)` →
  `gh api -X POST repos/{r}/issues/{부모}/sub_issues -F sub_issue_id=$ID`
  (번호가 아니라 **numeric id** 를 넘긴다).
- 이미 다른 부모에 걸린 이슈는 `-F replace_parent=true` 로 재부모화 (서브 이슈는
  부모 1개만 허용 — 422 "may only have one parent").
- 귀속 기준: **실행 라운드는 실행 umbrella**(예: #1904), 계획 이슈(#1883)가 아니라.
- 조회: `gh api repos/{r}/issues/{부모}/sub_issues` / 부모 확인은 GraphQL
  `issue(number:N){parent{number}}`.

관련: [[project_1582_refactor_umbrella]] [[feedback_task_numbering]]
