---
kind: canonical
status: draft
canonical: mydocs/tech/wasm_agent_surface/browser_bridge.md
last_verified: 2026-08-03
---

# 브라우저 내 MCP-유사 브리지 설계

> MCP 는 stdio JSON-RPC 다. **브라우저에는 stdio 가 없다.**
> 이 문서는 그 자리에 무엇을 놓을지 — `window.postMessage` 인가, `MessageChannel` 인가,
> Web Worker 인가 — 를 각각의 대가와 함께 확정하고, **문서 내용이 다른 origin 으로
> 새지 않게 하는 경계**를 명시한다.
> 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 둘째 줄에 대응한다.

이 문서의 모든 기술 주장에는 코드 경로(`파일:줄`)가 붙는다. 근거를 대지 못하는 항목은
**"확인되지 않음"** 으로 적었다. **성능은 측정하지 않았으므로 어떤 성능 주장도 하지 않는다.**
축 전체의 지도는 [README.md](README.md), 노출할 동사의 목록은
[self_description.md](self_description.md) 가 정한다.

---

## 0. 결론 먼저

1. **바닥부터 짓지 않는다.** studio 는 이미 `MessageChannel` 기반 요청/응답 RPC 를
   돌린다 — `rhwp-studio/src/embed/`(protocol 103줄 + rpc-router 102줄 + runtime 162줄)와
   클라이언트 `npm/editor`(`@rhwp/editor` 0.8.2, index 253줄 + transport 239줄).
   버전 협상·capability 협상·origin 고정·transferable 이 **이미 구현돼 있다.**
2. **그 프로토콜은 MCP 가 아니다.** `rhwp-request`/`rhwp-response` 라는 자체 봉투이고,
   메서드는 **11개**뿐이며 전부 뷰어/내보내기 축이다(`rpc-router.ts:70-101`).
   에이전트 동사(`digest`·`fields`·`inspect`)는 하나도 없다.
3. **채택: `MessageChannel` 위에 MCP JSON-RPC 를 얹는다.** 기존 embed 핸드셰이크로
   포트를 얻고, 그 포트 위 메시지를 JSON-RPC 로 바꾼다. `window.postMessage` 는
   핸드셰이크에만 쓰고 본 통신에는 쓰지 않는다 — 이유는 §3.
4. **Web Worker 는 옳은 방향이지만 지금은 아니다.** studio 전체가 메인 스레드에서
   WASM 을 돌린다(`rhwp-studio/src/core/wasm-bridge.ts:259`, `await init()`).
   `grep -rn "new Worker" rhwp-studio/src` 결과는 **0건**이다. Worker 로 옮기는 것은
   브리지 설계가 아니라 **스튜디오 아키텍처 변경**이다.
5. **보안 경계는 origin 하나다.** 기존 런타임이 이미 첫 연결의 origin 에 고정한다
   (`runtime.ts:130-133`). MCP-유사 브리지는 그 고정을 **약화시키지 않는 방식으로만**
   확장한다.

---

## 1. 이미 있는 것 — 전수 조사

### 1.1 계층

```
호스트 페이지 (임의 origin)
    │  npm/editor: RhwpEditor  (index.js 253줄)
    │      └ transport.js 239줄 — 세션 ID·타임아웃·transferable
    │
    │  ① window.postMessage({type:'rhwp-connect', ...}, [port2])
    ▼
<iframe> rhwp-studio (studio origin)
    │  installEmbedRuntime  (embed/runtime.ts:113)
    │      ├ 발신자 검사 (runtime.ts:117-127)
    │      ├ origin 고정   (runtime.ts:130-133)
    │      └ bindPort      (runtime.ts:42-73)
    │  ② 이후 전 통신은 MessagePort 위에서만
    ▼
routeEmbedRequest (embed/rpc-router.ts:63)
    ▼
studio 핸들러 (main.ts:1412 이후)
    ▼
wasm-bridge.ts → HwpDocument (src/wasm_api.rs:337)
```

배선 지점은 `rhwp-studio/src/main.ts:1412` 한 곳이다.

