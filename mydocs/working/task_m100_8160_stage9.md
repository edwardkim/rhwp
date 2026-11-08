# #8160 9단계: sample16 회귀 검사 이름 통합

## 요청과 범위

사용자 지시: 기존 hwp3-sample16 관련 회귀 테스트를 모두 제거하고 `issue_8160` 이름으로 새로 등록한 뒤 커밋한다.
작업 브랜치는 `task/8160-sample16-raw100`, 시작 head는
`021c486d1eaee17920a2b3e82ae896115b847669`다. 실제 작업 호스트는 `ubuntu-ted2`다.
원격 push·머지·이슈 종료 등 후속 처리는 승인 대기를 유지한다.

전용 검사뿐 아니라 공용 corpus·baseline의 sample16 등록도 정리한다. 원본·PDF·독립 기대값은 보존한다.
다른 문서 검사와 섞인 파일은 해당 sample16 부분만 분리한다. 범용 CLI·사용자 흐름 예시는
다른 기존 입력으로 계속 검사하고, 혼합 메타데이터 계약의 다른 정상 문서 행도 함께 보존한다.

## 다른 호스트와의 대조

사용자가 제공한 삭제·수정 목록의 출처를 실제 SSH로 확인했다. 아래는 신규·mydocs를 제외한 비교다.
다른 호스트의 작업 트리·브랜치·커밋은 변경하지 않았다.

| 호스트 | 브랜치 | 확인한 제거 변경 | 삭제/수정 |
| --- | --- | --- | --- |
| ubuntu-ted | `review/semanticist21-20261019` | `1ef34f94452493db15a8e33e641f97b81ed74916` | 26/58 |
| ubuntu-ted3 | `task/8153-handbook-raw100` | `075fddfe04..49fc235dbcbc17114d28da89dbb15941072b67cc` | 26/50 |

ubuntu-ted는 관찰 중 미커밋 변경을 커밋했고, 이어 `43e2697fe76e495fd6363123b3021270eac54cc7`에서
보관 자료를 제거했다. ubuntu-ted3는 `a45c1826db`, `d121453685`, `49fc235dbc`에서 제거·보관 정리를 완료했다.
확인 시점에 변환본 catalog 5개는 양쪽 모두 남아 있었고 원본 catalog는 ubuntu-ted3에서만 삭제돼 있었다.
이 단계에서는 원본·변환본 6개를 함께 정리하고 생성기에서 다시 생기지 않게 했다.

기존 혼합 진단의 K-water 항목, 메타데이터 검사의 정상 대조 문서, 일반 CLI 봉투·journey 예시는
그대로 보존하거나 sample16 이외의 기존 표본으로 이어 간다. 다른 호스트의 제거 diff를 그대로
복사해 관련 없는 정상 보호까지 잃지 않도록 실제 검사 단위로 대조했다.

## 보호 유지와 중복 제거

- 새 Rust 모듈 40개, 함수 86개와 실제 WASM JPEG 검사 1개를 등록했다.
- 기존 74개 테스트 본문은 모든 기대값·assertion을 그대로 이관했다. 이전/새 함수명으로 대조한다.
- 혼합 HFT 검사와 혼합 source-side 테두리 검사의 sample16 부분을 각각 독립 함수로 분리했다.
- parser source-side 원본 정렬 검사 1개를 integration으로 옮기고 입력 누락의 조용한 skip을 제거했다.
- 공용 corpus의 sample16 스타일 단사성·서로 다른 스타일 보존·동등성 경계를 한 검사로 모았다.
- 원본 CHAR_SHAPE의 검정 음영 방지·독립 한컴 회색 `0x00d8d8d8`·색 집합 보존을
  실제 변환기와 public 저장 경로 양쪽에서 한 검사로 보호한다.
- 단순 2022/2024 쪽수 검사 2개는 각각 64쪽의 내용·주소·소유 전쪽 계약에 통합했다.
- 기대값 없이 출력만 하던 진단 11개 파일·13개 함수는 제거했다. 혼합 진단 2개는 K-water를 유지한다.
- 옛 16쪽 placeholder catalog 6개와 집계·fingerprint·isolation·PDF manifest의 해당 행을 제거했다.
  생성기를 ignored 폴더에 실행해 초기 나머지 TSV 13개가 정식 파일과 바이트 단위로 일치함을 확인했다. 편람 제거 후 최종 11개도 재확인했다.
- `hwp2022.json`, `hwp2024.json`, `hwpx.json`과 독립 JPEG PNG는 바이트를 바꾸지 않았다.
  572개 내용/기준선 앵커, 630개 영역, ±2px, BCP 두 원본 줄과 그 뒤 본문 소유를 보존한다.
- renderer·parser의 제품 실행 코드는 바꾸지 않았다. 두 큰 source 파일은 `#[cfg(test)]` 내부 이관뿐이다.
  serializer의 공개 same-id corpus 상수에서는 sample16 행 하나와 편람 HWPX 행 34개를 제거했다. 이 상수는 테스트가 소비하며
  실제 문서의 조판·렌더링·직렬화 소비 경로를 바꾸지 않는다.

### 이전 등록에서 새 검사로의 대응

