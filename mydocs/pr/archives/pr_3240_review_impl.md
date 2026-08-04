---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3240 통합 실행 계획 — 활성 머리말 선택 시각 검증

#3240은 누적 순서의 마지막 renderer 변경이다. `4253b1c53`과 후속 보정 `f5bb1db2`를 함께 적용한
`cfaae968e`를 하나의 기능 단위로 취급한다. #3228이 같은 활성 머리말 정보를 소비하므로 두 변경은
동일 누적 tree에서 집중 테스트와 전체 회귀 테스트로 확인한다.

시각 검증은 synthetic regression fixture를 Native Skia PNG로 렌더하여
`mydocs/pr/assets/pr_3240_lpaiu-cs_issue3234_p002_p003_review.png`에 실제 보이는 결과를 보존한다.
Windows `win10-ted` 재생성까지 통과해야 통합 PR을 준비한다. 롤백이 필요하면 `e649cc810`과
`cfaae968e` 두 체리픽을 역순으로 revert한다.
