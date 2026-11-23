# PR #9305 리뷰 — 표가 여는 쪽의 저장 줄 원점

## 최종 판정

**승인 — 원 PR의 주장 범위와 통합 후보의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

## 접수와 검토 범위

[원 PR #9305](https://github.com/edwardkim/rhwp/pull/9305) · [이슈 #9290](https://github.com/edwardkim/rhwp/issues/9290) · 작성자 planet6897

- 원 PR head `57295596c6cd1d30cf181b22c450f3975d59031d`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 최신 제품 후보 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`
- 통합 시각 검증 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`

3쪽 첫 저장 줄의 1500HU는 문단 앞 간격이다. 기준에서는 이를 쪽 원점으로 사용해 표 아래의 줄이 20px 올라갔다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `db6a7aedbe0b7c0ef4344b025e6d721bccb7946f` | `d342027aa913bd4cdc5ec1051c8819e5372fb8ae` |
| 2 | `57295596c6cd1d30cf181b22c450f3975d59031d` | `9d5296c5d2e265fe9a019ee377e56cb127bceb03` |

## 입력과 독립 인쇄 기준

- townhall [원본](../../../samples/issue9290/labor-townhall-table-opened-page.hwpx) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue9290/labor-townhall-table-opened-page-2024.pdf).

## Native와 fresh WASM 검증

표와 그림은 앞선 검증 SHA의 Visual Sweep 96 DPI·2px raw 실루엣을 기록한다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

| 원본·쪽 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | ---: | ---: |
| townhall 3쪽 | 29.82489% → 97.31994% | 29.58628% → 97.20314% |

- townhall native 전쪽 3/3쪽, 최저 97.31994%(3쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/native/silhouette.tsv)
- townhall native [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/native/base-silhouette.tsv).

- townhall wasm 전쪽 3/3쪽, 최저 97.20314%(3쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/wasm/silhouette.tsv)
- townhall wasm [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/wasm/base-silhouette.tsv).

## 주장 페이지의 증적

- townhall 3쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/native/raw-003.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/native/pdf-003.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/native/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/native/overlay-003.png)
- townhall 3쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/wasm/raw-003.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/wasm/pdf-003.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/wasm/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9305/townhall/wasm/overlay-003.png)

## Merge 후 contributor PR comment 계획

실제 merge와 원 PR head를 확인한 뒤 [표가 여는 쪽의 저장 줄 원점](https://github.com/edwardkim/rhwp/pull/9305)에 기여 내용과 검증을 한국어 존댓말로 남긴다. 원 PR의 문서와 주장 페이지에 대응하는 실제 review와 overlay 이미지를 merge SHA의 raw URL로 표시한다.

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
