# 구현계획서 — 호스트 저장 완료 통지 API notifySaved (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660` (`upstream/devel` a803f079 기준)
- **수행계획서**: [`task_m100_2660.md`](task_m100_2660.md)
- **작성일**: 2026-07-21

## 1. 구현 계약

호스트가 내보내기 바이트의 영속화(또는 핸드오프)를 마친 뒤 `notifySaved(fileName?)`를
호출하면, 스튜디오는 ① dirty를 해제하고(`markClean('host-save')`) ② 자동 백업 draft의
IndexedDB **삭제 완료까지 기다린 뒤** `{ ok: true, wasDirty }`를 반환한다. 응답이
도착한 시점에는 팝업을 닫아도 안전하다.

- `exportHwp`/`exportHwpx`/`exportHml`은 무수정 — 자동 markClean 하지 않는다.
- 프로토콜 버전 1 유지. capability `'notify-saved-v1'`로 feature-detect.
- 문서 미로드/이미 clean 상태 호출은 관대한 no-op(`wasDirty: false`) — 멱등.
- autosave `flushNow` 진행 중 discard가 끼어들어도 draft가 부활하지 않는다
  (제너레이션 카운터).

## 2. 예상 코드 변경

### `rhwp-studio/src/recovery/autosave-manager.ts`

`AutosaveManager`에 discard 제너레이션 카운터 추가:

```ts
private discardGeneration = 0;

async discardCurrentDraft(reason = 'discard'): Promise<void> {
  this.discardGeneration += 1;
  // ...기존 로직 유지...
}

async flushNow(reason = 'manual'): Promise<void> {
  // ...
  const generationAtStart = this.discardGeneration;   // saveDraft 직전 캡처
  await this.store.saveDraft({ ... });
  if (this.discardGeneration !== generationAtStart) {
    await this.deleteDraft(current.draftId, 'discarded-during-save');
    return;                                            // lastSavedAt·saved status 생략
  }
  // ...기존 로직...
}
```

`saving` 중 discard → `pendingReason=null`이므로 finally의 재스케줄은 기존 동작대로
발생하지 않는다. 그 외 로직 무변경.

### `rhwp-studio/src/main.ts`

1. 싱글톤 선언부(L58-67) 아래에 코어 함수:

```ts
async function completeHostSave(fileName?: string): Promise<{ ok: true; wasDirty: boolean }> {
  const wasDirty = documentState.isDirty();
  if (fileName) wasm.fileName = fileName;
  documentState.markClean('host-save');                    // 기존 이벤트 체인 재사용
  await autosaveManager.discardCurrentDraft('host-save');  // 삭제 완료 보장 (멱등)
  return { ok: true, wasDirty };
}
```

2. DEV 전용 노출 블록(L74-81)과 별개로 **무조건 노출**:

```ts
(window as any).rhwpStudio = {
  notifySaved: (fileName?: string) => completeHostSave(fileName),
};
```

3. embed 핸들러 블록(L1281-)에 추가:

```ts
async notifySaved(fileName) {
  await initPromise;
  return completeHostSave(fileName);
},
```

### `rhwp-studio/src/embed/protocol.ts`

`EMBED_CAPABILITIES`에 `'notify-saved-v1'` 추가 (4번째 항목). 그 외 무변경.

### `rhwp-studio/src/embed/rpc-router.ts`

```ts
export interface EmbedNotifySavedResult { ok: true; wasDirty: boolean }

export interface EmbedRpcHandlers {
  // ...기존 10개...
  notifySaved(fileName?: string): Promise<EmbedNotifySavedResult>;
}

// routeEmbedRequest switch:
case 'notifySaved': return handlers.notifySaved(
  typeof params.fileName === 'string' && params.fileName.length > 0
    ? params.fileName
    : undefined,
);
```

v1 MessagePort 경로와 legacy 경로 모두 `routeEmbedRequest`를 공유하므로
`runtime.ts` 무수정.

### `npm/editor/transport.js`

`CAPABILITIES`(L2-6)에 `'notify-saved-v1'` 추가. `LONG_RUNNING_METHODS`에는 넣지
않음(기본 10초 타임아웃 충분 — draft 삭제는 밀리초 단위).

### `npm/editor/index.js` — `RhwpEditor`

`getRendererDiagnostics`(L164-176)와 동일한 capability 게이팅 패턴:

```js
async notifySaved(fileName) {
  if (!this._transport.supports('notify-saved-v1')) {
    throw new Error('notifySaved is not supported by this Studio');
  }
  const params = typeof fileName === 'string' && fileName.length > 0 ? { fileName } : {};
  return this._request('notifySaved', params);
}
```

legacy 폴백 모드에서는 `_peerCapabilities`가 비어 throw — `getRendererDiagnostics`와
동일한 기존 한계로 README에 명기.

### `npm/editor/index.d.ts`

```ts
/**
 * 내보내기 바이트의 영속화(업로드/핸드오프) 완료를 스튜디오에 통지합니다.
 * dirty 해제 + 자동복구 draft 삭제 완료 후 resolve — resolve 이후 창을 닫아도 안전.
 * 업로드 실패 시에는 호출하지 마세요(백업 draft 보존).
 */
notifySaved(fileName?: string): Promise<{ ok: true; wasDirty: boolean }>;
```

### 무변경 파일

