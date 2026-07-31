---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3632 리뷰 — 에이전트 역할 profile router

- 원 PR / 작성자: [#3632](https://github.com/edwardkim/rhwp/pull/3632) / `@kevin9327`
- 관련 이슈: [#3629](https://github.com/edwardkim/rhwp/issues/3629)
- 원 head / 통합 반영: `05d1c1ac0` / `0ea4ee794`

7개 직무 profile의 capability와 `mcp-serve --profile` 도구 목록을 제공한다. 목록 필터만으로는
`tools/call` dispatch를 보호하지 못하는 경계 결함을 통합 검토에서 발견해, `94cdd74ce`가 숨긴 session
도구 호출도 `isError`로 차단하도록 보정했다. `agent_profile_router_contract`와 전체 release-test exit 0,
fmt·clippy·release build를 확인했다. **통합 PR full CI 조건부 수용**이다.
