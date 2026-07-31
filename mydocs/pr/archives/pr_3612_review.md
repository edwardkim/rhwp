---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3612 리뷰 — 세션 조회·페이지 SVG 렌더

- 원 PR / 작성자: [#3612](https://github.com/edwardkim/rhwp/pull/3612) / `@kevin9327`
- 관련 이슈: [#3609](https://github.com/edwardkim/rhwp/issues/3609)
- 원 head / 통합 반영: `b533740cd` / `4c1407478`, `a6c81b6bd`

세션 핸들에서 문서 정보·필드·표를 무상태 JSON 봉투와 동형으로 조회하고, 편집 직후 같은 핸들에서 한
페이지 SVG를 저장하는 MCP 도구를 추가한다. 기존 helper를 재사용해 재파싱 없이 같은 데이터 계약을 유지한
점은 적절하다. 다만 page를 `u64 as u32`으로 축소하면 매우 큰 page가 0쪽으로 wrap될 수 있어, 통합
candidate의 별도 `94cdd74ce`가 범위 오류로 차단하고 회귀를 고정했다. 또한 0쪽 문서의 범위 진단은
underflow하지 않도록 별도 `208c3f618`이 빈 문서 오류를 먼저 반환한다.

`mcp_session_view_contract`를 포함한 전체 release-test가 exit 0이고, fmt·clippy·release build도
통과했다. 포함 PNG는 세션 fill 뒤 render 결과의 설명 증적이며, renderer 구현 변경이 아니므로 독립
PDF 충실도 판정으로 사용하지 않는다. **통합 PR 최신 head full CI 조건부 수용**이며, 원 PR은 통합 PR
merge 뒤 supersede close한다.
