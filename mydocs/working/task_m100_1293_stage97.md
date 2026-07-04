# task 1293 stage97 - tail/question 후보 클러스터링

## 목적

stage96 full sweep 후 남은 26개 flagged page는 대부분 `render_tree_frame_tail_overflow`와
`question_marker_drift`를 포함한다. stage97에서는 남은 후보를 유형별로 모아 실제 pagination 차이와
검출기 과검출을 나눈다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `bb0a187d task 1293: title overlap sweep 과검출 보정`
- stage96 full sweep:
  - total flagged pages: 26
  - `question_title_text_overlap`: 0
  - stable zero targets: 8개

## 처리 방향

- 남은 pages의 `render_tree_frame_tail_overflow_candidates`, `question_marker_drift_candidates`,
  `equation_text_overlap_candidates`를 표로 모은다.
- 실제 raster frame bleed, PDF frame bleed, red marker drift 수치를 함께 보아 detector false-positive와
  실제 flow mismatch를 분리한다.
- 과검출이면 sweep 조건을 좁히고, 실제 flow mismatch이면 다음 구현 stage의 우선순위를 정한다.

## 검증 계획

필요한 수정 범위에 따라 결정한다. CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 작업지시자 승인

2026-06-14 작업지시자가 자동 승인과 연속 커밋 진행을 지시했다.

## 클러스터링 결과

stage96 full sweep의 남은 flags 빈도:

| flag | count |
| --- | ---: |
| `question_marker_drift` | 23 |
| `large_ink_region_drift` | 22 |
| `render_tree_frame_tail_overflow` | 18 |
| `line_band_drift` | 14 |
| `red_marker_drift` | 13 |
| `column_line_band_drift` | 11 |
| `content_bottom_drift` | 2 |
| `equation_text_overlap` | 1 |

## 대표 패턴

### 1. 실제 question flow drift

`2023-09` p11:

- rhwp: `문12` title이 p11 왼쪽 단 하단에 남음
- PDF: `문12` title이 p11 오른쪽 단 상단에서 시작
- candidate:
  - `rhwp_pi=570`, `rhwp_bbox=[34.0,1073.3,357.2,12.0]`
  - `pdf_bbox=[402.9,90.7,38.1,12.0]`
  - `reasons=["column_drift","y_drift"]`

`dump-pages -p 10` 기준 `문12` title만 단0 하단에 있고, 본문 `pi=571`부터 단1에 있다. 이 유형은
question title orphan을 현재 단에 남기는 실제 flow mismatch로 본다.

### 2. 하단 tail overflow와 question drift 결합

`2022-09` p16, `2024-09-below20` p16:

- 실제 raster bleed는 6px, PDF outside bleed는 0px이지만 question marker drift가 크다.
- `문24`가 rhwp 오른쪽 단 상단, PDF 왼쪽 단 하단에 있어 단 위치가 다르다.
- 이 유형은 단순 glyph bleed suppression 대상이 아니라 앞쪽 flow 차이가 누적된 결과로 본다.

### 3. 큰 betweenNotes + belowLine 조합의 장문 tail

`2024-11-practice-above0-between20-below2` p15:

- `문23`이 rhwp p15 왼쪽 단 상단에 있지만 PDF는 p14 오른쪽 단 하단이다.
- `문29`의 긴 tail이 p15 오른쪽 단 하단을 크게 넘는다.
- 이 유형은 큰 `미주 사이` 조합에서 이전 tail과 다음 title/head group이 함께 어긋나는 실제 flow mismatch로 본다.

### 4. 검출기만으로 정리 가능한 후보는 제한적

- 남은 `render_tree_frame_tail_overflow`의 실제 bleed는 대체로 3~6px로 작다.
- 하지만 대부분 `question_marker_drift`나 `red_marker_drift`가 함께 있어 Stage93처럼 단순 bbox 과검출로
  suppression하면 실제 pagination mismatch를 숨길 위험이 있다.

## 판단

- Stage97에서는 코드 수정 없이 잔여 후보를 실제 flow mismatch 중심으로 재분류했다.
- 다음 구현 stage는 `2023-09` p11의 `문12`처럼 question title만 현재 단 하단에 남고 본문이 다음 단으로
  넘어가는 orphan title 케이스를 우선 본다.
- CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않았다.
