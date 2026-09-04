# Task #1759 최종 보고서 — razor-thin 메트릭 드리프트 측정 하니스 + 1차 서베이

## 요약
쪽 경계 razor-thin mismatch(수 px 누적 차) 가족의 계통적 원인 분해를 위한 측정 인프라를
구축하고 1차 서베이를 수행. **지배 원인 후보 = 표 렌더 높이 계통 과대(+25:1 편향)** 를
정량 확정해 이슈 #1760 으로 분리. 줄간격 환산은 전반 정합(median 0.00) 확인.

## 산출 1 — 측정 하니스 `tools/metric_drift_survey.py`
- 한글 PDF(pyhwpx 배치) + PyMuPDF(행 클러스터 pitch·find_tables 표 bbox) ↔
  rhwp(export-svg baseline · export-render-tree 표 bbox) 파일 배치 대조.
- 산출: pages.tsv(페이지 pitch delta) / tables.tsv(표 dw/dh/dy) / summary.tsv.
- 기지값 재현: 36388181 pitch +0.80px/줄, 경계 표 bottom -2.5px = dy -4.1 + dh +1.6 분해.
- 알려진 한계(문서화): 전면 서식 표 find_tables 부분 검출 → |dh|>30px 필터 필요,
  pitch 는 행 인구 차 오염 가능(라인수 ±20% 필터).

## 산출 2 — 1차 서베이 (razor 8 + MATCH 대조군 30, OK 37/38)
| 발견 | 내용 | 후속 |
|------|------|------|
| **표 높이 계통 과대** | 신뢰 매칭 33건 중 +25 / -1. 전형 +0.6~0.8px/표(크기 무관 상수성), 대형 +5~11px(행수 상관 의심). razor 경계 표(+1.6px)가 이 계열 | **이슈 #1760** (행 단위 분해 조사) |
| 줄 pitch 전반 정합 | 신뢰 64페이지 median +0.00 — 줄간격 환산 자체는 건강 | 이산 케이스(8.05→9.85 +1.8 반복 서식, 결재문서 -0.9 계열) 개별 조사 후보 — 추적 이슈 #1759 에 병기 |
| dy 는 종속 지표 | razor dy median -10.7(상류 누적 결과) | 원인 1·2 수정 후 재측정 |

## 검증
- py_compile 통과, rust 소스 무변경(도구+문서 전용) — 기존 게이트 영향 없음.
- 기지값 재현 게이트 충족 (dh/dy 분해로 계획서 수치 정밀화, stage1 보고서).

## 한계 / 후속
- 3단계(수렴 수정)는 #1760 부터 개별 태스크로. pitch 이산 케이스는 표본 확대 후 판단.
- per-line 정렬 기반 pitch 정밀화, 표 행 단위 경계 대조는 하니스 후속 확장.

## 산출물
- 도구: `tools/metric_drift_survey.py`
- 데이터: `output/poc/drift_survey1/` (TSV + 한글 PDF 캐시, 로컬)
- 후속: 이슈 #1760 (지배 원인), 추적 이슈 #1759 갱신
