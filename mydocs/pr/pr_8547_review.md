# PR #8547 리뷰 — 저장 쪽 경계에서 걸침 묶음 이어받기

## 최종 판정

**메인터너 보정 후 수용 가능**
원 PR의 검증 결과와 통합 브랜치의 최종 검증 결과를 구분합니다.

메인터너가 6쪽 원본의 독립 Print 증적과 전쪽 Native/fresh WASM 측정, `65d345f8fd`의 회귀 근거를 보강했습니다.
새 devel `1924d371d6` 반영 뒤 기능 보존과 최종 공통 검증을 완료합니다. PR 생성은 작업지시자 승인 대기입니다.

## 접수 및 적용

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8547](https://github.com/edwardkim/rhwp/pull/8547), `planet6897` → `devel` |
| 원 head | `802078a02454a66ef3dc917d6ba2c6461709dcd5` |
| 통합 적용 | 코드 `8767690869`, 증적 `dc87f4fa2e`; 충돌 없이 순서대로 체리픽 |
| 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 관련 이슈 | [#8532](https://github.com/edwardkim/rhwp/issues/8532) |
| reviewer | `jangster77` 요청 및 GitHub API 재확인 완료 |

## 주장과 독립 기준

저장된 셀 줄의 쪽 되감김에서 한 줄 높이의 걸침 묶음 앞 조각을 수용하고,
이어받는 블록의 예약 높이를 실제 배치와 맞춥니다. 원 PR이 제공한
[HWP](../../samples/issue8532/gangseo_exempt_certificates.hwp)와
[한컴 Print PDF](../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf)를 대조합니다.
PDF 파일명에는 `2020`이 있지만 메타데이터의 Creator는 `Hwp 2022 12.0.0.4547`,
Producer는 `Hancom PDF 1.3.0.547`이며 6쪽입니다. 입력 SHA-256은
`797268d10fc7605ae5e0e299e9fb386ea6c204f35ef4c4879bb611e66ef4545a`,
PDF SHA-256은 `30429a79cd935c983ca5ad66642e3b189c91d0c5485c31ab2d153026f17e88b7`입니다.

원 PR head에 첨부된 [Native 전쪽 TSV](assets/issue_8532/native/fixture.tsv)는 최저
97.92%(3쪽), [fresh WASM 전쪽 TSV](assets/issue_8532/wasm/fixture.tsv)는 최저
98.03%(3쪽)이고 두 출력 모두 Print와 6쪽입니다. 이는 **작성자 head의 결과**입니다.
3쪽의 걸침 묶음 앞 조각과 4쪽의 이어받는 조각을 각각
[3쪽 Native review](assets/issue_8532/review/fx_native_review_p3.png)·
[overlay](assets/issue_8532/review/fx_native_overlay_p3.png),
[4쪽 Native review](assets/issue_8532/review/fx_native_review_p4.png)·
[overlay](assets/issue_8532/review/fx_native_overlay_p4.png)로 직접 확인했습니다.
3쪽 끝 행과 4쪽 첫 행의 소속, 표 선과 글 내용이 Print와 대응합니다.

`dc87f4fa2e`의 Native Skia 실행 파일 SHA-256
`24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`에서
같은 입력·Print와 96 DPI로 다시 측정했습니다.
[통합 head 전쪽 TSV](assets/planet6897_20261103/pr_8547/gangseo-native.tsv)의
raw 값은 1~6쪽 순서로 99.85%, 99.99%, **97.95%**, 100.00%, 100.00%, 100.00%입니다.
쪽수 6/6, 90% 미만 0쪽입니다. 새 head의 주장 쪽도
[3쪽 review](assets/planet6897_20261103/pr_8547/gangseo-native-review-p003.png)·
[overlay](assets/planet6897_20261103/pr_8547/gangseo-native-overlay-p003.png),
[4쪽 review](assets/planet6897_20261103/pr_8547/gangseo-native-review-p004.png)·
[overlay](assets/planet6897_20261103/pr_8547/gangseo-native-overlay-p004.png)에서
앞 조각·이어받는 조각과 뒤 행의 소속을 직접 대조했습니다.

## 통합 코드의 fresh WASM 전쪽 측정

같은 제품 코드 `dc87f4fa2e`의 fresh WASM
(`c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`)을
2026-11-04에 같은 입력·독립 Print PDF, 96 DPI로 측정했습니다.
[전쪽 TSV](assets/planet6897_20261103/pr_8547/gangseo-wasm.tsv)는 6/6쪽,
90% 미만 0쪽이며 1~6쪽 raw는 99.87%, 99.99%, **97.98%**, 100.00%, 100.00%,
100.00%입니다.
[3쪽 review](assets/planet6897_20261103/pr_8547/gangseo-wasm-review-p003.png)·
[overlay](assets/planet6897_20261103/pr_8547/gangseo-wasm-overlay-p003.png)와
[4쪽 review](assets/planet6897_20261103/pr_8547/gangseo-wasm-review-p004.png)·
[overlay](assets/planet6897_20261103/pr_8547/gangseo-wasm-overlay-p004.png)를 같은 입력의
같은 쪽에 연결했습니다.

## 남은 검증

- 최소 글꼴 환경에서 `issue_8532_rowspan_hard_break`와 관련 대조군, 전체 회귀를 실행합니다.
- 기준 devel과 같은 입력·Print·글꼴 조건에서 페이지별 하락을 대조하고,
  다른 문서에 대한 #8547의 영향도 분리해 판정합니다.
- 통합 PR이 병합되면 실제 merge SHA와 검증한 증거를 원 PR에 연결하고 후속 처리합니다.
