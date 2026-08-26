# #5511 CLI 표면·원인 계보·보호 불변식 조사

## 1. 조사 질문과 결론

이 조사는 `src/main.rs`가 42,370줄이 된 사실만 보고 파일을 나누지 않는다. 다음 질문을
먼저 답한다.

1. 어떤 기능 증가 경로가 현재 구조를 만들었는가?
2. 실제 실행 표면과 help·capabilities·MCP 선언은 어디까지 일치하는가?
3. 이미 존재하는 CQRS·service 경계 중 무엇을 보호하고 무엇을 연결해야 하는가?
4. 이동 전에 어떤 계약을 테스트로 더 고정해야 하는가?

결론은 다음과 같다.

- 비대화는 단일 회귀가 아니라 CLI 기능을 `main.rs`에 직접 추가하는 298개 커밋의 누적
  결과다. 특히 2026-08-17~18 이틀 동안 이 파일을 건드린 32개 커밋 중 28개가 편집
  관련이었고, 줄 수는 30,942에서 42,370으로 늘었다.
- 7월 말부터 help, capabilities, MCP tool, JSON 봉투와 에이전트 계약이 함께 성장했지만
  최상위 dispatch까지 포괄하는 하나의 catalog는 생기지 않았다.
- 8월 16일 `src/service`가 문서 열기·조회 공통 축으로 신설됐으나 `src/main.rs`의
  `rhwp::service` 참조는 0이다. 새 경계는 존재하지만 기존 CLI가 그 경계로 이행되지 않은
  상태다.
- 실제 최상위 명령은 102개, capabilities 명령은 98개다. `export-llm`, `ir-sweep`,
  `dump-anchors`, `dump-carets` 네 명령은 dispatch에만 존재한다. 현재 테스트는
  help↔capabilities를 양방향으로 보호하지만 dispatch↔catalog 전수 대응은 보호하지 않는다.
- 따라서 첫 구현은 handler 이동이 아니다. 102개 dispatch 전수를 가시성 정책과 함께
  표현하는 characterization guard를 먼저 세우고, 현재 출력은 바꾸지 않은 채 catalog의
  소유권을 확정해야 한다.

## 2. 재현 기준

- Git 기준: `6eb4569a30eec88f742fe81084142c20cd48e31c`
- 운영체제: Ubuntu 24 on WSL2
- Rust: `rustc 1.93.1`, `cargo 1.93.1`
- nextest: 설치 버전 `0.9.137`, 저장소 권장 버전 `0.9.140`
- 제품 버전: `rhwp 0.8.4`

`capabilities`와 MCP 수치는 위 기준에서 `cargo build --bin rhwp`로 새로 만든
`target/debug/rhwp`의 실제 출력을 `jq`로 집계했다. 소스 수치는 `wc`, `rg`, 함수 선언
경계를 사용해 교차 확인했다.

## 3. 성장 계보

### 3.1 파일 크기 변화

각 날짜의 KST 23:59 이전 `upstream/devel` 마지막 커밋에서 `src/main.rs`를 측정했다.

| 날짜 | 커밋 | 줄 | bytes |
|---|---|---:|---:|
| 2026-03-27 | `e8f5809a54` | 1,526 | 70,824 |
| 2026-04-30 | `cf156a8ef5` | 2,649 | 120,227 |
| 2026-05-31 | `fe5d306c24` | 4,383 | 179,943 |
| 2026-06-30 | `66eb475543` | 5,251 | 212,510 |
| 2026-07-15 | `6caf973b2b` | 6,001 | 240,561 |
| 2026-07-31 | `ab12d4f299` | 10,830 | 444,056 |
| 2026-08-08 | `c391dbbdcd` | 19,343 | 834,804 |
| 2026-08-15 | `9a40da8818` | 25,606 | 1,089,517 |
| 2026-08-16 | `566194be7a` | 27,183 | 1,158,032 |
| 2026-08-17 | `ba097d6bf9` | 30,942 | 1,312,934 |
| 2026-08-18 | `6eb4569a30` | 42,370 | 1,792,248 |

초기 1,526줄에서 현재 42,370줄까지 약 27.8배가 됐다. 7월 15일 이후의 증가만
36,369줄이며, 이는 단순 parser·renderer 복잡도와 별개의 CLI 조립 부채다.

