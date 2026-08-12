---
kind: plan
status: draft
canonical: mydocs/plans/rhwp_studio_hwpctrl_plugin.md
last_verified: 2026-08-12
---

# 구현 명세서 — rhwp-studio 자동화 표면과 HwpCtrl 플러그인

- **상위 계획서**: [`rhwp_studio_hwpctrl_plugin.md`](rhwp_studio_hwpctrl_plugin.md). 설계 결정과
  게이트는 그쪽이 권위다. 이 문서는 **P0 산출물** — 타입, RPC 스키마, 결선 지점, 파일 단위 작업
  목록이다. 설계가 바뀌면 상위 계획서를 먼저 고친다.
- **기록 시각**: 2026-08-12 KST
- **읽는 법**: §1 은 착수 전 결정이 필요한 실측 결과, §2~§6 은 코드 계약, §7 은 작업 분해,
  §8 은 테스트 매트릭스다.

---

## 1. readOnly·chrome 실측 (착수 전 결정 사항)

### 1.1 readOnly — 단일 게이트가 80%, 나머지 20%가 문제

**측정**

| 사실 | 값 |
| --- | --- |
| 뮤테이션 라우터 진입 게이트 | `executeOperation` 첫 줄 `isOperationAllowedInEditMode(desc)` (`engine/input-handler.ts:2625`) |
| 기존 편집 모드 축 | `editMode: 'normal' \| 'form'` — 게이트 본체는 `input-handler.ts:3870` |
| `executeOperation` 호출부 | 138곳 (표 커맨드 24, 키보드 23, 텍스트 19, …) |
| 그중 `kind:'record'` | **27곳** |
| 문서 변경 브리지 메서드 원장 | **131개** (`core/mutation-method-registry.ts`) + 저작 시점 가드 |
| 커맨드 층 게이트 | `dispatcher.ts` 의 `FORM_MODE_BLOCKED_IDS`/`PREFIXES` + `canExecute(ctx.isEditable)` |
| `isEditable` 산출 | `main.ts:141` — `!isFormMode \|\| canEditFormField` |

**핵심**: `kind:'record'` 는 **이미 문서에 적용된** 뮤테이션을 기록만 한다(드래그·리사이즈·표
nudge). 그래서 form mode 게이트도 이것만은 통과시킨다 — `input-handler.ts:3872` 주석이 "여기서
드롭하면 undo 불가한 미기록 편집으로 남는다" 고 명시한다. **readOnly 도 같은 함정을 밟는다**:
게이트 한 줄로 `command`/`snapshot` 은 막히지만, record 계열은 이미 적용된 뒤라 **호출부에서
차단해야** 한다.

**작업량**

| # | 항목 | 규모 |
| --- | --- | --- |
| 1 | `editMode` 에 `'readonly'` 추가 + `isEditable` 반영 (`main.ts:141`) | 소 |
| 2 | `isOperationAllowedInEditMode` 에 readOnly 분기 (`command`/`snapshot` 전부 false) | 소 |
| 3 | record 직접-적용 경로 진입 차단 — mouse 드래그/리사이즈, picture, table nudge | **중** |
| 4 | 키 입력·IME 조합 경로 차단 (`input-handler-keyboard`) | 소~중 |
| 5 | dispatcher readOnly 차단 집합 (form-mode 목록 재사용 + 뮤테이팅 커맨드 전반) | 소 |
| 6 | 툴바·메뉴 비활성 표시 | **없음** — `canExecute` 가 이미 파생 |
| 7 | e2e: 타이핑·붙여넣기·드래그·표 편집이 문서를 바꾸지 않음 | 중 |

**결정**: readOnly 는 "창 제어" 가 아니라 **신규 편집 모드**다. 상위 계획서 §1.2 에서 창 제어처럼
쓴 것을 정정하고, **P4 가 아니라 독립 항목(P4.5)** 으로 뺀다. 브리지 초기 표면에서
`setReadOnly` 는 **미구현으로 두고**, 필요한 소비자는 `chrome:{...}` 숨김 + `commands` 미호출로
갈음한다. 이유: 3·4번이 입력 경로 전반을 건드려 studio 회귀 위험이 이 계획의 다른 어떤 항목보다
크고, hwpctrl 연동의 사용자 가치와 독립이다.

