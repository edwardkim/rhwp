---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3623 리뷰 — 문서 분할과 extract-pages 안내

- 원 PR / 작성자: [#3623](https://github.com/edwardkim/rhwp/pull/3623) / `@kevin9327`
- 관련 이슈: [#3622](https://github.com/edwardkim/rhwp/issues/3622)
- 원 head / 통합 반영: `1bf5ade0e` / `76bf731f3`, `2a3fc5afa`

`hwp_split_document`와 `extract-pages`의 자가 서술을 CLI/MCP에 넣고, 분할 산출물과 문서 안내를
연결한다. 원 branch의 merge commit은 제외하고 원 contributor 기능·증적 commit만 적용했다.
`split_document_tool_contract`와 전체 release-test exit 0, fmt·clippy·release build를 확인했다. split
PNG는 operation 증적일 뿐 layout pipeline의 변경 또는 visual sweep 합격 근거가 아니다. **통합 PR full CI
조건부 수용**이다.