### 3.2 구조가 이렇게 된 흐름

1. **초기 단일 진입점**: 파일 열기, 출력, 진단을 한 binary에서 빠르게 연결했다.
2. **기계 계약 추가**: 7월 말부터 JSON 봉투, capabilities, MCP 서버가 추가됐다.
   `mcp_tool_definitions`, `capabilities_command_entries`, `print_help`가 각각 큰 등록부가 됐다.
3. **계약 drift를 사후 테스트로 봉합**: help↔capabilities, capabilities↔MCP, schema와
   annotation 정합 테스트가 추가됐다. 이 테스트들은 실제 결함을 막았지만 등록 원천 자체를
   하나로 합치지는 못했다.
4. **service 경계의 후발 신설**: 커밋 `462f8daf7`에서 문서 열기·조회 service가 생겼다.
   신규 표면이 공통 경계를 쓸 수 있게 됐지만 기존 CLI handler 이행은 별도 과제로 남았다.
5. **편집 명령의 급속 누적**: 8월 17~18일 기여 통합 과정에서 command core와 계약
   테스트는 함께 늘었으나, CLI adapter도 동일 파일에 계속 추가됐다. 커밋 `e0851908b` 한
   건만으로 `src/main.rs`에 4,176줄이 추가됐다.
6. **복잡도 계획의 사각지대**: 기존 전역 리팩토링 지표는 CLI dispatch인 `main.rs`를
   모집단에서 제외했다. 당시 범위 판단은 유효했지만, CLI만의 후속 budget과 gate가 없어
   성장 제한이 작동하지 않았다.

즉, 개별 기능 구현의 품질 부족보다 **새 명령을 얇은 adapter로 제한할 구조적 입구가
없었다는 것**이 원인이다. 해결도 기여 명령을 되돌리는 방식이 아니라 catalog와 application
경계를 먼저 마련하는 순서여야 한다.

## 4. 현재 정적 기준선

| 항목 | 값 | 측정 의미 |
|---|---:|---|
| `src/main.rs` 줄 수 | 42,370 | 파일 budget 1,200 대비 35.3배 |
| bytes | 1,792,248 | release compile/link 비용에도 영향 |
| 최상위 함수 | 351 | module 밖 production 함수 선언 수 |
| `fn edit_*` | 92 | helper를 포함한 편집 함수 수 |
| edit dispatch arm | 88 | 사용자에게 노출된 edit 하위 명령 수 |
| `finish_edit_write` 참조 | 82 | 이미 존재하는 공통 저장 seam |
| CC>25 함수 | 19 | Clippy 기본 임계값 25의 현재 CLI 초과 수 |
| 최대 CC | 68 | `dump_controls` |
| `wasm_api::HwpDocument` 직접 참조 | 42 | adapter가 WASM facade에 결합 |
| `rhwp::service` 참조 | 0 | 신설 service 경계로 미이행 |
| `rhwp::model` 직접 참조 | 71 | 내부 모델 결합 |
| `rhwp::parser` 직접 참조 | 95 | parser 결합 |
| `rhwp::renderer` 직접 참조 | 28 | renderer 결합 |
| `rhwp::serializer` 직접 참조 | 16 | serializer 결합 |
| `rhwp::parser::detect_format` | 27 | 형식 판정의 adapter 중복 |

CC>25 함수는 `cargo clippy --bin rhwp -- -W clippy::cognitive_complexity`로 측정했다.
상위 항목은 `dump_controls` 68, `run_plan_engine` 57, `export_svg`·`ir_diff` 38,
`csv_to_table` 37, `show_info` 34다. 큰 등록부는 분기 복잡도만으로는 드러나지 않으므로
줄 수도 독립 지표로 유지한다.

| 큰 책임 | 시작 줄 | 다음 경계 | 대략적 크기 |
|---|---:|---:|---:|
| `mcp_tool_definitions` | 547 | 4,747 | 4,200줄 |
| `capabilities_command_entries` | 5,163 | 6,573 부근 | 1,410줄 |
| `print_help` | 6,906 | 8,153 | 1,247줄 |
| `dump_controls` | 17,222 | 다음 최상위 함수 | 약 1,269줄 |

`main()`의 match, 위 세 등록부와 각 handler가 서로 다른 형태로 명령 정보를 가진다.
따라서 파일 이동만 하면 중복과 drift는 새 파일로 함께 이동한다.

