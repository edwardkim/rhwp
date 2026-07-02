# Task #1811 최종 보고서 — HWPX p5 tail/line drift: 합성 seg 증거 오인 차단 + 제2원인 판별

## 요약

#1811(PR #1810 머지 후속)의 p5 tail/line drift 를 조사해 **두 개의 독립 원인**을
규명하고, 그중 원리적 결함(합성 seg 의 vpos 증거 오인)을 수정했다. 잔여(합성 seg
유닛 구성 격차)는 광역 주제로 판별 문서화해 분리한다.

- 이슈: #1811 / 브랜치: `local/task1811` (upstream/devel 6a27dcd2 + 열린 PR 21건)
- 수정: `composer/line_breaking.rs`, `height_cursor.rs` (+진단 2종)

## 원인 사슬 (stage1, `task_m100_1811_stage1.md`)

증상 = p4 끝 분할 표 pi52 컷이 HWPX end_cut=[2] vs HWP/한글 [3] → (사회기여) 문단이
p5 로 밀려 p5 하단 tail overflow.

**원인 1 (수정 완료)**: pi50(TAC 표 host)은 원본 XML 에 linesegarray 부재 →
로드-정규화(`reflow_line_segs`)가 seg 를 합성하는데 **synthetic 표식 없이** 생성 →
`vpos_adjust` lazy 전방 보정이 이를 실제 저장 증거로 오인, pi51 을 구세션 저장 flow
위치로 +9.6px 과대 전진(RHWP_VPOS_DEBUG 실측) → 분할 예산 −6.4px.

**원인 2 (판별·분리)**: 예산 정합 후에도 컷 [2] — pi52 셀 유닛 구성이 경로별 상이
(HWPX 합성 seg: 8유닛·높이 37.9/31.2·hard_break 5곳(문단별 vpos=0 리셋 오인) vs
HWP 저장 seg: 10유닛·30.9/24.3·hb 1곳). 근본 = **reflow 합성 seg 의 줄높이·vpos
공식이 한컴 계산과 불일치**하는 광역 주제(#1772/#1774 계열) — 국소 보정 회귀 반경이
커서 본 PR 비범위, 후속 트랙으로 분리.

## 수정 (stage2, `task_m100_1811_stage2.md`)

1. `reflow_line_segs`: 원본 부재(orig=None) 합성 seg 에
   `TAG_IMPLEMENTATION_PROPERTY(1<<31)` 부여 (컨버터 합성 lineseg 관례 정합).
2. `vpos_adjust`: prev seg 가 합성 태그면 전방 보정 bypass (sequential 신뢰).
3. 진단 자산: `RHWP_FLOW_DBG`(typeset 문단별 cur_h), `RHWP_CUT_DBG`(셀 유닛 시퀀스).

효과: pi51 +9.6px 소멸(pi52 진입 669.2→659.6), 분할 예산 117.0→126.6 (HWP 123.4 정합).

## 게이트

| 게이트 | 결과 |
|--------|------|
| #1810 기준 양 게이트 | vpos: pi18 2쪽 시작 ✓ / page_break: pi26 2쪽 잔존·5쪽 ✓ |
| 통제셋 92 | 일치 75 유지, under 14·over 3 불변 (over 내부 +2→+1 개선 방향) |
| `cargo test --release` 전체 | 2780 passed / 실질 실패 0 (7건 svg CRLF 노이즈 #1786) |

## 이슈 #1811 처리 제안

- 원인 분석(기대 작업 1·2) 완료, 수정 1 은 별도 조판 보정 PR(기대 작업 3)로 제출.
- p5 시각 후보의 완전 해소는 원인 2(합성 seg 유닛 구성) 해결이 전제 — 판별 결과를
  이슈에 보고하고 open 유지 또는 후속 이슈 분리는 메인테이너 판단에 위임.

## v2 정밀화 — seoul_1006 회귀 해소 (2026-07-03)

전면 차단(조기 return)이 big_hwpx seoul_1006 에서 회귀(PASS→STRUCT 641px, p7
Group+줄이 p8 로 이월)를 유발했다. 이등분·트레이스 판별:

- seoul_1006 의 +9.6px A/B 갈림은 **devel 부터 존재**하는 파스 경로 측정차(pi=41
  RowBreak 자리차지 표) — 본 수정이 만든 게 아님
- devel 에서는 저장 vpos 로의 **대형 재앵커 점프**(pi=42, +376px)가 양 경로를 같은
  저장 좌표에 재동기화해 쪽나눔이 수렴해 왔음 — 전면 차단이 이 안전망까지 제거

**정밀화**: 조기 return 을 최종 result 방향·크기 클램프로 교체 — 합성 seg 증거의
전방 이동 중 **소폭(≤48px, drift 대역)만 차단**. 되감기와 대형 재앵커(>48px)는 허용.
saved_bounds 의 유해 전진(+9.6px)은 차단 대역에, seoul_1006 의 재앵커(+376px)는
허용 대역에 정확히 들어간다.

**v2 검증** (통합 스택 기준):
- saved_bounds_cumulative_page_break: PASS 유지 (0.12px)
- seoul_1006: STRUCT 641px → **PASS 0.00**
- big_hwpx 2,500 배치: 직전 대비 **개선 1건(seoul_1006)/회귀 0** — PASS 2471
