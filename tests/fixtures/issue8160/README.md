# #8160 sample16 전 페이지 Print 회귀 계약

## 독립 기대값

`hwpx.json`, `hwp2024.json`, `hwp2022.json`은 각 원본 64쪽과 새 MCP 독립 Print의
SHA-256, 실제 96 DPI 기준선, 원문 내용과 원본 주소를 보존한다. PDF 출력 경로·실제
한컴 engine·job ID는 [MCP 증거](../../../mydocs/pr/assets/issue_8160/mcp-reference-refresh.json),
회귀 추가 전의 384쪽 raw 100%·정수 미대응 0은
[최종 전쪽 증거](../../../mydocs/pr/assets/issue_8160/candidate33-all-backends.json)를 따른다.

PDF의 `get_text("dict")` 줄에서 공백·제어 문자만 제외한 문자열을 읽고, 첫 span의
`origin[1] * 96 / 72`를 기준선으로 측정했다. 같은 쪽에서 원본 내용과 단일하게 대응하는
문구의 앞·중간·뒤를 선정했다. 가시 첫 런의 앞에 공백이 있는 줄은 글자 원점과 전진
프레임을 혼동하지 않도록 선정 대상에서 제외했다. 세 문서 각각 64쪽 모두에 중요 내용이 있으며
합계 572곳이다. 기대 좌표는 현재 렌더 트리의 bbox에서 복사하지 않았다.

문단·셀·글상자 주소와 문자 시작은 고정한 원본의 주소다. 정식 Rust 검사는 매번 원본을
다시 파싱하고 주소가 가리키는 실제 문단의 문자열과 각 런의 문자 범위를 대조한다.
같은 문구라도 다른 셀·본문·문단으로 이동하면 실패한다. 독립 기준선 ±2px, 해당 쪽의
내용·중복·소유, 원문 순서와 전체 쪽수도 함께 검사한다.
`issue_8160_all_pages.rs::verify_table_picture_rows`는 이전의 HWPX 18쪽 검사 보호를 세 문서로
확장해 표·그림·표의 원본 컨트롤과 셀 주소·비겹침을 확인한다. 중복 활성 case는 제거했다.

## 실제 전 페이지 시각 회귀

내용 관계 검사와 별도로 `scripts/verify_issue_8160_raw100.py`가 실제 Native/Canvas2D PNG
384개를 독립 Print PNG와 다시 비교한다. TSV의 반올림 값이나 이전 통과 판정을 쓰지 않는다.
threshold 232·흰색 padding·양방향 2px의 정수 미대응이 단 하나라도 있으면 exit 1이다.
쪽 누락·추가·잘못된 입력/PDF 해시·다른 profile/backend도 거부한다. 전체 가로·세로 paint는
이 비교가 보호하며 layout 전진 상자를 PDF 잉크 외곽선으로 잘못 고정하지 않는다.

기존 Visual Sweep을 같은 원본·독립 PDF·실제 원본 글꼴 프로그램으로 전쪽 실행한 뒤,
`runs.json`에 여섯 실행 폴더를 지정한다. 각 폴더는 `silhouette_manifest.json`,
`run_manifest.json`, `rhwp_png/`, `pdf_png/`를 포함한다. 형식은 다음과 같다.

```json
{"native":{"hwpx":"...","hwp2024":"...","hwp2022":"..."},
 "wasm":{"hwpx":"...","hwp2024":"...","hwp2022":"..."}}
```

```bash
venv/bin/python scripts/verify_issue_8160_raw100.py \
  --runs-json output/pr-review/<작업>/runs.json \
  --report output/pr-review/<작업>/raw100.json --workers 8
```

같은 검사로 수정 전 candidate31 WASM의 2022 18·35쪽을 실제 검출했다(382/384, exit 1).
candidate33 Native/표준 WASM은 384/384, exit 0이다. 실제 실패 페이지·정수 차이는
[회귀 검출 증거](../../../mydocs/pr/assets/issue_8160/all-page-regression-proof.json)에 있다.
음성 실행의 Native는 동일 대조군으로 유지했으며 이를 같은 source의 수정 전 전체 실행으로
표기하지 않는다. 라이선스가 있는 설치 글꼴은 fixture에 복제하지 않는다.


## 리베이스 후 Print 환경과 JPEG 경계

