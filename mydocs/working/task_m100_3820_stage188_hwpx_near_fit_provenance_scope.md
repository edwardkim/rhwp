# #3820 Stage 188 - HWPX near-fit provenance scope

## 목적

Stage 186의 native source-geometry guard 때문에 HWPX 원본과 HWPX-to-HWP 저장본이
384쪽으로 늘어난 회귀를 profile 의미에 맞게 분리한다.

## 원인

HWPX stored-layout은 HWP5 object declaration과 별도의 stored pagination frame을
갖는다. native HWP5에서 stale object frame을 배제하려고 추가한
`declared_excess_has_source_frame` 요구를 HWPX에 그대로 적용하면, 유효한
near-measured HWPX RowBreak fit도 차단된다.

## 수정

- native HWP5 near-fit에는 declared row geometry source proof를 계속 요구한다.
- `hwpx_stored_layout` 및 HWPX-to-HWP 저장 계보는 기존 near-measured stored frame
  경로를 유지한다.

## 검증 상태

`issue_3930_hwpx_hwp_save_layout` 결과:

- HWPX 원본과 HWPX-to-HWP 저장본의 #3930 render-tree/page-count contract: 통과
- native HWP Q&A page count: 384쪽으로 실패

native HWP 잔여 경계는 Stage 189에서 HWPX의 같은 Q&A page owner와 비교한다.
