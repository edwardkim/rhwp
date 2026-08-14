# #3820 Stage 178 - stored source-frame row split

## 목적

fixture 치수와 고정 pixel tail allowance 없이, HWP/HWPX가 저장한 `lineSeg` source
frame과 실제 `CellUnit` 경계를 이용해 RowBreak tail owner를 계산한다.

## 기준

- 작업 브랜치: `task/3820-production-fidelity-residual`
- 기준: `upstream/devel`의 `3c7b89356` 위 리베이스 상태

## 확인한 사실

1. HWPX 11절의 큰 103x2 `RowBreak` 표에는 `lineSeg.vertical_pos`가 0으로
   되감기는 행이 여러 개 있다. 모든 되감김을 물리 쪽 경계로 강제하면 HWPX가
   과분할되고, 일반 흐름만 따르면 381쪽으로 과소분할된다.
2. 저장 frame은 픽셀 allowance가 아니라 CellUnit의 정확한 source cut으로
   선택해야 한다. 한 줄의 response tail도 문단 전체나 table topology가 아니라
   다음 visible unit의 measured height로 전진해야 한다.
3. 단일 visible cell과 구조적 empty partner를 가진 직접 HWPX RowBreak 행은,
   whole-row fast path가 저장된 internal frame boundary를 건너뛸 수 있다.

## 구현

- `next_visible_unit_cut_for_row`로 0px 예산에서도 첫 실제 source unit을 선택했다.
- source-empty spacer 행, single-visible source cell, stored `vpos` rewind, two-line
  source-frame 높이를 layout engine의 일반 질의로 노출했다.
- QA table 차원·행 수·문단 수·선언 높이에 묶인 fixed allowance를 제거하고, source
  frame 또는 paragraph tail의 exact cut으로 대체했다.
- HWPX RowBreak paint tail은 고정 64px 대신 `split_total - CellUnit consumed - padding`
  이라는 같은 조각의 measured footprint로 계산했다.

## 스테이지 검증

```text
rhwp verify samples/2025 행정업무운영 편람(최종).hwp --expect-pages 383
PASS: actual=383

rhwp verify samples/2025 행정업무운영 편람(최종).hwpx --expect-pages 383
PASS: actual=383
```

HWPX-to-HWP 저장본의 source-frame provenance와 Studio WASM 최종 게이트는
Stage 182에서 이 스테이지의 일반 로직을 확장한 뒤 확정한다.

## 상태

Stage 180에서 native HWP5 first-frame owner를 별도로 보정한다.
