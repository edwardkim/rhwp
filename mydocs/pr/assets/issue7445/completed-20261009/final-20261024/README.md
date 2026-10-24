# 최종 전쪽 검증 증적 (2026-10-24)

제품 실행 HEAD: `1f54c3df17747b71c8be9b47b3198add4fa4fd3b`. 고정 base: `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`.

기존 50조건을 각 backend 1,836쪽씩 비교했다. 후보 Native/fresh WASM 총 3,672쪽은 누락·중복 없이
독립 Print 쪽수와 일치하며, raw 2px 실루엣 90% 미달과 같은 수치 쪽의 devel 대비 2%p 이상 하락이 없다.
모든 쪽이 RGB·글꼴 윤곽·절대 좌표까지 같다는 뜻은 아니다. [실행 검증 원장](validation.json)은 제품부 전체 회귀·필수 게이트와 진단 전용 lint 보정 뒤의
실제 결과를 구분한다. 새 commit의 소스 패키징 preflight와 최신 base/open PR 확인 뒤 제출하며,
GitHub CI·머지·후속 처리가 끝났다고 선언하는 묶음은 아니다.

## 검증 조건과 출처

- 실제 PDFium 152.0.7947.0 / pypdfium2 5.12.1 / smooth RGB v1, 96DPI, raw 2px·90% 기준이다.
- Native는 native-skia CLI PNG, fresh WASM은 현재 로컬 locked wrapper의 `--no-opt` 빌드를 사용했다. Docker 최적화 통과가 아니다.
- 후보 100조건은 모두 새 출력이다. base 100 TSV는 같은 고정 base·도구·글꼴·바이너리 hash를 검증한 stage83 결과를 재사용했다.
- [입력 감사](inputs.json)는 커밋한 HWP/HWPX/PDF와 실제 실행 바이트 SHA가 같음을 확인한다.
- [생산자](whole-producer.json), [정규화한 범위](scope50.json), [전쪽 원장](whole-pages.json), [정량 요약](summary.json)을 함께 읽는다.
- 과거 범위 파일의 origin metadata를 실제 실행 base/head로 오인하지 않도록 정규화한 범위를 따로 보존했다.
- base에서 쪽수가 다른 조건은 같은 수치 쪽의 점수만으로 내용 대응을 주장하지 않는다. 누락·추가 쪽 목록을 보존하고 후보의 독립 쪽수·내용 관계 회귀·직접 판독을 함께 확인했다.

## 조건별 최저 raw 일치율