| 이전 등록 | 새 활성 파일 | 처리 |
| --- | --- | --- |
| `tests/cases/verified_sample16_all_pages.rs` | [issue_8160_all_pages.rs](../../tests/cases/issue_8160_all_pages.rs) | reregister, 3개 |
| `tests/cases/verified_sample16_2024_print_boundaries.rs` | [issue_8160_hwp2024_print_boundaries.rs](../../tests/cases/issue_8160_hwp2024_print_boundaries.rs) | reregister, 5개 |
| `tests/cases/verified_sample16_equation_reading_order.rs` | [issue_8160_equation_reading_order.rs](../../tests/cases/issue_8160_equation_reading_order.rs) | reregister, 2개 |
| `tests/cases/verified_hwp3_original_print_boundaries.rs` | [issue_8160_hwp3_print_boundaries.rs](../../tests/cases/issue_8160_hwp3_print_boundaries.rs) | reregister, 5개 |
| `tests/cases/issue_7445_hwp3_source_frame_ownership.rs` | [issue_8160_hwp3_source_frame_ownership.rs](../../tests/cases/issue_8160_hwp3_source_frame_ownership.rs) | reregister, 3개 |
| `tests/cases/issue_7642_sample16_2022_paragraph_relations.rs` | [issue_8160_hwp2022_paragraph_relations.rs](../../tests/cases/issue_8160_hwp2022_paragraph_relations.rs) | reregister, 3개 |
| `tests/cases/issue_7642_page_border_center_relations.rs` | [issue_8160_page_border_center_relations.rs](../../tests/cases/issue_8160_page_border_center_relations.rs) | reregister, 1개 |
| `tests/cases/issue_7642_stored_column_margin_owner_relations.rs` | [issue_8160_stored_column_margin_ownership.rs](../../tests/cases/issue_8160_stored_column_margin_ownership.rs) | reregister, 2개 |
| `tests/cases/issue_3518_hwp3_sample16_page_roundtrip.rs` | [issue_8160_hwp3_page_roundtrip.rs](../../tests/cases/issue_8160_hwp3_page_roundtrip.rs) | reregister, 2개 |
| `tests/cases/issue_6060_fullwidth_cjk_quote.rs` | [issue_8160_fullwidth_cjk_quote.rs](../../tests/cases/issue_8160_fullwidth_cjk_quote.rs) | reregister, 1개 |
| `tests/issue_1008_gradient.rs` | [issue_8160_gradient_and_font_mapping.rs](../../tests/cases/issue_8160_gradient_and_font_mapping.rs) | reregister, 3개 |
| `tests/issue_1116.rs` | [issue_8160_toc_and_latin_font.rs](../../tests/cases/issue_8160_toc_and_latin_font.rs) | reregister, 6개 |
| `tests/issue_hwp3_bookmark_native.rs` | [issue_8160_bookmark_roundtrip.rs](../../tests/cases/issue_8160_bookmark_roundtrip.rs) | reregister, 2개 |
| `tests/issue_1105.rs` | [issue_8160_saved_page_boundaries.rs](../../tests/cases/issue_8160_saved_page_boundaries.rs) | extract, 9개 |
| `tests/cases/issue_4367_hwp3_convert_fourth_contract.rs` | [issue_8160_hwp3_storage_contract.rs](../../tests/cases/issue_8160_hwp3_storage_contract.rs) | extract, 2개 |
| `tests/cases/issue_5696_hwp3_lineseg_subtracts_binding_margin.rs` | [issue_8160_zero_gutter_source_width.rs](../../tests/cases/issue_8160_zero_gutter_source_width.rs) | extract, 1개 |
| `tests/cases/issue_4961_font_decision_trace.rs` | [issue_8160_unmodelled_source_geometry.rs](../../tests/cases/issue_8160_unmodelled_source_geometry.rs) | extract, 1개 |
| `tests/cases/issue_6865_wmf_monochrome_mask_rop.rs` | [issue_8160_wmf_mask_ownership.rs](../../tests/cases/issue_8160_wmf_mask_ownership.rs) | extract, 1개 |
| `tests/cases/issue_7445_original_font_program_ownership.rs` | [issue_8160_font_program_ownership.rs](../../tests/cases/issue_8160_font_program_ownership.rs) | extract, 3개 |
| `tests/cases/issue_4680_hwp3_paragraph_style_reference.rs` | [issue_8160_nondefault_source_style.rs](../../tests/cases/issue_8160_nondefault_source_style.rs) | extract, 1개 |
| `tests/issue_1139_inline_picture_duplicate.rs` | [issue_8160_footer_border_ownership.rs](../../tests/cases/issue_8160_footer_border_ownership.rs) | extract, 1개 |
| `tests/issue_2158_hwpx_vpos_reset_preserve.rs` | [issue_8160_hwpx_stored_page_ownership.rs](../../tests/cases/issue_8160_hwpx_stored_page_ownership.rs) | extract, 1개 |
| `tests/suites/issue_regression_pilot/issue_1035_alignment.rs` | [issue_8160_all_pages.rs](../../tests/cases/issue_8160_all_pages.rs) | merge-page-count-into-content-and-owner-contract, 2개 |
| `tests/suites/issue_regression_pilot/issue_1035_alignment.rs` | [issue_8160_legacy_2018_page_count.rs](../../tests/cases/issue_8160_legacy_2018_page_count.rs) | reregister, 1개 |
| `tests/issue_1086.rs` | [issue_8160_legacy_hwp5_page_count.rs](../../tests/cases/issue_8160_legacy_hwp5_page_count.rs) | extract, 1개 |
| `tests/cases/issue_7445_hft_glyph_paint_frame.rs` | [issue_8160_hft_print_frame.rs](../../tests/cases/issue_8160_hft_print_frame.rs) | split-mixed-document-test, 1개 |
| `src/parser/hwp3/mod.rs` | [issue_8160_hwp3_parser_alignment.rs](../../tests/cases/issue_8160_hwp3_parser_alignment.rs) | move-source-test-no-silent-fixture-skip, 1개 |
| `src/document_core/queries/rendering.rs` | [issue_8160_page_border_center_relations.rs](../../tests/cases/issue_8160_page_border_center_relations.rs) | split-source-test-public-api, 1개 |
| `tests/hwp5_password_fixture.rs` | [issue_8160_hwp5_password.rs](../../tests/cases/issue_8160_hwp5_password.rs) | reregister, 2개 |
| `tests/cases/info_hancom_save_version_contract.rs` | [issue_8160_save_version_metadata.rs](../../tests/cases/issue_8160_save_version_metadata.rs) | extract, 2개 |
| `tests/digest_v2_contract.rs` | [issue_8160_digest_structure_addresses.rs](../../tests/cases/issue_8160_digest_structure_addresses.rs) | extract, 2개 |
| `tests/explain_contract.rs` | [issue_8160_encrypted_explain_contract.rs](../../tests/cases/issue_8160_encrypted_explain_contract.rs) | extract, 3개 |
| `tests/cli_exit_codes_diagnostic_commands.rs` | [issue_8160_encrypted_record_dump_contract.rs](../../tests/cases/issue_8160_encrypted_record_dump_contract.rs) | extract, 1개 |
| `tests/issue_3693_structure_clause_context.rs` | [issue_8160_agreement_clause_context.rs](../../tests/cases/issue_8160_agreement_clause_context.rs) | extract, 1개 |
| `tests/issue_3695_structure_auto_policy.rs` | [issue_8160_agreement_auto_structure.rs](../../tests/cases/issue_8160_agreement_auto_structure.rs) | extract, 1개 |
| `tests/issue_3744_structure_clause_confidence.rs` | [issue_8160_agreement_clause_confidence.rs](../../tests/cases/issue_8160_agreement_clause_confidence.rs) | extract, 1개 |
| `tests/issue_3915_verify_axes_both_reported.rs` | [issue_8160_verify_axes.rs](../../tests/cases/issue_8160_verify_axes.rs) | extract, 1개 |
| `tests/cases/issue_4680_hwp3_style_pool_dedup.rs` | [issue_8160_source_style_pool.rs](../../tests/cases/issue_8160_source_style_pool.rs) | merge-sample16-corpus-assertions, 1개 |
| `tests/issue_4155_hwp3_char_shade_contract.rs` | [issue_8160_saved_character_shading.rs](../../tests/cases/issue_8160_saved_character_shading.rs) | merge-sample16-corpus-assertions, 1개 |

