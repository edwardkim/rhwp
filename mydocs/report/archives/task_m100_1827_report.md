# Task #1827 최종 보고서 — 쪽 상단 spacing_before 트림의 저장 vpos 증거 정합

## 요약

visual sweep 이중상(제출 PR 리뷰 후보 다수)의 지배 원인인 **쪽 상단 페이지 상수
오프셋(−5pt급)**을 해소했다. 렌더러의 column-top spacing_before 전량 트림(#853)이
한컴의 유지 결정(저장 첫줄 vpos≈sb 에 기록됨)과 모순되는 페이지에서 페이지 전체를
위로 밀던 결함 — 저장 vpos 증거 기반 클램프로 일반화.

- 이슈: #1827 / 수정: `src/renderer/layout/paragraph_layout.rs` 1곳
- 근거 계측: 줄별 Δy 시퀀스(pymupdf) — p2 전 줄 −5.1~−4.4pt 상수, 누적 성분 미미.

## 수정

쪽 상단(para_index>0) 문단의 spacing_before 처리를 3분기로:
- 한컴 유지(저장 첫줄 vpos: `0 < vpos0 ≤ sb`) → vpos0 만큼 적용 (#853 의
  para_index==0 클램프와 동형)
- 한컴 트림(vpos0=0) → 종전대로 0 (무변화)
- 증거 없음(합성 seg)·누적축 vpos(vpos0≫sb) → 종전대로 0 (무변화)

## 검증

| 항목 | 결과 |
|------|------|
| task1750 샘플 p2 줄별 Δy | **−5.1~−4.4pt → +1.9~+2.6pt** (p1 과 동일한 잔여 전역 오프셋으로 수렴 — 페이지 이중상 해소) |
| 통제셋 92 | 75/14/3 불변 (렌더 전용 — 쪽수 무영향) |
| clipping_gate (92) | 회귀 0 / baseline 이탈 0 |
| 쪽수 샘플 | byeolpyo 4/26 · giant 42 · scattered 53 · 1749 5쪽 무회귀 |
| `cargo test --release` 전체 | 2783 passed / 실질 실패 0 — **svg 골든 7종 내용 diff 0** (트림 일반화가 골든 문서 무영향; 7건 "실패"는 Windows autocrlf 노이즈 #1786) |

## 잔여 ±2pt "전역 오프셋" 판별 — 측정 아티팩트 (수정 대상 아님)

- 줄 **baseline(span origin)** 대조 결과 실제 잉크 위치는 **Δ중앙값 +0.5pt**
  (min +0.14 / max +0.89) — bbox top 의 +2pt 는 임베드 폰트 ascender 메트릭 차이
  (rhwp Palatino 0.732 vs 한컴 서브셋 1.0, 한글 폰트는 0.859 동일)로 생기는
  **측정 아티팩트**다. (1.0−0.732)×12pt ≈ 3.2pt 가 라인 구성에 따라 +2pt 로 관측.
- 판별 도구 정식화: `tools/compare_line_baselines.py` — 줄 baseline 시퀀스
  매칭·Δ통계·스텝(>1pt) 검출. bbox 기반 계측의 폰트 메트릭 오탐을 제거한다.
- 잔여 실차이 +0.5pt 수준의 미세 누적은 razor-thin 프로그램(#1759 계열) 범위.
- 도구가 부수 검출한 신규 리드: task1750 샘플 p5 수식 설명 줄 2곳의 −21~−23pt
  스텝 (수식 뒤 설명행 배치 차이) — 별개 국소 결함 후보로 후속 분류 대상.
- 표 셀 내 가로 오프셋(pr_1756)은 별개 소원인 — #1759 계열 범위.
