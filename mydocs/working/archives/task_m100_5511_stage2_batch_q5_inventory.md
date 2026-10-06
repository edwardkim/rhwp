# #5511 Stage 2 기능군 배치 Q5 — 진단 조회 inventory

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 기준: `52d8bf8eb3c3351cbabba00ce2b4e299d1930c01`
- 작성일: 2026-08-19
- 상태: Q5 실행 중 — characterization 선행

## 1. 범위와 시작 조건

Q5는 `info`, `dump-pages`, `dump` 세 read-only 진단 조회 adapter를 `src/main.rs`에서
물리적으로 분리한다. 시작 시 `origin/devel`, `upstream/devel`, 로컬 `devel`은 모두 시작 기준
SHA로 같고 작업 트리는 깨끗했다. 열린 PR에서 `src/main.rs`, `src/cli/queries/`, Q5 주요 계약
테스트와 겹치는 파일은 발견되지 않았다.

parser·serializer·renderer·layout 알고리즘, JSON schema, CLI 옵션·exit code와 사람용 출력은
변경하지 않는다. `info_json_value`는 `info` 단건뿐 아니라 batch, digest, MCP가 함께 쓰는 schema
단일 원천이므로 root 공유 seam으로 보존한다. `hu_to_mm`·`hu_to_mm_i`도 기존 vector output과
diagnostics가 함께 사용하므로 Q5 전용 모듈에 복제하지 않는다.

## 2. 시작 지표

| 책임 | 시작 줄 | 규모 | 시작 CC | 판정 |
|---|---:|---:|---:|---|
| `show_info` | 9,433 | 341줄 | 34 | 출력 책임별 helper 분해 후 이동 |
| `dump_pages` | 9,783 | 121줄 | 25 이하 | page diagnostic query로 이동 |
| `dump_controls` | 9,904 | 1,271줄 | 68 | 분해 없이 이동 금지 |
| `info_json_value` 계열 | 8,240 | 107줄 | 25 이하 | 공유 schema seam으로 root 유지 |

`src/main.rs`는 32,821줄이다. 기존 `src/cli/queries/diagnostics.rs`는 1,096줄이므로 Q5를 그
파일에 덧붙이면 1,200줄 상한을 즉시 위반한다.

## 3. 기존 보호와 공백

기존 계약은 다음 축을 이미 보호한다.

- `info --json`: schema, title, warnings, batch·MCP 동형성
- `dump-pages --json`: page filter, layout item schema, 범위·읽기 실패의 stdout 침묵과 exit
- `dump`: 알 수 없는 플래그 거부, section/paragraph filter 수용, 읽기·파싱 실패 exit
- HML: `info` format·encoding·warning과 `dump` 성공

그러나 성공한 사람용 출력 전체의 공백·순서·숫자 표기는 고정되지 않았다. Q5 시작 기준의
`samples/hwp3-sample.hwp` 경로를 `<SAMPLE>`로 정규화한 stdout 전체 SHA-256은 다음과 같다.

| 명령 | bytes | lines | SHA-256 |
|---|---:|---:|---|
| `info` | 2,317 | 38 | `bffcbf7de3bab9ff3b05dda97815afcbfe3d953e8a85098d2ba78ef9d37284ea` |
| `dump-pages -p 0` | 5,000 | 33 | `e542bef7cea773d38d6108588b8255005032567ce3fc964472ae84255cfbb5db` |
| `dump --section 0 --para 0` | 3,263 | 22 | `bb27d62a90f3deec83bf8b1a8270680baaf9253cb6b511e6f052fdc0422957ca` |

이 세 기준을 `tests/cases/q5_diagnostic_output_contract.rs`에 characterization으로 추가한다.

## 4. 책임 분해

한 기능군 안에서 다음 경계로 나눈다.

| 모듈 | 책임 | 상한 |
|---|---|---:|
| `cli/queries/info.rs` | 인자 해석, 문서 meta 사람 출력, 공유 JSON seam 호출 | 1,200줄 |
| `cli/queries/page_dump.rs` | page filter·compat 옵션, JSON/사람 page dump | 1,200줄 |
| `cli/queries/control_dump/mod.rs` | dump 인자, 문서·section·paragraph 순회와 완료 집계 | 1,200줄 |
| `cli/queries/control_dump/shape.rs` | 도형 공통 속성·재귀 shape 출력 | 1,200줄 |
| `cli/queries/control_dump/table.rs` | 표·셀·중첩 표 출력 | 1,200줄 |
| `cli/queries/control_dump/story.rs` | master page와 header/footer story 출력 | 1,200줄 |

`show_info`와 `dump_controls`는 먼저 책임 helper로 나눠 각 top-level handler의 CC를 25 이하로
내린다. helper 사이에는 모델 참조만 전달하며 root와 새 모듈의 양방향 호출이나 formatter 복제를
만들지 않는다. characterization 통과 뒤 물리 이동하고 같은 테스트를 다시 실행한다.

## 5. 중단 조건

- 세 stdout digest, 기존 JSON·exit·flag 계약 중 하나라도 달라짐
- Q5 모듈이 1,200줄을 넘거나 CC 25 초과 함수를 그대로 숨김
- `info_json_value`를 복제하거나 batch·digest·MCP schema 소유권을 갈라야 함
- parser·serializer·renderer 동작 수정이 필요함
- 최신 `devel` 또는 열린 PR이 같은 handler·test 경계를 변경함

발동하면 다음 이동을 멈추고 원인과 선택지를 메인테이너에게 보고한다.
