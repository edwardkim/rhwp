# #5511 Stage 2 기능군 배치 Q7 — IR·검증 adapter inventory와 복잡도 중단

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 HEAD: `2347add3304e1b8f06a3d683752d83654a38ab87`
- 통합 기준: `upstream/devel` `b914bdf4bf1a8f922f03ea6b141f0d9c2b10a98f`
- 수행일: 2026-08-20
- 상태: 중단 조건 발동 — characterization·책임 분해안 승인 대기

## 1. 실제 범위와 책임

Q7 책임 지도상의 1,684줄은 `test_field_roundtrip` 시작부터 `run_edit` 직전까지다. 마지막
56줄의 `collect_field_records`는 이 구간 안에 있지만 Q7 전용 구현이 아니다. `verify`, fields
query, batch와 MCP가 함께 쓰는 schema seam이므로 root에 보존한다. 실제 Q7 이동 대상은 1,628줄이다.

| 현재 범위 | 규모 | 책임 | 이동 후보 |
|---|---:|---|---|
| `test_field_roundtrip` | 104줄 | 필드 변경·HWP 저장·재로딩 내부 command | `cli/commands/internal_validation.rs` |
| IR diff helper·emitter | 626줄 | 문단·표·도형·DocInfo 차이 계산 | `cli/queries/ir_comparison.rs` |
| `cmd_verify` | 357줄 | 사후 기대조건 parsing·평가·판정 출력 | `cli/queries/verification.rs` |
| `dump_anchors`·`dump_carets` | 154줄 | stream anchor·caret geometry 진단 | `cli/queries/position_diagnostics.rs` |
| `ir_sweep`·`ir_diff` | 387줄 | 전수·선별 IR 비교 CLI | `cli/queries/ir_comparison.rs` |
| `collect_field_records` | 56줄 | single·batch·MCP 공유 field schema | root 공유 seam 유지 |

`test-shape`, `test-caption`, `gen-table`, `gen-pua`는 Q7 시작 경계보다 앞에 있고 이 배치의
internal round-trip 범위가 아니다. 이를 함께 가져오면 승인된 1,684줄 좌표와 책임 수가 달라지므로
이번 배치에서 이동하지 않는다.

제안한 네 모듈은 모두 1,200줄 이하로 유지할 수 있다. `ir_comparison.rs`가 diff 계산과 두 CLI를
소유하되 parser·serializer 의미를 바꾸지 않고, position 진단과 독립 `verify` 판정을 섞지 않는다.

## 2. 기존 보호 계약과 기준선

Q7에 직접 인접한 다음 12개 계약 모듈을 이동 전 기준선으로 실행해 104/104 통과했다.

- `verify_contract`, `agent_toolkit_contract`
- `ir_diff_json_contract`, `ir_diff_summary_mode`, `ir_diff_table_cells`
- `hwpx_password_fixture`, `hwp3_charcount_convention`, `issue_3494_char_count_convention`
- `cli_exit_codes_diagnostic_commands`, `diagnostics_flag_contract`
- `provenance_contract`, `cli_catalog_contract`

이 모집단은 다음 축을 이미 강하게 보호한다.

- `verify`의 기대조건 전 축, 순서, exit 0/1/2/3, JSON provenance와 stdout 순수성
- `ir-diff`의 동일·차이·구역 수·표 셀 차이, 암호 입력, summary·max-lines, 기본 모드 exit 0
- `test-field` 인자 누락·읽기 실패가 패닉 101 대신 exit 2/1로 끝나는 계약
- catalog·help·MCP 참여, JSON record field와 문서 파생값의 untrusted 표지

그러나 이동 전에 다음 관찰 공백을 characterization으로 고정해야 한다.

1. `test-field` 성공 경로의 출력 HWP 생성·재로딩과 입력 무훼손
2. `dump-anchors`의 stream 좌표·control 종류 사람 출력
3. `dump-carets --json -s/-p`의 순수 봉투·filter·caret row와 실패 시 stdout 침묵
4. `ir-sweep --json`의 동일 exit 0·차이 exit 3·diffCount/categories/truncated 및 text mode exit 0

새 기능을 추가하는 것이 아니라 현재 성공 동작과 부작용을 최소 integration 계약으로 고정하는
범위다. 새 source는 정책대로 `tests/cases/q7_verification_diagnostics_contract.rs` 하나에 둔다.