`document-dirty-state.ts`, `autosave-store.ts`, `embed/runtime.ts`,
`hwpctl/index.ts`(→ #2661), `command/commands/file.ts`, Rust/WASM 전체.

## 3. 테스트 계획

### 단위 — `rhwp-studio/tests/`

`embed-protocol.test.ts`:
- capability deepEqual(L33-37)에 `'notify-saved-v1'` 추가.
- `EmbedRpcHandlers` 리터럴 목 2곳(L51-65, L106-117)에
  `notifySaved: async () => ({ ok: true, wasDirty: true })` 추가
  (인터페이스 확장으로 TS가 누락을 강제 검출).
- 신규 assert:
  - `routeEmbedRequest('notifySaved', {}, handlers)` → 핸들러 호출,
    `fileName === undefined`.
  - `{ fileName: 'a.hwp' }` → `'a.hwp'` 전달.
  - `{ fileName: 123 }`, `{ fileName: '' }` → `undefined` 정규화.

신규 `autosave-discard-race.test.ts` (또는 기존 autosave 테스트 파일에 추가):
- mock store의 `saveDraft`를 지연 Promise로 만들어, `flushNow` 진행 중
  `discardCurrentDraft` 호출 → `saveDraft` resolve 후 draft가 재삭제되는지 검증.
- discard 없는 정상 flush는 삭제가 일어나지 않는지(회귀) 검증.

### SDK 계약 — `npm/editor/tests/`

기존 `renderer-diagnostics-v1.contract.test.mjs` 패턴으로 신규
`notify-saved-v1.contract.test.mjs`:
- `rhwp-connected`가 `'notify-saved-v1'` 광고 시: `editor.notifySaved('b.hwp')`가
  `{ method: 'notifySaved', params: { fileName: 'b.hwp' } }` 요청을 보내고 결과 반환.
- 인자 생략 시 `params: {}`.
- 미광고 시: 명시적 throw.
- `transport.test.mjs`의 클라이언트 CAPABILITIES assert 갱신.

### E2E — 신규 `rhwp-studio/e2e/embed-save-ack.test.mjs`

기존 `embed-transport.test.mjs`(SDK iframe 생성) + `autosave-recovery.test.mjs`
(IndexedDB/다이얼로그 헬퍼) 패턴 조합. DEV 전역(`__documentState`,
`__autosaveManager`) 활용.

- **TC-1 (positive)**: loadFile → iframe 내부 `markDirty` + `flushNow` → draft 존재
  확인 → `editor.exportHwp()` → `editor.notifySaved()` → `isDirty() === false` +
  draft 삭제 확인 → editor 재생성 → 복구 다이얼로그 미표시.
- **TC-2 (negative)**: 동일 절차에서 notifySaved 생략 → 재생성 시
  `.modal-overlay .dialog-wrap`에 "문서 복구" 표시.
- **TC-3 (capability)**: `rhwp-connected` capabilities에 `'notify-saved-v1'` 포함.
- **TC-4 (window API)**: 스튜디오 페이지 직접 접속 → markDirty → flushNow →
  `window.rhwpStudio.notifySaved()` → 리로드 → 다이얼로그 미표시.

## 4. 단계별 작업

### Stage 1 — 코어 + window API + 경합 방어

- `autosave-manager.ts` 제너레이션 카운터, `main.ts` `completeHostSave` +
  `window.rhwpStudio` 노출.
- 경합 단위 테스트 신규 작성 (red→green).
- 검증: `cd rhwp-studio && npm test`, `npx tsc --noEmit`.

### Stage 2 — embed RPC

- `protocol.ts` capability, `rpc-router.ts` 인터페이스·라우팅, `main.ts` 핸들러.
- `embed-protocol.test.ts` 갱신 + 신규 assert.
- 검증: `npm test`.

### Stage 3 — @rhwp/editor SDK

- `transport.js`, `index.js`, `index.d.ts` + 신규 계약 테스트.
- 검증: `npm test`(SDK 테스트 포함), `cd ../npm/editor && npm test`.

### Stage 4 — E2E

- `e2e/embed-save-ack.test.mjs` TC-1~4 작성.
- 검증: `npx vite --host 0.0.0.0 --port 7700 &` →
  `node e2e/embed-save-ack.test.mjs --mode=headless`,
  기존 회귀 `npm run e2e:embed`,
  `node e2e/autosave-recovery.test.mjs --mode=headless`.

### Stage 5 — 문서화 + 최종 회귀

- `npm/editor/README.md` 저장 계약 절:
  - iframe: export → 업로드 **성공 시에만** `notifySaved()` (실패 시 미호출 —
    백업 draft 보존).
  - 팝업: 핸드오프 즉시 `await editor.notifySaved()` 후 `window.close()`.
  - `postMessage` targetOrigin 명시적 오리진 사용 권고.
  - legacy 폴백 모드 한계(capability 미광고 → throw) 명기.
- 최종 회귀: `npm test 2>&1 | tee "$TMPDIR/task2660_full_test.log"`,
  `npm run build`, Stage 4 E2E 3종 재실행.
- 최종 결과보고서 `mydocs/report/task_m100_2660_report.md`.

## 5. 리스크·주의

- **`window.rhwpStudio` 이름 충돌**: 신규 전역이므로 충돌 없음. 확장 여지를 위해
  객체 리터럴로 노출(추후 메서드 추가 가능).
- **embed-protocol.test.ts의 소스 정규식 테스트**(L14-24): main.ts 수정 시 기존
  `getRendererDiagnostics` 정규식 매치가 깨지지 않도록 핸들러 블록 내 상대 순서 유지.
- **flushNow 조기 return**: 제너레이션 감지 시 `saved` status 콜백을 생략하므로
  상태 표시줄 autosave 표시 회귀 여부를 단위 테스트에서 확인.
- **E2E 타이밍**: TC-1/2의 draft 확인은 iframe 컨텍스트의 IndexedDB에서 수행
  (autosave-recovery.test.mjs 헬퍼 재사용).

## 6. 승인

구현계획서 승인 대기.
