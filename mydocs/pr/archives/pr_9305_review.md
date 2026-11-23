# PR #9305 리뷰 — 표가 여는 쪽의 저장 줄 원점

## 최신 통합 상태

#9365와 문서 변경 #9369를 포함한 `devel` 위의 적용 커밋을 기록했다. 집중 회귀 16건이 통과했으며, 최종 회귀와 시각 검증은 진행 중이다.

## 리베이스 전 검토 판정

원 PR의 조판·저장 주장을 통합 소스에서 확인했다. 전체 회귀와 정본 보호 결과, 실제 화면 검증 상태는 [통합 검증 기록](../assets/planet6897_stored_layout_20261123/README.md)에 연결한다.

## 접수와 검토 범위

[원 PR #9305](https://github.com/edwardkim/rhwp/pull/9305) · [이슈 #9290](https://github.com/edwardkim/rhwp/issues/9290) · 작성자 planet6897

- 원 PR head `57295596c6cd1d30cf181b22c450f3975d59031d`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `e7b45a7a71e9d038bd137a9dc2ac3ef37e0ac2e5`
- 최신 제품 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

3쪽 첫 저장 줄의 1500HU는 문단 앞 간격이다. 기준에서는 이를 쪽 원점으로 사용해 표 아래의 줄이 20px 올라갔다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `db6a7aedbe0b7c0ef4344b025e6d721bccb7946f` | `f6829cab07b85825c92fc080ae6ae19d0049aee7` |
| 2 | `57295596c6cd1d30cf181b22c450f3975d59031d` | `e2f74e5351010465ebeedaa71a382a807c85f07b` |

## 입력과 독립 인쇄 기준

- townhall [원본](../../../samples/issue9290/labor-townhall-table-opened-page.hwpx) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue9290/labor-townhall-table-opened-page-2024.pdf).

## Native와 fresh WASM 검증

Visual Sweep의 96 DPI·2px raw 실루엣을 사용했다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

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
