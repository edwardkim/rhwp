---
name: rhwp 자기 검증 ≠ 한컴 호환
description: rhwp 자기 직렬화 → 자기 재로드 일치는 한컴 호환을 보장하지 않음. HWP 저장 작업 시 작업지시자 한컴2020 수동 검증을 게이트로 삼아야 함
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
`from_bytes(serialize(doc)).page_count == doc.page_count` 가 통과해도 한컴 거부 가능성 상존.

**Why**: rhwp 파서는 자체 출력에 관대하지만 한컴은 엄격 (필드 누락·order·길이·char_count/text 동기화 등). #178 두 번째 시도 (2026-04-19) 가 rhwp 자기 호환 100% 성공 (페이지 9 회복) 했으나 한컴이 3개 샘플 모두 거부. 첫 시도 (4587685) 의 "안정적 경로 가정의 한계" 교훈이 자기 검증에도 적용됨을 재확인.

**How to apply**: HWP 직렬화 관련 작업의 완료 게이트는 (1) rhwp 자기 재로드 + (2) **한컴2020 수동 검증** 두 가지 모두. 자동 테스트만 그린이라고 작업 완료 보고 금지. 작업지시자 수동 검증 결과를 받기 전까지는 "검증 중" 상태로 유지.

상세: `mydocs/troubleshootings/task178_second_attempt_hancom_rejection.md`
