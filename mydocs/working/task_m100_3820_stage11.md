---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-05
---

# Task #3820 Stage 11 — p166 이후 여분 페이지 anchor 추적

## 목적

사용자 직접 비교에서 p166부터 기준 PDF와 rhwp의 흐름이 달라졌다. 전체 215쪽을 처음부터
다시 산출하지 않고, p166(0-based 165)부터 마지막 기준 PDF 페이지까지의 문자열 순서 anchor와
render-tree source `(pi, line range, control)`를 함께 사용해 **해당 범위의 첫 실제 owner 이탈**을
찾는다.

## 실행 계약

1. `fidelity_compare --text-only --layout-ledger 165 214`와 필요한 개별 `export-svg -p`만 사용해
   p166 이후의 text-owner·table 후보 원장을 갱신한다. 전체 SVG 재생성은 하지 않는다.
2. PDF와 rhwp의 p166 이후 page text를 순서 보존 anchor로 비교하여, 같은 물리 번호가 아닌 첫
   순서 이탈 위치를 정한다.
3. 해당 경계의 source paragraph/LINE_SEG, 각주, 표·그림 control을 `dump-pages`와 기준 PDF로
   대조한다.
4. 보정은 이 실제 source 계약을 focused regression으로 고정한 뒤에만 적용한다.

## 완료 기준

- p166 이후 범위에서 첫 owner 이탈 및 그 직접 원인이 재현 가능하게 기록된다.
- 이전 p118·p127·p156·p168 회귀를 침범하지 않는 좁은 보정 또는 기각 근거가 남는다.
- 수정 시 기준 PDF와 직접 비교한 시각 증적과 최신 page count 원장을 남긴다.

## 2026-08-04 진행 증적

### 해결 확인 — 본 fixture p182→p183 owner drift

- 원인: `pi=1904` 그림 67의 native HWP5 empty-host 2×1 `RowBreak` 표 뒤에 같은
  paragraph style의 빈 guide line `pi=1905..1909`가 있다. 저장 vpos는 모두 그림 표의
  paint span 안이지만, rhwp가 표의 선언 높이를 예약한 뒤 이 guide line을 다시 flow로
  소비해 `pi=1913`(기생충 질환)을 p183으로 이월했다.
- 보정: native HWP5·empty host·positive offset·2×1 그림 표·동일 PS·저장 table span
  내부라는 여섯 계약에서만 guide line을 0-height owner로 기록했다. 일반 빈 문단과
  non-figure table은 이 경로에서 제외한다.
- focused regression: `native_hwp5_figure_table_guides_keep_p182_paragraph_owner`는
  p182의 `pi=1913` 세 줄과 p183의 그림 68 owner를 고정한다.
- 직접 시각 대조: [p182/p183 기준·현재 2×2](../../tmp/stage11-current-p182-p183-fixed/p182-p183-reference-current-2x2.png).
  기준 PDF와 rhwp의 전체 페이지 수는 모두 215쪽으로 회복했다.
- 범위 재검사: `fidelity_compare --text-only 165 214`가 p166–215의 50/50쪽을 완료했다.
  원장은 `output/task-3820-3821-fidelity/stage11-postfix3-p166-end/`에 둔다. p176→177,
  p178→179 및 p200→201의 텍스트 owner 후보는 아직 PDF raster 직접 대조가 필요한
  **후보**이며, 해결로 판정하지 않는다.

### 해결 확인 — 본 fixture p199→p201 footnote/reset tail drift

- 원인: `pi=2285`의 각주 258)이 p199에 이미 각주가 있는 경우라는 이유로 먼저 배치되어,
  다음 문단 `pi=2286`의 명시적 `vpos=0` reset을 넘지 못했다. 그 결과 기준에서는 p200에
  있어야 할 258)이 p199에 붙고, `pi=2310`의 여섯 줄이 p200 footer 아래까지 흐른 뒤 p201의
  다섯 줄 tail이 사라졌다.
- 보정: native HWP5에서 마지막 visible marker 뒤의 **다음 문단이 명시적으로 0으로 reset**되고,
  예상 각주 높이가 현 페이지의 실제 예약 공간과 충돌할 때만 다음 페이지로 보낸다. 기존의
  first-footnote 전용 제한을 제거했지만, marker·reset·충돌 세 계약을 모두 만족해야 하므로
  일반 다중 각주에는 적용되지 않는다.
- focused regression:
  `native_hwp5_late_footnote_moves_to_next_reset_page_before_p200_tail`은 p199에서 258)의
  부재, p200에서의 존재와 `pi=2310` 첫 줄만, p201에서 나머지 다섯 줄을 고정한다.
- 직접 시각 대조: [p199–p201 기준·현재 2×3](../../tmp/stage11-current-p199-p201-fixed/p199-p201-reference-current-2x3.png).
  p199의 258) 조기 배치와 p200 footer 밖 tail은 사라졌고 p201의 본문 시작도 기준과 같다.

### p166–215 마감 재검사

- 최신 원장: `fidelity_compare --text-only 165 214`가 50/50쪽을 완료했고,
  [postfix4 결과](../../output/task-3820-3821-fidelity/stage11-postfix4-p166-end/)를 남겼다.
  rhwp `export-svg`와 기준 PDF의 전체 페이지 수는 모두 215쪽이다.