### 1.2 핸드셰이크 — 이미 협상이 있다

`rhwp-studio/src/embed/protocol.ts:1-7`:

```ts
export const EMBED_PROTOCOL_VERSION = 1 as const;
export const EMBED_CAPABILITIES = [
  'transferable-array-buffer',
  'hml-export',
  'renderer-diagnostics-v1',
  'notify-saved-v1',
] as const;
```

**이게 자기서술의 원시 형태다.** 브리지가 새로 발명할 필요가 없는 자리 —
[self_description.md](self_description.md) 의 `capabilities()` 결과를
`rhwp-connected` 응답에 실으면 된다.

협상 흐름(실측):

| 단계 | 코드 | 동작 |
| --- | --- | --- |
| 연결 시도 검증 | `protocol.ts:67-73` `isConnectAttempt` | `type`·`version`(safe integer)·`sessionId`(비어있지 않은 문자열) |
| 필수 capability | `protocol.ts:60-65` `isConnectMessage` | `transferable-array-buffer` 를 **반드시** 포함 |
| 버전 불일치 거부 | `runtime.ts:75-92` `rejectConnect` | `UNSUPPORTED_VERSION` 또는 `UNSUPPORTED_CAPABILITY` + `supportedVersions` |
| 성공 통지 | `runtime.ts:69-72` | `rhwp-connected` + 서버 capability 배열 |
| 요청 검증 | `protocol.ts:75-83` `isRequestEnvelope` | `version` 일치 + `method` 가 비어있지 않은 문자열 |

거부 응답이 `supportedVersions` 를 함께 준다는 점이 중요하다 — 클라이언트가 재시도를
판단할 수 있다. **MCP `initialize` 의 버전 협상과 같은 구조**다.

### 1.3 메서드는 11개, 전부 뷰어 축

`rhwp-studio/src/embed/rpc-router.ts:70-101` 의 `switch` 전수:

| 메서드 | 반환 | 축 |
| --- | --- | --- |
| `ready` | `boolean` | 수명주기 |
| `loadFile(data, fileName, skipUnsavedGuard, suppressDialogs)` | `{pageCount}` | 입력 |
| `pageCount` | `number` | 조회 |
| `getRendererDiagnostics(page)` | `EmbedRendererDiagnosticsV1` | 진단 |
| `getPageSvg(page)` | `string` | 렌더 |
| `exportHwp` / `exportHwpx` / `exportHml` | `Uint8Array` | 산출 |
| `getHmlSaveState` | `HmlSaveState` | 조회 |
| `exportHwpVerify` | `unknown` | 검증 |
| `notifySaved(fileName?)` | `{ok, wasDirty}` | 수명주기 |

**에이전트 동사는 0개다.** `digest`·`search`·`fields`·`extract-data`·`inspect` 어느
것도 없다. 그런데 그중 다수는 **WASM 에는 이미 있다**(`searchAllText`
`wasm_api.rs:4869`, `getFieldList` `4542` 등 — [self_description.md §1.3](self_description.md)).

> **브리지의 첫 작업은 전송을 발명하는 게 아니라, 이미 있는 동사를 라우터에 잇는 것이다.**

`getRendererDiagnostics` 는 `schemaVersion: 1` 을 봉투에 싣는다
(`rpc-router.ts:35-43`). embed 층에도 봉투 버저닝 선례가 이미 있다는 뜻이다.

### 1.4 클라이언트 — `@rhwp/editor`

`npm/editor/transport.js` 실측:

- 프로토콜 상수를 **복제**한다(`transport.js:1-10`) — TS 원본과 JS 사본이 별개다.
  드리프트 위험이 이미 존재하고, CI 가 이를 감시한다
  (`scripts/frontend-editor-embed.test.mjs`, `.github/workflows/ci.yml:951`).
- 세션 ID 는 `crypto.randomUUID()`, 없으면 `getRandomValues` 16바이트
  (`transport.js:17-25`). **약한 난수로 물러서지 않는다** — 둘 다 없으면 throw.
