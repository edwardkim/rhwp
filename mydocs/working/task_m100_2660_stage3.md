# Stage 3 완료보고서 — @rhwp/editor SDK notifySaved (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660`
- **구현계획서**: [`../plans/task_m100_2660_impl.md`](../plans/task_m100_2660_impl.md)
- **작성일**: 2026-07-21

## 1. 구현 내용

TDD(red→green). RED에서 4개 테스트가 의도한 이유로 실패함을 확인한 뒤 구현했다.

### `npm/editor/transport.js`

- 클라이언트 `CAPABILITIES`에 `'notify-saved-v1'` 추가.
- `LONG_RUNNING_METHODS`에는 미포함 — 기본 10초 타임아웃(draft 삭제는 밀리초 단위).

### `npm/editor/index.js` — `RhwpEditor.notifySaved(fileName?)`

- `getRendererDiagnostics`와 동일한 capability 게이팅: 스튜디오가
  `notify-saved-v1`을 광고하지 않으면(구버전 또는 legacy 폴백 연결) **요청을 보내지
  않고** `notifySaved is not supported by this Studio`로 명시적 실패.
- fileName은 비어있지 않은 문자열일 때만 `{ fileName }` 파라미터로 전달.
- JSDoc에 저장 계약 명기: resolve 이후 창을 닫아도 안전 / 업로드 실패 시 호출 금지.

### `npm/editor/index.d.ts`

- `RhwpEditor`에 `notifySaved(fileName?: string): Promise<{ ok: true; wasDirty: boolean }>`
  선언 + 저장 계약 JSDoc 추가.

## 2. 테스트 (RED → GREEN)

신규 `npm/editor/tests/notify-saved-v1.contract.test.mjs`
(기존 `renderer-diagnostics-v1.contract.test.mjs` + transport 테스트 MessageChannel
하네스 패턴 조합):

| 테스트 | RED | GREEN |
|---|---|---|
| capability 광고 시 `notifySaved('b.hwp')`/`notifySaved()`가 `{method:'notifySaved', params:{fileName}/{}}` 요청 후 결과 반환 | 실패 (메서드 부재) | 통과 |
| 미광고 시 요청 0건 + 명시적 throw | 실패 | 통과 |
| `index.d.ts` notifySaved 선언 계약 | 실패 | 통과 |
| `transport.test.mjs` 클라이언트 CAPABILITIES assert에 `'notify-saved-v1'` 추가 | 실패 | 통과 |

작성 과정 메모: 최초 RED 실행에서 assert 실패 시 `transport.destroy()`가 건너뛰어져
열린 MessagePort가 이벤트 루프를 잡아 `node --test`가 행 걸리는 문제를 발견,
계약 테스트를 try/finally(destroy + 서버 포트 close) 구조로 작성했다.

## 3. 검증 결과

- `cd npm/editor && node --test tests/*.test.mjs`: **21/21 통과**.
- `cd rhwp-studio && npm test`(SDK 테스트 포함 실행): **459개 중 457 통과, 실패 2** —
  Stage 1·2와 동일한 기존 로컬 환경 이슈(`cell-flow-boundary`,
  `canvaskit-resource-key`; Windows spawnSync, HEAD 동일 재현 확인됨).
  로그 `$TMPDIR/task2660_s3_green.log`.
- TypeScript 소스 무변경(SDK는 JS + d.ts) — tsc 재검증은 Stage 2와 동일 상태.

## 4. 호스트 사용 예 (팝업 흐름)

```js
const bytes = await editor.exportHwp();
const base64 = uint8ArrayToBase64(bytes);
window.opener?.postMessage(
  { action: 'documentExported', content: base64, format: 'hwp' },
  HOST_ORIGIN, // '*' 대신 명시적 오리진 권장
);
await editor.notifySaved(); // dirty 해제 + draft 삭제 완료 대기
window.close();
```

## 5. 다음 단계

Stage 4 — E2E (`e2e/embed-save-ack.test.mjs` TC-1~4 + 기존 회귀). 승인 후 진행.
