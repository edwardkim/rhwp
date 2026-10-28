# #5511 Stage 2 기능군 배치 Q6 — 변환·생성 adapter inventory

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 기준: `upstream/devel` `980bf59e406e9cd31d4b3ac9ffa21f356487b4ce`
- 기준선 merge commit: `9ffc9ea772a9bb96623e4a98f2437ef7096c587e`
- 작성일: 2026-08-20
- 상태: Q6 실행 중 — 기존 계약 충분, 물리 이동 진입

## 1. 범위와 시작 조건

Q6는 `convert`, `extract-pages`, `export-hwpx`, `export-hml`, `export-doclang`,
`build-from-ingest`, `scaffold` 일곱 file-producing adapter를 `src/main.rs`에서 분리한다.
`export-doclang`은 master 책임 지도에서 변환·HWPX/HML·생성군 사이에 함께 계측된 XML 변환
adapter이므로 Q6가 소유한다.

시작 전에 전날 Q5 HEAD와 작업 트리를 확인하고 `origin/devel`·`upstream/devel`의 최신
`980bf59e4`를 정상 merge했다. 원격 변경 52커밋은 Q6의 `src/main.rs` handler를 수정하지 않았고,
merge-tree도 충돌 없이 생성됐다. 열린 devel 대상 PR 2건에서 `src/main.rs`, 기존
`src/cli/commands/`, `src/cli/outputs/`와 겹치는 파일은 없었다.

parser·serializer·renderer 알고리즘, 포맷 변환 의미, JSON schema, stdout/stderr, exit code와
파일 생성 순서는 변경하지 않는다. `src/main.rs` 시작 크기는 31,095줄이다.

## 2. handler와 복잡도

| handler·helper 군 | 시작 범위 | 규모 | CC 판정 | 이동 소유권 |
|---|---:|---:|---|---|
| conversion verify 인자·오류 helper | 9,485~9,547 | 63줄 | 25 이하 | `cli/commands/conversion.rs` |
| `extract_pages` | 9,548~9,668 | 121줄 | 25 이하 | `cli/commands/conversion.rs` |
| `convert_hwp` | 9,669~9,876 | 208줄 | 25 이하 | `cli/commands/conversion.rs` |
| `export_doclang` | 9,877~10,037 | 161줄 | 25 이하 | `cli/outputs/doclang.rs` |
| `export_hwpx` | 10,038~10,248 | 211줄 | 25 이하 | `cli/commands/conversion.rs` |
| HML 인자·오류 helper와 `export_hml` | 10,249~10,379 | 131줄 | 25 이하 | `cli/commands/conversion.rs` |
| `build_from_ingest` | 10,380~10,523 | 144줄 | 25 이하 | `cli/commands/generation.rs` |
| `run_scaffold` | 10,524~10,665 | 142줄 | 25 이하 | `cli/commands/generation.rs` |

`cargo clippy --locked --bin rhwp -- -W clippy::cognitive_complexity`에서 Q6 대상 경고는
없었다. 따라서 큰 함수를 숨겨 옮기는 중단 조건은 발동하지 않는다. 계획한 세 구현 모듈은 각각
1,200줄보다 작고 conversion command, DocLang output, 새 문서 generation의 책임을 섞지 않는다.

## 3. 보호 계약

이동 전 다음 17개 계약 모듈의 선택 실행이 123/123 통과했다.

- 변환 검증과 쪽수: `issue_1638_convert_verify_gate`, `issue_1868_export_hwpx_cli`,
  `issue_3565_extract_pages`
- HML·DocLang: `hml_cli`, `export_hml_json_contract`, `doclang_export`,
  `export_doclang_json_contract`, `issue_3359_export_family_option_order`
- ingest·scaffold: `issue_3358_ingest_unknown_fields`, `genpreview_json_contract`,
  `scaffold_contract`
- 공통 CLI 계약: `cli_json_contract`, `cli_exit_codes`,
  `cli_exit_codes_diagnostic_commands`, `cli_exit_codes_dump_diag`, `provenance_contract`,
  `output_axis_json_contract`

이 모집단은 다음 관찰 축을 이미 고정한다.

- `convert`·`export-hwpx`의 IR/쪽수 검증, exit 3/4 우선순위와 JSON 판정 봉투
- `extract-pages`의 1 기준 범위, 문단 보존·삭제, HWP5 저장 후 재로딩
- HWP3/HWP5/HWPX→HWPX 페이지 보존과 HML 의미 보존·atomic write
- 입력과 출력의 동일 경로·symlink·hard link 덮어쓰기 거부와 실패 시 무산출
- DocLang XML·asset·loss 보고와 옵션 순서
- ingest unknown-field fail-closed, 공식 sample 생성과 scaffold IR round-trip
- 모든 JSON envelope의 provenance, 성공·사용법·런타임 stdout/stderr 분리

추가로 고정할 관찰 공백이 없어 새 characterization test는 만들지 않는다. 기존 계약을 이동 전후
동일 모집단으로 실행하는 것이 더 직접적인 move-only 증거다.

## 4. 공유 seam과 책임 경계

다음 요소는 Q6 전용 구현으로 복제하거나 성급히 이동하지 않는다.

- `ConversionVerifyOptions`: single convert와 Q4 `batch convert`가 공유하므로 root seam 유지
- `verification_exit_code`: exit 3/4 우선순위와 기존 source-side test를 함께 보존
- `paths_refer_to_same_file`: DocLang·HML뿐 아니라 batch convert와 P1 replay가 공유
- `load_document`·`load_document_core`, 입력·출력 password thread-local: 전 CLI 인증 seam
- `provenance::marked`와 `ENVELOPE_SCHEMA_VERSION`: JSON 정본

새 모듈은 이 seam을 호출할 뿐 소유권을 복제하지 않는다. `export-doclang`은 문서를 변경하지 않는
파일 output에, 나머지 변환·생성은 기존 CQRS 분류에 맞춰 command에 둔다. parser·serializer 내부
API를 새 wrapper로 감추거나 service 계층 전환을 선행하지 않는다.

## 5. 중단 조건

- 이동 전 123개 계약 중 출력·파일·exit·검증 판정이 하나라도 달라짐
- 새 모듈 1,200줄 초과 또는 Q6 함수에서 CC 25 초과 발생
- 변환 결과를 맞추기 위해 parser·serializer·renderer 동작 변경이 필요함
- 공유 seam 복제, command/output 양방향 참조 또는 public API 확대가 필요함
- 최신 `devel`이나 열린 PR이 같은 handler·test·module 경계를 변경함

발동하면 다음 이동을 멈추고 원인과 선택지를 메인테이너에게 보고한다.
