# Task #1811 2단계 완료 보고 — 합성 seg 증거 오인 차단 (수정 1) + 제2원인 판별

## 수정 1 (본 PR 범위) — 합성 seg 의 vpos 전방 보정 오인 차단

- `src/renderer/composer/line_breaking.rs`: `reflow_line_segs` 가 **원본 linesegarray
  부재 문단**(orig=None)에 합성하는 seg 에 `TAG_IMPLEMENTATION_PROPERTY(1<<31)` 부여 —
  컨버터의 합성 lineseg flags=0x8000_0000 관례와 정합.
- `src/renderer/height_cursor.rs`: `vpos_adjust` 에서 prev seg 가 합성 태그면 전방
  보정 bypass (sequential 신뢰) — 합성 seg 의 vpos 는 구세션 저장 flow 의 재구성이지
  실제 저장 증거가 아니다.
- **효과 실측**: pi50 seg tag 0x80060000, pi51 의 +9.6px 보정 소멸(pi52 진입 cur_h
  669.2→659.6, HWP 경로 662.8보다 정합), 분할 예산 117.0→126.6.

## 제2원인 판별 (별도 트랙 — 본 PR 비범위)

예산 정합 후에도 컷은 [2] 유지 (한글/HWP 경로 [3]). `RHWP_CUT_DBG`(신규 진단) 유닛
덤프로 규명:

| 경로 | pi52 셀 유닛 | 높이 | hard_break |
|------|-------------|------|-----------|
| HWPX (reflow 합성 seg) | 8개 | **37.9/31.2px** | **5곳** (합성 seg 가 문단마다 vpos=0 → 리셋 오인) |
| HWP (저장 seg) | 10개 | 30.9/24.3px (=lh1600+ls720) | 1곳 |

- 컷 [2] 의 직접 원인 = hb 오인 정지 + **합성 줄높이 과대(+23%)** 의 결합
  (31.2+37.9+37.9=107 > 예산 96.2 vs HWP 86.1 ≤ 93.0).
- 근본 = **reflow 합성 seg 의 줄높이·vpos 공식이 한컴 계산과 불일치** — 셀 내부
  문단 전반에 걸친 광역 주제(#1772/#1774 계열). 국소 보정은 회귀 반경이 커서 본
  PR 에서 분리, 판별 결과를 이슈에 보고.

## 게이트 (수정 1)

| 게이트 | 결과 |
|--------|------|
| #1810 기준 양 게이트 | vpos: pi18 2쪽 시작 ✓ / page_break: pi26 2쪽 잔존·5쪽 ✓ (hwpx/hwp 모두 2/5쪽) |
| 통제셋 92 | **일치 75 유지**, under 14 불변, over 3 불변 (내부 +2→+1 개선 방향 이동) |
| `cargo test --release` 전체 | **2780 passed / 실질 실패 0** (7건 svg_snapshot CRLF 노이즈 #1786, 내용 diff 0) |

산출: 신규 진단 `RHWP_FLOW_DBG`(typeset 흐름)·`RHWP_CUT_DBG`(셀 유닛) — env-gated,
RHWP_TABLE_DRIFT 관례.
