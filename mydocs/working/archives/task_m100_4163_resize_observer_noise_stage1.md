---
kind: working
status: completed
issue: 4163
last_verified: 2026-08-08
---

# Task #4163 Stage 1

## 구현·검증

- integration 검증 트리의 미커밋 수정 리프트(rAF 가 같은 렌더링 프레임 안이라 루프를 못 벗어남을 실측 — setTimeout 0 이연 채택).
- 검증: tsc ci-unit 무오류, studio suite 783/783, 레이어 전체 게이트 ALL-PASS 예정.
