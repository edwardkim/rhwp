# Planet6897 통합 검토의 전쪽 Native·fresh WASM TSV

- 실제 runtime·전체 회귀 head: `3cb5a4c290663343123074be1e2fc0dda18d3db4`. 추가 완료 페이지 회귀·lint head: `e25dbd0f5d47bc32686016c08054d2bf718be6a7`. runtime 소스 2,744개의 바이트가 같습니다.
- 기준 devel: `a6e9d18fc1061318055b4eb348daec201427ea03`. 기준 WASM은 실제 `f45878c7caafb43ca1fd017ddd93484a0acc6ad2`에서 빌드했고, #8135는 Native 전용이므로 컴파일된 WASM 입력의 동일성 및 실제 기능 보존을 별도로 확인했습니다.
- 72개 측정 입력의 전 페이지(backend별 합계 1,529쪽)를 Native/fresh WASM에서 각각 산출했습니다. 90% 미만·누락·측정 불가·후보의 독립 PDF 쪽수 불일치는 없습니다.
- 색상 경계 보정 전 2px `silhouette_raw_match_percent`, print, 96dpi이며 compat와 설치 글꼴 경로는 입력별 원장에 있습니다. 파일을 글꼴 패키지로 복사하지 않았습니다.
- 이미 이관한 hwpx-sample2와 생성 순서가 다른 수동 XML 진단은 수용 목록에 포함하지 않았습니다. 원본·기준 PDF·보존 근거를 지우거나 통과로 바꾸지 않았습니다.
- 모든 TSV는 생성 원본을 바이트 그대로 복사했습니다. [입력·독립 Print PDF·SHA·환경](inputs.tsv), [TSV별 SHA-256·바이트 수·producer 원장](integrity.tsv)을 같은 key/condition으로 연결합니다.
- 67개 HWP/HWPX는 독립 한컴 Print와, #8120 PDF 가져오기 대조군 5개는 원본 PDF와 비교합니다. [입력·기준 PDF 다운로드 색인](documents/inputs_and_print_pdfs.md), [PDF별 실제 Creator·Producer·출처 종류](pdf_reference_provenance.tsv), [실제 producer 바이너리·WASM 해시와 바이트 수](producer_integrity.tsv), [현재 source의 직접 확대 판독](documents/current_visual_reading.md)을 함께 확인합니다.
- 이 문서는 전쪽 수치·원본 보존 원장입니다. 전체 Rust/lint/Skia·GitHub CI·직접 PNG 판독의 완료를 수치만으로 선언하지 않습니다.
- base의 쪽수 불일치 진단 16조건의 추가 대응 TSV는 [별도 SHA-256·바이트 원장](correspondence_integrity.tsv)으로 확인합니다. 전체 쪽 번호·존재 여부를 보존한 자료이며 같은 번호가 같은 내용을 뜻하거나 쪽수 gate를 통과했다는 증거가 아닙니다.
- [원 PR 27개의 source head·기능 tip·현재 통합 SHA](documents/source27_integration_map.md)를 실제 Git 출처와 대조했습니다. 최초 계획 한 개 뒤 원 기여 124개가 연속하며 메인터너 보정이 이어집니다.

## 입력별 결과와 원본

