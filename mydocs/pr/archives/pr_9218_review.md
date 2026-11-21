# PR #9218 리뷰 — fix(render): 참조 없는 글머리표는 끝 모양의 진하게를 쓰지 않고, Wingdings 고리·네모 틀을 원 윤곽으로 그린다 (planet6897/rhwp-issue-docs#614)

## 최종 판정

**승인.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9218](https://github.com/edwardkim/rhwp/pull/9218) · planet6897 · devel
- 원 head `d69fe770b7e5e092746f366cb0849924adda8369` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `d69fe770b7e5e092746f366cb0849924adda8369` → `e025576cf3a06c3f6a4a8d3819072964655c3bb5`

## 원 PR 주장과 확인 범위

참조 없는 글머리표의 굵기와 Wingdings 원형 기호의 실제 윤곽을 대조한다.

현재 Native와 fresh WASM에서 전체 쪽수가 독립 Print와 같고, 두 경로 모두 98.00% 제출 기준을 충족했다.

## 검증 입력과 측정 결과

- 원본 `samples/issue614/issue614-bullet-end-char-shape.hwp` · SHA-256 `f959c73dc62f27da44db58a3c67c1c64d73c2a08784bbd7c99871acfe58b204a`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 45쪽
  `pdf/issue614/issue614-bullet-end-char-shape-2024.pdf` · SHA-256 `5e012fa99ebf7b13d72ccb2b940a3725dbc65da3a0d3585050c01023483f0193`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 45 | 99.55% (4쪽) | 0 |
| fresh WASM | 45 | 99.59% (4쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9218/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/silhouette.tsv)

2쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9218/native/raw-002.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9218/native/pdf-002.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9218/native/review/review_002.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/native/overlay/overlay_002.png)
- 2쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9218/native/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/native/overlay/overlay_002.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/overlay/overlay_002.png)
- 4쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9218/native/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/native/overlay/overlay_004.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/overlay/overlay_004.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 로컬 제품 검증 결과를 기록했다. [통합 PR #9283](https://github.com/edwardkim/rhwp/pull/9283)의 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/44881574506)와 [연계 검사](../assets/planet6897_9208_9236_20261121/integration-code-ci.json)를 통과했다.

## Merge 후 contributor PR comment 계획

참조 없는 글머리표의 굵기와 Wingdings 기호의 윤곽을 확인했습니다.

해당 문서의 데스크톱 한컴 Hancom PDF Print와 최신 후보의 Native·fresh WASM 비교를 기록한다. 통합 병합 SHA에서 다음 쪽의 review·overlay 바이트를 확인한 뒤 원 PR에 표시한다.

- Native · 2쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9218/native/review/review_002.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/native/overlay/overlay_002.png)), 4쪽 99.55% ([review](../assets/planet6897_9208_9236_20261121/cases/9218/native/review/review_004.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/native/overlay/overlay_004.png)).
- fresh WASM · 2쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/review/review_002.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/overlay/overlay_002.png)), 4쪽 99.59% ([review](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/review/review_004.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9218/wasm/overlay/overlay_004.png)).
