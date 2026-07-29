# PR #2386 검토 — insert_paragraph(sec,0) 구역 시작 표식 이관 (donggyun112 첫 PR)

- PR: https://github.com/edwardkim/rhwp/pull/2386 — Closes #2384 (자기 등록 이슈)

## 변경 본질

구역 0번 삽입 시 구역 시작 표식(column_type=Section/raw 0x03)이 밀려난
문단에 남아 선두 빈 쪽(1→2쪽)이 생기던 버그 — 표식이 내용이 아니라
**자리 속성**이라는 정확한 모델링으로 새 첫 문단에 이관. 중간 삽입은 불변
(사용자 나누기는 문단 소속 — 경계 판단 명확). 빈 구역은 패턴 불일치 no-op.

**CONTRIBUTING 회귀 가이드 완전 준수**: red→green 테스트 5건 + 수정 전
실패 증명(3/5) + split 동치 단언. 첫 PR 로는 드문 완성도.

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 수정 전 실패 증명 재현 | devel src 원복 → **3/5 FAIL** → 복원 5/5 |
| `cargo test --tests` | 실패 0 / clippy 0 |
| fmt | 1건 줄바꿈 — maintainer edit 정리 push(6be7c195) 후 통과 |
| CI | 전 job success (preflight cancelled 는 승인 경합 아티팩트 — 재실행 green) |

## 판단

**merge 권고.** 이슈 자기 등록 → 원인 모델링 → 경계 판단 → red→green,
첫 기여의 전 절차가 교본적.
