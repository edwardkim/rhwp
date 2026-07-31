---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3613 리뷰 — 표 cell 선택·세션 수정

- 원 PR / 작성자: [#3613](https://github.com/edwardkim/rhwp/pull/3613) / `@kevin9327`
- 관련 이슈: [#3603](https://github.com/edwardkim/rhwp/issues/3603)
- 원 head / 통합 반영: `51ea48618` / `1448bae6a`–`10f452a38`

`resolve_table_cell`로 표·행·열 선택을 한 곳에 모으고, `hwp_doc_set_cell`이 세션 문서의 단일 cell을
수정하도록 한다. 무상태 `export-tables`를 이용한 좌표 선택과 실제 HWPX 재저장 증적은 #3612 선행 merge에
묶이지 않도록 한 것이 좋다. row/column의 `u64 as u16` 축소는 65,536 이상이 0으로 돌아갈 수 있으므로,
통합 보정 `94cdd74ce`에서 `try_from` 범위 오류와 회귀를 추가했다.

`mcp_session_setcell_contract`와 전체 release-test exit 0, fmt·clippy·release build를 확인했다. 첨부
PNG는 선택적 설명 증적이며 layout 구현 변경이 아니므로 visual sweep 합격 주장으로 해석하지 않는다.
**통합 PR full CI 조건부 수용** 후 원 PR을 supersede close한다.
