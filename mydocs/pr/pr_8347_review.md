# PR #8347 검토 — fix(layout): 엇갈린 병합 칸의 행 높이를 칸 내용으로 푼다

## 최종 판정

**머지 보류 — 검증 진행 중.** 동일 입력의 최신 base/누적 후보 실행과 필수 검증을 완료한 뒤 판정을 갱신한다. 이 잠정 상태를 기여자 재작업 결론이나 새 회귀 검출로 사용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8347, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `19aed211d178996cf83aae89388ed91d6ec93bcf`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:52.364267+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Build & Test: failure, test-archive-b-shard-1 / Default-feature tests (Archive B): cancelled, Lint (fmt, clippy, WASM check): cancelled.
- 원 contributor의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례·필수 로컬 gate는 현재 검토 중이며 아직 완료로 판정하지 않는다.
