# Task #1129 Stage 5 - 초기 격자 보기 토글 기준 위치 반영

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 4 이후 수동 확인에서 격자 설정 창은 `격자 기준 위치: 쪽`으로 표시되지만, 문서 최초 로드 후 도구막대의 `격자 보기`를 누르면 격자가 `종이` 기준처럼 페이지 전체에 표시되는 문제가 확인됐다.

## 재현 조건

- 문서 최초 로드
- 격자 설정 확인 시:
  - 격자 기준 위치: `쪽`
  - 가로/세로 오프셋: `0mm, 0mm`
- 도구막대 `격자 보기` 클릭

## 기대 동작

- 설정 창의 기준 위치가 `쪽`이면 격자 오버레이도 즉시 `쪽` 기준 영역과 원점을 사용해야 한다.
- 설정 UI 상태와 실제 오버레이 렌더 상태가 달라지면 안 된다.

## 조사 방향

- `view:toggle-grid` 명령이 현재 `GridViewSettings.origin`을 그대로 전달하는지 확인한다.
- 페이지 최초 로드 시 `CanvasView`의 pageInfo가 최신 WASM `pageBorder*` 필드를 갖는지 확인한다.
- `pageBorder*` 필드가 없거나 0으로 들어오는 문서에서 `쪽` 기준 fallback이 종이 기준처럼 동작하는지 확인한다.
- 로컬 Playwright 기능 검증은 시각 판단 대신 `clip-path`, `background-position`, `PageInfo.pageBorder*` 값으로 수행한다.

## 확인 결과

`samples/KTX.hwp` 최초 로드 후 도구막대 `격자 보기`를 바로 클릭하는 경로를 자동 검증했다.

```text
dialog.origin.page = true
dialog.origin.paper = false
dialog.offset = 0mm, 0mm
PageInfo.marginLeft = 75.6px
PageInfo.pageBorderLeft = 18.9px
overlay.background-position = 20.955px 20.955px
overlay.clip-path = inset(20.955px)
```

초기 토글 경로에서도 UI 설정은 `쪽`이고, 실제 overlay도 `clip-path: inset(...)`으로 쪽 기준을 사용한다. 이전 화면과 다르게 보이는 경우에는 브라우저가 이전 WASM/JS 산출물을 캐시했거나 페이지를 새로고침하지 않은 상태일 가능성이 있다.

## 자동 검증

- 로컬 Playwright 기능 검증 통과
  - 최초 로드
  - 도구막대 `격자 보기` 클릭
  - `격자 설정` 창 상태 확인
  - overlay `clip-path`, `background-position`, `PageInfo.pageBorder*` 확인

## 대기

원인 확인 후 코드 수정, 자동 검증, 커밋을 마치고 작업지시자의 시각 판단을 기다린다.
