# Task 391: 셀 세로 가운데 정렬 높이 계산 오류 수정

## 현상

- **파일**: samples/gonggo-01.hwp 3페이지
- **문단**: s0:pi=81 ci=0 표, 셀[0] (14개 문단, va=Center)
- **증상**: 세로 가운데 정렬 시 `total_content_height`가 실제보다 작게 계산되어
  `mechanical_offset`이 과대 → 콘텐츠가 아래로 밀려 셀 하단 잘림
- **정상**: 콘텐츠 높이 ≥ 셀 높이이면 `offset=0` → 상단 시작 → 전부 표시

## 원인 분석

`calc_composed_paras_content_height`에서:
- `composed_paras`의 `line.line_height`를 합산
- 중첩 표의 높이가 LINE_SEG lh에 포함되지만, composer의 `line_height`와 다를 수 있음
- 14개 문단 + 중첩 표가 있는 복잡한 셀에서 차이 누적

## 수정 방향

`total_content_height`가 `inner_height`보다 크면 `mechanical_offset=0`이므로 문제 없음.
핵심은 `total_content_height`가 과소 계산되는 원인을 찾아 수정.

## 구현 계획 (3단계)

### 1단계: total_content_height 디버그 — 실제 렌더링 높이 vs 계산 높이 비교
### 2단계: 과소 원인 수정 (LINE_SEG lh 기반 fallback 또는 composed 높이 보정)
### 3단계: 검증 + 커밋
