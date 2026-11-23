# PR #9364 리뷰 — 음수 줄 간격과 표 행의 쪽 소속

## 최종 판정

**메인터너 보정 후 수용 가능 — #9353 해결 범위의 로컬 검증 완료.** 통합 제품 검증 소스는 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 병합은 제출한 head에서 GitHub CI와 충돌 여부를 확인한 뒤 진행한다.

[원 PR #9364](https://github.com/edwardkim/rhwp/pull/9364) · [이슈 #9353](https://github.com/edwardkim/rhwp/issues/9353)

- 원 head `7936bc78dc8f4510edde88693e397b6fc91bdbbd`
- 최신 devel `626a5b30ab773a48aec004a22ad789e508ae9c49`
- 검증 후보 `8b54abbd12eaff27b004571721ae81885b828201`

## 체리픽 출처

| 원 커밋 | 리베이스 후 적용 커밋 |
| --- | --- |
| `ba643ad1b01d58cc056cfd8bb95ca06072212ca8` | `ca3fff01a3f692dacde81a04258bfb86bfcf98a6` |
| `4ed8d51dc5658a023593e6eec9bd18e2ffeee63b` | `2565e74a005d1f306b275c30e4301c55a19fcb00` |
| `d206c337b606d8bb181180dd88baa64fa012ab7d` | `f3f7fd12b45dbdbc9e5b9cffb8854351e4a272bd` |
| `7936bc78dc8f4510edde88693e397b6fc91bdbbd` | `d593c539c6f0c7bfb315fcbea407cb1557cf6e9d` |

## 검토 범위

음수 줄 간격으로 앞 줄의 저장 전진에 놓인 다음 줄을 쪽 되감김으로 읽지 않는다. 두 줄 칸 전체가 2쪽에 남고 3쪽은 다음 행부터 시작한다.

[의왕시 원본](../../../samples/issue9353/uiwang_lane_min_length.hwp)과 [한컴 2024 Print](../../../pdf/issue9353/uiwang_lane_min_length-2024.pdf)는 3쪽이다. 원 PR 기준 PDF의 Creator·Producer·페이지 수와 SHA-256을 확인했다.

## 확인한 검증 결과

- 최신 제품 후보 `8b54abbd12eaff27b004571721ae81885b828201`의 집중 회귀 16건이 통과했다. 이 PR의 회귀 2건을 포함한다.
- 리베이스 전 후보 `1b06db582f5209bba290ea6de67653acf0d7ac12`의 Native raw 일치율은 2쪽 96.68%, 3쪽 97.67%였다. 한컴 2024 Print, 96 DPI, compat 2024로 측정했다.

## 최신 후보 전쪽 비교와 직접 판독

측정 소스 `b8baf0bc29bfbcff1aed9eb6c44982c71242d4d1`. Native 바이너리는 제품 소스가 같은 직전 커밋에서 빌드했고, fresh WASM은 이 SHA에서 macOS locked wrapper의 `--no-opt`로 빌드했다. 경로별로 96 DPI 및 2px raw 실루엣을 사용했다.

| 경로·쪽 | 기준 → 후보 | 직접 판독 자료 |
| --- | ---: | --- |
| native 2쪽 | 92.85% → 96.68% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/raw-002.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/pdf-002.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/review-002.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/overlay-002.png) |
| native 3쪽 | 43.55% → 97.67% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/raw-003.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/pdf-003.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/overlay-003.png) |
| wasm 2쪽 | 92.91% → 96.68% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/raw-002.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/pdf-002.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/review-002.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/overlay-002.png) |
| wasm 3쪽 | 43.72% → 97.71% | [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/raw-003.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/pdf-003.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/overlay-003.png) |

- native 전쪽 3쪽, 기준 대비 하락 0쪽. [기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/base-silhouette.tsv) · [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/native/silhouette.tsv)
- wasm 전쪽 3쪽, 기준 대비 하락 0쪽. [기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/base-silhouette.tsv) · [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/uiwang/wasm/silhouette.tsv)