## 검증

실행 로그는 `output/pr-review/issue8160-20261022/test-reorganization/`에 보존한다.
완료한 실제 결과와 검증한 후보를 구별한다.

- 초기 전용 검사 후보의 집중 nextest는 131/131 PASS다. 이후 공용 참조를 보완한 최종 후보의 결과와 구별한다.
- 초기 Native Skia 집중 실행은 `0 tests`를 감지해 **검증 불인정**으로 기록했다.
  포맷 변경 뒤 weighted suite 배정이 바뀐 것이 원인이다. 포맷 완료 후 suite를 다시 생성하고
  실행 이름·건수를 확인한다. `skia-zero-selection.log`와 `skia-initial-selection.json`에 보존했다.
- 자동 suite 도구 25/25 PASS. 최종 후보에서 해당 도구와 실행 범위를 다시 확인한다.
- 실제 표준 WASM JPEG 브라우저 1/1 PASS. 실제 제품은 기존 `5df67d329` 최적화 패키지다.
- 새 전쪽 PNG 진입점으로 기존 산출물을 다시 비교해 384/384 raw100·정수 미대응 0/0을 확인했다.
  새 제품 렌더링으로 표시하지 않는다. 제품·입력/PDF·글꼴 조건은 [8단계](task_m100_8160_stage8.md)와
  [최종 전쪽 근거](../pr/assets/issue_8160/final-9418/README.md)를 따른다.

### 최종 후보의 필수 검증 결과

복원 전 79개 후보의 lint·suite 정책 검사는 통과했다. 전체 nextest는 바이너리 컴파일 중
사용자 추가 요청에 따른 7개 검사 복원을 반영하기 위해 종료했고, 테스트 함수는 실행되지 않았다.
이 실행은 assertion 실패나 최종 후보의 통과 결과로 세지 않는다. 완료한 빌드 캐시는 보존했다.

복원 후 86개 후보는 포맷·Native Clippy·WASM lib Clippy·workspace build·workspace 전체 target Clippy·
suite 정책·unit tier 정책을 통과했다. 보정 후 전체 nextest는 **11,516/11,516 PASS**,
기본 제외 45개이며 #8160 기본 85개의 실제 실행도 누락·예상 외 0건으로 통과했다.
이 실행 시간은 1,587.417초다. Native Skia 집중 **3/3 PASS**도 실제 이름 대조까지 통과했다.
기본 85개와 Native 전용 1개를 합쳐 #8160 고유 함수 86개다. Skia 실행 3개 중 2개는 기본과 중복된다.
현재 작업 브랜치와 검증 checkout의 소스·fixture 5,022개는 바이트 단위로 일치한다.
수정 문서의 로컬 링크 116개는 누락이 없다.

