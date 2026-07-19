---
kind: memory
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-17
---

# PR 생성은 별도 승인 후 진행

- 날짜: 2026-05-25
- 출처: Task #1116 진행 중 작업지시자 피드백

## 규칙

내부 타스크에서 "PR 준비"와 "PR 생성"을 분리한다.

작업지시자가 PR 준비를 지시한 경우에는 커밋, 검증 기록, PR 본문 초안, 생성 명령 준비까지만 수행한다. GitHub에 실제 Open PR 또는 Draft PR을 만드는 `gh pr create` 실행은 별도 승인을 받은 뒤 진행한다.

승인 없이 PR을 열었으면 즉시 PR을 close하고, 후속 작업은 분석/계획/승인 대기 상태로 되돌린다.
