---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7409 메인터너 보정·통합 순서

1. 원 head `f1d205a481dc27a6b05a089844a0f81992b07366`에는 #7397의 변경이 선행한다. 통합에서는 #7397을 `e4e3392eac32f903d520805d1226e52837b4f958`로 한 번 적용하고 #7409의 고유 commit만 `35623acaf3e5c3bd8f23894521cfbf7eacf8b7a4`로 `-x` 적용했다.
2. 원 head는 x 기울기가 유효해도 y 기울기가 `None`이면 조기 `미측정`으로 반환했다. 공유 보정 `a6b27fb9dbc58c7f5de141011365933f7eb814d0`은 x만 필수로 두고 같은 y의 고유 8줄 0.8배 반례·x 분산이 없는 대조군을 추가했다. 이 commit은 #7397의 SVG 문자 순서·XML 속성·선택 쪽 수정도 담는다.
3. 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 Python 7건 PASS 및 실제 API 문서의 정상 배율 `제자리` 진단을 확인했다. 로그는 `output/pr-review/planet6897-20260924/diagnostics/`에만 둔다.
4. 원 PR 단독 merge는 보류하고 보정 포함 통합 PR의 exact head CI·mergeability를 확인한다. 실제 merge 뒤 #7397과 #7409의 각 고유 기여와 공유 보정을 구분해 알린다.
