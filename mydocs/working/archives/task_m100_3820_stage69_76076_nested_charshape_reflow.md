---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 69 — 76076 nested-cell CharShape reflow (기각)

## 출발점

Stage 68은 `76076_regulatory_analysis.hwp` p34의 nested table에서 fallback reflow가
첫 글자모양 하나만 사용해 분할 폭과 최종 paint 폭이 달라질 수 있다는 가설을 세웠다.
이 Stage는 실제 `CharShapeRef`를 reflow에 반영하되, 과거 80168 계열의 모든 셀 fallback
동작을 무차별로 바꾸지 않는 최소 범위를 확인한다.

## 판정 기준

1. 76076 p33--p36이 PDF의 fragment/줄바꿈 및 p34 우측 경계를 회복한다.
2. nested table height 측정과 최종 paint가 같은 composed line을 사용한다.
3. 기존 `overflow_cell_baseline`, `issue_1891`, 80168 관련 focused gate를 보존한다.
4. 실제 char-shape가 오직 동일 글꼴·크기·장평에서 음수 자간만 바꾸는 fallback에만
   적용할 수 있는지 검증한다. 글꼴 또는 크기가 다른 문단까지 확장하지 않는다.

## 실험과 판정

- p34 `근거설명`의 CharShapeRef(동일 서체·크기, 음수 자간 -0~-9)는 split run으로
  복원할 수 있었다.
- 그러나 PDF의 continuation 가용폭은 **437.3px**, 자간 복원 뒤 RHWP 최종 경로는
  **442.3px**였다. 이 5px 차이만으로도 줄 끝이 한 글자씩 밀렸다.
- 저장된 nested table 폭은 **487.6px**인데 render-normalization overlay가 부모 셀 폭
  **506.2px**로 투영하고 있었다. 저장 폭을 사용할 때의 continuation 폭은 PDF와 같은
  437.3px였다.

따라서 CharShape는 paint 폭 편차를 설명하는 보조 요인이지만 p34의 페이지/줄 경계의
근본 원인이 아니다. 셀 fallback에만 자간 분할을 추가하는 실험은 코드에 남기지 않았고,
다음 Stage에서 non-TAC nested table의 폭 투영 계약을 PDF 원본과 다시 검증한다.

## 증적

- 기준 PDF: `samples/issue1891/76076_regulatory_analysis-2024.pdf` p34
- 기준 입력: `samples/76076_regulatory_analysis.hwp`
- Stage 68 baseline: `mydocs/pr/assets/task_m100_3820_stage68_76076_p33_p036_nested_table/`
- 진단 출력: `RHWP_DIAG_RECOMP='13.3월부터 제작되는 분쇄기'`로 p34의 저장폭/실효폭
  별 줄 경계를 비교했다. 진단은 재현용 관측일 뿐 정상 경로의 동작을 바꾸지 않는다.
