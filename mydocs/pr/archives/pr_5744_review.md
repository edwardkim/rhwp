---
kind: pr-review
status: review-complete-pending-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5744 검토 - 음수 줄간격 문단 테두리

- source head `0651d12bc6b035fa7aba032f939ca35ae1122eac`, `MERGEABLE/CLEAN`, CI 성공을 확인했다.
- 음수 줄간격에서 문단 아래 테두리를 마지막 line box 아래로 clamp하고 양수 줄간격 기존 동작을 고정한다.
- 전체 release-test nextest 8,025건 및 native-Skia library gate를 통과했다.
- 수용 권고. 적용 commit은 `25d0e3a2c`이다.
