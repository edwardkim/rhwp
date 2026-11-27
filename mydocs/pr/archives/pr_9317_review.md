# PR #9317 리뷰 — 분할 표의 첫 조각에 속하는 사각형

## 최종 판정

**승인 — 원 PR의 주장 범위와 통합 후보의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

## 접수와 검토 범위

[원 PR #9317](https://github.com/edwardkim/rhwp/pull/9317) · [이슈 #9210](https://github.com/edwardkim/rhwp/issues/9210) · 작성자 planet6897

- 원 PR head `713e673063f17f2b1cd86efff37bb1e1b14224d5`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 최신 제품 후보 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`
- 통합 시각 검증 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`

기존 #9283이 구현한 배치를 두 개의 회귀 검사로 보존한다. 첫 조각 43쪽에 두 사각형이 한 번씩 나타나고, 이어지는 44~46쪽에는 다시 그려지지 않는다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `c788daf82dc08f70a6c1825577369193b58007d8` | `e91660efc02616e36026dc698ba7b44a0b5d351d` |
| 2 | `9cfee854b8db30b1b39f006c4395594e20d58905` | `dcdb60e4f44ae25a4a58f7d14bc0998fd86570f3` |
| 3 | `033812d7879506df24c14a0ca74fbcd143a1c229` | `0ad6c4325edcd51ff3164f8ff7c81a7a4032af02` |
| 4 | `713e673063f17f2b1cd86efff37bb1e1b14224d5` | `d191c3c1aabff3c858f6e2a0a9d2357cc0ed8376` |

## 입력과 독립 인쇄 기준

- sme-overlay [원본](../../../samples/issue9038/sme_announcement_split_table_successor.hwpx) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue9210/sme_announcement_split_table_successor-2020.pdf).
- MCP 영수증의 엔진은 2020이다. PDF 메타데이터에 표시된 빌드 11.0.0.1623은 영수증의 빌드와 같다.

## Native와 fresh WASM 검증

표와 그림은 앞선 검증 SHA의 Visual Sweep 96 DPI·2px raw 실루엣을 기록한다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

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

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

17쪽은 #9365가 반영된 최신 기준과 통합 후보의 PNG가 Native·fresh WASM 모두 일치했다. 해당 쪽 raw 값은 각각 94.46614%, 94.34458%다. 43~46쪽의 사각형 소속 검증 범위는 그대로 유지한다.

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
