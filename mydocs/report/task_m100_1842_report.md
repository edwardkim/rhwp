# 최종 결과보고서 — Task M100 #1842

## 이슈

[#1842 서식 표 셀 내부 인라인 개체 라인높이 — 저장 LINE_SEG lh 대비 재계산 드리프트 (3114781 p2 ±34pt)](https://github.com/edwardkim/rhwp/issues/1842)

## 요약

셀 내부의 **텍스트 없는 tac(글자처럼) 묶음 전용 문단**에서 `has_tac_shape`
라인높이 축소 분기가 max_fs=0 으로 퇴화(`font_lh=0`)해 저장 lh(3401HU=45.3px)
가 소실, '작성요령' 개체 아래 블록 전체가 −33pt 당겨졌다. 셀 컨텍스트 한정
가드(max_fs>0)로 수정 — **p2 Δbaseline median −33.93pt → +0.06pt** (한글 2022
오라클, n=54).

## 진단 경로

1. 렌더 트리 사다리 vs 저장 LINE_SEG vpos 대조: 셀 내부에서 어긋나는 곳은
   p[0](tac 묶음 전용) 한 줄뿐 — p[6]/p[9] 내부표 host 라인(lh 10858/9095)은
   저장값 사용 정상. 이슈의 −45/+15 국소 변동은 −33 시프트와 별개 국소 축.
2. 임시 계측: p[0] 에서 raw_lh=45.3, **max_fs=0.0**(빈 문단), tac_shape=true,
   explicit_h=false → 축소 분기 진입 (`raw_lh > 0*1.5` 항상 참) →
   `font_lh = 0*1.2 = 0` 퇴화.

## 수정과 판별 기준 (반례가 정제)

- 축소 분기("Shape 와 텍스트가 같은 줄이면 텍스트 baseline 을 폰트 기준으로")는
  **텍스트가 있어야(max_fs>0) 전제가 성립**한다.
- 1차 전역 가드는 본문의 같은 기하(sample16 pi=71 RFP 박스)에서 issue_1116
  한컴 핀 2건을 +10.4px(ls 1회분)로 깨뜨렸다 — 본문 경로는 reserved/
  skip-advance 보상 기계가 축소값을 전제로 이미 한컴 정합을 이루고 있다.
- 최종: **셀 컨텍스트 한정** `(cell_ctx.is_none() || max_fs > 0.0)` — 셀은
  3114781 오라클, 본문은 sample16 오라클이 각각 핀.

## 검증

- 3114781 p2: Δbaseline median **−33.93 → +0.06pt** (min −11.4 / max +15.0 은
  p1 에도 존재하는 별개 국소 축 — 아래 '남는 축')
- 핀: `tests/issue_1842.rs` (저장 lh 45.3px 라인 존재 + 사다리 78.7px) +
  픽스처 `samples/issue1842_cell_tac_group_lineheight.hwp` (46KB)
- issue_1116 (sample16 한컴 핀) 13/13 — 본문 경로 불변
- big_hwpx 2,500 A/B: **회귀 0 / 개선 1** (admrul_0694 OVER 16→PASS — 동류 개선)
- big_hwp 2,500 A/B: **완전 동일** (PASS 2495/OVER 5)
- 풀 스위트: PR CI 위임 (배치 검증 체계)

## 남는 축 (이슈/후속 기록)

- 국소 −11pt (p1 '결어' 구간, p2 min −11.4): tac 축과 무관한 별개 국소 축.
- p2 max +15.03 ('진료유형별 비보험' 반전): 내부표 인접 국소 축.
- 본문 tac-전용 문단의 "축소+보상" 구조 자체(#1898 판별과 동류)는 언젠가
  저장 lh 직사용으로 단순화할 가치가 있으나, 한컴 핀 재보정이 필요한 설계
  사안이라 현 이슈 범위 밖.

## 산출물

- 수정: src/renderer/layout/paragraph_layout.rs
- 픽스처·테스트: samples/issue1842_cell_tac_group_lineheight.hwp, tests/issue_1842.rs
- 문서: plans/task_m100_1842.md, 본 보고서
