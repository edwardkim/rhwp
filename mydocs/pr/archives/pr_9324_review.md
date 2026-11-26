# PR #9324 리뷰 — HWPX 변환 뒤 HWP 저장 줄 규칙 보존

## 최종 판정

**승인 — 원 PR의 주장 범위와 통합 후보의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

## 접수와 검토 범위

[원 PR #9324](https://github.com/edwardkim/rhwp/pull/9324) · [이슈 #9039](https://github.com/edwardkim/rhwp/issues/9039) · 작성자 planet6897

- 원 PR head `23f3393b374d431f0df5bacffc61c45293c0d932`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 최신 제품 후보 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`
- 통합 시각 검증 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`

Native CLI 최초 저장의 줄 차이 33건이 0건으로 줄었다. 두 번 저장해 열고 편집한 뒤 표지 값 1이 남았다. fresh WASM 공개 API도 HWPX 보조 항목과 다시 HWP로 저장한 스트림에 1을 보존했다. 값 2 HWP와 한컴 HWPX에는 보조 항목을 만들지 않았다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `dc476a823d4434701071086c1bb23c1a3daca4c1` | `cc11c40ca8660960c716db0e308c7de52a9d0c51` |
| 2 | `f7bb473885d95c6cef6017e0d86c4b58571e9991` | `f0d319f24e4deec8c2085273fd2c06a06200a77a` |
| 3 | `23f3393b374d431f0df5bacffc61c45293c0d932` | `675736fbaecb3d4a5c0375397d40d3919071fc3e` |

## 입력과 독립 인쇄 기준

- origin-marker [원본](../../../samples/issue2439_zero_offset_coanchored_float_exclusion.hwp) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue2439/issue2439_zero_offset_coanchored_float_exclusion-2020.pdf).

## Native와 fresh WASM 검증

표와 그림은 앞선 검증 SHA의 Visual Sweep 96 DPI·2px raw 실루엣을 기록한다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

| 원본·쪽 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | ---: | ---: |
| origin-marker 1쪽 | 97.71407% → 97.71407% | 97.67650% → 97.67650% |

- origin-marker native 전쪽 1/1쪽, 최저 97.71407%(1쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/native/silhouette.tsv)
- origin-marker native [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/native/base-silhouette.tsv).

- origin-marker wasm 전쪽 1/1쪽, 최저 97.67650%(1쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/wasm/silhouette.tsv)
- origin-marker wasm [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/wasm/base-silhouette.tsv).

## 주장 페이지의 증적

- origin-marker 1쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/native/raw-001.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/native/pdf-001.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/native/review-001.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/native/overlay-001.png)
- origin-marker 1쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/wasm/raw-001.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/wasm/pdf-001.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/wasm/review-001.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9324/origin-marker/wasm/overlay-001.png)

## Merge 후 contributor PR comment 계획

실제 merge와 원 PR head를 확인한 뒤 [HWPX 변환 뒤 HWP 저장 줄 규칙 보존](https://github.com/edwardkim/rhwp/pull/9324)에 기여 내용과 검증을 한국어 존댓말로 남긴다. 원 PR의 문서와 주장 페이지에 대응하는 실제 review와 overlay 이미지를 merge SHA의 raw URL로 표시한다.

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
