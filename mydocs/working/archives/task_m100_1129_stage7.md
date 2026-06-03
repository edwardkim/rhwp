# Task #1129 Stage 7 - 한컴 쪽 격자 눈금 및 쪽 표시 재정합

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 6에서 `쪽` 기준을 편집 용지 여백 기준으로 옮겼지만, 수동 비교에서 한컴오피스와 rhwp-studio의 격자 점 위치가 여전히 다르다.

추가로 한컴 화면의 쪽 경계/모서리 표시와 rhwp-studio의 표시도 다르게 보인다.

## 비교 조건

- 격자 보기: 켬
- 격자 모양: 점
- 격자 간격: 가로 10mm, 세로 10mm
- 격자 기준 위치: 쪽
- 기준 오프셋: 가로 0mm, 세로 0mm

## 문제 판단

이전 보정 후보:

- Stage 4: `PageBorderFill` 기준
- Stage 6: 편집 용지 여백 기준

둘 다 수동 비교에서 한컴 위치와 맞지 않았다.

한컴 화면상 `쪽` 기준 격자는 쪽 테두리 안쪽에서 시작하되, 단순 page border spacing 또는 본문 margin 중 하나를 그대로 쓰는 모델은 아닌 것으로 보인다. 따라서 격자 표시 영역과 격자 패턴 원점을 분리해야 한다.

## 수정 방향

- `쪽` 기준의 표시 영역은 쪽 테두리/쪽 표시 영역에 맞춘다.
- `쪽` 기준의 점 패턴 원점은 표시 영역 시작점에서 격자 간격만큼 바로 찍지 않고, 한컴처럼 안쪽 보정값을 적용한다.
- 쪽 경계/모서리 표시와 격자 overlay가 서로 어긋나 보이지 않도록 overlay clip과 page 표시 좌표의 기준을 일치시킨다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - `쪽/10mm/0,0` 설정
  - overlay `clip-path`, `background-position`, `background-size` 기록
- `npm run build`
- `cargo fmt --all -- --check && git diff --check`

## 구현 결과

- `쪽` 기준의 표시 영역을 다시 `PageBorderFill` 쪽 테두리 영역으로 맞췄다.
- 점 격자(`dots`)의 실제 점 위치는 기준점 바로 위가 아니라 격자 간격의 절반만큼 안쪽으로 보정했다.
- 선 격자(`horizontal`, `vertical`, `both`)는 기준선 자체가 보여야 하므로 반 칸 보정을 적용하지 않는다.

## 자동 검증

- `npm run build` 통과
- `cargo fmt --all -- --check && git diff --check` 통과
- 로컬 Playwright 기능 검증 통과
  - `samples/hwp3-sample16-hwp5.hwp`
  - 로드 시 HWPX 비표준 감지 모달에서 `그대로 보기` 선택
  - `쪽/10mm/0,0`
  - `PageInfo.marginLeft = 56.7px`, `PageInfo.marginTop = 37.8px`
  - `PageInfo.pageBorderLeft = 18.9px`
  - overlay `clip-path = inset(20.955px)`
  - overlay `background-size = 41.9048px 41.9048px`
  - overlay `background-position = 41.9074px 41.9074px`

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
