# Task #1720 최종 보고서 — 개체 단위 시각/geometry 회귀 인프라

## 산출물
- `tools/object_visual_regression.py` (356줄) — 개체 단위 rhwp vs 한글 대조 + baseline 회귀 하니스.
- `mydocs/manual/verification/object_visual_regression.md` — 사용 매뉴얼.

## 구성
| 단계 | 방법 | 상태 |
|------|------|------|
| rhwp 개체 geometry | `export-render-tree` → depth≥1 중첩 Table(1×1=그림 프레임) | ✅ |
| 한글 권위 렌더 | COM→PDF→PyMuPDF 96 DPI 래스터 + 이미지 bbox + `find_tables` 표 bbox | ✅ |
| rhwp 래스터 | `export-png`(native-skia) | ✅ |
| 개체 매칭 | **내용 기반**(셀 텍스트 3-gram Jaccard) 우선, 그림은 크기 폴백 | ✅ |
| 시각 갤러리 | 개체별 rhwp↔한글 side-by-side 크롭 HTML + TSV | ✅ |
| baseline 회귀 | 개체 geometry 스냅샷 저장/비교(±tol px) | ✅ |
| 재사용 | `--reuse` 로 재렌더 생략(빠른 반복) | ✅ |

## 검증 (대표 파일 승강기 [별표27])
- 전 파이프라인 동작: rhwp render-tree 42쪽/개체 25개, export-png 42쪽, 한글 48쪽/이미지 14개+표 10개.
- **baseline 회귀**: 동일 baseline → **0건(오탐 없음)**, 변형(h+50/page+1) → **정확 검출**(obj page-1 h-50).
- 좌표계 정합: render-tree px(96dpi) = export-png px(794×1123) = 한글 96dpi 래스터.

## 개체 레벨 발견 (인프라로 측정)
find_tables 로 한글 표 폭이 rhwp 와 정확 일치 확인(558/557/601/597/616/620 px). 큰 중첩표
높이 차: 한글이 rhwp 보다 약간 더 큼(26×9·70×5) — #1718 잔여 −6쪽의 후반부 개체 배치 누적과 정합.

## 매칭 개선 (내용 기반)
초기 크기 기반 매칭은 전폭 표들이 크기 우연 일치로 오매칭(표7 70×5 ↔ 26×9 부품표, 시각 확인).
셀 텍스트 3-gram Jaccard 매칭으로 교체하여 정확 정합 확보:
- 표7 70×5 ↔ 한글 표7 **J=0.88**, 표 p21 13×2 ↔ 한글 p21 12×2 **J=0.97** (동일 페이지·내용).
- 오매칭(표7↔26×9) 제거. side-by-side 로 rhwp 가 같은 표에서 행을 더 조밀히 채움(5.5.2 vs 한글 5.4.6)이
  드러나 −6쪽 원인을 개체 레벨로 시각화.

## 한계
- 텍스트 적은/없는 개체(그림)는 크기 폴백 — 근사, 갤러리 육안 병행.
- 표 분할 시 rhwp/한글 리포팅 단위 차로 높이 delta 는 조각 경계에서 직접 비교 제한(페이지·내용 매칭은 정확).
- render-tree 는 표/프레임 위주 — 프레임 없는 인라인 그림은 한글 이미지 bbox 로 보완.
- 신뢰 핵심: rhwp baseline 회귀(machine) + 내용 매칭 + 시각 갤러리(human).
