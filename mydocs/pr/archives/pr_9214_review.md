# PR #9214 리뷰 — fix(layout): 여러 문단 캡션의 높이는 닫힌 저장 줄 사다리의 끝을 쓴다 (#9213)

## 최종 판정

**머지 보류.** 원본 전쪽 Native·fresh WASM과 최신 후보 전체 회귀를 통과했다. 통합 후보의 정본 보호 3조건을 검증한 뒤 제출 여부를 확정한다.

## 접수 정보

- [원 PR #9214](https://github.com/edwardkim/rhwp/pull/9214) · planet6897 · devel
- 원 head `bcd60368a7c8838e2665e1734bb61a272c80cf69` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `7b18590a963e8ddf7a4f64882ab9d77e2599d134` → `eacc1dfe4d7020a353a39bf912c977e04b3d0172`
  - `bcd60368a7c8838e2665e1734bb61a272c80cf69` → `81ed77c81f6674d26d8911ce32675cb79f2417e8`

## 원 PR 주장과 확인 범위

두 문단 캡션의 저장 줄 위치와 위아래 표 경계를 대조한다.

메인터너가 바탕체의 결측 U+2027을 독립 Print와 같은 한컴바탕에서 공급하고, 표 안 차트와 오른쪽 정렬 QR의 기준을 보정했다. 원 PR의 15·30쪽 캡션 경계와 별도로 차트·QR이 있는 쪽도 확인한다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9213/labor-survey-2022h1-caption-ladder.hwpx` · SHA-256 `d1d618c0a38d0efdb3348d21ec81fc400780a8c801ccbbc367c8ea08b495482d`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 32쪽
  `pdf/issue9213/labor-survey-2022h1-caption-ladder-2024.pdf` · SHA-256 `cd9086f7c205e39a52d427b5559ac04fd3331650d6f0436c807eeb8a3a88bf78`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 32 | 99.20% (4쪽) | 0 |
| fresh WASM | 32 | 99.18% (4쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9214/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/silhouette.tsv)

30쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9214/native/raw-030.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9214/native/pdf-030.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9214/native/review/review_030.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/native/overlay/overlay_030.png)
- 4쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9214/native/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/native/overlay/overlay_004.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/overlay/overlay_004.png)
- 15쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9214/native/review/review_015.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/native/overlay/overlay_015.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/review/review_015.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/overlay/overlay_015.png)
- 30쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9214/native/review/review_030.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/native/overlay/overlay_030.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/review/review_030.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9214/wasm/overlay/overlay_030.png)

- 1쪽 표 안 차트 · Native 100.00% [review](../assets/planet6897_9208_9236_20261121/claim-detail/native/9214/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/native/9214/overlay/overlay_001.png) · fresh WASM 100.00% [review](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9214/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9214/overlay/overlay_001.png)
- 8쪽 오른쪽 정렬 QR · Native 99.73% [review](../assets/planet6897_9208_9236_20261121/claim-detail/native/9214/review/review_008.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/native/9214/overlay/overlay_008.png) · fresh WASM 99.73% [review](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9214/review/review_008.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9214/overlay/overlay_008.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 정본 보호 진행과 남은 조건을 기록했다. 최종 CI와 병합 SHA는 확정 후 기록한다.
