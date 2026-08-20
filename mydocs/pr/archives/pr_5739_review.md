---
kind: pr-review
status: review-complete-pending-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5739 검토 - Studio 도구 상자 표시 상태

- source head `1182c552825fec0f293cd44802d457e9087d58be`, `MERGEABLE/CLEAN`, CI 성공을 확인했다.
- 기본/서식 도구 상자의 저장값, 메뉴의 `aria-checked`, 첫 페인트 FOUC 방지를 함께 구현한다.
- 통합 후보에서 Studio TypeScript, 단위 테스트, headless E2E를 통과했다. E2E는 기본 표시, 즉시 토글,
  재시작 복원 및 38개 프레임의 숨김 상태를 확인했다.
- 수용 권고. 적용 commit은 `bc421623b`이다.
