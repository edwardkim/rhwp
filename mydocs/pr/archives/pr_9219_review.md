# PR #9219 리뷰 — fix(paint): 평행 그림자의 잉크 경계는 재생과 같은 이동량을 쓴다 (#9217)

## 최종 판정

**메인터너 보정 후 수용 가능.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9219](https://github.com/edwardkim/rhwp/pull/9219) · planet6897 · devel
- 원 head `78b31661ca5538938965f78b6362da69d5e65fdf` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `22427eadcdf3ba0fba859f5a75a8382bf367c6b5` → `4a136b84f846e50d7e47da3a4a12bdf80c12b14d`
  - `78b31661ca5538938965f78b6362da69d5e65fdf` → `507e4a23c7aef266ed23e144f91d65ab57201acf`

## 원 PR 주장과 확인 범위

평행 그림자의 이동량과 본문 clip 경계를 대조한다.

별도로 발견한 14쪽 사각 글머리표 뒤 약 9.2px 전진 차이를 메인터너가 보정했다. 독립 한컴 2024 Print의 휴먼명조 HFT 기호 전진과 한양중고딕 기호 공급을 대조했다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9217/research-task-order-toc-shadow.hwp` · SHA-256 `242072435d9c1c38ed496a1a3402e16fb904662b395cbe0fc7f3940e8959dd42`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 14쪽
  `pdf/issue9217/research-task-order-toc-shadow-2024.pdf` · SHA-256 `fb918350d1b1a3cb30516235fb714cdbb545d955a7592194e78ce18828bc62b6`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 14 | 99.87% (7쪽) | 0 |
| fresh WASM | 14 | 99.87% (7쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9219/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9219/wasm/silhouette.tsv)

2쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9219/native/raw-002.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9219/native/pdf-002.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9219/native/review/review_002.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9219/native/overlay/overlay_002.png)
- 2쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9219/native/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9219/native/overlay/overlay_002.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9219/wasm/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9219/wasm/overlay/overlay_002.png)
- 7쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9219/native/review/review_007.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9219/native/overlay/overlay_007.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9219/wasm/review/review_007.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9219/wasm/overlay/overlay_007.png)

- 14쪽 사각 기호와 첫 글자의 간격 · Native 100.00% [review](../assets/planet6897_9208_9236_20261121/claim-detail/native/9219/review/review_014.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/native/9219/overlay/overlay_014.png) · fresh WASM 100.00% [review](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9219/review/review_014.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9219/overlay/overlay_014.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 정본 보호 진행과 남은 조건을 기록했다. 최종 CI와 병합 SHA는 확정 후 기록한다.
