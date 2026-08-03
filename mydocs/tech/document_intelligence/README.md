---
kind: guide
status: active
canonical: mydocs/tech/document_intelligence/README.md
last_verified: 2026-08-03
---

# 문서 지능 서버 문서 지도 (M25)

`mydocs/tech/document_intelligence/`는 로드맵
[#3608](https://github.com/edwardkim/rhwp/issues/3608) **M25 "문서 지능 서버(LSP 유사)"**
의 설계를 담는다. 세 항목이 미완이고, 각각 문서 하나를 가진다.

> **이 디렉터리의 문서는 전부 설계다 — 구현이 아니다.** 여기 적힌 도구 이름·봉투 필드는
> 아직 `mcp-serve` 에 없다. "지금 있는 것"은 코드 경로(`파일:줄`)로, "설계된 것"은
> 제안으로 명시해 갈라 적는다. 지금 무엇이 있는지는 [MCP 연동
> 가이드](../../manual/mcp_integration_guide.md)와 [에이전트 지식
> 지도](../../manual/agent_knowledge_map.md)가 권위다.

## 왜 M25 인가

LSP(Language Server Protocol)는 에디터가 파일을 열어 둔 채 **서버가 상태를 들고** 있고,
편집이 들어오면 증분으로 갱신하며, "이 심볼을 누가 참조하나"에 답한다. rhwp 의 세션
도구(#3571·#3598·#3601·#3609)는 그중 **첫 칸**을 이미 채웠다 — 문서를 한 번 파싱해
`docId` 로 잡아 두고 재파싱 없이 반복 조회한다.

남은 세 칸이 M25 다.

| 항목 | 한 줄 | 문서 |
|---|---|---|
| 장수명 서버 모드 확장 | 열어 둔 문서가 디스크에서 바뀌면 어떻게 아는가 | [incremental_reparse.md](incremental_reparse.md) |
| 다문서 워크스페이스 핸들 | `docId` 하나가 문서 하나 — 프로젝트 단위는 무엇인가 | [workspace_handles.md](workspace_handles.md) |
| 참조 조회 | "어느 문서가 이 서식을 인용하나" 에 답할 수 있나 | [reference_queries.md](reference_queries.md) |

## 지금 있는 것 — 세션 모델 요약

권위는 코드다(`src/mcp_serve.rs`). 설계 논의의 출발점이므로 사실만 옮긴다.

| 사실 | 근거 |
|---|---|
| 핸들 하나 = 문서 하나. `SessionDoc { doc, source_is_hwpx, size_bytes, detected_format }` | `src/mcp_serve.rs:41-49` |
| **원본 경로를 보관하지 않는다** — 응답 봉투의 `source` 는 그 호출에서만 쓰이고 버려진다 | `src/mcp_serve.rs:703-720` |
| `docId` 는 `doc-{단조증가}`. 닫은 번호는 재사용하지 않는다 | `src/mcp_serve.rs:701-702`, `1423` |
| 핸들 테이블은 `run()` 지역 변수 — 스레드 간 공유 장치가 없다 | `src/mcp_serve.rs:114` |
| 서버 루프는 stdin 한 줄 → 응답 한 줄의 단일 스레드 블로킹 | `src/mcp_serve.rs:116-227` |
| `initialize` 는 `resources` 의 `subscribe`/`listChanged` 를 **둘 다 미지원**으로 선언 | `src/mcp_serve.rs:189-191` |
| 세션 도구 12종(open/close + 조회 6 + 편집 3 + save) | `src/mcp_serve.rs:617-633` |
| TTL·개수 상한·메모리 상한 없음 | [에이전트 경계 무결성 계약 S8](../agent_boundary_contract.md#s8--핸들-무결성) |

## 지금 없는 것 — 세 공백의 구체

세 문서가 각각 길게 다루지만, 무엇이 없는지는 한 표로 보는 편이 빠르다.

| 공백 | 구체적으로 무엇이 없나 | 그래서 무슨 일이 생기나 |
|---|---|---|
| 핸들에 **원본 경로가 없다** | `SessionDoc` 에 `path`·`mtime`·해시가 없다(`src/mcp_serve.rs:41-49`) | 열어 둔 문서가 디스크에서 바뀌어도 서버가 모른다. 재적재·중복 제거·감시가 전부 이 한 필드에서 막힌다 |
| **묶음을 표현할 자리가 없다** | 서버 상태는 `HashMap<docId, SessionDoc>` 하나뿐(`src/mcp_serve.rs:51-64`) | 어느 핸들이 서식이고 어느 것이 산출인지가 에이전트 컨텍스트에만 있다 |
| **원본 복구(fork)가 없다** | 세션 편집은 IR 에 누적만 한다(`src/mcp_serve.rs:510`, `:1234-1351`) | 메일머지를 세션으로 못 돈다. 그래서 `batch fill` 이 행마다 서식을 다시 파싱한다 |
| **책갈피 조회 표면이 없다** | `get_bookmarks_native` 가 WASM 에만 노출(`src/wasm_api.rs:7534`) | 문서 안의 상호참조를 **볼 수는 있어도 대상을 못 찾는다** |
| **문서 간 링크 자체가 없다** | 포맷에 그런 구조가 없다 — 코퍼스 353건 실측 0건 | 참조 조회는 링크가 아니라 닮음으로만 답할 수 있다 |

## 실측 요약 (2026-08-03, Windows 11, `target/release/rhwp.exe` v0.8.2)

세 문서가 공유하는 숫자다. 측정 방법과 전체 표는 각 문서에 있다.

| 항목 | 값 | 쓰이는 곳 |
|---|---:|---|
| 프로세스 기동만(`--version`) | 95 ms | 무상태 호출의 고정 비용 |
| `hwp_open` — 10.6 MB·393쪽 HWP | 126~132 ms | 재파싱 비용의 기준선 |
| `hwp_open` — 13.6 MB·387쪽 HWPX | 195 ms | 〃 |
| `hwp_open` — 208 KB 서식 | 4.8 ms | 메일머지 단가 |
| 세션 조회(`hwp_doc_info`) | 0.7~1.9 ms | 재파싱 회피 이득 |
| 전 구역 dirty 후 재조판 | ≈64 ms | 증분 조판의 상한 |
| 파일 읽기(캐시 적중, 10.2 MB) | 3.1 ms | I/O 는 병목이 아니다 |
| 핸들당 RSS | 파일 크기의 1.4~7.4배 | 워크스페이스 메모리 예산 |
| `batch fill` 한계비용(10.6 MB 서식) | 137 ms/행 | 서식이 행마다 재파싱된다 |

세션 vs 무상태의 **비율 계약**은 이미 공개 벤치가 고정했다 —
[세션 핸들 vs 무상태 CLI 벤치](../../report/bench_session_vs_stateless/README.md)(97 KB·54쪽
서식에서 ×120.6). 이 디렉터리는 그 벤치를 **대형 문서와 다문서**로 확장한다.

### 측정 방법과 재현

- 무상태 CLI 는 bash 에서 `date +%s%N` 으로 감싸 7회 반복.
- 세션은 Python `subprocess` 로 `rhwp mcp-serve` 를 **실제 stdio 구동**하고
  `time.perf_counter()` 로 요청 write → 응답 readline 왕복을 잰다. 순수 함수 벤치가
  아니라 JSON-RPC 왕복을 포함한 값이다.
- RSS 는 `Get-Process -Id <pid>` 의 `WorkingSet64` — 프로세스 전체 값이라 IR 자체 크기가
  아니라 **상한에 가까운 관측값**이다.
- 코퍼스 스캔은 `ls samples/*.hwp samples/*.hwpx | rhwp batch fields --json` 의 NDJSON 을
  집계했다(353건 중 350건 파싱 성공, 3건 오류).
- **절대값은 이 머신 의존이다.** 실행 파일 기동만으로 95 ms 가 드는 환경이므로,
  계약으로 쓸 것은 언제나 **비율과 자릿수**다.

## 이 문서 묶음의 결정 요약

각 문서가 근거와 함께 내린 결론만 모았다. 근거는 해당 문서에 있다.

| 결정 | 근거 위치 |
|---|---|
| 편집 델타 기반 **증분 재파싱은 하지 않는다.** 파일이 바뀌면 통째로 다시 읽는다 | [incremental_reparse §3](incremental_reparse.md#3-lsp-의-증분과-hwp-의-증분은-같은-문제가-아니다) |
| 조판 증분은 **이미 있다**(구역 dirty). M25 가 다시 만들 것이 아니다 | [incremental_reparse §2.6](incremental_reparse.md#26-파싱-비용과-조판-비용의-분해) |
| 감시의 값은 속도가 아니라 **스테일을 말할 수 있게 되는 것** | [incremental_reparse §5](incremental_reparse.md#5-그래서-감시는-무엇을-위한-것인가) |
| **자동 재로딩 금지** — 누적 편집을 조용히 버린다 | [incremental_reparse §5.3](incremental_reparse.md#53-자동-재로딩을-하지-않는-이유) |
| 워크스페이스는 **폴더가 아니라 역할 있는 명시적 집합** | [workspace_handles §4.2](workspace_handles.md#42-결정--c역할-있는-명시적-집합) |
| 가장 값이 큰 조각은 **`hwp_doc_fork`** — 코어에 이미 있고, 메일머지 137 ms/행을 없앤다 | [workspace_handles §3.1](workspace_handles.md#31-메일머지--서식-1--데이터-n행) |
| 전건 스캔은 `batch`, 반복 접근은 워크스페이스 | [workspace_handles §3.3](workspace_handles.md#33-서식-일괄-점검--폴더-전체를-훑는다) |
| **문서 간 링크는 없다** — 링크 기반 참조 도구는 만들지 않는다 | [reference_queries §4](reference_queries.md#4-결론--링크는-없다) |
| 참조는 **닮음으로 근사**하고, 임계값은 도구가 정하지 않는다 | [reference_queries §5](reference_queries.md#5-그러면-무엇으로-근사하나) |
| 문서가 가리킨 경로를 **열지 않는다** | [reference_queries §7](reference_queries.md#7-보안-경계--문서가-가리킨-경로를-따라가지-않는다) |

## 세 문서가 공유하는 판단 기준

1. **없는 기능을 있는 것처럼 쓰지 않는다.** HWP 포맷에 없는 연결 고리를 설계로 만들어
   내지 않는다. 근사라면 근사라고 적고 오탐 성격을 같이 적는다.
2. **근거는 코드 경로 또는 실측.** 둘 다 못 대면 본문에 **"확인되지 않음"** 으로 남긴다.
3. **경계 계약을 먼저 읽는다.** 핸들·자원 한계·경로는 이미 계약이 있다
   ([agent_boundary_contract.md](../agent_boundary_contract.md)). M25 는 그 위에 얹는
   것이지 다시 정하는 것이 아니다.
4. **문서 내용은 데이터, 제어가 아니다.** 참조를 따라가는 기능은 문서가 지목한 문자열을
   **경로로 해석**하게 만든다 — [에이전트 보안 축](../agent_security/README.md)이 다루는
   바로 그 경계다.

## 읽는 순서

- 처음이면 이 README → [incremental_reparse.md](incremental_reparse.md) 순.
  증분 재파싱 문서가 세션 모델의 수명과 무효화를 정의하고, 나머지 둘이 그 위에 선다.
- 메일머지·서식 일괄 처리가 목적이면 [workspace_handles.md](workspace_handles.md)만
  읽어도 된다 — 앞 문서에 의존하는 곳은 링크로 표시했다.
- "문서 간 관계"를 기대하고 왔다면 [reference_queries.md](reference_queries.md)의 §2 를
  먼저 보라. **HWP 코퍼스 353건에 문서→문서 링크는 0건**이라는 실측이 그 문서의 전제다.

## 관련 문서

| 주제 | 문서 |
|---|---|
| 세션 도구의 현재 사용법 | [MCP 연동 가이드](../../manual/mcp_integration_guide.md) |
| 작업별 명령 결정 표·봉투 어휘 | [에이전트 지식 지도](../../manual/agent_knowledge_map.md) |
| 핸들·자원 한계·경로 계약 | [에이전트 경계 무결성 계약](../agent_boundary_contract.md) |
| 문서가 에이전트를 조종하는 경로 | [에이전트 보안 문서 지도](../agent_security/README.md) |
| 표면 확장 절차(이슈→분석→구현→PR) | [에이전트 표면 플레이북](../../manual/agent_surface_playbook.md) |
| 세션 vs 무상태 공개 벤치 | [bench_session_vs_stateless](../../report/bench_session_vs_stateless/README.md) |
| 파서·IR 책임 경계 | [포맷 파서와 공통 Document IR 경계](../parser_architecture.md) |

## 이 문서 묶음이 다루지 않는 것

- **LSP 프로토콜 자체를 말하는 서버.** M25 의 "LSP 유사"는 *역할*의 비유다. rhwp 는 MCP
  서버이고, 새 전송·새 프로토콜을 추가하자는 제안이 아니다.
- **동시 클라이언트.** `mcp-serve` 는 stdio 전제라 한 호스트 전용이다
  ([S8](../agent_boundary_contract.md#s8--핸들-무결성)). 다중 클라이언트 격리는 범위 밖이다.
- **rhwp-studio(브라우저) 의 문서 상태.** WASM 대상에는 `mcp_serve` 가 아예 포함되지
  않는다(`src/mcp_serve.rs:16-17`).