## 5. 실제 명령 표면

### 5.1 최상위 command

| 표면 | 수 | 비고 |
|---|---:|---|
| `main()` 실제 dispatch | 102 | `--help`·`--version` 별도 |
| `capabilities.commands` | 98 | schemaVersion `1.0` |
| 사람용 help | 95 | capabilities의 명시적 hidden 3개 제외 |
| `capabilities --mcp` tools | 162 | 63개 고유 CLI command로 배선 |

capabilities의 98개 명령은 다음처럼 분류된다.

| category | 수 | 명령 |
|---|---:|---|
| batch | 2 | `batch`, `scan` |
| diagnostic | 29 | `harness-status`, `dump`, `dump-pages`, `dump-extents`, `dump-note-shape`, `dump-endnote-lines`, `dump-records`, `diag`, `ir-diff`, `verify`, `render-diff`, `layout-anomaly`, `hwpx-roundtrip`, `hwp5-roundtrip`, `measure-width`, `core-pages`, `bench`, `hwp5-*` probe 12종 |
| edit | 5 | `run`, `harness`, `csv-to-table`, `csv-to-chart`, `edit` |
| export | 25 | export·convert·scaffold·thumbnail 및 CSV 교환 명령 |
| internal | 5 | `test-shape`, `test-caption`, `test-field`, `gen-table`, `gen-pua` |
| query | 31 | info·검색·검증·감사·inspect 등 읽기 표면 |
| serve | 1 | `mcp-serve` |

capabilities가 help에서 의도적으로 숨기는 세 명령은 `core-pages`, `dump-extents`,
`measure-width`다. `tests/cli_json_contract.rs::HELP_HIDDEN`이 각 사유와 stale 여부를
검증한다.

### 5.2 발견된 dispatch-only 명령

| 명령 | 유입 커밋과 현재 근거 | Stage 1 처리 원칙 |
|---|---|---|
| `export-llm` | `5ef30eff7`에서 dispatch 추가; 전용 계약 테스트와 CLI 매뉴얼 절이 있음 | 사용자 표면 누락 가능성이 높지만 자동 노출 금지; 별도 계약 승인 |
| `ir-sweep` | `28202f6dc`에서 dispatch 추가; CLI 매뉴얼의 회귀 조사 도구 | diagnostic visibility를 명시 |
| `dump-anchors` | `b9e11432c`에서 dispatch 추가; CLI 매뉴얼의 레이아웃 디버그 도구 | diagnostic visibility를 명시 |
| `dump-carets` | `c85315ccd`에서 dispatch 추가; CLI 매뉴얼의 UI/레이아웃 디버그 도구 | diagnostic visibility를 명시 |

네 명령을 곧바로 capabilities나 help에 추가하면 additive라도 관찰 가능한 계약 변경이다.
#5511의 move-only 불변식을 지키기 위해 Stage 1의 catalog는 먼저 이 상태를 명시적으로
표현한다. 외부 노출 보정은 characterization test와 영향 분석 뒤 메인테이너가 별도로
승인해야 한다. `git log -S`로 각 유입을 확인했으며, 네 이름 모두 대응하는 `cmd(...)`
등록 이력은 없었다.

### 5.3 하위 command와 MCP

- `edit`: capabilities 선언 88개와 `run_edit` dispatch 88개가 일치한다.
- `inspect`: capabilities 선언 4개와 dispatch 4개가 일치한다.
- MCP: 162 tools, 고유 CLI command 63개, stdin tools 3개다.
- MCP annotation 4필드는 전 도구에 존재한다. read-only 56개, 파일 산출 선언 106개,
  destructive tool은 `hwp_redact` 1개다.

## 6. 의존 방향과 보호할 seam

현재 방향은 다음과 같다.

```text
process main / help / catalog / handlers
  ├─> wasm_api::HwpDocument
  ├─> parser / serializer / renderer / model
  ├─> document_core commands + queries
  └─> binary-local helpers

mcp_serve
  └─> crate::info_json_value 등 main.rs의 binary helper 6종(호출 8곳)

service::{open, query, error}
  └─> parser + document_core
      (main.rs와 mcp_serve.rs에서는 아직 사용하지 않음)
```

