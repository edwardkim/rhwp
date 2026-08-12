---
kind: guide
status: active
canonical: mydocs/tech/agent_runtime/surface_spec.md
last_verified: 2026-08-03
---

# 에이전트 런타임 문서 지도

> **v0.8.4 현행성 주의:** 이 축의 Python·Node 바인딩 비교는 철회 전 설계 이력이다.
> #4655 이후 현재 공식 진입로는 CLI·MCP·WASM이며 바인딩 경로를 실행 지침으로 쓰지 않는다.

`mydocs/tech/agent_runtime/` 는 **에이전트가 rhwp 를 어떤 길로 부르는가**를 다룬다.
로드맵 [#3869](https://github.com/edwardkim/rhwp/issues/3869) "설치 없는 실행"의 축이며,
두 갈래로 나뉜다.

- **설계 갈래** — 실행 파일 없이 부르는 새 표면(WASM)의 계약:
  [surface_spec.md](surface_spec.md), [envelope_parity.md](envelope_parity.md).
- **운용 갈래** — 지금 있는 진입로들을 어떻게 고르고 무엇이 깨지는지:
  [entrypoint_decision.md](entrypoint_decision.md), [cost_model.md](cost_model.md),
  [failure_dictionary.md](failure_dictionary.md).

둘은 같은 질문의 앞뒤다. 운용 갈래가 **"지금 무엇을 쓸 수 있나"** 를 답하고,
설계 갈래가 **"아무것도 쓸 수 없을 때 무엇을 만들 것인가"** 를 답한다.

## 왜 이 축이 생겼는가

rhwp의 공식 실행 진입로인 CLI와 MCP는 같은 실행 파일 관문을 공유한다.
([entrypoint_decision.md](entrypoint_decision.md)는 이를 `CLI 단건`·`CLI batch`·
`MCP 무상태`·`MCP 세션`·`run`으로 나눈다.)

| 진입로 | 전제 | 근거 |
| --- | --- | --- |
| CLI | `rhwp` 실행 파일이 `PATH` 에 있다 | `Cargo.toml:17-19` `[[bin]] name = "rhwp"` |
| MCP | 호스트가 `rhwp mcp-serve` 를 자식으로 띄운다 | `src/mcp_serve.rs` 가 stdio JSON-RPC 서버 |

둘 다 **바이너리를 구한 뒤에야** 시작된다. 임의 실행 파일 반입이 막힌 샌드박스나
프로세스 생성 자체가 없는 런타임에서 도는 에이전트에게 이 관문은 곧 벽이다.

```
        ┌───────────────────────────────────────────┐
문서 ──▶ │  rhwp 실행 파일을 먼저 구한다  ← 공통 관문 │ ──▶ CLI / MCP
        └───────────────────────────────────────────┘
                         ✕ 샌드박스 안 에이전트

문서 ──▶ [ WASM 모듈 ] ──▶ 에이전트 동사              ← 이 축이 설계하는 다섯 번째 길
```

rhwp 는 **이미 WASM 으로 컴파일된다**(`src/wasm_api.rs` 7,621줄, `wasm_bindgen`
372회, 명시 `js_name` export 364개 — 전부 실측). 그런데 그 표면은 **렌더링 지향**
이다(`get*` 121·`set*` 42·`render*` 14). 그리고 결정적으로, 그 표면에는
`schemaVersion` 이 **한 번도 나오지 않는다**(`grep -c`: `wasm_api.rs` 0회,
`main.rs` 112회). **에이전트가 읽는 봉투가 없다는 뜻이다.**

## 읽는 순서

| 순서 | 문서 | 갈래 | 언제 읽나 |
| --- | --- | --- | --- |
| 1 | [표면 명세](surface_spec.md) | 설계 | **이 축의 권위 문서.** 어떤 동사가 표면에 있고 왜 나머지는 없는지 |
| 2 | [봉투 동등성 계약](envelope_parity.md) | 설계 | 반환값이 CLI `--json` 과 같은 모양이어야 하는 이유와 강제 방법 |
| A | [진입로 판단표](entrypoint_decision.md) | 운용 | **지금 이 작업에 무엇을 쓸지** 고를 때. 실무 진입점 |
| B | [진입로 비용 모델](cost_model.md) | 운용 | 판단표의 성능 주장이 어디서 나왔는지 확인할 때 |
| C | [진입로별 실패 사전](failure_dictionary.md) | 운용 | 오류 문자열을 그대로 검색해 원인을 찾을 때 |
| — | 이 문서 | — | 축의 경계가 헷갈릴 때 |

**지금 rhwp 를 쓰는 사람**은 A 부터 읽는다. 1·2 는 아직 **계약뿐**이라 오늘 부를 수
있는 것이 아니다(아래 "지금 상태"). **새 표면 구현에 들어가는 사람**은 1 → 2 순으로
읽고, 2 의 §5(금지 규칙)와 §6(계약 테스트)을 다시 본다.

## 설계 갈래가 고정하는 것

네 가지가 [surface_spec.md](surface_spec.md)·[envelope_parity.md](envelope_parity.md)
를 관통한다.

**① 31개를 다 노출하지 않는다.**
`--json` 명령 31개는 CLI 라는 실행 맥락에서 자란 목록이고, 그 맥락의 절반
(프로세스·파일시스템·병렬·stderr)이 WASM 에 없다. 세 관문(바이트만으로 성립하는가 /
에이전트 동사인가 / 경계를 넘길 값이 컨텍스트에 들어갈 만한가)을 전부 통과하는
**18개만** 넣고 13개는 뺐다. 뺀 것마다 근거를 적었다
([surface_spec.md](surface_spec.md) §3).

**② 봉투는 CLI 와 같은 모양이다.**
필드 이름·타입·`null` 의미·빈 배열 규약이 같다. 다를 수밖에 없는 것(`source`·
exit code·파일 산출)은 **규칙으로 매핑**하고 그 규칙에 번호를 붙였다
([envelope_parity.md](envelope_parity.md) §4 R1~R7).

**③ 판정은 반환값, 실패는 예외.**
CLI exit 3/4(검증 단언 실패)는 **도구가 정상 동작한 결과**이므로 봉투 필드로
반환한다. exit 1/2만 실패로 다룬다. WASM도 같은 판정 의미를 유지한다.

**④ 근거 없는 주장을 적지 않는다.**
모든 기술 주장에 `파일:줄` 또는 실제 명령 출력이 붙는다. 대지 못하면
**"확인되지 않음"** 이라고 적는다. 성능은 **측정한 것만** 적는다 — 이 축의 문서는
WASM 성능을 한 번도 재지 않았고, 그래서 재지 않았다고 적혀 있다.

## 이 축이 무엇이 **아닌지**

여기가 이 문서에서 가장 중요한 절이다.

**① 기존 진입로의 대체가 아니다.**
CLI와 `mcp-serve`는 그대로 남는다. 이 축은 기존 진입로의 후계자가 아니다.
표면에서 뺀 13개 명령(`export-pdf`·`batch`·`build-from-ingest` 등)을 쓰려면
CLI 또는 MCP의 해당 표면을 쓴다.

**② 렌더링 WASM 표면의 개편이 아니다.**
`renderPageToCanvas` 계열 364개 export 는 rhwp-studio 의 계약이다. 이 축은 그것을
건드리지 않고 **옆에 새 표면을 놓는다.**

**③ 성능 개선 축이 아니다.**
"WASM 이 빠르다"는 이 축의 주장이 아니다. WASM 은 느리고, 얼마나 느린지는
**측정하지 않았다**. 이 축이 파는 것은 속도가 아니라 **도달 가능성**이다 — 실행
파일을 못 구하는 곳에서 rhwp 를 부를 수 있느냐.

**④ 보안 축이 아니다 — 다만 보안과 무관하지도 않다.**
위협 모델은 [agent_security/](../agent_security/) 가 다룬다. 이 축은 그 축의 결론을
**전제로** 쓴다. 두 방향의 영향만 기록한다.

- **좋아지는 쪽**: 경로가 없으므로 [agent_boundary_contract.md](../agent_boundary_contract.md)
  의 S5(문서 내용이 파일 경로에 섞이는 위협)와 S8(핸들 위조)이 **구조적으로 소멸**한다.
- **살펴야 하는 쪽**: 비밀번호가 JS 문자열로 존재하는 순간의 노출은 CLI 의
  `--password`(프로세스 목록 노출)와 다른 위협이다 — 판단은 보안 축이 한다
  ([surface_spec.md](surface_spec.md) §8 O4).

한 가지는 이 축이 적극적으로 주장한다: **`inspect` 축을 넣는 이유는 보안이다.**
`inspect` 는 문서 내용을 컨텍스트에 넣기 **전에** 부르는 동사인데, 실행 파일을 못
구해 rhwp 를 못 쓰는 에이전트는 그 검사를 건너뛰고 문서를 읽는다. **"설치 없는
실행"은 편의 문제이자 보안 문제다.**

**⑤ 에이전트의 무능을 다루지 않는다.**
환각·검증 누락·재시도 루프는 [weak_agent_proofing.md](../weak_agent_proofing.md) 의
주제다. 이 축은 그 문서가 만든 계약(`capabilities` 자기서술·`nextStep`·`changedPages`·
절단 표기)을 **그대로 옮긴다.** 새로 만들지 않는다.

## 지금 상태

**설계 갈래는 계약이고, 구현은 아직 없다.** (운용 갈래 A·B·C 는 오늘 동작하는
진입로를 서술하므로 그대로 쓸 수 있다.)

| 항목 | 상태 |
| --- | --- |
| 동사 목록과 제외 근거 | **확정** ([surface_spec.md](surface_spec.md) §3) |
| 봉투 매핑 규칙 R1~R7 | **확정** ([envelope_parity.md](envelope_parity.md) §4) |
| 판정/실패 규칙 V1~V3 | **확정** (같은 문서 §3) |
| WASM 동사 구현 | **없음** |
| 계약 테스트 | **설계만** (같은 문서 §6) |
| WASM 성능·메모리 측정 | **안 함** ([surface_spec.md](surface_spec.md) §7) — 기존 진입로의 실측은 [cost_model.md](cost_model.md) |

**구현과의 관계는 다른 축들과 같다** — 이 문서 축이 계약이고 코드가 그 구현이다.
충돌하면 문서를 고치고 코드를 맞춘다([`CLAUDE.md`](../../../CLAUDE.md) 의 canonical
규칙). 아직 없는 것은 각 문서에서 **"설계된 것"으로 명시**했다. 현재 동작은 언제나
`rhwp capabilities`·`rhwp --help`·바이너리 실행으로 재확인한다.

> v0.8.4에서 공식 Python·Node 바인딩은 철회됐다(#4655). 아래 명세의 현재 지원
> 진입로는 CLI·MCP·WASM이며, 바인딩을 근거로 든 본문은 철회 전 설계 이력이다.

## 인접 문서

- [agent_security/](../agent_security/) — 위협 모델. 이 축은 그 결론을 전제로 쓴다
- [agent_boundary_contract.md](../agent_boundary_contract.md) — S5 경로·S7 자원 한계·S8 핸들
- [envelope_provenance.md](../envelope_provenance.md) — `untrustedContent`/`untrustedFields` 계약의 단일 출처
- [weak_agent_proofing.md](../weak_agent_proofing.md) — 경량 에이전트 내성. 이 축이 옮겨 쓰는 계약의 출처
- [mydocs/manual/cli_commands.md](../../manual/cli_commands.md) — 31개 `--json` 명령의 사람용 계약. **정확한 인자는 언제나 여기와 `--help` 가 기준**
- [mydocs/manual/dev_environment_guide.md](../../manual/dev_environment_guide.md) — `wasm-pack build --target web` 절차
- 이슈 [#3869](https://github.com/edwardkim/rhwp/issues/3869) — 로드맵 "설치 없는 실행"
