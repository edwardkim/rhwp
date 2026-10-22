# Stage 48 분할 흐름 검증 증적

- 시작 HEAD: `6ab6b5e7206ff8740ea1a4051b96a4c4da54e1d2`. 정확한 코드 식별은 아래 파일 SHA와 patch SHA를 사용한다.
- Patch SHA-256: `4377a0637191c4e8b0fb561e56bb73f610dcceaa9442660a0437141d323918bc`.
- Frozen Native CLI SHA-256: `b51a22721ca0de95ffcc36cc06535283856b8e95e501f357424d37b5399f6de4`.
- Frozen rlib SHA-256: `6d47ba0ce3bbd6c8fc1d380682130650e3e89fa9faa12a5963f1f50d49aab5d0`.
- 현재 canonical Visual Sweep, Native Skia 인쇄 프로필, compat 원본별 2022/2024, 96DPI, PDFium 152.0.7947.0 / pdfium_smooth_rgb_v1, 원래 2px 실루엣 90%를 적용했다. 같은 713개 글꼴 파일 ledger를 사용하고 캡처 동시성은 총 8이다.
- fresh WASM, 전체 nextest, 필수 전체 게이트와 실제 소비 경로 검증은 이 단계에서 완료하지 않았다. PR 제출 승인이 아니다.

| 입력 | 전체 쪽수 | Native 최저 | 90% 미만 |
| --- | ---: | ---: | ---: |
| giant-hwp | 115 | 97.01264% | 0 |
| giant-hwpx | 115 | 97.01264% | 0 |
| split-tab-period | 197 | 94.69402% | 0 |
| split-current | 197 | 94.69402% | 0 |
| footer-7840 | 9 | 96.53204% | 0 |

## 출처와 비교 범위

- `before-native.tsv`는 Stage 47 코드의 v19 before 실행이다. 수정 전후 각 조건의 전체 쪽수를 확인했으며 4조건 624쪽의 소유별 연결 문자열 변화는 0, 시작 코드 대비 0.1%p 이상 하락은 0쪽이다. 소유별 문자열 감사만으로 서로 다른 소유자 사이의 전역 순서를 입증하지 않는다.
- 원본 HWP/HWPX 각 115쪽과 footer 9쪽은 해당 수정 전 PNG와 모두 byte 동일하다.
- 고정 devel `2e087a5b4bc1515ad877e8020b30114639a59392`의 분할본은 196쪽이어서 독립 PDF 197쪽과 불일치한다. v18 strict 실패를 보존했다. 이 분할본의 devel 같은 쪽 비교 gate를 통과했다고 주장하지 않는다.
- 옛 Stage 30 바이너리 대조는 같은 현재 글꼴·변환기로 두 입력 각 197쪽, 최저 94.69402%, 미달 0을 재현한 원인 진단이다. 정확한 pre-rebase 5e 또는 현재 head 검증을 대신하지 않는다.

## 직접 판독

- 25쪽 review/overlay: 앞 빈 줄 두 개의 점유와 TAC 표 원점이 복구됐고 뒤 본문은 유지된다. 이전 v28 그림을 함께 보존한다.
- 90쪽 review/overlay: 8.2.2의 닫는 네 줄 및 뒤 쪽 소유가 복구됐다. 140℃ 주변 줄바꿈·글자 간격 차이는 남아 있으며 94.69402%를 완전한 시각 일치라고 부르지 않는다. 새 px 핀을 추가하지 않았다.
- 137쪽 review/overlay: 제목 10.11.1부터 이후 문단의 소속이 복구됐다.
- 177쪽 review/overlay: 자식 표의 끝과 뒤 제목, 다음 쪽 그림의 소유가 복구됐다. 일부 글자 가장자리·테두리 굵기 및 미세 기준선 차이는 남는다.
- 중간 v29 후보는 196쪽을 만들어 실패했고, 최종 v31과 섞지 않는다. 테스트 기대값·90%·0.1%p·변환기를 완화하지 않았다.

## 입력과 독립 PDF SHA-256

- `giant-hwp` 입력 `/Users/tsjang/rhwp-7445-completed-20261009/samples/issue1949_giant_cell_nested_tables_perf.hwp`: `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e`
  - 독립 PDF `/Users/tsjang/rhwp-7445-completed-20261009/pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf`: `9fb08dea604813eed90a0449f29682d400be54dacd413352cc4f680f37cd4a44`
- `giant-hwpx` 입력 `/Users/tsjang/rhwp-7445-completed-20261009/samples/issue1949_giant_cell_nested_tables_perf.hwpx`: `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65`
  - 독립 PDF `/Users/tsjang/rhwp-7445-completed-20261009/pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf`: `ba065afdd3ace7fd6b87fe82848e6cae3d01915a1606e4d8a70e4c2153d57d73`
- `split-tab-period` 입력 `/Users/tsjang/rhwp-7445-completed-20261009/tests/fixtures/issue4138/tab-period-split.hwp`: `d9bc6b13d54f5179c2cf57e6a2310a28efce4f8f6d6e74137a5828c9878a24b6`
  - 독립 PDF `/Users/tsjang/rhwp-7445-completed-20261009/pdf/issue4138-tab-period-split-20261015-2024.pdf`: `687f1404e56a764258c93fb0e59d9fb1dd870b506d36451fbfa833fd207f8294`
