# PR #3475 검토 — export-structure 가 수식 내용을 보존

Issue: #3413 / author: planet6897 / reviewer: edwardkim / milestone: v1.0.0
연작: planet6897 7건 누적 검토(2026-07-28), 체리픽 2순위

## metadata (작성 시점 참고값)

| 항목 | 값 |
|---|---|
| 기능 커밋 | `4badc9b21` → 누적 `7f5b89f14` |
| 규모 | +75 -8 (`queries/rendering.rs`, `queries/structure.rs`, 테스트 +58) |
| CI | 원 PR head SUCCESS, mergeable CLEAN |

## 변경

`export-structure` 가 수식(Equation) 컨트롤의 내용을 떨어뜨려 텍스트 표면의 마지막 누락
축이 남아 있었다. #3413 계열(export-text 는 kevin9327 #3419 로 먼저 정정)의 structure 축
마무리다. 쿼리 표면만 변경하며 렌더·저장 경로는 건드리지 않는다.

## 검증

- focused 3건 통과 (structure 봉투에 수식 스크립트 보존, 무수식 문서 무회귀 포함)
- 누적 branch 전체 게이트: release-test 4253 passed / 0 failed, fmt·clippy 클린

## 시각 판정

불필요 — 쿼리 JSON 표면 변경, 렌더 출력 경로 무변경, fixture 추가 없음.

## 권고

**merge (통합 PR 경유).** #3413 이 부분 해결로 open 유지 중이면 이 PR 반영 뒤 잔여 축을
재확인해 close 여부를 판단한다.
