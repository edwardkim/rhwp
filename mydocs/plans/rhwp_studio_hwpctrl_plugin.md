---
kind: plan
status: draft
canonical: mydocs/plans/rhwp_studio_hwpctrl_plugin.md
last_verified: 2026-08-12
---

# 수행계획서 — rhwp-studio 자동화 표면과 HwpCtrl 플러그인

- **이슈**: 미발행 (착수 승인 시 발행)
- **브랜치**: `feat/studio-automation-plugin` (Phase 별 하위 브랜치)
- **기록 시각**: 2026-08-11 KST
- **base**: `stream/devel`
- **선행 계획서**: [`hwpctrl_ocx_full_compat.md`](hwpctrl_ocx_full_compat.md) — 이 계획서는 그 계획의
  P7(studio 이관)을 **앞당기지 않고**, 이관이 가능해지는 **배선**만 먼저 세운다.
- **목표**: 웹 페이지가 JavaScript 만으로 ① 지정한 `div` 안에 rhwp-studio 를 띄우고 그 창을
  제어하며 ② studio 가 들고 있는 **그 문서**를 HwpCtrl API 로 조작하고 ③ studio 의 메뉴/커맨드를
  호출·질의·변경할 수 있게 한다. **HTML 에 미리 심어두는 것이 아니라 JS 에서 구동**한다.
- **불변 조건 (양방향 독립)**: ① 플러그인을 싣지 않은 studio 는 **오늘과 완전히 같이** 동작하고
  코어는 플러그인을 정적으로 참조하지 않는다. ② `@rhwp/hwpctrl` 은 studio 없이도 **오늘처럼**
  문서를 다룬다(브라우저·Node 단독). 어느 쪽도 상대를 필요로 하지 않는다.

---

## 0. 결론 먼저

1. **문서는 한 벌만 존재해야 한다.** 지금 두 층은 각자 문서를 만든다 —
   studio 는 `WasmBridge.doc`(`rhwp-studio/src/core/wasm-bridge.ts:256`), `@rhwp/hwpctrl` 은
   `#doc = new wasm.HwpDocument(bytes)`(`npm/hwpctrl-ocx/src/index.mjs:1078`). 둘을 그냥 나란히
   쓰면 **IR 2벌 + 매 왕복마다 직렬화·재파싱**이다. 설계의 중심은 API 목록이 아니라
   **둘이 함께 있을 때 문서 소유권을 studio 하나로 못 박고 hwpctrl 을 차용자로 만드는 것**이다.
   `HwpCtrl` 생성자에 이미 있는 `doc` 주입 자리(`index.mjs:1036`)를 정식 계약으로 승격한다.
   **단, 이것은 결합 모드의 규칙일 뿐이다** — studio 가 없으면 hwpctrl 은 지금처럼 자기 문서를
   소유한다(§4.0). 의존은 **양방향 모두 없다**.
2. **플러그인은 iframe **안**에서 산다.** WASM 문서 핸들은 wasm 인스턴스 경계를 못 넘는다.
   부모 페이지가 직접 hwpctrl 을 들면 zero-copy 가 원천적으로 불가능하다. 따라서
   부모는 **RPC 로 조종만** 하고, hwpctrl 은 studio 와 같은 페이지·같은 wasm 인스턴스에서 돈다.
