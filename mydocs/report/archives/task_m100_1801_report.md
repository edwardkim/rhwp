# Task #1801 보고서 — visual sweep 일반 파일 입력 및 overlay 검토 개선

## 요약

visual sweep 도구를 PR 리뷰와 수동 시각 검증에 더 직접적으로 쓸 수 있도록 개선했다. 이제 preset target에
샘플을 등록하지 않아도 일반 HWP/HWPX 경로와 기준 PDF 경로를 바로 넘길 수 있고, 특정 페이지만 선택해
compare/overlay/review 산출물을 만들 수 있다.

## 변경 내용

- 일반 파일 입력:
  - `--hwp`
  - `--pdf`
  - `--key`
  - `--file-target`
- 페이지 선택:
  - `--page`
  - `--pages`
- overlay diff 산출:
  - `overlay/overlay_###.png`
  - `overlay/overlay_metrics.json`
  - `overlay_contact_sheet.png`
- review 산출:
  - `review/review_###.png`
  - `review_contact_sheet.png`
- 문서화:
  - 일반 파일 입력 예시
  - 특정 페이지 비교 예시
  - Codex 보고 규칙
  - `visual_accuracy_proxy_percent` 해석

## 검증 결과

| 항목 | 결과 |
|---|---|
| Python 문법 검사 | 통과 |
| CLI help | 신규 옵션 노출 확인 |
| 실제 smoke | 통과 |
| 페이지 선택 | `requested_pages=[2]`, 분석 페이지 1쪽 확인 |
| 산출물 파일명 | `compare_002.png`, `overlay_002.png`, `review_002.png` 확인 |
| 자동 일치율 보조값 | `visual_accuracy_proxy_percent=13.8381` 확인 |
| whitespace 검사 | `git diff --check` 통과 |

## 주의

`visual_accuracy_proxy_percent`는 사람 판정 정확도가 아니다. 기준 PDF와 rhwp PNG의 내용 픽셀 중심 raster
일치율 보조값이며, 값이 낮으면 `review_###.png`에서 빨강/파랑/주황 차이 영역을 직접 확인해야 한다.
