# PR #9364 리뷰 — 음수 줄 간격과 표 행의 쪽 소속

[원 PR #9364](https://github.com/edwardkim/rhwp/pull/9364) · [이슈 #9353](https://github.com/edwardkim/rhwp/issues/9353)

- 원 head `7936bc78dc8f4510edde88693e397b6fc91bdbbd`
- 최신 devel `e7b45a7a71e9d038bd137a9dc2ac3ef37e0ac2e5`
- 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

## 체리픽 출처

| 원 커밋 | 리베이스 후 적용 커밋 |
| --- | --- |
| `ba643ad1b01d58cc056cfd8bb95ca06072212ca8` | `bdb032d103c1e4076dcc7ba27ebf6f35b7de8867` |
| `4ed8d51dc5658a023593e6eec9bd18e2ffeee63b` | `dfb11e1c003fcfb944aa70cc1a811cf44eaeb2d9` |
| `d206c337b606d8bb181180dd88baa64fa012ab7d` | `66efe15a142d170b60ea12dab6235f5a19f953b1` |
| `7936bc78dc8f4510edde88693e397b6fc91bdbbd` | `6f522d5548f84496e137272378f634ecdba7b484` |

## 검토 범위

음수 줄 간격으로 앞 줄의 저장 전진에 놓인 다음 줄을 쪽 되감김으로 읽지 않는다. 두 줄 칸 전체가 2쪽에 남고 3쪽은 다음 행부터 시작한다.

[의왕시 원본](../../../samples/issue9353/uiwang_lane_min_length.hwp)과 [한컴 2024 Print](../../../pdf/issue9353/uiwang_lane_min_length-2024.pdf)는 3쪽이다. 원 PR 기준 PDF의 Creator·Producer·페이지 수와 SHA-256을 확인했다.

## 확인한 검증 결과

- 최신 제품 후보 `8b54abbd12eaff27b004571721ae81885b828201`의 집중 회귀 16건이 통과했다. 이 PR의 회귀 2건을 포함한다.
- 리베이스 전 후보 `1b06db582f5209bba290ea6de67653acf0d7ac12`의 Native raw 일치율은 2쪽 96.68%, 3쪽 97.67%였다. 한컴 2024 Print, 96 DPI, compat 2024로 측정했다.
- 최신 후보의 Native·fresh WASM 전쪽 TSV와 주장 페이지의 직접 판독 결과는 검증 완료 뒤 반영한다.
