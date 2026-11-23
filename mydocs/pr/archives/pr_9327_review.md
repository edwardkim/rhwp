# PR #9327 리뷰 — 캡션 그림과 TAC 그림 줄의 쪽 소속

## 최종 판정

**승인 — 원 PR의 주장 범위와 통합 후보의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

## 접수와 검토 범위

[원 PR #9327](https://github.com/edwardkim/rhwp/pull/9327) · [이슈 #8997](https://github.com/edwardkim/rhwp/issues/8997) · 작성자 planet6897

- 원 PR head `39d018f7a363910d0cd3a483a078214b66a6b423`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 최신 제품 후보 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`
- 통합 시각 검증 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`

식량안보 보고서의 로고는 1쪽, 목차 표는 2쪽에 위치한다. 표 2-6은 96쪽에서 2008년 행까지 포함하며, 97쪽은 2009년부터 이어진다. 전체 141쪽은 한컴 Print와 같다. 90~93쪽의 별도 흐름 문제는 #8997에서 계속 추적한다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `ee3bc54011e3eccaf2a25196b57eff6925f9d52e` | `012af37c897a03fa4ebffc6a5634d78bc7996761` |
| 2 | `b0d67cea936ca6ac7cd8e2bfdc122a750ef16dc8` | `25b32a5f0b2eaed53b3306d278e134efc169e9eb` |
| 3 | `0dd9b3020765c281289a07f580404965867a3d0a` | `8b68859efb4de337d874669ed05d63f859904e98` |
| 4 | `39d018f7a363910d0cd3a483a078214b66a6b423` | `33db88891ff35b56a877b4f35b229971dabcdb77` |

## 입력과 독립 인쇄 기준

- food [원본](../../../mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue8997/1543000-202100035-food-security-report-2020.pdf).
- jeju [원본](../../../samples/issue8759/6500000-201700118_jeju_tourism_education_plan.hwp) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue8759/6500000-201700118_jeju_tourism_education_plan-2020.pdf).

## Native와 fresh WASM 검증

표와 그림은 앞선 검증 SHA의 Visual Sweep 96 DPI·2px raw 실루엣을 기록한다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

| 원본·쪽 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | ---: | ---: |
| food 1쪽 | 76.77231% → 99.99388% | 76.97688% → 99.99394% |
| food 2쪽 | 10.02561% → 99.83921% | 10.02561% → 98.08753% |
| food 97쪽 | 35.38049% → 98.88830% | 34.92259% → 98.89185% |
| food 98쪽 | 24.65285% → 99.99597% | 24.33916% → 99.99602% |
| jeju 240쪽 | 98.73318% → 100.00000% | 98.69852% → 100.00000% |
| jeju 244쪽 | 97.42511% → 100.00000% | 97.46749% → 100.00000% |

- food native 전쪽 141/141쪽, 최저 24.29087%(93쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/silhouette.tsv)
  90% 미만 9쪽. 기준의 내용 소속 대조는 별도로 확인한다.

- food wasm 전쪽 141/141쪽, 최저 23.83372%(93쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/silhouette.tsv)
  90% 미만 9쪽. 기준의 내용 소속 대조는 별도로 확인한다.

- jeju native 전쪽 313/313쪽, 최저 42.73007%(49쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/silhouette.tsv)
  90% 미만 9쪽. 기준의 내용 소속 대조는 별도로 확인한다.
- jeju native [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/base-silhouette.tsv).

- jeju wasm 전쪽 313/313쪽, 최저 43.68676%(49쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/silhouette.tsv)
  90% 미만 9쪽. 기준의 내용 소속 대조는 별도로 확인한다.
- jeju wasm [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/base-silhouette.tsv).

## 주장 페이지의 증적

- food 1쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/raw-001.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/pdf-001.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/review-001.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/overlay-001.png)
- food 2쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/raw-002.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/pdf-002.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/review-002.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/overlay-002.png)
- food 97쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/raw-097.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/pdf-097.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/review-097.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/overlay-097.png)
- food 98쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/raw-098.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/pdf-098.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/review-098.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/overlay-098.png)
- food 1쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/raw-001.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/pdf-001.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/review-001.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/overlay-001.png)
- food 2쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/raw-002.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/pdf-002.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/review-002.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/overlay-002.png)
- food 97쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/raw-097.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/pdf-097.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/review-097.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/overlay-097.png)
- food 98쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/raw-098.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/pdf-098.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/review-098.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/overlay-098.png)
- jeju 240쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/raw-240.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/pdf-240.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/review-240.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/overlay-240.png)
- jeju 244쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/raw-244.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/pdf-244.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/review-244.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/native/overlay-244.png)
- jeju 240쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/raw-240.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/pdf-240.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/review-240.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/overlay-240.png)
- jeju 244쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/raw-244.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/pdf-244.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/review-244.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/jeju/wasm/overlay-244.png)
- 식량안보 96쪽 첫 조각의 2008년 행 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/raw-096.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/pdf-096.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/review-096.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/native/overlay-096.png).
- 식량안보 96쪽 첫 조각의 2008년 행 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/raw-096.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/pdf-096.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/review-096.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9327/food/wasm/overlay-096.png).

## Merge 후 contributor PR comment 계획

실제 merge와 원 PR head를 확인한 뒤 [캡션 그림과 TAC 그림 줄의 쪽 소속](https://github.com/edwardkim/rhwp/pull/9327)에 기여 내용과 검증을 한국어 존댓말로 남긴다. 원 PR의 문서와 주장 페이지에 대응하는 실제 review와 overlay 이미지를 merge SHA의 raw URL로 표시한다.

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