새로 복원한 도형 호스트 4개는 현재 후보의 실제 바이너리에서 먼저 실행해 4/4 PASS했다.
새 MCP HWPX PDF 8쪽에서 제목 기준선 354.239990234375px, 표 상단 366px,
하단 994.5919596354166px를 다시 읽어 기존 기대값과 일치함을 확인했다.
이는 이미 처리한 sample16 입력의 복원 검증이며, 선행 PR 반영 후 보존 검증을 대신하지 않는다.

## 추가 요청: 편람 회귀 제거

사용자 지시에 따라 `2025 행정업무운영 편람(최종).hwp`와 `.hwpx`를 사용하는 기존
회귀를 제거한다. 전용 검사뿐 아니라 상수·helper 호출 관계, 동적 corpus 수집,
fixture 생성기, Studio E2E의 실행 등록과 간접 입력도 추적한다. 다른 문서의 검사는
유지하며, 두 원본과 기준 PDF·과거 검증 증적은 보존한다. 편람의 신규 회귀는 다른 PR에서
추가하므로 이 단계에서는 재등록하지 않는다. 기존 sample16의 `issue_8160` 통합은 유지한다.


### 편람 제거 범위와 실행 경로

두 편람 파일은 이번 PR의 문서 검토·시각 비교·회귀 검증 범위에서 제외한다.
이 절의 확인은 검사 등록·수집·생성 경로 제거의 검증이며 편람 문서의 조판 결과를 재평가한 것이 아니다.

- 전용 파일 14개와 혼합 파일의 편람 부분에서 Rust 테스트 함수 54개를 제거했다.
  일반 문서·합성 알고리즘 대조군은 유지한다. `issue_4764_pdf_raster_fidelity`의 혼합 검사는
  편람 쪽수·landmark·catalog 항목만 제거하고 다른 7개 corpus 185쪽과 isolation 1,480건을 보호한다.
- `tests/fixtures/transferred_document_regressions.txt`의 두 정확한 basename을
  `corpus_scope`, `deferred_exam_scope`, `exam_fixtures`가 읽는다. HWP5/HWPX 왕복,
  convert-verify, render-diff, IR, envelope, 글자 모양·음영, 보안, 본문/셀 넘침·겹침·쪽수·render-page
  수집에서 재등장하지 않게 한다. basename·상대/절대 경로·Windows 구분자를 확인했다.
- 편람 baseline·same-id corpus 행, #4764 catalog 2개와 집계·fingerprint·isolation·PDF manifest 행을 제거했다.
  #4764 생성기와 글자 모양 추출 생성기에도 반영했다.
- Studio의 편람 전용 #3820 쪽수·loading busy cursor E2E와 npm/MANIFEST 등록을 제거했다.
  #4741은 편람 로딩 부분을 제거하고 합성 Canvas2D face·폭·캐시 검사를 유지했다.
  WMF crop 브라우저 검사의 편람 전용 진입점과 기존 묶음 import를 제거했다.
- 숨은 글자 clean fixture와 생성기의 편람 항목을 제거하고 catalog 성공 192건·예외 22건·총 214건을 맞췄다.
- bulk pipeline의 편람 sample·journey와 T04/T18/T19, recipe의 T09-B06/T09-J04 발췌·생성 경로를 제거했다.
  agent bench t08, security SE13/20/21/32/41/42/53/66과 대응 reference, HWP control 탐침 2개를 제거했다.
- 제품의 문서 열기·변환 API는 제외 목록을 적용하지 않는다. 사용자가 지정한 원본을 읽는 기능과
  원본/PDF/과거 증적, 다른 문서 입력·일반 합성 알고리즘 검사는 보존한다.

### 현재까지 완료한 추가 검증

- 실제 Rust 공용 helper를 포함한 임시 드라이버: 경로 변형의 제외·정상 대조 36건 PASS.
- #4764 생성: 남은 11개 TSV 바이트 일치. 글자 모양 생성기의 실제 동적 열거 1,276건에서 이관 입력 0건.
  이 열거 검증에서는 비싼 parser와 파일 쓰기만 가로챘으며 실제 추출 완료로 표시하지 않는다.
- recipe 발췌 생성: 107건, 이관 입력 0건. 숨은 글자 전체 fixture 재생성: 기존 JSON과 차이 0건.
- 관련 Python fixture/생성/범위 검사 86건 PASS. CI Impact metadata·HWP control 일반 계약 53건,
  합성 multiver envelope 5건 PASS.
- suite 도구 25건 PASS, 실제 표준 WASM Canvas 묶음 8건 PASS, #4741 합성 글꼴 E2E PASS.
- sample16 암호 UI는 issue_8160 HWP5와 일반 HWP3/HWPX 묶음을 실제 브라우저에서 모두 통과했다.
  일반 HWPX의 옛 23쪽 핀은 #7445의 독립 Print 24쪽 기록과 충돌하므로 제거하고 문서/metadata 쪽수 관계를 보호한다.
  64쪽 HWP5의 첫 Canvas 준비 timeout만 120초로 명시해 잘못된 조기 실패를 정정했다.
- sample16 이관 기존 함수 본문 74개 일치·좌표 JSON 3개 해시 불변·최초 이관 후보의 issue_8160 함수 79개를 재확인했다.

전체 Rust·Native Skia 최종 실행은 아래 최종 결과에 기록한다. 실행 로그·임시 드라이버·생성 비교물은 커밋하지 않는다.

## 기존 파일 삭제·수정 목록 (신규 생성 제외)

Git의 rename 표시는 이전 경로 삭제와 새 경로 생성으로 분리해 기록한다. 아래 목록은 현재 단계의 sample16 통합과 편람 등록 제거를 합친 변경이다.