- 타임아웃이 메서드별로 다르다(`transport.js:8-16`):
  `loadFile`·`exportHwp`·`exportHwpVerify`·`exportHwpx`·`exportHml` 은 60초, 나머지 10초.
- 이진 데이터는 **복사 후 transfer** 한다(`transport.js:27-38` `copiedBinary`/`prepareParams`).
  호출자의 버퍼를 detach 시키지 않기 위한 의도적 복사다.
- 응답 봉투 검증이 엄격하다(`transport.js:40-50`) — `result` 와 `error` 가
  **동시에 있거나 동시에 없으면 거부**한다.

이 규율(타임아웃 등급·복사 후 전송·배타적 result/error)은 **MCP 브리지에도 그대로
필요**하다. 새로 정할 것이 아니라 승계할 것이다.

### 1.5 확장 메시징은 별개 계보다

`rhwp-shared/security/sender-validator.js` 는 **`chrome.runtime` 메시징**용이지
`window.postMessage` 용이 아니다. 그럼에도 설계 원칙 하나를 준다.

```js
default:
  return { allowed: false, reason: `알 수 없는 메시지 유형: ${messageType}` };
```

**기본 거부(default-deny)** 다. 메시지 유형마다 발신자 부류를 명시하고
(`fetch-file` 은 내부 페이지만, `open-hwp` 는 content script 만), 모르는 유형은 막는다.
MCP 브리지의 메서드 라우팅도 같아야 한다 — `rpc-router.ts:100` 이 이미
`throw new Error("Unknown method")` 로 그렇게 한다.

### 1.6 hwpctl — 브라우저 안의 또 하나의 동사 표면

`rhwp-studio/src/hwpctl/`(444 + 116 + 77 + 73 + 51줄)은 한컴 `HwpCtrl` ActiveX 호환
객체를 WASM 위에 얹은 것이다(`hwpctl/index.ts:1-6` 주석).

여기에도 **자기서술의 원시 형태**가 있다 — `action-registry.ts:22-31` 의
`getRegisteredCount()` / `getImplementedCount()`. 다만 정직하게 적자면:

- 파일 주석은 "312개 Action의 등록 테이블"이라고 말한다(`action-registry.ts:4`).
- 실제 `STUB_ACTIONS` 배열 항목은 **30개**다(`action-registry.ts:43-73`, Wave 1~6 각 5개).
- 그리고 그 30개는 전부 `executor: null` 로 등록된다(`action-registry.ts:75-77`).

**주석의 312 는 목표치이지 실측이 아니다.** 브리지 자기서술이 이 함수를 인용하려면
"등록 30 / 구현 (executor 등록분)" 을 그대로 노출해야 하며, 312 를 광고하면 안 된다.

---

## 2. 브라우저에 stdio 가 없다는 뜻

MCP 는 stdio JSON-RPC 2.0 이다(rhwp 구현: `src/mcp_serve.rs`, 1,596줄 — **bin 전용**,
`src/main.rs:9`). 브라우저로 옮길 때 사라지는 것과 남는 것을 가른다.

| MCP 가 stdio 에서 얻는 것 | 브라우저에서 |
| --- | --- |
| 프로세스 경계 = 신뢰 경계 | **origin 경계**로 대체된다. 훨씬 미묘하다 |
| 순서 보장된 단일 바이트 스트림 | `MessagePort` 는 순서를 보장한다. `window.postMessage` 는 **여러 발신자가 섞인다** |
| 상대방이 하나 | 어떤 창·프레임·확장도 `postMessage` 할 수 있다 |
| 요청 ID 로 다중화 | 동일 |
| 이진 데이터는 base64 | **transferable 로 복사 없이 넘길 수 있다** (기존 embed 가 이미 씀, `runtime.ts:23-30`) |
| 서버 종료 = 프로세스 종료 | 포트 close (`runtime.ts:32-36` `releasePort`) |
| 수명 = 프로세스 수명 | 탭·프레임 수명. 새로고침으로 문서 상태가 날아간다 |

