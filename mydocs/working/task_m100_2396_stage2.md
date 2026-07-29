# Task #2396 Stage 2 완료 보고 — top-level same-window 예외 구현

- 이슈: #2396
- 브랜치: `codex/issue-2396-custom-scheme-legacy-request`
- 기준선: `upstream/devel@af5902b6`
- 선행 단계: `mydocs/working/task_m100_2396_stage1.md`

## 단계 범위

Stage 2는 `rhwp-studio/src/embed/runtime.ts`의 초기 source/origin gate 한 곳만 수정하고,
Stage 1에서 고정한 focused embed protocol 테스트를 실행했다. protocol helper, RPC router,
MessageChannel binding과 공개 API는 변경하지 않았다.

## 구현

runtime 설치 시 다음 window identity를 계산한다.

```ts
const isTopLevelSameWindow = options.parentWindow === options.hostWindow;
```

초기 gate는 source가 parent와 다르면 항상 거부하고, HTTP(S)가 아닌 origin은 top-level same-window가 아닌
경우에만 거부하도록 변경했다.

```text
event.source !== parentWindow
OR (origin이 usable HTTP(S)가 아님 AND parentWindow !== hostWindow)
→ transferred ports 정리 후 거부
```

## 보안 경계 확인

| 환경 | source | origin | 결과 |
|---|---|---|---|
| top-level same-window | parent/host와 동일 | custom scheme | 허용 |
| iframe parent | parent와 동일 | custom scheme | 거부 |
| forged sibling | parent와 다름 | custom scheme | 거부 |
| 기존 iframe parent | parent와 동일 | HTTP(S) | 허용 |

custom scheme 문자열 allowlist를 추가하지 않았고 `isUsableParentOrigin()`의 HTTP(S) 정의도 변경하지 않았다.
top-level에서는 요청 source가 host/parent window와 같은 실행 컨텍스트라는 identity만 예외 근거로 사용한다.

## focused 검증

```text
node --test rhwp-studio/tests/embed-protocol.test.ts
tests 14 / pass 14 / fail 0
```

- Stage 1 top-level positive case: timeout FAIL → PASS
- custom scheme iframe parent 및 forged sibling: PASS 유지
- 기존 embed protocol 12건: 12/12 PASS
- `git diff --check`: PASS

## 다음 승인 게이트

Stage 3에서는 코드 변경 없이 Studio + editor 전체 Node tests, 현재 기준선의 dev WASM 생성, Studio
production build와 추적 파일 범위 검사를 수행한다. 작업지시자 승인 전에는 Stage 3 전체 검증을 시작하지
않는다.