문제는 `main.rs`가 아래 계층을 직접 아는 것뿐 아니라 `mcp_serve`가 다시 `main.rs`의
helper를 올려다본다는 점이다. handler를 `src/cli`로 옮기면서 이 역방향 참조를 그대로
두면 순환 의존이나 새 shared God module이 생긴다.

다음 경계는 보호한다.

- `src/document_core/commands`와 `src/document_core/queries`: 도메인 CQRS의 현재 물리 경계
- `finish_edit_write`: 82개 편집 경로의 공통 저장·검증 seam
- `src/service::{DocumentService, OpenOptions, ServiceError}`: 전역 비밀번호와 문자열 오류
  판별을 대체할 방향
- `src/bin/rhwp-agent/caps.rs::CommandSpec`: 작은 명령 catalog의 참고 구현
- 기존 JSON envelope와 MCP annotation 유도 규칙

## 7. 계약 테스트 대응표

| 계약 | 현재 보호 | 확인된 공백 |
|---|---|---|
| help → capabilities | `capabilities_covers_every_help_command` | dispatch-only 명령은 양쪽에 없으면 통과 |
| capabilities → help | `help_covers_every_capabilities_command` + `HELP_HIDDEN` | dispatch 전수와는 대조하지 않음 |
| capabilities schema | `capabilities_schema_contract` | schema는 실제 98개만 검증 |
| JSON command → MCP | `capabilities_mcp_covers_every_json_command` | capabilities에서 빠진 명령은 모집단 밖 |
| edit/inspect 하위 dispatch | `capabilities_subcommands_contract` | 다른 부모·최상위 dispatch에는 일반화되지 않음 |
| MCP manifest → server | `mcp_server_contract` | binary helper의 소유 계층은 검증하지 않음 |
| MCP annotations | `mcp_tool_annotations_contract` | category 자체가 누락되면 검증 밖 |
| exit code 0/1/2 | `cli_exit_codes` 및 command별 계약 | 전 102개를 한 표에서 검증하지 않음 |
| password 배선 | `cli_password_stdin_command_parity_contract` | thread-local 전역 의존은 구조적으로 남음 |
| JSON field/provenance | `cli_json_contract`, `provenance_contract` 등 | command별 분산 테스트라 catalog 소유권 없음 |

focused 기준선은 위 여섯 계약 모듈과 `cli_exit_codes`를 nextest 식으로 선택해 97/97
통과했다. 이 결과는 현재 계약 테스트가 유효하다는 뜻이지, dispatch-only 네 항목이
의도됐다는 뜻은 아니다.

## 8. Stage 1 선행 변경 단위

Stage 1의 첫 PR 후보는 다음 순서를 지킨다.

1. **characterization guard**: `main()`의 102개 명령을 catalog inventory와 대조한다.
   네 dispatch-only 항목은 이름과 현재 visibility 사유가 있는 명시적 기준선으로 둔다.
2. **catalog 타입과 metadata 이동**: 이름, category, visibility, feature gate, JSON/batch,
   help/MCP 참여 정책을 표현한다. handler function pointer 결합은 후속 단위로 둔다.
3. **파생 표면의 동등성 검사**: 기존 help bytes, capabilities JSON semantic value,
   MCP manifest semantic value가 변경 전과 같음을 검증한다.
4. **중복 원천 제거**: 검증된 표면 하나씩 catalog 파생으로 바꾼다. 한 PR에서 help,
   capabilities, MCP, dispatch를 모두 동시에 전환하지 않는다.

첫 수직 절편은 handler 이동이 아니라 **명령 이름·가시성 catalog와 전수 drift guard**다.
이 단위가 가장 작은 위험으로 이후 모든 CQRS 이동의 경계를 고정한다.

## 9. 중단 조건

- 네 dispatch-only 명령의 노출 정책을 승인 없이 바꾸려는 경우
- help 문자열의 byte 차이 또는 JSON/MCP semantic 차이가 발생한 경우
- catalog가 `main.rs`의 handler 구현 타입을 과도하게 알아야 하는 경우
- `mcp_serve` 의존을 해결하기 위해 binary helper를 더 넓은 shared module로 옮기려는 경우
- 동시 PR이 같은 명령 등록부 또는 handler 계열을 변경한 경우

이 경우 현재 단위를 중단하고 기준선과 최신 `upstream/devel`을 다시 대조한다.
