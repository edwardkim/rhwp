# 최종 결과보고서 — Task M100 #1898 (기전 1)

## 이슈

[#1898 tac 인라인 그림 문단 렌더 줄 전진 +11.7px(gap 이중 가산) — 36388711 아웃라이어 기전 (#1858 분리)](https://github.com/edwardkim/rhwp/issues/1898)

## 요약

36388711 p9 불릿(tac 인라인 그림) 문단의 렌더 줄 전진이 layout 대비 +11.7px
(= 줄간격 1회분) 과대하던 렌더-레이아웃 자기 불일치를 수정했다. 원인은 이슈가
추정한 "렌더러의 그림 라인 전진량 gap 이중 가산"이 아니라, **tac 그림의
`PageItem::Shape` 항목이 vpos 기준점을 초기화 → 다음 문단 `vpos_adjust` 의
lazy_base 재산출이 trailing-ls bridge 를 재적용**하는 항목-간 주입이었다.
수정 후 p9 리스트 간격 44.8 → 33.1px (layout 33.1 / 한컴 오라클 32.9 정합).

## 진단 경로

1. `TAC_ADV` 트레이스 (`RHWP_DEBUG_PARA_TAC` — 이번에 pi 하드코딩을 콤마 목록
   env 로 일반화): 문단 내부 줄 전진은 정상 (lh=14.7 ls=11.7 label_extra=0).
2. `TAC_CURSOR` 트레이스 (`RHWP_DEBUG_TAC_CURSOR`): FullPara pi=95 dy=33.1 정상,
   **Shape pi=95 ci=0 dy=0.0 인데 다음 항목 y_in 이 +11.7** — 주입은 항목 사이.
3. `layout.rs` 항목 후처리: `PageItem::Shape` 전부에 대해 vpos 기준점
   (page/lazy base) 초기화 (#409/#1027 — "표/Shape 의 LINE_SEG lh 는 개체 높이를
   포함해 drift" 근거). 초기화 후 다음 문단의 `HeightCursor::vpos_adjust` 가
   base 재산출 경로에서 vpos 불연속(spacing_before 500HU 갭) + trailing-ls
   bridge(#1022 v2) → +880HU(11.7px).

## 수정과 판별 기준

`layout.rs` 초기화 조건에 예외: **실제 텍스트 줄에 통합된 tac 개체**
(호스트 문단 `para_has_visible_text` ∧ 컨트롤 treat_as_char)의 Shape 항목은
기준점을 초기화하지 않는다 — 호스트 LINE_SEG 는 텍스트 줄 높이이고 Shape
항목은 dy=0 이라 drift 근거가 성립하지 않는다.

**반례가 판별 기준을 정제했다**: 1차 blanket 면제(모든 tac Shape)는
sample16 pi=71 — **텍스트 없는 tac-전용 문단**(LINE_SEG lh=9764 = 34.4mm 박스
높이) — 에서 한컴 핀 2건(issue_1116)을 −8.5px 로 깨뜨렸다. tac-전용 문단은
lh 가 개체 높이를 포함하므로 종전 초기화가 맞다. 최종 기준 = 텍스트 보유 여부.

## 검증

- 36388711 p9: 불릿 리스트 TextLine 간격 **44.8 → 33.1px** (layout 33.1,
  한컴 2022 오라클 32.9 — `output/poc/task1858_oracle/36388711_hancom2022.pdf`)
- 핀: `tests/issue_1898.rs` (p9 pi=95/96/97 간격 33.1±1.5px)
- issue_1116 (sample16 한컴 핀) 13/13 — tac-전용 문단 경로 불변 확인
- cargo test 전 스위트 (195 바이너리) PASS
- big_hwp 2,500 A/B (origin/devel 054be69c 베이스): **완전 동일** (PASS 2495/OVER 5)
- big_hwpx 2,500 A/B: **완전 동일** (PASS 2483/STRUCT 9/OVER 8, 파일별 diff 0)
  — 자기 라운드트립은 본 수정에 대칭이므로 예상 결과이며, 시각 정답 검증은
  한컴 오라클(36388711)·한컴 핀(issue_1116)이 담당.

## 범위 외 (이슈 기전 2)

p8/p9 누적 적재 드리프트(저장 lineseg 분할점 vs rhwp 적재 +158.3px)는
#1759/#1763 계열 per-line 적재 축 — 이슈에 측정 기록 유지, 본 수정과 무관.

## 산출물

- 수정: `src/renderer/layout.rs` (tac 인라인 개체 기준점 초기화 예외),
  `src/renderer/layout/paragraph_layout.rs` (RHWP_DEBUG_PARA_TAC pi 목록화)
- 테스트: `tests/issue_1898.rs`
- 문서: plans/task_m100_1898.md, plans/task_m100_1898_impl.md, 본 보고서