### 삭제: 81개

- `.claude/skills/rhwp-bulk-pipeline/examples/lists/convert_ok.txt`
- `.claude/skills/rhwp-bulk-pipeline/examples/transcripts/T04.ndjson`
- `.claude/skills/rhwp-bulk-pipeline/examples/transcripts/T18.ndjson`
- `.claude/skills/rhwp-bulk-pipeline/examples/transcripts/T19.ndjson`
- `.claude/skills/rhwp-bulk-pipeline/fixtures/traces/T04.json`
- `.claude/skills/rhwp-bulk-pipeline/fixtures/traces/T18.json`
- `.claude/skills/rhwp-bulk-pipeline/fixtures/traces/T19.json`
- `.claude/skills/rhwp-recipes/fixtures/transcripts/T09-B06.json`
- `.claude/skills/rhwp-recipes/fixtures/transcripts/T09-J04.json`
- `gym/packs/security/reference/SE13.json`
- `gym/packs/security/reference/SE20.json`
- `gym/packs/security/reference/SE21.json`
- `gym/packs/security/reference/SE32.json`
- `gym/packs/security/reference/SE41.json`
- `gym/packs/security/reference/SE42.json`
- `gym/packs/security/reference/SE53.json`
- `gym/packs/security/reference/SE66.json`
- `gym/packs/security/tasks/SE13.json`
- `gym/packs/security/tasks/SE20.json`
- `gym/packs/security/tasks/SE21.json`
- `gym/packs/security/tasks/SE32.json`
- `gym/packs/security/tasks/SE41.json`
- `gym/packs/security/tasks/SE42.json`
- `gym/packs/security/tasks/SE53.json`
- `gym/packs/security/tasks/SE66.json`
- `rhwp-studio/e2e/issue-3820-hwp-hwpx-page-count.test.mjs`
- `rhwp-studio/e2e/loading-busy-cursor.test.mjs`
- `scripts/tests/print-jpeg-frame.browser.mjs`
- `scripts/tests/wmf-picture-crop.browser.mjs`
- `scripts/verify_sample16_raw100.py`
- `tests/cases/declared_table_height_last_row_absorbs.rs`
- `tests/cases/empty_host_stored_ladder_table_box.rs`
- `tests/cases/issue_3518_hwp3_sample16_page_roundtrip.rs`
- `tests/cases/issue_4753_group_image_hittest.rs`
- `tests/cases/issue_6060_fullwidth_cjk_quote.rs`
- `tests/cases/issue_6389_cell_stored_ladder_compresses_to_fit.rs`
- `tests/cases/issue_6907_hpf_manifest_keeps_master_page_owner.rs`
- `tests/cases/issue_7009_handbook_structure_landmarks.rs`
- `tests/cases/issue_7445_hwp3_source_frame_ownership.rs`
- `tests/cases/issue_7642_page_border_center_relations.rs`
- `tests/cases/issue_7642_sample16_2022_paragraph_relations.rs`
- `tests/cases/issue_7642_stored_column_margin_owner_relations.rs`
- `tests/cases/issue_7751_handbook_completed_boundaries.rs`
- `tests/cases/issue_7751_print_paper_frames.rs`
- `tests/cases/issue_7751_separated_cell_text_frames.rs`
- `tests/cases/issue_7751_wmf_picture_payload.rs`
- `tests/cases/issue_8151_nested_overlay_outer_margin.rs`
- `tests/cases/pagehide_marker_page_break_blank_page.rs`
- `tests/cases/verified_hwp3_original_print_boundaries.rs`
- `tests/cases/verified_sample16_2024_print_boundaries.rs`
- `tests/cases/verified_sample16_all_pages.rs`
- `tests/cases/verified_sample16_equation_reading_order.rs`
- `tests/diag_1042_2022.rs`
- `tests/diag_1042_2024_reverse.rs`
- `tests/diag_1042_cfb_check.rs`
- `tests/diag_1042_hwp3_vs_hwp5_paragraph.rs`
- `tests/diag_1042_hwp_summary.rs`
- `tests/diag_1042_normal_vs_abnormal.rs`
- `tests/diag_1042_pi162_attr1.rs`
- `tests/diag_1042_raw_binary.rs`
- `tests/diag_1042_trailing.rs`
- `tests/diag_1042_used_breakdown.rs`
- `tests/diag_1042_vpos_distribution.rs`
- `tests/fixtures/inspect_msec/envelopes/hidden-text/ht-clean-sample-2025_행정업무운영_편람_최종_.hwpx.json`
- `tests/fixtures/issue_4764/catalog_admin_hwp.tsv`
- `tests/fixtures/issue_4764/catalog_admin_hwpx.tsv`
- `tests/fixtures/issue_4764/catalog_hwp3_hwp5_2010.tsv`
- `tests/fixtures/issue_4764/catalog_hwp3_hwp5_2018.tsv`
- `tests/fixtures/issue_4764/catalog_hwp3_hwp5_2020.tsv`
- `tests/fixtures/issue_4764/catalog_hwp3_hwp5_2022.tsv`
- `tests/fixtures/issue_4764/catalog_hwp3_hwp5_2024.tsv`
- `tests/fixtures/issue_4764/catalog_hwp3_sample16.tsv`
- `tests/hwp5_password_fixture.rs`
- `tests/issue_1008_gradient.rs`
- `tests/issue_1116.rs`
- `tests/issue_3930_hwpx_hwp_save_layout.rs`
- `tests/issue_3931_declared_rowbreak.rs`
- `tests/issue_hwp3_bookmark_native.rs`
- `tests/suites/issue_regression_pilot/issue_1035_alignment.rs`
- `tools/hwpctrl_compat/probes/p2-footnote-chain.json`
- `tools/hwpctrl_compat/probes/p2-list-enum.json`

