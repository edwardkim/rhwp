# Task M100-2439 Stage 1 — 반복 표 겹침 재현과 회귀 가드

- 이슈: [#2439](https://github.com/edwardkim/rhwp/issues/2439)
- 브랜치: `fix/2439-split-table-flow`
- 작성일: 2026-07-19

## 1. 재현 결과

`upstream/devel` 28331531에서 사용자 제공 HWP를 SVG로 내보내면 8쪽이며, 0-based
page 5의 `para=53`에서 본문 하단을 3.8px 넘는 `LAYOUT_OVERFLOW`가 발생했다. 같은
visible host의 두 RowBreak 표 중 후행 표가 페이지 하단의 좁은 조각에 그려져 행과
라벨이 겹쳤다.

## 2. 원인

- fresh-page orphan 가드가 후행 표를 새 페이지로 이월한 뒤에도 이전 페이지의
  `para_start_height`를 placement/exclusion 기준으로 전달했다.
- `vertical_offset=0`인 첫 co-anchored 표가 후행 sibling을 위한 exclusion을 남기지
  않아 양수 offset 표의 시작점과 exclusion 높이가 첫 표 안으로 들어갔다.
- 같은 host의 서명 텍스트가 표 그룹 뒤에서 emit되어도 자기 exclusion을 무시했다.

## 3. 회귀 fixture

- `samples/hwpx/issue2439_page_local_float_exclusion.hwpx`
  - 첫 표는 현재 페이지에 남고, 후행 표는 fresh page로 통째 이월된다.
  - 후속 `AFTER FLOAT`는 이월 표 아래에서 시작해야 한다.
- `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp`
  - zero-offset 첫 표와 양수 offset 후행 표가 같은 host에 있다.
  - 두 표와 host post-text가 순서대로 겹치지 않아야 한다.

사용자 제공 원본은 재배포하지 않고 로컬 시각 오라클로만 사용했다.

## 4. RED 기준

신규 `tests/issue_2439.rs`는 이월 표의 fresh-page exclusion과 zero-offset 선행 표의
전체 exclusion, 표 그룹 뒤 host 텍스트의 시작 위치를 render tree bbox로 단언한다.

## 5. 후속 기록

이 문서는 최초 재현과 회귀 가드를 기록한 당시 스냅샷이다. 9쪽 중간 출력을 완료로
보았던 후속 판정은 [Stage 5](task_m100_2439_stage5.md)에서 철회했고, 추가 구현과 최종
10쪽 검증은 [Stage 6](task_m100_2439_stage6.md)에 기록했다.