- `split-current` 입력 `/Users/tsjang/rhwp-7445-completed-20261009/mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp`: `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`
  - 독립 PDF `/Users/tsjang/rhwp-7445-completed-20261009/pdf/issue4138-ascii-split-20261016-2024.pdf`: `5a82290de1571c692c7246fe6b57ae136602ae52c2068f2568fffc6c91bfd571`

## 생산 소스 SHA-256

- `src/renderer/layout/table_layout.rs`: `73a26e5253979fb6bc955f87025b4c72ba2a6c3e90f74062372c9e11ec679aa3`
- `src/renderer/layout/table_partial.rs`: `2cc777d06cea6e8eb368fd60aa9952f920a45389251bd232799f4aea31de35fb`
- `src/renderer/layout/table_layout/empty_line_flow.rs`: `cfeb8431bbd35c6c4bdd8315cfdfb35e53d08644c20368f340f21469d644d871`
- `src/renderer/layout/table_layout/row_physical_band.rs`: `fec3554012c6fed75da477b6508e3f2973c1cb34bd3a0ab301c07f7a03c2fa7f`
- `src/renderer/float_placement/rowbreak_frame.rs`: `a74d33c040d08f81f47ce1e391f3eec26c375e1f09b0e186613b1d0b277521c6`
- `src/renderer/typeset/table/scan/runner/row_step.rs`: `84ae95230de0ece13ff51505b77bf7142e96f051089d4a99b7d4c8f35aad1931`

## 산출물 SHA-256

- [giant-hwp-native.tsv](giant-hwp-native.tsv): `90bfa27e30629056a66dc1ad90b8c9e861783327b88220d4849a9cfd52851e84`
- [giant-hwpx-native.tsv](giant-hwpx-native.tsv): `90bfa27e30629056a66dc1ad90b8c9e861783327b88220d4849a9cfd52851e84`
- [split-tab-period-native.tsv](split-tab-period-native.tsv): `799eaf2e3c64e3f8a17855e3f755fbcae4dbd937ef03f667e1227f0de10880e3`
- [split-current-native.tsv](split-current-native.tsv): `aa0b430f7c86473f3e5ba05bcfa3f7c6ea575ce35116494eaa00c6bcc23cbbdb`
- [split-tab-period-before-native.tsv](split-tab-period-before-native.tsv): `5e200a88006a1f8c74cb17b30f5cfe1a69ed71fbff8db920d07a6dc5ef28a4b2`
- [split-current-before-native.tsv](split-current-before-native.tsv): `68dfbb660aae4b8ae2cd10584f0e6fc237eeb736675cd81cd5afa3f2067981ff`
- [footer-7840-native.tsv](footer-7840-native.tsv): `18c54784bf39beb86113197b1ea4400215b0e7efe3adf720307bb133687e4cd5`
- [after-current-p025-review.png](after-current-p025-review.png): `4f8a9e4ef53333465a642b43ea9a3489409c8fd06b2f59a3047463b89576bead`
- [after-current-p025-overlay.png](after-current-p025-overlay.png): `61b31b57a36ee593072b24450076abb71309c994ac731873972468848477d657`
- [after-current-p090-review.png](after-current-p090-review.png): `93105e5a2001545eb2e39c1eecb7e95d3c5784da19808e36d1c918a3d3c8708d`
- [after-current-p090-overlay.png](after-current-p090-overlay.png): `da162f0179e72914ddd65d8ab21892c5076b64bddbf6de79391b26e3244c4965`
- [after-current-p137-review.png](after-current-p137-review.png): `4247aced94f9a8671ab3164a42204ec9cfec0ccc4735d7016afa6341f1af40d1`
- [after-current-p137-overlay.png](after-current-p137-overlay.png): `760449aa7d5a23a3913367a668d12393c83d3bcdb61e1fdc89b921a7a604e93e`
- [after-current-p177-review.png](after-current-p177-review.png): `c90ce56e0620a1477d0d1e95f275497ea5dcbd01fea73f209b69f99e0f52c3db`
- [after-current-p177-overlay.png](after-current-p177-overlay.png): `b5020ec2077b0f4f97d91de834d1a3b57d8212bdeeadccce77f8cde0d19eb4b0`
- [intermediate-v28-p025-review.png](intermediate-v28-p025-review.png): `169dae1b88c506525e53bc23c6cfed6fe5661e5f9f93c4d0b3ebd2a7454bc21c`
- [intermediate-v28-p025-overlay.png](intermediate-v28-p025-overlay.png): `8c23f138a8bc1824de24854a4aa1c7705fdaf2f5d0c874d3eec3db07ebd91432`
- [intermediate-v28-p090-review.png](intermediate-v28-p090-review.png): `73c328a9571f9f313643ee9a797ee65fae7d1b01e2bef1abbe13a675da438442`
- [intermediate-v28-p090-overlay.png](intermediate-v28-p090-overlay.png): `9d20ec2a627febc0dcb7a2b13302681bdc096bac9b82327e98e395363c1c7399`

## 원 실행 기록

임시 스크립트, 원장 JSON, 로그는 ignored output에만 보존한다.

`/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022` 아래 `repair-focused-v31`, `split-picture-native-v31`, `picture-band-7840-control-v31`, `v31-whole-comparison.json`, `v31-footer-png-comparison.json`, `current-input-capacity-trace`가 출처다.