| 입력 | 경로 | PDF/출력 쪽수 | 최저 raw · 쪽 | 후보 TSV | 기준 TSV |
| --- | --- | --- | --- | --- | --- |
| night-guard | native | 1/1 | 97.82786% · 1 | [원본](tsv/current/native/night-guard.tsv) | [원본](tsv/base/native/night-guard.tsv) |
| night-guard | wasm | 1/1 | 97.52653% · 1 | [원본](tsv/current/wasm/night-guard.tsv) | [원본](tsv/base/wasm/night-guard.tsv) |
| merged-80168 | native | 157/157 | 93.90078% · 49 | [원본](tsv/current/native/merged-80168.tsv) | [원본](tsv/base/native/merged-80168.tsv) |
| merged-80168 | wasm | 157/157 | 91.66667% · 153 | [원본](tsv/current/wasm/merged-80168.tsv) | [원본](tsv/base/wasm/merged-80168.tsv) |
| curriculum-1342000 | native | 415/415 | 95.74615% · 93 | [원본](tsv/current/native/curriculum-1342000.tsv) | [원본](tsv/base/native/curriculum-1342000.tsv) · 기준은 쪽수 불일치 진단 |
| curriculum-1342000 | wasm | 415/415 | 95.76124% · 93 | [원본](tsv/current/wasm/curriculum-1342000.tsv) | [원본](tsv/base/wasm/curriculum-1342000.tsv) · 기준은 쪽수 불일치 진단 |
| 148738070 | native | 7/7 | 98.34102% · 1 | [원본](tsv/current/native/148738070.tsv) | [원본](tsv/base/native/148738070.tsv) · 기준은 쪽수 불일치 진단 |
| 148738070 | wasm | 7/7 | 98.38504% · 1 | [원본](tsv/current/wasm/148738070.tsv) | [원본](tsv/base/wasm/148738070.tsv) · 기준은 쪽수 불일치 진단 |
| 156465345 | native | 9/9 | 96.58283% · 9 | [원본](tsv/current/native/156465345.tsv) | [원본](tsv/base/native/156465345.tsv) · 기준은 쪽수 불일치 진단 |
| 156465345 | wasm | 9/9 | 96.56657% · 9 | [원본](tsv/current/wasm/156465345.tsv) | [원본](tsv/base/wasm/156465345.tsv) · 기준은 쪽수 불일치 진단 |
| 1150200 | native | 4/4 | 99.39534% · 3 | [원본](tsv/current/native/1150200.tsv) | [원본](tsv/base/native/1150200.tsv) · 기준은 쪽수 불일치 진단 |
| 1150200 | wasm | 4/4 | 99.36649% · 3 | [원본](tsv/current/wasm/1150200.tsv) | [원본](tsv/base/wasm/1150200.tsv) · 기준은 쪽수 불일치 진단 |
| merged-grid-fixture | native | 1/1 | 97.86164% · 1 | [원본](tsv/current/native/merged-grid-fixture.tsv) | [원본](tsv/base/native/merged-grid-fixture.tsv) |
| merged-grid-fixture | wasm | 1/1 | 97.82471% · 1 | [원본](tsv/current/wasm/merged-grid-fixture.tsv) | [원본](tsv/base/wasm/merged-grid-fixture.tsv) |
| nested-tac-fixture | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/nested-tac-fixture.tsv) | [원본](tsv/base/native/nested-tac-fixture.tsv) |
| nested-tac-fixture | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/nested-tac-fixture.tsv) | [원본](tsv/base/wasm/nested-tac-fixture.tsv) |
| eq-rm-space | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/eq-rm-space.tsv) | [원본](tsv/base/native/eq-rm-space.tsv) |
| eq-rm-space | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/eq-rm-space.tsv) | [원본](tsv/base/wasm/eq-rm-space.tsv) |
| eq-plusminus | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/eq-plusminus.tsv) | [원본](tsv/base/native/eq-plusminus.tsv) |
| eq-plusminus | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/eq-plusminus.tsv) | [원본](tsv/base/wasm/eq-plusminus.tsv) |
| eq-uppercase-greek | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/eq-uppercase-greek.tsv) | [원본](tsv/base/native/eq-uppercase-greek.tsv) |
| eq-uppercase-greek | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/eq-uppercase-greek.tsv) | [원본](tsv/base/wasm/eq-uppercase-greek.tsv) |
| original-nested-tac-row-drop | native | 1/1 | 99.56579% · 1 | [원본](tsv/current/native/original-nested-tac-row-drop.tsv) | [원본](tsv/base/native/original-nested-tac-row-drop.tsv) |
| original-nested-tac-row-drop | wasm | 1/1 | 99.51155% · 1 | [원본](tsv/current/wasm/original-nested-tac-row-drop.tsv) | [원본](tsv/base/wasm/original-nested-tac-row-drop.tsv) |
| original-eq-rm-quarterspace | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/original-eq-rm-quarterspace.tsv) | [원본](tsv/base/native/original-eq-rm-quarterspace.tsv) |
| original-eq-rm-quarterspace | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/original-eq-rm-quarterspace.tsv) | [원본](tsv/base/wasm/original-eq-rm-quarterspace.tsv) |
| added-issue7092-substitute-font-hft-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7092-substitute-font-hft-hwpx.tsv) | [원본](tsv/base/native/added-issue7092-substitute-font-hft-hwpx.tsv) |
| added-issue7092-substitute-font-hft-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7092-substitute-font-hft-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7092-substitute-font-hft-hwpx.tsv) |
| added-issue7092-substitute-font-none-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7092-substitute-font-none-hwpx.tsv) | [원본](tsv/base/native/added-issue7092-substitute-font-none-hwpx.tsv) |
| added-issue7092-substitute-font-none-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7092-substitute-font-none-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7092-substitute-font-none-hwpx.tsv) |
| added-issue7092-substitute-font-ttf-hwpx | native | 3/3 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7092-substitute-font-ttf-hwpx.tsv) | [원본](tsv/base/native/added-issue7092-substitute-font-ttf-hwpx.tsv) |
| added-issue7092-substitute-font-ttf-hwpx | wasm | 3/3 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7092-substitute-font-ttf-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7092-substitute-font-ttf-hwpx.tsv) |
| added-issue7679-tac-after-text-2024-hwp | native | 3/3 | 99.87326% · 3 | [원본](tsv/current/native/added-issue7679-tac-after-text-2024-hwp.tsv) | [원본](tsv/base/native/added-issue7679-tac-after-text-2024-hwp.tsv) |
| added-issue7679-tac-after-text-2024-hwp | wasm | 3/3 | 99.84974% · 3 | [원본](tsv/current/wasm/added-issue7679-tac-after-text-2024-hwp.tsv) | [원본](tsv/base/wasm/added-issue7679-tac-after-text-2024-hwp.tsv) |
| added-issue7687-empty-host-hwpx | native | 6/6 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7687-empty-host-hwpx.tsv) | [원본](tsv/base/native/added-issue7687-empty-host-hwpx.tsv) |
| added-issue7687-empty-host-hwpx | wasm | 6/6 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7687-empty-host-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7687-empty-host-hwpx.tsv) |
| added-issue7695-footnote-page-tail-2024-hwp | native | 32/32 | 99.77672% · 27 | [원본](tsv/current/native/added-issue7695-footnote-page-tail-2024-hwp.tsv) | [원본](tsv/base/native/added-issue7695-footnote-page-tail-2024-hwp.tsv) |
| added-issue7695-footnote-page-tail-2024-hwp | wasm | 32/32 | 99.84831% · 27 | [원본](tsv/current/wasm/added-issue7695-footnote-page-tail-2024-hwp.tsv) | [원본](tsv/base/wasm/added-issue7695-footnote-page-tail-2024-hwp.tsv) |
| added-issue7696-page-anchored-band-2024-hwp | native | 4/4 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7696-page-anchored-band-2024-hwp.tsv) | [원본](tsv/base/native/added-issue7696-page-anchored-band-2024-hwp.tsv) |
| added-issue7696-page-anchored-band-2024-hwp | wasm | 4/4 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7696-page-anchored-band-2024-hwp.tsv) | [원본](tsv/base/wasm/added-issue7696-page-anchored-band-2024-hwp.tsv) |
| added-issue7696-page-anchored-band-hwpx | native | 4/4 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7696-page-anchored-band-hwpx.tsv) | [원본](tsv/base/native/added-issue7696-page-anchored-band-hwpx.tsv) · 기준은 쪽수 불일치 진단 |
| added-issue7696-page-anchored-band-hwpx | wasm | 4/4 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7696-page-anchored-band-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7696-page-anchored-band-hwpx.tsv) · 기준은 쪽수 불일치 진단 |
| added-issue7870-flow_37-hwpx | native | 2/2 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7870-flow_37-hwpx.tsv) | [원본](tsv/base/native/added-issue7870-flow_37-hwpx.tsv) |
| added-issue7870-flow_37-hwpx | wasm | 2/2 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7870-flow_37-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7870-flow_37-hwpx.tsv) |
| added-issue7870-tac_37-hwpx | native | 2/2 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7870-tac_37-hwpx.tsv) | [원본](tsv/base/native/added-issue7870-tac_37-hwpx.tsv) |
| added-issue7870-tac_37-hwpx | wasm | 2/2 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7870-tac_37-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7870-tac_37-hwpx.tsv) |
| added-issue7972-inactive-cell-margin-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue7972-inactive-cell-margin-hwpx.tsv) | [원본](tsv/base/native/added-issue7972-inactive-cell-margin-hwpx.tsv) |
| added-issue7972-inactive-cell-margin-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue7972-inactive-cell-margin-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7972-inactive-cell-margin-hwpx.tsv) |
| added-issue7993-soft-hyphen-2024-hwp | native | 1/1 | 99.97226% · 1 | [원본](tsv/current/native/added-issue7993-soft-hyphen-2024-hwp.tsv) | [원본](tsv/base/native/added-issue7993-soft-hyphen-2024-hwp.tsv) |
| added-issue7993-soft-hyphen-2024-hwp | wasm | 1/1 | 99.97821% · 1 | [원본](tsv/current/wasm/added-issue7993-soft-hyphen-2024-hwp.tsv) | [원본](tsv/base/wasm/added-issue7993-soft-hyphen-2024-hwp.tsv) |
| added-issue7993-soft-hyphen-hwpx | native | 1/1 | 99.33618% · 1 | [원본](tsv/current/native/added-issue7993-soft-hyphen-hwpx.tsv) | [원본](tsv/base/native/added-issue7993-soft-hyphen-hwpx.tsv) · 기준은 쪽수 불일치 진단 |
| added-issue7993-soft-hyphen-hwpx | wasm | 1/1 | 99.33179% · 1 | [원본](tsv/current/wasm/added-issue7993-soft-hyphen-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7993-soft-hyphen-hwpx.tsv) · 기준은 쪽수 불일치 진단 |
| added-issue8067-flow_L1-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue8067-flow_L1-hwpx.tsv) | [원본](tsv/base/native/added-issue8067-flow_L1-hwpx.tsv) |
| added-issue8067-flow_L1-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue8067-flow_L1-hwpx.tsv) | [원본](tsv/base/wasm/added-issue8067-flow_L1-hwpx.tsv) |
| added-issue8067-flow_L1_tablepad-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue8067-flow_L1_tablepad-hwpx.tsv) | [원본](tsv/base/native/added-issue8067-flow_L1_tablepad-hwpx.tsv) |
| added-issue8067-flow_L1_tablepad-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue8067-flow_L1_tablepad-hwpx.tsv) | [원본](tsv/base/wasm/added-issue8067-flow_L1_tablepad-hwpx.tsv) |
| added-issue8067-flow_L3-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue8067-flow_L3-hwpx.tsv) | [원본](tsv/base/native/added-issue8067-flow_L3-hwpx.tsv) |
| added-issue8067-flow_L3-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue8067-flow_L3-hwpx.tsv) | [원본](tsv/base/wasm/added-issue8067-flow_L3-hwpx.tsv) |
| added-issue8067-flow_L3_tablepad-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue8067-flow_L3_tablepad-hwpx.tsv) | [원본](tsv/base/native/added-issue8067-flow_L3_tablepad-hwpx.tsv) |
| added-issue8067-flow_L3_tablepad-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue8067-flow_L3_tablepad-hwpx.tsv) | [원본](tsv/base/wasm/added-issue8067-flow_L3_tablepad-hwpx.tsv) |
| added-issue8067-tac_L1-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue8067-tac_L1-hwpx.tsv) | [원본](tsv/base/native/added-issue8067-tac_L1-hwpx.tsv) |
| added-issue8067-tac_L1-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue8067-tac_L1-hwpx.tsv) | [원본](tsv/base/wasm/added-issue8067-tac_L1-hwpx.tsv) |
| added-issue8067-tac_L3-hwpx | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/added-issue8067-tac_L3-hwpx.tsv) | [원본](tsv/base/native/added-issue8067-tac_L3-hwpx.tsv) |
| added-issue8067-tac_L3-hwpx | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/added-issue8067-tac_L3-hwpx.tsv) | [원본](tsv/base/wasm/added-issue8067-tac_L3-hwpx.tsv) |
| footnote8081-span36 | native | 2/2 | 99.86762% · 2 | [원본](tsv/current/native/footnote8081-span36.tsv) | [원본](tsv/base/native/footnote8081-span36.tsv) |
| footnote8081-span36 | wasm | 2/2 | 99.89205% · 2 | [원본](tsv/current/wasm/footnote8081-span36.tsv) | [원본](tsv/base/wasm/footnote8081-span36.tsv) |
| footnote8081-span38 | native | 2/2 | 99.87128% · 2 | [원본](tsv/current/native/footnote8081-span38.tsv) | [원본](tsv/base/native/footnote8081-span38.tsv) |
| footnote8081-span38 | wasm | 2/2 | 99.89504% · 2 | [원본](tsv/current/wasm/footnote8081-span38.tsv) | [원본](tsv/base/wasm/footnote8081-span38.tsv) |
| edit7866-a_whole_bold_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-a_whole_bold_after.tsv) | [원본](tsv/base/native/edit7866-a_whole_bold_after.tsv) |
| edit7866-a_whole_bold_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-a_whole_bold_after.tsv) | [원본](tsv/base/wasm/edit7866-a_whole_bold_after.tsv) |
| edit7866-a_whole_bold_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-a_whole_bold_before.tsv) | [원본](tsv/base/native/edit7866-a_whole_bold_before.tsv) |
| edit7866-a_whole_bold_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-a_whole_bold_before.tsv) | [원본](tsv/base/wasm/edit7866-a_whole_bold_before.tsv) |
| edit7866-b_first_char_bold_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-b_first_char_bold_after.tsv) | [원본](tsv/base/native/edit7866-b_first_char_bold_after.tsv) |
| edit7866-b_first_char_bold_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-b_first_char_bold_after.tsv) | [원본](tsv/base/wasm/edit7866-b_first_char_bold_after.tsv) |
| edit7866-b_first_char_bold_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-b_first_char_bold_before.tsv) | [원본](tsv/base/native/edit7866-b_first_char_bold_before.tsv) |
| edit7866-b_first_char_bold_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-b_first_char_bold_before.tsv) | [원본](tsv/base/wasm/edit7866-b_first_char_bold_before.tsv) |
| edit7866-c_second_char_bold_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-c_second_char_bold_after.tsv) | [원본](tsv/base/native/edit7866-c_second_char_bold_after.tsv) |
| edit7866-c_second_char_bold_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-c_second_char_bold_after.tsv) | [원본](tsv/base/wasm/edit7866-c_second_char_bold_after.tsv) |
| edit7866-c_second_char_bold_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-c_second_char_bold_before.tsv) | [원본](tsv/base/native/edit7866-c_second_char_bold_before.tsv) |
| edit7866-c_second_char_bold_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-c_second_char_bold_before.tsv) | [원본](tsv/base/wasm/edit7866-c_second_char_bold_before.tsv) |
| edit7866-d_red_longer_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-d_red_longer_after.tsv) | [원본](tsv/base/native/edit7866-d_red_longer_after.tsv) |
| edit7866-d_red_longer_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-d_red_longer_after.tsv) | [원본](tsv/base/wasm/edit7866-d_red_longer_after.tsv) |
| edit7866-d_red_longer_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-d_red_longer_before.tsv) | [원본](tsv/base/native/edit7866-d_red_longer_before.tsv) |
| edit7866-d_red_longer_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-d_red_longer_before.tsv) | [원본](tsv/base/wasm/edit7866-d_red_longer_before.tsv) |
| edit7866-e_para_start_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-e_para_start_after.tsv) | [원본](tsv/base/native/edit7866-e_para_start_after.tsv) |
| edit7866-e_para_start_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-e_para_start_after.tsv) | [원본](tsv/base/wasm/edit7866-e_para_start_after.tsv) |
| edit7866-e_para_start_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-e_para_start_before.tsv) | [원본](tsv/base/native/edit7866-e_para_start_before.tsv) |
| edit7866-e_para_start_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-e_para_start_before.tsv) | [원본](tsv/base/wasm/edit7866-e_para_start_before.tsv) |
| edit7866-f_para_end_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-f_para_end_after.tsv) | [원본](tsv/base/native/edit7866-f_para_end_after.tsv) |
| edit7866-f_para_end_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-f_para_end_after.tsv) | [원본](tsv/base/wasm/edit7866-f_para_end_after.tsv) |
| edit7866-f_para_end_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-f_para_end_before.tsv) | [원본](tsv/base/native/edit7866-f_para_end_before.tsv) |
| edit7866-f_para_end_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-f_para_end_before.tsv) | [원본](tsv/base/wasm/edit7866-f_para_end_before.tsv) |
| edit7866-g_table_cell_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-g_table_cell_after.tsv) | [원본](tsv/base/native/edit7866-g_table_cell_after.tsv) |
| edit7866-g_table_cell_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-g_table_cell_after.tsv) | [원본](tsv/base/wasm/edit7866-g_table_cell_after.tsv) |
| edit7866-g_table_cell_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-g_table_cell_before.tsv) | [원본](tsv/base/native/edit7866-g_table_cell_before.tsv) |
| edit7866-g_table_cell_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-g_table_cell_before.tsv) | [원본](tsv/base/wasm/edit7866-g_table_cell_before.tsv) |
| edit7866-h_prev_bold_match_plain_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-h_prev_bold_match_plain_after.tsv) | [원본](tsv/base/native/edit7866-h_prev_bold_match_plain_after.tsv) |
| edit7866-h_prev_bold_match_plain_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-h_prev_bold_match_plain_after.tsv) | [원본](tsv/base/wasm/edit7866-h_prev_bold_match_plain_after.tsv) |
| edit7866-h_prev_bold_match_plain_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-h_prev_bold_match_plain_before.tsv) | [원본](tsv/base/native/edit7866-h_prev_bold_match_plain_before.tsv) |
| edit7866-h_prev_bold_match_plain_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-h_prev_bold_match_plain_before.tsv) | [원본](tsv/base/wasm/edit7866-h_prev_bold_match_plain_before.tsv) |
| edit7866-i_shorter_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-i_shorter_after.tsv) | [원본](tsv/base/native/edit7866-i_shorter_after.tsv) |
| edit7866-i_shorter_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-i_shorter_after.tsv) | [원본](tsv/base/wasm/edit7866-i_shorter_after.tsv) |
| edit7866-i_shorter_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-i_shorter_before.tsv) | [원본](tsv/base/native/edit7866-i_shorter_before.tsv) |
| edit7866-i_shorter_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-i_shorter_before.tsv) | [원본](tsv/base/wasm/edit7866-i_shorter_before.tsv) |
| edit7866-j_longer_partial_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-j_longer_partial_after.tsv) | [원본](tsv/base/native/edit7866-j_longer_partial_after.tsv) |
| edit7866-j_longer_partial_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-j_longer_partial_after.tsv) | [원본](tsv/base/wasm/edit7866-j_longer_partial_after.tsv) |
| edit7866-j_longer_partial_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-j_longer_partial_before.tsv) | [원본](tsv/base/native/edit7866-j_longer_partial_before.tsv) |
| edit7866-j_longer_partial_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-j_longer_partial_before.tsv) | [원본](tsv/base/wasm/edit7866-j_longer_partial_before.tsv) |
| edit7866-k_two_shapes_then_plain_after | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-k_two_shapes_then_plain_after.tsv) | [원본](tsv/base/native/edit7866-k_two_shapes_then_plain_after.tsv) |
| edit7866-k_two_shapes_then_plain_after | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-k_two_shapes_then_plain_after.tsv) | [원본](tsv/base/wasm/edit7866-k_two_shapes_then_plain_after.tsv) |
| edit7866-k_two_shapes_then_plain_before | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/edit7866-k_two_shapes_then_plain_before.tsv) | [원본](tsv/base/native/edit7866-k_two_shapes_then_plain_before.tsv) |
| edit7866-k_two_shapes_then_plain_before | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/edit7866-k_two_shapes_then_plain_before.tsv) | [원본](tsv/base/wasm/edit7866-k_two_shapes_then_plain_before.tsv) |
| eq7873-eq-font-fraction | native | 1/1 | 99.56633% · 1 | [원본](tsv/current/native/eq7873-eq-font-fraction.tsv) | [원본](tsv/base/native/eq7873-eq-font-fraction.tsv) |
| eq7873-eq-font-fraction | wasm | 1/1 | 99.48605% · 1 | [원본](tsv/current/wasm/eq7873-eq-font-fraction.tsv) | [원본](tsv/base/wasm/eq7873-eq-font-fraction.tsv) |
| eq7873-eq-font-state | native | 1/1 | 99.68240% · 1 | [원본](tsv/current/native/eq7873-eq-font-state.tsv) | [원본](tsv/base/native/eq7873-eq-font-state.tsv) |
| eq7873-eq-font-state | wasm | 1/1 | 99.59905% · 1 | [원본](tsv/current/wasm/eq7873-eq-font-state.tsv) | [원본](tsv/base/wasm/eq7873-eq-font-state.tsv) |
| added-issue7873-font-state-document-hwpx | native | 1/1 | 95.51093% · 1 | [원본](tsv/current/native/added-issue7873-font-state-document-hwpx.tsv) | [원본](tsv/base/native/added-issue7873-font-state-document-hwpx.tsv) |
| added-issue7873-font-state-document-hwpx | wasm | 1/1 | 94.90263% · 1 | [원본](tsv/current/wasm/added-issue7873-font-state-document-hwpx.tsv) | [원본](tsv/base/wasm/added-issue7873-font-state-document-hwpx.tsv) |
| handbook-hwp | native | 383/383 | 92.49647% · 94 | [원본](tsv/current/native/handbook-hwp.tsv) | [원본](tsv/base/native/handbook-hwp.tsv) |
| handbook-hwp | wasm | 383/383 | 92.51694% · 94 | [원본](tsv/current/wasm/handbook-hwp.tsv) | [원본](tsv/base/wasm/handbook-hwp.tsv) |
| handbook-hwpx | native | 383/383 | 92.49658% · 94 | [원본](tsv/current/native/handbook-hwpx.tsv) | [원본](tsv/base/native/handbook-hwpx.tsv) |
| handbook-hwpx | wasm | 383/383 | 92.51660% · 94 | [원본](tsv/current/wasm/handbook-hwpx.tsv) | [원본](tsv/base/wasm/handbook-hwpx.tsv) |
| declared-row-band | native | 12/12 | 99.88299% · 2 | [원본](tsv/current/native/declared-row-band.tsv) | [원본](tsv/base/native/declared-row-band.tsv) |
| declared-row-band | wasm | 12/12 | 99.50222% · 2 | [원본](tsv/current/wasm/declared-row-band.tsv) | [원본](tsv/base/wasm/declared-row-band.tsv) |
| numbered-stale-tail-hwp | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/numbered-stale-tail-hwp.tsv) | [원본](tsv/base/native/numbered-stale-tail-hwp.tsv) · 기준은 쪽수 불일치 진단 |
| numbered-stale-tail-hwp | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/numbered-stale-tail-hwp.tsv) | [원본](tsv/base/wasm/numbered-stale-tail-hwp.tsv) · 기준은 쪽수 불일치 진단 |
| pdf8120-operators | native | 12/12 | 97.32493% · 1 | [원본](tsv/current/native/pdf8120-operators.tsv) | [원본](tsv/base/native/pdf8120-operators.tsv) |
| pdf8120-operators | wasm | 12/12 | 97.24382% · 1 | [원본](tsv/current/wasm/pdf8120-operators.tsv) | [원본](tsv/base/wasm/pdf8120-operators.tsv) |
| pdf8120-cell | native | 1/1 | 97.00256% · 1 | [원본](tsv/current/native/pdf8120-cell.tsv) | [원본](tsv/base/native/pdf8120-cell.tsv) |
| pdf8120-cell | wasm | 1/1 | 97.03257% · 1 | [원본](tsv/current/wasm/pdf8120-cell.tsv) | [원본](tsv/base/wasm/pdf8120-cell.tsv) |
| pdf8120-retained | native | 1/1 | 99.95532% · 1 | [원본](tsv/current/native/pdf8120-retained.tsv) | [원본](tsv/base/native/pdf8120-retained.tsv) |
| pdf8120-retained | wasm | 1/1 | 99.98911% · 1 | [원본](tsv/current/wasm/pdf8120-retained.tsv) | [원본](tsv/base/wasm/pdf8120-retained.tsv) |
| pdf8120-body-columns | native | 1/1 | 99.94584% · 1 | [원본](tsv/current/native/pdf8120-body-columns.tsv) | [원본](tsv/base/native/pdf8120-body-columns.tsv) |
| pdf8120-body-columns | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/pdf8120-body-columns.tsv) | [원본](tsv/base/wasm/pdf8120-body-columns.tsv) |
| pdf8120-solid-control | native | 1/1 | 99.91316% · 1 | [원본](tsv/current/native/pdf8120-solid-control.tsv) | [원본](tsv/base/native/pdf8120-solid-control.tsv) |
| pdf8120-solid-control | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/pdf8120-solid-control.tsv) | [원본](tsv/base/wasm/pdf8120-solid-control.tsv) |
| ordered-numbered-tail | native | 1/1 | 100.00000% · 1 | [원본](tsv/current/native/ordered-numbered-tail.tsv) | [원본](tsv/base/native/ordered-numbered-tail.tsv) · 기준은 쪽수 불일치 진단 |
| ordered-numbered-tail | wasm | 1/1 | 100.00000% · 1 | [원본](tsv/current/wasm/ordered-numbered-tail.tsv) | [원본](tsv/base/wasm/ordered-numbered-tail.tsv) · 기준은 쪽수 불일치 진단 |
| mirrored-83818 | native | 9/9 | 94.99922% · 2 | [원본](tsv/current/native/mirrored-83818.tsv) | [원본](tsv/base/native/mirrored-83818.tsv) |
| mirrored-83818 | wasm | 9/9 | 94.90934% · 2 | [원본](tsv/current/wasm/mirrored-83818.tsv) | [원본](tsv/base/wasm/mirrored-83818.tsv) |

