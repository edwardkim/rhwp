---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3634 리뷰 — 소형 모델용 digest macro

- 원 PR / 작성자: [#3634](https://github.com/edwardkim/rhwp/pull/3634) / `@kevin9327`
- 관련 이슈: [#3633](https://github.com/edwardkim/rhwp/issues/3633)
- 원 head / 통합 반영: `d388e807d` / `05de7534b`

`hwp_digest`가 문서 요약·다음 행동을 한 번의 작은 응답으로 제공하고, MCP에서도 같은 macro를 노출한다.
MCP schema의 선택 `maxChars`가 CLI로 전달되지 않던 결함은 통합 검토에서 확인해 `94cdd74ce`의
`optionalArgs` 실행과 실제 call 길이 회귀로 보정했다. `digest_macro_contract`와 전체 release-test exit 0,
fmt·clippy·release build를 확인했다. **통합 PR full CI 조건부 수용**이다.
