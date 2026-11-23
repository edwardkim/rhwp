# PR #9317 리뷰 — 분할 표의 첫 조각에 속하는 사각형

## 최종 판정

원 PR의 조판·저장 주장을 통합 소스에서 확인했다. 전체 회귀와 정본 보호 결과, 실제 화면 검증 상태는 [통합 검증 기록](../assets/planet6897_stored_layout_20261123/README.md)에 연결한다.

## 접수와 검토 범위

[원 PR #9317](https://github.com/edwardkim/rhwp/pull/9317) · [이슈 #9210](https://github.com/edwardkim/rhwp/issues/9210) · 작성자 planet6897

- 원 PR head `713e673063f17f2b1cd86efff37bb1e1b14224d5`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`

기존 #9283이 구현한 배치를 두 개의 회귀 검사로 보존한다. 첫 조각 43쪽에 두 사각형이 한 번씩 나타나고, 이어지는 44~46쪽에는 다시 그려지지 않는다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 통합 적용 커밋 |
| ---: | --- | --- |
| 1 | `c788daf82dc08f70a6c1825577369193b58007d8` | `20075f7e83b877153241bddbefa52ef3a67615c4` |
| 2 | `9cfee854b8db30b1b39f006c4395594e20d58905` | `8456981fe3ae388cca591a22ccaad1ed7ee0c084` |
| 3 | `033812d7879506df24c14a0ca74fbcd143a1c229` | `7a815c14bb36281ea28f356f04faa6947e405ac7` |
| 4 | `713e673063f17f2b1cd86efff37bb1e1b14224d5` | `d86d916960212196e8c89e41042f3b7366437ffb` |

## 입력과 독립 인쇄 기준

- sme-overlay [원본](../../../samples/issue9038/sme_announcement_split_table_successor.hwpx) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue9210/sme_announcement_split_table_successor-2020.pdf).
- MCP 영수증의 엔진은 2020이다. PDF 메타데이터에 표시된 빌드 11.0.0.1623은 영수증의 빌드와 같다.

## Native와 fresh WASM 검증

Visual Sweep의 96 DPI·2px raw 실루엣을 사용했다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

| 원본·쪽 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | ---: | ---: |
| sme-overlay 43쪽 | 98.37924% → 98.37924% | 98.23999% → 98.23999% |
| sme-overlay 44쪽 | 99.96246% → 99.96246% | 99.96243% → 99.96243% |
| sme-overlay 45쪽 | 98.51954% → 98.51954% | 98.51825% → 98.51825% |
| sme-overlay 46쪽 | 99.77630% → 99.77630% | 99.77623% → 99.77623% |

- sme-overlay native 전쪽 46/46쪽, 최저 98.37924%(43쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/silhouette.tsv)
- sme-overlay native [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/base-silhouette.tsv).

- sme-overlay wasm 전쪽 46/46쪽, 최저 98.23999%(43쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/silhouette.tsv)
- sme-overlay wasm [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/base-silhouette.tsv).

## 주장 페이지의 증적

- sme-overlay 43쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/raw-043.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/pdf-043.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/review-043.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/overlay-043.png)
- sme-overlay 44쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/raw-044.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/pdf-044.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/review-044.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/overlay-044.png)
- sme-overlay 45쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/raw-045.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/pdf-045.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/review-045.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/overlay-045.png)
- sme-overlay 46쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/raw-046.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/pdf-046.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/review-046.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/native/overlay-046.png)
- sme-overlay 43쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/raw-043.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/pdf-043.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/review-043.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/overlay-043.png)
- sme-overlay 44쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/raw-044.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/pdf-044.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/review-044.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/overlay-044.png)
- sme-overlay 45쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/raw-045.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/pdf-045.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/review-045.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/overlay-045.png)
- sme-overlay 46쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/raw-046.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/pdf-046.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/review-046.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9317/sme-overlay/wasm/overlay-046.png)

## Merge 후 contributor PR comment 계획

실제 merge와 원 PR head를 확인한 뒤 [분할 표의 첫 조각에 속하는 사각형](https://github.com/edwardkim/rhwp/pull/9317)에 기여 내용과 검증을 한국어 존댓말로 남긴다. 원 PR의 문서와 주장 페이지에 대응하는 실제 review와 overlay 이미지를 merge SHA의 raw URL로 표시한다.
