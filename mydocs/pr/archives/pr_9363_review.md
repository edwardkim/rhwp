# PR #9363 리뷰 — 저장 사다리가 남긴 쪽 머리 빈 줄

[원 PR #9363](https://github.com/edwardkim/rhwp/pull/9363) · [이슈 #9291](https://github.com/edwardkim/rhwp/issues/9291)

- 원 head `191827eafff500c6717247afc6cfc6e9b27b8161`
- 최신 devel `e7b45a7a71e9d038bd137a9dc2ac3ef37e0ac2e5`
- 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

## 체리픽 출처

| 원 커밋 | 리베이스 후 적용 커밋 |
| --- | --- |
| `5e79baa09edabe9fdd51ec8f6f2450f14ef3f429` | `043fada86bd419e3a93019e3b3e318533f644845` |
| `fe4208c97d47f7640afb76b2ccf8b8160e4af779` | `3b04cb55eead2ce1c4d735bb56dc357267bcc07f` |
| `191827eafff500c6717247afc6cfc6e9b27b8161` | `39ac206e989ea31f516ee8dac6d77ba0d2f86b1e` |

## 검토 범위

저장 사다리가 전진을 계상한 쪽 첫 빈 줄을 유지한다. 다음 제목을 빈 줄과 같은 위치에 둔 반례에서는 빈 줄을 숨긴다. HWP3 파서가 만든 합성 위치는 근거에서 제외한다.

[통계청 원본](../../../samples/issue9291/consumer-price-trend-page-start-empty-line.hwp)과 [한컴 2024 Print](../../../pdf/issue9291/consumer-price-trend-page-start-empty-line-2024.pdf)는 18쪽이다. 원 PR의 인쇄본은 Creator가 한컴 2022여서 동일한 입력을 MCP에서 한컴 2024로 다시 인쇄했다. [인쇄 영수증](../assets/planet6897_stored_layout_20261123/pr-9363/hancom-print.json)

## 확인한 검증 결과

- 최신 제품 후보 `8b54abbd12eaff27b004571721ae81885b828201`의 집중 회귀 16건이 통과했다. 이 PR의 회귀 2건을 포함한다.
- 리베이스 전 후보 `1b06db582f5209bba290ea6de67653acf0d7ac12`의 Native raw 일치율은 12쪽 99.85%, 18쪽 99.81%였다. 한컴 2024 Print, 96 DPI, compat 2024로 측정했다.
- 최신 후보의 Native·fresh WASM 전쪽 TSV와 주장 페이지의 직접 판독 결과는 검증 완료 뒤 반영한다.