### 1.2 chrome 토글 — 저렴하다

**측정**: 모든 chrome 조각이 안정된 id 를 갖는다 — `#studio-root`, `#studio-header`(메뉴바
`#menu-bar` 포함), `#icon-toolbar`, `#style-bar`, `#status-bar`, `#editor-area`. 그리고
`ViewportManager` 가 이미 `ResizeObserver` 로 편집 영역 크기를 추종한다
(`view/viewport-manager.ts:22,57`).

**결론**: chrome 토글은 `#studio-root` 에 클래스를 얹고 CSS 로 숨기는 것으로 끝난다. 높이 변화의
재배치는 기존 `ResizeObserver` 가 처리한다. **P4 에 그대로 둔다.**

```css
.rhwp-chrome-no-menu   #studio-header { display: none; }
.rhwp-chrome-no-toolbar #icon-toolbar, .rhwp-chrome-no-toolbar #style-bar { display: none; }
.rhwp-chrome-no-status  #status-bar   { display: none; }
```

**주의**: 숨김은 표시만 끈다. 커맨드 레지스트리·단축키는 그대로 살아 있고
`studio.commands.execute` 가 계속 동작한다(상위 계획서 §1.2 의 헤드리스 구성).

---

## 2. 타입 계약

> **P0 구현 시 확정된 두 가지 변경** (2026-08-12, 코드가 권위):
> 1. `CommandResult`/`CommandFailure` 는 `automation/types.ts` 가 아니라 **`command/types.ts`** 에
>    산다. `dispatcher` 가 그 타입을 쓰는데 자동화 층에 두면 `command → automation` 역방향 의존이
>    생긴다. 자동화는 재노출만 한다.
> 2. `PluginHost` 에 **wasm 모듈 네임스페이스를 주지 않는다**. 주면 플러그인이
>    `new HwpDocument(...)` 로 문서를 따로 만들 수 있고, 그 순간 §9.C-d 가 실측한 "두 문서가
>    조용히 갈라지는" 상태가 된다. 문서 생성·교체는 `loadDocument`/`createBlankDocument` 위임뿐이다.

### 2.1 `rhwp-studio/src/automation/types.ts` (신규)

```ts
import type { EditorContext } from '@/command/types';

export interface CommandInfo {
  id: string;
  label: string;
  shortcutLabel?: string;
  icon?: string;
  enabled: boolean;
  opensDialog?: boolean;
}

export type CommandFailure =
  | 'unregistered' | 'disabled' | 'blocked-in-form-mode' | 'needs-dialog' | 'threw';

export type CommandResult =
  | { ok: true }
  | { ok: false; reason: CommandFailure; message?: string };

export interface MenuNode {
  menuId: string;                 // data-menu 값 (file/edit/...)
  label: string;
  items: MenuItemNode[];
}
export interface MenuItemNode {
  commandId?: string;             // data-cmd. 없으면 구분선·비커맨드 항목
  label: string;
  enabled: boolean;
  submenu?: MenuItemNode[];
}

export interface MenuItemSpec {
  menuId: string;
  commandId: string;              // 'ext:' 접두사 필수
  position?: 'top' | 'bottom';
}

export interface StudioAutomation {
  listCommands(): CommandInfo[];        // registry 순회 — 메뉴에 없는 41개 포함
  getMenuModel(): MenuNode[];           // DOM 파생 — 메뉴에 실제로 있는 것만
  isEnabled(id: string): boolean;
  execute(id: string, params?: Record<string, unknown>): CommandResult;
  getContext(): EditorContext;
  registerCommand(def: ExtCommandDef): void;
  unregisterCommand(id: string): void;
  addMenuItem(spec: MenuItemSpec): void;
  removeMenuItem(commandId: string): void;
}
```

`ExtCommandDef` 는 `CommandDef`(`command/types.ts`)에서 `execute` 를 직렬화 가능한 형태로 좁힌
것이다 — 플러그인은 함수를 등록할 수 있지만 **부모 페이지는 등록할 수 없다**(§3.3).