### 수정: 96개

- `.claude/skills/rhwp-bulk-pipeline/fixtures/journeys.json`
- `.claude/skills/rhwp-bulk-pipeline/fixtures/samples.json`
- `.claude/skills/rhwp-bulk-pipeline/fixtures/traces_index.json`
- `.claude/skills/rhwp-bulk-pipeline/references/26_worked_traces.md`
- `.claude/skills/rhwp-bulk-pipeline/references/30_corpus.md`
- `.claude/skills/rhwp-bulk-pipeline/references/_gen_pack.py`
- `.claude/skills/rhwp-recipes/fixtures/transcripts_index.json`
- `.claude/skills/rhwp-recipes/references/_gen_pack.py`
- `gym/packs/security/README.md`
- `mydocs/plans/task_m100_8160.md`
- `mydocs/working/archives/task_m100_1042_stage1.md`
- `mydocs/working/task_m100_8160_stage5.md`
- `rhwp-studio/e2e/MANIFEST.md`
- `rhwp-studio/e2e/hwp-password-open.test.mjs`
- `rhwp-studio/e2e/local-font-partial-enumeration-issue4741.test.mjs`
- `rhwp-studio/package.json`
- `scripts/extract_char_shape_ir.py`
- `scripts/generate_issue_4764_residual_fixtures.py`
- `scripts/tests/run-rust-test-locking.test.mjs`
- `scripts/tests/rust-test-suite-manifest.test.mjs`
- `scripts/tests/test_gym_security_pack.py`
- `scripts/tests/test_review_only_fast_pass_workflows.py`
- `scripts/tests/visual-sweep-canvas-fonts.browser.mjs`
- `src/document_core/queries/rendering.rs`
- `src/parser/hwp3/mod.rs`
- `src/serializer/hwpx/char_shape_tables/same_id_corpus.rs`
- `tests/batch_extract_data_contract.rs`
- `tests/cases/agent_bulk_pipeline_skill_contract.rs`
- `tests/cases/agent_doc_triage_cli_ladder.rs`
- `tests/cases/armor_cli_contract.rs`
- `tests/cases/body_overflow_baseline.rs`
- `tests/cases/info_hancom_save_version_contract.rs`
- `tests/cases/issue_3500_char_shapes_roundtrip.rs`
- `tests/cases/issue_4367_hwp3_convert_fourth_contract.rs`
- `tests/cases/issue_4680_hwp3_paragraph_style_reference.rs`
- `tests/cases/issue_4680_hwp3_style_pool_dedup.rs`
- `tests/cases/issue_4764_pdf_raster_fidelity.rs`
- `tests/cases/issue_4961_font_decision_trace.rs`
- `tests/cases/issue_5696_hwp3_lineseg_subtracts_binding_margin.rs`
- `tests/cases/issue_5952_cell_note_overflow.rs`
- `tests/cases/issue_6856_drawing_structure_errors.rs`
- `tests/cases/issue_6865_wmf_monochrome_mask_rop.rs`
- `tests/cases/issue_7387_use_font_space_latin_slot.rs`
- `tests/cases/issue_7445_hft_glyph_paint_frame.rs`
- `tests/cases/issue_7445_original_font_program_ownership.rs`
- `tests/cases/issue_7445_stored_paragraph_flow_projection.rs`
- `tests/cases/off_canvas_baseline.rs`
- `tests/cases/security_hwp3_paragraph_depth.rs`
- `tests/cases/text_overlap_baseline.rs`
- `tests/cli_exit_codes_diagnostic_commands.rs`
- `tests/common/corpus_scope.rs`
- `tests/diag_1042_variant_check.rs`
- `tests/diag_1042_version_check.rs`
- `tests/digest_v2_contract.rs`
- `tests/explain_contract.rs`
- `tests/extract_data_contract.rs`
- `tests/fixtures/agent_doc_triage/journeys.json`
- `tests/fixtures/body_overflow_baseline.tsv`
- `tests/fixtures/char_shapes/corpus_same_id_para_char_shapes.jsonl`
- `tests/fixtures/deferred_visual_regression_inputs.tsv`
- `tests/fixtures/inspect_msec/README.md`
- `tests/fixtures/inspect_msec/catalog.json`
- `tests/fixtures/inspect_msec/matrices/catalog.tsv`
- `tests/fixtures/ir_field_sweep_baseline.tsv`
- `tests/fixtures/issue8160/README.md`
- `tests/fixtures/issue_4764/README.md`
- `tests/fixtures/issue_4764/catalog_all.tsv`
- `tests/fixtures/issue_4764/fingerprints.tsv`
- `tests/fixtures/issue_4764/isolation_cases.tsv`
- `tests/fixtures/issue_4764/pdf_pages.tsv`
- `tests/fixtures/off_canvas_baseline.tsv`
- `tests/fixtures/oracle_page_count_baseline.tsv`
- `tests/fixtures/render_page_samples.tsv`
- `tests/fixtures/text_overlap_baseline.tsv`
- `tests/info_title_contract.rs`
- `tests/issue_1086.rs`
- `tests/issue_1105.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`
- `tests/issue_2158_hwpx_vpos_reset_preserve.rs`
- `tests/issue_3693_structure_clause_context.rs`
- `tests/issue_3695_structure_auto_policy.rs`
- `tests/issue_3739_hwpx_same_char_shape_boundary.rs`
- `tests/issue_3744_structure_clause_confidence.rs`
- `tests/issue_3915_verify_axes_both_reported.rs`
- `tests/issue_4155_hwp3_char_shade_contract.rs`
- `tests/issue_929.rs`
- `tests/llm_export_contract.rs`
- `tests/mcp_password_contract.rs`
- `tests/overflow_cell_baseline.rs`
- `tests/support/deferred_exam_scope.rs`
- `tests/support/exam_fixtures.rs`
- `tests/unicode_deception_contract.rs`
- `tools/agent_bench/tasks.json`
- `tools/hwpctrl_compat/probes/README.md`
- `tools/inspect_msec/gen_msec_fixtures.py`
- `tools/llm_verifier/oracle_vs_self/fixtures/envelopes/multiver_disagree.json`

