# #5511 Stage 2 기능군 배치 M1 — metadata projection inventory

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 HEAD: `4ec480b0125fa7fd70641abf5835153d984e4b82`
- 통합 기준: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 수행일: 2026-08-20
- 상태: M1 실행 중 — 기존 계약과 byte hash 기준으로 책임별 물리 분리

## 1. 실제 범위

M1 책임 지도상의 7,569줄은 현재 `src/main.rs` 494~8,059행에 모인 metadata projection
7,566줄과 일치한다. handler가 아니라 catalog에서 파생되는 MCP 도구 정의, capabilities,
사람용 help 세 표면이다.

| 현재 범위 | 규모 | 책임 | 판정 |
|---|---:|---|---|
| `show_mcp_tools`~`mcp_tool_definitions` | 4,266줄 | MCP manifest·162개 tool schema·CLI wiring | 한 함수·한 모듈 이동 금지 |
| MCP annotation·capability helper~`export_provenance_map` | 2,056줄 | annotation 유도·command projection·검색·봉투 | entries와 projection 분리 |
| `print_help` | 1,244줄 | 공개·edit·protocol 사람용 help | 책임별 출력 함수 분리 |

MCP 정의의 `mcp_tool_definitions`는 데이터 중심이라 CC 25 초과 경고는 없지만 약 4,192줄짜리
단일 함수다. capabilities entries도 1,267줄, help도 1,244줄이어서 그대로 옮기면 새 모듈
1,200줄 상한을 넘는다. 따라서 기능 변경 없이 순서를 보존하는 작은 builder 군으로 나눈다.

## 2. 목표 소유권

정본 `src/cli/catalog.rs`는 이동하거나 복제하지 않는다. 새 `src/cli/metadata/`는 catalog를
읽어 외부 표면을 투영하는 adapter만 소유한다.

| 경로 | 소유권 | 예상 상한 |
|---|---|---:|
| `metadata/mcp/mod.rs` | manifest, 공통 schema/tool builder, password·annotation 후처리 | 300줄 |
| `metadata/mcp/read.rs` | 조회·export·inspection 도구 순서 1 | 900줄 |
| `metadata/mcp/exchange.rs` | batch·table/chart exchange·초기 edit 도구 순서 2 | 450줄 |
| `metadata/mcp/edit_content.rs` | image·text·table edit 도구 순서 3 | 800줄 |
| `metadata/mcp/edit_structure.rs` | 문서 구조·header/footer edit 도구 순서 4 | 800줄 |
| `metadata/mcp/edit_format.rs` | formatting·manifest·sanitize 도구 순서 5 | 400줄 |
| `metadata/mcp/protocol.rs` | replay·audit·lineage·gate 등 protocol 도구 순서 6 | 500줄 |
| `metadata/mcp/advanced.rs` | renderer probe·shape/form·note tail 도구 순서 7 | 700줄 |
| `metadata/capabilities/mod.rs` | 검색·봉투·annotation·subcommand 부착·공용 builder | 700줄 |
| `metadata/capabilities/core.rs` | commands 앞쪽 projection | 750줄 |
| `metadata/capabilities/extended.rs` | commands 뒤쪽·internal projection | 750줄 |
| `metadata/help/mod.rs` | help 조립 순서 | 100줄 |
| `metadata/help/public.rs` | 일반 query/export/diagnostic help | 650줄 |
| `metadata/help/edit.rs` | edit help | 700줄 |
| `metadata/help/protocol.rs` | agent protocol·internal·전역 옵션 tail | 250줄 |

각 하위 builder는 기존 `Vec` 순서대로 `extend`한다. 정렬이나 map 재조립을 하지 않아
did-you-mean 동률 해소, MCP tool order, JSON object/array order와 help 바이트를 유지한다.
`mcp_serve.rs`, agent manifest, inspect recovery는 root wrapper를 남기지 않고 새 metadata
경로를 직접 호출한다.

## 3. 보호 계약과 byte 기준선

M1에 직접 인접한 12개 계약 모듈 143/143이 이동 전 통과했다.

