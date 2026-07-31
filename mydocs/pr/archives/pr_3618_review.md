---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3618 리뷰 — `export-hml --json`과 MCP 봉투

- 원 PR / 작성자: [#3618](https://github.com/edwardkim/rhwp/pull/3618) / `@kevin9327`
- 관련 이슈: [#3616](https://github.com/edwardkim/rhwp/issues/3616)
- 원 head / 통합 반영: `056f4c98a` / `11382b435`

HML 재직렬화 산출물의 경로·manifest를 JSON으로 반환하고 MCP `hwp_export_hml`에 동일 봉투를 노출한다.
기존 #3615 ancestor는 재적용하지 않아 한 번의 통합 변경으로 보존했다. `export_hml_json_contract`와
전체 release-test exit 0을 확인했다. 첨부 round-trip PNG는 작업 증적이며 renderer/golden 변경이 아니므로
독립 visual sweep 합격 근거는 아니다. **통합 PR full CI 조건부 수용**이다.