| 조건 | 독립 쪽수 | Native 최저 | fresh WASM 최저 | 입력·PDF |
| --- | ---: | ---: | ---: | --- |
| sample16-2022 | 64 | [96.85410%](tsv/head/native/sample16-2022.tsv) | [96.84889%](tsv/head/wasm/sample16-2022.tsv) | [입력](<../../../../../../samples/hwp3-sample16-hwp5-2022.hwp>) · [PDF](<../../../../../../pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf>) |
| pii66-declared | 66 | [96.57165%](tsv/head/native/pii66-declared.tsv) | [96.56850%](tsv/head/wasm/pii66-declared.tsv) | [입력](<../../../../../../samples/task2097/75544_pii_bunseok.hwpx>) · [PDF](<../../../../../../pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf>) |
| sample16-hwpx | 64 | [96.75084%](tsv/head/native/sample16-hwpx.tsv) | [96.75000%](tsv/head/wasm/sample16-hwpx.tsv) | [입력](<../../../../../../samples/hwp3-sample16-hwp5.hwpx>) · [PDF](<../../../../../../pdf/hwp3-sample16-hwp5-lossless-2024.pdf>) |
| worklife6 | 6 | [98.95060%](tsv/head/native/worklife6.tsv) | [98.93640%](tsv/head/wasm/worklife6.tsv) | [입력](<../../../../../../samples/issue6145/worklife_balance_index_156607916.hwpx>) · [PDF](<../../../../../../pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf>) |
| asset60 | 60 | [96.08347%](tsv/head/native/asset60.tsv) | [96.08103%](tsv/head/wasm/asset60.tsv) | [입력](<../../../../../../samples/issue6031/3249937_asset_management_rules.hwpx>) · [PDF](<../../../../../../pdf/3249937_asset_management_rules-2020.pdf>) |
| rowbreak18 | 18 | [93.06876%](tsv/head/native/rowbreak18.tsv) | [92.99291%](tsv/head/wasm/rowbreak18.tsv) | [입력](<../../../../../../samples/rowbreak-problem-pages.hwp>) · [PDF](<../../../../../../pdf/rowbreak-problem-pages-hwp-2024.pdf>) |
| biotoxin205 | 205 | [98.42761%](tsv/head/native/biotoxin205.tsv) | [98.42748%](tsv/head/wasm/biotoxin205.tsv) | [입력](<../../../../../../samples/issue5941/1480000-201900698-native-neartop-reset.hwp>) · [PDF](<../../../../../../pdf/issue5941-1480000-201900698-2020.pdf>) |
| regulatory17 | 17 | [99.26826%](tsv/head/native/regulatory17.tsv) | [99.14280%](tsv/head/wasm/regulatory17.tsv) | [입력](<../../../../../../samples/issue7445/80250_regulatory_analysis.hwp>) · [PDF](<../../../../../../pdf/80250_regulatory_analysis-hwp-2024.pdf>) |
| ktx003 | 1 | [99.36494%](tsv/head/native/ktx003.tsv) | [99.38975%](tsv/head/wasm/ktx003.tsv) | [입력](<../../../../../../samples/basic/KTX-003.hwp>) · [PDF](<../../../../../../pdf/basic/KTX-003-hwp-2020.pdf>) |
| low-slack6535 | 1 | [99.99852%](tsv/head/native/low-slack6535.tsv) | [99.99851%](tsv/head/wasm/low-slack6535.tsv) | [입력](<../../../../../../samples/issue6535/36339092_low_slack_absorb_block.hwpx>) · [PDF](<../../../../../../pdf/36339092_low_slack_absorb_block-2020.pdf>) |
| press12 | 12 | [94.23037%](tsv/head/native/press12.tsv) | [94.18582%](tsv/head/wasm/press12.tsv) | [입력](<../../../../../../samples/issue3637/press_release_split_cell_nested_table.hwpx>) · [PDF](<../../../../../../pdf/issue3637/press_release_split_cell_nested_table-hwpx-2020.pdf>) |
| project2097-declared | 2 | [98.75834%](tsv/head/native/project2097-declared.tsv) | [98.79962%](tsv/head/wasm/project2097-declared.tsv) | [입력](<../../../../../../samples/task2097/1741000_project_application.hwp>) · [PDF](<../../../../../../pdf/task2097/1741000_project_application-2020.pdf>) |
| issue265 | 16 | [99.50695%](tsv/head/native/issue265.tsv) | [99.50826%](tsv/head/wasm/issue265.tsv) | [입력](<../../../../../../samples/issue_265.hwp>) · [PDF](<../../../../../../pdf/issue_265-hwp-2020.pdf>) |
| hwp3-alias | 16 | [99.50695%](tsv/head/native/hwp3-alias.tsv) | [99.50826%](tsv/head/wasm/hwp3-alias.tsv) | [입력](<../../../../../../samples/hwp3-sample.hwp>) · [PDF](<../../../../../../pdf/issue_265-hwp-2020.pdf>) |
| converted16 | 16 | [98.84446%](tsv/head/native/converted16.tsv) | [99.50438%](tsv/head/wasm/converted16.tsv) | [입력](<../../../../../../samples/hwp3-sample-hwp5.hwp>) · [PDF](<../../../../../../pdf/hwp3-sample-hwp5-hwp-2020.pdf>) |
| tac3 | 3 | [99.26036%](tsv/head/native/tac3.tsv) | [99.26027%](tsv/head/wasm/tac3.tsv) | [입력](<../../../../../../samples/issue6448/tac_cell_leftover_fits.hwpx>) · [PDF](<../../../../../../pdf/tac_cell_leftover_fits-2020.pdf>) |
| civil2 | 2 | [100.00000%](tsv/head/native/civil2.tsv) | [100.00000%](tsv/head/wasm/civil2.tsv) | [입력](<../../../../../../samples/issue6032/2912695_civil_petition_form.hwp>) · [PDF](<../../../../../../pdf/2912695_civil_petition_form-2020.pdf>) |
| ole-hwp | 1 | [100.00000%](tsv/head/native/ole-hwp.tsv) | [100.00000%](tsv/head/wasm/ole-hwp.tsv) | [입력](<../../../../../../samples/한셀OLE.hwp>) · [PDF](<../../../../../../pdf/한셀OLE-hwp-2020.pdf>) |
| ole-hwpx | 1 | [100.00000%](tsv/head/native/ole-hwpx.tsv) | [100.00000%](tsv/head/wasm/ole-hwpx.tsv) | [입력](<../../../../../../samples/한셀OLE.hwpx>) · [PDF](<../../../../../../pdf/한셀OLE-hwpx-2024.pdf>) |
| rnote-cp01 | 3 | [100.00000%](tsv/head/native/rnote-cp01.tsv) | [100.00000%](tsv/head/wasm/rnote-cp01.tsv) | [입력](<../../../../../../samples/rnote/labnote-001-cp-01.hwp>) · [PDF](<../../../../../../pdf/rnote/labnote-001-cp-01-hwp-2020.pdf>) |
| rnote-cp02 | 4 | [100.00000%](tsv/head/native/rnote-cp02.tsv) | [100.00000%](tsv/head/wasm/rnote-cp02.tsv) | [입력](<../../../../../../samples/rnote/labnote-001-cp-02.hwp>) · [PDF](<../../../../../../pdf/rnote/labnote-001-cp-02-hwp-2020.pdf>) |
| table32 | 1 | [100.00000%](tsv/head/native/table32.tsv) | [100.00000%](tsv/head/wasm/table32.tsv) | [입력](<../../../../../../tests/fixtures/issue7486_table_enter/table_enter_32.hwpx>) · [PDF](<../../../../../../pdf/issue7486_table_enter_32-2020.pdf>) |
| table33 | 2 | [100.00000%](tsv/head/native/table33.tsv) | [100.00000%](tsv/head/wasm/table33.tsv) | [입력](<../../../../../../tests/fixtures/issue7486_table_enter/table_enter_33.hwpx>) · [PDF](<../../../../../../pdf/issue7486_table_enter_33-2020.pdf>) |
| table40 | 2 | [100.00000%](tsv/head/native/table40.tsv) | [100.00000%](tsv/head/wasm/table40.tsv) | [입력](<../../../../../../tests/fixtures/issue7486_table_enter/table_enter_40.hwpx>) · [PDF](<../../../../../../pdf/issue7486_table_enter_40-2020.pdf>) |
| table30 | 2 | [100.00000%](tsv/head/native/table30.tsv) | [100.00000%](tsv/head/wasm/table30.tsv) | [입력](<../../../../../../tests/fixtures/issue7486_table_enter/table30_spacing300_enter9.hwpx>) · [PDF](<../../../../../../pdf/issue7486_table30_spacing300_enter9-2020.pdf>) |
| ole-enter-hwp | 1 | [100.00000%](tsv/head/native/ole-enter-hwp.tsv) | [100.00000%](tsv/head/wasm/ole-enter-hwp.tsv) | [입력](<../ole-enter-hwp.hwp>) · [PDF](<../../../../../../pdf/issue2069/ole-enter-hwp-2020.pdf>) |
| ole-enter-hwpx | 1 | [100.00000%](tsv/head/native/ole-enter-hwpx.tsv) | [100.00000%](tsv/head/wasm/ole-enter-hwpx.tsv) | [입력](<../ole-enter-hwpx.hwpx>) · [PDF](<../../../../../../pdf/issue2069/ole-enter-hwpx-2024.pdf>) |
| roadmap302 | 302 | [94.97284%](tsv/head/native/roadmap302.tsv) | [94.80964%](tsv/head/wasm/roadmap302.tsv) | [입력](<../../../../../../samples/issue5941/1490000-201600081_roadmap_research.hwp>) · [PDF](<../../../../../../pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf>) |
| disaster | 2 | [98.08038%](tsv/head/native/disaster.tsv) | [98.07890%](tsv/head/wasm/disaster.tsv) | [입력](<../../../../../../samples/task2097/21298295_byeolpyo5_disaster.hwp>) · [PDF](<../../../../../../pdf/task2097/21298295_byeolpyo5_disaster-hwp-2020.pdf>) |
| empty-cell | 1 | [98.71050%](tsv/head/native/empty-cell.tsv) | [98.62947%](tsv/head/wasm/empty-cell.tsv) | [입력](<../../../../../../samples/hwp3-empty-cell.hwp>) · [PDF](<../../../../../../pdf/hwp3-empty-cell-2020.pdf>) |
| land-table | 5 | [99.57180%](tsv/head/native/land-table.tsv) | [99.56600%](tsv/head/wasm/land-table.tsv) | [입력](<../../../../../../samples/issue6803/1376496-neighborhood-facility-land-table.hwp>) · [PDF](<../../../../../../pdf/1376496-neighborhood-facility-land-table-2020.pdf>) |
| ledger | 1 | [98.85800%](tsv/head/native/ledger.tsv) | [98.77877%](tsv/head/wasm/ledger.tsv) | [입력](<../../../../../../samples/issue4680/20117321_gijang_handover_bond_ledger_form.hwp>) · [PDF](<../../../../../../pdf/issue4680/20117321_gijang_handover_bond_ledger_form-2020.pdf>) |
| lighting | 2 | [99.33214%](tsv/head/native/lighting.tsv) | [99.21787%](tsv/head/wasm/lighting.tsv) | [입력](<../../../../../../samples/issue6860/3067979-road-lighting-photometric-hancom2020.hwpx>) · [PDF](<../../../../../../pdf/3067979-road-lighting-photometric-hancom2020.pdf>) |
| none-image-fill | 14 | [98.94759%](tsv/head/native/none-image-fill.tsv) | [98.94752%](tsv/head/wasm/none-image-fill.tsv) | [입력](<../../../../../../samples/issue7235/156086935_none_image_fill.hwp>) · [PDF](<../../../../../../pdf/issue7235/156086935_none_image_fill-2020.pdf>) |
| yeongwol4 | 4 | [99.57184%](tsv/head/native/yeongwol4.tsv) | [99.57157%](tsv/head/wasm/yeongwol4.tsv) | [입력](<../../../../../../samples/issue5699/20099369_yeongwol_forms.hwp>) · [PDF](<../../../../../../pdf/20099369_yeongwol_forms-2020.pdf>) |
| court2 | 2 | [96.97353%](tsv/head/native/court2.tsv) | [96.85628%](tsv/head/wasm/court2.tsv) | [입력](<../../../../../../samples/issue6034/2912735_court_report_form.hwp>) · [PDF](<../../../../../../pdf/2912735_court_report_form-2020.pdf>) |
| superscript3 | 3 | [97.84742%](tsv/head/native/superscript3.tsv) | [97.79599%](tsv/head/wasm/superscript3.tsv) | [입력](<../../../../../../samples/issue5756/156732409_superscript_advance.hwp>) · [PDF](<../../../../../../pdf/156732409_superscript_advance-2020.pdf>) |
| field-marker1 | 1 | [100.00000%](tsv/head/native/field-marker1.tsv) | [100.00000%](tsv/head/wasm/field-marker1.tsv) | [입력](<../../../../../../samples/issue7344/36494836_field_marker_tac_lines.hwpx>) · [PDF](<../../../../../../pdf/issue7344/36494836-2020.pdf>) |
| broken-ladder1 | 1 | [100.00000%](tsv/head/native/broken-ladder1.tsv) | [100.00000%](tsv/head/wasm/broken-ladder1.tsv) | [입력](<../../../../../../samples/issue7344/36312980_broken_cell_ladder_center.hwpx>) · [PDF](<../../../../../../pdf/issue7344/36312980-2020.pdf>) |
| missing-lineseg1 | 1 | [98.02990%](tsv/head/native/missing-lineseg1.tsv) | [98.02479%](tsv/head/wasm/missing-lineseg1.tsv) | [입력](<../../../../../../samples/issue7344/36455985_no_lineseg_cell_spacing_before.hwpx>) · [PDF](<../../../../../../pdf/issue7344/36455985-2020.pdf>) |
| quote1 | 1 | [98.58029%](tsv/head/native/quote1.tsv) | [97.79782%](tsv/head/wasm/quote1.tsv) | [입력](<../../../../../../samples/issue7235/156467175_press_release_header_logo_p1.hwp>) · [PDF](<../../../../../../pdf/issue7235/156467175_press_release_header_logo_p1-2020.pdf>) |
| giant4138-original | 115 | [97.01264%](tsv/head/native/giant4138-original.tsv) | [97.01236%](tsv/head/wasm/giant4138-original.tsv) | [입력](<../../../../../../samples/issue1949_giant_cell_nested_tables_perf.hwp>) · [PDF](<../../../../../../pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf>) |
| giant4138-tab-period | 197 | [94.69402%](tsv/head/native/giant4138-tab-period.tsv) | [94.69402%](tsv/head/wasm/giant4138-tab-period.tsv) | [입력](<../../../../../../tests/fixtures/issue4138/tab-period-split.hwp>) · [PDF](<../../../../../../pdf/issue4138-tab-period-split-20261015-2024.pdf>) |
| giant4138-current-ascii | 197 | [94.69402%](tsv/head/native/giant4138-current-ascii.tsv) | [94.69402%](tsv/head/wasm/giant4138-current-ascii.tsv) | [입력](<../regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp>) · [PDF](<../../../../../../pdf/issue4138-ascii-split-20261016-2024.pdf>) |
| pii66-default | 66 | [96.57165%](tsv/head/native/pii66-default.tsv) | [96.56850%](tsv/head/wasm/pii66-default.tsv) | [입력](<../../../../../../samples/task2097/75544_pii_bunseok.hwpx>) · [PDF](<../../../../../../pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf>) |
| giant4138-hwpx115 | 115 | [97.01264%](tsv/head/native/giant4138-hwpx115.tsv) | [97.01236%](tsv/head/wasm/giant4138-hwpx115.tsv) | [입력](<../../../../../../samples/issue1949_giant_cell_nested_tables_perf.hwpx>) · [PDF](<../../../../../../pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf>) |
| original-sample16 | 64 | [97.01457%](tsv/head/native/original-sample16.tsv) | [97.00639%](tsv/head/wasm/original-sample16.tsv) | [입력](<../../../../../../samples/hwp3-sample16.hwp>) · [PDF](<../../../../../../pdf/hwp3-sample16-current-print-20261017-2020.pdf>) |
| original-sample11-user-print | 151 | [93.13188%](tsv/head/native/original-sample11-user-print.tsv) | [93.13713%](tsv/head/wasm/original-sample11-user-print.tsv) | [입력](<../../../../../../samples/hwp3-sample11.hwp>) · [PDF](<../../../../../../pdf/hwp3-sample11-ui-print-2020.pdf>) |
| new-upstream-field01 | 3 | [99.99969%](tsv/head/native/new-upstream-field01.tsv) | [99.99969%](tsv/head/wasm/new-upstream-field01.tsv) | [입력](<../../../../../../samples/field-01.hwp>) · [PDF](<../../../../../../pdf/field-01-hwp-2020.pdf>) |
| receipt7017 | 1 | [99.97136%](tsv/head/native/receipt7017.tsv) | [99.97136%](tsv/head/wasm/receipt7017.tsv) | [입력](<../../../../../../samples/issue7017/2528375-science-invention-contest-receipt.hwp>) · [PDF](<../../../../../../pdf/2528375-science-invention-contest-receipt-2020.pdf>) |