## 3. 복잡도 중단 증거

`cargo clippy --locked --bin rhwp -- -W clippy::cognitive_complexity` 결과 Q7 함수 세 개가
상한 25를 넘었다.

| 함수 | CC | 상한 | 판정 |
|---|---:|---:|---|
| `ir_diff_paragraph_fields` | 28 | 25 | 분해 없이 이동 금지 |
| `cmd_verify` | 29 | 25 | 분해 없이 이동 금지 |
| `ir_diff` | 38 | 25 | 분해 없이 이동 금지 |

세 함수는 각각 문단 scalar/control 비교, 옵션 parsing/조건 평가/출력, 입력 load/문단 비교/
DocInfo 비교/출력을 한 함수에 겹쳐 갖고 있다. 파일만 옮기면 root의 줄 수만 줄고 복잡도를 새
God module로 숨기므로 마스터 계획의 중단 조건에 해당한다.

권장안에서는 다음 순수 helper 경계로 분리한다.

- `ir_diff_paragraph_fields`: scalar·control/table·textbox 비교 결과 수집
- `cmd_verify`: `VerifyArgs` parsing, 문서 실측·expectation 평가, JSON/사람 판정 출력
- `ir_diff`: `IrDiffArgs` parsing, 암호 적용 load, section/paragraph·ParaShape·TabDef 비교, 출력

분해 전후 동일 계약을 실행하고 각 함수가 CC 25 이하인지 다시 계측한다. 비교 항목, 카테고리,
출력 순서와 exit 의미는 변경하지 않는다.

## 4. 선택지

### A. 최소 characterization 후 세 고복잡도 함수를 분해하고 Q7 전체 이동 — 권장

characterization을 독립 커밋으로 먼저 고정한다. 이어서 IR comparison을 제자리 helper로 분해해
CC 상한을 맞춘 뒤 `ir_comparison.rs`로 이동하고, `verify`, position diagnostics,
internal field validation을 각 CQRS 모듈로 옮긴다. `collect_field_records`는 공유 seam으로 root에
남긴다. 각 구현 커밋마다 104개 기존 계약과 신규 Q7 계약을 focused 실행하고, 배치 끝에서 전체
release-test·정적·정책 게이트를 수행한다.

### B. CC 25 이하 함수만 먼저 이동

`test-field`, `dump-anchors`, `dump-carets`, `ir-sweep`만 이동하고 `verify`·`ir-diff`와 비교 helper는
root에 남긴다. 즉시 위험은 작지만 하나의 IR/검증 기능군이 양쪽에 갈리고 Q7 완료 조건을 달성하지
못해 별도 승인·전체 회귀를 다시 반복해야 한다.

### C. Q7 전체 보류

제품·계약 변경 없이 다른 기능군으로 넘어간다. 현재 동작 위험은 없지만 1,628줄의 검증 adapter와
CC 25 초과 세 함수가 root에 남아 Stage 2 종료 조건을 막는다.

## 5. 원격·동시 작업 위험

최종 fetch 기준 `origin/devel`과 `upstream/devel`은 `b914bdf4b`로 같고 현재 Q6 HEAD의 조상이다.
가상 merge는 충돌 없이 성공했다. 열린 devel 대상 PR은 #5647, #5689, #5691, #5693, #5695
다섯 건이다. #5689는 별도 `src/bin/rhwp-q-more/`와 계약을 추가하고 나머지는 #5447 문서·test
또는 Studio 변경이다. Q7의 `src/main.rs` 구간, 제안한 `src/cli/` 모듈, Q7 characterization
경로와 겹치는 PR은 없다.

이 판정은 시점 증거다. 구현 재개와 push 직전에 exact base·열린 PR·merge-tree를 다시 확인한다.
remote push는 수행하지 않았다.

## 6. 승인 요청

권장안 A는 먼저 네 공백을 계약으로 고정하고, 고복잡도 세 함수를 책임별 helper로 분해한 뒤 Q7
전체를 CQRS 경계로 이동한다. parser·serializer·renderer·WASM 동작이나 공개 CLI 계약은 바꾸지
않는다. 승인되면 characterization 커밋부터 시작한다.
