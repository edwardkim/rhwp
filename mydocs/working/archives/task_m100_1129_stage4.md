# Task #1129 Stage 4 - 한컴 격자 눈금 원점 재정합

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 1~3 구현 후 수동 비교에서 한컴오피스의 `격자 기준 위치: 쪽`과 rhwp-studio의 격자 눈금 원점이 다르게 보였다.

비교 조건:

- 격자 보기: 켬
- 격자 모양: 점
- 격자 간격: 가로 10mm, 세로 10mm
- 격자 기준 위치: 쪽
- 기준 오프셋: 가로 0mm, 세로 0mm

## 문제 판단

이전 구현은 `쪽` 기준을 본문 여백 기준으로 해석해 `marginLeft`, `marginTop + marginHeader`를 원점과 clip 범위로 사용했다.

한컴 비교 화면에서는 `쪽` 기준이 본문 여백보다 바깥의 쪽 테두리/쪽 영역 기준에 가까워 보인다. 그래서 rhwp-studio 격자가 오른쪽/아래로 밀려 보였다.

## 수정 방향

- `PageInfo`에 쪽 테두리/쪽 영역 좌표를 추가한다.
- HWP5/HWPX의 `PageBorderFill` 기준과 spacing을 사용해 쪽 영역을 계산한다.
- 격자 기준 위치가 `쪽`이면 본문 여백이 아니라 쪽 테두리/쪽 영역 좌표를 원점과 clip 범위로 사용한다.
- 격자 기준 위치가 `종이`이면 기존처럼 페이지 전체를 표시 범위로 둔다.

## 현재 진행

- `src/document_core/queries/rendering.rs`에 `pageBorderLeft/Right/Top/Bottom` JSON 필드 추가
- `rhwp-studio/src/core/types.ts`에 선택 필드 추가
- `rhwp-studio/src/view/grid-overlay.ts`에서 `쪽` 기준 계산을 page border 영역 기반으로 변경
- WASM 산출물 갱신: `wasm-pack build --target web --release`

## 자동 검증

- `wasm-pack build --target web --release` 통과
- `npm run build` 통과
- `cargo test test_parse_section_with_section_def --lib -- --nocapture` 통과
- `cargo fmt --all -- --check && git diff --check` 통과
- 로컬 Playwright 기능 검증 통과
  - `samples/hwp3-sample19-hwp5.hwp`
  - `PageInfo.marginLeft`: `132.3px`
  - `PageInfo.pageBorderLeft`: `18.9px`
  - `쪽/10mm/0,0` overlay `background-position`: `20.955px 20.955px`
  - `쪽/10mm/0,0` overlay `clip-path`: `inset(20.955px)`

## 대기

자동 검증 후 커밋하고, 최종 시각 판단은 작업지시자 확인을 기다린다.
