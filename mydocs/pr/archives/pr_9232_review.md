# PR #9232 리뷰 — fix(typeset): 어울림 표 양옆 차선에 이어진 뒤 문단은 표 옆에서 자기 줄만 소비한다 (#9228)

## 최종 판정

**승인.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9232](https://github.com/edwardkim/rhwp/pull/9232) · planet6897 · devel
- 원 head `6cdc41c1a275bc13093a2650e2364d8d21a169ff` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `e8f6401fbfd3e5c8175997ff424964ab0e4b95d3` → `55da61a62958b84f9c507c5d546739a6ecbebdee`
  - `6cdc41c1a275bc13093a2650e2364d8d21a169ff` → `f41dc7b9ebcac38f349b57ef49d128c84eaf9abe`

## 원 PR 주장과 확인 범위

어울림 표 양옆의 상자와 뒤 문단의 저장 줄을 대조한다.

공통 쪽의 내용 소속을 점검했다. base의 추가 쪽과 후보 21쪽을 쪽 번호만으로 비교하지 않는다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9228/regulatory-sandbox-press-release-flanking-box.hwpx` · SHA-256 `7f4bce81e2955a0fc7524b70e6e00e2217f23a85d95cb45d2b21093bd79d05a7`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 21쪽
  `pdf/issue9228/regulatory-sandbox-press-release-flanking-box-2024.pdf` · SHA-256 `9b43307c5f98cb3b86ea1612f7c2d790ced521ed38719ca1ae6ca90e595f1ca7`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 21 | 99.91% (1쪽) | 0 |
| fresh WASM | 21 | 99.90% (1쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9232/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/silhouette.tsv)

6쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9232/native/raw-006.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9232/native/pdf-006.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_006.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_006.png)
- 1쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_001.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/overlay/overlay_001.png)
- 5쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_005.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_005.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/review/review_005.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/overlay/overlay_005.png)
- 6쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_006.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_006.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/review/review_006.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/overlay/overlay_006.png)

## 고정 devel의 쪽수 차이

고정 `devel`(`cb85758e`) Native 출력은 22쪽이다. 한컴 Print와 리베이스 직후 후보(`9acc5fde86`) Native 출력은 각각 21쪽이다. `devel` 7쪽으로 밀린 기업 신청단계별 지원 표와 지원단 운영 문단은 후보 6쪽에서 상단 표의 뒤를 잇는다. `devel` 6·7쪽의 공백을 정규화한 render tree 텍스트 801자가 후보 6쪽과 순서까지 같고, 나머지 20쪽도 대응하는 내용과 일치했다. 최종 후보의 두 경로는 별도로 대조한다.

- [쪽 대응 및 비교 JSON](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-native-final/report.json)
- `devel` 6쪽 대 한컴 Print 6쪽 · [review](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-native-final/base-006/review/review_006.png) · [overlay](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-native-final/base-006/overlay/overlay_006.png)
- `devel` 7쪽 대 한컴 Print 6쪽 · [review](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-native-final/base-007/review/review_007.png) · [overlay](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-native-final/base-007/overlay/overlay_007.png)

- fresh WASM에서도 고정 `devel` 22쪽, 후보와 한컴 Print 21쪽을 확인했다. 6·7쪽을 이은 본문을 포함해 대응하는 21쪽에서 정규화한 render tree 텍스트가 같았다. [쪽 대응 JSON](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-wasm-final/report.json) · [devel 7쪽 review](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-wasm-final/base-007/review/review_007.png) · [overlay](../assets/planet6897_9208_9236_20261121/pagination/pagination-9232-wasm-final/base-007/overlay/overlay_007.png)

[쪽 분할 검토 종합](../assets/planet6897_9208_9236_20261121/pagination/pagination-review-disposition.json).

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 로컬 제품 검증 결과를 기록했다. [통합 PR #9283](https://github.com/edwardkim/rhwp/pull/9283)의 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/44881574506)와 [연계 검사](../assets/planet6897_9208_9236_20261121/integration-code-ci.json)를 통과했다.

## Merge 후 contributor PR comment 계획

어울림 표 양쪽에 이어지는 문단의 글줄과 상자 소속을 확인했습니다. 쪽수가 다른 고정 기준은 대응 내용으로 맞춰 대조했습니다.

해당 문서의 데스크톱 한컴 Hancom PDF Print와 최신 후보의 Native·fresh WASM 비교를 기록한다. 통합 병합 SHA에서 다음 쪽의 review·overlay 바이트를 확인한 뒤 원 PR에 표시한다.

- Native · 1쪽 99.91% ([review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_001.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_001.png)), 5쪽 99.99% ([review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_005.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_005.png)), 6쪽 99.96% ([review](../assets/planet6897_9208_9236_20261121/cases/9232/native/review/review_006.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/native/overlay/overlay_006.png)).
- fresh WASM · 1쪽 99.90% ([review](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/review/review_001.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/overlay/overlay_001.png)), 5쪽 99.99% ([review](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/review/review_005.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/overlay/overlay_005.png)), 6쪽 99.96% ([review](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/review/review_006.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9232/wasm/overlay/overlay_006.png)).
