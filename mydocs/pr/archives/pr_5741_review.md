---
kind: pr-review
status: review-complete-pending-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5741 검토 - Studio 로딩 대기 커서

- source head `ffa92460ba46658f663a9e45a26e0fd88e3cf2d4`, `MERGEABLE/CLEAN`, CI 성공을 확인했다.
- `loadFile -> loadBytes` 중첩 경로가 depth 계수로 wait 커서를 유지하고 `finally`에서 복구하는지 확인했다.
- #5739와 `style.css` 충돌은 각각의 독립 CSS 규칙을 모두 유지해 해결했다. 통합 E2E에서 큰 HWP 로딩 중
  busy/wait 프레임 38/41, 완료 후 `busy=false`, `cursor=auto`를 확인했다.
- 수용 권고. 적용 commit은 `012fb8e1d`이다.