### 2.2 `rhwp-studio/src/plugin/types.ts` (신규)

```ts
export interface DocumentLease {
  readonly handle: HwpDocument;     // wasm-bindgen 객체 — free() 금지
  readonly generation: number;      // WasmBridge._documentGeneration 사본
}

export interface Tx {
  /** 트랜잭션 안에서만 유효한 문서 접근. 무효 lease 면 DOCUMENT_RELEASED 를 던진다. */
  doc(): HwpDocument;
  /** 조판을 미룬다. 트랜잭션 종료 시 한 번에 flush 한다. */
  deferPagination(): void;
}

export interface PluginHost {
  readonly wasm: typeof import('@/core/wasm-bridge');   // studio 가 초기화한 모듈
  borrowDocument(): DocumentLease;
  transaction<T>(label: string, fn: (tx: Tx) => T): T;
  automation: StudioAutomation;
  events: { on(name: string, cb: (payload?: unknown) => void): () => void };
  onDocumentSwap(cb: (lease: DocumentLease) => void): () => void;
  /** 문서 교체 위임 — 플러그인은 직접 new HwpDocument 하지 않는다. */
  loadDocument(bytes: Uint8Array, fileName?: string): void;
  createBlankDocument(): void;
}

export interface StudioPlugin {
  readonly id: string;
  readonly apiVersion: 1;
  activate(host: PluginHost): PluginSurface | Promise<PluginSurface>;
  deactivate?(): void;
}

/** 플러그인이 노출하는 메서드 집합. RPC 는 이 표면만 호출한다. */
export type PluginSurface = Record<string, (...args: unknown[]) => unknown>;
```

### 2.3 등록 원장 (unload 회수의 실체)

호스트는 플러그인별로 다음을 추적하다가 `unload` 시 **역순으로** 철거한다. 플러그인의
`deactivate()` 성실성에 기대지 않는다(상위 계획서 §4.4).

```ts
interface PluginLedger {
  commands: Set<string>;              // registerCommand 로 들어온 ext: id
  menuItems: Set<string>;             // addMenuItem 이 심은 commandId
  eventUnsubs: Array<() => void>;     // events.on 반환값
  swapUnsubs: Array<() => void>;      // onDocumentSwap 반환값
  activeTx: number;                   // 진행 중 트랜잭션 수 — 0 이 될 때까지 unload 대기
}
```

---

## 3. RPC 스키마 (embed protocol v1 + capability)

### 3.1 capability

기존 배열(`embed/protocol.ts:2`)에 추가: `automation-v1`, `plugin-loader-v1`, `hwpctrl-v1`,
`event-subscription-v1`, `chrome-v1`. **`EMBED_PROTOCOL_VERSION` 은 1 을 유지한다.**

### 3.2 메서드

| method | params | result | 오류 |
| --- | --- | --- | --- |
| `automation.listCommands` | – | `CommandInfo[]` | – |
| `automation.getMenuModel` | – | `MenuNode[]` | – |
| `automation.isEnabled` | `{id}` | `boolean` | – |
| `automation.execute` | `{id, params?}` | `CommandResult` | `INVALID_REQUEST` |
| `automation.getContext` | – | `EditorContext` | – |
| `automation.addMenuItem` | `MenuItemSpec` | `{ok:true}` | `EXT_PREFIX_REQUIRED`, `UNKNOWN_MENU` |
| `automation.removeMenuItem` | `{commandId}` | `{ok:true}` | – |
| `plugin.list` | – | `{id, apiVersion, active}[]` | – |
| `plugin.load` | `{id}` | `{id, methods:string[]}` | `PLUGIN_NOT_ALLOWED`, `PLUGIN_ACTIVATE_FAILED` |
| `plugin.unload` | `{id}` | `{ok:true}` | `PLUGIN_NOT_LOADED`, `TX_TIMEOUT` |
| `hwpctrl.invoke` | `{method, args}` | `unknown` | `PLUGIN_NOT_LOADED`, `DOCUMENT_RELEASED`, `RPC_ERROR` |
| `hwpctrl.batch` | `{ops:[{m,a}], label?}` | `unknown[]` | 위와 동일 |
| `events.subscribe` | `{name}` | `{ok:true}` | `UNKNOWN_EVENT` |
| `events.unsubscribe` | `{name}` | `{ok:true}` | – |
| `chrome.set` | `{menu?, toolbar?, statusbar?}` | `{applied}` | – |
| `chrome.get` | – | `{menu, toolbar, statusbar}` | – |
| `session.teardown` | – | `{ok:true}` | – |