## 두 호스트의 후속 제거 재대조와 복원

2026-10-25 사용자 추가 지시: `ubuntu-ted`의 PR을 먼저 처리한다. 그 PR과
`ubuntu-ted3`에서 sample16과 함께 잘못 빠진 보호는 이 PR에서 복원한다.
원격 호스트의 dirty 작업은 변경하지 않았다. 비교는 고정한 제거 커밋과 그 부모의 원문을 읽었다.

| 호스트 | 제거 커밋 | 제거 전 원문 |
| --- | --- | --- |
| ubuntu-ted | `1ef34f94452493db15a8e33e641f97b81ed74916` | `3859d1948444deadc6a5bc6e5343911f870f97f1` |
| ubuntu-ted3 | `a45c1826db19209596b4436bfd2f62f975fd96d3` | `075fddfe048ba02ae902333d6df65cb45d0fdc97` |

첫 대조 시점의 head는 각각 `d3344f46b3c0500b41d8ce5e7bc96f450fcea1ea`,
`3606a4de0e9d0e67210bf73c37a7c509151bf29c`였다. 이때 ubuntu-ted의 미커밋 변경은
확정 근거로 복사하지 않았다. 재확인 시점에는 ubuntu-ted가
`ee05c8f28f83d7d541382f423d57092e2dca90e5`, ubuntu-ted3가
`73b1589aa6`까지 진행했다. ubuntu-ted3의 `tests/llm_export_contract.rs` 미커밋 변경도 보존했다.
ubuntu-ted의 `b3a9ef3d089309feedc5fbbb6b88466ca5de7de8`에서 공용 제외 목록과
Python·Studio helper·oracle-public 동적 수집·page-roundtrip의 옛 실패 등록 정리가 고정됐다.
이 경로는 선행 PR 반영 후 통합하며 현재 브랜치에서 이미 전부 차단됐다고 판정하지 않는다.

- 두 호스트에서 제거된 테스트 179개 항목(호스트 간 중복 포함)을 기존 이름·#8160 대응·병합·출력 전용 진단으로 분류했다.
  대응이 있는 기존 테스트의 assertion 차이는 0건이었다.
- `stored_shape_host_physical_frame.rs`는 이 PR의 옛 base에 없던 선행 작업의 검사라 처음 이관에서
  빠졌음을 확인했다. [복원 파일](../../tests/cases/issue_8160_stored_shape_host_physical_frame.rs)에
  HWP/HWPX 각 2개, 총 4개 함수를 기대값·helper와 함께 복원했다. 함수명만 #8160으로 바꿨다.
- 암호 문서의 Native 열기는 MCP·armor 프로토콜 보호를 대체하지 않는다. 공유 fixture에서 빠진
  HWP5의 [MCP 세션·무상태 전달](../../tests/cases/issue_8160_mcp_password.rs) 2개와
  [armor stdin·응답 봉투](../../tests/cases/issue_8160_armor_password.rs) 1개도 #8160으로 복원했다.
  64쪽 및 암호 비노출·오류·응답 기대값을 유지한다.
- 옛 BCP 이름은 이미 원본 두 줄·두 기준선·뒤 문단 소유 검사가 보호한다. 별도로 중복 등록하지 않는다.
- K-water 진단과 혼합 메타데이터의 정상 문서 보호는 유지했다. sample16의 출력 전용 버전 진단은
  #8160 실제 메타데이터 검사로 이어 가며 print-only 함수를 다시 등록하지 않는다.
- 현재 #8160은 40개 Rust 모듈·86개 테스트 함수다. 기존 79개 후보의 실행 결과와 복원 후 후보의
  검증 결과를 구분한다. 검증 checkout은 기존 실행 중에 바꾸지 않았다.

ubuntu-ted PR 선행 반영 후 최신 devel 위에서 이 PR을 리베이스하고, 공용 제외 목록을 하나로
정리하며 선행 PR의 개선과 sample16 세 문서의 전쪽 raw100 보존을 함께 확인한다.
이번 PR의 검토·시각 비교·회귀 범위에서 제외한 편람 두 파일은 복원하지 않는다.


### 전체 실행에서 확인한 fixture 분리 보정