## devel의 쪽수 불일치

8개 입력의 devel은 독립 PDF와 쪽수가 달라 정상적인 같은 물리 쪽 대응으로 판정하지 않습니다. 원본 raster의 같은 번호에서 측정 가능한 raw와 전체 페이지 존재 원장을 그대로 보존했습니다. 해당 `*-correspondence.tsv`의 빈 raw는 누락된 대응이며 0이나 통과로 바꾸지 않았습니다. 텍스트 대응 탐색은 진단이며 실루엣 수치나 직접 판독을 대체하지 않습니다.

| 입력 | 경로 | 기준 출력/PDF 쪽수 | 전체 쪽 존재 원장 |
| --- | --- | ---: | --- |
| curriculum-1342000 | native | 420/415 | [원장](tsv/base/native/curriculum-1342000-correspondence.tsv) |
| curriculum-1342000 | wasm | 420/415 | [원장](tsv/base/wasm/curriculum-1342000-correspondence.tsv) |
| 148738070 | native | 9/7 | [원장](tsv/base/native/148738070-correspondence.tsv) |
| 148738070 | wasm | 9/7 | [원장](tsv/base/wasm/148738070-correspondence.tsv) |
| 156465345 | native | 10/9 | [원장](tsv/base/native/156465345-correspondence.tsv) |
| 156465345 | wasm | 10/9 | [원장](tsv/base/wasm/156465345-correspondence.tsv) |
| 1150200 | native | 5/4 | [원장](tsv/base/native/1150200-correspondence.tsv) |
| 1150200 | wasm | 5/4 | [원장](tsv/base/wasm/1150200-correspondence.tsv) |
| added-issue7696-page-anchored-band-hwpx | native | 2/4 | [원장](tsv/base/native/added-issue7696-page-anchored-band-hwpx-correspondence.tsv) |
| added-issue7696-page-anchored-band-hwpx | wasm | 2/4 | [원장](tsv/base/wasm/added-issue7696-page-anchored-band-hwpx-correspondence.tsv) |
| added-issue7993-soft-hyphen-hwpx | native | 2/1 | [원장](tsv/base/native/added-issue7993-soft-hyphen-hwpx-correspondence.tsv) |
| added-issue7993-soft-hyphen-hwpx | wasm | 2/1 | [원장](tsv/base/wasm/added-issue7993-soft-hyphen-hwpx-correspondence.tsv) |
| numbered-stale-tail-hwp | native | 2/1 | [원장](tsv/base/native/numbered-stale-tail-hwp-correspondence.tsv) |
| numbered-stale-tail-hwp | wasm | 2/1 | [원장](tsv/base/wasm/numbered-stale-tail-hwp-correspondence.tsv) |
| ordered-numbered-tail | native | 2/1 | [원장](tsv/base/native/ordered-numbered-tail-correspondence.tsv) |
| ordered-numbered-tail | wasm | 2/1 | [원장](tsv/base/wasm/ordered-numbered-tail-correspondence.tsv) |