`stdinTools` 라는 개념도 갈 곳이 없다. CLI 자기서술은
`"stdinTools": ["hwp_batch", "hwp_batch_search"]` 를 싣지만(실측), 브라우저에는 stdin 이
없으므로 이 필드는 **의미가 다르게** 되거나 부재로 명시돼야 한다
([self_description.md §4.3](self_description.md)).

---

## 3. 전송 후보 3종

### 후보 A — `window.postMessage`

호스트 창과 iframe 이 직접 주고받는다. 기존 코드에 **레거시 경로로 남아 있다**
(`runtime.ts:94-111` `handleLegacy`, `hwpctl-load` 타입 포함).

- 장점: 구현이 가장 단순. 포트 수명 관리가 없다. 확장 content script 에서도 쓸 수 있다.
- 단점:
  - **모든 발신자가 같은 채널에 온다.** 매 메시지마다 `event.source`·`event.origin` 을
    확인해야 한다. 기존 코드가 실제로 그렇게 한다(`runtime.ts:117-127`).
  - 응답을 `targetOrigin` 으로 되쏘아야 한다(`runtime.ts:110`). 실수하면
    `"*"` 로 새는 전형적 취약점이 된다.
  - 세션 개념이 없어 여러 소비자가 동시에 붙으면 구분이 어렵다.

기존 코드가 레거시 경로를 **가장 낮은 우선순위**로 두는 점을 보라 —
포트 바인딩이 성립한 뒤에는 레거시를 아예 무시한다(`runtime.ts:152` `if (binding) return;`).
이건 이미 내려진 판정이다.

### 후보 B — `MessageChannel` (채택)

핸드셰이크 한 번으로 전용 포트 쌍을 만들고, 이후 통신은 그 포트에서만 한다.

- 장점:
  - **채널이 사설이다.** 포트를 가진 쪽만 말할 수 있다. 매 메시지 origin 검사가
    구조적으로 불필요해진다(그래도 첫 연결에서 origin 을 고정한다 — `runtime.ts:130-133`).
  - 세션 ID 로 다중화가 이미 된다(`protocol.ts:23-30`).
  - transferable 이 자연스럽다(`runtime.ts:23-30`).
  - **이미 구현·테스트돼 있다** (`rhwp-studio/e2e/embed-transport.test.mjs`,
    `package.json` 의 `e2e:embed`).
- 단점:
  - 포트 수명 관리가 필요하다(`releasePort`/`releasePorts`, `runtime.ts:32-40`).
  - 여전히 **같은 스레드**다. 무거운 호출이 UI 를 멈춘다(§3.4).
  - 확장 background(service worker)에서 쓰려면 별도 경로가 필요하다 — 확인되지 않음.

### 후보 C — Web Worker

WASM 을 워커에서 돌리고 브리지가 워커와 대화한다.

- 장점: 긴 파싱·검색이 UI 를 멈추지 않는다. 에이전트 호출은 본질적으로 배치성이라
  **개념적으로 가장 옳다.**
- 단점 (실측 근거):
  - **studio 는 워커를 쓰지 않는다.** `grep -rn "new Worker" rhwp-studio/src` = 0건.
    WASM 초기화는 메인 스레드다(`wasm-bridge.ts:259` `await init()`).
  - 문서 상태(`HwpDocument`)가 워커 안에 있으면 **렌더 경로가 전부 깨진다.**
    `renderPageToCanvas` 계열은 캔버스 컨텍스트를 만진다. OffscreenCanvas 로 옮기는 것은
    별개의 대형 작업이다.
  - 즉 워커 채택은 브리지 결정이 아니라 **스튜디오 아키텍처 변경**이다.

### 판정

| 기준 | A postMessage | B MessageChannel | C Worker |
| --- | --- | --- | --- |
| 사설 채널 | ✗ | ○ | ○ |
| 기존 구현 재사용 | 부분(레거시) | **전부** | 없음 |
| UI 블로킹 회피 | ✗ | ✗ | ○ |
| 확장에서 사용 | ○ | 확인되지 않음 | 확인되지 않음 |
| 렌더 경로 영향 | 없음 | 없음 | **큼** |
| 추가 작업량 | 소 | **소** | 대 |

