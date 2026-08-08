---
kind: working
status: completed
issue: 4179
last_verified: 2026-08-08
---

# Task #4179 Stage 1

## 구현·검증

- integration 검증 트리에서 커밋 리프트(원 커밋 d530e0f16) — 컷 메타 기반 후보 제외 + 전용 회귀 테스트.
- 검증: 레이어 전체 게이트(별첨 게이트 로그) — nextest 전체·Skia·studio 포함 ALL-PASS. 실측 효과: 115쪽 문서 열기 경로의 O(pages) 빌드 소멸(#4145 계열 개선과 합산).
