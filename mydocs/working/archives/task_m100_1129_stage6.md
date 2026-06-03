# Task #1129 Stage 6 - 격자 쪽 기준 눈금 원점 재보정

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 4에서 `쪽` 기준을 `PageBorderFill` 테두리 영역으로 옮겼지만, 수동 비교에서 최초 로드 후 `쪽/0,0` 격자가 여전히 종이 기준처럼 너무 바깥쪽에 표시되는 것으로 확인됐다.

Stage 5는 코드 수정 없이 자동 검증 기록만 커밋했으나, 시각 판단 결과와 맞지 않으므로 추가 코드 보정이 필요하다.

## 문제 판단

한컴오피스의 `격자 기준 위치: 쪽`은 종이 전체나 쪽 테두리 선 기준보다 안쪽의 쪽 여백 기준에 더 가깝다.

기존 rhwp-studio 구현:

- `종이`: 페이지 전체
- `쪽`: `PageBorderFill` 기반 테두리 영역

보정 방향:

- `종이`: 페이지 전체 유지
- `쪽`: 편집 용지의 쪽 여백 영역(`marginLeft`, `marginTop`, `marginRight`, `marginBottom`) 기준으로 변경
- 머리말/꼬리말 여백은 격자 보기 영역에서 제외하지 않는다. 한컴 화면에서 격자가 상단 쪽 영역에도 보이기 때문이다.

## 검증 계획

- `npm run build`
- `cargo fmt --all -- --check && git diff --check`
- 로컬 Playwright 기능 검증
  - 최초 로드 후 `격자 보기` 클릭
  - `쪽` 설정 유지 확인
  - overlay `clip-path`가 `marginLeft/marginTop` 기반 inset인지 확인

## 구현 결과

- `rhwp-studio/src/view/grid-overlay.ts`
  - `쪽` 기준 `getPageGridAreaPx()`를 `PageBorderFill` 기반에서 `PageInfo.marginLeft/marginTop/marginRight/marginBottom` 기반으로 변경
  - `종이` 기준은 기존처럼 `clip-path: none` 유지

## 자동 검증

- `npm run build` 통과
- `cargo fmt --all -- --check && git diff --check` 통과
- 로컬 Playwright 기능 검증 통과
  - `samples/KTX.hwp`
  - 최초 로드 후 도구막대 `격자 보기` 클릭
  - 설정 창 `쪽` 선택 유지, 오프셋 `0mm, 0mm`
  - `PageInfo.marginLeft = 75.6px`, `PageInfo.marginTop = 56.7px`
  - overlay `background-position = 83.8201px 62.8651px`
  - overlay `clip-path = inset(62.8651px 83.8201px)`

## 시각 판단 대기

자동 검증 기준으로는 `쪽` 기준이 종이 가장자리/PageBorder 기준보다 안쪽의 쪽 여백 기준으로 이동했다. 최종 한컴 정합 여부는 작업지시자의 수동 비교를 기다린다.