**B 를 채택한다.** A 는 핸드셰이크 전용으로 유지한다(포트를 넘기려면 어차피
`window.postMessage` 가 필요하다 — `runtime.ts:128`). C 는 **문 열어 두기**:
JSON-RPC 봉투는 전송에 무관하므로, 나중에 워커로 옮겨도 **메서드 계약은 그대로**다.
이게 JSON-RPC 를 쓰는 실질적 이유다.

---

## 4. 설계 — MCP-유사 브리지

### 4.1 계층

```
 에이전트 호스트 (JS)
      │  MCP 클라이언트 (JSON-RPC 2.0)
      ▼
 ┌──────────────────────────────────────────┐
 │ rhwp-mcp-bridge  (신규, 얇음)             │
 │  · initialize / tools/list / tools/call   │
 │  · JSON-RPC ↔ embed 봉투 변환             │
 └──────────────────────────────────────────┘
      │  기존 MessagePort (재사용)
      ▼
 ┌──────────────────────────────────────────┐
 │ embed runtime (runtime.ts:113) — 무변경    │
 │ rpc-router  (rpc-router.ts:63) — 확장      │
 └──────────────────────────────────────────┘
      ▼
 wasm_api  (에이전트 동사 — self_description.md)
```

**embed 런타임은 건드리지 않는다.** 검증·origin 고정·포트 수명은 이미 맞다.
확장은 `rpc-router.ts` 의 `switch` 에 메서드를 더하는 일이다.

### 4.2 JSON-RPC 를 어디에 얹는가 — 두 방식

**방식 1: embed 메서드 하나로 터널링**

```
{type:'rhwp-request', version:1, sessionId, id, method:'mcp', params:{ …JSON-RPC… }}
```

- 장점: embed 프로토콜 **버전을 올리지 않아도 된다.** 기존 클라이언트가 안 깨진다.
- 단점: 봉투가 이중이 된다. 오류가 두 층에서 나온다(embed `RPC_ERROR` vs JSON-RPC
  `error.code`). 디버깅이 나빠진다.

**방식 2: 포트 위 메시지를 JSON-RPC 로 직접 (권장)**

핸드셰이크에서 클라이언트가 `capabilities: ['mcp-jsonrpc-v1']` 을 선언하면,
그 포트는 **JSON-RPC 모드**로 바인딩된다. `bindPort`(`runtime.ts:42`)가 분기한다.

- 장점: 봉투가 하나. 표준 MCP 클라이언트를 거의 그대로 쓸 수 있다.
- 단점: `bindPort` 에 분기가 생긴다. 두 모드를 다 테스트해야 한다.

**방식 2 를 권장한다.** 근거: capability 협상 기제가 **이미 있고**
(`protocol.ts:60-65`), 미지원 시 `UNSUPPORTED_CAPABILITY` 로 명확히 거부하는 경로도
있다(`runtime.ts:82-88`). 새 모드를 안전하게 도입할 자리가 준비돼 있다.

### 4.3 메서드 매핑

| MCP | 브리지에서 | 출처 |
| --- | --- | --- |
| `initialize` | `rhwp-connect` 핸드셰이크에 흡수. 응답에 `protocolVersion`·서버 정보 | `runtime.ts:69-72` 확장 |
| `tools/list` | `capabilitiesMcp()` 결과를 그대로 | [self_description.md §4.1](self_description.md) |
| `tools/call` | `rpc-router` 의 메서드 디스패치 | `rpc-router.ts:70` 확장 |
| `resources/*` | **하지 않는다** (초기) | §5.5 |
| `notifications/*` | **하지 않는다** (초기) | 확인되지 않음 |

`tools/list` 가 CLI `capabilities --mcp` 와 같은 모양이어야 한다는 요구는
[self_description.md §5](self_description.md) 의 동등성 가드가 담당한다.
**브리지는 자기서술을 생산하지 않는다 — 운반만 한다.** 이 분리를 지키지 않으면
목록이 세 벌(CLI·WASM·브리지)이 된다.

