---
kind: pr-review
status: review-complete-pending-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5754 검토 - 상태 표시줄 문서 쪽번호

- source head `d6d1490f888636047a8b574aed676d76b4fea43e`, `MERGEABLE/CLEAN`, CI 성공을 확인했다.
- 현재 표시는 문서의 NewNumber를 따르고 분모는 물리 page count를 유지한다.
- #5741과 `main.ts` import 충돌은 두 기능의 import를 함께 보존해 해결했다. headless E2E에서
  `1 / 1 쪽`에서 NewNumber 삽입 뒤 `7 / 1 쪽`으로 바뀌는 것을 확인했다.
- 수용 권고. 적용 commit은 `2405a551e`이다.
