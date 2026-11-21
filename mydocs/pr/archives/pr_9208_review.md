# PR #9208 리뷰 — fix(font): 설치되지 않은 TTF 원본을 한컴 기본 대체(한컴바탕)로 표시하고 HFT 한양중고딕 ∙ 를 1em 칸 가운데에 둔다

## 최종 판정

**승인.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9208](https://github.com/edwardkim/rhwp/pull/9208) · planet6897 · devel
- 원 head `7733e209cffbb17829f9bf9d4376de1b4be0dbcc` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `7733e209cffbb17829f9bf9d4376de1b4be0dbcc` → `9cac3be25775192c2ef874a1aabb10fbd33f04aa`

## 원 PR 주장과 확인 범위

한글 원본 TTF가 없을 때의 한컴바탕 선택과 HFT 한양중고딕 가운데점 점유를 대조한다.

문체부 바탕체를 MBatang에 연결했다. 그 프로그램에 누락된 ASCII와 구두점에는 독립 한컴 Print와 같은 한컴바탕 글리프를 공급했다. 12쪽의 줄 끝과 구두점, 주장의 4·5·14·15쪽을 함께 대조한다.

## 검증 입력과 측정 결과

- 원본 `samples/issue547/online-search-rfp.hwp` · SHA-256 `bcc0ac10f2410a85b677efea2ed034789820781bed5379109a987de28168924f`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 22쪽
  `pdf/issue547/online-search-rfp-2024.pdf` · SHA-256 `60c46d0c41bd739a7f3609cf5bee2fc0e2178216ce89a9ee602bdd89512d5885`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 22 | 99.68% (16쪽) | 0 |
| fresh WASM | 22 | 99.68% (16쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9208/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/silhouette.tsv)

4쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9208/native/raw-004.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9208/native/pdf-004.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9208/native/review/review_004.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/native/overlay/overlay_004.png)
- 4쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9208/native/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/native/overlay/overlay_004.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/overlay/overlay_004.png)
- 5쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9208/native/review/review_005.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/native/overlay/overlay_005.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/review/review_005.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/overlay/overlay_005.png)
- 14쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9208/native/review/review_014.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/native/overlay/overlay_014.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/review/review_014.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/overlay/overlay_014.png)
- 15쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9208/native/review/review_015.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/native/overlay/overlay_015.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/review/review_015.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/overlay/overlay_015.png)
- 16쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9208/native/review/review_016.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/native/overlay/overlay_016.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/review/review_016.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9208/wasm/overlay/overlay_016.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 정본 보호 진행과 남은 조건을 기록했다. 최종 CI와 병합 SHA는 확정 후 기록한다.