CLI 도구 39개 중 브라우저에서 의미가 성립하는 것만 나간다. `path` 를 받는 도구는
전부 `data: Uint8Array` 로 바뀌므로 **`inputSchema` 가 달라진다** — 이 차이는
자기서술 층에서 이미 처리된다([self_description.md §4.3](self_description.md)).

### 4.4 재사용 지점과 새로 필요한 것

| 항목 | 상태 | 근거 |
| --- | --- | --- |
| 버전 협상 | **재사용** | `protocol.ts:60-83` |
| capability 협상 | **재사용** | `protocol.ts:2-7`, `runtime.ts:75-92` |
| origin 고정 | **재사용** | `runtime.ts:130-133` |
| 세션 ID | **재사용** | `transport.js:18-25` |
| 요청 ID 다중화 | **재사용** | `protocol.ts:23-30` |
| transferable 이진 | **재사용** | `runtime.ts:23-30`, `transport.js:27-38` |
| 메서드별 타임아웃 | **재사용** | `transport.js:8-16` |
| JSON-RPC 2.0 봉투 | 신규 | — |
| `tools/list` 운반 | 신규 (얇음) | 자기서술이 생산 |
| 도구 이름 ↔ 라우터 매핑 | 신규 | `rpc-router.ts` 확장 |
| 에이전트 동사 라우팅 | 신규 | wasm_api 에 이미 있는 것부터 |

**새로 짜야 하는 코드는 놀랄 만큼 적다.** 어려운 부분(협상·경계·수명)은 이미 있다.

### 4.5 이진 데이터

MCP 는 JSON 이므로 이진을 base64 로 싣는다. 브라우저에서는 그럴 이유가 없다.

- 요청: `loadFile` 이 이미 `Uint8Array`/`ArrayBuffer` 를 받는다(`rpc-router.ts:56-61`).
  `allowLegacyArray` 는 레거시 경로 전용이다.
- 응답: `postPortResponse`(`runtime.ts:23-30`)가 `Uint8Array` 결과를 **복사한 뒤
  transfer** 한다.

**따라서 `tools/call` 의 결과가 이진이면 base64 로 바꾸지 않는다.** 대신 MCP
`content` 배열에서 그 자리를 참조로 두고 실제 바이트는 transfer 로 보낸다.
표준 MCP 클라이언트와의 호환성이 여기서 갈리며, **어떻게 표현할지는 확인되지 않음** —
설계 시 결정해야 할 열린 항목이다.

---

## 5. 보안 경계