- 자동 owner 후보는 p176→177과 p178→179 두 건만 남았다. 둘은 [p176–p179
  기준·현재 2×4](../../tmp/stage11-current-p176-p179-postfix4/p176-p179-reference-current.png)와
  PDF text/RenderTree 직접 대조에서 실제 pagination 차이가 아닌 footnote·URL text 추출
  순서의 false positive로 판정했다. 특히 p176의 232)–234)와 p177의 235) 각주는 기준과
  같은 페이지에 전부 보존된다.
- 따라서 이 범위에서 확인된 실제 owner drift(p182→p183, p199→p201)는 보정되었으며,
  자동 원장은 후보를 발견하는 신호로만 사용하고 raster 대조 없이 결함 해결/등록을 판정하지
  않는다.

### 신규 등록 — issue2007 중첩 셀 pagination 전면 불일치

- 대상: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` (17쪽)
- 재현 결과: 최신 rhwp SVG export는 24쪽으로, 기준보다 7쪽 과분할되고
  `overflowCellLines=1266`을 보고한다. `--text-only` 1–17쪽 비교는 owner 후보를
  보고하지 않았지만, 이는 SVG 전체 쪽수를 세지 않는 모드라 중첩 표의
  visual/pagination 결함을 검출하지 못한 것이다.
- 최초 직접 증상: p1부터 표 scale·행 높이가 기준과 다르고, p3에서 `pi=7`의 1×1
  native HWP5 `RowBreak` 외부 표(필드 `impt_reprt0`) 안 1×1 중첩 표의 continuation이
  frame 밖으로 누적된다. 현재 p3은 `PartialTable pi=2 ci=1 rows=1..2`로 렌더되며,
  기준의 셀 단위 조판과 다르게 이후 7쪽을 추가로 만든다.
- 증적: [p1 기준·현재](../../tmp/stage11-issue2007-current-png/pair-001.png),
  [p3 기준·현재](../../tmp/stage11-issue2007-current-png/pair-003.png),
  [p1–17 contact sheet](../../tmp/stage11-issue2007-current-png/reference-current-p001-p017-contact.png),
  전체 SVG/PNG는 `tmp/stage11-issue2007-current-svg/`,
  `tmp/stage11-issue2007-current-png/`, `tmp/stage11-issue2007-reference-png/`에 보관한다.
- 상태: 이 Stage 11의 p166 이후 범위 보정과 분리된 **현재 브랜치의 미해결 결함**이다.
  다음 단계에서 `pi=7`의 outer/nested `RowBreak` continuation contract를 별도 focused
  regression으로 고정한 뒤 셀 fragment pagination을 수정한다.

### 2026-08-05 candidate15 재검사 — 페이지 수 회복·두부 문자 보정

- 최신 candidate의 전체 `export-svg`는 기준 PDF와 같이 **17쪽**이고, 1–17쪽 text-only
  원장은 17/17쪽 완료·누락 0이다. [page-count 원장](../../tmp/stage11-issue2007-page-compare-20260805-candidate15/page-count-ledger.tsv)과
  [run-state](../../tmp/stage11-issue2007-page-compare-20260805-candidate15/run-state.tsv)를 보관했다.
- p4의 outer/nested `RowBreak` 표는 [기준·현재 비교](../../tmp/stage11-issue2007-page-compare-20260805-candidate15/p04-reference-current.png)에서
  물리 페이지와 표 구조가 같은 4쪽으로 회복됐다. 보존 companion PDF는 DejaVu 기반이라
  글꼴 모양 자체는 한컴 정답지로 사용하지 않고, 페이지·표 owner 대조용 참고 자료로만 쓴다.
- 중첩 표 글머리표 `U+F02FB`가 공개 글꼴에서 두부로 보였다. 한컴 검증 기록의 작은 오른쪽
  삼각형 `▸`을 좁게 매핑해 [p13 현재 렌더](../../tmp/stage11-issue2007-page-compare-20260805-candidate15/current-13.png)에서
  실제 표시를 확인했다. 17쪽 전수 [glyph-risk 원장](../../tmp/stage11-issue2007-page-compare-20260805-candidate15/svg-glyph-risk-report.tsv)은
  raw PUA/U+FFFD 0건이다.
- `fidelity_compare`에는 PDF 텍스트 대조와 독립된 raw PUA/U+FFFD 후보 원장을 추가했다.
  이 경로는 이전 candidate의 raw `U+F02FB` 1,027건을 즉시 탐지하며, 현재 candidate에서는
  0건으로 닫힌다.
- HWP 2020 MCP 직접 PDF 변환은 `timeout_seconds=900`에서 서버 응답 timeout으로 끝나
  새 한컴 PDF는 생성되지 않았다. 기존 보존 PDF와 이미 기록된 한컴 glyph 검증을 사용했고,
  이 timeout은 layout 해결이나 glyph 매핑의 성공 근거로 사용하지 않는다.
- 상태: **U+F02FB 두부와 24→17쪽 과분할은 해결 확인**. 나머지 중첩 표의 페이지별 raster
  정합은 별도 잔여 검토 대상으로 유지한다.
