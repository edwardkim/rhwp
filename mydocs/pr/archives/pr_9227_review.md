# PR #9227 리뷰 — fix(typeset): 단 폭을 채운 어울림 개체는 밀린 글줄이 넘어가도 떠나는 쪽 말미에 남긴다 (#9223)

## 최종 판정

**승인.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9227](https://github.com/edwardkim/rhwp/pull/9227) · planet6897 · devel
- 원 head `b9632d620e1100941769e00fb4f923f259078823` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `3a24a22e40a2db82fa440ca47119a6caa4ab1f1c` → `9e8085f808580abb36bc0cfda1e8f8bdfc57aac6`
  - `b9632d620e1100941769e00fb4f923f259078823` → `88e6c6bd856c5501aa2fcd94014b1c43ada66433`

## 원 PR 주장과 확인 범위

단 폭을 채운 안내 띠가 앞쪽에 남고 밀린 글줄은 다음 쪽에 붙는지 대조한다.

현재 Native와 fresh WASM에서 3쪽 모두 100.00%로 표시된다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9223/ccus-press-release-square-band.hwpx` · SHA-256 `043f5db6444ecbe6a23a7e70f1dfbc61fdc66331e807302dc26c647628197a66`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 3쪽
  `pdf/issue9223/ccus-press-release-square-band-2024.pdf` · SHA-256 `9ffb7b895f9f77cb669d4dbef56a5285b7e7bc0f8d1e3d66114fe0fdc20dec17`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 3 | 100.00% (1쪽) | 0 |
| fresh WASM | 3 | 100.00% (1쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9227/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/silhouette.tsv)

1쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9227/native/raw-001.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9227/native/pdf-001.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9227/native/review/review_001.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/native/overlay/overlay_001.png)
- 1쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9227/native/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/native/overlay/overlay_001.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/overlay/overlay_001.png)
- 2쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9227/native/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/native/overlay/overlay_002.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/overlay/overlay_002.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 로컬 제품 검증 결과를 기록했다. [통합 PR #9283](https://github.com/edwardkim/rhwp/pull/9283)의 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/44881574506)와 [연계 검사](../assets/planet6897_9208_9236_20261121/integration-code-ci.json)를 통과했다.

## Merge 후 contributor PR comment 계획

단 폭을 채운 개체는 앞쪽에 남고 밀린 글줄은 다음 쪽에 이어지는 것을 확인했습니다.

해당 문서의 데스크톱 한컴 Hancom PDF Print와 최신 후보의 Native·fresh WASM 비교를 기록한다. 통합 병합 SHA에서 다음 쪽의 review·overlay 바이트를 확인한 뒤 원 PR에 표시한다.

- Native · 1쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9227/native/review/review_001.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/native/overlay/overlay_001.png)), 2쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9227/native/review/review_002.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/native/overlay/overlay_002.png)).
- fresh WASM · 1쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/review/review_001.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/overlay/overlay_001.png)), 2쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/review/review_002.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9227/wasm/overlay/overlay_002.png)).
