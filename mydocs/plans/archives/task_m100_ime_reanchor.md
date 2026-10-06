# 수행계획 — task_m100_ime_reanchor

- **이슈**: [#4245](https://github.com/edwardkim/rhwp/issues/4245) (#4150 IME 조합 계열 후속)
- **브랜치**: `task_m100_ime_reanchor_guard` (stack: `task_m100_4151_cell_block_toggle_sync` 위)
- **기준**: `upstream/devel` `5a4f26d0d`
- **작성 시각**: 2026-08-08 KST

## 1. 목표

IME 조합 업데이트의 raw replace(`input-handler-text.ts` 조합 분기)가 wasm 의 deferred replace
범위 가드에 거부되면 예외가 `onInput` 밖으로 던져져 핸들러가 죽고, 조합 추적
(compositionAnchor/Length)이 낡은 값으로 wedge 되어 이후 모든 조합 업데이트가 연쇄 실패한다.
거부를 잡아 조합을 현재 캐럿에 재정박해 입력 스트림을 잇는다.

## 2. 변경 경계

- 정상 경로 무변경 — 가드 거부(외부 변이로 앵커·길이가 낡은 경합) 시에만 복구 분기.
- 복구는 "현재 캐럿 재정박 + 이번 조합 텍스트 재삽입" — 그것도 실패하면 이번 업데이트만
  버리고 로그(다음 캐럿 이동에서 자연 동기화).
- `onCompositionEnd`는 raw 뮤테이션이 없어(command 기록만) 변경 불요 — 확인 완료.

## 3. 구현·래칫 순서

1. 조합 분기의 `replaceTextAtRaw`를 try/catch — 거부 시 warn + `compositionAnchor`를 현재
   캐럿으로 재정박, `compositionLength=0`, 재삽입 시도.
2. 소스 계약 테스트: 조합 replace 가 try 블록 안에 있고, catch 가 현재 캐럿 기준 재정박 +
   길이 리셋을 수행.
3. 실브라우저 강제 트립: `compositionLength` 오염 후 조합 업데이트 → uncaught 없음·복구·
   최종 텍스트 정합·undo 완전 복원 확인.

## 4. 검증 게이트

- `tests/composition-replace-reanchor.test.ts` 2종
- tsc ci-unit / `npm run test` 전체
- 실브라우저(:7701) 강제 트립 시나리오

원격 push, PR 생성, 이슈 comment·close는 별도 승인 전 수행하지 않는다.