제품 source `5df67d3299befb467a49dc4eaafd12d6e1877e2d`에서 세 파일 각각 Native와
fresh 표준 최적화 WASM 64/64쪽 raw100·실제 양방향 정수 미대응 0/0을 재확인했다.
[단계 8](../../../mydocs/working/task_m100_8160_stage8.md)의 재측정·직접 판독 기록을 따른다.
좌표 검사는 이 독립 Print와 전쪽 PNG의 프로그램 선택 환경을 사용한다. 기존 572개
앵커·630개 영역의 기대값과 ±2px 공차는 유지한다. 글꼴 바이트를 fixture에 복제하지 않는다.

`hwp2022-header-logo.png`는 같은 새 MCP PDF의 1쪽 96 DPI raster에서
`[52, 64, 185, 102]`를 잘라 얻은 133×38px 독립 기대값이다. 원본/PDF 해시는
`hwp2022.json`의 기존 필드를, 영역·PNG 해시·원래 전쪽 PNG 해시·임계값 232·대칭 2px는
같은 파일의 `browser_raster_regions`를 따른다. 기존 64쪽의 좌표 행은 바꾸지 않았다.

[`issue_8160_print_jpeg_frame.browser.mjs`](../../../scripts/tests/issue_8160_print_jpeg_frame.browser.mjs)는 실제
WASM과 Canvas로 같은 표지를 그리고 원본 그림 두 개·JPEG 인쇄 프레임·64쪽·decode 성공을
확인한 뒤 이 독립 영역과 양방향 정수 미대응 0/0을 검사한다. 설치 글꼴은 필요하지 않다.
기존 `visual-sweep-canvas-fonts.browser.mjs`의 import로 수집하며 CI 개별 호출을 늘리지 않는다.
임시 추출·진단 도구는 정식 검사에 포함하지 않는다.


최신 리베이스 후에도 같은 독립 기대값으로 전쪽을 재검증했다.
[현재 6개 전쪽 TSV·실제 PNG 정수 0/0·정식 검사 결과](../../../mydocs/pr/assets/issue_8160/final-9418/README.md)를
따르며 위 candidate33은 이전 검증 이력이다. 현재 제품과 정식 테스트의 source를 구분해 보존한다.

## 현재 활성 검사 이름 (#8160 통합)

기존 sample16 전용 검사와 공용 corpus의 해당 입력 등록을 제거하고 아래 `issue_8160_*` 모듈로 모았다.
Rust 86개 함수(기본 78개, Native Skia 전용 1개)와 실제 WASM JPEG 브라우저 검사 1개다.
572개 앵커·630개 영역·±2px·좌표 JSON 3개와 독립 JPEG PNG는 변경하지 않았다.

단순 2022/2024 쪽수 검사 2개는 전쪽 내용·소유 검사로 통합했다. 기대값 없는 진단
11개 파일·13개 함수는 제거했고, 혼합 진단 2개는 K-water 항목을 유지했다.
옛 16쪽 sample16 catalog 6개와 집계·baseline의 해당 행은 제거했으며 나머지 입력과 검사는 유지한다.
메타데이터 검사의 다른 정상 문서 행, 일반 CLI 봉투와 사용자 흐름 예시도 보존했다.

