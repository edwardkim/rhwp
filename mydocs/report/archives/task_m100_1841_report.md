# Task #1841 최종 보고 — 자리차지 표 outer_margin bottom 재개 갭 수정

## 결론

visible-host 자리차지(TopAndBottom) 표 직후 본문 재개에 표의 `outer_margin_bottom` 을
반영하도록 layout(실제 렌더 y)·typeset(pagination 북키핑)을 대칭 수정. 결재문서 계열
(헤더 표 852HU=8.5pt)의 전면 −8.5pt 상향이 해소되고, SO-SUEOP p22 권위 PDF 대조에서
본문·후속 앵커가 0.8~1.7px 정합으로 수렴했다.

## 수정

1. `layout.rs` 표-in-문단 렌더: visible-host float 의 비양수 offset 재개 y 두 분기에
   `visible_outer_bottom_px` 가산 (`table_visual_end (+inter_float_gap) + ob`,
   `table_y_before.max(table_visual_end + ob)`). 양수 offset(part-3) 분기 불변.
2. `typeset.rs format_table`: `outer_bottom` 을 `is_tac || (is_para_topbottom_float
   && para_has_non_whitespace_text)` 로 확장 (렌더 가산과 대칭). 전면(모든 비-TAC)
   적용은 한컴 핀(1086/1156)과 충돌하여 기각 — visible-host float 형상 한정.

## 보상 오차 핀 정정 (2 테스트)

수정으로 종전 렌더의 보상 오차를 고정하던 핀들이 드러나 근거와 함께 정정:

- `issue_1789_exclusion_probe_line_spacing`: 529.9px 핀은 om 누락 렌더값.
  저장 vpos 산술 = 34925HU(465.7px)+body_top 75.6 = **541.3px** 로 정정 (샘플 =
  동작소방서 36385142 — 한글 PDF baseline p1 median +0.07pt 로 교차 검증).
- `issue_1692` p22: ① "표 하단 +1.5px 이내" → "+om_bottom(11.36px)±1.5px"
  (권위 pdf/SO-SUEOP-2024.pdf p22 실측: 표 하단 246.0pt → 본문 y0 253.3pt).
  ② 후속 질문 pdftotext yMin=359.796pt 핀 → PyMuPDF line bbox 재측정 366.72pt
  (=489.0px). 종전 핀은 om 누락 렌더(479.3px)와 좌표 관례 차이로 우연 일치했던 값 —
  사전-수정 렌더는 본문 시작이 권위 PDF 대비 12px 위였다.

## 검증

- 케이스 (한글 2022 PDF, sweep proxy 재현 메트릭):
  - 동작소방서 36385142 p1: baseline median −8.45 → **+0.07pt**, proxy 13.8 → **27.8%**
  - 관악소방서 36389312 p1: median −8.95 → −0.43pt, proxy 7.1 → **9.8%**
  - SO-SUEOP p22 (HWP3): 본문 첫 줄 rhwp 337.0 vs 권위 337.8px / 후속 질문 490.7 vs 489.0px
- cargo test --release 전수: 80개 타깃 전부 통과 (핀 정정한 issue_1789/issue_1692 포함)
- big_hwpx 2,500 render-diff: PASS 2471 / OVER 6 / STRUCT 19 / PAGE 4 — 직전 스택(1811v2) 대비 파일 단위 회귀 0.
  (render-diff 는 A 원본·B 라운드트립을 동일 규칙으로 렌더하므로 양쪽을 동일 이동시키는 본 수정에는 diff 불변 — 개선은 한글 PDF 대조 sweep 메트릭으로 측정, 위 케이스 참조)
- big_hwp 2,500 네이티브 render-diff: PASS 2494 / OVER 4 / STRUCT 2 — 직전 스택 대비 파일 단위 회귀 0.
- 핀 테스트(issue_1692/issue_1789) bare devel(스택 PR 미포함) 재실행 통과 — PR 대상 브랜치 정합 확인.

## 잔여 (비범위, 분리 기록)

- 하단 결재부 −34pt (관악 p1 하단/동작 p2): 선행 표들(TAC 4x12 등)의 높이 부족 누적 —
  표 높이 트랙(#1759/#1763) 소관
- 셀 내부 '****' 국소 −17pt: 셀 콘텐츠 축
- 36388711 p7 5.75%: 다페이지 흐름 시프트 (#1774 leak 계열)
- 36381023: 세로 정합(median +0.02pt) 완료 — 잔여는 자간(가로) 축