## 확대 판독과 실제 소비 경로

[대표 전쪽 review/overlay 판독](direct-review.json)은 38개 조건·쪽·backend 항목을 구분한다.
로드맵 마지막 상자, sample11 목차·연결 그림, 원본 sample16 수식·dot·표, #4138 원본 및 두
분할 저장본, 개인정보 마지막 조각, 독성물질 표와 그래프, 최저 rowbreak18 쪽을 확인했다.

- 최저 rowbreak18의16쪽은 Native 93.06876%, WASM 92.99291%다. 같은 base 쪽은 각각87.46657%,92.01257%다. 하단 표의 얇은 선과 본문에 미세 위치·획 차이가 남는다.
- sample11의50쪽은 사용자 UI Print에 존재하는 회색·갈색 면과 편집기 화면의 차이를 기존 독립 대조로 분리했다. 면을 임의로 추가하거나 비교를 mask하지 않았다.
- 원본 sample16의6쪽 수식 괄호·원 외곽선 및 글리프 AA의 미세 차이는 잔여이며 원 글꼴과 동일하다고 주장하지 않는다.
- #4138 두 분할 저장본은 각각197쪽 전쪽 비교에 더해1·10·31·197쪽의 줄 소속과 내용 순서·끝 경계를 직접 대조했다.

