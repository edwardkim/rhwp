# Task #1129 Stage 1 - 격자 보기 오버레이 구현

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 작업 내용

- `rhwp-studio` 보기 메뉴와 도구막대의 `격자 보기`를 실제 명령으로 연결했다.
- 페이지 캔버스 위에 비인쇄 DOM 오버레이를 추가해 한컴오피스식 점/선 격자를 표시할 수 있게 했다.
- 격자 오버레이는 페이지별 캔버스 위치, 크기, zoom 값을 따라가며, 스크롤/재렌더링/페이지 해제 시 함께 갱신 또는 제거된다.

## 변경 파일

- `rhwp-studio/index.html`
- `rhwp-studio/src/command/commands/view.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/view/grid-overlay.ts`
- `rhwp-studio/src/view/grid-settings.ts`
- `rhwp-studio/src/styles/editor.css`

## 구현 메모

- 격자는 문서 데이터에 저장하지 않는 보기 전용 상태로 두었다.
- `behindText`/`inFrontOfText`는 현재 캔버스 렌더러 구조상 별도 DOM z-index/opacity 정책으로 표현한다.
- 실제 출력/저장 결과에는 영향을 주지 않는다.
