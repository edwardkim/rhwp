---
kind: canonical
status: active
canonical: mydocs/tech/agent_architecture/mcp_spec_ledger.md
last_verified: 2026-08-08
---

# MCP 스펙 개정 추종 대장

**MCP(Model Context Protocol) 스펙이 개정될 때마다 "우리 표면에 무엇이 닿는가"를
한 곳에서 판정하는 대장이다.** [#4220](https://github.com/edwardkim/rhwp/issues/4220) T1.

서버는 개정판 하나를 통째로 말한다 — `protocolVersion` 은 원자 단위라서, 개정 항목을
골라 반쯤 추종하면 "2025-06-18 이라고 광고하면서 2026-07-28 의 몸통을 내보내는" 서버가
된다(그 위험은 `src/mcp_serve.rs` 의 `SUPPORTED_PROTOCOL_VERSIONS` 주석이 이미
문서화했다). 그래서 이 대장의 단위는 **항목이 아니라 개정판**이고, 각 개정판 행은
"우리 표면에 닿는 항목 전부"를 열거한다.

이 대장이 낡으면 계약 테스트 `tests/mcp_spec_ledger_contract.rs` 가 red 가 된다 —
§5 기계 대사 구역이 그 접점이다. [R75 결정 D-41 제안(PR #4206)](https://github.com/edwardkim/rhwp/pull/4206)이
"스펙 개정發 수선 반복"을 rmcp 재평가 트리거로 명시했는데, 이 대장이 바로 그 반복을
세는 측정 장치다.

---

## 1. 서버가 구현한 개정판과 표면 — 실측 (2026-08-08, devel dc7d7adcc)

**구현 개정판: `2025-06-18` 단일.** 전송은 stdio(줄 단위 JSON-RPC 2.0) 뿐이다 —
Streamable HTTP·OAuth·SSE 표면이 없으므로, 개정판의 HTTP·인가 축 변경은 이 대장에서
"해당 없음"으로 접는다.

| 표면 | 위치(실측) | 스펙 근거 |
|---|---|---|
| `PROTOCOL_VERSION = "2025-06-18"` | `src/mcp_serve.rs:24` | versioning |
| `SUPPORTED_PROTOCOL_VERSIONS`(단일 원소) | `src/mcp_serve.rs:32` | lifecycle §Version Negotiation |
| JSON-RPC 예약 코드 -32700/-32600/-32601/-32602 | `src/mcp_serve.rs:34~37` | JSON-RPC 2.0 |
| `RESOURCE_NOT_FOUND = -32002` 선언·사용 | `src/mcp_serve.rs:39`·`471` | 2025-06-18 resources 규약 |
| JSON-RPC 배치 거부(개정판 사유 문구) | `src/mcp_serve.rs:145~161` | 2025-06-18 changelog "Remove support for JSON-RPC batching" |
| `initialize` — 버전 협상·capabilities `{tools:{},resources:{}}`·serverInfo | `src/mcp_serve.rs:185~197`·`256~263` | lifecycle |
| `ping` | `src/mcp_serve.rs:198` | utilities/ping |
| `tools/list` · `tools/call` | `src/mcp_serve.rs:199~210` | server/tools |
| `resources/list` · `resources/read` | `src/mcp_serve.rs:211~219` | server/resources |
| `structuredContent`(JSON stdout 재파싱 절약) | `src/mcp_serve.rs:761~765` | 2025-06-18 신설 |
| 알림(id 없음) 무응답 — `notifications/initialized` 포함 | `src/mcp_serve.rs:167~170` | lifecycle |
| 도구 인자 형식 오류는 `isError`(프로토콜 오류 아님) | `tests/mcp_arg_validation_contract.rs:96`·`155` | 2025-11-25 SEP-1303 과 이미 정합 |

이 값을 고정한 계약 테스트(#4116 축):

- `tests/mcp_server_contract.rs:18` — `SERVER_PROTOCOL_VERSION = "2025-06-18"` 상수,
  `:509~` 버전 협상, `:610~624` 배치 거부, `:781~782` 미지 리소스 -32002
- `tests/mcp_resources_contract.rs:162~163` — 미지 스키마 URI -32002
- 계약 테스트 **12파일**이 `"2025-06-18"` 문자열로 `initialize` 한다(전수는
  `grep -rln '"initialize"' tests/*.rs`)

문서 접점: [플레이북](../../manual/agent_surface_playbook.md) `:229~232`,
[실패 사전](../../manual/agent_troubleshooting_guide.md) `:1219~1222` 가
협상 응답 예시로 이 개정판 문자열을 싣고 있다.

---

## 2. 개정판별 차이 대장 — 우리 표면에 닿는 항목만

공식 개정 연혁(1차 출처, 2026-08-08 확인):
[versioning](https://modelcontextprotocol.io/specification/versioning) 이 밝히는 현행판은
**2026-07-28**(확정 공지: [공식 블로그 2026-07-28](https://blog.modelcontextprotocol.io/posts/2026-07-28/)).
우리 구현판과 현행판 사이에는 개정이 **둘** 있다: `2025-11-25` → `2026-07-28`.

### 2-1. `2025-06-18` → `2025-11-25`

출처: [2025-11-25 changelog](https://modelcontextprotocol.io/specification/2025-11-25/changelog) (2026-08-08 확인).
아이콘 메타데이터·URL elicitation·sampling 도구 호출·tasks(실험)·OAuth 축 강화가
큰 줄기인데, **대부분 우리 표면(stdio·tools·resources) 밖이다.** 닿는 것만 적는다.

| 바뀐 것 | 우리 현재 값(실측) | 추종 시 바꿀 지점 |
|---|---|---|
| 입력 검증 오류는 프로토콜 오류가 아니라 Tool Execution Error(`isError`)로 — SEP-1303 | **이미 그렇게 한다** — 인자 형식 오류는 `isError` (`tests/mcp_arg_validation_contract.rs:96`) | 없음(정합 확인만) |
| stdio 서버의 stderr 는 모든 로깅에 사용 가능(명확화) | stdout 순수성 유지, 진단은 stderr — 정합 | 없음 |
| `Implementation`(serverInfo)에 선택 `description` 필드 | 미포함(`src/mcp_serve.rs:192~196`) | 선택 — `initialize` 응답에 한 필드 |
| 도구 이름 지침(SEP-986) | `hwp_*` 단일 규칙 — 정합 | 없음 |
| JSON Schema 2020-12 를 기본 방언으로(SEP-1613) | inputSchema 는 방언 무선언(단순 object/properties) | 없음(2020-12 하위 호환 형태) |
| 협상 가능한 개정판 목록에 `2025-11-25` 추가 | `SUPPORTED_PROTOCOL_VERSIONS` 단일 원소(`src/mcp_serve.rs:32`) | `src/mcp_serve.rs:24·32` + §1 의 계약 테스트·문서 전부 |

**판정 메모** — 이 개정은 우리 표면 기준 **행동 변경이 사실상 0**이다(이미 정합 2건,
선택 1건). 추종의 실비용은 개정판 문자열 승격뿐이다.

### 2-2. `2025-11-25` → `2026-07-28` (현행 확정판)

출처: [2026-07-28 changelog](https://modelcontextprotocol.io/specification/2026-07-28/changelog) ·
[versioning-and-compatibility](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning)
(2026-08-08 확인). **이번엔 프로토콜 골격이 바뀐다** — 무상태화. 닿는 항목:

| 바뀐 것 | 우리 현재 값(실측) | 추종 시 바꿀 지점 |
|---|---|---|
| `initialize`/`notifications/initialized` 핸드셰이크 **제거** — 매 요청 `_meta` 에 `io.modelcontextprotocol/protocolVersion` 등 탑재(SEP-2575) | `initialize` 가 수명주기 입구(`src/mcp_serve.rs:185`), 알림 무응답(`:167~170`) | 디스패치 골격(`src/mcp_serve.rs:184~225`)·`negotiate_protocol_version`(`:256~263`)·initialize 를 쓰는 계약 테스트 12파일·플레이북/실패 사전 예시 |
| `server/discover` RPC **필수**(MUST) | 미구현 — 현재 `-32601 "지원하지 않는 메서드"` 응답(디스패치 fallthrough `:220~224`) | 신규 핸들러 + 계약 테스트 신설 |
| `ping` 제거 | 구현(`src/mcp_serve.rs:198`) | 핸들러 제거 또는 dual-era 유지 판단 |
| 모든 결과에 `resultType` 필수(`"complete"`/`"input_required"`, SEP-2322) | 미탑재 — `ok_response` 공통 봉투(`src/mcp_serve.rs:238~240`) | `ok_response` 한 곳 + 응답 형태를 고정한 계약 테스트 |
| `tools/list`·`resources/list`·`resources/read` 결과에 `ttlMs`·`cacheScope` 필수(CacheableResult, SEP-2549) | 미탑재(`src/mcp_serve.rs:199~219`) | 세 핸들러 + 계약 테스트 |
| **미지 리소스 오류 코드 `-32002` → `-32602`** (JSON-RPC 정합화) | `RESOURCE_NOT_FOUND = -32002`(`src/mcp_serve.rs:39`, 사용 `:471`) | `src/mcp_serve.rs:39` + `tests/mcp_server_contract.rs:781`·`tests/mcp_resources_contract.rs:162` — **#4116 이 고정한 값 직격** |
| 오류 코드 대역 정책: `-32020~-32099` 는 MCP 예약, `UnsupportedProtocolVersion = -32022` | 자체 코드는 예약 코드 5종뿐 — 예약 대역 침범 없음 | 추종 시 `-32022` 오류 신설 |
| 프로토콜 수준 세션 제거 — 상태는 "서버가 발급한 핸들을 **일반 도구 인자**로"(SEP-2567) | `hwp_open` 이 핸들을 발급하고 도구 인자로 받는다 — **정확히 그 처방 패턴** | 없음(설계 정합 — stdio 라 `Mcp-Session-Id` 무관) |
| `tools/list` 결정론적 순서 SHOULD | 정적 선언 배열 순서 — 정합 | 없음 |
| `structuredContent` 임의 JSON 값 허용(완화) | JSON object 만 내보냄 — 하위 집합이라 정합 | 없음(제약 완화 여지만) |
| Roots·Sampling·Logging deprecated(12개월 창) | 셋 다 미구현 | 없음 |
| MRTR(서버발 요청 → `input_required` 재시도 패턴) | 서버발 요청 미사용 | `resultType` 채택에 포함 |

**하위 호환 실측 근거** — [2026-07-28 호환성 매트릭스](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning#backward-compatibility-with-initialization-based-versions):
dual-era 클라이언트는 stdio 에서 `server/discover` 를 먼저 던져 보고, **"인식되는
modern 오류가 아닌" 응답이면 legacy 로 판정해 `initialize` 로 폴백한다.** 우리 서버의
`-32601` 이 정확히 그 legacy 신호다 — 즉 dual-era 클라이언트와는 **현행 서버가 스펙이
정의한 경로로 계속 동작한다.** 끊기는 것은 modern 전용 클라이언트뿐이다.

---

## 3. 추종 판정 절차 — 새 개정이 나오면

1. **감지** — [versioning](https://modelcontextprotocol.io/specification/versioning) 의
   현행판 표기와 [공식 블로그](https://blog.modelcontextprotocol.io/) 확정 공지를
   1차 출처로 삼는다(확인 날짜를 함께 적는다).
2. **행 추가** — §2 에 개정판 행을 추가한다: 바뀐 것 / 우리 현재 값(실측 줄번호) /
   추종 시 바꿀 지점(코드·계약 테스트·문서). §1 표면 목록 밖의 변경은
   "해당 없음"으로 명시해 침묵과 구분한다.
3. **영향 열거** — 바꿀 지점은 반드시 세 축을 다 적는다: `src/mcp_serve.rs` 의
   실측 줄, 그 값을 고정한 계약 테스트 파일, 개정판 문자열을 싣는 문서.
4. **판정은 메인테이너** — 추종/보류는 이 대장이 정하지 않는다. 판정에 필요한
   재료(호환성 경로·클라이언트 생태계 채택·바꿀 지점의 크기)만 준비한다.
5. **기록** — 판정 결과와 근거를 이 대장과 [결정 대장](decision_log.md)에 남기고,
   채택 시 §5 기계 대사 구역을 함께 갱신한다(안 하면 계약 테스트가 red 로 막는다).
   개정 추종으로 수선이 반복되면 그 횟수가 D-41(rmcp 재평가)의 트리거 입력이 된다.

## 4. 현행 판정 제안 (2026-08-08)

**제안: `2025-06-18` 유지, 확정판(2026-07-28) 추종은 클라이언트 생태계 채택 실측과
함께 메인테이너가 판정한다.** 근거:

- **지금 안 끊긴다** — §2-2 하위 호환 실측 근거대로, dual-era 클라이언트는
  `server/discover` 실패를 보고 `initialize` 로 폴백한다. 확정 공지도 Tier 1 SDK 의
  dual-era 지원과 12개월 deprecation 창을 명시했다.
- **부분 선반영은 금지** — 예컨대 `-32002 → -32602` 만 미리 바꾸면 2025-06-18 을
  광고하면서 그 개정판의 resources 규약을 어기는 서버가 된다. 개정판은 원자 단위다.
- **재평가 트리거(실측 가능)** — ① 주요 MCP 호스트가 modern 전용(legacy 폴백 제거)
  전환을 공지할 때 ② rhwp 사용자의 modern 전용 클라이언트 접속 실패가 보고될 때
  ③ `2025-11-25` 승격만 먼저 요구되는 실사용이 나타날 때(§2-1 — 실비용이 문자열
  승격뿐이라 독립 판정 가능).

## 5. 기계 대사 구역 — `tests/mcp_spec_ledger_contract.rs` 가 파싱한다

아래 블록은 계약 테스트가 실물(서버 광고 값·소스·계약 테스트 파일)과 대사한다.
**본문 서술을 고치면 이 블록을 함께 고친다** — 어긋나면 테스트가 red 다.

<!-- MACHINE-LEDGER-BEGIN -->
```text
implemented-revision: 2025-06-18
resource-not-found-code: -32002
resource-not-found-test-files: tests/mcp_server_contract.rs, tests/mcp_resources_contract.rs
serve-methods: initialize, ping, tools/list, tools/call, resources/list, resources/read
```
<!-- MACHINE-LEDGER-END -->

- `implemented-revision` — 서버가 광고하는 `protocolVersion`. `src/mcp_serve.rs` 의
  `PROTOCOL_VERSION` 상수·`tests/mcp_server_contract.rs` 의 `SERVER_PROTOCOL_VERSION`
  상수·살아있는 서버의 `initialize` 응답, 세 곳 모두와 대사한다.
- `resource-not-found-code` — 미지 리소스 오류 코드. 소스 상수·살아있는 서버 응답과
  대사하고, 이 코드 문자열을 싣는 계약 테스트 파일 집합이
  `resource-not-found-test-files` 와 일치하는지 `tests/*.rs` 전수 grep 으로 확인한다.
- `serve-methods` — 디스패치가 실제로 받는 프로토콜 메서드 전수.
  `src/mcp_serve.rs` 의 match 팔과 집합 일치로 대사한다.