[실제 소비 원장](actual-consumers.json)은 Native31·Canvas2D31·CanvasKit12건의 독립 PDF 점수와
실제 초기화 응답·backend·surface·readiness·글꼴 프로그램 provenance를 구분한다. 기존 비트맵3개와
일반 보존 대조군을 추가 관측한 것이며50조건의 HWP 보정 범위를 확대하지 않았다.

글꼴 준비 부족으로 Canvas2D에 fallback한10관측은 CanvasKit 통과가 아니다. 잘못된 fixture URL을
입력 SHA guard가 거절한9관측도 최종 성공에 합산하지 않았다. 정확한 커밋 바이트를 제품의 기존
문서 열기 이벤트로 전달한 새 실행을 별도로 기록했다. 같은 크기와 전체 RGB hash가 같은 모드의
대표 판독 재사용은 각 관측의 backend/readiness 검증을 대신하지 않는다.

이 디렉터리에는 최종 TSV·대표 PNG·정리한 JSON만 보존한다. 원 로그·실행 script·전체 PNG·중간
파일은 ignored output에 보존하며 새 지침·문턱·예외를 추가하지 않았다.

[과거 TSV 보존 형식 영수증](historical-tsv-curation.json)은 제출 점검에서 CRLF를 LF로 정리한
과거36개 사본의 원 commit·원 SHA·현재 SHA와 셀 값/순서 동일성을 연결한다. 과거 metadata의
hash는 원 바이트의 hash이며 현재 정리한 사본의 hash와 구분한다. 이 묶음의 전쪽200 TSV와
실제 실행 output, 측정값·문턱은 변경하지 않았다.
