# PR #9363 리뷰 — 저장 사다리가 남긴 쪽 머리 빈 줄

## 최종 판정

**메인터너 보정 후 수용 가능 — #9291 해결 범위의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

[원 PR #9363](https://github.com/edwardkim/rhwp/pull/9363) · [이슈 #9291](https://github.com/edwardkim/rhwp/issues/9291)

- 원 head `191827eafff500c6717247afc6cfc6e9b27b8161`
- 최신 devel `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

## 체리픽 출처

| 원 커밋 | 리베이스 후 적용 커밋 |
| --- | --- |
| `5e79baa09edabe9fdd51ec8f6f2450f14ef3f429` | `406f16656789ac505766cd08debb81d5fa82f7ec` |
| `fe4208c97d47f7640afb76b2ccf8b8160e4af779` | `3485c0ffde21033957926dd1ebe90523e75dcf90` |
| `191827eafff500c6717247afc6cfc6e9b27b8161` | `9f8c5e7069078d9add44345192a3b5c0bdb9cc13` |

## 검토 범위

저장 사다리가 전진을 계상한 쪽 첫 빈 줄을 유지한다. 다음 제목을 빈 줄과 같은 위치에 둔 반례에서는 빈 줄을 숨긴다. HWP3 파서가 만든 합성 위치는 근거에서 제외한다.

[통계청 원본](../../../samples/issue9291/consumer-price-trend-page-start-empty-line.hwp)과 [한컴 2024 Print](../../../pdf/issue9291/consumer-price-trend-page-start-empty-line-2024.pdf)는 18쪽이다. 원 PR의 인쇄본은 Creator가 한컴 2022여서 동일한 입력을 MCP에서 한컴 2024로 다시 인쇄했다. [인쇄 영수증](../assets/planet6897_stored_layout_20261123/pr-9363/hancom-print.json)

## 확인한 검증 결과

- 최신 제품 후보 `8b54abbd12eaff27b004571721ae81885b828201`의 집중 회귀 16건이 통과했다. 이 PR의 회귀 2건을 포함한다.
- 리베이스 전 후보 `1b06db582f5209bba290ea6de67653acf0d7ac12`의 Native raw 일치율은 12쪽 99.85%, 18쪽 99.81%였다. 한컴 2024 Print, 96 DPI, compat 2024로 측정했다.

## 최신 후보 전쪽 비교와 직접 판독

측정 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`. Native 바이너리는 제품 소스가 같은 직전 커밋에서 빌드했고, fresh WASM은 이 SHA에서 macOS locked wrapper의 `--no-opt`로 빌드했다. 경로별로 96 DPI 및 2px raw 실루엣을 사용했다.

| 경로·쪽 | 기준 → 후보 | 직접 판독 자료 |
| --- | ---: | --- |
| native 12쪽 | 61.04% → 99.85% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/raw-012.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/pdf-012.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/review-012.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/overlay-012.png) |
| native 18쪽 | 30.20% → 99.81% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/raw-018.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/pdf-018.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/review-018.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/overlay-018.png) |
| wasm 12쪽 | 61.33% → 99.82% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/raw-012.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/pdf-012.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/review-012.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/overlay-012.png) |
| wasm 18쪽 | 30.14% → 99.79% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/raw-018.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/pdf-018.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/review-018.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/overlay-018.png) |

- native 전쪽 18쪽, 기준 대비 하락 0쪽. [기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/base-silhouette.tsv) · [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/native/silhouette.tsv)
- wasm 전쪽 18쪽, 기준 대비 하락 0쪽. [기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/base-silhouette.tsv) · [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9363/consumer-price/wasm/silhouette.tsv)

표와 뒤따르는 설명이 한컴 Print와 같은 쪽에 있다. 표 안 글자 윤곽에는 미세한 차이가 보인다. 범위 밖 5쪽은 기준과 후보 PNG의 픽셀이 각 backend에서 동일하고, Native 57.54%, WASM 57.52%다.

원 PR의 한컴 2022 출력 대신 같은 입력을 한컴 2024 MCP의 Hancom PDF Print로 다시 출력했다. 그 기준의 전쪽 대조와 아래 증적을 사용한다.

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## Merge 후 contributor PR comment 계획

실제 통합 PR과 병합 SHA를 확인한 뒤 입증한 주장과 보정 범위를 존댓말로 안내한다. 같은 원본의 해당 쪽 review·overlay를 병합 SHA 고정 raw URL로 게시한다.

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