## 직접 판독한 대표 자료

- [80168 표제의 같은 쪽 소유·내용 보존](documents/merged-80168.md)
- [편람 HWPX 완료 서식의 원본 내용·단 소유·여백](documents/handbook-hwpx.md)

핵심 페이지의 Native/fresh WASM review·standalone overlay는 문서별 자료와 통합 PR 본문에서 제공합니다. 실제 출력의 작은 문구·수식·간격·앞뒤 조각·후속 본문을 확인한 결과와 자동 raw gate는 구분합니다.

- [현재 출력의 직접 판독·공통 컷 전후 8쪽·수식 확대](documents/current_visual_reading.md)
- [완료 수식 5문서의 독립 원점·전후 실행·핵심 회귀](../planet6897_completed_equation_pages_20261022/README.md)
- [원 PR별 추가 대표 판독·Native/WASM review 및 overlay](../planet6897_source27_representatives_20261022/README.md): 원 PR이 명시한 저장 줄 없는 HWPX의 하이픈 재조판 비적용 범위도 기록합니다. 해당 HWPX의 높은 수치를 줄 나눔 일치로 표현하지 않으며 #7993을 닫지 않습니다.

## #7967 누락 범위의 추가 확인

기존 70개 원장을 원 PR별로 재대조하면서 빠진 exam_eng·exam_kor 전 28쪽을 실제 같은 producer의 base/후보 × Native/fresh WASM으로 추가했습니다. 원래 측정 원장은 덮어쓰지 않았고 새 8개 TSV 원본을 합쳤습니다. 90% 미만·쪽수 불일치·0.1%p 이상 하락은 없습니다. [직접 판독과 기존 px 검사](documents/paragraph-border-7967.md)를 연결합니다.

