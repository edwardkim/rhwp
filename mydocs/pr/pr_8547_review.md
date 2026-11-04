# PR #8547 리뷰 — 저장 쪽 경계에서 걸침 묶음 이어받기

## 최종 판정

**메인터너 보정 후 수용 가능**

완료한 실측과 남은 통합 공통 검증을 구분합니다. 원 PR의 직접 승인·닫기와 통합 PR 생성은 수행하지 않았습니다.
당시 base 대비 전쪽 대조는 완료했습니다. 새 보정의 필수 검증과 전체 증적 감사를 마친 뒤 최종 수용을 판정합니다. PR 생성은 작업지시자 승인 대기입니다.

## 추가 보정의 Native·fresh WASM 결과 — 필수 회귀 검증 대기

공통 물리 프레임 전달 보정 `e61ed1ca7329d3dacffc9c7fc76e602f30f43f66`의
Native·fresh WASM을 96 DPI·같은 원본·같은 독립 Print PDF로 전 6쪽 재측정했습니다.
3쪽 raw는 Native 97.95% → 100.00%, WASM 97.98% → 100.00%로 개선됐고 나머지
쪽의 표시값은 유지됐습니다. 두 경로 모두 Print와 6쪽이 일치합니다.

| 새 보정 근거 | Native | fresh WASM |
| --- | --- | --- |
| 전 6쪽 TSV | [TSV](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native.tsv) · 최저 99.85% | [TSV](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm.tsv) · 최저 99.87% |
| 3쪽 · 위생 앞 조각의 아래 외곽 | [review](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p003-review.png) / [overlay](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p003-overlay.png) | [review](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p003-review.png) / [overlay](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p003-overlay.png) |
| 4쪽 · 영양사 이어받기와 뒤 행 | [review](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p004-review.png) / [overlay](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-p004-overlay.png) | [review](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p004-review.png) / [overlay](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-p004-overlay.png) |
| 전쪽 무하락 대조 | [Native 비교](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-comparison.json) | [WASM 비교](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-comparison.json) |
| 실제 제품과 소스 연결 | [제품·소스](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/native-product.json) | [빌드·패키지·증적 해시](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/wasm-provenance.json) |

[앞·뒤 쪽수·내용 컷 대조](assets/planet6897_20261103/pr_8547/closing-frame-e61ed1ca73/page3-owned-cut.json)에서
6쪽과 내용 컷은 그대로이며 실제 물리 점유만 보정됐습니다. Native·WASM의 3·4쪽 review와
별도 overlay를 직접 판독해 3쪽 표 하단, 4쪽 영양사 이어받기와 뒤 행의 소유·순서를
확인했습니다. 100.00%는 2px 관용 실루엣 표시값이며 엄격 RGB 완전 일치를 뜻하지 않습니다.

새 보정의 필수 회귀·정상 대조군 검증은 남아 있습니다. 아래 보정 전 제품의 전체 검증
통과를 새 보정의 통과로 표시하지 않으며, 이를 완료한 뒤 최종 판정을 갱신합니다.

## 보정 전 검토 코드와 증거 출처

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8547](https://github.com/edwardkim/rhwp/pull/8547) |
| 원 head | `802078a02454a66ef3dc917d6ba2c6461709dcd5` |
| 현재 통합 적용 커밋 | `a6f332bf20a5baaaae5e2b430bd7a5414f7ddafb, 029ca31d2e6279b872deb93292b2f5f3759ab3fd` |
| 보정 전 필수 검증 제품 / 새 base | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` / `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 전쪽 Native 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| fresh WASM 실제 빌드 | `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` (위 제품 코드와 동등한 구현) |
| reviewer | `jangster77` 요청; 2026-11-04T17:44:08.556Z 고정한 원 head·Open·non-Draft 확인 |

기존 전쪽 TSV/PNG는 실제 측정 producer의 결과로 재사용합니다. 최신 소스의 재출력으로 표시하지 않습니다.
보정 전 제품의 최소 글꼴 전체 Rust 12,561개 통과·Native Skia·Rust lint 및 fresh WASM 실제 실행과
새 base 기능 보존 집중 대조 14쪽을 완료했습니다. [제품 검증](assets/planet6897_20261103/latest-product-validation-a7e5ce7112.json)과
[최신 리베이스 대조](assets/planet6897_20261103/rebase_scope_22947f6a90/base-comparison.json)를 연결합니다.

## 보정 전 전쪽 Native/fresh WASM 증적

| 원본 · 독립 Print | Native TSV · 전쪽 최저 | fresh WASM TSV · 전쪽 최저 |
| --- | --- | --- |
| [gangseo_exempt_certificates.hwp](../../samples/issue8532/gangseo_exempt_certificates.hwp) · [issue8532_gangseo_exempt_certificates-2020.pdf](../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf) | [TSV](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo_exempt_certificates-e5b4dd3b-native.tsv) · 97.95000% (6쪽) | [TSV](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo_exempt_certificates-e5b4dd3b-wasm.tsv) · 97.98000% (6쪽) |

96 DPI, 색상 경계 보정 전 2px 내용 실루엣입니다. 입력·Print PDF는 기존 동일 파일을 재사용하고 전체 쪽수와 누락을 확인했습니다.

| 주장 쪽·범위 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| gangseo_exempt_certificates.hwp 3쪽 · 위생 걸침 묶음의 앞 조각과 저장 쪽 경계 | [review](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-native-review.png) / [overlay](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-native-overlay.png) | [review](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p003-p003-wasm-overlay.png) |
| gangseo_exempt_certificates.hwp 4쪽 · 위생 걸침 묶음 이어받기와 뒤 행 | [review](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-native-review.png) / [overlay](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-native-overlay.png) | [review](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8547/current-4b149b6cdf/pr8547-gangseo-p004-p004-wasm-overlay.png) |

## 보정 전 경계의 직접 판독

Native/fresh WASM의 3·4쪽을 각각 직접 대조했습니다. 위생 묶음의 앞 조각은 3쪽, 이어받는 내용은 4쪽에 보존됩니다. 다만 3쪽 마지막 행 아래 외곽 높이는 독립 Print와 다릅니다. 원본 저장값·원 해결 PR과 현재 조각의 요구/예약/실제 높이를 대조하기 전에는 이 차이를 정상이나 해결 완료로 판정하지 않습니다. 4쪽의 행·내용 순서와 외곽은 대응합니다.

[직접 판독 기록과 PNG 해시](assets/planet6897_20261103/direct-reading-current.json)에 실제 관측을 보존했습니다.

## 남은 통합 검증과 원격 조건

동일 환경의 당시 base 대비 Native·WASM 각각 80개 입력·406쪽 대조는 하락 허용폭 0.1%p 초과 0쪽으로 완료했습니다.
[당시 제품의 대조 기록](assets/planet6897_20261103/base-comparison-capture-4b149b6cdf.json)을 연결하며,
새 보정 제품의 정상 대조군·회귀 검증과 주장별 증적 최종 감사는 남아 있습니다.
[완료·대기 검증 기록](assets/planet6897_20261103/review-preparation-a7e5ce7112.json)을 따릅니다.

아래 선행 분석은 당시 코드·증적의 기록입니다. 당시 남은 검증 문장은 현재 상태가 아니며 위 현재 기록과 구분합니다.

<details>
<summary>선행 구현 분석·독립 근거·당시 실행 기록</summary>

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

</details>
