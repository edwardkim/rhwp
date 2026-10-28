# Task #2396 Stage 1 완료 보고 — custom scheme 회귀 계약 고정

- 이슈: #2396
- 브랜치: `codex/issue-2396-custom-scheme-legacy-request`
- 기준선: `upstream/devel@af5902b6`
- 수행계획서: `mydocs/plans/task_m100_2396.md`
- 구현계획서: `mydocs/plans/task_m100_2396_impl.md`

## 단계 범위

Stage 1은 `rhwp-studio/tests/embed-protocol.test.ts`의 회귀 테스트만 변경했다.
`rhwp-studio/src/embed/runtime.ts`와 protocol/router 구현은 변경하지 않았다.

## 추가한 계약

### custom scheme top-level same-window

- `hostWindow === parentWindow`
- `event.source === parentWindow`
- `event.origin === 'alhangeul-studio://app'`
- legacy `ready` 요청이 같은 target origin으로 `rhwp-response`를 반환해야 한다.

현행 구현에서는 초기 origin gate에서 요청이 폐기되어 50ms timeout으로 실패했다. 이 결과로 #2396의
회귀를 unit runtime 경계에서 재현했다.

### custom scheme iframe parent와 forged sibling

- iframe 환경의 exact parent source라도 custom scheme origin이면 거부한다.
- source가 parent와 다른 forged sibling 요청은 custom scheme origin이어도 거부한다.
- 두 경우 모두 handler와 response가 호출되지 않고 transferred port가 닫혀야 한다.

두 negative case는 현행 구현에서 통과해 수정 후에도 유지해야 할 보안 기준선으로 고정했다.

## 검증 결과

```text
node --test rhwp-studio/tests/embed-protocol.test.ts
tests 14 / pass 13 / fail 1
```

| 계약 | 결과 | 판정 |
|---|---|---|
| 기존 embed protocol 12건 | 12/12 PASS | 기존 기준선 유지 |
| custom scheme top-level same-window | timeout FAIL | #2396 회귀 재현 성공 |
| custom scheme iframe parent + forged sibling | PASS | 보안 거부 기준선 유지 |

## 다음 승인 게이트

Stage 2에서는 `installEmbedRuntime()`의 초기 gate 한 곳만 변경한다. parent/host identity가 같은
top-level same-window에만 unusable origin 예외를 적용하고, Stage 1의 14개 focused 테스트를 모두
통과시키는 것이 목표다. 작업지시자 승인 전에는 Stage 2 구현을 시작하지 않는다.
