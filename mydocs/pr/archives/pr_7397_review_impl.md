---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7397 메인터너 보정·통합 순서

1. 원 head `1d0123f39fd398e6702225a693fcc9dfd529b914`의 고유 commit을 최신 base 위에 `-x`로 적용한 SHA는 `e4e3392eac32f903d520805d1226e52837b4f958`다.
2. 원 head의 run 내부 문자 정렬 손상을 제어 SVG로 재현했다. 공유 보정 `a6b27fb9dbc58c7f5de141011365933f7eb814d0`이 글자 순서를 보존하고 XML 속성 순서·선택 쪽 경계도 수정했다. 이는 #7409 도구에도 적용된다.
3. 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 Python 7건 PASS와 실제 API 49·60쪽 `비교가능_주의` 진단을 확인했다. 로그는 `output/pr-review/planet6897-20260924/diagnostics/`에만 둔다.
4. 원 PR 단독 merge는 보류하고 보정 포함 통합 PR의 exact head CI·mergeability를 확인한다. 실제 merge 뒤 두 PR의 기여와 공유 보정을 각각 설명한다.
