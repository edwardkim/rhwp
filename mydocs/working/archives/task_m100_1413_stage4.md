# Task M100 #1413 — 4단계 완료 보고서 (7인자 군 13개 + 가이드 + 빌드)

- 브랜치: `local/task1413`
- 작성일: 2026-06-20

## 1. 7인자 군 *Ex (13개)

setNoteEquationPropertiesEx / setFormValueInCellEx / setActiveFieldInCellEx(bool) /
removeFieldAtInCellEx(String) / pasteHtmlInCellEx / moveLineEndpointEx(i32) /
mergeTableCellsEx / insertTextInCellEx / insertClickHereFieldEx / getTextInCellEx /
getFieldInfoAtInCellEx(String) / evaluateTableFormulaEx / deleteTextInCellEx.

- 반환 타입을 positional 과 동일하게(`Result`/`bool`/`String`) 맞춤. props/value 중첩
  객체는 `json_object`, 좌표 i32 는 `json_i32`.

## 2. 동치 테스트 (10건 추가, 누적 24건)

- insert_text/get_text/delete_text/paste_html/merge_table_cells/insert_click_here_field
  (Result) + set_active_field(bool)/get_field_info/remove_field(String)/
  evaluate_table_formula — 각 *Ex == positional.
- 셋업 복잡한 3개(setNoteEquationProperties/setFormValueInCell/moveLineEndpoint)는 *Ex
  구현·컴파일 검증(동치 테스트 생략 — 노트/폼/연결선 셋업 부담).

## 3. 가이드 + CHANGELOG

- `mydocs/manual/wasm_api_options_convention.md` 신규: *Ex 설계 관행(고인자 권장, 하이브리드,
  camelCase 키, 중간 삽입 금지, breaking change CHANGELOG 표기).
- `CHANGELOG.md` `[Unreleased] ### API` 에 *Ex 26개 + 규약.

## 4. 검증

- `cargo test --lib task1413`: **24/24 passed**.
- `cargo fmt --check`: CLEAN. `cargo clippy --lib`: **0**.
- WASM 빌드(Docker) 성공, `pkg/rhwp.d.ts` 에 `Ex(options` **27개 노출**(createTableEx + 26).

## 5. 완료

#1413 4단계 전부 완료. 최종 보고서: `mydocs/report/task_m100_1413_report.md`.