브라우저는 **새 경계 하나**를 추가한다: origin. 위협 모델의 전제는
[../agent_security/threat_model.md](../agent_security/threat_model.md) 를 그대로 따르고,
여기서는 **브라우저에서만 생기는 것**만 적는다. 구현 축은
[#3787](https://github.com/edwardkim/rhwp/issues/3787).

### 5.1 지금 지켜지는 것 (실측)

| 방어 | 코드 | 무엇을 막나 |
| --- | --- | --- |
| 발신자 창 고정 | `runtime.ts:117-127` — `event.source !== options.parentWindow` 면 즉시 반환 | 임의 창의 명령 주입 |
| origin 스킴 제한 | `protocol.ts:95-103` `isUsableParentOrigin` — `http:`/`https:` 만, `"null"` 거부 | sandbox·`file:` origin 의 불투명 발신자 |
| **첫 origin 에 고정** | `runtime.ts:130-133` — `binding` 이 있으면 다른 origin 은 무시 | 세션 중간의 origin 전환 |
| 단일 바인딩 | `runtime.ts:143-146` — 이미 바인딩되면 새 포트를 close | 동시 다중 소비자 |
| 미사용 포트 폐기 | `runtime.ts:125`·`129` `releasePorts` | 떠도는 포트 누수 |
| 바인딩 후 레거시 무시 | `runtime.ts:152` | 포트 우회 다운그레이드 |
| 미지 메서드 거부 | `rpc-router.ts:100` | 임의 함수 호출 |
| 응답 origin 지정 | `runtime.ts:110` — `{targetOrigin: event.origin}` | 응답 광역 브로드캐스트 |

**MCP-유사 브리지는 이 여덟 개를 하나도 약화시키지 않는다.** 특히 "첫 origin 고정"은
JSON-RPC 모드에서도 그대로다 — 포트가 이미 바인딩된 뒤에 모드가 결정되기 때문이다.

### 5.2 문서 내용이 새는 경로

브리지가 열리면 **문서 본문이 RPC 응답으로 나간다.** 지금은 페이지 SVG·내보내기
바이트뿐이지만, 에이전트 동사가 붙으면 `getTextRange`·`searchAllText`·`getFieldList`
결과가 나간다. 즉 **텍스트가 경계를 넘는다.**

경계를 넘는 지점은 정확히 두 곳이다.

1. `postPortResponse`(`runtime.ts:23-30`) — 포트로. 수신자는 **바인딩된 origin 하나**.
2. `handleLegacy`(`runtime.ts:110`) — `targetOrigin: event.origin` 으로. 역시 하나.

따라서 **"어느 origin 이 문서를 볼 수 있나"는 첫 연결 시점에 결정되고 바뀌지 않는다.**
이건 좋은 성질이며, 브리지 설계에서 **반드시 보존해야 하는 불변식**이다.

지켜야 할 규칙:

- **응답 `targetOrigin` 에 `"*"` 를 쓰지 않는다.** 한 번이라도 쓰면 위 불변식이 무너진다.
- **`tools/list` 는 문서 내용을 담지 않는다.** 자기서술은 문서와 무관해야 한다.
  담기면 문서를 열지 않은 소비자에게도 내용이 샌다.
- **오류 메시지에 문서 텍스트를 넣지 않는다.** 현재 `RPC_ERROR` 는 `error.message` 에
  예외 문자열을 그대로 싣는다(`runtime.ts:64`). 코어 오류가 문서 조각을 포함하면
  그게 유출 경로가 된다. **확인되지 않음** — 실제로 포함하는지 대조하지 않았다.

### 5.3 위협 모델과의 접속

문서는 공격자가 만들 수 있다는 전제
([threat_model.md](../agent_security/threat_model.md))는 브라우저에서 더 날카롭다.

- **인젝션 판정이 브라우저에 없다.** `inspect` 계열이 WASM 에 노출돼 있지 않다
  ([self_description.md §1.3](self_description.md)). 즉 브리지를 통해 나가는 텍스트에는
  **아무 판정도 붙지 않는다.**
- **출처 표지도 없다.** `untrustedContent`/`untrustedFields` 가 WASM 봉투에 없다
  (`grep` 결과 0). 소비 에이전트는 받은 문자열이 문서에서 왔는지 알 수 없다.

> **따라서 브리지를 먼저 열고 판정을 나중에 붙이면, 그 사이 기간 동안 rhwp 는
> 판정 없는 문서 통로가 된다.** 순서를 지켜야 한다 — 자기서술(표지 포함) → 브리지.

소비자 쪽 책임 경계는 [consumer_guide.md](../agent_security/consumer_guide.md) 를 따른다.
브리지는 **경계까지**만 책임진다.

### 5.4 CSP

확장은 WASM 실행을 명시적으로 허용한다.

```
"content_security_policy": {
  "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
}
```

`rhwp-chrome/manifest.json:42-44`, `rhwp-firefox/manifest.json:51-53`.

`'wasm-unsafe-eval'` 없이는 WASM 이 돌지 않는다. 데모 페이지를 호스팅할 때도
같은 제약이 걸린다 — [zero_install_onboarding.md §4](zero_install_onboarding.md).

studio 의 `index.html` 은 인라인 스크립트를 피하도록 이미 작성돼 있다
(`rhwp-studio/index.html:8-10` 주석: "확장 CSP가 인라인 스크립트를 금지하므로 외부
파일로 분리(#1444)"). 브리지 코드도 **인라인 금지 규칙을 승계**해야 한다.

### 5.5 초기에 하지 않는 것

- **`resources/*` 를 노출하지 않는다.** MCP resources 는 "에이전트가 문서를 도구로 읽는"
  표면이다(#3608 Stage 7). 브라우저에서 이걸 열면 origin 경계와 문서 경계가 겹쳐
  경로 분석이 복잡해진다. 자기서술과 `tools/*` 가 안정된 뒤에 재검토한다.
- **여러 소비자 동시 바인딩을 허용하지 않는다.** 현재 단일 바인딩
  (`runtime.ts:143-146`)을 유지한다. 다중화는 "누가 무엇을 봤나"를 추적 불가능하게 만든다.
- **확장 background 경로를 겸하지 않는다.** 그건 `chrome.runtime` 계보이고
  `sender-validator.js` 가 담당한다(§1.5). 두 계보를 한 라우터에 합치면 발신자 검증
  규칙이 섞인다.

---

## 6. 조각 분해

| # | 조각 | 검증 |
| --- | --- | --- |
| B1 | `EMBED_CAPABILITIES` 에 `mcp-jsonrpc-v1` 추가 + 협상 분기 | 미지원 클라이언트가 종전대로 동작 (`e2e:embed`) |
| B2 | `bindPort` 의 JSON-RPC 모드 — `initialize` 응답만 | 핸드셰이크 왕복 테스트 |
| B3 | `tools/list` — 자기서술 결과 운반 (§4.3) | CLI `capabilities --mcp` 와 도구 이름 집합 대조 |
| B4 | `tools/call` — 기존 11개 메서드 먼저 | 각 메서드 왕복 |
| B5 | 에이전트 동사 라우팅 — WASM 에 **이미 있는 것**부터 (`searchAllText`·`getFieldList`) | 봉투 동등성 |
| B6 | 없는 동사 — 자기서술 S5 완료 후 | — |
| B7 | 오류 메시지 유출 점검 (§5.2) | 문서 텍스트 미포함 단언 |

**B3 을 B5 보다 먼저** 한다. 목록 없이 동사를 열면 소비자가 하드코딩하고, 그 순간
자기서술이 장식이 된다.

---

## 7. 확인되지 않음

1. **확장 background(service worker)에서 `MessageChannel` 브리지가 성립하는가** —
   `sender-validator.js` 계보와 어떻게 만나는지 미조사.
2. **`RPC_ERROR` 메시지가 문서 텍스트를 포함하는가** (`runtime.ts:64`) — 코어 오류
   문자열을 전수 대조하지 않았다.
3. **이진 결과의 MCP `content` 표현** — transfer 와 표준 호환의 절충안 미정(§4.5).
4. **동시 다중 소비자 요구가 실제로 있는가** — 지금은 단일 바인딩으로 충분하다고
   가정했다.
5. **브리지 왕복 지연** — 측정하지 않았다. 이 문서에 성능 주장이 없는 이유다.
6. **`e2e/embed-transport.test.mjs` 의 커버리지 범위** — 파일 존재와 스크립트 등록만
   확인했고 내용은 읽지 않았다.

---

## 8. 관련 문서

- [README.md](README.md) — 이 축의 지도
- [self_description.md](self_description.md) — 브리지가 **운반할 목록**을 정한다
- [zero_install_onboarding.md](zero_install_onboarding.md) — 브리지가 실려 나가는 배포 형태
- [../agent_security/threat_model.md](../agent_security/threat_model.md) — 문서 신뢰 전제
- [../agent_security/attack_surface.md](../agent_security/attack_surface.md) — 표면별 노출 매핑
- [../agent_security/consumer_guide.md](../agent_security/consumer_guide.md) — 소비자 책임
- [../agent_boundary_contract.md](../agent_boundary_contract.md) — 경계 무결성 계약
- 이슈 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M24 ·
  [#3869](https://github.com/edwardkim/rhwp/issues/3869) ·
  [#3787](https://github.com/edwardkim/rhwp/issues/3787)
