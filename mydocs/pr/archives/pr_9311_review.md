# PR #9311 리뷰 — 용지 기준 자리차지 표 위의 빈 줄

## 최종 판정

**승인 — 원 PR의 주장 범위와 통합 후보의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

## 접수와 검토 범위

[원 PR #9311](https://github.com/edwardkim/rhwp/pull/9311) · [이슈 #9274](https://github.com/edwardkim/rhwp/issues/9274) · 작성자 planet6897

- 원 PR head `003ae2c9b65253f2aef52e3cb09a01d5bfec6341`
- 통합 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`
- 리베이스 전 시각·회귀 검증 코드 `330af8608ac741b14041d3e372284a6111346b9c`
- 최신 통합 기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 최신 제품 후보 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`
- 통합 시각 검증 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`

이어지는 문단은 용지 기준 표의 위쪽을 먼저 채운다. 표와 닿는 줄부터 표 아래에 배치한다. 고정 기준과 후보 모두 7쪽의 표와 꼬리말 배치가 한컴 출력과 일치한다.

### 체리픽 적용

| 적용 순서 | 원 커밋 | 최신 리베이스 후 적용 커밋 |
| ---: | --- | --- |
| 1 | `950e73dedbff443d509fe2783a15044f06fc63f0` | `33359a0aa0afa72f6e4f14e6a0ccab2f6caa7e1b` |
| 2 | `003ae2c9b65253f2aef52e3cb09a01d5bfec6341` | `b2de641749cdc72abf15edee929b78c5bf032770` |

## 입력과 독립 인쇄 기준

- decree-band [원본](../../../samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx) 및 [데스크톱 한컴 Print PDF](../../../pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2024.pdf).
- 새 MCP 한컴 2024 Print와 기존 PDF의 96 DPI RGB 래스터는 7쪽 모두 SHA-256이 같다. 기존 기준을 보존한다.

## Native와 fresh WASM 검증

표와 그림은 앞선 검증 SHA의 Visual Sweep 96 DPI·2px raw 실루엣을 기록한다. macOS의 locked WASM 빌드는 `--no-opt`로 수행했다.

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

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