복원 후 첫 전체 실행은 11,516개 중 11,514개 PASS, 2개 FAIL, 기본 제외 45개로 종료했다.
#8160의 기본 85개는 이름 대조까지 전부 PASS였다. 실패 두 건 중 일반 MCP 무상태 검사에서 입력이 HWP3로 바뀐 뒤에도 64쪽 기대값이
남아 있음을 검출했다. 제품은 실제 HWP3의 24쪽을 반환했다. 이 단계의 분리 과정에서 만든
불일치이므로 기대값을 임의 완화하지 않고 같은 `FIXTURES[0]` 행의 `Some(24)`를 읽도록 고쳤다.
sample16 HWP5의 새 #8160 검사는 64쪽을 그대로 보호한다.

일반·#8160 MCP helper는 #3289의 기존 `rhwp_bin()` 방식에 맞춰 nextest archive가 주입하는
런타임 실행 경로를 먼저 읽도록 정리했다. 암호 비노출·응답·페이지 기대값은 유지했다.
기존 전체 실행의 검증 checkout은 실행 중에 바꾸지 않았고, 종료 후 보정 후보의 lint·집중 MCP·
전체 nextest·Native Skia를 순차로 다시 실행한다.

#4764의 합성 PDF builder 검사는 제거 전 catalog를 전제로 23쪽마다 표본을 만들고
20개 초과를 요구해 남은 catalog의 12개 표본에서 실패했다. 남은 185개 manifest 행을
모두 실제 PDF 생성·페이지 트리 파싱으로 확인하고 정확히 185개가 실행되도록 개선했다.
임계값을 낮추거나 제거 문서를 다시 등록하지 않았다.


### 보정 후보 집중 검증과 로컬 커밋 경계

보정 후 일반 MCP 4개·#8160 HWP5 MCP 2개·#4764 합성 PDF builder 1개, 총 7개를 실제로
집중 실행해 7/7 PASS했다. PDF builder는 남은 185개 입력을 모두 생성·파싱했다.
포맷·Native/WASM/workspace Clippy·workspace build·suite/unit 정책도 다시 PASS했다.
이 결과와 복원·보정 코드를 먼저 로컬 일반 커밋으로 고정했다. 코드 커밋 당시에는 전체 nextest와
Native Skia가 진행 중이었으며, 아래 최종 결과에서 모두 PASS로 완료했다. 원격 PR은 갱신하지 않았다.

빌드 잡 2개에서는 자동 재배정된 suite의 재컴파일이 길어져, 16 CPU·가용 메모리 13GiB와
rustc RSS 1GiB 미만을 확인한 뒤 완료 캐시를 보존하고 빌드 잡을 8개로 바꿨다.
집중 검사 빌드 중의 의도적 중단이며 assertion 실패가 아니다. 소스와 테스트 스레드 12개는 유지했다.
재개한 집중 검사 빌드는 2분 39초에 완료했다.


### 코드 커밋 고정과 보정 후 전체 결과

복원·통합·편람 등록 제거·두 검증 보정을
`404c0de4be62f14542497ddf94f7f2b6af4eaa44`에 로컬 커밋했다. 원격 push는 하지 않았다.
보정 후 전체 nextest는 11,516개 전부 PASS, 기본 제외 45개로 완료했다.
실제 #8160 이름 집합 85개도 기대 집합과 일치한다.

선행 PR 반영 후에는 이 코드 커밋을 기준으로 #8160의 40개 모듈·86개 함수뿐 아니라
변경한 기존 파일에 남긴 일반 `#[test]` 함수 261개 인벤토리도 대조한다.
상위 PR의 의도적 정정·이름 변경은 독립 근거로 분류하고, 자동 적용 과정에서 일반 대조 문서의
보호까지 사라지지 않도록 확인한다. 인벤토리·로그·중간 JSON은 ignored 경로에만 보존한다.


### 최종 검증 완료와 제출 순서

| 검증 | 결과 | 실제 범위 |
| --- | --- | --- |
| fmt | PASS | workspace 전체 |
| Clippy | PASS | Native·WASM lib·workspace all-targets 각 `-D warnings` |
| workspace build | PASS | locked |
| suite/unit 정책 | PASS | base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a` |
| 보정 집중 nextest | 7/7 PASS | 일반 MCP 4개·#8160 MCP 2개·185행 PDF builder 1개 |
| 전체 nextest | 11,516/11,516 PASS | 기본 제외 45개, 1,587.417초 |
| #8160 기본 실행 집합 | 85/85 PASS | 누락·예상 외 0개 |
| Native Skia 집중 | 3/3 PASS | 실제 PNG 이중선 coverage 포함, 함수 실행 51.49초 |

최종 실행 로그와 시작·종료 source SHA-256은 `parallel-final-gates.json`,
`parallel-final-*.log`에 보존한다. 검증 checkout은
`/home/tsjang/rhwp-8160-wasm-candidate6`이며, 코드 커밋의 소스·fixture 5022개를
다시 바이트 대조해 차이 0개를 확인했다. 임시 로그·생성 suite·인벤토리 JSON은 커밋하지 않는다.

원격 `upstream/devel`은 재확인 시 `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`,
PR #8188의 원격 head는 `021c486d1eaee17920a2b3e82ae896115b847669`다.
로컬 복원·보정과 검증은 완료했으며, 사용자 지시에 따라 ubuntu-ted PR을 먼저 처리한다.
그 반영 후 최신 devel에서 공용 제외 목록·helper를 통합하고, 선행 기능과 현재의 정상 대조 검사,
sample16 세 문서 각각 64쪽의 Native/fresh WASM raw100을 다시 검증한 뒤 기존 PR에 후속 제출한다.
그 전까지 원격 push·PR 갱신·머지·이슈 종료는 하지 않는다. CI 이후 후속 처리의 승인 대기도 유지한다.
