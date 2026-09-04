# Task M100 #3931 — Stage 2 저장 fragment 경계 복원

- 날짜: 2026-08-15 KST
- 기준: `upstream/devel` `fbca0aa6c22db9a30e6c417190ae4ddfe924773e`
- 대상: native HWP5의 빈 host·다행 RowBreak 표
- 상태: 구현 및 focused 회귀 통과

## 결과

sec=10 `pi=23`의 대상 셀은 저장 LINE_SEG pitch 24.51px와 16줄 전체 높이 392.3px를 그대로
유지한다. 첫 문단의 마지막 줄은 rhwp page index 290, 둘째 문단의 첫 줄은 index 291에 속한다.
한컴 2020 PDF에서 확인한 12줄+4줄 물리 fragment 소유권과 같은 형태다.

첫 fragment 표의 하단도 본문 하단 안에 들어간다. 이전 후보에서 발생한 14.3px
`LAYOUT_OVERFLOW`는 제거했으며, 이 조건을 render tree의 Table/Body bbox 비교로 래칫했다.
전체 HWP 페이지 수는 393쪽에서 392쪽으로 한 쪽 줄었다. 383쪽 오라클까지 남은 차이는 Stage 3에서
`pi=14`와 HWPX를 별도로 다룬다.

## 구현 경계

전역 줄높이나 선언 높이 tolerance를 바꾸지 않고 다음 세 증거를 함께 사용했다.

1. `typeset.rs`는 빈 host, 다행 RowBreak, 내부 저장 `vpos` reset, 다음 source 문단 되감김,
   저장 하단의 현재 쪽 수용 가능성을 모두 만족할 때만 flow anchor를 저장 상단으로 재동기화한다.
2. `table_layout.rs`는 내부 reset 직전의 마지막 가시 줄이 현재 조각에 들어갈 때만 그 조각의
   후행 line spacing과 문단 뒤 간격을 컷 높이에서 제외한다. 셀 전체 높이는 줄이지 않는다.
3. `table_partial.rs`는 같은 내부 reset 증거가 있고 직전 host 여백이 저장 host line advance
   이내일 때만 그 여백을 회수한다. 일반 TopAndBottom 표의 push-down과 반복 fragment 여백은
   기존 동작을 유지한다.

## 검증

- `issue_3931_declared_rowbreak`: 3 통과, 1 Stage 3 RED ignore
- `issue_3738_rowbreak_table_footnote_fragment`: 33/33 통과
- `issue_3930_hwpx_hwp_save_layout`: 2/2 통과
- #874, #2097, #2105, #2439, #3236, #1156, #1748 focused 회귀: 전건 통과
- `cargo fmt --all`, `git diff --check`: 통과

## Stage 3 실행 지점

Stage 2의 HWP 저장 reset 특례를 일반화하지 않는다. `pi=14`의 근소한 declared 초과가 실제 첫
fragment scan보다 앞서 이월되는 경로를 별도로 계측하고, scanner가 안전한 첫 조각을 만들 수
있는 경우에만 선이월을 보류한다. 이후 저장 되감김 증거가 없는 HWPX에 같은 구조 판정을 적용한다.
