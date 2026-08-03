---
kind: review
status: rework-required
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-04
---

# PR #3908 검토 - R1~R100 에이전트 로드맵

- 원 head: `215fca1bb82a199b2f6283cb74cee74c07f4322f`
- 범위: `mydocs/tech/agent_roadmap/` 11편과 기술 문서 지도 연결.
- 예외 절차: 1,724줄 문서 대형 PR로 `rework_and_exceptions` 절차를 적용했다.
- 시각 검증: 불필요. 문서 전용 변경이다.

## 차단 사항

문서 첫머리는 "완료 표기는 머지 링크와 함께만"이라고 정하지만, 아직 열린 PR을
`[완료]`로 표시한다. 검토 시점에 #3808·#3898은 모두 `BEHIND`인 열린 PR인데,
`track_d_discovery.md`는 각각 planSchema 접합(R32), 레시피 3(R34)을 완료로 적었다.
`track_b_guards_security.md`도 열린 #3903의 결과를 "이제" 확정된 상태로 서술한다.

또한 R36은 #3889의 148개 필드 전수 설명을 전제로 하지만, 해당 PR의 active 가이드
수치는 현재 후보의 실제 159개와 다르다. 이를 그대로 연결하면 로드맵의 DoD가
실측 대상과 불일치한다.

## 요청 사항

열린 PR은 `[이슈]` 또는 `[문서]`로 표시하고, 실제 merge commit이 생긴 뒤에만
`[완료]`와 merge 링크로 승격해야 한다. #3889 보정 후 필드 수와 DoD를 재실측하고,
같은 로컬 자기서술 결과로 다시 검토해야 한다.
