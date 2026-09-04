# #5511 Stage 1 작업 보고 — 최상위 명령 catalog와 drift guard

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- Stage 1 통합 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 수행일: 2026-08-18
- 상태: 완료 — Stage 2 승인 대기

## 1. 구현 결과

`src/cli/catalog.rs`에 실제 최상위 dispatch 102개를 선언했다. 각 항목은 다음 계약만
소유하며 handler 함수나 도메인 타입을 참조하지 않는다.

- 명령 이름과 category
- help·capabilities 가시성 및 예외 사유
- 필요한 Cargo feature
- JSON·batch 계약 참여 여부
- MCP 참여 여부

Stage 0에서 발견한 `export-llm`, `ir-sweep`, `dump-anchors`, `dump-carets`는 자동으로
노출하지 않았다. 네 항목은 이유가 있는 `DispatchOnly` 상태로 catalog에 남아 현재의
관찰 가능한 계약을 보존한다. `core-pages`, `dump-extents`, `measure-width`도 기존처럼
capabilities에는 있으나 help에는 없는 `Hidden` 상태다.

## 2. 단일 원천으로 전환한 표면

1. 알 수 없는 명령의 did-you-mean 후보와 순서는 catalog의 capabilities 참여 순서에서
   파생한다.
2. `capabilities.commands[]`의 category, JSON, batch, feature 값은 catalog에서 읽는다.
   기존 등록부는 summary·flags·recordFields 같은 명령별 상세 설명만 보탠다.
3. MCP tool의 CLI command는 catalog에서 다시 해석하고, MCP 비참여 명령의 배선을
   거부한다. 반대로 catalog에서 MCP 참여로 선언한 63개 명령은 하나 이상의 tool이
   존재해야 한다.
4. help 비노출 허용목록은 테스트 내부 상수에서 제거하고 catalog의 `Hidden(reason)`을
   사용한다.
5. 새 `cli_catalog_contract`는 실제 `main()` dispatch, help, capabilities, MCP와 catalog를
   양방향으로 대조한다.

help 산문과 MCP tool별 JSON Schema는 서로 다른 정보량을 가진 상세 payload이므로 이번
단위에서 한 구조로 합치지 않았다. 대신 명령의 존재·분류·참여 여부를 catalog가 소유하고,
상세 payload가 어긋나면 mirror contract가 실패하도록 했다. handler와 dispatch 의미도
변경하지 않았다.

## 3. 지표 변화

| 항목 | Stage 0 | Stage 1 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 42,370 | 42,260 | -110 |
| `src/cli/catalog.rs` | 없음 | 1,023 | 신규, 모듈 상한 1,200 이하 |
| catalog 명령 | 없음 | 102 | 실제 dispatch 전수 |
| capabilities 명령 | 98 | 98 | 변화 없음 |
| help 명령 | 95 | 95 | 변화 없음 |
| MCP 연결 CLI 명령 | 63 | 63 | 변화 없음 |
| CLI CC>25 함수 | 19 | 19 | handler 변경 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이 단계의 LOC 감소가 작은 것은 handler를 옮기는 단계가 아니기 때문이다. 대신 신규 명령이
category·JSON·batch·feature·MCP 참여를 여러 등록부에 서로 다르게 적어도 통과하던 구조를
먼저 닫았다.

## 4. 관찰 가능한 동작 동등성

Stage 0 release-test 바이너리와 Stage 1 바이너리를 byte 단위로 비교했다.

- `rhwp --help`
- `rhwp capabilities`
- `rhwp capabilities --mcp`
- 알 수 없는 명령 `rhwp inof`의 stderr 복구 출력

네 출력은 모두 동일했다. 새 catalog는 dispatch-only 네 명령을 help나 capabilities에
추가하지 않는다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| `cargo test --test cli_catalog_contract` | 4/4 통과 |
| CLI·capabilities·MCP focused nextest | 91/91 통과 |
| MCP/help focused nextest | 62/62 통과 |
| release-test 전체 nextest | 최신 기준선 재실행 7,304/7,304 통과, 3 slow, 38 skipped, 170.341초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 716 sources / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다.

## 6. 원격 이동과 충돌 판정

전체 검증 중 `upstream/devel`이 `e5ef2620b`에서 `c3c35306b`로 4커밋 이동했다. 유입된
#5483·#5515 통합 변경은 parser·serializer·renderer와 해당 테스트를 수정했으며,
`src/main.rs`, `src/cli/`, `cli_json_contract`와 겹치지 않았다. Stage 1의 다섯 로컬 커밋은
최신 기준선 위로 충돌 없이 재배치했다. 재배치 후 release-test 전체 7,304건과 clippy,
doc-test, manifest, unit tier를 모두 다시 실행했다.

열린 PR 중 #5501·#5506·#5517이 `Cargo.toml` 테스트 타깃 블록을 수정하지만 CLI 소스와
계약 테스트를 겹쳐 수정하는 PR은 없었다. push 전 다시 fetch하고 생성 블록 충돌 여부를
확인해야 한다.

## 7. 다음 단계 관문

Stage 2에서는 이 catalog를 기준으로 read-only query 한 계열을 첫 수직 절편으로 선택해
`src/cli/queries/`로 물리적으로 이동한다. 이번 단계에서는 다음을 의도적으로 하지 않았다.

- handler 함수 포인터를 catalog에 결합
- root/edit dispatch 의미 변경
- parser·serializer·renderer 동작 변경
- help 문구·JSON/MCP schema 변경
- service layer 이행

Stage 1 문서 커밋과 최신 기준선 재검증을 마친 뒤 메인테이너의 Stage 2 진입 승인을
요청한다. remote push는 별도 승인 전 수행하지 않는다.
