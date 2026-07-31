---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3631 리뷰 — 경량 에이전트 내성 설계

- 원 PR / 작성자: [#3631](https://github.com/edwardkim/rhwp/pull/3631) / `@kevin9327`
- 관련 이슈: [#3630](https://github.com/edwardkim/rhwp/issues/3630)
- 원 head / 통합 반영: `538d0082b` / `457c3c9ea`

작은 모델이 복합 명령을 안전하게 조합하도록 CLI/MCP contract 확장 원칙과 실패 경계를 설계 문서로
정리한다. 구현을 대체하는 명령이 아닌 설명·라우팅 문서이므로 changed-from 링크와 metadata 검사로
검증했다. 실 구현 계약은 함께 들어온 JSON·profile·digest contract tests가 보호한다. **통합 PR과 함께
수용**한다.