- `cli_json_contract`, `cli_catalog_contract`
- `capabilities_schema_contract`, `capabilities_subcommands_contract`
- `mcp_server_contract`, `mcp_tool_annotations_contract`, `mcp_resources_contract`,
  `mcp_spec_ledger_contract`
- `did_you_mean_contract`, `agent_profile_router_contract`
- `schema_version_registry_contract`, `provenance_contract`

이 계약은 catalog↔dispatch↔help↔capabilities↔MCP 참여와 순서, 162개 도구 schema·CLI 배선,
profile filtering, annotations, resource manifest, 검색·subcommand, provenance와 schema version을
실물 출력으로 대조한다.

추가 기능을 고정하는 characterization은 필요하지 않다. 대신 이동 전 debug binary의 stdout
SHA-256을 배치 내 byte parity 기준으로 사용한다.

| 명령 | SHA-256 |
|---|---|
| `rhwp --help` | `41e7ab065c67e23ffa74dc2ea444c7e842f286db02227f8057f07e9d872b0423` |
| `rhwp capabilities` | `a397674e46a2354b9a711f3870ad2585b532363519923b7b6b38022978f5383c` |
| `rhwp capabilities --mcp` | `0deb8c9c33577e5ac1a87d020f9dbd4c6b0012fa52f2424848b4c97363a935e0` |
| `rhwp capabilities --mcp --profile 개발통합` | `c5a2e78ad397ed1c0f53625dfcb0a5acce71b647db10a3ccbe0f94dad10b05fa` |
| `rhwp capabilities --search '표 병합' --json` | `4516487192a9c3246593c186c0b682a15aca8f4e02267c9fbc5e78b398ecee9a` |
| `rhwp capabilities --search 없는단어999 --json` | `8db3289f79ccb85879d400fe67c85c72e9b7f4722b0658b4d1fb3eed2fd3d01f` |

각 구현 커밋 뒤 관련 focused 계약과 hash를 다시 비교한다. crate version이나 표면 내용을 바꾸는
작업이 아니므로 여섯 hash는 모두 완전 일치해야 한다.

## 4. 구현 순서

1. MCP 공용 builder와 도구군 7개를 이동하고 모든 소비자를 새 경로로 전환한다.
2. capabilities command entries를 두 파일로 나누고 검색·봉투·provenance projection을 이동한다.
3. help 출력을 public·edit·protocol로 분해하고 root dispatch를 새 조립 함수에 연결한다.
4. 각 절편에서 format·diff, byte hash, 관련 focused 계약을 실행한다.
5. 최종 HEAD에서 전체 release-test·clippy·doc-test·manifest·unit-tier·CI 정책을 실행한다.

동작, 문구, JSON schema, 출력 순서, catalog 값, 공개 API를 바꾸지 않는다. metadata를
application/service 계층으로 승격하거나 MCP 서버 protocol 구현까지 가져오지 않는다.

## 5. 원격·동시 작업 위험

최종 fetch에서 `origin/devel`과 `upstream/devel`은 `b914bdf4b`로 같고 현재 HEAD의 조상이다.
열린 devel 대상 PR은 #5647, #5689, #5691, #5693, #5695, #5707, #5709, #5710이다. #5707이
agent capability 문서·skill-router를 바꾸지만 `src/main.rs`, `src/cli/metadata/`, `src/mcp_serve.rs`
또는 M1 직접 계약 source는 변경하지 않는다. 나머지도 renderer, Studio, q-more, 별도 계약 경로다.

M1 완료·통합·push 직전에 exact base와 열린 PR head를 다시 확인한다. remote push는 수행하지 않는다.

## 6. 중단 조건

- 여섯 stdout hash 중 하나라도 달라짐
- catalog↔help↔capabilities↔MCP 동형 계약 또는 profile filtering이 달라짐
- 새 모듈 1,200줄 초과나 CC 25 초과가 발생함
- root wrapper, 양방향 참조, tool schema 복제가 필요함
- 동시 PR이 같은 source·contract 경계를 변경함

발동하면 다음 metadata 군으로 진행하지 않고 원인과 선택지를 메인테이너에게 보고한다.
