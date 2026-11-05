# c183 fresh WASM의 PR #8389·#8397 전쪽 보존 검증

**중복 제외 17문서·1,770쪽·원 TSV 2,153행에서 표시값 하락 0행·누락 0행·검증 오류 0건**이다.
같은 쪽의 원 TSV 기준 표시값과 현재 표시값을 소수 셋째 자리에서 반올림한 두 자리로 대조했다. 상승은 허용한다.
문서별 전쪽은 90% 이상이고 issue7905의 6쪽은 모두 98% 이상이다. 평균값으로 미달을 상쇄하지 않았다.

실제 producer는 `c183a1ac61c2aa4188989fd93032cae68d3538c1`이다. 최신 ba2 통합 source의 새 전쪽 출력으로 표시하지 않는다.
충돌 없는 리베이스 뒤 전체 재비교를 반복하지 않는 작업 지시에 따라 직전 완료 증거를 출처와 함께 재사용한다.
최신 변경 소비 경로의 실제 base/candidate 검증은 [최신 통합 상태](../../README.md)에 별도로 연결한다.

- Native binary SHA-256: `cbb91433ec4e5b968ea7820dae1af09aa712991ae5b3027c68590fee5f2138d9`.
- fresh JS SHA-256: `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
- fresh WASM SHA-256: `f79bffaabc76cd4917791dc3dbececa2560cba8001fcb17578ba03076a5ce77e`.
- 96 DPI·Print profile·색상 경계 보정 전 2px raw. 시각 측정은 시스템 글꼴 전체와 기존 명시 대체 계약을 사용한다.
- 5자리 정밀 manifest raw와 canonical TSV 두 자리 표시의 일치·원본/PDF/product SHA·쪽수·연속 전쪽 범위를 확인했다.
- sample10의 외부 참조 그림 `oracle.gif`·`rdb02.gif`·`s1.jpg`를 실제 공급하고 바이트 SHA를 대조했다.
- 한컴 Print PDF·입력·기준 TSV·허용값을 변경하지 않았고 글꼴 파일을 배포하지 않는다.

| 문서 · 원본 · 독립 Print | 전체 쪽수 · 정밀 raw 최저 | 실제 fresh WASM TSV · SHA-256 |
| --- | --- | --- |
| `issue7905` · [jongbu_law.hwpx](<../../../legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx>) · [jongbu_law_hancom2018_print.pdf](<../../../legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf>) | 6쪽 · 98.86659% | [TSV](wasm/issue7905.tsv) · `e18b07f77903f2eb05b779457a1bb606a3c0ca6aa9854a94a6320c7993334942` |
| `issue8040-insert` · [exam-social-insert.hwp](<../../../../../../samples/pr8040/exam-social-insert.hwp>) · [exam-social-insert-2020.pdf](<../../../../../../pdf/pr8040/exam-social-insert-2020.pdf>) | 4쪽 · 99.98866% | [TSV](wasm/issue8040-insert.tsv) · `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| `issue8040-insert-delete` · [exam-social-insert-delete.hwp](<../../../../../../samples/pr8040/exam-social-insert-delete.hwp>) · [exam-social-insert-delete-2020.pdf](<../../../../../../pdf/pr8040/exam-social-insert-delete-2020.pdf>) | 4쪽 · 99.98866% | [TSV](wasm/issue8040-insert-delete.tsv) · `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| `sample16-hwpx` · [hwp3-sample16-hwp5.hwpx](<../../../../../../samples/hwp3-sample16-hwp5.hwpx>) · [hwp3-sample16-hwp5-hwpx-2024.pdf](<../../../../../../pdf/hwp3-sample16-hwp5-hwpx-2024.pdf>) | 64쪽 · 99.99273% | [TSV](wasm/sample16-hwpx.tsv) · `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa` |
| `sample16-hwp2024` · [hwp3-sample16-hwp5-2024.hwp](<../../../../../../samples/hwp3-sample16-hwp5-2024.hwp>) · [hwp3-sample16-hwp5-2024-hwp-2024.pdf](<../../../../../../pdf/hwp3-sample16-hwp5-2024-hwp-2024.pdf>) | 64쪽 · 99.99273% | [TSV](wasm/sample16-hwp2024.tsv) · `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa` |
| `sample16-hwp2022` · [hwp3-sample16-hwp5-2022.hwp](<../../../../../../samples/hwp3-sample16-hwp5-2022.hwp>) · [hwp3-sample16-hwp5-2022-hwp-2020.pdf](<../../../../../../pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf>) | 64쪽 · 99.99816% | [TSV](wasm/sample16-hwp2022.tsv) · `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| `handbook-hwp` · [2025 행정업무운영 편람(최종).hwp](<../../../../../../samples/2025 행정업무운영 편람(최종).hwp>) · [2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf](<../../../../../../pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf>) | 383쪽 · 99.96402% | [TSV](wasm/handbook-hwp.tsv) · `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| `handbook-hwpx` · [2025 행정업무운영 편람(최종).hwpx](<../../../../../../samples/2025 행정업무운영 편람(최종).hwpx>) · [2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf](<../../../../../../pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf>) | 383쪽 · 99.96402% | [TSV](wasm/handbook-hwpx.tsv) · `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| `seo-body-list-typed` · [body-list-typed-hwp.hwp](<../../../../../../tests/fixtures/issue_8109_saved_fonts/body-list-typed-hwp.hwp>) · [body-list-typed-hwp-2020.pdf](<../../../../../../pdf/issue8109/print/body-list-typed-hwp-2020.pdf>) | 1쪽 · 100.0% | [TSV](wasm/seo-body-list-typed.tsv) · `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| `seo-helvetica-styles` · [control-styles-hwp.hwp](<../../../../../../tests/fixtures/issue_8109_saved_fonts/control-styles-hwp.hwp>) · [control-styles-hwp-2020.pdf](<../../../../../../pdf/issue8109/print/control-styles-hwp-2020.pdf>) | 1쪽 · 100.0% | [TSV](wasm/seo-helvetica-styles.tsv) · `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| `seo-pic-crop-01` · [pic-crop-01.hwp](<../../../../../../samples/pic-crop-01.hwp>) · [pic-crop-01-2022.pdf](<../../../../../../pdf/pic-crop-01-2022.pdf>) | 1쪽 · 94.65178% | [TSV](wasm/seo-pic-crop-01.tsv) · `7678d6f2f5d362601ec7573b64be6cc0077e70485b9fbdd18272914d4058f420` |
| `seo-shape-group-02` · [shape-group-02.hwp](<../../../../../../samples/shape-group-02.hwp>) · [shape-group-02-hwp-2020.pdf](<../../../../../../pdf/shape-group-02-hwp-2020.pdf>) | 1쪽 · 100.0% | [TSV](wasm/seo-shape-group-02.tsv) · `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| `seo-re-03-latin` · [re-03-latin-only-hancom.hwp](<../../../../../../samples/re-03-latin-only-hancom.hwp>) · [re-03-latin-only-hancom-hwp-2020.pdf](<../../../../../../pdf/re-03-latin-only-hancom-hwp-2020.pdf>) | 1쪽 · 100.0% | [TSV](wasm/seo-re-03-latin.tsv) · `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| `sample10` · [hwp3-sample10.hwp](<../../../../../../samples/hwp3-sample10.hwp>) · [hwp3-sample10-2020.pdf](<../../../../../../pdf/issue8371/hwp3-sample10-2020.pdf>) | 763쪽 · 99.3972% | [TSV](wasm/sample10.tsv) · `3f55d35cb8f9f8ec5946ab5010f085c0518bb79a8c734c7a11c28227277d88a2` |
| `yeongwol` · [20099369_yeongwol_forms.hwp](<../../../../../../samples/issue5699/20099369_yeongwol_forms.hwp>) · [20099369_yeongwol_forms-2020.pdf](<../../../../../../pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf>) | 4쪽 · 99.97763% | [TSV](wasm/yeongwol.tsv) · `b81ecaef16fbdec585263b66c710bcb0dc2727edf103ace7c8c431a6bbd874bf` |
| `outline` · [issue3307_outline_number.hwpx](<../../../../../../samples/task3307/issue3307_outline_number.hwpx>) · [issue3307_outline_number-2020.pdf](<../../../../../../pdf/task3307/issue3307_outline_number-2020.pdf>) | 9쪽 · 99.93728% | [TSV](wasm/outline.tsv) · `70cc9459d29c3dc4f5a4f33d1d19d0efb7fa6b2e47bb39230779c46f68041010` |
| `nested` · [issue2007_nested_cell_pagination_42065.hwp](<../../../../../../samples/basic/issue2007_nested_cell_pagination_42065.hwp>) · [issue2007_nested_cell_pagination_42065-2020.pdf](<../../../../../../pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf>) | 17쪽 · 96.35271% | [TSV](wasm/nested.tsv) · `f61db5f2635e7dc30c201e6d11a2e7677f2b7e109e77617effc1c1c5f0c98514` |

[원 TSV 2,153행 대조표](wasm-display-comparison.tsv) · [같은 c183 Native 결과](native-validation.md).

비교 기준 TSV SHA-256 `71d18230f18121580fdd98f8d50068469196d04724d434d98495bef6edaef9b4`.
실제 대조표 SHA-256 `c8af95bfd4aeb2f1aaac13fa609a37281896f4911a9e17068cbd80ea06ddc448`.

원시 실행 로그·manifest·체크포인트와 재개 출처는 ignored output에 유지한다.
HWP 편람의 초기 부분 실행은 Git 출처 변경으로 resume가 거부되어 이전 manifest를 수정하지 않고 보존한 뒤 같은 고정 제품으로 새 실행을 완료했다.
최종 전쪽 manifest와 383쪽 TSV를 확인했으며 부분 실행·실패한 resume를 완료 증거로 쓰지 않는다.