| 입력 | 경로 | PDF/출력 쪽수 | 후보 최저 raw | 후보 TSV | 기준 TSV |
| --- | --- | ---: | ---: | --- | --- |
| paragraph-border-exam-eng | native | 8/8 | 97.16295% | [원문](tsv/current/native/paragraph-border-exam-eng.tsv) | [원문](tsv/base/native/paragraph-border-exam-eng.tsv) |
| paragraph-border-exam-kor | native | 20/20 | 96.63620% | [원문](tsv/current/native/paragraph-border-exam-kor.tsv) | [원문](tsv/base/native/paragraph-border-exam-kor.tsv) |
| paragraph-border-exam-eng | wasm | 8/8 | 96.96447% | [원문](tsv/current/wasm/paragraph-border-exam-eng.tsv) | [원문](tsv/base/wasm/paragraph-border-exam-eng.tsv) |
| paragraph-border-exam-kor | wasm | 20/20 | 96.67089% | [원문](tsv/current/wasm/paragraph-border-exam-kor.tsv) | [원문](tsv/base/wasm/paragraph-border-exam-kor.tsv) |

## 최종 저장 프레임 보정 뒤 재실행

`3cb5a4c290`의 편집 표 프레임 인증 보정 뒤 72개 입력의 Native/fresh WASM 전 1,529쪽을 각각 다시 실행했습니다. 최종 후보의 3,058개 raster는 직전 검증 runtime `10995c1ef8c718fb50453bc08d72551ec17c3b0d`와 RGB가 같습니다. 같은 새 PNG에서 review·overlay와 확대 자료 115개를 재생성했습니다. 이 기록은 전쪽 수치·새 산출물의 출처이며 필수 Rust·CI·직접 판독 완료를 대신하지 않습니다.

## 실제 로컬 실행 결과

[실행 명령·전체 회귀·추가 회귀·lint·Studio·fresh WASM·Native 소비 결과](documents/validation_commands.md)를 실제 종료 영수증에서 기록했습니다. 이후 정확한 제출 code head의 GitHub CI와 trailing·merge를 별도로 확인합니다.