3. **hwpctrl 의 조작도 studio 의 undo 를 통과해야 한다.** studio 의 undo 는
   `executeOperation → CommandHistory` 경유가 전제이고, 미기록 뮤테이션은
   "undo 불가 · redo 스택 미무효화 · 스냅샷 복원에 동반 파괴" 3중 위험을 만든다
   (`src/core/mutation-method-registry.ts` 서두, #2027/#2037/#2053/#2077 재발 계급).
   플러그인이 doc 을 직접 만지면 이 계급이 그대로 되살아난다. **모든 hwpctrl 뮤테이션은
   studio 가 여는 트랜잭션 안에서만** 실행한다.
4. **메뉴 조작의 진입점은 이미 있다.** `CommandRegistry`/`CommandDispatcher` 가 메뉴·툴바·키보드의
   단일 실행 경로다(`src/command/dispatcher.ts:31`). `StudioExtensionAPI`
   (`src/command/extension-api.ts`)도 있으나 **어디서도 인스턴스화되지 않는 죽은 코드**이고
   메뉴 조작이 `querySelector('.md-item')` DOM 직조작이다. 이 층은 **데이터 모델 기반으로 다시
   세운다**.
5. **성능의 승부처는 API 하나하나가 아니라 커밋 단위다.** 호출 1건마다 재조판·재렌더하면
   `PutFieldText` 100 번이 100 번의 전체 조판이다. **배치 = 트랜잭션 = undo 1스텝 = 재조판 1회**로
   묶고, 조판은 이미 있는 `beginDeferredPagination`/`step`/`flush` 를 태운다.

6. **"plugin bridge" 는 두 개념이 붙어 있는 이름이다.** 부모↔studio 를 잇는 **통로**와 studio 안에
   얹히는 **기능 확장**은 수명도 신뢰 경계도 다르다. 하나로 부르면 "브리지를 끄면 플러그인도
   꺼지나?" 같은 질문에 답이 없다. 아래 §0.1 처럼 **Bridge / Plugin 을 분리**해 부른다.

### 0.1 명명 (권장안)

| 개념 | 권장 표현 | 코드 | 이유 |
| --- | --- | --- | --- |
| 부모 페이지가 쥐는 studio 인스턴스 제어권 | **Studio Bridge** | `createStudio()` → `StudioBridge` | "다리"는 경계를 넘는 통로라는 뜻만 갖는다. 기능을 암시하지 않는다 |
| 그 통로가 나르는 것 | Bridge **channel**(RPC) | `embed/` 유지 | 기존 embed 프로토콜의 확장이지 새 물건이 아니다 |
| studio 안의 확장 접수처 | **Plugin Host** | `src/plugin/host.ts` | 호스트/게스트 관계가 이름에 드러난다 |
| 얹히는 기능 | **Plugin** (`hwpctrl` plugin) | `@rhwp/hwpctrl/studio-plugin` | 무엇이 얹혔는지로 부른다 |
| 커맨드·메뉴 조작 표면 | **Automation** | `studio.commands.*` | 문서 조작(hwpctrl)과 UI 조작을 이름으로 가른다 |

- 대안으로 검토한 것: `Controller`(무엇을 제어하는지 안 드러남), `Host API`(주체가 뒤집혀 혼동),
  `Automation Bridge`(Bridge 가 자동화 전용이라는 오해 — Bridge 는 로드·창 제어도 나른다).
- 한국어 표기: **브리지 / 플러그인 / 자동화**. 문서에서 "플러그인 브리지"라는 결합 표현은 쓰지
  않는다.

**hwpctrl 은 bridge 가 아니라 plugin 이다.** 판정 기준은 수명이다.

| | Bridge | Plugin |
| --- | --- | --- |
| 무엇 | 부모 페이지 ↔ studio 인스턴스를 잇는 **통로** | studio 위에 얹혔다 내려가는 **기능** |
| 수명 | 인스턴스와 함께 나고 죽는다 (`createStudio`~`destroy`) | 인스턴스 도중에 **붙고 떨어진다** |
| 없으면 | 외부에서 조종할 수 없을 뿐, studio 는 온전하다 | hwpctrl API 만 사라지고, studio 는 온전하다 |
| 개수 | 인스턴스당 1 | 0..N |

`Bridge` 는 "끊으면 통신이 끊긴다"를, `Plugin` 은 "빼면 기능이 빠진다"를 뜻한다. 요구가
**"hwpctrl 을 종료해도 studio 는 살아 있어야 한다"** 이므로 이름은 `Plugin` 이 맞다 — 그 독립성이
바로 플러그인이라는 낱말이 약속하는 것이다. `unload` 를 계약에 넣는 것도 플러그인이기 때문이다
(§4.4). 반대로 통로를 `Plugin` 이라 부르면 "통로를 빼도 되나"가 미결로 남는다.

---

## 1. 최종 그림

### 1.1 계층

```
[부모 웹페이지]  <div id="app">          [studio 인스턴스 = 그 div 안의 iframe]
  const s = await createStudio('#app')     bridge runtime (rpc-router)
    s.commands.execute('edit:copy') ─┐   │
    s.hwpctrl.PutFieldText(...)      ├──▶├─ automation host ─▶ CommandRegistry/Dispatcher
    s.batch([...])                   │   │        │
    s.on('command-state-changed')  ◀─┘   │        └─▶ plugin host ─▶ @rhwp/hwpctrl/studio-plugin
                                         │                              │ (doc 핸들 차용)
    s.setChrome({menu:false}) ──────────▶└─ WasmBridge.doc  ◀───────────┘  뮤테이션은 트랜잭션 경유
    s.destroy()
```

### 1.2 구동과 창 제어 (JS 진입점)

`@rhwp/editor` 의 `createEditor('#editor')` 가 이미 "선택자에 iframe 을 심는" 일을 한다. 이
계획은 그것을 **인스턴스 수명과 창 제어를 갖춘 브리지**로 넓힌다. HTML 에 `<iframe>` 을 미리
써두는 방식은 지원 대상이 아니다 — 모든 것이 JS 에서 시작한다.

> **마운트는 이동이 아니다 (§9.B 실측).** iframe 을 다른 요소로 DOM 이동시키면
> 브라우저가 **문서를 재로드**한다 — 실측에서 `state:'EDITED'` → `'INITIAL'`, load 2회,
> `MessagePort` 사망. 같은 부모로의 재삽입도, 조상 wrapper 를 옮기는 것도 마찬가지다.
> 그래서 `mount` 는 **최초 1회 지정**이고, 화면상의 이동·숨김은 **컨테이너 CSS**로 한다
> (실측: `display:none` 토글과 위치·transform 변경은 재로드 없음, 상태 보존).

```js
import { createStudio } from '@rhwp/editor';

const studio = await createStudio('#app', {        // 또는 HTMLElement 직접 전달
  plugins: ['hwpctrl'],
  chrome: { menu: true, toolbar: true, statusbar: false },
  zoom: 1.0,
  autoResize: true,                                 // 컨테이너 크기 추종 (ResizeObserver)
});

studio.setChrome({ menu: false }); // 메뉴바 숨김 — 제어는 JS 로만
studio.resize();                   // autoResize:false 일 때 수동 반영
studio.setZoom(1.5);
studio.focus(); studio.blur();
await studio.destroy();            // iframe 제거 + wasm 문서 해제 + 리스너 해제
```

**계약**

- **컨테이너 소유권**: 브리지는 넘겨받은 요소의 **자식만** 만든다. 요소 자체의 스타일·클래스를
  바꾸지 않는다(레이아웃은 호스트 페이지의 책임). iframe 은 `width/height:100%` 로 채운다.
- **이동 대신 재생성**: 정말로 다른 컨테이너로 옮겨야 하면 `destroy()` → `createStudio(새 div)` →
  문서 재적재다. 브리지는 이 왕복을 **숨기지 않는다** — `mount('#other')` 같은 API 를 두면
  "상태가 유지될 것" 이라는 잘못된 기대를 판다.
- **다중 인스턴스**: 한 페이지에 여러 studio 를 띄울 수 있다. 인스턴스마다 별도 iframe =
  별도 wasm = **별도 메모리**다. 이 비용은 §6 게이트에 명시하고, 문서 비교처럼 실제로 두 벌이
  필요한 경우 외에는 한 인스턴스에 문서를 갈아끼우도록 안내한다.
- **`destroy()` 는 반드시 회수한다**: iframe 제거, `MessagePort` close, `ResizeObserver` 해제,
  studio 쪽 `releaseDocument()`. 마운트/파괴 100회 반복에서 힙이 자라지 않아야 한다(§6).
- **읽기 전용은 이 표면에 없다.** 실측 결과 studio 에 읽기 전용 모드 자체가 없고(`isEditable` 은
  양식 모드 파생), 만드는 일은 입력 경로 전반을 건드리는 **신규 편집 모드**다. 구현 명세서 §1.1 에서
  독립 항목(P4.5)으로 분리했다.
- **창 제어와 자동화의 경계**: 크기·표시·줌처럼 **문서를 바꾸지 않는 것**은 브리지의
  창 제어이고, 문서를 바꾸는 것은 자동화·hwpctrl 이다. 줌은 `view:zoom-*` 커맨드와 같은 내부
  경로를 태워 UI 상태가 갈리지 않게 한다.
- **`chrome:{menu:false}`** 는 메뉴를 **숨기기만** 한다. 커맨드 레지스트리는 그대로 살아 있어
  `studio.commands.execute` 가 계속 동작한다 — "UI 없는 헤드리스 편집기" 구성이 이것으로 선다.
- **기존 `createEditor` 는 유지한다.** `createStudio` 는 그 상위 집합이고, `createEditor` 는
  얇은 래퍼로 남겨 기존 소비자를 깨지 않는다.

- **같은 오리진 단일 페이지**로 쓰는 경우(스튜디오를 직접 호스팅)에도 같은 API 를
  `window.rhwp` 로 노출한다. RPC 계층만 건너뛴다 — 표면은 하나다.
- 부모가 넘기는 값은 **구조화 복제 가능한 데이터만**. 콜백·함수는 넘기지 않는다(이벤트는 구독으로).

## 2. 계층과 산출물

| # | 층 | 위치 | 새로 만드나 |
| --- | --- | --- | --- |
| L1 | 커맨드 자동화 호스트 | `rhwp-studio/src/automation/` | 신규 (extension-api.ts 흡수·재작성) |
| L2 | 플러그인 호스트 | `rhwp-studio/src/plugin/` | 신규 |
| L3 | HwpCtrl 플러그인 | `npm/hwpctrl-ocx/src/studio-plugin.mjs` | 신규 (기존 패키지에 subpath export) |
| L4 | embed RPC 확장 | `rhwp-studio/src/embed/{protocol,rpc-router}.ts` | 기존 확장 |
| L5 | 부모 SDK (Studio Bridge) | `npm/editor/` | 기존 확장 (`createEditor` → `createStudio`) |
| L6 | 창 제어 수신부 | `rhwp-studio/src/automation/chrome.ts` | 신규 (chrome 토글·zoom 적용) |

`rhwp-studio/src/hwpctl/`(구 studio 내장 층)은 **이 계획에서 손대지 않는다.** 선행 계획서 §6.2 의
동결 대상이다. 새 배선이 서면 그 층 철거가 "제거 + 리다이렉트"로 끝난다.

## 3. L1 — 커맨드 자동화 호스트

### 3.1 표면

```ts
interface StudioAutomation {
  listCommands(): CommandInfo[];              // {id, label, shortcutLabel?, icon?, enabled}
  isEnabled(id: string): boolean;
  execute(id: string, params?: Record<string, unknown>): CommandResult;
  getContext(): EditorContext;                // command/types.ts 의 스냅샷 그대로
  // 메뉴 모델
  getMenuModel(): MenuNode[];
  addMenuItem(spec: MenuItemSpec): void;      // {menuId, commandId, position, after?}
  removeMenuItem(commandId: string): void;
  registerCommand(def: ExtCommandDef): void;  // id 는 'ext:' 접두사 강제 (기존 규칙 승계)
  unregisterCommand(id: string): void;
}
```

- `execute` 는 `CommandDispatcher.dispatch` 를 그대로 태운다. 즉 **양식 모드 차단
  (`isBlockedInFormMode`)·`canExecute` 게이트가 자동화에도 동일하게 걸린다.** 자동화가 UI 보다
  더 많은 것을 할 수 있게 만들지 않는다 — 우회로를 하나 더 내면 그게 곧 버그의 출처다.
- 반환은 예외가 아니라 판정이다: `{ ok: true } | { ok: false, reason: 'unregistered'|'disabled'|'blocked-in-form-mode'|'threw', message }`.
  지금 `dispatch` 는 넷을 전부 `false` 로 뭉갠다 — 자동화에서는 구분이 필요하므로 `dispatch` 에
  판정 반환 오버로드를 추가하되 기존 `boolean` 호출부는 그대로 둔다.

### 3.2 메뉴 모델 — 실측에 따른 범위 (§9-1 해소)

**메뉴의 선언은 `index.html` 에 있다**(실측: 최상위 `.menu-item` 8, `.md-item` 197, 서브메뉴
`.md-sub` 32, 고유 `data-cmd` 140). `ui/menu-bar.ts` 는 모델이 아니라 **컨트롤러**다 —
`querySelectorAll('.menu-item')` 로 주워 이벤트를 걸고, 레이블·단축키는
`syncMenuShortcutLabels(container, registry)` 로 registry 에서 채우며, 클릭은
`data-cmd` → `dispatcher.dispatch`, 활성 상태는 열릴 때 `dispatcher.isEnabled` 로 갱신한다.
즉 **구조는 HTML, 의미는 registry** 로 이미 갈려 있다.

따라서 "모델화"는 전부-아니면-전무가 아니다. 세 갈래로 쪼갠다.

| 기능 | 방법 | Phase |
| --- | --- | --- |
| `listCommands()` | **registry 순회** (DOM 무관) | P1 |
| `getMenuModel()` | **DOM 파생** — `.menu-item`/`.md-sub`/`.md-item[data-cmd]` 스캔 | P1 |
| `execute`/`isEnabled` | dispatcher 재사용 | P1 |
| `addMenuItem`/`removeMenuItem` | 현행 DOM 삽입 + **호스트 등록 원장**(§4.4 회수 대상) | P1 |
| `index.html` 197 항목 → 선언 데이터 이관 | 렌더러 재작성 | **P6, 선택** |

- **`listCommands()` 와 `getMenuModel()` 은 같지 않다.** 실측상 registry 커맨드 중 **41개가 메뉴
  마크업에 없다**(`field:edit`, `format:char-spacing-*` 등 — 컨텍스트 메뉴·단축키 전용).
  자동화는 메뉴에 없는 커맨드도 실행할 수 있어야 하므로 두 표면을 분리한다.
- 반대 방향 드리프트(마크업의 `data-cmd` 가 registry 에 없음)는 정적 대조에서 **0** 이었다
  (미해결로 보인 25건은 전부 `view:zoom-${pct}` 같은 헬퍼 생성 id). 다만 정적 대조로는
  헬퍼 인자까지 못 따라가므로, **런타임 registry 대조 가드**를 P1 에 넣어 이 0 을 고정한다.
- `index.html` 이관을 P6 으로 미루는 이유: 질의·실행·`ext:` 추가라는 사용자 가치가 이관 없이
  전부 서고, 197 항목 이관은 그 자체로 UI 회귀 위험이 큰 별개 작업이다.

### 3.3 다이얼로그 정책

`file:open`·`format:char-shape` 처럼 다이얼로그를 여는 커맨드는 자동화에서 무한 대기가 된다.
커맨드 정의에 `opensDialog: true` 를 표기하고 자동화 호출에 정책을 둔다.

| 정책 | 동작 |
| --- | --- |
| `reject`(기본) | `{ok:false, reason:'needs-dialog'}` — 부모가 대체 API 를 쓰게 강제 |
| `params` | `params` 로 다이얼로그 결과를 대입해 즉시 적용 (다이얼로그별 `dialog-apply` 경유) |
| `interactive` | 실제로 띄운다. 부모는 사용자 조작 완료 이벤트를 기다린다 |

이 표기는 **원장이 필요하다** — 커맨드 정의에 필드를 추가하고 `tests/` 에 "정의에 없는 커맨드가
다이얼로그를 연다" 를 잡는 소스 가드를 붙인다(`mutation-method-registry` 와 같은 방식).

## 4. L2/L3 — 플러그인 호스트와 HwpCtrl 플러그인

### 4.0 두 실행 모드 — studio 없이도 hwpctrl 은 문서를 다룬다

독립은 양방향이다. studio 는 hwpctrl 없이 살고(§7), **hwpctrl 도 studio 없이 산다**. 이것은 새로
만드는 성질이 아니라 **오늘 패키지가 이미 가진 성질이고, 이 계획이 깨뜨리지 말아야 할 것**이다.

| | **standalone 모드** (오늘) | **plugin 모드** (신규) |
| --- | --- | --- |
| 진입점 | `createHwpCtrl({ wasm, onSave, ... })` | `activate(host)` — `@rhwp/hwpctrl/studio-plugin` |
| 문서 소유 | **hwpctrl 자신** (`new wasm.HwpDocument(bytes)`) | studio (차용) |
| 화면 | 없음. 필요하면 `CreatePageImage` 로 호스트가 그림 | studio 가 그린다 |
| undo | hwpctrl 자체 스냅샷 | studio `CommandHistory` 트랜잭션 |
| `Open`/`Clear` | 자기 `#doc` 교체 | studio `loadDocument` 로 위임 |
| `SaveAs` | `onSave` 훅 | `onSave` 훅 → 브리지 |
| 실행 환경 | 브라우저, **Node**, 워커 | 브라우저(studio iframe) |

- **패키지는 studio 를 import 하지 않는다.** `studio-plugin.mjs` 는 `host` 객체의 **모양(duck
  type)만** 알고, `rhwp-studio` 를 참조하지 않는다. 그래야 패키지가 단독으로 배포·테스트된다.
- **공통 코어 하나, 어댑터 둘.** 문서 조작 본체(`index.mjs`)는 그대로 두고, 소유·undo·조판
  무효화만 **DocumentAdapter** 인터페이스로 뽑는다. standalone 어댑터는 오늘 동작 그대로,
  plugin 어댑터는 §4.2/§4.3 대로. 두 모드가 **같은 API 구현을 공유**해야 원장(312/484)이 한 벌로
  유지된다 — 갈라지면 호환 수치가 모드마다 달라진다.
- **기존 gate 는 그대로 판정자다.** `npm --prefix npm/hwpctrl-ocx run gate` 는 standalone 모드를
  잰다. 이 계획의 어떤 Phase 도 그 수치를 떨어뜨리면 안 된다(§7-8).
- `@rhwp/hwpctrl` 을 Node 에서 쓰는 경로(서버측 서식 채움 등)는 이 계획으로 **영향받지 않는다**.
  plugin 모드 코드가 DOM·studio 를 정적으로 끌어오면 그 경로가 깨지므로, subpath 를 갈라
  `@rhwp/hwpctrl`(코어)과 `@rhwp/hwpctrl/studio-plugin`(어댑터)을 **다른 진입점**으로 둔다.

### 4.1 플러그인 계약 (코어 무의존의 핵심)

```ts
interface StudioPlugin {
  readonly id: string;             // 'hwpctrl'
  readonly apiVersion: 1;
  activate(host: PluginHost): PluginSurface | Promise<PluginSurface>;
  deactivate?(): void;
}
interface PluginHost {
  wasm: WasmNamespace;                 // studio 가 초기화한 그 wasm 모듈 (재초기화 금지)
  borrowDocument(): DocHandle;         // 현재 문서 핸들 — 소유권은 studio
  transaction<T>(label: string, fn: (tx: Tx) => T): T;   // 유일한 뮤테이션 통로
  automation: StudioAutomation;
  events: { on(name, cb): () => void };
  onDocumentSwap(cb: (doc: DocHandle) => void): () => void;
}
```

- **코어는 플러그인을 import 하지 않는다.** 등록은 바깥에서 들어온다 —
  `window.rhwp.plugins.register(plugin)` 또는 embed RPC `plugin.load({url})`(동적 `import()`).
  코어에 남는 것은 `plugin/host.ts` 뿐이고 이 파일은 hwpctrl 을 모른다.
- 미등록 상태에서 `hwpctrl.*` RPC 는 `{code:'PLUGIN_NOT_LOADED'}`. studio 는 아무 영향 없다.

### 4.2 문서 공유 (메모리)

- `borrowDocument()` 는 `WasmBridge` 가 들고 있는 `HwpDocument` 를 **그대로** 넘긴다. 복사·직렬화
  없음(§9.C-a 로 확인: 주입 핸들의 `__wbg_ptr` 이 동일). 플러그인은 `free()` 하지 않는다 —
  해제는 소유자만 한다.
- **차용은 raw 핸들이 아니라 토큰이다**: `{ handle, generation }`. 매 호출 진입에서
  `generation === WasmBridge._documentGeneration && handle.__wbg_ptr !== 0` 을 검사하고, 어긋나면
  `DOCUMENT_RELEASED` 로 **던진다**. 근거는 §9.C-c — 검사가 없으면 해제된 문서에 대한 호출이
  예외가 아니라 `0`·빈 문자열이라는 **조용한 오답**으로 돌아온다.
- hwpctrl 의 `Open`/`Clear` 는 doc 을 **교체**한다(`index.mjs:1068,1078,1132`). 이 경로는 플러그인
  모드에서 **가로채서** `WasmBridge.loadDocument`/`createNewDocument` 로 넘긴다. 그래야 원자 교체·
  `_documentGeneration` 증가·이전 doc `free()`·최근 문서 연결 유지(#3474)가 유지된다.
  플러그인은 `onDocumentSwap` 으로 새 핸들을 다시 받는다. **가로채지 않으면 두 문서가 조용히
  갈라진다** — §9.C-d 실측: `Open()` 뒤 소유자 참조는 옛 문서를 그대로 가리킨 채 계속 살아 있다.
- `SaveAs` 는 플러그인 모드에서 다운로드를 직접 트리거하지 않는다. `onSave` 훅으로 바이트를 올려
  부모 SDK 가 받는다(embed 는 이미 transferable ArrayBuffer 를 협상한다).

### 4.3 뮤테이션 라우팅 (undo 안전)

```
editor.hwpctrl.batch(ops)  ─▶ host.transaction('hwpctrl: 25 ops', tx => { ...ops... })
                                 │ 진입: CommandHistory 에 스냅샷 커맨드 1건 push
                                 │ 본문: hwpctrl 이 doc 을 뮤테이트 (조판 무효화만 표시)
                                 └ 종료: flushDeferredPagination → dirty page 부분 repaint → 이벤트 1회
```

- **배치 1건 = undo 1스텝.** 100 개 필드 채움을 100 번 되돌리게 만들지 않는다.
- 단건 호출(`editor.hwpctrl.PutFieldText(...)`)은 **암묵 1-op 트랜잭션**이다. 계약은 같다.
- `WASM_MAX_SNAPSHOTS`(100)·`SNAPSHOT_ID_BUDGET` 예산(`engine/history.ts`)을 자동화가 폭주로
  소진할 수 있다 — 배치를 강제하는 이유가 하나 더 있는 셈이고, 스냅샷 압박 시 경고를 올린다.
- `mutation-routing-guard` 원장에 플러그인 경로를 추가해 **직접 뮤테이션을 저작 시점에 차단**한다.

### 4.4 종료 계약 — 플러그인을 내려도 studio 는 산다

`studio.plugins.unload('hwpctrl')`(및 `list()`/`load()`)는 **정상 경로**다. 예외 처리가 아니다.

- **문서는 남는다.** doc 은 처음부터 studio 소유이고 플러그인은 차용자였다. unload 는 차용을
  반납할 뿐 문서·커서·선택·undo 스택·dirty 상태 어느 것도 건드리지 않는다. 사용자는 하던 편집을
  이어서 한다.
- **플러그인이 남긴 것은 호스트가 회수한다.** 등록한 `ext:` 커맨드, 추가한 메뉴 항목, 구독한
  이벤트, `onDocumentSwap` 훅을 **호스트가 등록 원장으로 추적**하다가 unload 시 일괄 철거한다.
  플러그인의 `deactivate()` 성실성에 기대지 않는다 — 기대면 죽은 메뉴 항목이 남는다.
- **진행 중 트랜잭션이 있으면 unload 는 그것을 기다리거나 롤백한다.** 반쯤 적용된 문서를 남기고
  내려가는 경로는 없다.
- **멱등·재장착 가능**: unload 후 다시 load 하면 같은 문서 위에서 처음처럼 동작한다. hwpctrl 자체
  상태(ParameterSet, `InitScan` 진행 커서, 내부 위치)는 unload 로 **버려진다** — 문서가 아니라
  세션 상태이기 때문이다. 부모 SDK 는 unload 후 `studio.hwpctrl.*` 호출에
  `{code:'PLUGIN_NOT_LOADED'}` 를 돌려준다.
- **메모리**: unload 는 플러그인 모듈의 참조만 끊는다. wasm 문서는 studio 소유라 그대로다.
  즉 unload 로 회수되는 것은 JS 측 상태뿐이고, 그 크기는 §6 게이트로 잰다.
- **역방향은 성립하지 않는다**: `studio.destroy()` 는 플러그인을 먼저 내리고 자신을 파괴한다.
  플러그인이 studio 보다 오래 살 수 없다.

## 5. L4/L5 — RPC 와 부모 SDK

### 5.1 프로토콜

`EMBED_PROTOCOL_VERSION` 은 **1 을 유지**하고 capability 로 넓힌다(`embed/protocol.ts:2`):
`automation-v1`, `hwpctrl-v1`, `plugin-loader-v1`, `event-subscription-v1`.
구버전 studio 에 붙은 신버전 SDK 는 capability 부재로 **기능만 비활성**되고 기존 임베드는 그대로
돈다. 새 메서드: `automation.list|execute|isEnabled|context|menu.*`, `plugin.load|unload|list`,
`hwpctrl.invoke|batch`, `events.subscribe|unsubscribe`.

### 5.2 부모 SDK

```js
const studio = await createStudio('#app', { plugins: ['hwpctrl'] });

await studio.commands.execute('edit:copy');
const cmds = await studio.commands.list();          // 메뉴 항목 + 활성 상태

await studio.hwpctrl.batch(h => {                   // 한 번의 메시지, 한 번의 조판
  h.PutFieldText('기안자', '홍길동');
  h.PutFieldText('기안일', '2026-08-11');
  h.Run('MoveDocEnd');
});
const bytes = await studio.hwpctrl.saveAs('기안문.hwp', 'Hwp');

studio.on('command-state-changed', (ctx) => { /* 툴바 동기화 */ });
```

창 제어(`mount`/`setChrome`/`setZoom`/`destroy`)는 §1.2 의 표면이 그대로 같은 채널을 탄다.
RPC 메서드는 `chrome.set|get`, `view.setZoom`, `view.setReadOnly` 이고, `mount`/`destroy` 는
부모 측 DOM 작업이라 RPC 가 아니다(단 `destroy` 는 종료 직전 studio 에 `session.teardown` 을 보내
문서 해제를 요청한다).

`batch` 의 콜백은 **부모에서 실행되지 않는다** — 호출을 기록해 `[{m, a}, ...]` 배열로 직렬화한 뒤
한 메시지로 보낸다(체이닝 기록 프록시). 반환값이 필요한 호출은 배치 결과 배열에서 인덱스로 꺼낸다.
**함수 문자열을 iframe 에서 `eval` 하는 경로는 만들지 않는다.**

### 5.3 신뢰 경계

- 부모 오리진 검사는 기존 `isUsableParentOrigin` 을 그대로 쓴다. 여기에 **호스트가 정하는
  allowlist** 를 더한다 — 자동화는 "임베드해서 보기"보다 훨씬 강한 권한이다.
- `plugin.load({url})` 은 기본 **차단**. 허용 목록(같은 오리진 + 명시 URL)만 통과시킨다.
  기본 배포에서는 `plugins: ['hwpctrl']` 처럼 **번들에 이미 있는 이름**만 허용한다.
- 문서에서 온 값(필드명·본문)을 자동화가 다시 프롬프트/평가에 넣는 경로는 만들지 않는다
  (`mydocs/tech/envelope_provenance.md` 계약).

## 6. 속도·메모리 설계 (측정 가능한 목표)

| 축 | 설계 | 게이트 | **실측 (2026-08-12)** |
| --- | --- | --- | --- |
| 문서 IR | studio 소유 1벌, 플러그인은 차용 | 복제 없음 | **핸들 포인터 동일** — 구조적 증명(§9.C-a). WASM 힙은 JS 에서 관측 불가(모듈이 `memory` 미노출) |
| 미사용 비용 | 플러그인 동적 import | 미로드 시 로드 0 | **별도 청크 54.4 kB(gzip 16.2)** — 올리지 않으면 로드되지 않음 |
| 배포 분리 | `RHWP_WITHOUT_HWPCTRL=1` 상수 분기 tree-shake | 산출물·빌드 의존 모두 0 | **청크 소멸 확인. `npm/hwpctrl-ocx` 폴더가 없어도 빌드 성공** |
| RPC | 배치 1왕복 | 배치 N = postMessage 1 | **배치 100회 = 1회 / 개별 100회 = 100회** |
| 조판·왕복 이득 | 배치 = 트랜잭션 1건 = 조판 1회 | 배치가 개별보다 빠름 | **11~13ms vs 583~585ms (약 45배)** |
| undo | 배치 = 스냅샷 1건 | undo 1회로 전체 복원 | **20-op 배치를 undo 1회로 바이트 단위 복원** |
| 플러그인 수명 | load↔unload 는 JS 상태만 | 잔여 등록물 0 | **20회 왕복 후 커맨드 181 → 181, 로드 1개** |
| 인스턴스 수명 | destroy 가 iframe·port·문서 회수 | DOM 잔여 0 | **컨테이너 자식 0, 문서 내 iframe 0** |
| 인스턴스 힙 | — | 회귀 감시선 | **사이클당 1.14MB, 선형(11.45/11.42MB)** — 아래 주 참조 |
| 해제 안전 | 차용 토큰 검사(§9.C) | 조용한 오답 0 | **`DOCUMENT_RELEASED` 로 예외** (문서 교체 미통지를 실제로 잡아냄) |

**인스턴스 힙에 대한 단서.** create↔destroy 20회에 22.9MB 가 남고 앞뒤 절반이 11.45/11.42MB 로
**선형**이다 — 캐시 포화가 아니라 사이클마다 약 1.1MB 가 남는다. 다만 **귀속을 못 가렸다**:
same-origin iframe 은 부모와 같은 isolate 를 써서 `usedJSHeapSize` 에 iframe 내부 힙이 섞이고,
SDK 없이 순수 iframe 으로 돌린 대조군(0.85MB)은 studio 부팅 완료를 기다리지 않아 공평한 비교가
아니다. 그래서 게이트는 **현재 값을 회귀 감시선**(사이클당 1.8MB)으로 잡고, 원인 규명은 §9.E 에
남긴다. 실용적 함의는 하나다 — **인스턴스를 반복 생성하기보다 한 인스턴스에 문서를 갈아끼운다**.

**하지 않을 것**: hwpctrl 호출마다 문서 바이트를 왕복시키는 편의 경로. 초기에 편하다는 이유로
넣으면 그게 기본이 되고, 위 표의 모든 목표가 무의미해진다.

## 7. 양방향 독립을 강제하는 장치

"studio 는 hwpctrl 없이, hwpctrl 은 studio 없이" 는 선언이 아니라 **게이트**여야 한다.

1. **정적 의존 금지**: `src/{core,engine,view,ui,command}/**` 에서 `plugin/`·`hwpctrl` import 를
   금지하는 소스 가드 테스트. 역방향(`plugin/` → 코어)만 허용.
2. **번들 델타 게이트**: 플러그인 미로드 빌드의 엔트리 청크 크기 회귀 검사.
3. **기존 e2e 전량**: `rhwp-studio/package.json` 의 e2e 스위트를 플러그인 없는 구성으로 그대로
   통과시킨다(현재가 기준선).
4. **이중 구성 e2e**: 신규 e2e 는 `--with-plugin` / 없이 두 번 돈다. 플러그인 있는 쪽만 테스트를
   추가하면 "없을 때 깨짐" 이 조용히 들어온다.
5. **런타임 실패 격리**: 플러그인 `activate` 예외는 studio 를 죽이지 않는다 — 로그 + RPC 에러로
   끝내고 편집기는 계속 산다.
6. **unload 생존 e2e** (§4.4 의 판정): 문서를 열고 → hwpctrl 로 편집하고 → `unload('hwpctrl')`
   한 뒤, ① 문서 내용·페이지 수 동일 ② 커서·선택 유지 ③ undo 로 hwpctrl 편집이 되돌려짐
   ④ 키보드 입력·메뉴 실행 정상 ⑤ 저장 산출물 동일 ⑥ 플러그인이 넣은 메뉴 항목 사라짐.
   이 여섯이 "종료해도 studio 는 산다"의 실제 정의다.
7. **브리지 없는 구동**: studio 를 직접 URL 로 열었을 때(부모 없음) 자동화·플러그인 배선이
   붙지 않은 채 오늘과 동일하게 동작한다.

반대 방향(§4.0)도 같은 강도로 잠근다.

8. **standalone gate 무회귀**: `npm --prefix npm/hwpctrl-ocx run gate` 의 통과 수치가 이 계획의
   어느 Phase 에서도 내려가지 않는다. 원장(312/484)은 두 모드가 **같은 값**이어야 한다.
9. **studio 정적 참조 금지**: `npm/hwpctrl-ocx/src/**` 가 `rhwp-studio` 를 import 하지 않음을
   소스 가드로 검사. `studio-plugin.mjs` 는 `host` 의 모양만 안다.
10. **Node 스모크**: DOM 없는 Node 에서 `import('@rhwp/hwpctrl')` → `Open`/`PutFieldText`/`SaveAs`
    가 동작한다. plugin 어댑터가 DOM 을 정적으로 끌어오면 여기서 즉시 깨진다.
11. **모드 동등성 테스트**: 같은 시나리오를 standalone 과 plugin 두 모드로 돌려 **문서 산출
    바이트가 동일**함을 확인한다. 갈리면 어댑터가 조작 의미를 바꾼 것이다.

## 8. 단계

| Phase | 내용 | 완료 판정 | 상태 |
| --- | --- | --- | --- |
| **P0** 계약 | 이 문서의 표면을 타입으로 고정(`automation/types.ts`, `plugin/types.ts`), dispatch 판정 반환 | 타입 컴파일. 런타임 변경 0. (메뉴 구성·iframe 이동 실측은 §9.A/§9.B 로 **완료**) | 완료 (2026-08-12) |
| **P1** 자동화 호스트 | L1 구현(§3.2 의 P1 갈래만), `window.rhwp.automation` 노출, `extension-api.ts` 흡수·삭제, 런타임 registry↔마크업 대조 가드 | 커맨드 질의·실행·활성 판정 e2e, `data-cmd` 드리프트 0 유지. 기존 스위트 무회귀 | 완료 |
| **P2** 플러그인 호스트 | L2 + `transaction` + `onDocumentSwap` + **load/unload 등록 원장**(§4.4) + 뮤테이션 가드 확장 | 더미 플러그인으로 트랜잭션 1스냅샷·부분 repaint, load↔unload 잔여 0 | 완료 |
| **P3** hwpctrl 플러그인 | L3, `DocumentAdapter` 분리(§4.0) + `@rhwp/hwpctrl/studio-plugin` subpath, 차용 토큰 검사(§9.C), **커서 좌표 변환기**(§9.D), Open/Clear/SaveAs 가로채기 | `PutFieldText`→화면 반영→`exportHwp` 왕복 e2e, §7-6 unload 생존 e2e, §6 메모리 게이트 | 완료 |
| **P4** 브리지·SDK | L4/L5/L6, `createStudio` 수명(mount/destroy)·창 제어, capability 협상, 배치 프록시, 이벤트 구독 | 부모 페이지 데모에서 §1.2·§5.2 코드 그대로 동작, §6 RPC·수명 게이트 | 완료 |
| **P5** 성능 | deferred pagination 결선, 부분 repaint 결선, 벤치 하니스 | §6 표 전 항목 수치 기록 | 완료 — §6 실측 |
| **P6** 마감(선택) | `index.html` 메뉴 197항목의 선언 데이터 이관(§3.2), 다이얼로그 정책 원장, 문서 3층 갱신 | `npm/editor/README.md`·`npm/hwpctrl-ocx/README.md`·본 계획서 수치 갱신 | 완료 — 메뉴 이관은 미착수(선택) |

각 Phase 는 독립 PR 이고 `mydocs/manual/pr_review/local_validation.md` §4.3 의 studio 범위 게이트를
따른다. 레이아웃·조판 경계를 건드리는 P5 는 `--lib` 가 아니라 **전체 `cargo test`** 를 돌린다.

## 9. 실측 기록과 남은 미결

### 9.A 해소 — 메뉴 구성 방식 (구 미결 1)

**측정 (2026-08-11, 소스·마크업 정적 분석)**

| 항목 | 값 |
| --- | --- |
| `index.html` 최상위 `.menu-item` | 8 (file/edit/view/insert/format/page/table/tool) |
| `.md-item` / `.md-sub` | 197 / 32 |
| 고유 `data-cmd` | 140 |
| `commands/*.ts` 등록 id (정적 리터럴) | 156 |
| 마크업에 있으나 registry 정적 리터럴에 없음 | 25 → **전부 헬퍼 생성**(`view:zoom-${pct}`, `view:theme-${mode}`, 캡션 9방위 등) → 실질 드리프트 **0** |
| registry 에 있으나 메뉴 마크업에 없음 | **41** (컨텍스트 메뉴·단축키 전용) |

**결론**: 구조는 HTML 선언, 의미(레이블·활성·실행)는 registry/dispatcher — 이미 갈려 있다.
따라서 §3.2 처럼 **질의(registry 순회 + DOM 파생)는 P1 에서 싸게 서고, `index.html` 이관은 P6
선택 사항**으로 내린다. 부수 소득 두 가지: ① `listCommands()`≠`getMenuModel()` 을 분리해야 할
근거(41건) ② 런타임 registry 대조 가드로 드리프트 0 을 고정해야 할 자리.

### 9.B 해소 — iframe DOM 이동 (구 미결 7)

**측정 (2026-08-11, Chrome 149 headless / puppeteer, http origin, 자식에 32MB
`WebAssembly.Memory` + 변경된 상태 + 부모와 `MessageChannel` 연결)**

| 조작 | 재로드 | 자식 상태 | MessagePort |
| --- | --- | --- | --- |
| 다른 `div` 로 `appendChild` | **예** (load 2회) | `EDITED`→`INITIAL` 소실 | 죽음 (ping 무응답) |
| **같은** 부모로 재삽입 | **예** | 소실 | — |
| 조상 wrapper 째로 이동 | **예** | 소실 | — |
| `display:none` ↔ 복원 | 아니오 | 보존 | 생존 |
| `position`/`left`/`transform` 변경 | 아니오 | 보존 | 생존 |

재현 방법: http 로 서빙한 부모 페이지가 자식 iframe 을 심고 → 자식의 `window.__state` 를 바꾸고
→ 조작을 가한 뒤 `loadId`(로드마다 새로 뽑는 난수)·`__state`·port ping 응답을 비교한다.
프로브 스크립트는 세션 scratchpad 에 있으며 **저장소에 남기지 않는다** — P4 에서 이 판정이 회귀
가드로 필요하면 `rhwp-studio/e2e/` 에 정식 테스트로 다시 쓴다.

**결론**: `mount('#other')` 는 **만들지 않는다**. 마운트는 `createStudio(container)` 최초 1회,
화면상의 이동·숨김은 컨테이너 CSS, 진짜 이동이 필요하면 `destroy()` → 재생성 → 문서 재적재다.
"이동해도 문서가 유지되는 API" 는 브라우저 수준에서 불가능하므로 표면에 두지 않는다.
`display:none` 이 상태를 보존한다는 것은 **탭 UI 에서 studio 를 숨겼다 되살리는 구성이 안전**하다는
뜻이기도 하다 — 이것을 권장 패턴으로 문서화한다.

### 9.C 해소 — 핸들 차용의 실제 수명 (구 미결 2)

**측정 (2026-08-11, Node 22 + `pkg/` WASM + `@rhwp/hwpctrl` 직접 구동, `samples/table-001.hwp`)**

| 실험 | 결과 |
| --- | --- |
| (a) `createHwpCtrl({ wasm, doc })` 로 주입한 핸들 | `h.getWasmDoc() === doc`, `__wbg_ptr` 동일 → **zero-copy 성립** |
| (b) 소유자가 `doc.free()` 후 **직접** 호출 | `THREW: null pointer passed to rust` (`__wbg_ptr` 0) |
| (c) 같은 상태에서 **hwpctrl 경유** 호출 | `PageCount()` → **0**, `GetTextFile()` → **빈 문자열**. 예외 없음 |
| (d) hwpctrl `Open()` 이 문서를 갈아끼운 뒤 | 소유자 참조는 **옛 문서 그대로 살아 있고**(`ownerStillUsable: true`) ptr 만 갈린다 |
| (e) 잘못된 인자 호출 | 그 호출만 실패하고 인스턴스는 계속 정상(후속 호출·새 문서 생성 모두 OK) |

**결론 — 설계에 두 개의 강제 조항이 생긴다.**

1. **use-after-free 가 침묵한다(c).** wasm-bindgen 은 정직하게 던지는데 hwpctrl 의 방어적
   `this.#doc?.x?.()` + 기본값 파싱이 그것을 삼켜 **0·빈 문자열이라는 그럴듯한 오답**으로 바꾼다.
   따라서 plugin 어댑터는 매 호출 진입에서 **핸들 유효성(세대 토큰 + `__wbg_ptr !== 0`)을 검사**하고
   무효면 `{code:'DOCUMENT_RELEASED'}` 로 **던진다**. "빌린 핸들은 조용히 틀리지 않는다" 를 계약에
   못 박는다.
2. **`Open`/`Clear` 가로채기는 선택이 아니다(d).** 가로채지 않으면 두 문서가 조용히 갈라져
   studio 는 옛 문서를, hwpctrl 은 새 문서를 만진다 — 화면과 저장 산출물이 어긋나는 최악의 형태다.
   §4.2 의 위임은 이 실측이 근거다.

캡슐화 문제는 **소유권 이전이 아니라 차용 토큰**으로 푼다: `borrowDocument()` 는 raw 핸들이 아니라
`{ handle, generation }` 을 주고, 호스트는 `_documentGeneration`(이미 `wasm-bridge.ts` 에 있다)이
바뀌면 이전 토큰을 무효로 본다. `WasmBridge.doc` 의 `private` 는 그대로 둔다.

### 9.D 해소 — 두 커서 좌표계 (구 미결 3)

**측정 (같은 하니스, `table-001.hwp` 131셀 / `issue1949_giant_cell_nested_tables_perf.hwp`
198리스트·깊이 2)**

- hwpctrl 은 `{list, para, pos}`, studio 는 `DocumentPosition{sectionIndex, paragraphIndex,
  charOffset, cellPath[]}` 다. **좌표계가 다르다.**
- 그런데 `getCursorModel()` 의 리스트 엔트리가 **변환에 필요한 필드를 이미 전부 담고 있다**:
  `listId, isCell, hostListId, sectionIndex, hostPara, controlIndex, cellIndex, row, col, ...`.
  별도 WASM API 를 새로 파지 않아도 된다.
- **변환 규칙 (실증 완료)**: `hostListId` 를 따라 루트까지 사슬을 세우고
  `sectionIndex = chain[0].sectionIndex`, `parentParaIndex = chain[0].hostPara`,
  `path[k] = { controlIndex, cellIndex, cellParaIndex: chain[k+1]?.hostPara ?? 대상문단 }`.
  - 단층 표: `list 2` → `(sec 0, para 1, [{ctrl 0, cell 0, cellPara 0}])` 로 변환해 그 좌표로
    삽입하니 셀 문단 길이 **3 → 7**(4자 삽입) — **정확히 그 셀**을 지목했다.
  - 중첩 표(깊이 2): 같은 규칙으로 **11 → 15** — 중첩에서도 성립한다.
    (`cellParaIndex` 를 0 으로 고정하면 실패한다. 자식 표가 놓인 부모 셀 안의 문단 번호가
    반드시 들어가야 한다 — 첫 시도가 여기서 틀렸다.)
- **두 커서는 지금 서로를 전혀 모른다.** hwpctrl 이 `SetPos(2,0,0)` 으로 셀에 들어가도
  `doc.getCaretPosition()` 은 `{0,2,0}` 그대로다. 애초에 후자는 **문서에 저장된 캐럿**이지 라이브
  UI 커서가 아니다(`wasm_api.rs:5888` 주석이 이 구분을 명시한다).

**결론**: 기본안대로 **studio 커서를 단일 진실**로 삼는다. plugin 어댑터는 hwpctrl 의
`{list,para,pos}` 를 위 규칙으로 studio 좌표로 옮겨 `CursorState` 에 반영하고, `GetPos` 는 역방향
(`sectionIndex/hostPara/controlIndex/cellIndex` 로 리스트 역인덱싱)으로 답한다. 변환기는 **P3 의
첫 산출물**이고, 위 두 문서를 회귀 픽스처로 고정한다.

~~미확증 하나~~ **해소 (2026-08-12)**: `GetTextFile` 이 빈 문자열을 주던 것은 **`pkg/` WASM 이
낡아서**였다. `getTextFileUnicode`/`getTextFileText` 는 `wasm_api.rs` 에 있는데 빌드된 pkg 에는 없어,
`this.#doc?.getTextFileUnicode?.() ?? '""'` 의 optional chaining 이 그것을 **빈 문자열로 삼켰다**.
재빌드 후 `GetTextFile` 은 **표 안 텍스트까지 포함**해 돌려준다(table-001 386자, footnote-01 1832자).
같은 재빌드로 `run gate` 의 기존 실패 8건도 사라졌다. 교훈은 §9.F 에 적는다.

### 9.F 교훈 — 낡은 `pkg/` 가 만든 유령 결함

P4 검증 중 `GetTextFile` 이 본문 40문단짜리 문서에서도 빈 문자열을 돌려주는 것을 발견했다.
패키지 결함으로 보였으나 원인은 **빌드 산출물이 소스보다 낡은 것**이었다(pkg 13:20, 소스 16:45).

두 가지가 겹쳐 오래 숨었다.

1. **방어적 optional chaining 이 부재를 침묵시킨다.** `#doc?.getTextFileUnicode?.() ?? '""'` 는
   API 가 없을 때 예외 대신 빈 문자열을 준다 — §9.C 가 지적한 "조용한 오답" 과 같은 계급이다.
2. **재빌드가 gate 실패 8건도 함께 지웠다.** 그 8건을 "기존 실패" 로 보고했었는데, 실은 낡은
   WASM 이 만든 유령이었다. 기준선을 재기 전에 **`pkg/` 빌드 시각을 먼저 본다**.

플러그인 계약에 이 교훈이 이미 반영돼 있다 — plugin 어댑터는 optional chaining 삼킴을 금지하고
lease 무효를 **던진다**(§9.C-1).

### 9.E 남은 미결

1. ~~메뉴 구성 방식~~ → §9.A 해소.
2. ~~`WasmBridge.doc` 캡슐화~~ → §9.C 해소 (차용 토큰 + 세대 검사).
3. ~~두 커서 좌표계~~ → §9.D 해소 (studio 커서 단일 진실 + 변환 규칙 실증).
4. 조판 정밀도 의존 API(`MoveLine*`/`MovePage*`)는 studio 조판을 그대로 물려받는다 — 자동화
   문서에 한계로 명시할지, 원장의 `substituted` 로 올릴지.
5. 플러그인 로드 시점: 이제 **동적 load/unload 를 정면 지원**하므로(§4.4), "이미 열린 문서에
   뒤늦게 붙는" 경로가 기본 경로다. 붙는 순간 문서 상태를 어떻게 관측하는지 확정 필요.
6. `unload` 가 트랜잭션 진행 중에 들어오면 **대기인가 롤백인가**. 대기는 hung 위험, 롤백은
   사용자가 본 화면을 되돌린다. 기본안: 대기 + 타임아웃 후 롤백, 타임아웃 값은 P2 에서 실측.
7. ~~iframe DOM 이동~~ → §9.B 해소 (`mount` 제거, CSS 이동 권장).
8. **인스턴스 반복 생성의 사이클당 ~1.1MB 잔류**(§6)의 귀속 — SDK / studio 부팅 / 브라우저
   회수 특성 중 무엇인가. 공평한 대조군은 "부팅 완료까지 기다린 순수 iframe" 이다.
9. 다중 인스턴스에서 wasm 을 공유할 여지가 있는가(같은 오리진 iframe 간 SharedArrayBuffer 등).
   기본안은 **공유하지 않음** — 격리가 단순하고 안전하다. 비용은 §6 에 명시.

## 10. 이 계획이 하지 않는 것

- `rhwp-studio/src/hwpctl/` 철거 (선행 계획서 P7 소관)
- `@rhwp/hwpctrl` 의 원장 커버리지 확대 (312/484 는 이 계획과 독립)
- 부모 페이지가 iframe 없이 hwpctrl 만으로 화면을 그리는 경로 (§0-2 로 배제)
- 임의 스크립트를 iframe 안에서 평가하는 API (§5.2)
