# PR #9311 리뷰 — 용지 기준 자리차지 표 위의 빈 줄

## 최신 통합 상태

#9365와 문서 변경 #9369를 포함한 `devel` 위의 적용 커밋을 기록했다. 집중 회귀 16건이 통과했으며, 최종 회귀와 시각 검증은 진행 중이다.

## 리베이스 전 검토 판정

원 PR의 조판·저장 주장을 통합 소스에서 확인했다. 전체 회귀와 정본 보호 결과, 실제 화면 검증 상태는 [통합 검증 기록](../assets/planet6897_stored_layout_20261123/README.md)에 연결한다.

## 접수와 검토 범위

[원 PR #9311](https://github.com/edwardkim/rhwp/pull/9311) · [이슈 #9274](https://github.com/edwardkim/rhwp/issues/9274) · 작성자 planet6897

- 원 PR head `003ae2c9b65253f2aef52e3cb09a01d5bfec6341`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `e7b45a7a71e9d038bd137a9dc2ac3ef37e0ac2e5`
- 최신 제품 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

이어지는 문단은 용지 기준 표의 위쪽을 먼저 채운다. 표와 닿는 줄부터 표 아래에 배치한다. 고정 기준과 후보 모두 7쪽의 표와 꼬리말 배치가 한컴 출력과 일치한다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `950e73dedbff443d509fe2783a15044f06fc63f0` | `a9268a5e18f18c151ff86185a5089cf4433ebccc` |
| 2 | `003ae2c9b65253f2aef52e3cb09a01d5bfec6341` | `be41cbc3e8519e139c8addfb828628448aa412f5` |

## 입력과 독립 인쇄 기준

- decree-band [원본](../../../samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2024.pdf).
- 새 MCP 한컴 2024 Print와 기존 PDF의 96 DPI RGB 래스터는 7쪽 모두 SHA-256이 같다. 기존 기준을 보존한다.

## Native와 fresh WASM 검증

Visual Sweep의 96 DPI·2px raw 실루엣을 사용했다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

| 원본·쪽 | Native 기준 → 후보 | fresh WASM 기준 → 후보 |
| --- | ---: | ---: |
| decree-band 7쪽 | 100.00000% → 100.00000% | 100.00000% → 100.00000% |

- decree-band native 전쪽 7/7쪽, 최저 99.98220%(1쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/native/silhouette.tsv)
- decree-band native [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/native/base-silhouette.tsv).

- decree-band wasm 전쪽 7/7쪽, 최저 99.98220%(1쪽). [TSV](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/wasm/silhouette.tsv)
- decree-band wasm [동일 입력의 기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/wasm/base-silhouette.tsv).

## 주장 페이지의 증적

- decree-band 7쪽 native [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/native/raw-007.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/native/pdf-007.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/native/review-007.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/native/overlay-007.png)
- decree-band 7쪽 wasm [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/wasm/raw-007.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/wasm/pdf-007.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/wasm/review-007.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9311/decree-band/wasm/overlay-007.png)

## Merge 후 contributor PR comment 계획

실제 merge와 원 PR head를 확인한 뒤 [용지 기준 자리차지 표 위의 빈 줄](https://github.com/edwardkim/rhwp/pull/9311)에 기여 내용과 검증을 한국어 존댓말로 남긴다. 원 PR의 문서와 주장 페이지에 대응하는 실제 review와 overlay 이미지를 merge SHA의 raw URL로 표시한다.