| 검사 파일 | 함수 수 |
| --- | ---: |
| [issue_8160_agreement_auto_structure](../../cases/issue_8160_agreement_auto_structure.rs) | 1 |
| [issue_8160_agreement_clause_confidence](../../cases/issue_8160_agreement_clause_confidence.rs) | 1 |
| [issue_8160_agreement_clause_context](../../cases/issue_8160_agreement_clause_context.rs) | 1 |
| [issue_8160_all_pages](../../cases/issue_8160_all_pages.rs) | 3 |
| [issue_8160_bookmark_roundtrip](../../cases/issue_8160_bookmark_roundtrip.rs) | 2 |
| [issue_8160_digest_structure_addresses](../../cases/issue_8160_digest_structure_addresses.rs) | 2 |
| [issue_8160_encrypted_explain_contract](../../cases/issue_8160_encrypted_explain_contract.rs) | 3 |
| [issue_8160_encrypted_record_dump_contract](../../cases/issue_8160_encrypted_record_dump_contract.rs) | 1 |
| [issue_8160_equation_reading_order](../../cases/issue_8160_equation_reading_order.rs) | 2 |
| [issue_8160_font_program_ownership](../../cases/issue_8160_font_program_ownership.rs) | 3 |
| [issue_8160_footer_border_ownership](../../cases/issue_8160_footer_border_ownership.rs) | 1 |
| [issue_8160_fullwidth_cjk_quote](../../cases/issue_8160_fullwidth_cjk_quote.rs) | 1 |
| [issue_8160_gradient_and_font_mapping](../../cases/issue_8160_gradient_and_font_mapping.rs) | 3 |
| [issue_8160_hft_print_frame](../../cases/issue_8160_hft_print_frame.rs) | 1 |
| [issue_8160_hwp2022_paragraph_relations](../../cases/issue_8160_hwp2022_paragraph_relations.rs) | 3 |
| [issue_8160_hwp2024_print_boundaries](../../cases/issue_8160_hwp2024_print_boundaries.rs) | 5 |
| [issue_8160_hwp3_page_roundtrip](../../cases/issue_8160_hwp3_page_roundtrip.rs) | 2 |
| [issue_8160_hwp3_parser_alignment](../../cases/issue_8160_hwp3_parser_alignment.rs) | 1 |
| [issue_8160_hwp3_print_boundaries](../../cases/issue_8160_hwp3_print_boundaries.rs) | 5 |
| [issue_8160_hwp3_source_frame_ownership](../../cases/issue_8160_hwp3_source_frame_ownership.rs) | 3 |
| [issue_8160_hwp3_storage_contract](../../cases/issue_8160_hwp3_storage_contract.rs) | 2 |
| [issue_8160_hwp5_password](../../cases/issue_8160_hwp5_password.rs) | 2 |
| [issue_8160_hwpx_stored_page_ownership](../../cases/issue_8160_hwpx_stored_page_ownership.rs) | 1 |
| [issue_8160_legacy_2018_page_count](../../cases/issue_8160_legacy_2018_page_count.rs) | 1 |
| [issue_8160_legacy_hwp5_page_count](../../cases/issue_8160_legacy_hwp5_page_count.rs) | 1 |
| [issue_8160_nondefault_source_style](../../cases/issue_8160_nondefault_source_style.rs) | 1 |
| [issue_8160_page_border_center_relations](../../cases/issue_8160_page_border_center_relations.rs) | 2 |
| [issue_8160_save_version_metadata](../../cases/issue_8160_save_version_metadata.rs) | 2 |
| [issue_8160_saved_character_shading](../../cases/issue_8160_saved_character_shading.rs) | 1 |
| [issue_8160_saved_page_boundaries](../../cases/issue_8160_saved_page_boundaries.rs) | 9 |
| [issue_8160_source_style_pool](../../cases/issue_8160_source_style_pool.rs) | 1 |
| [issue_8160_stored_column_margin_ownership](../../cases/issue_8160_stored_column_margin_ownership.rs) | 2 |
| [issue_8160_toc_and_latin_font](../../cases/issue_8160_toc_and_latin_font.rs) | 6 |
| [issue_8160_unmodelled_source_geometry](../../cases/issue_8160_unmodelled_source_geometry.rs) | 1 |
| [issue_8160_verify_axes](../../cases/issue_8160_verify_axes.rs) | 1 |
| [issue_8160_wmf_mask_ownership](../../cases/issue_8160_wmf_mask_ownership.rs) | 1 |
| [issue_8160_zero_gutter_source_width](../../cases/issue_8160_zero_gutter_source_width.rs) | 1 |

JPEG: [issue_8160_print_jpeg_frame.browser.mjs](../../../scripts/tests/issue_8160_print_jpeg_frame.browser.mjs).
전쪽 실제 PNG: [verify_issue_8160_raw100.py](../../../scripts/verify_issue_8160_raw100.py).
이전→새 검사 대응과 실제 검증은 [9단계](../../../mydocs/working/task_m100_8160_stage9.md)를 따른다.


### 선행 PR에서 빠진 보호의 복원

- [도형 호스트·후행 표](../../cases/issue_8160_stored_shape_host_physical_frame.rs): 원본 HWP/HWPX의 소유·전체 내용·독립 Print 기준선과 표 경계 4개.
- [HWP5 MCP 암호 전달](../../cases/issue_8160_mcp_password.rs): 세션 및 무상태 도구 2개.
- [HWP5 armor 봉투](../../cases/issue_8160_armor_password.rs): stdin 암호와 전체 응답 봉투 1개.

복원 원문·SHA와 선행 PR 순서는 [9단계](../../../mydocs/working/task_m100_8160_stage9.md)에 기록한다.
