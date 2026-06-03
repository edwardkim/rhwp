# Task #1129 Stage 8 - 한컴 기준 점 격자 시각 재보정

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 7에서 점 격자 원점을 반 칸 안쪽으로 보정했지만, 수동 비교에서 여전히 한컴오피스 기준 화면과 다르다.

사용자 비교:

- 한컴오피스 기준 화면
- rhwp-studio 화면
- 한컴 확대 영역

차이:

- 점 격자의 표시 시작/끝 위치가 한컴과 다르게 보인다.
- 점의 농도/표시 방식도 한컴 기준과 다르게 보인다.
- 쪽 경계 주변의 격자 표시가 한컴의 쪽 표시와 맞지 않는다.

## 판단

Stage 7의 반 칸 보정은 추측성 보정이었고 한컴 시각 기준을 만족하지 못했다.

이번 스테이지에서는 다음을 함께 보정한다.

- 점 격자 원점
- 점 격자 색/불투명도/크기
- 쪽 기준 clip 범위

## 수정 방향

- 점 격자 반 칸 원점 보정은 제거한다.
- 한컴처럼 점은 더 작고 진하게 보이도록 조정한다.
- `쪽` 기준 clip 범위는 쪽 테두리 선과 격자가 겹쳐 보이지 않도록 쪽 테두리 영역보다 안쪽으로 1px 보정한다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - Stage 7의 점 격자 반 칸 원점 보정을 제거했다.
  - 점 격자 색상을 더 진하게 하고 점 반지름을 줄였다.
  - 격자 overlay 불투명도를 높였다.
  - `쪽` 기준 clip 범위를 쪽 테두리보다 1px 안쪽으로 보정했다.

## 검증 계획

- `samples/hwp3-sample16-hwp5.hwp` 로드
- 로드 시 HWPX 비표준 감지 모달에서 `그대로 보기`
- `쪽/10mm/0,0` 설정
- overlay `clip-path`, `background-position`, `background-size`, `background-image`, `opacity` 기록
- `npm run build`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/10mm/0,0`
  - overlay 수: `2`
  - `background-image`: `radial-gradient(circle, rgba(0, 38, 160, 0.85) 0px, rgba(0, 38, 160, 0.85) 0.55px, rgba(0, 0, 0, 0) 0.75px)`
  - `background-size`: `41.9048px 41.9048px`
  - `background-position`: `20.955px 20.955px`
  - `clip-path`: `inset(21.955px)`
  - `opacity`: `1`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
