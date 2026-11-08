# 수식 관련 HML 회귀의 #7445 이관

사용자 지시로 수식 관련 HML 검사 13개와 복합 검사 1개의 수식 경로를 별도로 이관한다.
원본 출력을 먼저 보정한 뒤 독립 기대값에 따라 복구한다. 이관은 결함 해결 또는 PASS가 아니다.

## 제거 전 검사

보존 source head: `fe2c580ef65ba7a2df3ef44bc9ab1b2d8acac83e`.
[검사 본문 보존본](removed-tests.rs.txt)은 자동 Rust test 수집 대상이 아니다.
복합 DTO의 일반 HML·비-HML 부분은 유지하고 이름에서 수식 부분을 뺀다.

- `tests/hml_parser.rs`: `parses_repo_equation_fixture_into_shared_ir_without_equation_warnings`, `repo_equation_fixture_contract_has_ordered_scripts_and_no_source_identifiers`, `imported_inline_equation_has_intrinsic_bbox_between_text_and_is_hittable`, `equation_supports_font_color_entities_and_cdata`, `unknown_equation_semantics_emit_durable_exact_path_warnings`, `unknown_equation_diagnostic_values_are_bounded_on_unicode_boundaries`, `equation_accepts_only_the_first_direct_script_and_warns_for_duplicates_and_nested_scripts`, `rejects_unclosed_equation_script_as_invalid_xml`
- `tests/hml_serializer.rs`: `equation_exports_canonically_escapes_and_reparses_edited_and_untouched_scripts`, `equation_attributes_preserve_asymmetric_color_and_optional_font`, `equation_invalid_xml_and_stale_offsets_are_aggregated`, `unknown_equation_semantics_blockers_survive_equation_edits`
- `src/wasm_api/tests.rs`: `test_hml_save_state_is_one_canonical_dto_for_hml_non_hml_and_unknown_equation`, `test_unknown_equation_values_survive_edit_undo_redo_and_save_state`

## 원본과 기준 출력

- 원본: `tests/fixtures/hml/exambank_math_equations_min.hml`와 동일한 공식 샘플
  `samples/hml/exambank_math_equations_min.hml`, SHA-256
  `b51be49cde780d39b92f42cfd1cbd58474900c46c12c4df971f79bd511c7045a`.
- 사용자의 정상 수동 한컴 2024 Print: `samples/hml/exambank_math_equations_min-2024.pdf`,
  SHA-256 `d5a494de0eb1eb7b14216fc7fb2197007ba7a30c61bf57f49d9b1bc101c13548`, 1쪽.
- 한컴 저장본 HWP/HWPX와 각 2020 MCP Print는
  [공식 샘플 출처](../../../../../samples/hml/README.md)에 기록한다. 원본 HML과 동일한
  출력 계약이라고 추정하거나 원본을 저장본으로 교체하지 않는다.

## 이관 시 관측

생산 보정 `4ea557a1d0b4aeab51f8e125390f2a60f4b89aed`와 같은 source의 Native/fresh WASM을
수동 Print와 재대조했다. 각 1쪽, 2px 관용 내용 실루엣 일치율 **6.22797%**이며
`pr_review_gate=re_review_required`다. Native SHA-256은
`ac18fd011ae24aa28b13918d70307da72e60a3e6f0a6c2ae76087d62e7bb595a`, fresh WASM은
`640a68c9e933c7eb1cf86a81f738497d2251443065f64d9b1add7995ab5ffef1`이다.
[대표 review](hml-original-review.png)로 원본의 작은 본문과 RHWP의 정상 크기 본문 차이를 보존한다.
현재 Native 위치 보정 빌드의 결과로 바꾸어 보고하지 않는다.

원본 HML은 RATIO·RELSIZE 자식과 수식 SHAPEOBJECT가 빠져 있다. 한컴 저장본은 장평·상대 크기
0과 떠 있는 수식을 기록하지만 RHWP HML adapter는 기본 백분율과 인라인 상자를 사용한다.
원본의 호환 해석과 정상 HML export의 100 기본값 계약은 구분해야 한다.
HWP 저장 수식 캐시의 재계산·미세 첨자와 기호 차이도 같은 파생 진단에서 남았다.
원인이 같다고 확정하지 않는다.

## 유지·복구 조건

일반 HML parser/serializer, 누락 RATIO·RELSIZE·ShadeColor 저장 계약 및 원본·Print PDF는 유지한다.
원래 37개 시각 입력 중 HML 1개와 그 저장본 진단을 #7445로 분리하고 나머지 36개를 계속 검증한다.
TSV·JSON·로그·active/deferred manifest는 ignored
`output/pr-review/issue7642/equation-regression-scope-20261008/hml-transfer-7445/`와
상위 폴더에만 보존한다.

#7445에서 원본의 모든 페이지를 Native/fresh WASM과 독립 Print로 대조해 최저 90% 이상을
확인하고, 작은 본문·수식 크기·첨자·기호·앞뒤 간격을 확대 판독한 뒤 회귀를 복구한다.
실물 배치는 고정 px 대신 문단·줄·개체 소유와 내용 순서·보존·포함·겹침 관계로 검사한다.
