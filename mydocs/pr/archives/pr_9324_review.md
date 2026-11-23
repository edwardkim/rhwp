# PR #9324 리뷰 — HWPX 변환 뒤 HWP 저장 줄 규칙 보존

## 최종 판정

원 PR의 조판·저장 주장을 통합 소스에서 확인했다. 전체 회귀와 정본 보호 결과, 실제 화면 검증 상태는 [통합 검증 기록](../assets/planet6897_stored_layout_20261123/README.md)에 연결한다.

## 접수와 검토 범위

[원 PR #9324](https://github.com/edwardkim/rhwp/pull/9324) · [이슈 #9039](https://github.com/edwardkim/rhwp/issues/9039) · 작성자 planet6897

- 원 PR head `23f3393b374d431f0df5bacffc61c45293c0d932`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`

Native CLI 최초 저장의 줄 차이 33건이 0건으로 줄었다. 두 번 저장해 열고 편집한 뒤 표지 값 1이 남았다. fresh WASM 공개 API도 HWPX 보조 항목과 다시 HWP로 저장한 스트림에 1을 보존했다. 값 2 HWP와 한컴 HWPX에는 보조 항목을 만들지 않았다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 통합 적용 커밋 |
| ---: | --- | --- |
| 1 | `dc476a823d4434701071086c1bb23c1a3daca4c1` | `650b31e0dd37af0d031a1ec7a7da1bb8c129355e` |
| 2 | `f7bb473885d95c6cef6017e0d86c4b58571e9991` | `cb2498a7323ff6554e307f41db80eea15e64d0ce` |
| 3 | `23f3393b374d431f0df5bacffc61c45293c0d932` | `39734509110909540e2aae519da0ad1a1f99d88d` |

## 입력과 독립 인쇄 기준

- origin-marker [원본](../../../samples/issue2439_zero_offset_coanchored_float_exclusion.hwp) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue2439/issue2439_zero_offset_coanchored_float_exclusion-2020.pdf).

## Native와 fresh WASM 검증

Visual Sweep의 96 DPI·2px raw 실루엣을 사용했다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

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
