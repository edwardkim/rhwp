# Task 398: LINE_SEG 일치율 측정 인프라 구축

## 수행 목표

HWP 파일의 원본 LINE_SEG와 rhwp `reflow_line_segs()` 결과를 자동 비교하는 테스트 인프라를 구축한다. 이후 Task 399~401의 역공학 작업에서 개선 효과를 정량적으로 측정하는 기반이 된다.

## 배경

v1.0.0의 핵심 전략은 한컴 동일 조판 구현(전략 C)이다. 이를 위해 rhwp의 reflow 결과가 한컴 원본과 얼마나 일치하는지 측정할 수 있어야 한다. 현재는 이를 정량적으로 확인할 방법이 없다.

## 구현 계획

### 1단계: LINE_SEG 비교 함수 구현

- 원본 LINE_SEG(HWP 파일 파싱 결과)와 reflow 결과를 필드별로 비교하는 함수 작성
- 비교 대상 필드: `text_start`, `segment_width`, `line_height`, `text_height`, `baseline_distance`, `line_spacing`, `vertical_pos`
- 필드별 일치/불일치 + 차이값(delta)을 구조화된 결과로 반환

### 2단계: 샘플 HWP 대상 일치율 측정 테스트

- `samples/` 폴더의 HWP 파일들에 대해 일괄 측정
- 각 문단별: 원본 LINE_SEG 줄 수 vs reflow 줄 수, 필드별 일치율
- 전체 요약: 문단 수, 줄 수 일치율, 필드별 평균 오차

### 3단계: CLI 서브커맨드 또는 테스트 출력

- `cargo test` 또는 CLI 서브커맨드로 일치율 리포트를 출력
- 불일치 패턴(줄바꿈 위치, 폭, baseline 등)을 분류하여 Task 399~401의 우선순위 판단 근거 제공

## 산출물

- LINE_SEG 비교 함수 (src/ 내 모듈)
- 일치율 측정 테스트
- 베이스라인 일치율 리포트 (`mydocs/working/task_398_baseline.md`)
- 단계별 완료 보고서

## 비고

- 코드 변경 범위: 비교/측정 로직 추가만. 기존 reflow 로직 수정 없음
- Task 399~401에서 reflow 개선 시 이 인프라로 효과 측정