2쪽은 21대 이상·50대 이하가 들어 있는 행을 끝까지 담고, 3쪽의 표는 51대 이상 행으로 시작한다. 3쪽 비고 3번은 원본 저장 줄과 한컴 Print의 줄 경계가 한 글자 다르다. 원본 저장값과 기준·후보 Native/WASM은 모두 `별 / 표 1의`로 나뉘고, 한컴 Print는 `별표 / 1의`로 나뉜다. 표 행 나눔 보정을 적용하기 전에도 재현된다. [원본 레코드와 네 출력의 줄 경계](../assets/planet6897_stored_layout_20261123/pr-9364/note-line-boundary.json)

실제 검사는 전체 쪽수와 두 줄짜리 셀의 행 소속을 검사한다. 2·3쪽의 절대 좌표는 고정하지 않았다.

## 최신 리베이스와 증적 재사용

기준 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 리베이스 결과 `b6e1c2c2d1a429c7912e3114eb988a5f0839fa1b`. 적용 커밋 32개의 패치가 일치했고 제품·테스트·원본·PDF·시각 검증 스크립트의 트리는 리베이스 전후에 동일했다. 후속 메인터너 보정은 명시적으로 글꼴 슬롯과 자연 전진을 바꾼 경우에만 활성화한다. [통합 증적](../assets/planet6897_stored_layout_20261123/README.md)

## 메인터너 보정과 두 한컴 인쇄 조건

의왕시 원본의 3쪽 비고 3번에서 저장 줄은 `별 / 표 1의`이고, 원 PR의 한컴 2024 13.0.0.564 Print는 함초롬바탕의 반각 낫표를 사용해 `별표 / 1의`로 나뉜다. 동일 원본을 MCP 한컴 2024 13.0.0.3901의 Hancom PDF Print로 재인쇄하면 한컴바탕의 전각 낫표와 기존 저장 경계가 재현된다.

보정 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`는 명시적으로 선택한 슬롯별 글꼴의 자연 전진을 실제 줄 채움에 전달한다. 원래의 물리 줄 수·기준선과 저장 IR을 보존하며, 설정을 해제하면 원래 저장 줄을 다시 사용한다. [슬롯 환경](../assets/planet6897_stored_layout_20261123/pr-9364/hcr-symbol-slot.json)

Native의 함초롬바탕 조건은 2쪽 95.38% → 100.00%, 3쪽 95.71% → 100.00%다. 전각 대조는 전쪽 100.00%다. fresh WASM은 2쪽 95.36% → 100.00%, 3쪽 95.65% → 100.00%다. 전각 대조는 두 경로 모두 보정 전후 3쪽의 PNG 픽셀이 일치했다.

[입력·Print 영수증 및 전후 TSV 해시](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/validation.json) · [전각 낫표 재인쇄본](../../../pdf/issue9353/mcp-13.0.0.3901/uiwang_lane_min_length-2024.pdf)

- native 함초롬바탕 조건 [기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/base-silhouette.tsv) · [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/silhouette.tsv)
  - 2쪽 [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/raw-002.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/pdf-002.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/review-002.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/overlay-002.png)
  - 3쪽 [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/raw-003.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/pdf-003.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/overlay-003.png)
- native 전각 대조 [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/fullwidth-control/silhouette.tsv) · [3쪽 review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/fullwidth-control/review-003.png) · [3쪽 overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/fullwidth-control/overlay-003.png)
- wasm 함초롬바탕 조건 [기준 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/base-silhouette.tsv) · [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/silhouette.tsv)
  - 2쪽 [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/raw-002.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/pdf-002.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/review-002.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/overlay-002.png)
  - 3쪽 [raw PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/raw-003.png) · [한컴 Print PNG](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/pdf-003.png) · [review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/hcr/overlay-003.png)
- wasm 전각 대조 [후보 TSV](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/fullwidth-control/silhouette.tsv) · [3쪽 review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/fullwidth-control/review-003.png) · [3쪽 overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/wasm/fullwidth-control/overlay-003.png)

## Merge 후 contributor PR comment 계획

실제 통합 PR과 병합 SHA를 확인한 뒤 입증한 주장과 보정 범위를 존댓말로 안내한다. 같은 원본의 해당 쪽 review·overlay를 병합 SHA 고정 raw URL로 게시한다.

## 통합 제출과 CI

[통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)의 코드 후보와 통과한 GitHub 검사는 [통합 검토 기록](pr_9372_review.md)에 정리했다.
