# PR #9236 리뷰 — fix(render): 정렬 배치 그림 채우기는 영역보다 큰 축을 영역 크기로 줄여 그린다 (#9233)

## 최종 판정

**머지 보류.** 원본 전쪽 Native·fresh WASM과 최신 후보 전체 회귀를 통과했다. 통합 후보의 정본 보호 3조건을 검증한 뒤 제출 여부를 확정한다.

## 접수 정보

- [원 PR #9236](https://github.com/edwardkim/rhwp/pull/9236) · planet6897 · devel
- 원 head `0312407308e6689f04c99a40ac35a744ef262857` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `9b904d5de82e8a541385cd0bd7afdd58eafe8349` → `74f5ca5853085e5b8972425791b3d76e035f33f0`
  - `0312407308e6689f04c99a40ac35a744ef262857` → `328d6566bb72439f376c3f1b3eb599a29d35b0be`

## 원 PR 주장과 확인 범위

표지 제목 위아래 곡선 띠의 전체 모양과 도형 경계를 대조한다.

원본 Print의 자동 꼬리말이 본문과 별도로 출력된다. 설정 한 항목만 바꾼 대조군에서 PDF 차이는 꼬리말 띠에만 있었다. 인쇄 작업의 실제 파일명·현지 시각·쪽 번호와 저장된 자동 바닥글 설정을 공통 Print 레이어에서 사용했다. 원본 HWP와 독립 PDF의 바이트를 고정하고 22쪽을 다시 비교했다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9233/high-school-transfer-plan-cover-band.hwp` · SHA-256 `70189a97083aa0046594f0c2bfc985888a607060414245d62235e09796008314`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 22쪽
  `pdf/issue9233/high-school-transfer-plan-cover-band-2024.pdf` · SHA-256 `8f07d75ed293f723c5d622fb6c5230cf5e52e751b58f85316d341cd7ce849d30`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 22 | 98.56% (20쪽) | 0 |
| fresh WASM | 22 | 98.51% (20쪽) | 0 |

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9236/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9236/wasm/silhouette.tsv)

1쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9236/native/raw-001.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9236/native/pdf-001.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9236/native/review/review_001.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9236/native/overlay/overlay_001.png)
- 1쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9236/native/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9236/native/overlay/overlay_001.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9236/wasm/review/review_001.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9236/wasm/overlay/overlay_001.png)
- 20쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9236/native/review/review_020.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9236/native/overlay/overlay_020.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9236/wasm/review/review_020.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9236/wasm/overlay/overlay_020.png)

- 2쪽 인쇄 작업의 자동 바닥글 · Native 100.00% [review](../assets/planet6897_9208_9236_20261121/claim-detail/native/9236/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/native/9236/overlay/overlay_002.png) · fresh WASM 100.00% [review](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9236/review/review_002.png) [overlay](../assets/planet6897_9208_9236_20261121/claim-detail/wasm/9236/overlay/overlay_002.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 정본 보호 진행과 남은 조건을 기록했다. 최종 CI와 병합 SHA는 확정 후 기록한다.

Studio에서 인쇄 미리보기 22쪽과 닫은 뒤 편집 상태 복원을 확인했다. 브라우저가 초기화에 사용한 JS·WASM 해시를 기록했다. [실제 Studio 검증](../assets/planet6897_9208_9236_20261121/current-studio-consumption.json).
