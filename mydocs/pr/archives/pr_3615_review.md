---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3615 리뷰 — `convert --json`과 `hwp_convert_hwp5`

- 원 PR / 작성자: [#3615](https://github.com/edwardkim/rhwp/pull/3615) / `@kevin9327`
- 관련 이슈: [#3605](https://github.com/edwardkim/rhwp/issues/3605)
- 원 head / 통합 반영: `a5e06d171` / `dfda3168a`

HWPX에서 편집 가능한 HWP5로 변환하는 CLI에 stdout JSON envelope을 추가하고 같은 기능을 MCP에
선언한다. binary output과 JSON metadata의 경계를 유지하는 계약이며, 후속 #3618·#3620의 공통 ancestor로
한 번만 통합했다. output-axis JSON contract와 전체 release-test exit 0, fmt·clippy·release build를
확인했다. **통합 PR full CI 조건부 수용**이다.
