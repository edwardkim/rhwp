# Task #1759 구현계획서 — 드리프트 측정 하니스 + 1차 서베이 (3단계)

수행계획서: `mydocs/plans/task_m100_1759.md`

## Stage 1 — 하니스 구현 + 기지값 재현
- `tools/metric_drift_survey.py`:
  - 입력: `--files ...` 또는 `--list <paths.txt>`, `-o <dir>`.
  - 한글측: pyhwpx PDF(1회 생성, 캐시) → fitz 페이지별 span baseline(Y 목록, 행 클러스터
    median pitch — hangul_pdf_baseline 로직 재사용) + `find_tables` bbox(pt→px ×96/72).
  - rhwp측: `export-svg`(text y baseline) + `export-render-tree`(Table bbox: pi/rows/cols).
  - 표 매칭: 페이지별 중심 거리 + 크기 유사도, 임계 미달 = unmatched.
  - 산출: `pages.tsv`(file/page/hg_pitch/rh_pitch/delta/lines), `tables.tsv`
    (file/page/pi/w·h delta/matched), `summary.tsv`(파일 요약 + 매칭률).
- 게이트: 36388181 에서 표 h delta ≈ -2.5px(rhwp 작음)·pitch delta ≈ +0.8px 재현.

## Stage 2 — 1차 서베이 실행
- 대상: razor-thin 확정 사례 8건(36388181/36386747/36397394/36386170/36397543/
  36371084/3024019/36382196 — S1/S2 분류) + MATCH 표본 30건(seed 42, 대조군).
- 분석: pitch delta 분포(문서군별), 표 h delta 분포, mismatch 유무와의 상관.

## Stage 3 — 원인 분해 보고 + 후속 이슈 후보
- 지배 원인군(예: 특정 줄간격 환산, 표 행높이 padding 등) 상위 정리 → 최종보고서 +
  후속 이슈 후보 목록(등록은 작업지시자 판단 병기) → squash → PR (도구 + 문서).
