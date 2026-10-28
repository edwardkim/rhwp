# Task #1811 1단계 조사 보고 — p5 tail/line drift 원인 절반 규명 (진행 중)

## 증상 재확인 (visual sweep 후보의 실체)

리뷰 PNG(`mydocs/pr/assets/pr_1752_visual_review_p5_followup_candidate.png`) 판독:
한글 p5 는 (청정아트)부터 시작하는데 **rhwp(HWPX) p5 는 (사회기여) 문단이 상단에 하나
더** 있다 — p4→p5 경계에서 rhwp 가 유닛 하나를 일찍 끊어 이후 내용이 밀리고 p5
하단에서 ⑥ 제목이 잘림(`frame_tail_overflow`/`line_band_drift` 후보의 실체).

## 정량 (통합 베이스, HWPX vs HWP vs 한글 2024 PDF)

- 페이지 bbox 수준: 양 경로 사실상 동일 (p5 Δbot −9 공통) — bbox 로는 비가시.
- **p4 끝 분할 표 pi52 컷: HWPX end_cut=[2] vs HWP end_cut=[3]** (한글 = [3] 동작).
  분할 예산 `avail_for_rows`: HWPX **117.0** vs HWP **123.4** — 정확히 **6.4px(480HU)**.
- pi52 진입 시 `cur_h`: HWPX 669.2 vs HWP 662.8 (+6.4). p1~p3 경계는 동일(내부 used 차이는
  vpos 흡수로 무해), p4 내부(pi47~51)에서만 벌어짐.
- **렌더 y 는 양 경로 일치**(pi50 표 y=355.2/h=383.6, pi52 시작 ≈910) — 렌더러의 vpos
  보정이 드리프트를 흡수. 순수 **typeset 예산 회계만** 어긋난다.

## 구조적 원인 (확정 사실)

- **pi50(TAC 표 host 문단)은 원본 HWPX XML 에 linesegarray 자체가 없다** (ir-diff:
  line_segs A=0 vs B=1; 파서는 충실 — 합성 없음).
- 원본 HWPX 의 주변 저장 flow 를 역산하면 pi50 의 표줄 advance 는
  `lh 28769 + gap 720`(pi49 끝 223964 → 225164 → +29489 → pi51 254653 정합) —
  즉 **한컴 자신의 저장 flow gap = 720**.
- rhwp 변환 HWP 는 pi50 lineseg 를 `gap=1200` 으로 재합성 → 이후 vpos 전부 +480HU.
- 그런데 페이지네이션 결과는 HWP 경로가 한글 PDF 와 일치(여유 +6.4px)하고, HWPX
  경로가 −6.4px 부족 — **양 경로 모두 저장 flow 를 그대로 신뢰한 결과가 아니라,
  lineseg 유무에 따라 vpos-sync/계산 advance 경로가 갈리면서 정반대 부호의 잔차**가
  생긴 것. (HWPX: 저장상대 664.4 vs cur 669.2, +4.8 과소비 / HWP: 저장상대 670.8 vs
  cur 662.8, −8.0 미달.)

## 항목별 분해 (RHWP_FLOW_DBG 계측)

| 항목 | HWPX advance | HWP advance | Δ |
|------|-------------:|------------:|---|
| pi50 (TAC 표 host, lineseg 부재) | +388.4 | +391.6 | HWP +3.2 |
| pi51 (빈 문단, lh500+gap400=12.0) | **+21.6** | +12.0 | **HWPX +9.6 (=720HU)** |
| 순합 (pi52 진입 cur_h) | 669.2 | 662.8 | +6.4 |

- 지배 항 = **pi51 에서 HWPX 경로가 저장 세그(12.0px) 외에 +9.6px(720HU) 를 추가**
  — pi50 lineseg 부재로 인한 vpos 보정(전방 correction) 경로의 과잉으로 추정.
  720 = pi50 의 원본 저장 flow gap 과 동일 값.
- HWP 경로는 pi51 에서 저장 vpos 로의 sync 를 하지 않고(자체 누적 유지) 12.0 만 소비.

## 발생 기전 완전 규명 (RHWP_VPOS_DEBUG)

- pi51 의 +9.6px = `HeightCursor::vpos_adjust` **lazy 경로 전방 보정**:
  `VPOS_CORR path=lazy pi=51 prev_pi=50 prev_vpos=225164 prev_ls=720 vpos_end=254653
  y_in=647.59 → result=657.19 (+9.6) applied=true`. HWP 경로는 동일 식이 정확히
  0 보정(650.79 == 저장 상대값).
- prev(pi50) 의 seg 는 **로드-정규화가 생성한 것**: 원본 XML 에 linesegarray 가 없어
  `document_core/commands/document.rs` 의 "HWPX: TAC 표 문단 LINE_SEG lh 보정" +
  `reflow_line_segs` 가 vpos=225164/lh=28769/ls=720 seg 를 합성 — 그러나
  **synthetic 태그(0x80000000) 없이 flags=0x60000** 으로 생성되어, vpos 보정이
  이를 실제 저장 증거로 오인해 원본 저장 flow(한컴 구세션 값) 위치로 전방 이동시킨다.
  한글 2024 재조판은 그보다 조밀하게 흘러 유닛 하나를 더 담는다(컷 [3]).

## 수정 설계 (다음 단계)

- 1안(선호): reflow/TAC-lh 보정이 **생성하는 seg 에 synthetic 태그를 부여**하고,
  `vpos_adjust` 의 prev seg 선택에서 synthetic 을 실증거로 쓰지 않도록(전방 보정
  bypass — #991 '분할 표 직후 sequential 신뢰'와 동형). 단 `is_synthetic_line_seg`
  소비처(전체 6곳)의 행동 변화를 전수 점검해야 함.
- 2안(협소): `vpos_adjust` lazy 경로에서 prev 문단이 "TAC 표 보유 + 원본 lineseg
  부재(reflow 흔적)" 인 경우만 보정 skip.
- 게이트: 본 샘플 컷 [2]→[3] 복원 + #1810 기준 양 게이트(vpos pi18/page_break pi26)
  + 통제셋 92 + cargo test 전체.

## 남은 규명 (다음 단계·구버전 메모)

- typeset p4 흐름의 **항목별 cur_h/advance 분해 계측**(RHWP_FLOW_DBG 일회 계측)으로
  HWPX 경로 +6.4px 의 정확한 발생 항목(pi50 TAC advance vs pi51/52 vpos-sync 거부)을
  특정한다.
- 수정 방향 후보: lineseg-less TAC host 의 advance 를 인접 저장 vpos(pi49 끝→pi51)
  로 앵커(원본 flow 정합, HWPX 데이터만으로 결정 가능) — 렌더는 이미 일치하므로
  typeset 예산만 정합하면 컷이 [3] 으로 복원될 전망.
- 게이트: 본 샘플 양 게이트(#1810 기준: vpos pi18 2쪽 시작 / page_break pi26 2쪽
  마지막·5쪽) + 통제셋 92 + `cargo test`.