- **바이너리**: `hwpctrl.invoke` 의 결과가 `Uint8Array` 면 기존 `postPortResponse`
  (`embed/runtime.ts`)가 이미 transferable 로 넘긴다. 새 배선 불필요.
- **타임아웃**: 일반 10s / `hwpctrl.batch` 60s (`npm/editor` 기본값 계승).
- **이벤트 역방향**: studio → 부모는 `{type:'rhwp-event', name, payload}` 를 같은 포트로 보낸다.
  **구독자가 없으면 보내지 않는다**(상위 계획서 §6 게이트).

### 3.3 신뢰 경계에서 잘라내는 것

부모는 **함수를 넘길 수 없다**. 따라서 `automation.registerCommand` 는 **RPC 표면에 없다** —
`ext:` 커맨드 등록은 iframe 안의 플러그인만 할 수 있다. 부모가 자기 코드를 실행시키려면 플러그인을
만들어 `plugin.load` 로 올려야 하고, 그 목록은 호스트가 정한 allowlist 다.

---

## 4. `transaction` 결선 — 기존 undo 기구에 얹는다

`SnapshotCommand`(`engine/command.ts:2006`)의 계약이 이미 필요한 것을 전부 갖고 있다.

- `operation(wasm) => DocumentPosition | null` — **`null` 이면 무변경으로 기록 자체를 취소**한다
  (Task #2370). 아무것도 안 바꾼 배치가 undo 스택을 더럽히지 않는다.
- `execute` 가 before 스냅샷을 뜬 뒤 operation 을 돌리고, **throw 하면 before 로 자동 롤백**한다
  (#3350). 즉 **배치 실패 시 원자성은 이미 보장된다** — 새로 만들 필요가 없다.

```ts
// plugin/host.ts
transaction<T>(label: string, fn: (tx: Tx) => T): T {
  let result!: T;
  this.ledger.activeTx += 1;
  try {
    this.inputHandler.executeOperation({
      kind: 'snapshot',
      operationType: `plugin:${this.pluginId}:${label}`,
      operation: (wasm) => {
        const before = wasm.getDocumentDigest?.();     // 무변경 판정용
        result = fn(this.makeTx(wasm));
        if (wasm.getDocumentDigest?.() === before) return null;   // → 기록 취소
        return this.cursor.getPosition();
      },
    });
  } finally {
    this.ledger.activeTx -= 1;
  }
  return result;
}
```

- **조판**: `Tx.deferPagination()` 이 호출되면 operation 안에서는
  `beginDeferredPagination`/`step` 만 돌리고, `executeOperation` 이 끝낸 뒤
  `flushDeferredPagination` → dirty page 부분 repaint 로 마무리한다. 배치 = 재조판 1회.
- **무변경 판정**: 위 스케치는 digest 비교를 쓴다. `WasmBridge` 에 그 값이 없으면 **P2 에서
  판정 방식을 확정**한다(후보: 페이지 수 + 문단 수 + 뮤테이터 호출 여부 플래그). 이 결정은 P2 의
  첫 작업이다.
- **중첩 금지**: 트랜잭션 안에서 다시 `transaction` 을 부르면 `NESTED_TX` 로 던진다. 스냅샷이
  이중으로 쌓여 undo 1스텝 계약이 깨진다.

---

## 5. hwpctrl 어댑터

### 5.1 `DocumentAdapter` (패키지 내부, studio 무의존)

```js
// npm/hwpctrl-ocx/src/adapter.mjs
export const StandaloneAdapter = {
  doc(state) { return state.doc; },
  replace(state, bytes) { state.doc = new state.wasm.HwpDocument(bytes); },
  mutate(state, label, fn) { return fn(state.doc); },   // 오늘 동작 그대로
};
```

plugin 어댑터는 같은 세 메서드를 host 로 위임한다 — `doc()` 은 lease 검사 후 핸들,
`replace()` 는 `host.loadDocument(bytes)`, `mutate()` 는 `host.transaction(label, tx => fn(tx.doc()))`.
`index.mjs` 본체는 어댑터만 부르고 **둘 중 어느 쪽인지 모른다**. 이것이 원장 312/484 를 한 벌로
유지하는 방법이다.

### 5.2 lease 검사 (§9.C 실측 대응)

```js
function docOf(state) {
  const { handle, generation } = state.lease;
  if (generation !== state.host.currentGeneration() || handle.__wbg_ptr === 0) {
    const e = new Error('문서가 이미 해제되었습니다'); e.code = 'DOCUMENT_RELEASED'; throw e;
  }
  return handle;
}
```

**금지**: plugin 모드에서 `this.#doc?.x?.()` 형태의 optional-chaining 삼킴. 실측에서 해제된 문서에
대한 `PageCount()` 가 예외 없이 `0` 을 돌려줬다. 어댑터 경유 호출은 **삼키지 않는다**.

### 5.3 커서 좌표 변환기 (§9.D 실증 규칙)

```js
// listId → { sectionIndex, parentParaIndex, cellPath }
export function listToStudio(model, listId) {
  const byId = new Map((model.lists ?? []).map((l) => [l.listId, l]));
  const chain = [];
  for (let cur = byId.get(listId), g = 0; cur && g < 64; g += 1) {
    chain.unshift(cur);
    if (cur.hostListId === 0) break;
    cur = byId.get(cur.hostListId);
  }
  if (!chain.length) return null;
  return {
    sectionIndex: chain[0].sectionIndex,
    parentParaIndex: chain[0].hostPara,
    cellPath: chain.map((c, i) => ({
      controlIndex: c.controlIndex,
      cellIndex: c.cellIndex,
      // 자식 표가 놓인 부모 셀 안의 문단 번호. 0 으로 고정하면 중첩에서 틀린다.
      cellParaIndex: i + 1 < chain.length ? chain[i + 1].hostPara : 0,
    })),
  };
}
```

역방향(`studioToList`)은 같은 모델을 `(sectionIndex, hostPara, controlIndex, cellIndex)` 로
역인덱싱한다. **모델은 문서가 바뀌면 버린다** — hwpctrl 이 이미 `#listModel` 캐시를 문서 교체 시
비운다. 회귀 픽스처: `samples/table-001.hwp`(단층 131셀),
`samples/issue1949_giant_cell_nested_tables_perf.hwp`(깊이 2, 198리스트).

---

## 6. 오류 코드

| code | 의미 | 발생처 |
| --- | --- | --- |
| `PLUGIN_NOT_LOADED` | 해당 플러그인이 안 올라와 있음 | RPC 라우터 |
| `PLUGIN_NOT_ALLOWED` | allowlist 밖 | 플러그인 호스트 |
| `PLUGIN_ACTIVATE_FAILED` | `activate` 예외 — studio 는 계속 산다 | 플러그인 호스트 |
| `DOCUMENT_RELEASED` | lease 무효 (§5.2) | 어댑터 |
| `NESTED_TX` | 트랜잭션 중첩 | 호스트 |
| `TX_TIMEOUT` | unload 가 트랜잭션을 기다리다 만료 → 롤백 | 호스트 |
| `EXT_PREFIX_REQUIRED` / `UNKNOWN_MENU` | 메뉴 조작 인자 오류 | 자동화 |
| `UNSUPPORTED_VERSION` / `INVALID_REQUEST` / `RPC_ERROR` | 기존 코드 그대로 | embed runtime |

---

## 7. 파일 단위 작업 목록

### P0 — 계약 (런타임 변경 0)

| 파일 | 작업 |
| --- | --- |
| `src/automation/types.ts` | 신규 — §2.1 |
| `src/plugin/types.ts` | 신규 — §2.2 |
| `src/command/dispatcher.ts` | `dispatchWithResult(): CommandResult` 추가. 기존 `dispatch(): boolean` 유지 |

### P1 — 자동화 호스트

| 파일 | 작업 |
| --- | --- |
| `src/automation/host.ts` | 신규 — registry 순회, DOM 파생 메뉴 모델, dispatcher 위임 |
| `src/automation/menu-dom.ts` | 신규 — `.menu-item`/`.md-sub`/`.md-item[data-cmd]` 스캔·삽입·제거 |
| `src/command/extension-api.ts` | **삭제** — 자동화 호스트로 흡수 |
| `src/main.ts` | `window.rhwpStudio` 확장(이미 있는 전역, `main.ts:94`)에 `automation` 부착 |
| `tests/automation-registry-drift.test.ts` | 신규 — 런타임 registry ↔ 마크업 `data-cmd` 대조 0 |
| `e2e/automation-commands.test.mjs` | 신규 — 질의·실행·활성 판정 |

### P2 — 플러그인 호스트

| 파일 | 작업 |
| --- | --- |
| `src/plugin/host.ts` | 신규 — lease, transaction(§4), 등록 원장(§2.3), load/unload |
| `src/core/wasm-bridge.ts` | `currentGeneration()` 노출(읽기 전용). `doc` 은 private 유지 |
| `src/core/mutation-method-registry.ts` | 플러그인 경유 뮤테이션 분류 추가 |
| `e2e/plugin-lifecycle.test.mjs` | 신규 — 더미 플러그인 load↔unload 잔여 0, 트랜잭션 1스냅샷 |

### P3 — hwpctrl 플러그인

| 파일 | 작업 |
| --- | --- |
| `npm/hwpctrl-ocx/src/adapter.mjs` | 신규 — §5.1 |
| `npm/hwpctrl-ocx/src/cursor-map.mjs` | 신규 — §5.3 |
| `npm/hwpctrl-ocx/src/studio-plugin.mjs` | 신규 — `StudioPlugin` 구현. studio import 금지 |
| `npm/hwpctrl-ocx/src/index.mjs` | 소유·교체·뮤테이션을 어댑터 경유로 전환 |
| `npm/hwpctrl-ocx/package.json` | `exports` 에 `./studio-plugin` 추가 |
| `npm/hwpctrl-ocx/test/adapter_parity.test.mjs` | 신규 — 두 모드 산출 바이트 동일 |
| `npm/hwpctrl-ocx/test/cursor_map.test.mjs` | 신규 — 단층·중첩 픽스처 왕복 |
| `e2e/hwpctrl-plugin.test.mjs` | 신규 — PutFieldText → 화면 반영 → exportHwp, unload 생존 |

### P4 — 브리지·SDK

| 파일 | 작업 |
| --- | --- |
| `src/embed/protocol.ts` | capability 5종 추가 |
| `src/embed/rpc-router.ts` | §3.2 메서드 라우팅 |
| `src/main.ts` | `installEmbedRuntime`(`main.ts:1442`) handlers 에 신규 핸들러 추가 |
| `src/automation/chrome.ts` | 신규 — `#studio-root` 클래스 토글 (§1.2) |
| `src/style.css` | `.rhwp-chrome-no-*` 규칙 |
| `npm/editor/index.js` | `createStudio` + `commands`/`hwpctrl`/`plugins`/`chrome` 프록시, 배치 기록 프록시 |
| `npm/editor/index.d.ts` | 타입 |
| `e2e/bridge-lifecycle.test.mjs` | 신규 — create↔destroy 100회, 배치 1왕복 |

### P4.5 — readOnly (독립, §1.1)

착수 여부는 별도 결정. `input-handler`(게이트+record 경로), `dispatcher`, `main.ts` 컨텍스트,
전용 e2e.

### P5/P6 — 성능·마감

상위 계획서 §8 그대로.

---

## 7.5 게이트 러너 — 한 명령으로 전부

검증이 여섯 군데에 흩어져 있으면(타입·studio 단위·패키지 단위·e2e 5종·번들·hwpctrl gate)
"무엇까지 돌렸는지" 가 사람 기억에 남고, 기억은 회귀를 놓친다.

```bash
cd rhwp-studio
npm run gate:bridge                    # 전부 (약 52초)
npm run gate:bridge:quick              # 브라우저 없이 (타입·단위·번들)
npm run gate:bridge -- --only=e2e      # e2e 만
CHROME_EXTRA_ARGS='--js-flags=--expose-gc' npm run gate:bridge   # 힙 측정 정밀
```

러너(`scripts/gate_bridge.mjs`)가 하는 일 중 사람이 자주 틀리는 것 하나 — **dev 서버 수명 관리**다.
손으로 띄우면 껐다 켜는 것을 잊어 낡은 번들에 대고 통과하는 일이 생긴다. 러너는 띄우고 반드시
내린다. 이미 떠 있으면 **남의 서버를 내리지 않고** 그대로 쓰되 "낡은 번들일 수 있다" 고 알린다.

산출 예:

```
  ok   tsc (main, ci-unit)                OK
  ok   studio 단위                          838 pass / 0 fail
  ok   hwpctrl 패키지 단위                     21 pass / 0 fail (파일 5종)
  ok   e2e automation-commands            23 PASS / 0 FAIL
  ...
  ok   build + 플러그인 청크 분리                 studio-plugin 청크 54.38kB
  ok   hwpctrl standalone gate            시나리오 101건 OK

  결과: 통과 (10단계, 52s)
```

실패하면 종료 코드 1 과 함께 실패 단계를 이름으로 알리고, 그 단계의 마지막 출력을 붙인다
(일부러 깨뜨려 확인함). 개별 e2e 는 `npm run e2e:automation` 처럼 따로도 돌릴 수 있고,
그 배선은 `e2e/MANIFEST.md` 가 원장으로 검사한다.

## 8. 테스트 매트릭스

| 판정 | 방법 | Phase |
| --- | --- | --- |
| 커맨드 질의·실행 | e2e | P1 |
| registry ↔ 마크업 드리프트 0 | 런타임 단위 테스트 | P1 |
| 배치 = undo 1스텝 | e2e (undo 1회로 전체 복원) | P2 |
| 배치 = 조판 1회 | 계측 훅 카운터 | P5 |
| load↔unload 잔여 0 | e2e (커맨드·메뉴·리스너 수) | P2 |
| unload 후 studio 생존 6항목 | e2e (상위 계획서 §7-6) | P3 |
| 해제 문서 호출이 예외 | 단위 테스트 | P3 |
| 좌표 변환 단층·중첩 | 단위 테스트 (픽스처 2종) | P3 |
| 두 모드 산출 동일 | 패키지 테스트 | P3 |
| standalone gate 무회귀 | `npm --prefix npm/hwpctrl-ocx run gate` | 전 Phase |
| Node 스모크 (DOM 없음) | 패키지 테스트 | P3 |
| studio 정적 참조 금지 | 소스 가드 | P3 |
| 코어 → 플러그인 import 금지 | 소스 가드 | P2 |
| 플러그인 없는 기존 e2e 전량 | 기존 스위트 | 전 Phase |
| create↔destroy 100회 힙 | e2e + `performance.measureUserAgentSpecificMemory` 또는 CDP | P4 |

---

## 9. 이 명세가 확정하지 못한 것

1. **무변경 판정 수단** (§4) — digest 비교가 가능한지 `WasmBridge` 실측 필요. P2 첫 작업.
2. **`unload` 대기 타임아웃 값** — `activeTx` 가 0 이 되기를 기다리는 상한. P2 에서 실측.
3. **`InitScan`/`GetTextFile` 의 표 텍스트 포함 여부** (상위 §9.D 미확증). P3 에서 확인.
4. **다이얼로그 정책 원장** — `opensDialog` 표기를 커맨드 정의에 넣고 소스 가드로 강제하는 작업.
   P6.
5. **다중 인스턴스 wasm 공유** — 기본은 공유하지 않음. 비용만 계측해 README 에 기록.
